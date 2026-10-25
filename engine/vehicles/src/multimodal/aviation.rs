//! 6-DOF aerodynamic aircraft flight simulation, atmospheric density lapse,
//! lift/drag polar, control surfaces, stall modeling, ground effect,
//! and tricycle landing gear oleo strut dynamics.
//!
//! Maintains independent flight mechanics strictly decoupled from road friction models.

use crate::multimodal::MultimodalAudioEvent;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Classification of aircraft airframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AviationCategory {
    /// Commercial passenger airliner (e.g. A320 / B737).
    CommercialAirliner,
    /// Single-engine light aircraft (e.g. Cessna 172).
    GeneralAviation,
    /// High-performance twin-engine business jet.
    BusinessJet,
}

/// Dynamic flight control inputs from pilot / autopilot.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FlightInputs {
    /// Longitudinal pitch control stick / yoke [-1.0, 1.0] (negative = pitch down, positive = pitch up).
    pub elevator: f32,
    /// Lateral roll control stick / yoke [-1.0, 1.0] (negative = roll left, positive = roll right).
    pub aileron: f32,
    /// Directional yaw rudder pedals [-1.0, 1.0] (negative = nose left, positive = nose right).
    pub rudder: f32,
    /// Engine throttle command [0.0, 1.0].
    pub throttle: f32,
    /// Wing trailing-edge flap deployment ratio [0.0, 1.0].
    pub flaps: f32,
    /// Flight spoiler / speedbrake deployment [0.0, 1.0].
    pub spoilers: f32,
    /// Wheel brakes application on landing gear [0.0, 1.0].
    pub wheel_brakes: f32,
    /// Reverse thrust engagement on jet engines.
    pub reverse_thrust: bool,
    /// Landing gear deployment state.
    pub gear_down: bool,
}

/// 6-DOF aerodynamic aircraft simulation model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AircraftFlight {
    pub name: String,
    pub category: AviationCategory,

    // Spatial & rigid-body states in world coordinates
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,

    // Airframe physical dimensions & mass
    /// Gross aircraft mass in kilograms.
    pub mass_kg: f32,
    /// Wing reference planform area in m^2.
    pub wing_area_m2: f32,
    /// Wing total span in meters.
    pub wingspan_meters: f32,
    /// Mean aerodynamic chord (MAC) in meters.
    pub mean_chord_meters: f32,
    /// Wing aspect ratio: AR = b^2 / S.
    pub aspect_ratio: f32,
    /// Oswald wing efficiency span factor (typically 0.75 - 0.85).
    pub oswald_efficiency: f32,
    /// Maximum sea-level engine thrust in Newtons.
    pub max_thrust_newtons: f32,

    // Aerodynamic coefficients
    pub cl_zero: f32,
    pub cl_alpha: f32,
    pub cd_zero: f32,
    pub stall_angle_rad: f32,

    // Flight telemetry
    pub true_airspeed_mps: f32,
    pub indicated_airspeed_knots: f32,
    pub altitude_msl_meters: f32,
    pub climb_rate_mps: f32,
    pub angle_of_attack_rad: f32,
    pub sideslip_angle_rad: f32,
    pub is_stalled: bool,
    pub in_ground_effect: bool,
    pub is_airborne: bool,
    pub gear_compression_meters: f32,
    /// Pending audio events emitted during simulation.
    pub audio_events: Vec<MultimodalAudioEvent>,
}

impl AircraftFlight {
    /// Creates a twin-engine commercial airliner (A320/B737 class).
    pub fn new_airliner(name: &str, position: Vec3, heading_radians: f32) -> Self {
        let wing_area = 122.6;
        let wingspan = 34.1;
        let mean_chord = 4.19;
        let aspect_ratio = (wingspan * wingspan) / wing_area;

        Self {
            name: name.to_string(),
            category: AviationCategory::CommercialAirliner,
            position,
            orientation: Quat::from_rotation_y(heading_radians),
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass_kg: 68_000.0, // 68 metric tons
            wing_area_m2: wing_area,
            wingspan_meters: wingspan,
            mean_chord_meters: mean_chord,
            aspect_ratio,
            oswald_efficiency: 0.82,
            max_thrust_newtons: 240_000.0, // 2x 120 kN CFM56/LEAP
            cl_zero: 0.24,
            cl_alpha: 5.5, // per radian (~0.096 per deg)
            cd_zero: 0.020,
            stall_angle_rad: 15.0_f32.to_radians(),
            true_airspeed_mps: 0.0,
            indicated_airspeed_knots: 0.0,
            altitude_msl_meters: position.y,
            climb_rate_mps: 0.0,
            angle_of_attack_rad: 0.0,
            sideslip_angle_rad: 0.0,
            is_stalled: false,
            in_ground_effect: true,
            is_airborne: false,
            gear_compression_meters: 0.0,
            audio_events: Vec::new(),
        }
    }

