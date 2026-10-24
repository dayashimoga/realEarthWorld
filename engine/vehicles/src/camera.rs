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
    /// First-person interior cockpit perspective from the driver's seat.
    Cockpit,
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

    /// Cycles through camera perspectives (Chase -> Hood -> Cockpit -> Bumper -> Chase).
    pub fn cycle_perspective(&mut self) {
        self.perspective = match self.perspective {
            VehicleCameraPerspective::Chase => VehicleCameraPerspective::Hood,
            VehicleCameraPerspective::Hood => VehicleCameraPerspective::Cockpit,
            VehicleCameraPerspective::Cockpit => VehicleCameraPerspective::Bumper,
            VehicleCameraPerspective::Bumper => VehicleCameraPerspective::Chase,
        };
    }

    /// Computes desired camera position, yaw, and pitch from the active vehicle state.
    pub fn update(&mut self, vehicle: &DynamicVehicle, dt: f32) -> (Vec3, f32, f32) {
        let rot_mat = Mat3::from_quat(vehicle.orientation);
        let car_forward = rot_mat.col(2);
        let car_up = rot_mat.col(1);
        let car_right = rot_mat.col(0);

        // Camera yaw convention aligns with camera.rs: front = (cos(yaw)*cos(pitch), sin(pitch), sin(yaw)*cos(pitch)).
        // Therefore, horizontal forward (x, z) maps to yaw = z.atan2(x).
        let target_yaw = car_forward.z.atan2(car_forward.x);

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
                // Mounted 0.8m forward and 0.9m up relative to center
                self.current_camera_pos = vehicle.position + car_forward * 0.8 + car_up * 0.9;
                self.current_yaw = target_yaw;
                self.current_pitch = -0.05;
            }
            VehicleCameraPerspective::Cockpit => {
                // Driver eye position in left-hand drive cockpit: -0.38m X, +1.05m Y, +0.15m Z
                self.current_camera_pos =
                    vehicle.position + car_right * -0.38 + car_up * 1.05 + car_forward * 0.15;
                self.current_yaw = target_yaw;
                self.current_pitch = -0.06;
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

    /// Generic pose-based camera positioning supporting multimodal vehicles.
    pub fn update_pose(
        &mut self,
        position: Vec3,
        orientation: glam::Quat,
        chase_distance: f32,
        chase_height: f32,
        dt: f32,
    ) -> (Vec3, f32, f32) {
        let rot_mat = Mat3::from_quat(orientation);
        let forward = rot_mat.col(2);
        let target_yaw = forward.z.atan2(forward.x);

        match self.perspective {
            VehicleCameraPerspective::Chase => {
                let lookahead = forward * 2.0;
                let target_cam =
                    position - forward * chase_distance + Vec3::new(0.0, chase_height, 0.0);
                if self.current_camera_pos == Vec3::ZERO {
                    self.current_camera_pos = target_cam;
                    self.current_yaw = target_yaw;
                } else {
                    let blend = (self.position_smoothing * dt).clamp(0.0, 1.0);
                    self.current_camera_pos = self.current_camera_pos.lerp(target_cam, blend);
                    let mut diff = target_yaw - self.current_yaw;
                    while diff > std::f32::consts::PI {
                        diff -= 2.0 * std::f32::consts::PI;
                    }
                    while diff < -std::f32::consts::PI {
                        diff += 2.0 * std::f32::consts::PI;
                    }
                    self.current_yaw += diff * blend;
                }
                let to_target = (position + lookahead) - self.current_camera_pos;
                let pitch = (to_target.y / to_target.length().max(0.1)).asin();
                self.current_pitch = pitch.clamp(-0.4, 0.2);
            }
            VehicleCameraPerspective::Hood | VehicleCameraPerspective::Cockpit => {
                let eye_h = (chase_height * 0.45).max(0.8);
                self.current_camera_pos = position + forward * 0.5 + Vec3::new(0.0, eye_h, 0.0);
                self.current_yaw = target_yaw;
                self.current_pitch = -0.05;
            }
            VehicleCameraPerspective::Bumper => {
                self.current_camera_pos =
                    position + forward * (chase_distance * 0.25) + Vec3::new(0.0, 0.4, 0.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamics::VehicleConfig;

    #[test]
    fn test_camera_perspective_cycling() {
        let mut cam = VehicleCameraController::new(VehicleCameraPerspective::Chase);
        assert_eq!(cam.perspective, VehicleCameraPerspective::Chase);

        cam.cycle_perspective();
        assert_eq!(cam.perspective, VehicleCameraPerspective::Hood);

        cam.cycle_perspective();
        assert_eq!(cam.perspective, VehicleCameraPerspective::Cockpit);

        cam.cycle_perspective();
        assert_eq!(cam.perspective, VehicleCameraPerspective::Bumper);

        cam.cycle_perspective();
        assert_eq!(cam.perspective, VehicleCameraPerspective::Chase);
    }

    #[test]
    fn test_cockpit_camera_positioning() {
        let mut cam = VehicleCameraController::new(VehicleCameraPerspective::Cockpit);
        let veh = DynamicVehicle::new(VehicleConfig::default(), Vec3::new(10.0, 5.0, 20.0), 0.0);

        let (pos, yaw, pitch) = cam.update(&veh, 0.016);
        // Vehicle forward is along +Z, right is +X, up is +Y
        assert!((pos.x - (10.0 - 0.38)).abs() < 1e-3);
        assert!((pos.y - (5.0 + 1.05)).abs() < 1e-3);
        assert!((pos.z - (20.0 + 0.15)).abs() < 1e-3);
        // When vehicle is pointing along +Z (North), camera yaw is +PI/2
        assert!((yaw - std::f32::consts::FRAC_PI_2).abs() < 1e-3);
        assert!((pitch - (-0.06)).abs() < 1e-3);
    }

    #[test]
    fn test_vehicle_camera_cardinal_direction_alignment() {
        let mut cam = VehicleCameraController::new(VehicleCameraPerspective::Chase);
        // Test 4 cardinal directions
        // 1. Vehicle facing North (+Z, yaw = 0.0) -> camera looks North (yaw = PI/2)
        let veh_n = DynamicVehicle::new(VehicleConfig::default(), Vec3::ZERO, 0.0);
        let (_, yaw_n, _) = cam.update(&veh_n, 0.016);
        assert!((yaw_n - std::f32::consts::FRAC_PI_2).abs() < 1e-3);

        // 2. Vehicle facing East (+X, yaw = PI/2) -> camera looks East (yaw = 0.0)
        let mut cam_e = VehicleCameraController::new(VehicleCameraPerspective::Chase);
        let veh_e = DynamicVehicle::new(
            VehicleConfig::default(),
            Vec3::ZERO,
            std::f32::consts::FRAC_PI_2,
        );
        let (_, yaw_e, _) = cam_e.update(&veh_e, 0.016);
        assert!(yaw_e.abs() < 1e-3);

        // 3. Vehicle facing South (-Z, yaw = PI) -> camera looks South (yaw = -PI/2)
        let mut cam_s = VehicleCameraController::new(VehicleCameraPerspective::Chase);
        let veh_s = DynamicVehicle::new(VehicleConfig::default(), Vec3::ZERO, std::f32::consts::PI);
        let (_, yaw_s, _) = cam_s.update(&veh_s, 0.016);
        assert!((yaw_s - (-std::f32::consts::FRAC_PI_2)).abs() < 1e-3);

        // 4. Vehicle facing West (-X, yaw = -PI/2) -> camera looks West (yaw = PI)
        let mut cam_w = VehicleCameraController::new(VehicleCameraPerspective::Chase);
        let veh_w = DynamicVehicle::new(
            VehicleConfig::default(),
            Vec3::ZERO,
            -std::f32::consts::FRAC_PI_2,
        );
        let (_, yaw_w, _) = cam_w.update(&veh_w, 0.016);
        assert!((yaw_w.abs() - std::f32::consts::PI).abs() < 1e-3);
    }
}
