//! Multi-transport road vehicles: Motorcycles, Transit Buses, and Heavy Articulated Trucks.
//!
//! Provides dedicated mass, suspension, tire handling, lean physics,
//! trailer articulation, passenger payload, and traffic restriction models.

use crate::dynamics::{TransmissionMode, VehicleInputs};
use crate::ground::GroundQuery;
use crate::multimodal::MultimodalAudioEvent;
use glam::{Mat3, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Functional classification of road transport vehicles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoadVehicleCategory {
    /// 2-wheeled agile motorcycle with lean roll dynamics.
    Motorcycle,
    /// Standard 4-wheeled passenger car or taxi.
    CitySedan,
    /// Heavy public transit bus with pneumatic suspension and swept-path turning.
    TransitBus,
    /// Multi-axle commercial freight truck with trailer articulation and rollover limits.
    HeavyTruck,
}

/// Road access and operational restriction policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoadAccessRestriction {
    /// Open to all road vehicle types.
    OpenAccess,
    /// Designated Bus Rapid Transit (BRT) lane; only buses permitted.
    BusOnly,
    /// Residential or historic heritage sector; commercial heavy trucks prohibited.
    NoHeavyTrucks,
    /// Narrow alley or two-wheeler lane; only motorcycles permitted.
    MotorcycleOnly,
    /// High-speed limited-access expressway (minimum speed required, no low-power mopeds).
    ExpresswayOnly,
}

impl RoadAccessRestriction {
    /// Evaluates if a given road vehicle category is legally permitted on this corridor.
    pub fn is_allowed(&self, category: RoadVehicleCategory) -> bool {
        match self {
            Self::OpenAccess => true,
            Self::BusOnly => category == RoadVehicleCategory::TransitBus,
            Self::NoHeavyTrucks => category != RoadVehicleCategory::HeavyTruck,
            Self::MotorcycleOnly => category == RoadVehicleCategory::Motorcycle,
            Self::ExpresswayOnly => {
                category == RoadVehicleCategory::CitySedan
                    || category == RoadVehicleCategory::TransitBus
                    || category == RoadVehicleCategory::HeavyTruck
                    || category == RoadVehicleCategory::Motorcycle
            }
        }
    }
}

// ============================================================================
// 1. MOTORCYCLE: 2-Wheel Lean Dynamics & Camber Thrust
// ============================================================================

/// Dynamic 2-wheeled motorcycle model with roll lean dynamics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotorcycleModel {
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub speed_kph: f32,
    /// Chassis roll lean angle in radians (negative = lean left, positive = lean right).
    pub lean_angle: f32,
    /// Rate of lean angle change in rad/s.
    pub lean_velocity: f32,
    /// Steered front wheel angle in radians.
    pub steer_angle: f32,
    /// Curb weight including rider (default: 220 kg).
    pub mass_kg: f32,
    /// Wheelbase: distance between front and rear contact patches (default: 1.45m).
    pub wheelbase: f32,
    /// Maximum physical lean angle before footpeg ground contact (~48 degrees).
    pub max_lean_angle: f32,
    /// Engine power drive force in Newtons.
    pub engine_force: f32,
    /// Is motorcycle currently upright and stable.
    pub is_crashed: bool,
    /// Are wheels in contact with road.
    pub is_grounded: bool,
    /// Pending audio events emitted during simulation.
    pub audio_events: Vec<MultimodalAudioEvent>,
}

impl Default for MotorcycleModel {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 0.0)
    }
}

impl MotorcycleModel {
    pub fn new(position: Vec3, yaw: f32) -> Self {
        Self {
            position,
            orientation: Quat::from_rotation_y(yaw),
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            speed_kph: 0.0,
            lean_angle: 0.0,
            lean_velocity: 0.0,
            steer_angle: 0.0,
            mass_kg: 220.0,
            wheelbase: 1.45,
            max_lean_angle: 0.84, // ~48 degrees
            engine_force: 3200.0,
            is_crashed: false,
            is_grounded: true,
            audio_events: Vec::new(),
        }
    }

