//! Real-Time Authoritative Multiplayer Client Runtime.
//!
//! Handles non-blocking UDP communication with the authoritative server,
//! client-side prediction, input buffer reconciliation, and remote vehicle interpolation.

use crate::protocol::{
    ChannelType, ClientInputFrame, NetworkError, NetworkMessage, PROTOCOL_VERSION,
};
use crate::replication::{ClientPredictor, RemoteEntityInterpolator};
use crate::session::ConnectionState;
use crate::transport::UdpSocketTransport;
use glam::{Quat, Vec3};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;

/// Active render state for a remote vehicle entity interpolated across server snapshots.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteVehicleRenderState {
    pub entity_id: u32,
    pub owner_player_id: u32,
    pub vehicle_type: String,
    pub position: Vec3,
    pub orientation: Quat,
    pub speed_kph: f32,
    pub color: [f32; 4],
}

/// Real-time connection and telemetry status summary.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiplayerStatus {
    pub state: ConnectionState,
    pub ping_ms: f32,
    pub player_id: Option<u32>,
    pub controlled_entity_id: Option<u32>,
    pub remote_player_count: usize,
    pub packets_rx: u64,
    pub packets_tx: u64,
    pub bytes_rx: u64,
    pub bytes_tx: u64,
    pub server_tick: u64,
    pub server_addr: SocketAddr,
}

/// Internal tracked remote entity.
struct TrackedRemoteEntity {
    owner_player_id: u32,
    vehicle_type: String,
    interpolator: RemoteEntityInterpolator,
    last_speed_kph: f32,
    color: [f32; 4],
}

/// Production multiplayer client runtime for Real Earth.
pub struct MultiplayerClient {
    pub player_name: String,
    pub server_addr: SocketAddr,
    pub state: ConnectionState,
    pub transport: UdpSocketTransport,
    pub session_id: Option<String>,
    pub player_id: Option<u32>,
    pub controlled_entity_id: Option<u32>,
    pub predictor: ClientPredictor,
    tracked_remotes: HashMap<u32, TrackedRemoteEntity>,
    pub ping_ms: f32,
    pub last_ping_sent: Instant,
    pub last_packet_received: Instant,
    pub packets_rx: u64,
    pub packets_tx: u64,
    pub bytes_rx: u64,
    pub bytes_tx: u64,
    pub server_tick: u64,
    pub reconciliation_tolerance: f32,
}

impl MultiplayerClient {
    /// Initializes a new multiplayer client targeting the specified authoritative server.
    pub fn new(player_name: &str, server_addr: SocketAddr) -> Result<Self, NetworkError> {
        // Bind to ephemeral UDP socket
        let transport = UdpSocketTransport::bind("0.0.0.0:0")
            .map_err(|e| NetworkError::Transport(e.to_string()))?;

        Ok(Self {
            player_name: player_name.to_string(),
            server_addr,
            state: ConnectionState::Disconnected,
            transport,
            session_id: None,
            player_id: None,
            controlled_entity_id: None,
            predictor: ClientPredictor::default(),
            tracked_remotes: HashMap::new(),
            ping_ms: 20.0,
            last_ping_sent: Instant::now(),
            last_packet_received: Instant::now(),
            packets_rx: 0,
            packets_tx: 0,
            bytes_rx: 0,
            bytes_tx: 0,
            server_tick: 0,
            reconciliation_tolerance: 0.25, // 25cm divergence threshold
        })
    }

    /// Initiates connection handshake with the authoritative server.
    pub fn connect(&mut self) -> Result<(), NetworkError> {
        let hello = NetworkMessage::ClientHello {
            client_version: PROTOCOL_VERSION,
            client_name: self.player_name.clone(),
            auth_token: String::new(),
        };

        self.transport
            .send_message(self.server_addr, ChannelType::ReliableOrdered, &hello)?;
        self.packets_tx += 1;
        self.state = ConnectionState::Connecting;
        self.last_ping_sent = Instant::now();
        self.last_packet_received = Instant::now();

        Ok(())
    }

