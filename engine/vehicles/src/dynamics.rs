//! Dedicated automotive raycast vehicle dynamics solver, suspension physics,
//! and tire friction circle model.

use crate::ground::GroundQuery;
use glam::{Mat3, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Identifiers for the 4 vehicle wheels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WheelIndex {
    FrontLeft = 0,
    FrontRight = 1,
    RearLeft = 2,
    RearRight = 3,
}

/// Transmission operating modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TransmissionMode {
    #[default]
    Drive,
    Reverse,
    Neutral,
    Park,
}

/// Driving control inputs for the vehicle dynamics model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VehicleInputs {
    /// Throttle input normalized from 0.0 (idle) to 1.0 (full throttle).
    pub throttle: f32,
    /// Service brake input normalized from 0.0 to 1.0.
    pub brake: f32,
    /// Steering input from -1.0 (full left) to +1.0 (full right).
    pub steer: f32,
    /// Handbrake engagement flag (locks rear wheels).
    pub handbrake: bool,
    /// Transmission gear selection.
    pub gear_mode: TransmissionMode,
}

impl Default for VehicleInputs {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            brake: 0.0,
            steer: 0.0,
            handbrake: false,
            gear_mode: TransmissionMode::Drive,
        }
    }
}

/// Detailed vehicle chassis, suspension, engine, and tire parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleConfig {
    /// Total curb weight in kilograms (e.g. 1450 kg for midsize sedan).
    pub mass_kg: f32,
    /// Wheelbase: longitudinal distance between front and rear axles in meters.
    pub wheelbase: f32,
    /// Track width: lateral distance between left and right wheels in meters.
    pub track_width: f32,
    /// Chassis collision half-extents: [half_width, half_height, half_length].
    pub half_extents: Vec3,
    /// Center of mass vertical offset relative to geometric center.
    pub com_height_offset: f32,
    /// Suspension strut uncompressed rest length in meters (default: 0.35m).
    pub suspension_rest_length: f32,
    /// Suspension coil spring stiffness in N/m (default: 35,000 N/m).
    pub suspension_stiffness: f32,
    /// Suspension shock absorber damping coefficient in N*s/m (default: 3,200 N*s/m).
    pub suspension_damping: f32,
    /// Wheel radius in meters (default: 0.33m ~ 17-inch rim with tire).
    pub wheel_radius: f32,
    /// Maximum steering lock angle in radians (default: 0.61 rad ~ 35°).
    pub max_steer_angle: f32,
    /// Peak engine drive force at wheels in Newtons (default: 6,500 N).
    pub peak_engine_force: f32,
    /// Maximum service brake force per wheel in Newtons (default: 4,000 N front, 2,800 N rear).
    pub max_brake_force: f32,
    /// Handbrake force on rear wheels in Newtons (default: 6,000 N).
    pub handbrake_force: f32,
    /// Tire-ground friction coefficient (default: 0.95 dry asphalt, 0.65 wet).
    pub tire_friction_coeff: f32,
    /// Tire lateral cornering stiffness in N/rad (default: 32,000 N/rad).
    pub cornering_stiffness: f32,
    /// Aerodynamic drag coefficient factor (0.5 * rho * Cd * A) in kg/m (default: 0.42).
    pub aero_drag_coeff: f32,
    /// Rolling resistance coefficient (default: 0.015).
    pub rolling_resistance_coeff: f32,
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            mass_kg: 1450.0,
            wheelbase: 2.7,
            track_width: 1.6,
            half_extents: Vec3::new(0.9, 0.6, 2.2),
            com_height_offset: -0.25,
            suspension_rest_length: 0.35,
            suspension_stiffness: 35000.0,
            suspension_damping: 3200.0,
            wheel_radius: 0.33,
            max_steer_angle: 0.61,
            peak_engine_force: 6500.0,
            max_brake_force: 3500.0,
            handbrake_force: 6000.0,
            tire_friction_coeff: 0.95,
            cornering_stiffness: 32000.0,
            aero_drag_coeff: 0.42,
            rolling_resistance_coeff: 0.015,
        }
    }
}