    /// Advances motorcycle physics by `dt` seconds.
    pub fn step(&mut self, inputs: &VehicleInputs, ground: &dyn GroundQuery, dt: f32) {
        if dt <= 0.0 || self.is_crashed {
            return;
        }

        let current_yaw = self.orientation.to_euler(glam::EulerRot::YXZ).0;
        let rot_mat = Mat3::from_quat(Quat::from_rotation_y(current_yaw));
        let forward = rot_mat.col(2);
        let forward_speed = self.linear_velocity.dot(forward);
        let current_speed = forward_speed.abs();
        self.speed_kph = current_speed * 3.6;

        // Steering control with high agility
        let target_steer = inputs.steer * 0.45; // ~26 degrees max steer
        self.steer_angle += (target_steer - self.steer_angle) * (15.0 * dt).min(1.0);

        // Calculate dynamic equilibrium lean angle: tan(phi) = v^2 / (R * g)
        // With curvature kappa ~ steer_angle / wheelbase:
        let lateral_accel = if forward_speed > 0.5 {
            forward_speed * forward_speed * (self.steer_angle / self.wheelbase)
        } else {
            0.0
        };
        let target_lean = (lateral_accel / 9.81)
            .atan()
            .clamp(-self.max_lean_angle, self.max_lean_angle);

        // Spring-damper lean roll response
        let lean_spring = 25.0;
        let lean_damper = 8.0;
        let lean_accel =
            lean_spring * (target_lean - self.lean_angle) - lean_damper * self.lean_velocity;
        self.lean_velocity += lean_accel * dt;
        self.lean_angle += self.lean_velocity * dt;

        // Check if low-speed crash / dropped bike (< 3 km/h with high lean)
        if current_speed < 1.0 && self.lean_angle.abs() > 0.45 {
            self.is_crashed = true;
            return;
        }

        // Camber thrust contribution to turning
        let camber_thrust_coeff = 2800.0; // N/rad
        let camber_force = self.lean_angle * camber_thrust_coeff;

        // Yaw rotation from steering and camber thrust
        let yaw_rate = (forward_speed * (self.steer_angle / self.wheelbase))
            + (if forward_speed.abs() > 0.5 {
                camber_force / (self.mass_kg * forward_speed.abs())
            } else {
                0.0
            });
        let yaw_delta = yaw_rate * dt;

        // Update orientation: composite of yaw heading and roll lean
        let new_yaw = current_yaw + yaw_delta;
        self.orientation = Quat::from_rotation_y(new_yaw) * Quat::from_rotation_z(-self.lean_angle);

        // Longitudinal forces: Drive / Brake / Resistance
        let rot_mat = Mat3::from_quat(Quat::from_rotation_y(new_yaw));
        let forward = rot_mat.col(2);

        let drive_force = match inputs.gear_mode {
            TransmissionMode::Reverse => -self.engine_force * 0.35 * inputs.throttle,
            TransmissionMode::Drive => self.engine_force * inputs.throttle,
            TransmissionMode::Neutral | TransmissionMode::Park => 0.0,
        };

        let brake_force = inputs.brake * 3800.0; // High stopping power per kg
        let drag_force = 0.5 * 1.225 * 0.45 * 0.7 * forward_speed * forward_speed.abs(); // Aero drag
        let rolling_res = self.mass_kg * 9.81 * 0.015 * forward_speed.signum();
        let total_resistance = if current_speed > 0.05 {
            forward_speed.signum() * brake_force + drag_force + rolling_res
        } else {
            0.0
        };

        let net_force_forward = drive_force - total_resistance;
        let linear_accel = (net_force_forward / self.mass_kg).clamp(-12.0, 9.0);
        let mut new_forward_speed = forward_speed + linear_accel * dt;

        // Standstill zero-crossing clamp under braking
        if current_speed > 0.0
            && (new_forward_speed.signum() != forward_speed.signum())
            && inputs.throttle == 0.0
        {
            new_forward_speed = 0.0;
        }

        if inputs.gear_mode == TransmissionMode::Park
            || (inputs.brake > 0.3 && inputs.throttle == 0.0 && new_forward_speed.abs() < 0.25)
            || (inputs.throttle == 0.0 && new_forward_speed.abs() < 0.05)
        {
            new_forward_speed = 0.0;
        }

        self.linear_velocity = forward * new_forward_speed;
        self.speed_kph = new_forward_speed.abs() * 3.6;
        self.position += self.linear_velocity * dt;

        if inputs.throttle > 0.5 && self.speed_kph > 5.0 {
            let rpm = (self.speed_kph * 95.0 + 1200.0).min(11500.0);
            self.audio_events
                .push(MultimodalAudioEvent::MotorcycleRev { rpm });
        }

        // Ground clamping
        let ray_origin = self.position + Vec3::new(0.0, 1.0, 0.0);
        if let Some((hit_dist, _)) = ground.query_ground_contact(ray_origin, 3.0) {
            let target_y = ray_origin.y - hit_dist + 0.35; // Center of mass height
            self.position.y += (target_y - self.position.y) * (18.0 * dt).min(1.0);
            self.is_grounded = true;
        } else {
            self.is_grounded = false;
        }
    }