    /// Creates a single-engine general aviation aircraft (Cessna 172 class).
    pub fn new_general_aviation(name: &str, position: Vec3, heading_radians: f32) -> Self {
        let wing_area = 16.2;
        let wingspan = 11.0;
        let mean_chord = 1.47;
        let aspect_ratio = (wingspan * wingspan) / wing_area;

        Self {
            name: name.to_string(),
            category: AviationCategory::GeneralAviation,
            position,
            orientation: Quat::from_rotation_y(heading_radians),
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass_kg: 1_150.0,
            wing_area_m2: wing_area,
            wingspan_meters: wingspan,
            mean_chord_meters: mean_chord,
            aspect_ratio,
            oswald_efficiency: 0.78,
            max_thrust_newtons: 3_200.0,
            cl_zero: 0.31,
            cl_alpha: 4.8,
            cd_zero: 0.027,
            stall_angle_rad: 14.0_f32.to_radians(),
            true_airspeed_mps: 0.0,
            indicated_airspeed_knots: 0.0,
            altitude_msl_meters: position.y,
            climb_rate_mps: 0.0,
            angle_of_attack_rad: 0.0,
            sideslip_angle_rad: 0.0,
            is_stalled: false,
            in_ground_effect: true,
            is_airborne: false,
            gear_compression_meters: 0.0,
            audio_events: Vec::new(),
        }
    }

    /// Computes atmospheric air density at current geometric altitude using barometric lapse rate.
    pub fn atmospheric_density(&self) -> f32 {
        let sea_level_density = 1.225; // kg/m^3
        let scale_height = 8500.0; // meters
        let altitude = self.position.y.max(0.0);
        sea_level_density * (-altitude / scale_height).exp()
    }