impl VehicleConfig {
    /// High-performance sports coupe: lightweight, responsive steering, stiffer suspension.
    pub fn sports_coupe() -> Self {
        Self {
            mass_kg: 1250.0,
            wheelbase: 2.5,
            track_width: 1.65,
            half_extents: Vec3::new(0.9, 0.55, 2.1),
            com_height_offset: -0.30,
            suspension_rest_length: 0.28,
            suspension_stiffness: 42000.0,
            suspension_damping: 3800.0,
            wheel_radius: 0.33,
            max_steer_angle: 0.65,
            peak_engine_force: 8500.0,
            max_brake_force: 4500.0,
            handbrake_force: 7000.0,
            tire_friction_coeff: 1.05,
            cornering_stiffness: 38000.0,
            aero_drag_coeff: 0.34,
            rolling_resistance_coeff: 0.014,
        }
    }

    /// Standard passenger sedan: balanced comfort, stability, and handling.
    pub fn sedan() -> Self {
        Self::default()
    }

    /// High ground-clearance SUV: longer suspension travel, robust all-terrain damping.
    pub fn suv() -> Self {
        Self {
            mass_kg: 1950.0,
            wheelbase: 2.85,
            track_width: 1.72,
            half_extents: Vec3::new(0.98, 0.75, 2.4),
            com_height_offset: -0.20,
            suspension_rest_length: 0.42,
            suspension_stiffness: 38000.0,
            suspension_damping: 3600.0,
            wheel_radius: 0.38,
            max_steer_angle: 0.58,
            peak_engine_force: 7800.0,
            max_brake_force: 4200.0,
            handbrake_force: 6500.0,
            tire_friction_coeff: 0.92,
            cornering_stiffness: 30000.0,
            aero_drag_coeff: 0.48,
            rolling_resistance_coeff: 0.018,
        }
    }

    /// Urban commercial taxi: nimble steering, quick low-end throttle response.
    pub fn taxi() -> Self {
        Self {
            mass_kg: 1400.0,
            wheelbase: 2.65,
            track_width: 1.6,
            half_extents: Vec3::new(0.9, 0.6, 2.2),
            com_height_offset: -0.25,
            suspension_rest_length: 0.34,
            suspension_stiffness: 34000.0,
            suspension_damping: 3100.0,
            wheel_radius: 0.32,
            max_steer_angle: 0.68,
            peak_engine_force: 6800.0,
            max_brake_force: 3600.0,
            handbrake_force: 6200.0,
            tire_friction_coeff: 0.96,
            cornering_stiffness: 33000.0,
            aero_drag_coeff: 0.40,
            rolling_resistance_coeff: 0.015,
        }
    }

    /// Public transit bus: high passenger capacity, long wheelbase, stable cruising.
    pub fn transit_bus() -> Self {
        Self {
            mass_kg: 9500.0,
            wheelbase: 5.8,
            track_width: 2.1,
            half_extents: Vec3::new(1.25, 1.4, 5.5),
            com_height_offset: -0.40,
            suspension_rest_length: 0.45,
            suspension_stiffness: 120000.0,
            suspension_damping: 15000.0,
            wheel_radius: 0.52,
            max_steer_angle: 0.48,
            peak_engine_force: 18000.0,
            max_brake_force: 12000.0,
            handbrake_force: 16000.0,
            tire_friction_coeff: 0.88,
            cornering_stiffness: 85000.0,
            aero_drag_coeff: 0.65,
            rolling_resistance_coeff: 0.018,
        }
    }