    /// Polls incoming network packets, processes server snapshots, reconciles prediction,
    /// and maintains connection keepalives.
    pub fn poll_network(&mut self, current_time_ms: u64) -> Result<(), NetworkError> {
        // 1. Drain incoming UDP socket packets
        while let Ok(Some((_hdr, msg, src_addr))) = self.transport.recv_message() {
            if src_addr != self.server_addr {
                continue; // Ignore unsolicited packets
            }

            self.packets_rx += 1;
            self.last_packet_received = Instant::now();

            match msg {
                NetworkMessage::ServerHello {
                    session_id,
                    assigned_player_id,
                    ..
                } => {
                    self.session_id = Some(session_id);
                    self.player_id = Some(assigned_player_id);
                    self.state = ConnectionState::Connected;
                }
                NetworkMessage::HandshakeReject { reason } => {
                    self.state = ConnectionState::Disconnected;
                    return Err(NetworkError::SessionError(format!(
                        "Server rejected handshake: {:?}",
                        reason
                    )));
                }
                NetworkMessage::SpawnEntity {
                    entity_id,
                    owner_player_id,
                    vehicle_type,
                    state,
                } => {
                    if Some(owner_player_id) == self.player_id {
                        // Local controlled vehicle
                        self.controlled_entity_id = Some(entity_id);
                        self.predictor.position = state.position;
                        self.predictor.orientation = state.orientation;
                        self.predictor.linear_velocity = state.linear_velocity;
                    } else {
                        // Remote vehicle
                        let mut interpolator = RemoteEntityInterpolator::new(entity_id, 80);
                        interpolator.push_snapshot(current_time_ms, state.clone());

                        // Generate distinct color per owner player ID
                        let color = match owner_player_id % 5 {
                            0 => [0.15, 0.45, 0.95, 1.0], // Azure Blue
                            1 => [0.95, 0.55, 0.10, 1.0], // Amber Orange
                            2 => [0.10, 0.85, 0.45, 1.0], // Emerald Green
                            3 => [0.85, 0.15, 0.85, 1.0], // Violet
                            _ => [0.95, 0.15, 0.25, 1.0], // Crimson
                        };

                        self.tracked_remotes.insert(
                            entity_id,
                            TrackedRemoteEntity {
                                owner_player_id,
                                vehicle_type,
                                interpolator,
                                last_speed_kph: state.speed_kph,
                                color,
                            },
                        );
                    }
                }
                NetworkMessage::DespawnEntity { entity_id, .. } => {
                    self.tracked_remotes.remove(&entity_id);
                    if self.controlled_entity_id == Some(entity_id) {
                        self.controlled_entity_id = None;
                    }
                }
                NetworkMessage::ServerSnapshot {
                    tick,
                    server_time_ms,
                    states,
                } => {
                    self.server_tick = tick;
                    for st in states {
                        if Some(st.entity_id) == self.controlled_entity_id {
                            // Reconcile client prediction against authoritative server snapshot
                            self.predictor.reconcile_with_server_snapshot(
                                tick,
                                st.position,
                                st.orientation,
                                st.linear_velocity,
                                self.reconciliation_tolerance,
                            );
                        } else if let Some(remote) = self.tracked_remotes.get_mut(&st.entity_id) {
                            remote.last_speed_kph = st.speed_kph;
                            remote.interpolator.push_snapshot(server_time_ms, st);
                        }
                    }
                }
                NetworkMessage::Pong {
                    client_timestamp_ms,
                    ..
                } => {
                    let rtt = current_time_ms.saturating_sub(client_timestamp_ms) as f32;
                    let alpha = 0.2;
                    self.ping_ms = (1.0 - alpha) * self.ping_ms + alpha * rtt;
                }
                NetworkMessage::Disconnect { reason } => {
                    self.state = ConnectionState::Disconnected;
                    return Err(NetworkError::SessionError(format!(
                        "Disconnected by server: {}",
                        reason
                    )));
                }
                _ => {}
            }
        }

        // 2. Periodic heartbeat ping (every 1.0s when connected)
        if self.state == ConnectionState::Connected
            && self.last_ping_sent.elapsed() >= std::time::Duration::from_secs(1)
        {
            self.last_ping_sent = Instant::now();
            let ping = NetworkMessage::Ping {
                client_timestamp_ms: current_time_ms,
            };
            let _ = self.transport.send_message(
                self.server_addr,
                ChannelType::UnreliableSequenced,
                &ping,
            );
            self.packets_tx += 1;
        }

        // 3. Timeout detection (no packet for 5 seconds)
        if self.state == ConnectionState::Connected
            && self.last_packet_received.elapsed() >= std::time::Duration::from_secs(5)
        {
            self.state = ConnectionState::Reconnecting;
        }

        Ok(())
    }

    /// Sends a vehicle driving input frame to the authoritative server and advances local prediction.
    pub fn send_input(&mut self, input: &ClientInputFrame) -> Result<(), NetworkError> {
        // Advance zero-latency local prediction immediately
        self.predictor.predict_step(input);

        // Transmit input to authoritative server over low-latency channel
        if self.state == ConnectionState::Connected {
            let msg = NetworkMessage::Input(input.clone());
            self.transport.send_message(
                self.server_addr,
                ChannelType::UnreliableSequenced,
                &msg,
            )?;
            self.packets_tx += 1;
        }

        Ok(())
    }

    /// Evaluates interpolated render states for all visible remote vehicles.
    pub fn get_remote_vehicles(&self, render_time_ms: u64) -> Vec<RemoteVehicleRenderState> {
        let mut list = Vec::with_capacity(self.tracked_remotes.len());

        for (&entity_id, remote) in &self.tracked_remotes {
            if let Some((pos, orient)) = remote
                .interpolator
                .evaluate_interpolated_state(render_time_ms)
            {
                list.push(RemoteVehicleRenderState {
                    entity_id,
                    owner_player_id: remote.owner_player_id,
                    vehicle_type: remote.vehicle_type.clone(),
                    position: pos,
                    orientation: orient,
                    speed_kph: remote.last_speed_kph,
                    color: remote.color,
                });
            }
        }

        list
    }

    /// Cleanly disconnects from the multiplayer server.
    pub fn disconnect(&mut self, reason: &str) {
        if self.state == ConnectionState::Connected {
            let msg = NetworkMessage::Disconnect {
                reason: reason.to_string(),
            };
            let _ =
                self.transport
                    .send_message(self.server_addr, ChannelType::ReliableOrdered, &msg);
            self.packets_tx += 1;
        }
        self.state = ConnectionState::Disconnected;
        self.controlled_entity_id = None;
        self.tracked_remotes.clear();
    }

    /// Returns a structured snapshot of connection and network telemetry.
    pub fn status(&self) -> MultiplayerStatus {
        MultiplayerStatus {
            state: self.state,
            ping_ms: self.ping_ms,
            player_id: self.player_id,
            controlled_entity_id: self.controlled_entity_id,
            remote_player_count: self.tracked_remotes.len(),
            packets_rx: self.packets_rx,
            packets_tx: self.packets_tx,
            bytes_rx: self.bytes_rx,
            bytes_tx: self.bytes_tx,
            server_tick: self.server_tick,
            server_addr: self.server_addr,
        }
    }
}