    /// Advances the 6-DOF aerodynamic simulation by `dt` seconds.
    pub fn step(&mut self, inputs: &FlightInputs, ground_elevation: f32, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        self.altitude_msl_meters = self.position.y;
        self.climb_rate_mps = self.linear_velocity.y;

        let rho = self.atmospheric_density();
        let sea_level_rho = 1.225;

        // 1. Airframe Orientation Vectors
        let forward_dir = self.orientation * Vec3::Z; // Aircraft longitudinal X/Z axis
        let up_dir = self.orientation * Vec3::Y;
        let right_dir = self.orientation * Vec3::X;

        // Transform world velocity into aircraft body frame
        let inv_orient = self.orientation.conjugate();
        let body_velocity = inv_orient * self.linear_velocity;

        let tas = self.linear_velocity.length();
        self.true_airspeed_mps = tas;
        self.indicated_airspeed_knots = (tas * (rho / sea_level_rho).sqrt()) * 1.94384;

        // Dynamic pressure q = 0.5 * rho * V^2
        let dynamic_pressure = 0.5 * rho * tas * tas;

        // Angle of Attack (alpha) & Sideslip (beta)
        if body_velocity.z.abs() > 0.5 {
            self.angle_of_attack_rad = (-body_velocity.y).atan2(body_velocity.z);
            self.sideslip_angle_rad = (body_velocity.x / tas.max(1.0)).asin();
        } else {
            self.angle_of_attack_rad = 0.0;
            self.sideslip_angle_rad = 0.0;
        }

        // 2. Aerodynamic Lift & Stall Modeling
        let flap_lift_bonus = inputs.flaps.clamp(0.0, 1.0) * 0.65;
        let effective_stall_angle = self.stall_angle_rad + (inputs.flaps * 2.0_f32.to_radians());

        let cl = if self.angle_of_attack_rad.abs() <= effective_stall_angle {
            self.is_stalled = false;
            self.cl_zero + self.cl_alpha * self.angle_of_attack_rad + flap_lift_bonus
        } else {
            // Post-stall flow separation: lift drops precipitously
            if !self.is_stalled {
                self.audio_events
                    .push(MultimodalAudioEvent::AircraftStallWarning);
            }
            self.is_stalled = true;
            let stall_sign = self.angle_of_attack_rad.signum();
            let max_cl = self.cl_zero + self.cl_alpha * effective_stall_angle + flap_lift_bonus;
            max_cl * 0.45 * stall_sign
        };

        // Ground effect factor: reduces induced drag when close to ground
        let height_above_ground = (self.position.y - ground_elevation).max(0.1);
        let h_over_b = height_above_ground / self.wingspan_meters;
        let ground_effect_factor = if h_over_b < 1.0 {
            self.in_ground_effect = true;
            (16.0 * h_over_b * h_over_b) / (1.0 + 16.0 * h_over_b * h_over_b)
        } else {
            self.in_ground_effect = false;
            1.0
        };

        // 3. Aerodynamic Drag Polar
        let induced_drag_coeff = (cl * cl)
            / (std::f32::consts::PI * self.oswald_efficiency * self.aspect_ratio)
            * ground_effect_factor;
        let gear_drag = if inputs.gear_down { 0.018 } else { 0.0 };
        let flap_drag = inputs.flaps * 0.035;
        let spoiler_drag = inputs.spoilers * 0.055;
        let cd = self.cd_zero + induced_drag_coeff + gear_drag + flap_drag + spoiler_drag;

        let lift_force_n = dynamic_pressure * self.wing_area_m2 * cl;
        let drag_force_n = dynamic_pressure * self.wing_area_m2 * cd;

        // Lift acts perpendicular to relative velocity in pitch plane, Drag acts anti-parallel to velocity
        let lift_vector = if tas > 1.0 {
            let vel_normalized = self.linear_velocity / tas;
            let lift_direction = vel_normalized.cross(right_dir).normalize_or_zero();
            lift_direction * lift_force_n
        } else {
            up_dir * lift_force_n
        };

        let drag_vector = if tas > 0.1 {
            -(self.linear_velocity / tas) * drag_force_n
        } else {
            Vec3::ZERO
        };

        // 4. Propulsion Thrust (Turbofan / Propeller)
        let density_ratio = (rho / sea_level_rho).powf(0.7);
        let engine_thrust_n = if inputs.reverse_thrust {
            -self.max_thrust_newtons * 0.45 * inputs.throttle.clamp(0.0, 1.0) * density_ratio
        } else {
            self.max_thrust_newtons * inputs.throttle.clamp(0.0, 1.0) * density_ratio
        };
        let thrust_vector = forward_dir * engine_thrust_n;

        // 5. Gravity
        let gravity_vector = Vec3::new(0.0, -self.mass_kg * 9.81, 0.0);

        // 6. Tricycle Landing Gear & Ground Contact Simulation
        let mut gear_normal_force = Vec3::ZERO;
        let mut gear_friction_force = Vec3::ZERO;
        let static_gear_height = 2.4; // Landing gear neutral height below aircraft CG

        let dist_to_ground = self.position.y - ground_elevation;
        let was_airborne = self.is_airborne;
        if dist_to_ground <= static_gear_height && inputs.gear_down {
            self.is_airborne = false;
            if was_airborne && self.linear_velocity.y < -0.5 {
                self.audio_events
                    .push(MultimodalAudioEvent::AircraftGearTouchdown {
                        descent_mps: -self.linear_velocity.y,
                    });
            }
            let penetration = static_gear_height - dist_to_ground;
            self.gear_compression_meters = penetration.clamp(0.0, 0.8);

            // Oleo strut spring-damper reaction
            let strut_spring_k = (self.mass_kg * 9.81) / 0.35; // Stiff enough to support static mass
            let strut_damper_c = 2.0 * (strut_spring_k * self.mass_kg).sqrt() * 0.7; // Critically damped

            let spring_force = strut_spring_k * self.gear_compression_meters;
            let damping_force = -strut_damper_c * self.linear_velocity.y;
            let total_gear_reaction = (spring_force + damping_force).max(0.0);

            gear_normal_force = Vec3::new(0.0, total_gear_reaction, 0.0);

            // Wheel rolling and braking friction
            let brake_mu = 0.02 + inputs.wheel_brakes.clamp(0.0, 1.0) * 0.45;
            let ground_speed = Vec3::new(self.linear_velocity.x, 0.0, self.linear_velocity.z);
            if ground_speed.length() > 0.05 {
                let brake_n = total_gear_reaction * brake_mu;
                gear_friction_force = -ground_speed.normalize() * brake_n;
            }
        } else {
            self.is_airborne = true;
            self.gear_compression_meters = 0.0;
        }

        // Net Total Forces & Linear Acceleration
        let net_force = lift_vector
            + drag_vector
            + thrust_vector
            + gravity_vector
            + gear_normal_force
            + gear_friction_force;
        let linear_accel = net_force / self.mass_kg;

        self.linear_velocity += linear_accel * dt;
        self.position += self.linear_velocity * dt;

        // Prevent terrain penetration if on ground
        if self.position.y < (ground_elevation + static_gear_height - 0.8) {
            self.position.y = ground_elevation + static_gear_height - 0.8;
            if self.linear_velocity.y < 0.0 {
                self.linear_velocity.y = 0.0;
            }
        }

        // 7. Angular Dynamics & Control Surface Moments
        let q_s = dynamic_pressure * self.wing_area_m2;
        let elevator_authority = inputs.elevator.clamp(-1.0, 1.0);
        let aileron_authority = inputs.aileron.clamp(-1.0, 1.0);
        let rudder_authority = inputs.rudder.clamp(-1.0, 1.0);

        // Control moments (pitch, roll, yaw)
        let pitch_moment = -q_s * self.mean_chord_meters * (0.85 * elevator_authority);
        let roll_moment = -q_s * self.wingspan_meters * (0.35 * aileron_authority);
        let yaw_moment = q_s * self.wingspan_meters * (0.22 * rudder_authority);

        // Rotational inertia approximations for transport category
        let i_xx = self.mass_kg * self.wingspan_meters * self.wingspan_meters / 18.0;
        let i_yy = self.mass_kg * self.mean_chord_meters * 12.0 * 12.0 / 12.0;
        let i_zz = i_xx + i_yy;

        // Aerodynamic damping rates
        let roll_damping = -i_xx * 2.8 * self.angular_velocity.z;
        let pitch_damping = -i_yy * 2.5 * self.angular_velocity.x;
        let yaw_damping = -i_zz * 1.9 * self.angular_velocity.y;

        let roll_accel = (roll_moment + roll_damping) / i_xx;
        let pitch_accel = (pitch_moment + pitch_damping) / i_yy;
        let yaw_accel = (yaw_moment + yaw_damping) / i_zz;

        // Integrate angular velocities
        self.angular_velocity.x += pitch_accel * dt;
        self.angular_velocity.y += yaw_accel * dt;
        self.angular_velocity.z += roll_accel * dt;

        // Update attitude quaternion
        let rot_delta = Quat::from_scaled_axis(
            up_dir * (self.angular_velocity.y * dt)
                + right_dir * (self.angular_velocity.x * dt)
                + forward_dir * (self.angular_velocity.z * dt),
        );
        self.orientation = (rot_delta * self.orientation).normalize();
    }