    /// Commercial heavy freight hauler: maximum payload tonnage, heavy duty torque.
    pub fn heavy_truck() -> Self {
        Self {
            mass_kg: 14000.0,
            wheelbase: 6.2,
            track_width: 2.2,
            half_extents: Vec3::new(1.3, 1.6, 6.0),
            com_height_offset: -0.50,
            suspension_rest_length: 0.48,
            suspension_stiffness: 160000.0,
            suspension_damping: 22000.0,
            wheel_radius: 0.56,
            max_steer_angle: 0.44,
            peak_engine_force: 28000.0,
            max_brake_force: 18000.0,
            handbrake_force: 24000.0,
            tire_friction_coeff: 0.85,
            cornering_stiffness: 110000.0,
            aero_drag_coeff: 0.72,
            rolling_resistance_coeff: 0.020,
        }
    }
}

/// Dynamic simulation state of an individual wheel.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WheelState {
    pub index: WheelIndex,
    /// Attachment anchor position in chassis local space.
    pub local_anchor: Vec3,
    /// Current suspension spring compression in meters.
    pub compression: f32,
    /// Rate of compression change in m/s.
    pub compression_velocity: f32,
    /// Vertical normal force exerted by ground in Newtons.
    pub normal_force_fz: f32,
    /// Wheel steering angle in radians relative to chassis forward.
    pub steer_angle: f32,
    /// Angular rotation speed of wheel in radians/second.
    pub angular_velocity: f32,
    /// Lateral tire slip angle in radians.
    pub slip_angle: f32,
    /// Whether wheel tire is currently contacting ground surface.
    pub in_contact: bool,
    /// Longitudinal tractive/braking force in Newtons.
    pub longitudinal_force_fx: f32,
    /// Lateral cornering force in Newtons.
    pub lateral_force_fy: f32,
}

impl WheelState {
    pub fn new(index: WheelIndex, local_anchor: Vec3) -> Self {
        Self {
            index,
            local_anchor,
            compression: 0.0,
            compression_velocity: 0.0,
            normal_force_fz: 0.0,
            steer_angle: 0.0,
            angular_velocity: 0.0,
            slip_angle: 0.0,
            in_contact: false,
            longitudinal_force_fx: 0.0,
            lateral_force_fy: 0.0,
        }
    }
}

/// Complete state snapshot of a dynamic vehicle for networking, recording, and tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleStateSnapshot {
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub speed_kph: f32,
    pub engine_rpm: f32,
    pub current_gear: i8,
    pub is_grounded: bool,
    pub is_overturned: bool,
    pub wheel_compressions: [f32; 4],
    pub wheel_normal_forces: [f32; 4],
}

/// Autonomous, realistic 4-wheel vehicle physics simulation model.
pub struct DynamicVehicle {
    pub config: VehicleConfig,
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub wheels: [WheelState; 4],
    pub speed_kph: f32,
    pub engine_rpm: f32,
    pub current_gear: i8,
    pub is_grounded: bool,
    pub is_overturned: bool,
}

impl DynamicVehicle {
    /// Creates a new vehicle placed at `initial_position` with `initial_yaw`.
    pub fn new(config: VehicleConfig, initial_position: Vec3, initial_yaw: f32) -> Self {
        let half_w = config.track_width * 0.5;
        let half_l = config.wheelbase * 0.5;
        let anchor_y = config.com_height_offset;

        let wheels = [
            WheelState::new(WheelIndex::FrontLeft, Vec3::new(-half_w, anchor_y, half_l)),
            WheelState::new(WheelIndex::FrontRight, Vec3::new(half_w, anchor_y, half_l)),
            WheelState::new(WheelIndex::RearLeft, Vec3::new(-half_w, anchor_y, -half_l)),
            WheelState::new(WheelIndex::RearRight, Vec3::new(half_w, anchor_y, -half_l)),
        ];

        let orientation = Quat::from_rotation_y(initial_yaw);

        Self {
            config,
            position: initial_position,
            orientation,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            wheels,
            speed_kph: 0.0,
            engine_rpm: 800.0,
            current_gear: 1,
            is_grounded: false,
            is_overturned: false,
        }
    }