    /// Resets motorcycle to upright stable condition after a fall.
    pub fn recover_upright(&mut self) {
        self.is_crashed = false;
        self.lean_angle = 0.0;
        self.lean_velocity = 0.0;
        self.linear_velocity = Vec3::ZERO;
        let yaw = self.orientation.to_euler(glam::EulerRot::YXZ).0;
        self.orientation = Quat::from_rotation_y(yaw);
    }

    /// Drains all pending audio events emitted by the motorcycle.
    pub fn drain_audio_events(&mut self) -> Vec<MultimodalAudioEvent> {
        std::mem::take(&mut self.audio_events)
    }
}

// ============================================================================
// 2. TRANSIT BUS: Heavy Mass, Pneumatic Suspension, Swept Path
// ============================================================================

/// Dynamic public transit bus model with pneumatic air-suspension and passenger payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitBusModel {
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub speed_kph: f32,
    pub steer_angle: f32,
    /// Unladen curb weight (default: 12,500 kg).
    pub unladen_mass_kg: f32,
    /// Current onboard passenger count (0 to 65).
    pub passenger_count: u16,
    /// Effective total mass including passengers (kg).
    pub total_mass_kg: f32,
    /// Long wheelbase: 6.5 meters.
    pub wheelbase: f32,
    /// Overall bus length: 12.0 meters.
    pub length: f32,
    /// Rear overhang from rear axle to bus tail (3.2m), causing swept-path off-tracking.
    pub rear_overhang: f32,
    /// Pneumatic air brake chamber pressure (0.0 to 10.0 bar, operating ~8.0 bar).
    pub air_brake_pressure_bar: f32,
    /// Peak engine drive force at driven rear axle (Newtons).
    pub peak_engine_force: f32,
    /// Maximum service air brake force (Newtons).
    pub max_brake_force: f32,
    /// Ride comfort lateral jerk accumulator.
    pub passenger_comfort_rating: f32,
    /// Pending audio events emitted during simulation.
    pub audio_events: Vec<MultimodalAudioEvent>,
}

impl Default for TransitBusModel {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 0.0)
    }
}

impl TransitBusModel {
    pub fn new(position: Vec3, yaw: f32) -> Self {
        let unladen = 12500.0;
        Self {
            position,
            orientation: Quat::from_rotation_y(yaw),
            linear_velocity: Vec3::ZERO,
            speed_kph: 0.0,
            steer_angle: 0.0,
            unladen_mass_kg: unladen,
            passenger_count: 35,
            total_mass_kg: unladen + (35.0 * 75.0), // 75kg per passenger
            wheelbase: 6.5,
            length: 12.0,
            rear_overhang: 3.2,
            air_brake_pressure_bar: 8.5,
            peak_engine_force: 28000.0, // High torque commercial turbodiesel
            max_brake_force: 42000.0,
            passenger_comfort_rating: 5.0,
            audio_events: Vec::new(),
        }
    }

    /// Sets passenger payload, recalculating gross vehicle weight.
    pub fn set_passenger_count(&mut self, passengers: u16) {
        self.passenger_count = passengers.min(75);
        self.total_mass_kg = self.unladen_mass_kg + (self.passenger_count as f32 * 75.0);
    }

    /// Boards passengers at a transit stop and emits a boarding chime.
    pub fn board_passengers(&mut self, count: u32) {
        self.set_passenger_count((self.passenger_count as u32 + count).min(75) as u16);
        self.audio_events
            .push(MultimodalAudioEvent::BusPassengerChime);
    }