    /// Computes instrument approach guidance (localizer lateral deviation in meters and glideslope vertical deviation in meters)
    /// towards a runway threshold with a 3.0-degree glidepath.
    pub fn compute_approach_guidance(
        &self,
        threshold_pos: Vec3,
        runway_heading_rad: f32,
    ) -> (f32, f32) {
        let delta = self.position - threshold_pos;
        let rwy_forward = Vec3::new(runway_heading_rad.sin(), 0.0, runway_heading_rad.cos());
        let rwy_right = Vec3::new(runway_heading_rad.cos(), 0.0, -runway_heading_rad.sin());

        let dist_along_centerline = -delta.dot(rwy_forward);
        let localizer_dev_meters = delta.dot(rwy_right);

        let glidepath_angle_rad = 3.0_f32.to_radians();
        let target_altitude = dist_along_centerline.max(0.0) * glidepath_angle_rad.tan();
        let glideslope_dev_meters = self.position.y - (threshold_pos.y + target_altitude);

        (localizer_dev_meters, glideslope_dev_meters)
    }

    /// Enforces authentic regional airspace boundaries (e.g. 2400m radius from package anchor).
    ///
    /// If aircraft approaches the boundary:
    /// - At warning threshold (> 2160m): triggers turnback vectoring.
    /// - At outer boundary (>= 2400m): vectors velocity toward center and clamps horizontal position.
    pub fn enforce_airspace_boundary(&mut self, max_radius_m: f32) -> bool {
        let horiz_dist = (self.position.x * self.position.x + self.position.z * self.position.z).sqrt();
        if horiz_dist > max_radius_m * 0.90 {
            let dir_to_center = Vec3::new(-self.position.x, 0.0, -self.position.z).normalize_or_zero();
            let forward = self.orientation * Vec3::Z;
            let dot = forward.dot(dir_to_center);

            if dot < 0.3 {
                let speed = self.linear_velocity.length();
                let blend = ((horiz_dist - max_radius_m * 0.90) / (max_radius_m * 0.10)).clamp(0.0, 1.0);
                let new_dir = forward.lerp(dir_to_center, blend * 0.6).normalize();
                self.linear_velocity = new_dir * speed;

                let target_yaw = new_dir.x.atan2(new_dir.z);
                let target_rot = Quat::from_rotation_y(target_yaw);
                self.orientation = self.orientation.slerp(target_rot, blend * 0.08);
            }

            if horiz_dist > max_radius_m {
                let clamped_dir = Vec3::new(self.position.x, 0.0, self.position.z).normalize_or_zero();
                self.position.x = clamped_dir.x * max_radius_m;
                self.position.z = clamped_dir.z * max_radius_m;
            }
            true
        } else {
            false
        }
    }