    /// Primary physics integration step advancing vehicle dynamics by `dt` seconds.
    pub fn step(&mut self, inputs: &VehicleInputs, ground: &dyn GroundQuery, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        let rot_mat = Mat3::from_quat(self.orientation);
        let forward = rot_mat.col(2); // Forward vector along Z
        let right = rot_mat.col(0); // Right vector along X
        let up = rot_mat.col(1); // Up vector along Y

        // Check if vehicle is overturned (chassis up vector points downwards)
        self.is_overturned = up.y < 0.2;

        // Update wheel steer angles (Ackermann front wheel steering)
        let steer_rad = inputs.steer * self.config.max_steer_angle;
        self.wheels[0].steer_angle = steer_rad;
        self.wheels[1].steer_angle = steer_rad;
        self.wheels[2].steer_angle = 0.0;
        self.wheels[3].steer_angle = 0.0;

        let mut total_force = Vec3::new(0.0, -9.81 * self.config.mass_kg, 0.0); // Gravity
        let mut total_torque = Vec3::ZERO;
        let mut grounded_wheels = 0;

        let max_ray_dist = self.config.suspension_rest_length + self.config.wheel_radius + 0.2;

        // Evaluate 4 independent suspension struts and tires
        for wheel in &mut self.wheels {
            let world_anchor = self.position + rot_mat * wheel.local_anchor;

            // Cast suspension ray downward in world space
            if let Some((hit_dist, normal)) =
                ground.query_ground_contact(world_anchor, max_ray_dist)
            {
                let uncompressed_len =
                    self.config.suspension_rest_length + self.config.wheel_radius;
                let current_compression = (uncompressed_len - hit_dist).max(0.0);
                let comp_vel = (current_compression - wheel.compression) / dt;

                wheel.compression = current_compression;
                wheel.compression_velocity = comp_vel;

                // Spring & damper force
                let spring_force = self.config.suspension_stiffness * current_compression;
                let damper_force = self.config.suspension_damping * comp_vel;
                let normal_force = (spring_force + damper_force).max(0.0);
                wheel.normal_force_fz = normal_force;
                wheel.in_contact = true;
                grounded_wheels += 1;

                // Wheel local coordinate frame
                let wheel_rot = Quat::from_axis_angle(up, wheel.steer_angle);
                let wheel_forward = wheel_rot * forward;
                let wheel_right = wheel_rot * right;

                // Linear velocity at wheel contact point (chassis linear + angular cross-product)
                let r_anchor = world_anchor - self.position;
                let contact_vel = self.linear_velocity + self.angular_velocity.cross(r_anchor);

                let v_forward = contact_vel.dot(wheel_forward);
                let v_lateral = contact_vel.dot(wheel_right);

                // Tire Slip Angle: continuous across all speeds without zero-slip deadzone
                wheel.slip_angle = if v_forward.abs() > 0.5 {
                    (v_lateral / v_forward.abs()).atan()
                } else {
                    (v_lateral / 0.5).clamp(-1.0, 1.0) * (std::f32::consts::FRAC_PI_4)
                };

                // Longitudinal Force (Traction & Braking)
                let mut f_long = 0.0;
                let compute_brake = |brake_ratio: f32, max_f: f32, vel: f32| -> f32 {
                    if brake_ratio <= 0.0 {
                        0.0
                    } else {
                        let force = brake_ratio * max_f;
                        let v_thresh = 0.4;
                        if vel.abs() < v_thresh {
                            force * (vel / v_thresh)
                        } else {
                            force * vel.signum()
                        }
                    }
                };

                match inputs.gear_mode {
                    TransmissionMode::Drive => {
                        let drive_mult = if wheel.index == WheelIndex::RearLeft
                            || wheel.index == WheelIndex::RearRight
                        {
                            0.6
                        } else {
                            0.4
                        };
                        f_long += inputs.throttle * self.config.peak_engine_force * drive_mult;
                        f_long -=
                            compute_brake(inputs.brake, self.config.max_brake_force, v_forward);
                    }
                    TransmissionMode::Reverse => {
                        f_long -= inputs.throttle * self.config.peak_engine_force * 0.5;
                        f_long -=
                            compute_brake(inputs.brake, self.config.max_brake_force, v_forward);
                    }
                    TransmissionMode::Neutral => {
                        f_long -=
                            compute_brake(inputs.brake, self.config.max_brake_force, v_forward);
                    }
                    TransmissionMode::Park => {
                        f_long -= compute_brake(1.0, 15000.0, v_forward);
                    }
                }

                // Handbrake on rear wheels
                if inputs.handbrake
                    && (wheel.index == WheelIndex::RearLeft || wheel.index == WheelIndex::RearRight)
                {
                    f_long -= compute_brake(1.0, self.config.handbrake_force, v_forward);
                }

                // Rolling resistance with smooth near-zero ramp
                let rolling_res = self.config.rolling_resistance_coeff
                    * normal_force
                    * (v_forward / 0.3).clamp(-1.0, 1.0);
                f_long -= rolling_res;

                // Lateral Cornering Force
                let f_lat = -self.config.cornering_stiffness * wheel.slip_angle;

                // Friction Circle limit: sqrt(Fx^2 + Fy^2) <= mu * Fz
                let max_tire_friction = self.config.tire_friction_coeff * normal_force;
                let current_tire_force_mag = (f_long * f_long + f_lat * f_lat).sqrt();

                let (final_fx, final_fy) = if current_tire_force_mag > max_tire_friction
                    && current_tire_force_mag > 1e-3
                {
                    let scale = max_tire_friction / current_tire_force_mag;
                    (f_long * scale, f_lat * scale)
                } else {
                    (f_long, f_lat)
                };

                wheel.longitudinal_force_fx = final_fx;
                wheel.lateral_force_fy = final_fy;

                // Total force from this wheel on the chassis
                let tire_force_world =
                    normal * normal_force + wheel_forward * final_fx + wheel_right * final_fy;
                total_force += tire_force_world;

                // Torque from this wheel around center of mass
                total_torque += r_anchor.cross(tire_force_world);
            } else {
                wheel.compression = 0.0;
                wheel.compression_velocity = 0.0;
                wheel.normal_force_fz = 0.0;
                wheel.in_contact = false;
                wheel.longitudinal_force_fx = 0.0;
                wheel.lateral_force_fy = 0.0;
            }
        }

        self.is_grounded = grounded_wheels >= 2;

        // Aerodynamic drag opposing velocity
        let speed = self.linear_velocity.length();
        if speed > 0.1 {
            let drag_force = -self.config.aero_drag_coeff * speed * self.linear_velocity;
            total_force += drag_force;
        }

        // Numerical Integration (Semi-Implicit Euler)
        let inv_mass = 1.0 / self.config.mass_kg;
        let acceleration = total_force * inv_mass;
        self.linear_velocity += acceleration * dt;

        // Hill-hold static lock when stationary on slope with brake applied
        if inputs.brake > 0.5 && inputs.throttle == 0.0 && self.linear_velocity.length() < 0.15 {
            self.linear_velocity = Vec3::ZERO;
        }

        // Moment of Inertia approximation for cuboid chassis: I = (1/12) * m * (d1^2 + d2^2)
        let dims = self.config.half_extents * 2.0;
        let ix = (1.0 / 12.0) * self.config.mass_kg * (dims.y * dims.y + dims.z * dims.z);
        let iy = (1.0 / 12.0) * self.config.mass_kg * (dims.x * dims.x + dims.z * dims.z);
        let iz = (1.0 / 12.0) * self.config.mass_kg * (dims.x * dims.x + dims.y * dims.y);
        let inv_inertia = Vec3::new(1.0 / ix, 1.0 / iy, 1.0 / iz);

        // Angular acceleration with damping
        let angular_accel = rot_mat * (inv_inertia * (rot_mat.transpose() * total_torque));
        self.angular_velocity += angular_accel * dt;
        self.angular_velocity *= 0.98; // Natural rotational damping

        // Update Position & Orientation
        self.position += self.linear_velocity * dt;

        // Anti-tunneling / anti-sinking chassis safety clamp
        if let Some((hit_dist, _)) = ground.query_ground_contact(self.position, 10.0) {
            let min_chassis_y = (self.position.y - hit_dist) + self.config.wheel_radius * 0.5;
            if self.position.y < min_chassis_y {
                self.position.y = min_chassis_y;
                if self.linear_velocity.y < 0.0 {
                    self.linear_velocity.y = 0.0;
                }
            }
        }

        let ang_vel_len = self.angular_velocity.length();
        if ang_vel_len > 1e-4 {
            let delta_rot =
                Quat::from_axis_angle(self.angular_velocity / ang_vel_len, ang_vel_len * dt);
            self.orientation = (delta_rot * self.orientation).normalize();
        }

        // Telemetry state updates
        let forward_speed_ms = self.linear_velocity.dot(forward);
        self.speed_kph = forward_speed_ms * 3.6;

        // Simulated Engine RPM calculation
        let abs_speed = self.speed_kph.abs();
        self.current_gear = if abs_speed < 20.0 {
            1
        } else if abs_speed < 45.0 {
            2
        } else if abs_speed < 75.0 {
            3
        } else if abs_speed < 110.0 {
            4
        } else {
            5
        };

        let target_rpm = 800.0 + (abs_speed * 45.0) + (inputs.throttle * 1200.0);
        self.engine_rpm = target_rpm.clamp(800.0, 6500.0);
    }

