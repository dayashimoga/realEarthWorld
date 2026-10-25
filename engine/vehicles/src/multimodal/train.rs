//! Railway train consist physics, tractive effort curves, Davis resistance,
//! steel wheel-on-rail adhesion limits, and Westinghouse pneumatic air brake simulation.

use crate::multimodal::MultimodalAudioEvent;
use realearth_navigation::{RailNetworkGraph, RailTrackPose, SignalAspect};
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Type of railway locomotive propulsion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocomotiveType {
    /// 25 kV AC overhead catenary electric locomotive (e.g. WAP-7, WAG-9).
    Electric,
    /// Diesel-electric traction generator/motor unit (e.g. WDG-4, GE Evolution).
    DieselElectric,
    /// High-speed multiple-unit EMU/DMU consist.
    MultipleUnit,
}

/// Dynamic state of the train braking system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainBrakeState {
    /// Brakes released; brake pipe at standard running pressure (5.0 bar).
    Released,
    /// Controlled service braking application (4.5 to 3.5 bar).
    ServiceBraking,
    /// Full service brake application (3.5 bar).
    FullService,
    /// Emergency brake venting: brake pipe vented to atmosphere (0.0 bar).
    Emergency,
}

/// A single rail vehicle (locomotive, passenger coach, or freight hopper/tanker).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainCar {
    pub id: String,
    pub mass_kg: f32,
    pub length_meters: f32,
    pub axle_count: u32,
    pub is_powered: bool,
    pub tare_weight_kg: f32,
    pub cargo_tonnage: f32,
}

impl TrainCar {
    pub fn new_locomotive(id: &str, mass_kg: f32, length_meters: f32, axle_count: u32) -> Self {
        Self {
            id: id.to_string(),
            mass_kg,
            length_meters,
            axle_count,
            is_powered: true,
            tare_weight_kg: mass_kg,
            cargo_tonnage: 0.0,
        }
    }

    pub fn new_wagon(id: &str, tare_kg: f32, cargo_kg: f32, length_meters: f32) -> Self {
        Self {
            id: id.to_string(),
            mass_kg: tare_kg + cargo_kg,
            length_meters,
            axle_count: 4,
            is_powered: false,
            tare_weight_kg: tare_kg,
            cargo_tonnage: cargo_kg / 1000.0,
        }
    }
}

/// Davis train resistance coefficients: R(v) = A + B*v + C*v^2
/// Accounts for axle journal bearing friction (A), wheel flange contact / track wave deflection (B),
/// and aerodynamic cross-sectional head/skin/tail drag (C).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DavisCoefficients {
    /// Rolling and bearing resistance coefficient (Newtons).
    pub a_journal_bearing: f32,
    /// Flange contact and dynamic track deflection resistance (N / (m/s)).
    pub b_flange_track: f32,
    /// Aerodynamic profile and skin friction drag (N / (m/s)^2).
    pub c_aerodynamic: f32,
}

impl Default for DavisCoefficients {
    fn default() -> Self {
        Self {
            a_journal_bearing: 2500.0,
            b_flange_track: 45.0,
            c_aerodynamic: 4.8,
        }
    }
}

