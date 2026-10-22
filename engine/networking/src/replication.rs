//! Client-side prediction, authoritative reconciliation with input replay buffer,
//! and remote entity snapshot interpolation.

use crate::protocol::{ClientInputFrame, EntityStatePayload};
use glam::{Quat, Vec3};
use std::collections::VecDeque;

/// A buffered client input frame retained for authoritative reconciliation.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferedInput {
    pub tick: u64,
    pub delta_time: f32,
    pub throttle: f32,
    pub steer: f32,
    pub brake: f32,
    pub predicted_position: Vec3,
    pub predicted_velocity: Vec3,
}

/// Client-side predictor providing zero perceived control latency while maintaining
/// server authority through rewind-and-replay reconciliation.
#[derive(Debug, Clone)]
pub struct ClientPredictor {
    pub current_tick: u64,
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub max_speed_mps: f32,
    pub input_buffer: VecDeque<BufferedInput>,
    /// Number of reconciliation rewind-and-replays performed.
    pub reconciliation_count: u64,
}

impl Default for ClientPredictor {
    fn default() -> Self {
        Self::new(Vec3::ZERO, Quat::IDENTITY, 60.0)
    }
}

impl ClientPredictor {
    pub fn new(position: Vec3, orientation: Quat, max_speed_mps: f32) -> Self {
        Self {
            current_tick: 0,
            position,
            orientation,
            linear_velocity: Vec3::ZERO,
            max_speed_mps,
            input_buffer: VecDeque::with_capacity(128),
            reconciliation_count: 0,
        }
    }

    /// Predicts local movement for the current client tick based on driver input.
    pub fn predict_step(&mut self, input: &ClientInputFrame) {
        self.current_tick = input.tick;
        let dt = input.delta_time;

        // Kinematic integration matching authoritative vehicle model
        let forward_dir = self.orientation * Vec3::Z;
        let accel_rate = 8.5; // m/s^2 forward drive
        let brake_rate = 14.0; // m/s^2 braking deceleration

        let speed = self.linear_velocity.dot(forward_dir);
        let mut net_accel = input.throttle * accel_rate;
        if input.brake > 0.05 {
            net_accel -= input.brake * brake_rate * speed.signum();
        }

        let new_speed = (speed + net_accel * dt).clamp(-15.0, self.max_speed_mps);
        self.linear_velocity = forward_dir * new_speed;

        // Steering yaw rate
        let yaw_rate = -input.steer * 0.9 * (new_speed / self.max_speed_mps).abs().max(0.2);
        let rot_delta = Quat::from_rotation_y(yaw_rate * dt);
        self.orientation = (rot_delta * self.orientation).normalize();

        self.position += self.linear_velocity * dt;

        // Store in replay buffer
        if self.input_buffer.len() >= 128 {
            self.input_buffer.pop_front();
        }
        self.input_buffer.push_back(BufferedInput {
            tick: input.tick,
            delta_time: dt,
            throttle: input.throttle,
            steer: input.steer,
            brake: input.brake,
            predicted_position: self.position,
            predicted_velocity: self.linear_velocity,
        });
    }

    /// Reconciles client prediction against an authoritative server snapshot.
    /// If divergence exceeds `divergence_tolerance_meters`, snaps back and replays subsequent inputs.
    pub fn reconcile_with_server_snapshot(
        &mut self,
        server_tick: u64,
        server_pos: Vec3,
        server_orient: Quat,
        server_vel: Vec3,
        divergence_tolerance_meters: f32,
    ) -> bool {
        // Find predicted state corresponding to server tick
        let mut needs_replay = false;

        if let Some(pos) = self.input_buffer.iter().position(|b| b.tick == server_tick) {
            let predicted = &self.input_buffer[pos];
            let error = (predicted.predicted_position - server_pos).length();

            if error > divergence_tolerance_meters {
                // Prediction diverged! Rewind state to authoritative server snapshot
                self.position = server_pos;
                self.orientation = server_orient;
                self.linear_velocity = server_vel;
                self.reconciliation_count += 1;
                needs_replay = true;
            }

            // Remove all inputs acknowledged by the server up to server_tick
            self.input_buffer.drain(..=pos);
        }

        if needs_replay {
            // Replay all remaining unacknowledged inputs forward to present
            for input in &mut self.input_buffer {
                let forward_dir = self.orientation * Vec3::Z;
                let accel_rate = 8.5;
                let brake_rate = 14.0;

                let speed = self.linear_velocity.dot(forward_dir);
                let mut net_accel = input.throttle * accel_rate;
                if input.brake > 0.05 {
                    net_accel -= input.brake * brake_rate * speed.signum();
                }

                let new_speed =
                    (speed + net_accel * input.delta_time).clamp(-15.0, self.max_speed_mps);
                self.linear_velocity = forward_dir * new_speed;

                let yaw_rate = -input.steer * 0.9 * (new_speed / self.max_speed_mps).abs().max(0.2);
                let rot_delta = Quat::from_rotation_y(yaw_rate * input.delta_time);
                self.orientation = (rot_delta * self.orientation).normalize();

                self.position += self.linear_velocity * input.delta_time;

                // Update buffered prediction records
                input.predicted_position = self.position;
                input.predicted_velocity = self.linear_velocity;
            }
        }

        needs_replay
    }
}

/// Timestamped snapshot record for remote entity interpolation.
#[derive(Debug, Clone, PartialEq)]
pub struct TimedSnapshot {
    pub timestamp_ms: u64,
    pub state: EntityStatePayload,
}