    /// Flip recovery: resets the vehicle orientation upright onto the nearest ground surface.
    pub fn recover_on_surface(&mut self, ground_elevation: f32) {
        // Extract current yaw angle
        let rot_mat = Mat3::from_quat(self.orientation);
        let forward = rot_mat.col(2);
        let current_yaw = forward.x.atan2(forward.z);

        self.orientation = Quat::from_rotation_y(current_yaw);
        self.position.y =
            ground_elevation + self.config.wheel_radius + self.config.suspension_rest_length + 0.1;
        self.linear_velocity = Vec3::ZERO;
        self.angular_velocity = Vec3::ZERO;
        self.is_overturned = false;
    }

    /// Captures a complete serializable state snapshot.
    pub fn snapshot(&self) -> VehicleStateSnapshot {
        VehicleStateSnapshot {
            position: self.position,
            orientation: self.orientation,
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            speed_kph: self.speed_kph,
            engine_rpm: self.engine_rpm,
            current_gear: self.current_gear,
            is_grounded: self.is_grounded,
            is_overturned: self.is_overturned,
            wheel_compressions: [
                self.wheels[0].compression,
                self.wheels[1].compression,
                self.wheels[2].compression,
                self.wheels[3].compression,
            ],
            wheel_normal_forces: [
                self.wheels[0].normal_force_fz,
                self.wheels[1].normal_force_fz,
                self.wheels[2].normal_force_fz,
                self.wheels[3].normal_force_fz,
            ],
        }
    }

