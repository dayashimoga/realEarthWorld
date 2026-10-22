//! Hydrodynamic vessel physics: Archimedes buoyancy volume displacement,
//! metacentric righting stability, wave-surface interaction, hull resistance,
//! and propeller/rudder dynamics.
//!
//! Maintains independent water semantics decoupled from road and rail networks.

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Classification of marine vessels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarineVesselType {
    /// Large container cargo ship (deep draft, high inertia).
    CargoContainerShip,
    /// Fast passenger/vehicle ferry (medium draft, twin screw).
    PassengerFerry,
    /// Agile harbor tugboat (high bollard pull, azimuth maneuvering).
    HarborTugboat,
}

/// Dynamic marine vessel control inputs.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MarineInputs {
    /// Engine throttle command [-1.0, 1.0] (negative = astern/reverse).
    pub throttle: f32,
    /// Rudder angle command [-1.0, 1.0] (negative = port, positive = starboard).
    pub rudder: f32,
    /// Bow thruster lateral thrust command [-1.0, 1.0].
    pub bow_thruster: f32,
}

/// Dynamic hydrodynamic simulation model for marine vessels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarineVessel {
    pub name: String,
    pub vessel_type: MarineVesselType,
    // Spatial state
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub heading_radians: f32,
    pub roll_angle: f32,
    pub pitch_angle: f32,
    pub speed_knots: f32,

    // Hull geometry & hydrostatics
    /// Overall vessel length in meters.
    pub length_beam_meters: f32,
    /// Maximum hull beam (width) in meters.
    pub beam_meters: f32,
    /// Total vessel mass including displacement in kilograms.
    pub mass_kg: f32,
    /// Block coefficient Cb = V / (L * B * T), typically 0.60 to 0.85.
    pub block_coefficient: f32,
    /// Transverse metacentric height GM in meters (positive = stable righting moment).
    pub metacentric_height_gm: f32,
    /// Longitudinal metacentric height GML in meters.
    pub longitudinal_gm: f32,
    /// Design equilibrium draft in meters.
    pub design_draft_meters: f32,

    // Propulsion & steering parameters
    /// Maximum forward propeller thrust in Newtons.
    pub max_thrust_newtons: f32,
    /// Maximum rudder deflection in radians (~35 degrees).
    pub max_rudder_angle_rad: f32,
    /// Current rudder angle in radians.
    pub current_rudder_angle: f32,
    /// Water density kg/m^3 (1025.0 for seawater).
    pub water_density: f32,
}

impl MarineVessel {
    /// Creates a container cargo ship.
    pub fn new_container_ship(name: &str, position: Vec3, heading_radians: f32) -> Self {
        Self {
            name: name.to_string(),
            vessel_type: MarineVesselType::CargoContainerShip,
            position,
            orientation: Quat::from_rotation_y(heading_radians),
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            heading_radians,
            roll_angle: 0.0,
            pitch_angle: 0.0,
            speed_knots: 0.0,
            length_beam_meters: 260.0,
            beam_meters: 32.2,
            mass_kg: 45_000_000.0, // 45,000 metric tons displacement
            block_coefficient: 0.72,
            metacentric_height_gm: 2.1,
            longitudinal_gm: 180.0,
            design_draft_meters: 11.5,
            max_thrust_newtons: 1_800_000.0,
            max_rudder_angle_rad: 35.0_f32.to_radians(),
            current_rudder_angle: 0.0,
            water_density: 1025.0,
        }
    }

    /// Creates a passenger ferry.
    pub fn new_ferry(name: &str, position: Vec3, heading_radians: f32) -> Self {
        Self {
            name: name.to_string(),
            vessel_type: MarineVesselType::PassengerFerry,
            position,
            orientation: Quat::from_rotation_y(heading_radians),
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            heading_radians,
            roll_angle: 0.0,
            pitch_angle: 0.0,
            speed_knots: 0.0,
            length_beam_meters: 85.0,
            beam_meters: 16.0,
            mass_kg: 3_200_000.0, // 3,200 metric tons
            block_coefficient: 0.58,
            metacentric_height_gm: 1.8,
            longitudinal_gm: 60.0,
            design_draft_meters: 3.8,
            max_thrust_newtons: 450_000.0,
            max_rudder_angle_rad: 35.0_f32.to_radians(),
            current_rudder_angle: 0.0,
            water_density: 1025.0,
        }
    }

