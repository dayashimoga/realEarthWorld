//! Railway train consist physics, tractive effort curves, Davis resistance,
//! steel wheel-on-rail adhesion limits, and Westinghouse pneumatic air brake simulation.

use crate::multimodal::MultimodalAudioEvent;
use realearth_navigation::rail_graph::SignalAspect;
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
        }
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
}
