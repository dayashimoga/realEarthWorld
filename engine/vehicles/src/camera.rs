//! Automotive camera systems: third-person chase camera, hood camera, and bumper camera.

use crate::dynamics::DynamicVehicle;
use glam::{Mat3, Vec3};

/// Viewing perspective modes relative to the vehicle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleCameraPerspective {
    /// Third-person spring-damper chase camera following behind and above the car.
    Chase,
    /// First-person view mounted on the vehicle hood facing forward.
    Hood,
    /// Low-slung front bumper perspective.
    Bumper,
}

/// Dynamic vehicle camera controller maintaining smooth motion without visual snapping.
pub struct VehicleCameraController {
    pub perspective: VehicleCameraPerspective,
    /// Distance behind the vehicle in Chase mode (default: 5.5m).
    pub chase_distance: f32,
    /// Height above the vehicle in Chase mode (default: 2.2m).
    pub chase_height: f32,
    /// Camera position smoothing factor.
    pub position_smoothing: f32,
    /// Current interpolated camera position in world space.
    pub current_camera_pos: Vec3,
    /// Current interpolated camera yaw angle in radians.
    pub current_yaw: f32,
    /// Current interpolated camera pitch angle in radians.
    pub current_pitch: f32,
}

impl Default for VehicleCameraController {
    fn default() -> Self {
        Self {
            perspective: VehicleCameraPerspective::Chase,
            chase_distance: 5.5,
            chase_height: 2.2,
            position_smoothing: 8.0,
            current_camera_pos: Vec3::ZERO,
            current_yaw: 0.0,
            current_pitch: -0.15,
        }
    }
}

impl VehicleCameraController {
    pub fn new(perspective: VehicleCameraPerspective) -> Self {
        Self {
            perspective,
            ..Default::default()
        }
    }

    /// Cycles through camera perspectives (Chase -> Hood -> Bumper -> Chase).
    pub fn cycle_perspective(&mut self) {
        self.perspective = match self.perspective {
            VehicleCameraPerspective::Chase => VehicleCameraPerspective::Hood,
            VehicleCameraPerspective::Hood => VehicleCameraPerspective::Bumper,
            VehicleCameraPerspective::Bumper => VehicleCameraPerspective::Chase,
        };
    }

    /// Computes desired camera position, yaw, and pitch from the active vehicle state.
    pub fn update(&mut self, vehicle: &DynamicVehicle, dt: f32) -> (Vec3, f32, f32) {
        let rot_mat = Mat3::from_quat(vehicle.orientation);
        let car_forward = rot_mat.col(2);
        let car_up = rot_mat.col(1);

        let target_yaw = car_forward.x.atan2(car_forward.z);

        match self.perspective {
            VehicleCameraPerspective::Chase => {
                // Target position: behind and above vehicle
                let lookahead_offset = car_forward * 1.5;
                let target_cam_pos = vehicle.position - car_forward * self.chase_distance
                    + Vec3::new(0.0, self.chase_height, 0.0);

                if self.current_camera_pos == Vec3::ZERO {
                    self.current_camera_pos = target_cam_pos;
                    self.current_yaw = target_yaw;
                } else {
                    let blend = (self.position_smoothing * dt).clamp(0.0, 1.0);
                    self.current_camera_pos = self.current_camera_pos.lerp(target_cam_pos, blend);

                    // Angular shortest-path lerp for yaw
                    let mut diff = target_yaw - self.current_yaw;
                    while diff > std::f32::consts::PI {
                        diff -= 2.0 * std::f32::consts::PI;
                    }
                    while diff < -std::f32::consts::PI {
                        diff += 2.0 * std::f32::consts::PI;
                    }
                    self.current_yaw += diff * blend;
                }

                // Look slightly down at target lookahead point ahead of vehicle
                let to_target = (vehicle.position + lookahead_offset) - self.current_camera_pos;
                let pitch = (to_target.y / to_target.length().max(0.1)).asin();
                self.current_pitch = pitch.clamp(-0.4, 0.2);
            }
            VehicleCameraPerspective::Hood => {
                // Mounted 0.5m forward and 0.9m up relative to center
                self.current_camera_pos = vehicle.position + car_forward * 0.8 + car_up * 0.9;
                self.current_yaw = target_yaw;
                self.current_pitch = -0.05;
            }
            VehicleCameraPerspective::Bumper => {
                // Mounted 1.8m forward and 0.35m up
                self.current_camera_pos = vehicle.position + car_forward * 1.8 + car_up * 0.35;
                self.current_yaw = target_yaw;
                self.current_pitch = 0.0;
            }
        }

        (
            self.current_camera_pos,
            self.current_yaw,
            self.current_pitch,
        )
    }
}