/// Dynamic train consist simulation model with authentic longitudinal physics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainConsist {
    pub name: String,
    pub locomotive_type: LocomotiveType,
    pub cars: Vec<TrainCar>,
    /// Continuous rated power output in Watts (e.g. 4.5 MW = 4,500,000 W).
    pub rated_power_watts: f32,
    /// Maximum tractive effort limit at low speed in Newtons (e.g. 450,000 N).
    pub max_tractive_effort_n: f32,
    /// Coefficient of adhesion between steel wheel and steel rail (typically 0.28 - 0.35).
    pub adhesion_coefficient: f32,
    /// Mass resting on driven powered axles in kg.
    pub driven_mass_kg: f32,
    /// Davis equation resistance parameters.
    pub davis_coefficients: DavisCoefficients,

    // Dynamic states
    /// Arc-length position along current track segment in meters.
    pub position_s: f64,
    /// Longitudinal velocity in m/s (positive = forward).
    pub velocity_mps: f32,
    /// Speed in km/h.
    pub speed_kph: f32,
    /// Current longitudinal acceleration in m/s^2.
    pub acceleration_mps2: f32,
    /// Throttle demand [0.0, 1.0].
    pub throttle: f32,
    /// Direction reverser: 1 = forward, -1 = reverse, 0 = neutral.
    pub reverser: i8,
    /// Westinghouse brake pipe pressure in bar (running: 5.0 bar, emergency: 0.0 bar).
    pub brake_pipe_pressure_bar: f32,
    /// Brake cylinder effective pressure in bar (released: 0.0 bar, maximum: 3.8 bar).
    pub brake_cylinder_pressure_bar: f32,
    /// Target brake application state.
    pub brake_state: TrainBrakeState,
    /// Active wheel slip detection (tractive demand exceeded rail adhesion limit).
    pub wheel_slip: bool,
    /// Emergency brake tripped.
    pub emergency_brake: bool,
    /// Id of active track segment.
    pub current_track_id: Option<String>,
    /// Last observed signal aspect.
    pub current_signal: SignalAspect,
    /// Pending audio events emitted during simulation.
    pub audio_events: Vec<MultimodalAudioEvent>,
    /// Active route sequence of segment IDs.
    pub active_route: Vec<String>,
    /// Index of segment within active route.
    pub current_segment_idx: usize,
    /// Distance along current segment in meters.
    pub distance_along_segment: f64,
    /// Total cumulative distance traversed along active route in meters.
    pub total_route_distance: f64,
    /// True when train has reached the end of the line / buffer stop.
    pub is_at_buffer_stop: bool,
    /// Current 3D track pose of the locomotive.
    pub track_pose: Option<RailTrackPose>,
    /// Current 3D track poses of each trailing carriage.
    pub carriage_poses: Vec<RailTrackPose>,
}

impl TrainConsist {
    /// Creates a new locomotive-led train consist.
    pub fn new_locomotive(
        name: &str,
        locomotive_type: LocomotiveType,
        rated_power_watts: f32,
        max_tractive_effort_n: f32,
        locomotive_mass_kg: f32,
        length_meters: f32,
    ) -> Self {
        let loco = TrainCar::new_locomotive(name, locomotive_mass_kg, length_meters, 6);
        Self {
            name: name.to_string(),
            locomotive_type,
            cars: vec![loco],
            rated_power_watts,
            max_tractive_effort_n,
            adhesion_coefficient: 0.32,
            driven_mass_kg: locomotive_mass_kg,
            davis_coefficients: DavisCoefficients::default(),
            position_s: 0.0,
            velocity_mps: 0.0,
            speed_kph: 0.0,
            acceleration_mps2: 0.0,
            throttle: 0.0,
            reverser: 1,
            brake_pipe_pressure_bar: 5.0,
            brake_cylinder_pressure_bar: 0.0,
            brake_state: TrainBrakeState::Released,
            wheel_slip: false,
            emergency_brake: false,
            current_track_id: None,
            current_signal: SignalAspect::Clear,
            audio_events: Vec::new(),
            active_route: Vec::new(),
            current_segment_idx: 0,
            distance_along_segment: 0.0,
            total_route_distance: 0.0,
            is_at_buffer_stop: false,
            track_pose: None,
            carriage_poses: Vec::new(),
        }
    }

    /// Creates an authentic Caltrain commuter consist (MPI MP36PH-3C diesel-electric locomotive + 4 Bombardier BiLevel cars).
    pub fn new_caltrain_diesel() -> Self {
        let mut train = Self::new_locomotive(
            "Caltrain MP36PH-3C #925",
            LocomotiveType::DieselElectric,
            2_684_000.0,
            290_000.0,
            131_500.0,
            17.15,
        );
        for i in 1..=4 {
            train.add_wagon(TrainCar::new_wagon(
                &format!("Caltrain-BiLevel-{}", 4000 + i),
                50_000.0,
                5_000.0,
                25.9,
            ));
        }
        train
    }

    /// Adds a trailing wagon or passenger coach to the consist.
    pub fn add_wagon(&mut self, wagon: TrainCar) {
        if wagon.is_powered {
            self.driven_mass_kg += wagon.mass_kg;
        }
        self.cars.push(wagon);
    }

    /// Total gross mass of the entire train consist in kilograms.
    pub fn total_mass_kg(&self) -> f32 {
        self.cars.iter().map(|c| c.mass_kg).sum()
    }

    /// Total length of the train consist from leading coupler to tail end in meters.
    pub fn total_length_meters(&self) -> f32 {
        self.cars.iter().map(|c| c.length_meters).sum()
    }

