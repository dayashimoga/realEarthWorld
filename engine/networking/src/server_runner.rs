//! Standalone Headless Authoritative Multiplayer Server Runner.
//!
//! Manages UDP socket transport, client session lifecycles, authoritative physics ticks,
//! vehicle spawning, anti-cheat validation, and Area of Interest (AoI) snapshot broadcasting.

use crate::protocol::{
    ChannelType, ClientInputFrame, EntityStatePayload, NetworkMessage, PlayerRole, PROTOCOL_VERSION,
};
use crate::server::AuthoritativeServer;
use crate::session::ConnectionState;
use crate::transport::UdpSocketTransport;
use glam::{Quat, Vec3};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Summary of activity performed in a single server tick.
#[derive(Debug, Clone, Default)]
pub struct ServerStepSummary {
    pub tick: u64,
    pub packets_received: usize,
    pub packets_sent: usize,
    pub active_players: usize,
    pub total_entities: usize,
    pub violations_detected: usize,
}

/// Standalone Headless Authoritative Server wrapping simulation state and real UDP transport.
pub struct HeadlessAuthoritativeServer {
    pub server: AuthoritativeServer,
    pub transport: UdpSocketTransport,
    /// Maps client SocketAddr to session_id.
    pub addr_to_session: HashMap<SocketAddr, String>,
    /// Maps session_id to client SocketAddr.
    pub session_to_addr: HashMap<String, SocketAddr>,
    /// Pre-allocated spawn points for connecting players.
    pub spawn_slots: Vec<(Vec3, Quat)>,
    pub next_spawn_idx: usize,
    /// Cumulative telemetry counters.
    pub total_packets_rx: u64,
    pub total_packets_tx: u64,
    pub total_bytes_rx: u64,
    pub total_bytes_tx: u64,
    pub dropped_packets: u64,
    pub last_snapshot_time: Instant,
    pub snapshot_interval: Duration,
    pub running: Arc<AtomicBool>,
}