/// Remote vehicle snapshot interpolator for smooth rendering without jitter.
#[derive(Debug, Clone)]
pub struct RemoteEntityInterpolator {
    pub entity_id: u32,
    pub snapshots: VecDeque<TimedSnapshot>,
    pub interpolation_delay_ms: u64,
}

impl RemoteEntityInterpolator {
    pub fn new(entity_id: u32, interpolation_delay_ms: u64) -> Self {
        Self {
            entity_id,
            snapshots: VecDeque::with_capacity(32),
            interpolation_delay_ms,
        }
    }

    /// Pushes a newly received authoritative server snapshot.
    pub fn push_snapshot(&mut self, timestamp_ms: u64, state: EntityStatePayload) {
        if self.snapshots.len() >= 32 {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(TimedSnapshot {
            timestamp_ms,
            state,
        });
    }

    /// Evaluates interpolated position and orientation at client render time.
    pub fn evaluate_interpolated_state(&self, render_time_ms: u64) -> Option<(Vec3, Quat)> {
        if self.snapshots.is_empty() {
            return None;
        }

        let target_time = render_time_ms.saturating_sub(self.interpolation_delay_ms);

        // If only 1 snapshot, or target time is older than oldest snapshot
        if self.snapshots.len() == 1 || target_time <= self.snapshots.front()?.timestamp_ms {
            let snap = self.snapshots.front()?;
            return Some((snap.state.position, snap.state.orientation));
        }

        // If target time is newer than newest snapshot, extrapolate latest
        if target_time >= self.snapshots.back()?.timestamp_ms {
            let snap = self.snapshots.back()?;
            return Some((snap.state.position, snap.state.orientation));
        }

        // Find sandwiching snapshots: s0.timestamp <= target_time <= s1.timestamp
        for i in 0..self.snapshots.len() - 1 {
            let s0 = &self.snapshots[i];
            let s1 = &self.snapshots[i + 1];

            if s0.timestamp_ms <= target_time && target_time <= s1.timestamp_ms {
                let dt = (s1.timestamp_ms - s0.timestamp_ms) as f32;
                let alpha = if dt > 0.0 {
                    ((target_time - s0.timestamp_ms) as f32 / dt).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let pos = s0.state.position.lerp(s1.state.position, alpha);
                let orient = s0.state.orientation.slerp(s1.state.orientation, alpha);
                return Some((pos, orient));
            }
        }

        let snap = self.snapshots.back()?;
        Some((snap.state.position, snap.state.orientation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_prediction_and_reconciliation_replay() {
        let mut predictor = ClientPredictor::new(Vec3::ZERO, Quat::IDENTITY, 50.0);

        // Simulate 5 client ticks of acceleration forward
        for tick in 1..=5 {
            let input = ClientInputFrame {
                tick,
                delta_time: 0.1,
                throttle: 1.0,
                steer: 0.0,
                brake: 0.0,
                handbrake: false,
                client_timestamp_ms: tick * 100,
            };
            predictor.predict_step(&input);
        }

        let predicted_tick5_pos = predictor.position;
        assert!(predicted_tick5_pos.z > 0.5);
        assert_eq!(predictor.input_buffer.len(), 5);

        // Scenario A: Server snapshot matches prediction within tolerance (no divergence)
        let matched = predictor.reconcile_with_server_snapshot(
            3,
            predictor.input_buffer[2].predicted_position,
            predictor.orientation,
            predictor.linear_velocity,
            0.1,
        );
        assert!(!matched, "Matching snapshot must not trigger rewind replay");
        assert_eq!(predictor.reconciliation_count, 0);
        assert_eq!(predictor.input_buffer.len(), 2); // Ticks 1..=3 acknowledged & pruned

        // Scenario B: Server snapshot indicates divergence at tick 4 (e.g. struck obstacle)
        let server_diverged_pos = Vec3::new(0.0, 0.0, 1.2);
        let replayed = predictor.reconcile_with_server_snapshot(
            4,
            server_diverged_pos,
            Quat::IDENTITY,
            Vec3::new(0.0, 0.0, 2.0),
            0.05,
        );
        assert!(replayed, "Divergence must trigger rewind and replay");
        assert_eq!(predictor.reconciliation_count, 1);
        assert_eq!(predictor.input_buffer.len(), 1); // Only tick 5 remaining
    }

    #[test]
    fn test_remote_entity_snapshot_interpolation() {
        let mut interpolator = RemoteEntityInterpolator::new(42, 100);

        let snap0 = EntityStatePayload {
            entity_id: 42,
            owner_player_id: 1,
            position: Vec3::new(0.0, 0.0, 0.0),
            orientation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            speed_kph: 0.0,
            flags: 0,
        };

        let snap1 = EntityStatePayload {
            entity_id: 42,
            owner_player_id: 1,
            position: Vec3::new(0.0, 0.0, 20.0),
            orientation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            speed_kph: 72.0,
            flags: 0,
        };

        interpolator.push_snapshot(1000, snap0);
        interpolator.push_snapshot(1200, snap1);

        // At render_time = 1200, with 100ms interpolation delay -> target_time = 1100 (halfway)
        let (pos, _) = interpolator
            .evaluate_interpolated_state(1200)
            .expect("Should interpolate halfway");

        assert!(
            (pos.z - 10.0).abs() < 1e-3,
            "Midpoint interpolation should equal 10.0, got {}",
            pos.z
        );
    }
}