    /// Restores vehicle dynamic state from a snapshot for deterministic rewinds.
    pub fn restore_snapshot(&mut self, snapshot: &VehicleStateSnapshot) {
        self.position = snapshot.position;
        self.orientation = snapshot.orientation;
        self.linear_velocity = snapshot.linear_velocity;
        self.angular_velocity = snapshot.angular_velocity;
        self.speed_kph = snapshot.speed_kph;
        self.engine_rpm = snapshot.engine_rpm;
        self.current_gear = snapshot.current_gear;
        self.is_grounded = snapshot.is_grounded;
        self.is_overturned = snapshot.is_overturned;
        for i in 0..4 {
            self.wheels[i].compression = snapshot.wheel_compressions[i];
            self.wheels[i].normal_force_fz = snapshot.wheel_normal_forces[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::{FlatGround, SlopedGround};

    #[test]
    fn test_vehicle_suspension_equilibrium_on_flat_ground() {
        let config = VehicleConfig::default();
        let ground = FlatGround::new(0.0);
        // Place vehicle at rest height
        let mut car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, 1.0, 0.0), 0.0);

        let inputs = VehicleInputs::default();

        // Step 120 frames at 60Hz (2.0 seconds) to settle into suspension equilibrium
        for _ in 0..120 {
            car.step(&inputs, &ground, 1.0 / 60.0);
        }

        assert!(car.is_grounded, "Car must be grounded in equilibrium");
        assert!(car.position.y > 0.0, "Car must not fall through ground");
        assert!(car.position.y < 1.0, "Car must settle onto suspension");

        // Total normal forces from 4 wheels should approximately equal m * g
        let total_normal: f32 = car.wheels.iter().map(|w| w.normal_force_fz).sum();
        let expected_weight = config.mass_kg * 9.81;
        assert!(
            (total_normal - expected_weight).abs() < expected_weight * 0.15,
            "Total suspension force ({:.1}N) should match vehicle weight ({:.1}N)",
            total_normal,
            expected_weight
        );
    }

    #[test]
    fn test_vehicle_acceleration_and_braking() {
        let config = VehicleConfig::default();
        let ground = FlatGround::new(0.0);
        let mut car = DynamicVehicle::new(config, Vec3::new(0.0, 0.7, 0.0), 0.0);

        // 1. Settle
        for _ in 0..60 {
            car.step(&VehicleInputs::default(), &ground, 1.0 / 60.0);
        }

        // 2. Full throttle for 2 seconds (120 frames)
        let throttle_input = VehicleInputs {
            throttle: 1.0,
            ..Default::default()
        };
        for _ in 0..120 {
            car.step(&throttle_input, &ground, 1.0 / 60.0);
        }

        let speed_after_accel = car.speed_kph;
        assert!(
            speed_after_accel > 30.0,
            "Car should accelerate past 30 km/h (was {:.1} km/h)",
            speed_after_accel
        );

        // 3. Full braking for 1.5 seconds (90 frames)
        let brake_input = VehicleInputs {
            brake: 1.0,
            ..Default::default()
        };
        for _ in 0..90 {
            car.step(&brake_input, &ground, 1.0 / 60.0);
        }

        let speed_after_brake = car.speed_kph;
        assert!(
            speed_after_brake < speed_after_accel * 0.4,
            "Car should decelerate significantly under heavy braking (was {:.1} km/h)",
            speed_after_brake
        );
    }

    #[test]
    fn test_slope_response_and_gravity() {
        let config = VehicleConfig::default();
        // Ground sloping upwards along Z (slope = 0.15 ~ 15% grade)
        let ground = SlopedGround::new(0.0, 0.0, 0.15);
        let mut car = DynamicVehicle::new(config, Vec3::new(0.0, 1.0, 0.0), 0.0);

        // Settle on slope without throttle or brakes
        for _ in 0..60 {
            car.step(&VehicleInputs::default(), &ground, 1.0 / 60.0);
        }

        assert!(car.is_grounded, "Car should contact sloped ground");
        // Gravitational pull along slope should induce backward/downward roll when in Neutral
        let neutral_input = VehicleInputs {
            gear_mode: TransmissionMode::Neutral,
            ..Default::default()
        };
        for _ in 0..60 {
            car.step(&neutral_input, &ground, 1.0 / 60.0);
        }

        // Vehicle should have negative Z velocity (rolling down the slope)
        assert!(
            car.linear_velocity.z < 0.0,
            "Car should roll down the slope in Neutral"
        );
    }

    #[test]
    fn test_flip_recovery() {
        let config = VehicleConfig::default();
        // Create upside-down car (pitch = 180 degrees)
        let mut car = DynamicVehicle::new(config, Vec3::new(0.0, 1.0, 0.0), 0.0);
        car.orientation = Quat::from_rotation_x(std::f32::consts::PI);
        car.is_overturned = true;

        assert!(car.is_overturned);

        // Execute recovery onto flat surface at elevation 0.0m
        car.recover_on_surface(0.0);

        assert!(!car.is_overturned, "Car should no longer be overturned");
        assert!(car.position.y > 0.0, "Car should be above ground");
    }
}