impl HeadlessAuthoritativeServer {
    /// Binds a new headless authoritative server to the specified UDP address (e.g. "0.0.0.0:7777" or "127.0.0.1:0").
    pub fn bind(
        bind_addr: &str,
        tick_rate_hz: u32,
        max_players: usize,
        cell_size_meters: f32,
    ) -> Result<Self, std::io::Error> {
        let transport = UdpSocketTransport::bind(bind_addr)?;
        let server = AuthoritativeServer::new(tick_rate_hz, max_players, cell_size_meters);

        // Standard street-level spawn slots spaced 15m apart
        let mut spawn_slots = Vec::with_capacity(32);
        for i in 0..32 {
            let offset_x = (i % 8) as f32 * 12.0;
            let offset_z = (i / 8) as f32 * 20.0;
            spawn_slots.push((Vec3::new(offset_x, 0.5, offset_z), Quat::IDENTITY));
        }

        let snapshot_interval = Duration::from_secs_f32(1.0 / (tick_rate_hz as f32).min(30.0));

        Ok(Self {
            server,
            transport,
            addr_to_session: HashMap::new(),
            session_to_addr: HashMap::new(),
            spawn_slots,
            next_spawn_idx: 0,
            total_packets_rx: 0,
            total_packets_tx: 0,
            total_bytes_rx: 0,
            total_bytes_tx: 0,
            dropped_packets: 0,
            last_snapshot_time: Instant::now(),
            snapshot_interval,
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Returns the local bound UDP socket address.
    pub fn local_addr(&self) -> SocketAddr {
        self.transport.local_addr()
    }

    /// Advances the authoritative server by processing all incoming network packets,
    /// ticking simulation state, and broadcasting AoI snapshots.
    pub fn step(&mut self, _dt: f32) -> ServerStepSummary {
        let mut summary = ServerStepSummary {
            tick: self.server.server_tick,
            ..Default::default()
        };

        // 1. Drain and handle incoming network packets
        while let Ok(Some((_header, msg, src_addr))) = self.transport.recv_message() {
            self.total_packets_rx += 1;
            summary.packets_received += 1;

            match msg {
                NetworkMessage::ClientHello {
                    client_version,
                    client_name,
                    auth_token,
                } => {
                    self.handle_client_hello(src_addr, client_version, &client_name, &auth_token);
                }
                NetworkMessage::Input(frame) => {
                    self.handle_client_input(src_addr, frame);
                }
                NetworkMessage::Ping {
                    client_timestamp_ms,
                } => {
                    self.handle_ping(src_addr, client_timestamp_ms);
                }
                NetworkMessage::Disconnect { reason } => {
                    self.handle_disconnect(src_addr, &reason);
                }
                _ => {
                    // Stale or unhandled message variant dropped safely
                }
            }
        }

        // 2. Advance authoritative server tick and poll timeouts
        self.server.step_tick();
        summary.tick = self.server.server_tick;

        // Clean up timed-out players and broadcast despawns
        let timed_out = self.server.sessions.poll_timeouts();
        for player_id in timed_out {
            let entity_ids: Vec<u32> = self
                .server
                .entities
                .values()
                .filter(|e| e.owner_player_id == player_id)
                .map(|e| e.entity_id)
                .collect();

            for eid in entity_ids {
                self.server.entities.remove(&eid);
                self.broadcast_message(
                    ChannelType::ReliableOrdered,
                    &NetworkMessage::DespawnEntity {
                        entity_id: eid,
                        reason: "Session timed out".to_string(),
                    },
                    None,
                );
            }
        }

        // 3. Emit Area of Interest snapshots at target rate (e.g. 30–60 Hz)
        if self.last_snapshot_time.elapsed() >= self.snapshot_interval {
            self.last_snapshot_time = Instant::now();
            self.broadcast_aoi_snapshots(&mut summary);
        }

        summary.active_players = self.session_to_addr.len();
        summary.total_entities = self.server.entities.len();
        summary.violations_detected = self.server.security_violations.len();

        summary
    }

    /// Handles a new client handshake request.
    fn handle_client_hello(
        &mut self,
        src_addr: SocketAddr,
        client_version: u16,
        client_name: &str,
        auth_token: &str,
    ) {
        // 1. Version verification
        if client_version != PROTOCOL_VERSION {
            let reject = NetworkMessage::HandshakeReject {
                reason: crate::protocol::RejectReason::VersionMismatch(PROTOCOL_VERSION),
            };
            let _ = self
                .transport
                .send_message(src_addr, ChannelType::ReliableOrdered, &reject);
            return;
        }

        // 2. Register or re-authenticate session
        let (session_id, player_id) = if let Some(existing_session) = self
            .server
            .sessions
            .sessions
            .values_mut()
            .find(|s| s.auth_token == auth_token && !auth_token.is_empty())
        {
            existing_session.state = ConnectionState::Connected;
            existing_session.last_heartbeat = Instant::now();
            (
                existing_session.session_id.clone(),
                existing_session.player_id,
            )
        } else {
            match self
                .server
                .sessions
                .register_player(client_name, PlayerRole::Player)
            {
                Ok(new_session) => (new_session.session_id, new_session.player_id),
                Err(_) => {
                    let reject = NetworkMessage::HandshakeReject {
                        reason: crate::protocol::RejectReason::ServerFull,
                    };
                    let _ = self.transport.send_message(
                        src_addr,
                        ChannelType::ReliableOrdered,
                        &reject,
                    );
                    return;
                }
            }
        };

        self.addr_to_session.insert(src_addr, session_id.clone());
        self.session_to_addr.insert(session_id.clone(), src_addr);

        // 3. Send ServerHello
        let hello_reply = NetworkMessage::ServerHello {
            session_id: session_id.clone(),
            assigned_player_id: player_id,
            server_tick_rate: self.server.tick_rate_hz,
            role: PlayerRole::Player,
        };
        let _ = self
            .transport
            .send_message(src_addr, ChannelType::ReliableOrdered, &hello_reply);
        self.total_packets_tx += 1;

        // 4. Spawn authoritative vehicle for player if not already assigned
        let (spawn_pos, spawn_orient) =
            self.spawn_slots[self.next_spawn_idx % self.spawn_slots.len()];
        self.next_spawn_idx += 1;

        if let Ok(new_entity_id) =
            self.server
                .spawn_vehicle(player_id, "SportsCoupe", spawn_pos, spawn_orient)
        {
            let entity = self.server.entities.get(&new_entity_id).unwrap();
            let new_spawn_msg = NetworkMessage::SpawnEntity {
                entity_id: new_entity_id,
                owner_player_id: player_id,
                vehicle_type: "SportsCoupe".to_string(),
                state: EntityStatePayload {
                    entity_id: new_entity_id,
                    owner_player_id: player_id,
                    position: entity.position,
                    orientation: entity.orientation,
                    linear_velocity: entity.linear_velocity,
                    angular_velocity: entity.angular_velocity,
                    speed_kph: entity.speed_kph,
                    flags: 0,
                },
            };

            // Send all existing world entities to the newly joined player
            for (eid, ent) in &self.server.entities {
                if *eid != new_entity_id {
                    let existing_spawn = NetworkMessage::SpawnEntity {
                        entity_id: *eid,
                        owner_player_id: ent.owner_player_id,
                        vehicle_type: ent.vehicle_type.clone(),
                        state: EntityStatePayload {
                            entity_id: *eid,
                            owner_player_id: ent.owner_player_id,
                            position: ent.position,
                            orientation: ent.orientation,
                            linear_velocity: ent.linear_velocity,
                            angular_velocity: ent.angular_velocity,
                            speed_kph: ent.speed_kph,
                            flags: 0,
                        },
                    };
                    let _ = self.transport.send_message(
                        src_addr,
                        ChannelType::ReliableOrdered,
                        &existing_spawn,
                    );
                    self.total_packets_tx += 1;
                }
            }

            // Broadcast newly spawned vehicle to all connected players (including self)
            self.broadcast_message(ChannelType::ReliableOrdered, &new_spawn_msg, None);
        }
    }

    /// Handles a high-frequency vehicle input frame from a client.
    fn handle_client_input(&mut self, src_addr: SocketAddr, frame: ClientInputFrame) {
        if let Some(session_id) = self.addr_to_session.get(&src_addr).cloned() {
            if let Some(session) = self.server.sessions.sessions.get(&session_id) {
                if let Some(entity_id) = session.controlled_entity_id {
                    let _ = self
                        .server
                        .process_client_input(&session_id, entity_id, &frame);
                }
            }
        }
    }

    /// Handles a heartbeat ping from a client.
    fn handle_ping(&mut self, src_addr: SocketAddr, client_timestamp_ms: u64) {
        let pong = NetworkMessage::Pong {
            client_timestamp_ms,
            server_timestamp_ms: self.server.server_tick * (1000 / self.server.tick_rate_hz as u64),
        };
        let _ = self
            .transport
            .send_message(src_addr, ChannelType::UnreliableSequenced, &pong);
        self.total_packets_tx += 1;

        if let Some(session_id) = self.addr_to_session.get(&src_addr) {
            if let Some(session) = self.server.sessions.sessions.get_mut(session_id) {
                session.last_heartbeat = Instant::now();
            }
        }
    }

    /// Handles a client disconnect notification.
    fn handle_disconnect(&mut self, src_addr: SocketAddr, reason: &str) {
        if let Some(session_id) = self.addr_to_session.remove(&src_addr) {
            self.session_to_addr.remove(&session_id);

            if let Some(session) = self.server.sessions.sessions.remove(&session_id) {
                if let Some(entity_id) = session.controlled_entity_id {
                    self.server.entities.remove(&entity_id);
                    self.broadcast_message(
                        ChannelType::ReliableOrdered,
                        &NetworkMessage::DespawnEntity {
                            entity_id,
                            reason: format!("Player disconnected: {}", reason),
                        },
                        None,
                    );
                }
            }
        }
    }

    /// Broadcasts a network message to all connected clients, optionally excluding one.
    pub fn broadcast_message(
        &mut self,
        channel: ChannelType,
        msg: &NetworkMessage,
        exclude_addr: Option<SocketAddr>,
    ) {
        for &addr in self.session_to_addr.values() {
            if Some(addr) != exclude_addr {
                let _ = self.transport.send_message(addr, channel, msg);
                self.total_packets_tx += 1;
            }
        }
    }

    /// Broadcasts Area of Interest snapshots tailored to each client's spatial position.
    fn broadcast_aoi_snapshots(&mut self, summary: &mut ServerStepSummary) {
        let session_ids: Vec<String> = self.session_to_addr.keys().cloned().collect();

        for session_id in session_ids {
            if let Some(session) = self.server.sessions.sessions.get(&session_id) {
                let player_id = session.player_id;
                if let Some(&dest_addr) = self.session_to_addr.get(&session_id) {
                    if let Some(snapshot) = self.server.build_snapshot_for_player(player_id, 1000.0)
                    {
                        let _ = self.transport.send_message(
                            dest_addr,
                            ChannelType::UnreliableSequenced,
                            &snapshot,
                        );
                        self.total_packets_tx += 1;
                        summary.packets_sent += 1;
                    }
                }
            }
        }
    }

    /// Starts the server loop on a dedicated background thread.
    pub fn spawn_in_background(
        bind_addr: &str,
        tick_rate_hz: u32,
        max_players: usize,
    ) -> Result<(SocketAddr, Arc<AtomicBool>, JoinHandle<()>), std::io::Error> {
        let mut server = Self::bind(bind_addr, tick_rate_hz, max_players, 1000.0)?;
        let bound_addr = server.local_addr();
        let running = server.running.clone();
        let running_thread = running.clone();

        let handle = std::thread::spawn(move || {
            let tick_dur = Duration::from_secs_f32(1.0 / tick_rate_hz as f32);
            while running_thread.load(Ordering::Relaxed) {
                let start = Instant::now();
                server.step(tick_dur.as_secs_f32());
                let elapsed = start.elapsed();
                if elapsed < tick_dur {
                    std::thread::sleep(tick_dur - elapsed);
                }
            }
        });

        Ok((bound_addr, running, handle))
    }
}