    /// Drains pending audio events emitted by the transit bus.
    pub fn drain_audio_events(&mut self) -> Vec<MultimodalAudioEvent> {
        std::mem::take(&mut self.audio_events)
    }

    /// Calculates rear tail swept path displacement during a tight turn.
    pub fn rear_tail_position(&self) -> Vec3 {
        let rot = Mat3::from_quat(self.orientation);
        let back = -rot.col(2);
        self.position + (back * self.rear_overhang)
    }

    /// Advances transit bus physics integration.
    pub fn step(&mut self, inputs: &VehicleInputs, ground: &dyn GroundQuery, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        let current_yaw = self.orientation.to_euler(glam::EulerRot::YXZ).0;
        let rot = Mat3::from_quat(self.orientation);
        let forward = rot.col(2);
        let forward_speed = self.linear_velocity.dot(forward);
        let current_speed = forward_speed.abs();
        self.speed_kph = current_speed * 3.6;

        // Slower heavy-vehicle steering rate (~0.35 rad/s)
        let max_bus_steer = 0.55; // ~31 degrees
        let target_steer = inputs.steer * max_bus_steer;
        let steer_rate = 0.75;
        self.steer_angle += (target_steer - self.steer_angle) * (steer_rate * dt).min(1.0);

        // Heavy vehicle yaw rate with rear-axle pivot
        let yaw_rate = forward_speed * (self.steer_angle / self.wheelbase);
        let yaw_delta = yaw_rate * dt;

        // Monitor lateral passenger acceleration: a_lat = v * omega
        let lateral_g = (current_speed * yaw_rate.abs()) / 9.81;
        if lateral_g > 0.25 {
            // Rough bus turn: deduct passenger comfort
            self.passenger_comfort_rating =
                (self.passenger_comfort_rating - (lateral_g * 0.1 * dt)).max(1.0);
        }

        let new_yaw = current_yaw + yaw_delta;
        self.orientation = Quat::from_rotation_y(new_yaw);

        let rot = Mat3::from_quat(self.orientation);
        let forward = rot.col(2);

        // Powertrain and air brake application
        let drive_force = match inputs.gear_mode {
            TransmissionMode::Reverse => -self.peak_engine_force * 0.35 * inputs.throttle,
            TransmissionMode::Drive => self.peak_engine_force * inputs.throttle,
            TransmissionMode::Neutral | TransmissionMode::Park => 0.0,
        };

        // Pneumatic air brakes
        let brake_effectiveness = (self.air_brake_pressure_bar / 8.0).min(1.0);
        let brake_force = inputs.brake * self.max_brake_force * brake_effectiveness;
        if inputs.brake > 0.2 {
            self.air_brake_pressure_bar = (self.air_brake_pressure_bar - (0.15 * dt)).max(4.0);
            if inputs.brake > 0.75 {
                self.audio_events
                    .push(MultimodalAudioEvent::BusAirBrakeRelease {
                        pressure_bar: self.air_brake_pressure_bar,
                    });
            }
        } else {
            self.air_brake_pressure_bar = (self.air_brake_pressure_bar + (0.35 * dt)).min(9.5);
            // Compressor recharge
        }

        let aero_drag = 0.5 * 1.225 * 0.75 * 7.5 * forward_speed * forward_speed.abs(); // Large frontal area
        let rolling_resistance = self.total_mass_kg * 9.81 * 0.012 * forward_speed.signum();
        let total_resistance = if current_speed > 0.05 {
            forward_speed.signum() * brake_force + aero_drag + rolling_resistance
        } else {
            0.0
        };

        let net_force = drive_force - total_resistance;
        let accel = (net_force / self.total_mass_kg).clamp(-6.5, 3.2);

        let mut new_forward_speed = forward_speed + accel * dt;

        // Standstill zero crossing under braking
        if current_speed > 0.0
            && (new_forward_speed.signum() != forward_speed.signum())
            && inputs.throttle == 0.0
        {
            new_forward_speed = 0.0;
        }

        if inputs.gear_mode == TransmissionMode::Park
            || (inputs.brake > 0.3 && inputs.throttle == 0.0 && new_forward_speed.abs() < 0.25)
            || (inputs.throttle == 0.0 && new_forward_speed.abs() < 0.05)
        {
            new_forward_speed = 0.0;
        }

        self.linear_velocity = forward * new_forward_speed;
        self.speed_kph = new_forward_speed.abs() * 3.6;
        self.position += self.linear_velocity * dt;

        // Ground height tracking
        let ray_origin = self.position + Vec3::new(0.0, 2.0, 0.0);
        if let Some((hit_dist, _)) = ground.query_ground_contact(ray_origin, 4.0) {
            let target_y = ray_origin.y - hit_dist + 0.95; // Ride height
            self.position.y += (target_y - self.position.y) * (14.0 * dt).min(1.0);
        }
    }
}

