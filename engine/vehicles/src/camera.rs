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
    /// Speed-aware dynamic field of view in degrees.
    pub current_fov_deg: f32,
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
            current_fov_deg: 60.0,
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

    /// Computes speed-aware field of view in degrees (60.0° base widening up to 72.0° at high speeds).
    pub fn compute_fov_degrees(speed_kph: f32) -> f32 {
        let speed_ratio = (speed_kph.abs() / 120.0).clamp(0.0, 1.0);
        60.0 + 12.0 * speed_ratio
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

    /// Resets the camera controller state, snapping to target vehicle pose on next update.
    pub fn reset(&mut self) {
        self.current_camera_pos = Vec3::ZERO;
        self.current_yaw = 0.0;
        self.current_pitch = -0.15;
        self.current_fov_deg = 60.0;
    }

    /// Computes desired camera position, yaw, and pitch from the active vehicle state.
    pub fn update(&mut self, vehicle: &DynamicVehicle, dt: f32) -> (Vec3, f32, f32) {
        self.update_with_ground(vehicle, dt, None)
    }

    /// Computes desired camera position, yaw, and pitch with terrain elevation clamping.
    pub fn update_with_ground(
        &mut self,
        vehicle: &DynamicVehicle,
        dt: f32,
        ground_elevation: Option<f32>,
    ) -> (Vec3, f32, f32) {
        self.current_fov_deg = Self::compute_fov_degrees(vehicle.speed_kph);

        let rot_mat = Mat3::from_quat(vehicle.orientation);
        let car_forward = rot_mat.col(2);
        let car_up = rot_mat.col(1);
        let car_right = rot_mat.col(0);

        // Camera yaw convention aligns with camera.rs: front = (cos(yaw)*cos(pitch), sin(pitch), sin(yaw)*cos(pitch)).
        // Therefore, horizontal forward (x, z) maps to yaw = z.atan2(x).
        let target_yaw = car_forward.z.atan2(car_forward.x);

        match self.perspective {
            VehicleCameraPerspective::Chase => {
                // Lookahead accounts for forward velocity and front steering angle
                let steer_lookahead = car_right * (vehicle.wheels[0].steer_angle * 1.5);
                let speed_bonus = (vehicle.speed_kph / 80.0).clamp(0.0, 2.5);
                let lookahead_offset = car_forward * (1.5 + speed_bonus) + steer_lookahead;

                let mut target_cam_pos = vehicle.position - car_forward * self.chase_distance
                    + Vec3::new(0.0, self.chase_height, 0.0);

                if let Some(ground_y) = ground_elevation {
                    target_cam_pos.y = target_cam_pos.y.max(ground_y + 0.85);
                }

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

                if let Some(ground_y) = ground_elevation {
                    self.current_camera_pos.y = self.current_camera_pos.y.max(ground_y + 0.85);
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
        self.update_pose_with_ground(
            position,
            orientation,
            chase_distance,
            chase_height,
            0.0,
            dt,
            None,
        )
    }

    /// Generic pose-based camera positioning supporting multimodal vehicles with speed FOV and ground clamping.
    #[allow(clippy::too_many_arguments)]
    pub fn update_pose_with_ground(
        &mut self,
        position: Vec3,
        orientation: glam::Quat,
        chase_distance: f32,
        chase_height: f32,
        speed_kph: f32,
        dt: f32,
        ground_elevation: Option<f32>,
    ) -> (Vec3, f32, f32) {
        self.current_fov_deg = Self::compute_fov_degrees(speed_kph);

        let rot_mat = Mat3::from_quat(orientation);
        let forward = rot_mat.col(2);
        let target_yaw = forward.z.atan2(forward.x);

        match self.perspective {
            VehicleCameraPerspective::Chase => {
                let lookahead = forward * 2.0;
                let mut target_cam =
                    position - forward * chase_distance + Vec3::new(0.0, chase_height, 0.0);
                if let Some(ground_y) = ground_elevation {
                    target_cam.y = target_cam.y.max(ground_y + 0.85);
                }

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

                if let Some(ground_y) = ground_elevation {
                    self.current_camera_pos.y = self.current_camera_pos.y.max(ground_y + 0.85);
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

    #[test]
    fn test_speed_aware_fov() {
        assert_eq!(VehicleCameraController::compute_fov_degrees(0.0), 60.0);
        assert_eq!(VehicleCameraController::compute_fov_degrees(60.0), 66.0);
        assert_eq!(VehicleCameraController::compute_fov_degrees(120.0), 72.0);
        assert_eq!(VehicleCameraController::compute_fov_degrees(180.0), 72.0); // Clamped at 72
    }

    #[test]
    fn test_camera_terrain_elevation_clamping() {
        let mut cam = VehicleCameraController::new(VehicleCameraPerspective::Chase);
        let veh = DynamicVehicle::new(VehicleConfig::default(), Vec3::new(0.0, 5.0, 0.0), 0.0);

        // Ground elevation below vehicle (normal flat ground at Y=0.0)
        let (pos_flat, _, _) = cam.update_with_ground(&veh, 0.016, Some(0.0));
        assert!(pos_flat.y > 0.85);

        // Ground elevation high up on steep hill (e.g. hill slope at Y=6.5)
        let (pos_hill, _, _) = cam.update_with_ground(&veh, 0.016, Some(6.5));
        assert!(pos_hill.y >= 6.5 + 0.85);
    }
}