    /// Computes instantaneous water surface height at vessel position.
    pub fn water_surface_height(&self, time: f32) -> f32 {
        // Open ocean harmonic wave profile
        let wave_amplitude = 0.6; // 0.6m swell
        let wave_length = 40.0;
        let k = 2.0 * std::f32::consts::PI / wave_length;
        let omega = (9.81 * k).sqrt();
        wave_amplitude * (k * self.position.x - omega * time).sin()
    }

    /// Advances the marine vessel hydrodynamic simulation by `dt` seconds.
    pub fn step(&mut self, inputs: &MarineInputs, time: f32, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        // 1. Water Surface & Archimedes Buoyancy Heave Equilibrium
        let wave_h = self.water_surface_height(time);
        let waterplane_area =
            self.length_beam_meters * self.beam_meters * self.block_coefficient * 0.9;
        let equilibrium_draft = self.mass_kg / (self.water_density * waterplane_area);

        // Submergence depth: how far bottom of hull is below instantaneous water surface
        let hull_bottom = self.position.y - self.design_draft_meters;
        let actual_submerged_depth = (wave_h - hull_bottom).max(0.0);
        let submerged_volume = waterplane_area * actual_submerged_depth;

        // Archimedes buoyant force
        let buoyant_force = self.water_density * submerged_volume * 9.81;
        let gravity_force = self.mass_kg * 9.81;
        let heave_damping =
            2.0 * (self.mass_kg * 9.81 * waterplane_area * self.water_density).sqrt();
        let net_vertical_force =
            (buoyant_force - gravity_force) - (heave_damping * self.linear_velocity.y);
        let vertical_accel = net_vertical_force / self.mass_kg;

        // 2. Metacentric Stability & Righting Moment: M = Mass * g * GM * sin(phi)
        let righting_moment_roll =
            -self.mass_kg * 9.81 * self.metacentric_height_gm * self.roll_angle.sin();
        let roll_inertia = self.mass_kg * (self.beam_meters * self.beam_meters + 4.0 * 4.0) / 12.0;
        let roll_damping = -roll_inertia * 0.8 * self.angular_velocity.z;
        let roll_accel = (righting_moment_roll + roll_damping) / roll_inertia;

        let righting_moment_pitch =
            -self.mass_kg * 9.81 * self.longitudinal_gm * self.pitch_angle.sin();
        let pitch_inertia =
            self.mass_kg * (self.length_beam_meters * self.length_beam_meters) / 12.0;
        let pitch_damping = -pitch_inertia * 1.2 * self.angular_velocity.x;
        let pitch_accel = (righting_moment_pitch + pitch_damping) / pitch_inertia;

        // 3. Propulsion Thrust & Steering Rudder Hydrofoil Lift
        let forward_dir = Vec3::new(self.heading_radians.sin(), 0.0, self.heading_radians.cos());
        let lateral_dir = Vec3::new(self.heading_radians.cos(), 0.0, -self.heading_radians.sin());

        // Rudder slew rate
        let target_rudder = inputs.rudder.clamp(-1.0, 1.0) * self.max_rudder_angle_rad;
        let rudder_rate = 0.25; // rad/s
        self.current_rudder_angle +=
            (target_rudder - self.current_rudder_angle).clamp(-rudder_rate * dt, rudder_rate * dt);

        let forward_speed = self.linear_velocity.dot(forward_dir);
        let lateral_speed = self.linear_velocity.dot(lateral_dir);

        // Propeller thrust along ship longitudinal axis
        let thrust = inputs.throttle.clamp(-0.5, 1.0) * self.max_thrust_newtons;

        // Hull Hydrodynamic Resistance
        // Skin friction + wave making drag
        let wetted_surface = 2.0 * self.length_beam_meters * equilibrium_draft
            + self.length_beam_meters * self.beam_meters;
        let cf = 0.0025; // ITTC-57 friction line approx
        let skin_friction =
            0.5 * self.water_density * forward_speed * forward_speed.abs() * wetted_surface * cf;
        let wave_making = 120.0 * forward_speed.powi(3) * forward_speed.signum();
        let hull_drag = skin_friction + wave_making;

        // Rudder hydrodynamic lift: L = 0.5 * rho * V^2 * A_rudder * Cl_alpha * rudder_angle
        let rudder_area = (self.length_beam_meters * self.design_draft_meters) / 45.0;
        let prop_wash = (2.0 * thrust.max(0.0) / (self.water_density * 15.0)).sqrt();
        let water_flow_speed = (forward_speed.abs() + prop_wash).max(2.5);
        let rudder_lift = 0.5
            * self.water_density
            * water_flow_speed
            * water_flow_speed
            * rudder_area
            * 2.5
            * self.current_rudder_angle;

        // Turning moment around vertical (Y) axis
        let rudder_arm = -self.length_beam_meters * 0.45;
        let yaw_moment = rudder_lift * rudder_arm;
        let yaw_inertia = pitch_inertia; // Approximates yaw inertia for slender beam
        let yaw_damping = -yaw_inertia * 0.9 * self.angular_velocity.y;
        let yaw_accel = (yaw_moment + yaw_damping) / yaw_inertia;

        // Bow thruster lateral force
        let bow_force = inputs.bow_thruster.clamp(-1.0, 1.0) * (self.max_thrust_newtons * 0.15);

        // Net forces
        let forward_accel = (thrust - hull_drag) / self.mass_kg;
        let lateral_accel = (rudder_lift + bow_force - lateral_speed * 15000.0) / self.mass_kg;

        // 4. State Integration
        self.linear_velocity.y += vertical_accel * dt;
        let forward_vel_delta = forward_accel * dt;
        let lateral_vel_delta = lateral_accel * dt;

        self.linear_velocity += forward_dir * forward_vel_delta + lateral_dir * lateral_vel_delta;
        self.position += self.linear_velocity * dt;

        self.angular_velocity.x += pitch_accel * dt;
        self.angular_velocity.y += yaw_accel * dt;
        self.angular_velocity.z += roll_accel * dt;

        self.pitch_angle += self.angular_velocity.x * dt;
        self.heading_radians += self.angular_velocity.y * dt;
        self.roll_angle += self.angular_velocity.z * dt;

        self.orientation = Quat::from_rotation_y(self.heading_radians)
            * Quat::from_rotation_x(self.pitch_angle)
            * Quat::from_rotation_z(self.roll_angle);

        let horizontal_speed = (self.linear_velocity.x * self.linear_velocity.x
            + self.linear_velocity.z * self.linear_velocity.z)
            .sqrt();
        self.speed_knots = horizontal_speed * 1.94384; // m/s to knots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marine_buoyancy_and_draft_equilibrium() {
        let mut ship = MarineVessel::new_ferry("Goa-Express", Vec3::new(0.0, 0.0, 0.0), 0.0);
        let inputs = MarineInputs::default();

        // Advance 10 seconds (600 ticks) allowing heave to settle to Archimedes equilibrium
        for tick in 0..600 {
            let time = (tick as f32) / 60.0;
            ship.step(&inputs, time, 1.0 / 60.0);
        }

        // Ship should be floating stably near sea level
        assert!(
            ship.position.y.abs() < 2.0,
            "Ship should remain floating near surface datum, was y={}",
            ship.position.y
        );
        assert!(ship.linear_velocity.y.abs() < 0.2);
    }

