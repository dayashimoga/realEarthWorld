//! Kinematic Ackermann vehicle steering and motion model.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Kinematic bicycle / Ackermann model parameters and state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AckermannVehicle {
    /// Distance between front and rear axles in meters (e.g. 2.7m for standard sedan).
    pub wheelbase: f32,
    /// Maximum steering lock angle in radians (e.g. 0.6 rad ~ 35°).
    pub max_steer_angle: f32,
    /// Current steering angle in radians (positive = left).
    pub steer_angle: f32,
    /// Forward linear velocity in m/s.
    pub velocity: f32,
    /// Current heading yaw angle in radians.
    pub yaw: f32,
    /// 2D Local position (East, North) in meters.
    pub position: Vec2,
}

impl Default for AckermannVehicle {
    fn default() -> Self {
        Self {
            wheelbase: 2.7,
            max_steer_angle: 0.6,
            steer_angle: 0.0,
            velocity: 0.0,
            yaw: 0.0,
            position: Vec2::ZERO,
        }
    }
}

impl AckermannVehicle {
    pub fn new(wheelbase: f32, max_steer_angle: f32) -> Self {
        Self {
            wheelbase,
            max_steer_angle,
            ..Default::default()
        }
    }

    /// Sets steering input clamped to maximum steering lock.
    pub fn set_steer(&mut self, steer_rad: f32) {
        self.steer_angle = steer_rad.clamp(-self.max_steer_angle, self.max_steer_angle);
    }

    /// Sets throttle / target forward velocity.
    pub fn set_velocity(&mut self, velocity_ms: f32) {
        self.velocity = velocity_ms;
    }

    /// Integrates bicycle kinematic equations forward by `dt` seconds:
    /// dx/dt = v * cos(yaw)
    /// dy/dt = v * sin(yaw)
    /// dyaw/dt = (v / L) * tan(steer_angle)
    pub fn step(&mut self, dt: f32) {
        let dx = self.velocity * self.yaw.cos() * dt;
        let dy = self.velocity * self.yaw.sin() * dt;

        let dyaw = (self.velocity / self.wheelbase) * self.steer_angle.tan() * dt;

        self.position.x += dx;
        self.position.y += dy;
        self.yaw += dyaw;
    }
}
