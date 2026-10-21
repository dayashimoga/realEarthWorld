//! Pure Pursuit path following controller for autonomous vehicle navigation.

use crate::kinematics::AckermannVehicle;
use glam::Vec2;

/// Path tracking controller for autonomous vehicles using Pure Pursuit.
#[derive(Debug, Clone)]
pub struct PurePursuitController {
    pub waypoints: Vec<Vec2>,
    pub current_target_idx: usize,
    pub lookahead_distance: f32,
    pub target_speed_ms: f32,
}

impl PurePursuitController {
    pub fn new(waypoints: Vec<Vec2>, lookahead_distance: f32, target_speed_ms: f32) -> Self {
        Self {
            waypoints,
            current_target_idx: 0,
            lookahead_distance: lookahead_distance.max(1.0),
            target_speed_ms,
        }
    }

    /// Updates vehicle steering and velocity to track waypoints.
    pub fn update(&mut self, vehicle: &mut AckermannVehicle) -> bool {
        if self.waypoints.is_empty() || self.current_target_idx >= self.waypoints.len() {
            vehicle.set_velocity(0.0);
            return true; // Reached goal
        }

        // Find next waypoint beyond lookahead distance
        while self.current_target_idx < self.waypoints.len() {
            let target = self.waypoints[self.current_target_idx];
            let dist = (target - vehicle.position).length();
            if dist > self.lookahead_distance || self.current_target_idx == self.waypoints.len() - 1
            {
                break;
            }
            self.current_target_idx += 1;
        }

        let target = self.waypoints[self.current_target_idx];
        let diff = target - vehicle.position;
        let dist = diff.length();

        if dist < 0.5 && self.current_target_idx == self.waypoints.len() - 1 {
            vehicle.set_velocity(0.0);
            return true;
        }

        // Angle to target relative to global X-axis
        let target_angle = diff.y.atan2(diff.x);
        // Alpha is angle between vehicle heading and line to lookahead point
        let mut alpha = target_angle - vehicle.yaw;
        while alpha > std::f32::consts::PI {
            alpha -= 2.0 * std::f32::consts::PI;
        }
        while alpha < -std::f32::consts::PI {
            alpha += 2.0 * std::f32::consts::PI;
        }

        // Pure pursuit steering: delta = atan2(2 * L * sin(alpha), Ld)
        let steer = (2.0 * vehicle.wheelbase * alpha.sin() / self.lookahead_distance).atan();
        vehicle.set_steer(steer);
        vehicle.set_velocity(self.target_speed_ms);

        false
    }
}
