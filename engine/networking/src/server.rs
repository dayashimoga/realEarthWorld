//! Authoritative multiplayer server coordinator: tick loop, input validation,
//! anti-cheat speed-hack verification, Area of Interest replication, and cell handoff.

use crate::interest::InterestManager;
use crate::protocol::{ClientInputFrame, EntityStatePayload, NetworkError, NetworkMessage};
use crate::session::SessionManager;
use glam::{Quat, Vec3};
use std::collections::HashMap;

/// Authoritative physical state of an entity on the server.
#[derive(Debug, Clone)]
pub struct ServerEntity {
    pub entity_id: u32,
    pub owner_player_id: u32,
    pub vehicle_type: String,
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub max_speed_mps: f32,
    pub max_accel_mps2: f32,
    pub speed_kph: f32,
    pub last_input_tick: u64,
}

/// Authoritative planetary multiplayer world-cell server.
#[derive(Debug)]
pub struct AuthoritativeServer {
    pub server_tick: u64,
    pub tick_rate_hz: u32,
    pub sessions: SessionManager,
    pub interest_manager: InterestManager,
    pub entities: HashMap<u32, ServerEntity>,
    next_entity_id: u32,
    /// Detected cell handoff events for active entities.
    pub pending_handoffs: Vec<(u32, String)>,
    /// Security audit violation log.
    pub security_violations: Vec<String>,
}

impl Default for AuthoritativeServer {
    fn default() -> Self {
        Self::new(60, 64, 1000.0)
    }
}

impl AuthoritativeServer {
    pub fn new(tick_rate_hz: u32, max_players: usize, cell_size_meters: f32) -> Self {
        Self {
            server_tick: 0,
            tick_rate_hz,
            sessions: SessionManager::new(max_players, 10.0, 15.0),
            interest_manager: InterestManager::new(cell_size_meters),
            entities: HashMap::new(),
            next_entity_id: 100,
            pending_handoffs: Vec::new(),
            security_violations: Vec::new(),
        }
    }

    /// Spawns an authoritative vehicle entity assigned to a connected player.
    pub fn spawn_vehicle(
        &mut self,
        player_id: u32,
        vehicle_type: &str,
        spawn_pos: Vec3,
        spawn_orient: Quat,
    ) -> Result<u32, NetworkError> {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;

        let entity = ServerEntity {
            entity_id,
            owner_player_id: player_id,
            vehicle_type: vehicle_type.to_string(),
            position: spawn_pos,
            orientation: spawn_orient,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            max_speed_mps: 70.0,  // ~250 km/h
            max_accel_mps2: 12.0, // High-performance sports car limit
            speed_kph: 0.0,
            last_input_tick: self.server_tick,
        };

        self.entities.insert(entity_id, entity);
        self.interest_manager
            .update_entity_position(entity_id, spawn_pos);

        // Associate with player session
        for session in self.sessions.sessions.values_mut() {
            if session.player_id == player_id {
                session.controlled_entity_id = Some(entity_id);
                break;
            }
        }

        Ok(entity_id)
    }

    /// Ingests and validates client input frame with anti-cheat checks.
    pub fn process_client_input(
        &mut self,
        session_id: &str,
        entity_id: u32,
        input: &ClientInputFrame,
    ) -> Result<(), NetworkError> {
        // 1. Validate payload format (NaN/infinity/range checks)
        input.validate()?;

        // 2. Validate session & rate limiter
        let session = self
            .sessions
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| NetworkError::SessionError("Session not found".to_string()))?;

        if !session.rate_limiter.try_consume() {
            return Err(NetworkError::RateLimitExceeded(120));
        }

        // 3. Authorization check: player can only control their own entity
        if session.controlled_entity_id != Some(entity_id) {
            let log_msg = format!(
                "Security Alert: Player {} attempted unauthorized control of entity {}",
                session.player_id, entity_id
            );
            self.security_violations.push(log_msg.clone());
            return Err(NetworkError::Unauthorized(log_msg));
        }

        // 4. Physical simulation update
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            let dt = input.delta_time;
            let forward_dir = entity.orientation * Vec3::Z;

            let current_speed = entity.linear_velocity.dot(forward_dir);
            let mut accel = input.throttle * 8.5;
            if input.brake > 0.05 {
                accel -= input.brake * 14.0 * current_speed.signum();
            }

            // Anti-cheat acceleration clamp
            let clamped_accel = accel.clamp(-entity.max_accel_mps2, entity.max_accel_mps2);
            let new_speed = (current_speed + clamped_accel * dt).clamp(-15.0, entity.max_speed_mps);

            entity.linear_velocity = forward_dir * new_speed;
            entity.speed_kph = new_speed.abs() * 3.6;

            // Steering yaw
            let yaw_rate = -input.steer * 0.9 * (new_speed / entity.max_speed_mps).abs().max(0.2);
            let rot_delta = Quat::from_rotation_y(yaw_rate * dt);
            entity.orientation = (rot_delta * entity.orientation).normalize();

            // Displacement integration
            let displacement = entity.linear_velocity * dt;
            entity.position += displacement;
            entity.last_input_tick = input.tick;

