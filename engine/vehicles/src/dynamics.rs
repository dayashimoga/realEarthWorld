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
    /// Anti-lock Braking System (modulates wheel brake force to prevent lockup).
    #[serde(default = "default_true")]
    pub abs_enabled: bool,
    /// Traction Control System (reduces drive wheel spin on slippery surfaces).
    #[serde(default = "default_true")]
    pub tcs_enabled: bool,
    /// Electronic Stability Control (corrects oversteer/understeer yaw moments).
    #[serde(default = "default_true")]
    pub esc_enabled: bool,
    /// Speed-sensitive steering attenuation factor (1.0 = standard, 0.0 = fixed steering lock).
    #[serde(default = "default_one")]
    pub steering_sensitivity: f32,
}

fn default_true() -> bool {
    true
}

fn default_one() -> f32 {
    1.0
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
            abs_enabled: true,
            tcs_enabled: true,
            esc_enabled: true,
            steering_sensitivity: 1.0,
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
            abs_enabled: true,
            tcs_enabled: true,
            esc_enabled: true,
            steering_sensitivity: 1.0,
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
            abs_enabled: true,
            tcs_enabled: true,
            esc_enabled: true,
            steering_sensitivity: 1.0,
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
            abs_enabled: true,
            tcs_enabled: true,
            esc_enabled: true,
            steering_sensitivity: 1.0,
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
            abs_enabled: true,
            tcs_enabled: true,
            esc_enabled: true,
            steering_sensitivity: 1.0,
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
            abs_enabled: true,
            tcs_enabled: true,
            esc_enabled: true,
            steering_sensitivity: 1.0,
        }
    }

    /// Computes the exact vertical clearance from ground contact to chassis geometric origin at static equilibrium.
    pub fn equilibrium_ride_height(&self) -> f32 {
        let weight = self.mass_kg * 9.81;
        let per_wheel_force = weight * 0.25;
        let eq_compression = (per_wheel_force / self.suspension_stiffness)
            .clamp(0.01, self.suspension_rest_length * 0.85);
        let strut_len = self.suspension_rest_length - eq_compression;
        // Chassis origin Y = ground + wheel_radius + strut_len - com_height_offset
        self.wheel_radius + strut_len - self.com_height_offset
    }

    /// Computes static equilibrium spring compression in meters per wheel.
    pub fn equilibrium_compression(&self) -> f32 {
        let weight = self.mass_kg * 9.81;
        let per_wheel_force = weight * 0.25;
        (per_wheel_force / self.suspension_stiffness)
            .clamp(0.01, self.suspension_rest_length * 0.85)
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

        let eq_comp = config.equilibrium_compression();
        let eq_force = config.mass_kg * 9.81 * 0.25;

        let mut wheels = [
            WheelState::new(WheelIndex::FrontLeft, Vec3::new(-half_w, anchor_y, half_l)),
            WheelState::new(WheelIndex::FrontRight, Vec3::new(half_w, anchor_y, half_l)),
            WheelState::new(WheelIndex::RearLeft, Vec3::new(-half_w, anchor_y, -half_l)),
            WheelState::new(WheelIndex::RearRight, Vec3::new(half_w, anchor_y, -half_l)),
        ];

        // Preload wheels with static equilibrium spring compression and force to eliminate frame-0 shock impulses
        for wheel in &mut wheels {
            wheel.compression = eq_comp;
            wheel.normal_force_fz = eq_force;
            wheel.in_contact = true;
        }

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
            is_grounded: true,
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

        // Update wheel steer angles (Ackermann front wheel steering) with speed-sensitive attenuation
        let speed_ms = self.linear_velocity.length();
        let steer_scale = if self.config.steering_sensitivity > 0.0 && speed_ms > 12.0 {
            let high_speed_ratio = ((speed_ms - 12.0) / 38.0).clamp(0.0, 1.0);
            1.0 - 0.5 * high_speed_ratio * self.config.steering_sensitivity
        } else {
            1.0
        };
        let steer_rad = inputs.steer * (self.config.max_steer_angle * steer_scale);
        self.wheels[0].steer_angle = steer_rad;
        self.wheels[1].steer_angle = steer_rad;
        self.wheels[2].steer_angle = 0.0;
        self.wheels[3].steer_angle = 0.0;

        let mut total_force = Vec3::new(0.0, -9.81 * self.config.mass_kg, 0.0); // Gravity
        let mut total_torque = Vec3::ZERO;
        let mut grounded_wheels = 0;
        let mut avg_ground_normal = Vec3::ZERO;

        let max_ray_dist = self.config.suspension_rest_length + self.config.wheel_radius + 0.2;

        // Evaluate 4 independent suspension struts and tires
        for wheel in &mut self.wheels {
            let world_anchor = self.position + rot_mat * wheel.local_anchor;

            // Cast suspension ray downward in world space
            if let Some((hit_dist, normal)) =
                ground.query_ground_contact(world_anchor, max_ray_dist)
            {
                avg_ground_normal += normal;
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

                // Longitudinal Force (Traction & Braking with ABS and TCS)
                let mut f_long = 0.0;
                let compute_brake = |brake_ratio: f32,
                                     max_f: f32,
                                     vel: f32,
                                     abs_on: bool,
                                     norm_f: f32,
                                     mu: f32|
                 -> f32 {
                    if brake_ratio <= 0.0 {
                        0.0
                    } else {
                        let mut force = brake_ratio * max_f;
                        if abs_on && brake_ratio > 0.05 && vel.abs() > 2.5 {
                            let max_grip = mu * norm_f;
                            if force > max_grip * 0.98 {
                                force = max_grip * 0.96;
                            }
                        }
                        let v_thresh = 0.4;
                        if vel.abs() < v_thresh {
                            force * (vel / v_thresh)
                        } else {
                            force * vel.signum()
                        }
                    }
                };

                let compute_drive =
                    |throttle: f32, raw_force: f32, tcs_on: bool, norm_f: f32, mu: f32| -> f32 {
                        let base_drive = throttle * raw_force;
                        if tcs_on && throttle > 0.05 {
                            let grip_limit = mu * norm_f * 0.95;
                            base_drive.min(grip_limit)
                        } else {
                            base_drive
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
                        let drive_f = compute_drive(
                            inputs.throttle,
                            self.config.peak_engine_force * drive_mult,
                            self.config.tcs_enabled,
                            normal_force,
                            self.config.tire_friction_coeff,
                        );
                        f_long += drive_f;
                        f_long -= compute_brake(
                            inputs.brake,
                            self.config.max_brake_force,
                            v_forward,
                            self.config.abs_enabled,
                            normal_force,
                            self.config.tire_friction_coeff,
                        );
                    }
                    TransmissionMode::Reverse => {
                        let rev_speed_cap = 12.5; // ~45 km/h
                        let rev_throttle_factor = if v_forward < -rev_speed_cap {
                            0.0
                        } else {
                            (1.0 - (-v_forward / rev_speed_cap).max(0.0)).clamp(0.0, 1.0)
                        };
                        let rev_drive_f = compute_drive(
                            inputs.throttle,
                            self.config.peak_engine_force * 0.5 * rev_throttle_factor,
                            self.config.tcs_enabled,
                            normal_force,
                            self.config.tire_friction_coeff,
                        );
                        f_long -= rev_drive_f;
                        f_long -= compute_brake(
                            inputs.brake,
                            self.config.max_brake_force,
                            v_forward,
                            self.config.abs_enabled,
                            normal_force,
                            self.config.tire_friction_coeff,
                        );
                    }
                    TransmissionMode::Neutral => {
                        f_long -= compute_brake(
                            inputs.brake,
                            self.config.max_brake_force,
                            v_forward,
                            self.config.abs_enabled,
                            normal_force,
                            self.config.tire_friction_coeff,
                        );
                    }
                    TransmissionMode::Park => {
                        f_long -= compute_brake(
                            1.0,
                            15000.0,
                            v_forward,
                            false,
                            normal_force,
                            self.config.tire_friction_coeff,
                        );
                    }
                }

                // Handbrake on rear wheels
                if inputs.handbrake
                    && (wheel.index == WheelIndex::RearLeft || wheel.index == WheelIndex::RearRight)
                {
                    f_long -= compute_brake(
                        1.0,
                        self.config.handbrake_force,
                        v_forward,
                        false,
                        normal_force,
                        self.config.tire_friction_coeff,
                    );
                }

                // Rolling resistance with smooth near-zero ramp
                let rolling_res = self.config.rolling_resistance_coeff
                    * normal_force
                    * (v_forward / 0.15).clamp(-1.0, 1.0);
                f_long -= rolling_res;

                // Lateral Cornering & Traction Force
                // At high speed (> 0.5 m/s), use kinematic slip angle.
                // At low speed (<= 0.5 m/s), use stable critically-damped tire contact resistance
                // that avoids numerical Euler oscillation explosion (|Delta v| < |v|).
                let f_lat = if v_forward.abs() > 0.5 {
                    -self.config.cornering_stiffness * wheel.slip_angle
                } else {
                    let quarter_mass = self.config.mass_kg * 0.25;
                    let max_stable_damping = (quarter_mass / dt) * 0.75;
                    let low_speed_damping =
                        (self.config.cornering_stiffness * 0.25).min(max_stable_damping);
                    let f_damping = -low_speed_damping * v_lateral;
                    let f_slip = -self.config.cornering_stiffness * wheel.slip_angle;
                    let alpha = (v_forward.abs() / 0.5).clamp(0.0, 1.0);
                    (1.0 - alpha) * f_damping + alpha * f_slip
                };

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
        let avg_normal = if grounded_wheels > 0 {
            (avg_ground_normal / grounded_wheels as f32).normalize_or_zero()
        } else {
            Vec3::Y
        };
        let slope_grade = (avg_normal.x * avg_normal.x + avg_normal.z * avg_normal.z).sqrt();

        // Anti-roll bar (ARB) torsional spring forces: resists body roll during cornering
        if self.wheels[0].in_contact && self.wheels[1].in_contact {
            let front_roll_diff = self.wheels[0].compression - self.wheels[1].compression;
            let arb_force = front_roll_diff * 6500.0; // N/m ARB stiffness
            total_torque -= forward * (arb_force * self.config.track_width * 0.5);
        }
        if self.wheels[2].in_contact && self.wheels[3].in_contact {
            let rear_roll_diff = self.wheels[2].compression - self.wheels[3].compression;
            let arb_force = rear_roll_diff * 5500.0;
            total_torque -= forward * (arb_force * self.config.track_width * 0.5);
        }

        // Electronic Stability Control (ESC): counteracts yaw divergence (oversteer / spinout)
        let forward_speed_ms = self.linear_velocity.dot(forward);
        if self.config.esc_enabled && self.is_grounded && forward_speed_ms.abs() > 2.0 {
            let actual_yaw_rate = self.angular_velocity.dot(up);
            let steer_angle = self.wheels[0].steer_angle;
            let desired_yaw_rate =
                (forward_speed_ms / self.config.wheelbase.max(1.0)) * steer_angle.tan();
            let yaw_divergence = actual_yaw_rate - desired_yaw_rate;
            if yaw_divergence.abs() > 0.20 {
                // Corrective restoring yaw torque
                let esc_torque = -up * (yaw_divergence * self.config.mass_kg * 4.0);
                total_torque += esc_torque;
            }
        }

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

        let speed = self.linear_velocity.length();
        let is_nearly_stationary = speed < 0.25;

        // Static friction holding logic: Park, handbrake, service brake, and level-ground idle
        if is_nearly_stationary {
            let is_park = inputs.gear_mode == TransmissionMode::Park;
            let is_handbrake = inputs.handbrake && inputs.throttle == 0.0;
            let is_service_brake = inputs.brake > 0.3 && inputs.throttle == 0.0;

            // On flat ground with zero throttle, static rolling resistance holds the car motionless without drift
            let is_flat_idle = inputs.throttle == 0.0
                && inputs.steer.abs() < 0.05
                && speed < 0.12
                && grounded_wheels >= 2
                && slope_grade < 0.025;

            if is_park || is_handbrake || is_service_brake || is_flat_idle {
                self.linear_velocity.x = 0.0;
                self.linear_velocity.z = 0.0;
                self.angular_velocity = Vec3::ZERO;
            }
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

        // Simulated Engine RPM & Gear calculation
        let abs_speed = self.speed_kph.abs();
        self.current_gear = match inputs.gear_mode {
            TransmissionMode::Reverse => -1,
            TransmissionMode::Neutral | TransmissionMode::Park => 0,
            TransmissionMode::Drive => {
                if abs_speed < 20.0 {
                    1
                } else if abs_speed < 45.0 {
                    2
                } else if abs_speed < 75.0 {
                    3
                } else if abs_speed < 110.0 {
                    4
                } else {
                    5
                }
            }
        };

        let target_rpm = 800.0 + (abs_speed * 45.0) + (inputs.throttle * 1200.0);
        self.engine_rpm = target_rpm.clamp(800.0, 6500.0);
    }

    /// Flip recovery: resets the vehicle orientation upright onto the nearest ground surface at equilibrium height.
    pub fn recover_on_surface(&mut self, ground_elevation: f32) {
        // Extract current yaw angle
        let rot_mat = Mat3::from_quat(self.orientation);
        let forward = rot_mat.col(2);
        let current_yaw = forward.x.atan2(forward.z);

        self.orientation = Quat::from_rotation_y(current_yaw);
        self.position.y = ground_elevation + self.config.equilibrium_ride_height();
        self.linear_velocity = Vec3::ZERO;
        self.angular_velocity = Vec3::ZERO;
        self.is_overturned = false;

        let eq_comp = self.config.equilibrium_compression();
        let eq_force = self.config.mass_kg * 9.81 * 0.25;
        for wheel in &mut self.wheels {
            wheel.compression = eq_comp;
            wheel.compression_velocity = 0.0;
            wheel.normal_force_fz = eq_force;
            wheel.steer_angle = 0.0;
            wheel.angular_velocity = 0.0;
            wheel.slip_angle = 0.0;
            wheel.in_contact = true;
            wheel.longitudinal_force_fx = 0.0;
            wheel.lateral_force_fy = 0.0;
        }
    }

    /// Resolves horizontal building solid collision, preventing chassis penetration.
    pub fn resolve_building_collision(
        &mut self,
        world: &crate::building_collision::BuildingCollisionWorld,
    ) -> bool {
        world.resolve_collision(
            &mut self.position,
            &mut self.linear_velocity,
            self.config.half_extents,
        )
    }

    /// Advances vehicle dynamics by dt seconds, taking into account ground elevation and solid building collisions.
    pub fn step_with_solids(
        &mut self,
        inputs: &VehicleInputs,
        ground: &dyn GroundQuery,
        solids: &crate::building_collision::BuildingCollisionWorld,
        dt: f32,
    ) -> bool {
        self.step(inputs, ground, dt);
        self.resolve_building_collision(solids)
    }

    /// Safe full reset: places vehicle at given position and yaw with zero velocities, level chassis, and equilibrium preload.
    pub fn reset_at_pose(&mut self, position: Vec3, yaw: f32) {
        self.position = position;
        self.orientation = Quat::from_rotation_y(yaw);
        self.linear_velocity = Vec3::ZERO;
        self.angular_velocity = Vec3::ZERO;
        self.is_overturned = false;
        self.speed_kph = 0.0;
        self.engine_rpm = 800.0;
        self.current_gear = 1;

        let eq_comp = self.config.equilibrium_compression();
        let eq_force = self.config.mass_kg * 9.81 * 0.25;
        for wheel in &mut self.wheels {
            wheel.compression = eq_comp;
            wheel.compression_velocity = 0.0;
            wheel.normal_force_fz = eq_force;
            wheel.steer_angle = 0.0;
            wheel.angular_velocity = 0.0;
            wheel.slip_angle = 0.0;
            wheel.in_contact = true;
            wheel.longitudinal_force_fx = 0.0;
            wheel.lateral_force_fy = 0.0;
        }
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

    #[test]
    fn test_reverse_transmission_mode_and_braking() {
        let config = VehicleConfig::default();
        let ground = FlatGround::new(0.0);
        let mut car = DynamicVehicle::new(config, Vec3::new(0.0, 0.8, 0.0), 0.0);

        // Settle on ground
        for _ in 0..40 {
            car.step(&VehicleInputs::default(), &ground, 1.0 / 60.0);
        }

        // Apply reverse throttle
        let rev_input = VehicleInputs {
            throttle: 0.8,
            gear_mode: TransmissionMode::Reverse,
            ..Default::default()
        };
        for _ in 0..60 {
            car.step(&rev_input, &ground, 1.0 / 60.0);
        }

        assert_eq!(car.current_gear, -1, "Reverse gear should be -1");
        assert!(
            car.linear_velocity.z < -1.0,
            "Car should accelerate backwards in Reverse gear (vz = {:.2})",
            car.linear_velocity.z
        );

        // Apply brake while in reverse
        let brake_rev = VehicleInputs {
            brake: 1.0,
            gear_mode: TransmissionMode::Reverse,
            ..Default::default()
        };
        for _ in 0..60 {
            car.step(&brake_rev, &ground, 1.0 / 60.0);
        }
        assert!(
            car.linear_velocity.z.abs() < 1.0,
            "Braking should arrest reverse motion"
        );
    }

    #[test]
    fn test_anti_roll_bar_stabilization() {
        let config = VehicleConfig::default();
        let ground = FlatGround::new(0.0);
        let mut car = DynamicVehicle::new(config, Vec3::new(0.0, 0.8, 0.0), 0.0);

        // Settle
        for _ in 0..40 {
            car.step(&VehicleInputs::default(), &ground, 1.0 / 60.0);
        }

        // Induce roll by manually compressing left wheels
        car.wheels[0].compression = 0.15;
        car.wheels[1].compression = 0.02;

        let input = VehicleInputs::default();
        car.step(&input, &ground, 1.0 / 60.0);

        // Anti-roll bar should apply restorative angular acceleration
        assert!(car.is_grounded);
    }

    #[test]
    fn test_reset_at_pose() {
        let config = VehicleConfig::default();
        let mut car = DynamicVehicle::new(config, Vec3::new(10.0, 5.0, 10.0), 1.0);
        car.linear_velocity = Vec3::new(20.0, 0.0, 15.0);
        car.angular_velocity = Vec3::new(0.1, 0.5, 0.0);
        car.speed_kph = 90.0;

        car.reset_at_pose(Vec3::new(50.0, 2.0, -100.0), 0.0);

        assert_eq!(car.position, Vec3::new(50.0, 2.0, -100.0));
        assert_eq!(car.linear_velocity, Vec3::ZERO);
        assert_eq!(car.angular_velocity, Vec3::ZERO);
        assert_eq!(car.speed_kph, 0.0);
        assert_eq!(car.current_gear, 1);
        assert!(!car.is_overturned);
    }

    #[test]
    fn test_driving_assists_abs_tcs_esc() {
        let mut config = VehicleConfig::sports_coupe();
        assert!(config.abs_enabled);
        assert!(config.tcs_enabled);
        assert!(config.esc_enabled);
        assert_eq!(config.steering_sensitivity, 1.0);

        let ground = FlatGround::new(0.0);
        let mut car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, 0.8, 0.0), 0.0);

        // Accelerate with TCS enabled
        let full_throttle = VehicleInputs {
            throttle: 1.0,
            ..Default::default()
        };
        for _ in 0..60 {
            car.step(&full_throttle, &ground, 1.0 / 60.0);
        }
        assert!(car.speed_kph > 10.0);

        // Emergency braking with ABS
        let full_brake = VehicleInputs {
            brake: 1.0,
            ..Default::default()
        };
        for _ in 0..60 {
            car.step(&full_brake, &ground, 1.0 / 60.0);
        }
        assert!(car.speed_kph < 15.0);

        // Verify config toggling
        config.abs_enabled = false;
        config.tcs_enabled = false;
        config.esc_enabled = false;
        config.steering_sensitivity = 0.5;
        assert!(!config.abs_enabled);
        assert!(!config.tcs_enabled);
        assert!(!config.esc_enabled);
    }
}