    /// Applies train service brakes to a target pressure level.
    pub fn apply_service_brake(&mut self, brake_input: f32) {
        let clamped = brake_input.clamp(0.0, 1.0);
        if clamped > 0.8 {
            self.brake_state = TrainBrakeState::FullService;
        } else if clamped > 0.05 {
            self.brake_state = TrainBrakeState::ServiceBraking;
        } else {
            self.brake_state = TrainBrakeState::Released;
        }
    }

    /// Triggers immediate emergency brake application (venting train brake pipe).
    pub fn trigger_emergency_brake(&mut self) {
        self.emergency_brake = true;
        self.brake_state = TrainBrakeState::Emergency;
        self.throttle = 0.0;
        self.audio_events.push(MultimodalAudioEvent::TrainHorn);
    }

    /// Sounds the train horn/whistle.
    pub fn sound_horn(&mut self) {
        self.audio_events.push(MultimodalAudioEvent::TrainHorn);
    }

    /// Drains all pending audio events emitted by the train consist.
    pub fn drain_audio_events(&mut self) -> Vec<MultimodalAudioEvent> {
        std::mem::take(&mut self.audio_events)
    }

    /// Resets emergency brake after train comes to complete stop.
    pub fn reset_emergency_brake(&mut self) -> bool {
        if self.velocity_mps.abs() < 0.01 {
            self.emergency_brake = false;
            self.brake_state = TrainBrakeState::Released;
            true
        } else {
            false
        }
    }

    /// Advances the train physical simulation by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        let total_mass = self.total_mass_kg();
        if total_mass <= 1.0 {
            return;
        }

        // 1. Westinghouse Brake Pipe & Cylinder Dynamics
        let target_pipe_pressure = match self.brake_state {
            TrainBrakeState::Released => 5.0,
            TrainBrakeState::ServiceBraking => 4.2,
            TrainBrakeState::FullService => 3.5,
            TrainBrakeState::Emergency => 0.0,
        };

        // Brake pipe propagation rate (acoustic air wave venting/charging)
        let pipe_rate = if self.brake_state == TrainBrakeState::Emergency {
            8.0 // Rapid venting
        } else {
            1.2 // Normal service charging/exhaust
        };

        if self.brake_pipe_pressure_bar < target_pipe_pressure {
            self.brake_pipe_pressure_bar =
                (self.brake_pipe_pressure_bar + pipe_rate * dt).min(target_pipe_pressure);
        } else if self.brake_pipe_pressure_bar > target_pipe_pressure {
            self.brake_pipe_pressure_bar =
                (self.brake_pipe_pressure_bar - pipe_rate * dt).max(target_pipe_pressure);
        }

        // Triple valve: cylinder pressure builds inversely to brake pipe drop below 5.0 bar
        let pipe_drop = (5.0 - self.brake_pipe_pressure_bar).max(0.0);
        let target_cylinder_pressure = (pipe_drop * (3.8 / 1.5)).clamp(0.0, 3.8);
        let cyl_rate = 2.5; // cylinder filling/exhaust rate
        if self.brake_cylinder_pressure_bar < target_cylinder_pressure {
            self.brake_cylinder_pressure_bar =
                (self.brake_cylinder_pressure_bar + cyl_rate * dt).min(target_cylinder_pressure);
        } else {
            self.brake_cylinder_pressure_bar =
                (self.brake_cylinder_pressure_bar - cyl_rate * dt).max(target_cylinder_pressure);
        }

        // Effective braking force
        let brake_friction_coeff = 0.14; // cast-iron / composite brake shoes on wheel treads
        let brake_ratio = self.brake_cylinder_pressure_bar / 3.8;
        let braking_force_n = brake_ratio * total_mass * 9.81 * brake_friction_coeff;

        // 2. Locomotive Tractive Effort & Rail Adhesion Limit
        let current_speed = self.velocity_mps.abs();
        let max_adhesion_force = self.adhesion_coefficient * self.driven_mass_kg * 9.81;