    #[test]
    fn test_metacentric_roll_righting_moment() {
        let mut ship =
            MarineVessel::new_container_ship("Ever-RealEarth", Vec3::new(0.0, 0.0, 0.0), 0.0);

        // Perturb vessel roll heel by 12 degrees (~0.21 radians)
        ship.roll_angle = 12.0_f32.to_radians();

        let inputs = MarineInputs::default();

        // Advance 15 seconds to let righting moment damp roll heel back to upright
        for tick in 0..900 {
            let time = (tick as f32) / 60.0;
            ship.step(&inputs, time, 1.0 / 60.0);
        }

        assert!(
            ship.roll_angle.abs() < 2.0_f32.to_radians(),
            "Metacentric GM must right the vessel towards upright, current roll: {} deg",
            ship.roll_angle.to_degrees()
        );
    }

    #[test]
    fn test_marine_propulsion_and_rudder_turning() {
        let mut ship = MarineVessel::new_ferry("Coastal-Runner", Vec3::ZERO, 0.0);

        let inputs = MarineInputs {
            throttle: 1.0,
            rudder: 0.8,
            ..Default::default()
        };

        for tick in 0..1500 {
            let time = (tick as f32) / 60.0;
            ship.step(&inputs, time, 1.0 / 60.0);
        }

        assert!(
            ship.speed_knots > 5.0,
            "Ferry should have accelerated, speed: {} knots",
            ship.speed_knots
        );
        assert!(
            ship.heading_radians.abs() > 0.05,
            "Rudder should cause ship heading to rotate, current heading: {} rad",
            ship.heading_radians
        );
    }
}