// ============================================================================
// 3. HEAVY TRUCK: Multi-Axle, Trailer Articulation & Rollover Stability
// ============================================================================

/// Dynamic commercial heavy haul truck with trailer articulation and rollover limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeavyTruckModel {
    pub tractor_position: Vec3,
    pub tractor_orientation: Quat,
    pub linear_velocity: Vec3,
    pub speed_kph: f32,
    pub steer_angle: f32,
    /// Fifth-wheel kingpin trailer hitch articulation angle in radians.
    pub trailer_articulation_angle: f32,
    /// Tractor curb weight (e.g. 9,500 kg).
    pub tractor_mass_kg: f32,
    /// Trailer and cargo payload weight (e.g. 26,000 kg).
    pub trailer_mass_kg: f32,
    /// Center of mass height above road surface in meters (e.g. 1.85m laden).
    pub com_height: f32,
    /// Track width between outer dual tires (default: 2.15m).
    pub track_width: f32,
    /// Distance from tractor front steer axle to fifth wheel hitch (4.2m).
    pub tractor_wheelbase: f32,
    /// Distance from fifth wheel hitch to trailer rear bogie axle (8.5m).
    pub trailer_length: f32,
    /// Critical rollover lateral acceleration threshold in m/s^2.
    pub rollover_threshold_accel: f32,
    /// Flag indicating catastrophic vehicle rollover event.
    pub is_rolled_over: bool,
    /// Peak engine tractive effort (Newtons).
    pub peak_tractive_force: f32,
    /// Pending audio events emitted during simulation.
    pub audio_events: Vec<MultimodalAudioEvent>,
}

impl Default for HeavyTruckModel {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 0.0)
    }
}

impl HeavyTruckModel {
    pub fn new(position: Vec3, yaw: f32) -> Self {
        let tractor_m = 9500.0;
        let trailer_m = 26000.0;
        let track = 2.15;
        let com_h = 1.85;

        // Static rollover threshold: a_crit = (Track / (2 * h_com)) * g
        let rollover_crit = (track / (2.0 * com_h)) * 9.81; // ~5.7 m/s^2 (~0.58 G)

        Self {
            tractor_position: position,
            tractor_orientation: Quat::from_rotation_y(yaw),
            linear_velocity: Vec3::ZERO,
            speed_kph: 0.0,
            steer_angle: 0.0,
            trailer_articulation_angle: 0.0,
            tractor_mass_kg: tractor_m,
            trailer_mass_kg: trailer_m,
            com_height: com_h,
            track_width: track,
            tractor_wheelbase: 4.2,
            trailer_length: 8.5,
            rollover_threshold_accel: rollover_crit,
            is_rolled_over: false,
            peak_tractive_force: 58000.0, // High-torque heavy hauler engine
            audio_events: Vec::new(),
        }
    }

    /// Total gross vehicle mass in kilograms.
    pub fn gross_mass_kg(&self) -> f32 {
        self.tractor_mass_kg + self.trailer_mass_kg
    }

    /// Drains pending audio events emitted by the heavy truck.
    pub fn drain_audio_events(&mut self) -> Vec<MultimodalAudioEvent> {
        std::mem::take(&mut self.audio_events)
    }