        let mut tractive_effort_n = 0.0;
        if self.throttle > 0.0 && !self.emergency_brake && self.reverser != 0 {
            // Power curve: P = F * v => F = P / v
            let speed_ref = current_speed.max(1.5); // Stall limit transition
            let power_tractive = (self.rated_power_watts / speed_ref) * self.throttle;
            let demanded_tractive = power_tractive.min(self.max_tractive_effort_n * self.throttle);

            if demanded_tractive > max_adhesion_force {
                // Wheel slip condition: kinetic friction coefficient is ~70% of static adhesion
                self.wheel_slip = true;
                tractive_effort_n = max_adhesion_force * 0.72;
            } else {
                self.wheel_slip = false;
                tractive_effort_n = demanded_tractive;
            }
        } else {
            self.wheel_slip = false;
        }

        let traction_sign = self.reverser as f32;

        // 3. Davis Train Resistance: R = A + B*v + C*v^2
        let resistance_n = self.davis_coefficients.a_journal_bearing
            + self.davis_coefficients.b_flange_track * current_speed
            + self.davis_coefficients.c_aerodynamic * current_speed * current_speed;

        // 4. Net Longitudinal Force & Acceleration
        let net_force_n = if current_speed > 0.05 {
            let motion_dir = self.velocity_mps.signum();
            (tractive_effort_n * traction_sign) - (motion_dir * (resistance_n + braking_force_n))
        } else if tractive_effort_n > 0.0 && self.brake_cylinder_pressure_bar < 0.5 {
            // Overcoming standstill friction
            let static_breakaway = self.davis_coefficients.a_journal_bearing * 1.5;
            if tractive_effort_n > static_breakaway {
                (tractive_effort_n - static_breakaway) * traction_sign
            } else {
                0.0
            }
        } else {
            // Stationary held by brakes or lack of tractive effort
            0.0
        };

        self.acceleration_mps2 = net_force_n / total_mass;

        // Velocity integration
        let new_velocity = self.velocity_mps + self.acceleration_mps2 * dt;

        // Handle standstill zero-crossing under braking
        if current_speed > 0.0 && (new_velocity.signum() != self.velocity_mps.signum()) {
            if braking_force_n > tractive_effort_n {
                self.velocity_mps = 0.0;
                self.acceleration_mps2 = 0.0;
            } else {
                self.velocity_mps = new_velocity;
            }
        } else {
            self.velocity_mps = new_velocity;
        }

        // Complete standstill friction clamp under active braking
        if self.velocity_mps.abs() < 0.10
            && self.brake_cylinder_pressure_bar > 0.5
            && self.throttle == 0.0
        {
            self.velocity_mps = 0.0;
            self.acceleration_mps2 = 0.0;
        }

        self.speed_kph = self.velocity_mps.abs() * 3.6;
        self.position_s += (self.velocity_mps as f64) * (dt as f64);

        if self.speed_kph > 35.0 && self.speed_kph < 38.0 {
            self.audio_events
                .push(MultimodalAudioEvent::TrainFlangeSqueal {
                    speed_kph: self.speed_kph,
                });
        }