    /// Drains all pending audio events emitted during simulation.
    pub fn drain_audio_events(&mut self) -> Vec<MultimodalAudioEvent> {
        std::mem::take(&mut self.audio_events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airliner_aerodynamics_and_ground_effect() {
        let mut jet = AircraftFlight::new_airliner("Airbus-A320", Vec3::new(0.0, 2.0, 0.0), 0.0);
        assert!(jet.in_ground_effect);

        let ground_alt = 0.0;
        let mut inputs = FlightInputs {
            throttle: 1.0,
            flaps: 0.5,
            gear_down: true,
            ..Default::default()
        };

        // Advance 25 seconds of takeoff ground roll
        for _ in 0..1500 {
            jet.step(&inputs, ground_alt, 1.0 / 60.0);
        }

        assert!(
            jet.true_airspeed_mps > 50.0,
            "Jet should accelerate down runway, speed: {} m/s ({} knots)",
            jet.true_airspeed_mps,
            jet.indicated_airspeed_knots
        );

        // Rotate elevator back for climb-out
        inputs.elevator = 0.6;
        for _ in 0..600 {
            jet.step(&inputs, ground_alt, 1.0 / 60.0);
        }

        assert!(
            jet.is_airborne,
            "Aircraft should become airborne after rotation"
        );
        assert!(
            jet.position.y > 10.0,
            "Aircraft should gain altitude: y={}",
            jet.position.y
        );
        assert!(jet.climb_rate_mps > 1.0);
    }

    #[test]
    fn test_aerodynamic_stall_regime() {
        let mut cessna = AircraftFlight::new_general_aviation(
            "Cessna-Skyhawk",
            Vec3::new(0.0, 1500.0, 0.0),
            0.0,
        );

        // Initialize at slow airspeed with high pitch up attitude (22 deg)
        cessna.orientation = Quat::from_rotation_x(-22.0_f32.to_radians());
        cessna.linear_velocity = Vec3::new(0.0, 0.0, 18.0); // Below stall speed

        let inputs = FlightInputs {
            elevator: 1.0,
            ..Default::default()
        };

        cessna.step(&inputs, 0.0, 0.1);

        assert!(cessna.is_stalled, "Aircraft should be in stall regime");
    }

    #[test]
    fn test_landing_gear_compression_and_touchdown() {
        let mut jet = AircraftFlight::new_airliner("Boeing-737", Vec3::new(0.0, 3.5, 0.0), 0.0);
        // Descending towards runway at 2.5 m/s (~500 fpm)
        jet.linear_velocity = Vec3::new(0.0, -2.5, 65.0);

        let inputs = FlightInputs {
            gear_down: true,
            wheel_brakes: 0.8,
            ..Default::default()
        };

        // Advance touchdown
        for _ in 0..120 {
            jet.step(&inputs, 0.0, 1.0 / 60.0);
        }

        assert!(
            !jet.is_airborne,
            "Aircraft should have touched down on runway"
        );
        assert!(
            jet.gear_compression_meters > 0.0,
            "Landing gear oleo struts must compress on touchdown"
        );
        assert!(jet.linear_velocity.y.abs() < 1.0);
    }
}