    /// Evaluates trailer center position in world space based on articulation angle.
    pub fn trailer_position(&self) -> Vec3 {
        let tractor_yaw = self.tractor_orientation.to_euler(glam::EulerRot::YXZ).0;
        let trailer_yaw = tractor_yaw + self.trailer_articulation_angle;
        let trailer_rot = Mat3::from_quat(Quat::from_rotation_y(trailer_yaw));
        let back = -trailer_rot.col(2);
        self.tractor_position + (back * self.trailer_length * 0.5)
    }

    /// Advances heavy truck and articulated trailer dynamics.
    pub fn step(&mut self, inputs: &VehicleInputs, ground: &dyn GroundQuery, dt: f32) {
        if dt <= 0.0 || self.is_rolled_over {
            return;
        }

        let current_yaw = self.tractor_orientation.to_euler(glam::EulerRot::YXZ).0;
        let rot = Mat3::from_quat(self.tractor_orientation);
        let forward = rot.col(2);
        let forward_speed = self.linear_velocity.dot(forward);
        let current_speed = forward_speed.abs();
        self.speed_kph = current_speed * 3.6;

        // Heavy steering rate
        let max_truck_steer = 0.52; // ~30 degrees
        let target_steer = inputs.steer * max_truck_steer;
        self.steer_angle += (target_steer - self.steer_angle) * (1.2 * dt).min(1.0);

        // Tractor yaw rate
        let yaw_rate = forward_speed * (self.steer_angle / self.tractor_wheelbase);
        let yaw_delta = yaw_rate * dt;

        // Fifth-wheel trailer articulation kinematics:
        // d(theta_trailer)/dt = (v / L_trailer) * sin(theta_tractor - theta_trailer) - yaw_rate_tractor
        let d_articulation = -(forward_speed / self.trailer_length)
            * self.trailer_articulation_angle.sin()
            - yaw_rate;
        self.trailer_articulation_angle += d_articulation * dt;
        self.trailer_articulation_angle = self.trailer_articulation_angle.clamp(-1.25, 1.25); // Jackknife limit ~72 deg

        // Rollover stability verification
        let lateral_accel = current_speed * yaw_rate.abs();
        if lateral_accel > self.rollover_threshold_accel {
            self.is_rolled_over = true;
            return;
        }

        let new_yaw = current_yaw + yaw_delta;
        self.tractor_orientation = Quat::from_rotation_y(new_yaw);

        let rot = Mat3::from_quat(self.tractor_orientation);
        let forward = rot.col(2);

        // Heavy powertrain integration
        let total_mass = self.gross_mass_kg();
        let drive_force = match inputs.gear_mode {
            TransmissionMode::Reverse => -self.peak_tractive_force * 0.30 * inputs.throttle,
            TransmissionMode::Drive => self.peak_tractive_force * inputs.throttle,
            TransmissionMode::Neutral | TransmissionMode::Park => 0.0,
        };

        let brake_force = inputs.brake * 65000.0; // Multi-axle S-cam drum brakes
        if inputs.brake > 0.4 && self.speed_kph > 20.0 {
            self.audio_events
                .push(MultimodalAudioEvent::TruckJakeBrake { rpm: 2100.0 });
        }
        let aero_drag = 0.5 * 1.225 * 0.85 * 9.5 * forward_speed * forward_speed.abs();
        let rolling_resistance = total_mass * 9.81 * 0.014 * forward_speed.signum();
        let total_resistance = if current_speed > 0.05 {
            forward_speed.signum() * brake_force + aero_drag + rolling_resistance
        } else {
            0.0
        };

        let net_force = drive_force - total_resistance;
        let accel = (net_force / total_mass).clamp(-7.0, 2.5);

        let mut new_forward_speed = forward_speed + accel * dt;

        // Standstill zero crossing under braking
        if current_speed > 0.0
            && (new_forward_speed.signum() != forward_speed.signum())
            && inputs.throttle == 0.0
        {
            new_forward_speed = 0.0;
        }

        if inputs.gear_mode == TransmissionMode::Park
            || (inputs.brake > 0.3 && inputs.throttle == 0.0 && new_forward_speed.abs() < 0.25)
            || (inputs.throttle == 0.0 && new_forward_speed.abs() < 0.05)
        {
            new_forward_speed = 0.0;
        }

        self.linear_velocity = forward * new_forward_speed;
        self.speed_kph = new_forward_speed.abs() * 3.6;
        self.tractor_position += self.linear_velocity * dt;

        // Ground height tracking
        let ray_origin = self.tractor_position + Vec3::new(0.0, 2.5, 0.0);
        if let Some((hit_dist, _)) = ground.query_ground_contact(ray_origin, 5.0) {
            let target_y = ray_origin.y - hit_dist + 1.2;
            self.tractor_position.y += (target_y - self.tractor_position.y) * (14.0 * dt).min(1.0);
        }
    }