        // 5. Automatic Train Protection (ATP) Signal Compliance
        if self.current_signal == SignalAspect::Stop && self.velocity_mps.abs() > 0.1 {
            self.trigger_emergency_brake();
        }
    }

    /// Binds the train consist to an active rail route and initializes all carriage poses along the track.
    pub fn bind_route(
        &mut self,
        graph: &RailNetworkGraph,
        route_segments: Vec<String>,
        anchor: Wgs84Coord,
    ) {
        self.active_route = route_segments;
        self.current_segment_idx = 0;
        self.distance_along_segment = 0.0;
        self.total_route_distance = 0.0;
        self.is_at_buffer_stop = false;
        self.update_track_poses(graph, anchor);
    }

    /// Explicitly sets the distance along the active route and updates all consist poses.
    pub fn set_route_distance(
        &mut self,
        graph: &RailNetworkGraph,
        distance_m: f64,
        anchor: Wgs84Coord,
    ) {
        let total_len = graph.total_route_length(&self.active_route);
        self.total_route_distance = distance_m.clamp(0.0, total_len);
        self.update_track_poses(graph, anchor);
    }

    /// Updates 3D poses (locomotive and all carriages) along the active route.
    pub fn update_track_poses(&mut self, graph: &RailNetworkGraph, anchor: Wgs84Coord) {
        if self.active_route.is_empty() {
            return;
        }

        let loco_sample =
            graph.sample_route_pose(&self.active_route, self.total_route_distance, anchor);

        if let Some((pose, seg_idx, dist_on_seg, at_buffer)) = loco_sample {
            self.track_pose = Some(pose);
            self.current_segment_idx = seg_idx;
            self.distance_along_segment = dist_on_seg;
            self.is_at_buffer_stop = at_buffer;
            if seg_idx < self.active_route.len() {
                self.current_track_id = Some(self.active_route[seg_idx].clone());
                if let Some(seg) = graph.segments.get(&self.active_route[seg_idx]) {
                    self.current_signal = seg.signal_aspect;
                }
            }
        }

        // Articulate carriages along track behind the locomotive
        let mut carriage_poses = Vec::with_capacity(self.cars.len().saturating_sub(1));
        if !self.cars.is_empty() {
            let loco_len = self.cars[0].length_meters as f64;
            let mut cumulative_dist_offset = loco_len * 0.5;

            for i in 1..self.cars.len() {
                let car_len = self.cars[i].length_meters as f64;
                cumulative_dist_offset += car_len * 0.5;
                let car_s = (self.total_route_distance - cumulative_dist_offset).max(0.0);
                if let Some((car_pose, _, _, _)) =
                    graph.sample_route_pose(&self.active_route, car_s, anchor)
                {
                    carriage_poses.push(car_pose);
                }
                cumulative_dist_offset += car_len * 0.5;
            }
        }
        self.carriage_poses = carriage_poses;
    }

    /// Advances the train physical simulation by `dt` seconds and computes new track poses.
    pub fn step_along_route(&mut self, graph: &RailNetworkGraph, anchor: Wgs84Coord, dt: f32) {
        if self.active_route.is_empty() {
            self.step(dt);
            return;
        }

        // Buffer stop check
        let total_route_len = graph.total_route_length(&self.active_route);
        if self.is_at_buffer_stop && self.velocity_mps > 0.0 && self.reverser > 0 {
            self.velocity_mps = 0.0;
            self.acceleration_mps2 = 0.0;
            self.throttle = 0.0;
            return;
        }

        // Step longitudinal physics (Westinghouse brakes, Davis resistance, tractive effort)
        self.step(dt);

        // Update distance along route
        let new_dist = self.total_route_distance + (self.velocity_mps as f64) * (dt as f64);
        if new_dist >= total_route_len {
            self.total_route_distance = total_route_len;
            self.velocity_mps = 0.0;
            self.acceleration_mps2 = 0.0;
            self.is_at_buffer_stop = true;
        } else if new_dist <= 0.0 {
            self.total_route_distance = 0.0;
            if self.velocity_mps < 0.0 {
                self.velocity_mps = 0.0;
                self.acceleration_mps2 = 0.0;
            }
            self.is_at_buffer_stop = false;
        } else {
            self.total_route_distance = new_dist;
            self.is_at_buffer_stop = false;
        }

        self.update_track_poses(graph, anchor);
    }

    /// Curvature of the track under the locomotive in rad/m.
    pub fn current_curvature(&self) -> f32 {
        self.track_pose
            .as_ref()
            .map(|p| p.curvature_rad_per_m)
            .unwrap_or(0.0)
    }

    /// Number of trailing carriages in the consist.
    pub fn carriage_count(&self) -> usize {
        self.cars.len().saturating_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_train_consist_mass_and_length() {
        // WAP-7 locomotive: 123 tons, 20.5 meters
        let mut train = TrainConsist::new_locomotive(
            "WAP-7",
            LocomotiveType::Electric,
            4_500_000.0,
            450_000.0,
            123_000.0,
            20.5,
        );

        // Add 10 passenger coaches (45 tons, 24 meters each)
        for i in 1..=10 {
            train.add_wagon(TrainCar::new_wagon(
                &format!("LHB-AC-{}", i),
                42_000.0,
                3_000.0,
                24.0,
            ));
        }

        assert_eq!(train.cars.len(), 11);
        let expected_mass = 123_000.0 + (10.0 * 45_000.0);
        assert_eq!(train.total_mass_kg(), expected_mass);

        let expected_length = 20.5 + (10.0 * 24.0);
        assert_eq!(train.total_length_meters(), expected_length);
    }

    #[test]
    fn test_train_tractive_acceleration_and_davis_resistance() {
        let mut train = TrainConsist::new_locomotive(
            "WAP-7",
            LocomotiveType::Electric,
            4_500_000.0,
            450_000.0,
            123_000.0,
            20.5,
        );

        train.throttle = 1.0;
        train.reverser = 1;

        // Step for 10 seconds (600 ticks)
        for _ in 0..600 {
            train.step(1.0 / 60.0);
        }

        assert!(
            train.speed_kph > 20.0,
            "Train should have accelerated past 20 km/h, current: {}",
            train.speed_kph
        );
        assert!(train.position_s > 25.0);
        assert!(!train.wheel_slip);
    }

    #[test]
    fn test_westinghouse_pneumatic_brake_propagation_and_stopping() {
        let mut train = TrainConsist::new_locomotive(
            "WAP-7",
            LocomotiveType::Electric,
            4_500_000.0,
            450_000.0,
            123_000.0,
            20.5,
        );

        // Accelerate up to speed first
        train.throttle = 1.0;
        for _ in 0..600 {
            train.step(1.0 / 60.0);
        }
        let top_speed = train.speed_kph;
        assert!(top_speed > 20.0);

        // Cut power and trigger emergency brake
        train.throttle = 0.0;
        train.trigger_emergency_brake();

        // Advance 25 seconds under emergency venting
        for _ in 0..1500 {
            train.step(1.0 / 60.0);
        }

        assert_eq!(
            train.brake_pipe_pressure_bar, 0.0,
            "Emergency venting should exhaust pipe to 0.0 bar"
        );
        assert_eq!(
            train.velocity_mps, 0.0,
            "Train must come to complete stop under emergency brake"
        );
    }

    #[test]
    fn test_automatic_train_protection_trips_at_stop_signal() {
        let mut train = TrainConsist::new_locomotive(
            "WAP-7",
            LocomotiveType::Electric,
            4_500_000.0,
            450_000.0,
            123_000.0,
            20.5,
        );

        train.throttle = 0.8;
        train.step(0.1);
        assert!(train.velocity_mps > 0.0);

        // Aspect switches to Stop
        train.current_signal = SignalAspect::Stop;
        train.step(0.1);

        assert!(train.emergency_brake);
        assert_eq!(train.brake_state, TrainBrakeState::Emergency);
    }

    #[test]
    fn test_caltrain_diesel_consist_route_binding_and_track_following() {
        let rail = RailNetworkGraph::create_san_francisco_caltrain_corridor();
        let route = RailNetworkGraph::build_caltrain_mainline_route();
        let anchor = Wgs84Coord::new(37.7764, -122.3942, 5.0);

        let mut consist = TrainConsist::new_caltrain_diesel();
        assert_eq!(consist.cars.len(), 5); // 1 loco + 4 bilevel coaches
        assert_eq!(consist.carriage_count(), 4);

        consist.bind_route(&rail, route.segment_ids.clone(), anchor);
        assert!(consist.track_pose.is_some());
        assert_eq!(consist.carriage_poses.len(), 4);

        let initial_loco_pos = consist.track_pose.unwrap().position;
        // Locomotive should be at 0,0,0 relative to anchor
        assert!(initial_loco_pos.length() < 0.1);

        // Apply throttle and step along track for 10 seconds (600 ticks)
        consist.throttle = 1.0;
        consist.reverser = 1;
        for _ in 0..600 {
            consist.step_along_route(&rail, anchor, 1.0 / 60.0);
        }

        assert!(
            consist.total_route_distance > 10.0,
            "Caltrain consist should have traveled >10m in 10s, actual: {}",
            consist.total_route_distance
        );
        let advanced_loco_pos = consist.track_pose.unwrap().position;
        assert!(advanced_loco_pos.length() > 10.0);

        // Verify zero cross-track deviation
        let xtrack_err =
            rail.route_cross_track_distance(&route.segment_ids, advanced_loco_pos, anchor);
        assert!(
            xtrack_err < 0.05,
            "Locomotive cross-track error ({}) must be under 5cm",
            xtrack_err
        );

        // Verify each carriage also has zero cross-track error
        for (idx, c_pose) in consist.carriage_poses.iter().enumerate() {
            let c_xtrack =
                rail.route_cross_track_distance(&route.segment_ids, c_pose.position, anchor);
            assert!(
                c_xtrack < 0.05,
                "Carriage {} cross-track error ({}) must be under 5cm",
                idx + 1,
                c_xtrack
            );
        }
    }
}