            // 5. Update Area of Interest and detect boundary handoffs
            if let Some((_, new_cell)) = self
                .interest_manager
                .update_entity_position(entity_id, entity.position)
            {
                self.pending_handoffs
                    .push((entity_id, new_cell.to_string_id()));
            }
        }

        Ok(())
    }

    /// Advances the authoritative server tick loop.
    pub fn step_tick(&mut self) {
        self.server_tick += 1;
        self.sessions.poll_timeouts();
        for session in self.sessions.sessions.values_mut() {
            session.rate_limiter.tokens =
                (session.rate_limiter.tokens + 1.0).min(session.rate_limiter.capacity);
        }
    }

    /// Generates tailored Area of Interest state snapshots for a specific player session.
    pub fn build_snapshot_for_player(
        &self,
        player_id: u32,
        interest_radius_meters: f32,
    ) -> Option<NetworkMessage> {
        // Find player's controlled entity position
        let player_entity = self
            .entities
            .values()
            .find(|e| e.owner_player_id == player_id);

        let center_pos = player_entity.map(|e| e.position).unwrap_or(Vec3::ZERO);

        // Spatial Area of Interest filtering
        let visible_ids = self
            .interest_manager
            .query_entities_in_radius(center_pos, interest_radius_meters);

        let states: Vec<EntityStatePayload> = visible_ids
            .into_iter()
            .filter_map(|id| self.entities.get(&id))
            .map(|e| EntityStatePayload {
                entity_id: e.entity_id,
                owner_player_id: e.owner_player_id,
                position: e.position,
                orientation: e.orientation,
                linear_velocity: e.linear_velocity,
                angular_velocity: e.angular_velocity,
                speed_kph: e.speed_kph,
                flags: 0,
            })
            .collect();

        Some(NetworkMessage::ServerSnapshot {
            tick: self.server_tick,
            server_time_ms: self.server_tick * (1000 / self.tick_rate_hz as u64),
            states,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::PlayerRole;

    #[test]
    fn test_authoritative_server_session_and_vehicle_lifecycle() {
        let mut server = AuthoritativeServer::new(60, 8, 1000.0);

        // Register client
        let session = server
            .sessions
            .register_player("SpeedyRider", PlayerRole::Player)
            .unwrap();

        // Spawn vehicle
        let entity_id = server
            .spawn_vehicle(session.player_id, "Sedan", Vec3::ZERO, Quat::IDENTITY)
            .unwrap();

        assert_eq!(server.entities.len(), 1);
        assert_eq!(server.entities.get(&entity_id).unwrap().owner_player_id, 1);

        // Process valid input
        let input = ClientInputFrame {
            tick: 1,
            delta_time: 1.0 / 60.0,
            throttle: 1.0,
            steer: 0.0,
            brake: 0.0,
            handbrake: false,
            client_timestamp_ms: 100,
        };

        let result = server.process_client_input(&session.session_id, entity_id, &input);
        assert!(result.is_ok());

        let entity = server.entities.get(&entity_id).unwrap();
        assert!(entity.position.z > 0.0);
        assert!(entity.speed_kph > 0.0);
    }

    #[test]
    fn test_unauthorized_entity_control_security_rejection() {
        let mut server = AuthoritativeServer::new(60, 8, 1000.0);

        let p1 = server
            .sessions
            .register_player("Player1", PlayerRole::Player)
            .unwrap();
        let p2 = server
            .sessions
            .register_player("Player2", PlayerRole::Player)
            .unwrap();

        let e1 = server
            .spawn_vehicle(p1.player_id, "Truck", Vec3::ZERO, Quat::IDENTITY)
            .unwrap();

        // Player 2 attempts to send inputs controlling Player 1's truck (E1)
        let malicious_input = ClientInputFrame {
            tick: 1,
            delta_time: 1.0 / 60.0,
            throttle: 1.0,
            steer: 0.0,
            brake: 0.0,
            handbrake: false,
            client_timestamp_ms: 100,
        };

        let result = server.process_client_input(&p2.session_id, e1, &malicious_input);
        assert!(result.is_err());
        assert_eq!(server.security_violations.len(), 1);
    }

    #[test]
    fn test_spatial_cell_handoff_emission() {
        let mut server = AuthoritativeServer::new(60, 8, 500.0); // 500m cells

        let p1 = server
            .sessions
            .register_player("HighSpeedHauler", PlayerRole::Player)
            .unwrap();
        let e1 = server
            .spawn_vehicle(
                p1.player_id,
                "Sedan",
                Vec3::new(498.0, 0.0, 0.0),
                Quat::IDENTITY,
            )
            .unwrap();

        // Step across 500m boundary: forward along X
        let entity = server.entities.get_mut(&e1).unwrap();
        entity.orientation = Quat::from_rotation_y(90.0_f32.to_radians());
        entity.linear_velocity = Vec3::new(25.0, 0.0, 0.0); // 25 m/s along X

        let input = ClientInputFrame {
            tick: 1,
            delta_time: 0.2, // Valid delta_time (<= 0.5s)
            throttle: 1.0,
            steer: 0.0,
            brake: 0.0,
            handbrake: false,
            client_timestamp_ms: 1000,
        };

        server
            .process_client_input(&p1.session_id, e1, &input)
            .unwrap();
        assert!(
            !server.pending_handoffs.is_empty(),
            "Handoff must be emitted when crossing cell"
        );
    }
}