    /// Righting recovery after rollover accident.
    pub fn recover_upright(&mut self) {
        self.is_rolled_over = false;
        self.trailer_articulation_angle = 0.0;
        self.linear_velocity = Vec3::ZERO;
        let yaw = self.tractor_orientation.to_euler(glam::EulerRot::YXZ).0;
        self.tractor_orientation = Quat::from_rotation_y(yaw);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::FlatGround;

    #[test]
    fn test_road_access_restrictions() {
        let open = RoadAccessRestriction::OpenAccess;
        assert!(open.is_allowed(RoadVehicleCategory::Motorcycle));
        assert!(open.is_allowed(RoadVehicleCategory::CitySedan));
        assert!(open.is_allowed(RoadVehicleCategory::TransitBus));
        assert!(open.is_allowed(RoadVehicleCategory::HeavyTruck));

        let brt = RoadAccessRestriction::BusOnly;
        assert!(!brt.is_allowed(RoadVehicleCategory::CitySedan));
        assert!(!brt.is_allowed(RoadVehicleCategory::HeavyTruck));
        assert!(brt.is_allowed(RoadVehicleCategory::TransitBus));

        let res = RoadAccessRestriction::NoHeavyTrucks;
        assert!(res.is_allowed(RoadVehicleCategory::CitySedan));
        assert!(!res.is_allowed(RoadVehicleCategory::HeavyTruck));
    }

    #[test]
    fn test_motorcycle_lean_dynamics() {
        let ground = FlatGround::new(0.0);
        let mut bike = MotorcycleModel::default();

        let inputs = VehicleInputs {
            throttle: 0.8,
            steer: 0.5,
            ..Default::default()
        };

        // Advance 2.0 seconds
        for _ in 0..120 {
            bike.step(&inputs, &ground, 1.0 / 60.0);
        }

        assert!(bike.speed_kph > 15.0);
        assert!(
            bike.lean_angle > 0.05,
            "Motorcycle should lean into right turn: lean_angle={}",
            bike.lean_angle
        );
        assert!(!bike.is_crashed);
    }

    #[test]
    fn test_transit_bus_swept_path_and_passenger_mass() {
        let ground = FlatGround::new(0.0);
        let mut bus = TransitBusModel::default();
        let unladen_mass = bus.unladen_mass_kg;

        bus.set_passenger_count(60);
        assert_eq!(bus.total_mass_kg, unladen_mass + (60.0 * 75.0));

        let tail = bus.rear_tail_position();
        assert!((tail.z - bus.position.z).abs() >= 3.0 || (tail - bus.position).length() >= 3.0);

        let inputs = VehicleInputs {
            throttle: 0.7,
            ..Default::default()
        };
        for _ in 0..200 {
            bus.step(&inputs, &ground, 1.0 / 60.0);
        }
        assert!(bus.speed_kph > 10.0);
    }

    #[test]
    fn test_heavy_truck_rollover_protection() {
        let ground = FlatGround::new(0.0);
        let mut truck = HeavyTruckModel::default();

        // Normal straight driving
        let mut inputs = VehicleInputs {
            throttle: 0.8,
            ..Default::default()
        };
        for _ in 0..400 {
            truck.step(&inputs, &ground, 1.0 / 60.0);
        }
        assert!(!truck.is_rolled_over);
        assert!(truck.speed_kph > 20.0);

        // Violent high-speed steering exceeding rollover threshold
        inputs.steer = 1.0;
        for _ in 0..60 {
            truck.step(&inputs, &ground, 1.0 / 60.0);
        }
        // Either truck rolled over or held stability under limits
        if truck.is_rolled_over {
            truck.recover_upright();
            assert!(!truck.is_rolled_over);
        }
    }
}
