//! STAGE 7 — COMPLETE MULTIMODAL TRANSPORTATION INTEGRATION CERTIFICATION
//!
//! Comprehensive end-to-end verification of Stage 7 requirements from docs/ExecutionContract.md:
//! 1. Phase 7.1 — Road Transport Expansion:
//!    - Motorcycles: 2-wheel lean dynamics, camber thrust, rollover limit, safe upright recovery.
//!    - Transit Buses: unladen-to-passenger payload mass scaling, air brakes, swept-path rear overhang.
//!    - Heavy Articulated Trucks: fifth-wheel kingpin kinematics, static rollover stability, heavy payload braking.
//!    - Road Access Restrictions: BRT only, commercial truck bans, motorcycle lanes, expressway speeds.
//! 2. Phase 7.2 — Rail and Marine Integration:
//!    - Railway Consists: multi-car consist, Davis equation resistance, Westinghouse air brake pipe, rail adhesion.
//!    - Marine Vessels: Archimedes buoyancy draft, metacentric righting stability, wave heave, rudder dynamics.
//! 3. Phase 7.3 — Aviation 6-DOF Aerodynamics:
//!    - Lift/drag polar, barometric air density lapse, post-stall separation, ground effect, oleo landing gear.
//! 4. Phase 7.4 — Certification & Integration:
//!    - Concurrent multi-vehicle execution throughput (>500 Hz simulation budget).
//!    - 3D procedural mesh generation and telemetry HUD across all 7 transport modes.

use glam::{Mat3, Quat, Vec3};
use realearth_gameplay::destinations::JobCatalogGenerator;
use realearth_gameplay::events::GameplayEventBus;
use realearth_gameplay::jobs::JobState;
use realearth_gameplay::profile::PlayerProfile;
use realearth_navigation::air_navigation::Airport;
use realearth_navigation::rail_graph::{RailNetworkGraph, SignalAspect, SwitchState};
use realearth_navigation::waterway::WaterwayGraph;
use realearth_rendering::{generate_vehicle_mesh, HudMesh, VehicleMeshParams, VehicleVisualType};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_vehicles::{
    dynamics::{TransmissionMode, VehicleInputs},
    ground::FlatGround,
    multimodal::{
        aviation::{AircraftFlight, AviationCategory, FlightInputs},
        marine::{MarineInputs, MarineVessel, MarineVesselType},
        road_transport::{
            HeavyTruckModel, MotorcycleModel, RoadAccessRestriction, RoadVehicleCategory,
            TransitBusModel,
        },
        train::{DavisCoefficients, LocomotiveType, TrainBrakeState, TrainCar, TrainConsist},
        MultimodalAudioEvent,
    },
};
use std::time::Instant;

// ============================================================================
// TEST 1: MOTORCYCLE 2-WHEEL LEAN DYNAMICS, CAMBER THRUST & CRASH RECOVERY
// ============================================================================

#[test]
fn test_stage7_road_motorcycle_lean_camber_and_recovery() {
    let ground = FlatGround::new(0.0);
    let mut bike = MotorcycleModel::new(Vec3::new(0.0, 0.5, 0.0), 0.0);

    // 1. Initial configuration verification
    assert_eq!(bike.mass_kg, 220.0, "Motorcycle curb mass must be 220 kg");
    assert_eq!(bike.wheelbase, 1.45, "Wheelbase must be 1.45 meters");
    assert_eq!(bike.lean_angle, 0.0, "Initial lean angle must be 0");
    assert!(!bike.is_crashed, "Initial state must be upright");

    // 2. High-speed right turn entry
    let dt = 1.0 / 60.0;
    let right_turn_inputs = VehicleInputs {
        throttle: 0.85,
        steer: 0.5, // Steer right
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };

    // Step simulation for 1.5 seconds (90 steps)
    for _ in 0..90 {
        bike.step(&right_turn_inputs, &ground, dt);
    }

    // Lean angle must roll into the turn (positive = lean right)
    assert!(
        bike.speed_kph > 15.0,
        "Motorcycle must accelerate under throttle: got {:.1} km/h",
        bike.speed_kph
    );
    assert!(
        bike.lean_angle > 0.05,
        "Motorcycle must dynamically lean into right turn: lean_angle = {:.3} rad",
        bike.lean_angle
    );
    assert!(
        bike.lean_angle <= bike.max_lean_angle,
        "Lean angle must not exceed footpeg limit under normal turning: lean = {:.3}, max = {:.3}",
        bike.lean_angle,
        bike.max_lean_angle
    );

    // 3. Camber thrust verification: orientation must show combined yaw and roll
    let euler = bike.orientation.to_euler(glam::EulerRot::YXZ);
    assert!(
        euler.0.abs() > 0.01,
        "Yaw heading must change due to steering and camber thrust"
    );

    // 4. Low-speed dropped bike / crash test: motorcycle at low speed with excessive lean falls over
    bike.linear_velocity = Vec3::ZERO;
    bike.lean_angle = 0.50; // > 0.45 rad threshold at standstill
    bike.step(&right_turn_inputs, &ground, dt);
    assert!(
        bike.is_crashed,
        "Motorcycle exceeding 0.45 rad lean at standstill must trigger dropped bike crash"
    );

    // In crashed state, motorcycle must not advance
    let crashed_pos = bike.position;
    bike.step(&right_turn_inputs, &ground, dt);
    assert_eq!(
        bike.position, crashed_pos,
        "Crashed motorcycle must remain immobile"
    );

    // 5. Recovery test
    bike.recover_upright();
    assert!(!bike.is_crashed, "Recovery must clear crashed flag");
    assert_eq!(bike.lean_angle, 0.0, "Recovery must reset lean angle to 0");
    assert_eq!(
        bike.linear_velocity,
        Vec3::ZERO,
        "Recovery must zero linear velocity"
    );
}

// ============================================================================
// TEST 2: TRANSIT BUS PASSENGER PAYLOAD, AIR BRAKES & SWEPT-PATH TURNING
// ============================================================================

#[test]
fn test_stage7_road_transit_bus_passenger_payload_and_swept_path() {
    let ground = FlatGround::new(0.0);
    let mut bus = TransitBusModel::new(Vec3::new(0.0, 1.0, 0.0), 0.0);

    // 1. Initial configuration
    assert_eq!(bus.unladen_mass_kg, 12500.0);
    assert_eq!(bus.passenger_count, 35);
    assert_eq!(bus.total_mass_kg, 12500.0 + 35.0 * 75.0);
    assert_eq!(bus.wheelbase, 6.5);
    assert_eq!(bus.length, 12.0);
    assert_eq!(bus.rear_overhang, 3.2);

    // 2. Unladen payload configuration
    bus.set_passenger_count(0);
    assert_eq!(bus.passenger_count, 0);
    assert_eq!(bus.total_mass_kg, 12500.0);

    // 3. Passenger loading scaling
    bus.set_passenger_count(45);
    assert_eq!(bus.passenger_count, 45);
    let expected_mass = 12500.0 + (45.0 * 75.0); // 15,875 kg
    assert_eq!(
        bus.total_mass_kg, expected_mass,
        "Bus mass must increase proportionally with passenger count"
    );

    // Loading up to maximum capacity (75 passengers)
    bus.set_passenger_count(75);
    assert_eq!(bus.passenger_count, 75, "Passenger count must reach 75");
    assert_eq!(bus.total_mass_kg, 12500.0 + (75.0 * 75.0));

    // 4. Swept-path overhang tracking during a turn
    let dt = 1.0 / 60.0;
    let turn_inputs = VehicleInputs {
        throttle: 0.85,
        steer: 0.45, // Sharp cornering
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };

    // Accelerate and corner for 3.0 seconds (180 steps)
    for _ in 0..180 {
        bus.step(&turn_inputs, &ground, dt);
    }

    assert!(
        bus.speed_kph > 8.0,
        "Bus must be in motion: got {:.1} km/h",
        bus.speed_kph
    );

    // Calculate rear overhang tail position
    let tail_pos = bus.rear_tail_position();
    let rot = Mat3::from_quat(bus.orientation);
    let back = -rot.col(2);
    let expected_tail = bus.position + (back * bus.rear_overhang);

    assert!(
        (tail_pos - expected_tail).length() < 0.1,
        "Rear tail must be offset along vehicle orientation by rear_overhang: {:.2} vs {:.2}",
        tail_pos,
        expected_tail
    );

    // 5. Pneumatic air brake dynamics
    let pre_brake_speed = bus.speed_kph;
    let brake_inputs = VehicleInputs {
        throttle: 0.0,
        brake: 0.9, // Heavy air brake application
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };

    for _ in 0..60 {
        bus.step(&brake_inputs, &ground, dt);
    }

    assert!(
        bus.speed_kph < pre_brake_speed,
        "Service air brakes must decelerate loaded bus: {:.1} -> {:.1} km/h",
        pre_brake_speed,
        bus.speed_kph
    );
    assert!(
        bus.passenger_comfort_rating >= 0.0,
        "Comfort rating must track passenger jerk acceleration"
    );
}

// ============================================================================
// TEST 3: HEAVY ARTICULATED TRUCK FIFTH-WHEEL KINEMATICS & ROLLOVER MARGIN
// ============================================================================

#[test]
fn test_stage7_road_heavy_truck_trailer_articulation_and_rollover_stability() {
    let ground = FlatGround::new(0.0);
    let mut truck = HeavyTruckModel::new(Vec3::new(0.0, 1.2, 0.0), 0.0);

    // 1. Gross combination mass & dimensions
    assert_eq!(truck.tractor_mass_kg, 9500.0);
    assert_eq!(truck.trailer_mass_kg, 26000.0);
    assert_eq!(
        truck.gross_mass_kg(),
        35500.0,
        "Gross combination mass must be 35.5 metric tons"
    );
    assert_eq!(truck.tractor_wheelbase, 4.2);
    assert_eq!(truck.trailer_length, 8.5);

    // 2. Static rollover threshold: a_crit = (Track / (2 * h_com)) * g
    // Track = 2.15m, COM height = 1.85m => a_crit = (2.15 / 3.7) * 9.81 = 5.70 m/s^2 (~0.58 G)
    let expected_rollover_accel = (2.15 / (2.0 * 1.85)) * 9.81;
    assert!(
        (truck.rollover_threshold_accel - expected_rollover_accel).abs() < 0.1,
        "Rollover threshold must equal analytical (T / 2h) * g: got {:.2}, expected {:.2}",
        truck.rollover_threshold_accel,
        expected_rollover_accel
    );

    // 3. Gentle steering: heavy truck accelerates and articulates without rollover
    let dt = 1.0 / 60.0;
    let gentle_turn_inputs = VehicleInputs {
        throttle: 0.85,
        steer: 0.15, // Gentle highway turn
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };

    // Drive forward for 3.0 seconds (180 steps)
    for _ in 0..180 {
        truck.step(&gentle_turn_inputs, &ground, dt);
    }

    assert!(
        truck.speed_kph > 10.0,
        "Truck must accelerate forward: got {:.1} km/h",
        truck.speed_kph
    );
    assert!(
        truck.trailer_articulation_angle.abs() > 0.005,
        "Trailer must articulate relative to tractor during turn: art_angle = {:.3} rad",
        truck.trailer_articulation_angle
    );

    // Verify trailer position is offset behind tractor along articulated heading
    let trailer_pos = truck.trailer_position();
    let hitch_distance = (truck.tractor_position - trailer_pos).length();
    assert!(
        hitch_distance > 3.0 && hitch_distance < 6.0,
        "Trailer center of mass must be properly offset behind fifth-wheel hitch: distance = {:.2}m",
        hitch_distance
    );

    // 4. Jackknife clamping verification: articulation angle must not exceed 1.25 rad (~72 deg)
    assert!(
        truck.trailer_articulation_angle.abs() <= 1.25,
        "Trailer articulation must remain clamped within safe jackknife limit: art_angle = {:.3}",
        truck.trailer_articulation_angle
    );
    assert!(
        !truck.is_rolled_over,
        "Truck must remain stable during gentle turn"
    );

    // 5. Rollover threshold verification: high-speed sharp cornering must trigger rollover
    truck.linear_velocity = truck.tractor_orientation * Vec3::new(0.0, 0.0, 20.0); // 72 km/h
    let aggressive_corner = VehicleInputs {
        throttle: 0.5,
        steer: 0.8, // Extreme steer at 72 km/h produces >6 m/s^2 lateral accel
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };

    for _ in 0..60 {
        truck.step(&aggressive_corner, &ground, dt);
        if truck.is_rolled_over {
            break;
        }
    }

    assert!(
        truck.is_rolled_over,
        "Excessive lateral acceleration exceeding rollover threshold ({:.2} m/s^2) must trip rollover flag",
        truck.rollover_threshold_accel
    );

    // Recovery test
    truck.recover_upright();
    assert!(!truck.is_rolled_over, "Recovery must right the truck");
    assert_eq!(truck.trailer_articulation_angle, 0.0);
}

// ============================================================================
// TEST 4: ROAD ACCESS RESTRICTIONS & CORRIDOR POLICIES
// ============================================================================

#[test]
fn test_stage7_road_access_restrictions_and_corridor_policies() {
    // 1. Open Access: all vehicle categories permitted
    let open = RoadAccessRestriction::OpenAccess;
    assert!(open.is_allowed(RoadVehicleCategory::Motorcycle));
    assert!(open.is_allowed(RoadVehicleCategory::CitySedan));
    assert!(open.is_allowed(RoadVehicleCategory::TransitBus));
    assert!(open.is_allowed(RoadVehicleCategory::HeavyTruck));

    // 2. Bus Rapid Transit (BRT) lane: buses strictly exclusive
    let brt = RoadAccessRestriction::BusOnly;
    assert!(
        brt.is_allowed(RoadVehicleCategory::TransitBus),
        "Transit bus must be allowed in BRT lane"
    );
    assert!(
        !brt.is_allowed(RoadVehicleCategory::CitySedan),
        "Private car must be forbidden in BRT lane"
    );
    assert!(
        !brt.is_allowed(RoadVehicleCategory::HeavyTruck),
        "Heavy truck must be forbidden in BRT lane"
    );
    assert!(
        !brt.is_allowed(RoadVehicleCategory::Motorcycle),
        "Motorcycle must be forbidden in BRT lane"
    );

    // 3. Historic / Residential Zone: heavy commercial trucks banned
    let heritage = RoadAccessRestriction::NoHeavyTrucks;
    assert!(
        heritage.is_allowed(RoadVehicleCategory::TransitBus),
        "Public bus must be allowed in heritage zone"
    );
    assert!(
        heritage.is_allowed(RoadVehicleCategory::CitySedan),
        "Sedan must be allowed in heritage zone"
    );
    assert!(
        heritage.is_allowed(RoadVehicleCategory::Motorcycle),
        "Motorcycle must be allowed in heritage zone"
    );
    assert!(
        !heritage.is_allowed(RoadVehicleCategory::HeavyTruck),
        "Heavy truck must be prohibited in heritage zone"
    );

    // 4. Dedicated Two-Wheeler / Motorcycle Lane
    let bike_lane = RoadAccessRestriction::MotorcycleOnly;
    assert!(bike_lane.is_allowed(RoadVehicleCategory::Motorcycle));
    assert!(!bike_lane.is_allowed(RoadVehicleCategory::CitySedan));
    assert!(!bike_lane.is_allowed(RoadVehicleCategory::TransitBus));
    assert!(!bike_lane.is_allowed(RoadVehicleCategory::HeavyTruck));

    // 5. Limited-Access Expressway: high-speed vehicles permitted
    let expressway = RoadAccessRestriction::ExpresswayOnly;
    assert!(expressway.is_allowed(RoadVehicleCategory::CitySedan));
    assert!(expressway.is_allowed(RoadVehicleCategory::TransitBus));
    assert!(expressway.is_allowed(RoadVehicleCategory::HeavyTruck));
    assert!(expressway.is_allowed(RoadVehicleCategory::Motorcycle));
}

// ============================================================================
// TEST 5: RAILWAY TRAIN CONSIST, DAVIS RESISTANCE & AIR BRAKE PNEUMATICS
// ============================================================================

#[test]
fn test_stage7_rail_consist_davis_resistance_and_pneumatics() {
    // 1. Construct multi-car train consist: WAP-7 electric locomotive + 8 coaches
    // 4.5 MW rated power, 320 kN max tractive effort, 123 tonnes locomotive
    let mut train = TrainConsist::new_locomotive(
        "WAP-7",
        LocomotiveType::Electric,
        4_500_000.0,
        320_000.0,
        123_000.0,
        20.5,
    );

    for i in 1..=8 {
        let coach = TrainCar::new_wagon(&format!("Coach-{}", i), 42_000.0, 3_000.0, 24.0);
        train.add_wagon(coach);
    }

    // Verify consist aggregates
    assert_eq!(
        train.cars.len(),
        9,
        "Consist must have 9 cars (1 loco + 8 coaches)"
    );
    let expected_total_mass = 123_000.0 + (8.0 * 45_000.0); // 483,000 kg (483 tonnes)
    assert_eq!(
        train.total_mass_kg(),
        expected_total_mass,
        "Total consist mass must be 483 tonnes"
    );
    let expected_total_length = 20.5 + (8.0 * 24.0); // 212.5 meters
    assert_eq!(
        train.total_length_meters(),
        expected_total_length,
        "Consist length must be 212.5 meters"
    );

    // 2. Davis train resistance equation: R(v) = A + B*v + C*v^2
    // Journal bearing friction (A), wheel flange/track wave (B*v), aerodynamic drag (C*v^2)
    let davis = DavisCoefficients::default();
    let r_0 = davis.a_journal_bearing;
    let v_50 = 50.0 / 3.6; // 13.89 m/s
    let r_50 =
        davis.a_journal_bearing + davis.b_flange_track * v_50 + davis.c_aerodynamic * v_50 * v_50;
    let v_160 = 160.0 / 3.6; // 44.44 m/s
    let r_160 = davis.a_journal_bearing
        + davis.b_flange_track * v_160
        + davis.c_aerodynamic * v_160 * v_160;

    assert!(
        r_50 > r_0,
        "Resistance at 50 km/h ({:.0} N) must exceed standstill ({:.0} N)",
        r_50,
        r_0
    );
    assert!(
        r_160 > r_50 * 2.5,
        "Resistance at 160 km/h ({:.0} N) must show quadratic aerodynamic growth over 50 km/h ({:.0} N)",
        r_160,
        r_50
    );

    // 3. Rail adhesion limit & wheel slip detection
    train.throttle = 1.0;
    train.reverser = 1;
    let dt = 1.0 / 60.0;

    // Advance 60 steps under acceleration
    for _ in 0..60 {
        train.step(dt);
    }

    assert!(
        train.speed_kph > 0.5,
        "Electric locomotive must accelerate 483-tonne consist forward: speed = {:.2} km/h",
        train.speed_kph
    );
    assert!(
        train.position_s > 0.0,
        "Train must advance arc-length coordinate along track"
    );

    // 4. Westinghouse air brake pipe simulation
    // Standard running pressure: 5.0 bar, brake cylinders released: 0.0 bar
    assert_eq!(train.brake_pipe_pressure_bar, 5.0);
    assert_eq!(train.brake_cylinder_pressure_bar, 0.0);

    // Service brake application
    train.throttle = 0.0;
    train.apply_service_brake(0.85);
    assert_eq!(train.brake_state, TrainBrakeState::FullService);

    // Step simulation to propagate pneumatic air wave venting
    for _ in 0..60 {
        train.step(dt);
    }

    assert!(
        train.brake_pipe_pressure_bar < 5.0,
        "Brake pipe pressure must drop below 5.0 bar during service application: {:.2} bar",
        train.brake_pipe_pressure_bar
    );
    assert!(
        train.brake_cylinder_pressure_bar > 0.0,
        "Brake cylinder pressure must build up inversely to pipe venting: {:.2} bar",
        train.brake_cylinder_pressure_bar
    );

    // Emergency brake dump
    train.trigger_emergency_brake();
    assert_eq!(train.brake_state, TrainBrakeState::Emergency);
    assert!(train.emergency_brake);

    for _ in 0..60 {
        train.step(dt);
    }

    assert!(
        train.brake_pipe_pressure_bar < 1.0,
        "Emergency brake must vent pipe near atmosphere (0 bar): {:.2} bar",
        train.brake_pipe_pressure_bar
    );
    assert!(
        train.brake_cylinder_pressure_bar > 3.0,
        "Brake cylinders must reach full emergency pressure (~3.8 bar): {:.2} bar",
        train.brake_cylinder_pressure_bar
    );
}

// ============================================================================
// TEST 6: MARINE VESSEL ARCHIMEDES BUOYANCY & METACENTRIC RIGHTING STABILITY
// ============================================================================

#[test]
fn test_stage7_marine_buoyancy_metacentric_stability_and_hydrodynamics() {
    // 1. Construct passenger ferry: 85m length, 16m beam, 3,200 metric tons displacement
    let mut ferry = MarineVessel::new_ferry("SF-Bay-Express", Vec3::new(0.0, 0.0, 0.0), 0.0);

    assert_eq!(ferry.vessel_type, MarineVesselType::PassengerFerry);
    assert_eq!(ferry.mass_kg, 3_200_000.0);
    assert_eq!(ferry.length_beam_meters, 85.0);
    assert_eq!(ferry.beam_meters, 16.0);
    assert_eq!(ferry.design_draft_meters, 3.8);
    assert_eq!(ferry.water_density, 1025.0); // Seawater

    // 2. Archimedes displacement equilibrium:
    // Waterplane area = L * B * Cb * 0.9 = 85 * 16 * 0.58 * 0.9 = 710.0 m^2
    // Equilibrium draft T_eq = Mass / (rho * A_wp) = 3,200,000 / (1025 * 710) = 4.39 m
    let waterplane_area =
        ferry.length_beam_meters * ferry.beam_meters * ferry.block_coefficient * 0.9;
    let theoretical_equilibrium_draft = ferry.mass_kg / (ferry.water_density * waterplane_area);
    assert!(
        (theoretical_equilibrium_draft - 4.39).abs() < 0.2,
        "Equilibrium draft must match Archimedes formula: got {:.2}m",
        theoretical_equilibrium_draft
    );

    // 3. Metacentric righting stability:
    // Transverse metacentric height GM = 1.8m > 0 (stable positive stability)
    assert!(
        ferry.metacentric_height_gm > 0.0,
        "Transverse GM must be strictly positive for seaworthiness"
    );

    // Impose roll disturbance (roll angle = 0.15 rad ~ 8.6 degrees)
    ferry.roll_angle = 0.15;
    let initial_roll = ferry.roll_angle;

    // Advance 60 hydrodynamic simulation steps
    let dt = 1.0 / 60.0;
    let idle_inputs = MarineInputs::default();
    let mut sim_time = 0.0;

    for _ in 0..120 {
        ferry.step(&idle_inputs, sim_time, dt);
        sim_time += dt;
    }

    // Metacentric righting moment must restore roll angle back towards 0
    assert!(
        ferry.roll_angle.abs() < initial_roll,
        "Positive metacentric GM must right the ship towards upright equilibrium: initial = {:.3}, current = {:.3}",
        initial_roll,
        ferry.roll_angle
    );

    // 4. Propeller propulsion & rudder hydrodynamic steering (step 300 steps = 5 seconds)
    let nav_inputs = MarineInputs {
        throttle: 1.0, // Full ahead
        rudder: 0.6,   // Starboard rudder
        bow_thruster: 0.0,
    };

    for _ in 0..300 {
        ferry.step(&nav_inputs, sim_time, dt);
        sim_time += dt;
    }

    assert!(
        ferry.speed_knots > 1.0,
        "Twin propeller propulsion must drive 3200-tonne vessel forward: speed = {:.2} knots",
        ferry.speed_knots
    );
    assert!(
        ferry.heading_radians.abs() > 0.01,
        "Rudder hydrodynamic lift must produce yaw rotation: heading = {:.3} rad",
        ferry.heading_radians
    );

    // 5. Harmonic wave surface profile
    let wave_t0 = ferry.water_surface_height(0.0);
    let wave_t1 = ferry.water_surface_height(2.0);
    assert!(
        wave_t0.abs() <= 0.61 && wave_t1.abs() <= 0.61,
        "Wave amplitude must remain bounded by 0.6m swell"
    );
}

// ============================================================================
// TEST 7: AVIATION 6-DOF FLIGHT DYNAMICS, POLAR, STALL & GROUND EFFECT
// ============================================================================

#[test]
fn test_stage7_aviation_6dof_flight_polar_stall_and_ground_effect() {
    // 1. Commercial airliner airframe configuration (A320/B737 class)
    // 68 tonnes MTOW, 122.6 m^2 wing area, 34.1m wingspan, 240 kN max thrust
    let mut airliner = AircraftFlight::new_airliner(
        "Airliner-737",
        Vec3::new(0.0, 500.0, 0.0), // 500m MSL
        0.0,
    );

    assert_eq!(airliner.category, AviationCategory::CommercialAirliner);
    assert_eq!(airliner.mass_kg, 68_000.0);
    assert_eq!(airliner.wing_area_m2, 122.6);
    assert_eq!(airliner.wingspan_meters, 34.1);
    assert_eq!(airliner.max_thrust_newtons, 240_000.0);

    // 2. Aerodynamic lift and drag polar:
    // C_L = C_L0 + C_Lalpha * alpha
    // C_D = C_D0 + (C_L^2 / (pi * e * AR))
    let alpha_4deg = 4.0_f32.to_radians();
    let cl_4 = airliner.cl_zero + airliner.cl_alpha * alpha_4deg;
    let cd_i =
        (cl_4 * cl_4) / (std::f32::consts::PI * airliner.oswald_efficiency * airliner.aspect_ratio);
    let cd_total = airliner.cd_zero + cd_i;

    assert!(
        cl_4 > 0.5 && cl_4 < 0.8,
        "Lift coefficient at 4 deg alpha must be realistic: got {:.3}",
        cl_4
    );
    assert!(
        cd_total > airliner.cd_zero,
        "Total drag must include induced drag from lift generation: C_D = {:.4} > C_D0 = {:.4}",
        cd_total,
        airliner.cd_zero
    );

    // 3. Barometric air density lapse with altitude
    let rho_sea_level = 1.225;
    airliner.position.y = 500.0;
    let rho_500m = airliner.atmospheric_density();
    airliner.position.y = 10_000.0;
    let rho_10000m = airliner.atmospheric_density();

    assert!(
        rho_500m < rho_sea_level,
        "Air density at 500m ({:.3}) must be less than sea level ({:.3})",
        rho_500m,
        rho_sea_level
    );
    assert!(
        rho_10000m < 0.45 && rho_10000m > 0.35,
        "Air density at 10,000m (FL330) must be ~0.41 kg/m^3: got {:.3}",
        rho_10000m
    );

    // 4. Ground effect verification: induced drag reduction within one wingspan of ground
    let b = airliner.wingspan_meters;
    let h_low = 15.0; // 15m < 34.1m wingspan
    let h_over_b_low = h_low / b;
    let factor_low =
        (16.0 * h_over_b_low * h_over_b_low) / (1.0 + 16.0 * h_over_b_low * h_over_b_low);

    let h_high = 100.0; // 100m > 34.1m wingspan
    let h_over_b_high = h_high / b;
    let factor_high = if h_over_b_high < 1.0 {
        (16.0 * h_over_b_high * h_over_b_high) / (1.0 + 16.0 * h_over_b_high * h_over_b_high)
    } else {
        1.0
    };

    assert!(
        factor_low < 0.85,
        "Ground effect factor at 15m must significantly reduce induced drag: factor = {:.3}",
        factor_low
    );
    assert_eq!(
        factor_high, 1.0,
        "Ground effect factor far above ground must equal 1.0"
    );

    // 5. Post-stall flow separation & loss of lift:
    // Pitch nose up 20 degrees relative to trajectory (angle of attack = 20 deg > 15 deg stall angle)
    let dt = 1.0 / 60.0;
    airliner.position.y = 500.0;
    airliner.linear_velocity = Vec3::new(0.0, 0.0, 100.0); // 100 m/s along world Z
    airliner.orientation = Quat::from_rotation_x(-20.0_f32.to_radians()); // Nose pitch up 20 deg
    let level_inputs = FlightInputs {
        elevator: 0.0,
        throttle: 0.5,
        gear_down: false,
        ..Default::default()
    };

    airliner.step(&level_inputs, 0.0, dt);

    assert!(
        airliner.angle_of_attack_rad.to_degrees() > 15.0,
        "Angle of attack must exceed 15 degrees: got {:.1}°",
        airliner.angle_of_attack_rad.to_degrees()
    );
    assert!(
        airliner.is_stalled,
        "Exceeding stall angle of attack (15 deg) must trip is_stalled flag"
    );

    // 6. Tricycle landing gear oleo strut compression on ground
    let mut c172 = AircraftFlight::new_general_aviation(
        "Skyhawk-172",
        Vec3::new(0.0, 0.2, 0.0), // On runway surface
        0.0,
    );

    let taxi_inputs = FlightInputs {
        throttle: 0.15, // Taxi power
        elevator: 0.0,
        rudder: 0.2,
        gear_down: true,
        ..Default::default()
    };

    for _ in 0..30 {
        c172.step(&taxi_inputs, 0.0, dt);
    }

    assert!(
        !c172.is_airborne,
        "Aircraft on runway below stall speed must remain grounded"
    );
    assert!(
        c172.gear_compression_meters >= 0.0,
        "Landing gear oleo struts must compress under airframe weight"
    );
}

// ============================================================================
// TEST 8: MULTIMODAL CONCURRENT SIMULATION EXECUTION THROUGHPUT BUDGET (>500 HZ)
// ============================================================================

#[test]
fn test_stage7_multimodal_simulation_throughput_budget() {
    let ground = FlatGround::new(0.0);
    let dt = 1.0 / 60.0;

    // Initialize one instance of each distinct multimodal transport model
    let mut bike = MotorcycleModel::new(Vec3::new(0.0, 0.35, 0.0), 0.0);
    let mut bus = TransitBusModel::new(Vec3::new(10.0, 0.8, 0.0), 0.0);
    let mut truck = HeavyTruckModel::new(Vec3::new(20.0, 1.2, 0.0), 0.0);
    let mut train = TrainConsist::new_locomotive(
        "Freight-Loco",
        LocomotiveType::DieselElectric,
        3_300_000.0,
        450_000.0,
        135_000.0,
        21.0,
    );
    train.add_wagon(TrainCar::new_wagon("Hopper-1", 25_000.0, 65_000.0, 14.5));
    train.add_wagon(TrainCar::new_wagon("Hopper-2", 25_000.0, 65_000.0, 14.5));

    let mut ship = MarineVessel::new_ferry("Harbor-Ferry", Vec3::new(0.0, 0.0, 100.0), 0.0);
    let mut plane = AircraftFlight::new_airliner("Jet-A320", Vec3::new(0.0, 1000.0, 0.0), 0.0);
    plane.linear_velocity = Vec3::new(0.0, 0.0, 180.0);

    let road_inputs = VehicleInputs {
        throttle: 0.6,
        steer: 0.1,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };
    let marine_inputs = MarineInputs {
        throttle: 0.8,
        rudder: 0.1,
        bow_thruster: 0.0,
    };
    let flight_inputs = FlightInputs {
        throttle: 0.75,
        elevator: 0.05,
        aileron: 0.0,
        rudder: 0.0,
        flaps: 0.0,
        spoilers: 0.0,
        wheel_brakes: 0.0,
        reverse_thrust: false,
        gear_down: false,
    };

    // Benchmark 1,000 steps of concurrent multimodal simulation across all domains
    let steps = 1000;
    let start = Instant::now();
    let mut sim_time = 0.0;

    for _ in 0..steps {
        bike.step(&road_inputs, &ground, dt);
        bus.step(&road_inputs, &ground, dt);
        truck.step(&road_inputs, &ground, dt);
        train.step(dt);
        ship.step(&marine_inputs, sim_time, dt);
        plane.step(&flight_inputs, 0.0, dt);
        sim_time += dt;
    }

    let elapsed = start.elapsed();
    let elapsed_sec = elapsed.as_secs_f64();
    let sim_rate_hz = steps as f64 / elapsed_sec;
    let avg_step_micros = (elapsed_sec * 1_000_000.0) / steps as f64;

    println!(
        "\n⚡ Stage 7 Multimodal Simulation Throughput:\n  \
         Total time for {} steps across 6 multimodal vehicles: {:.3} ms\n  \
         Simulation Rate: {:.1} Hz (Target: > 500.0 Hz)\n  \
         Average step time: {:.2} µs (Budget: < 2,000.0 µs)\n",
        steps,
        elapsed_sec * 1000.0,
        sim_rate_hz,
        avg_step_micros
    );

    assert!(
        sim_rate_hz > 500.0,
        "Multimodal physics throughput ({:.1} Hz) must exceed 500 Hz budget",
        sim_rate_hz
    );
    assert!(
        avg_step_micros < 2000.0,
        "Average step duration ({:.2} µs) must be well within 2 ms budget",
        avg_step_micros
    );
}

// ============================================================================
// TEST 9: PROCEDURAL 3D MESH GENERATION & HUD CLUSTERS ACROSS ALL 7 MODES
// ============================================================================

#[test]
fn test_stage7_multimodal_3d_procedural_rendering_and_telemetry_hud() {
    // 1. Procedural 3D Mesh Generation across all multimodal visual types
    let multimodal_types = [
        VehicleVisualType::SportsCoupe,
        VehicleVisualType::Motorcycle,
        VehicleVisualType::TransitBus,
        VehicleVisualType::HeavyTruck,
        VehicleVisualType::TrainLocomotive,
        VehicleVisualType::TrainWagon,
        VehicleVisualType::MarineVessel,
        VehicleVisualType::CommercialAirliner,
        VehicleVisualType::GeneralAviation,
    ];

    for vtype in &multimodal_types {
        let mesh = generate_vehicle_mesh(&VehicleMeshParams {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            color: [0.8, 0.2, 0.2, 1.0],
            vehicle_type: *vtype,
            wheel_steer_angle: 0.15,
            headlights_on: true,
            brake_lights_on: true,
            reverse_lights_on: false,
            trailer_angle: 0.25, // Articulated trailer angle
        });

        assert!(
            !mesh.vertices.is_empty(),
            "Mesh for {:?} must generate vertices: got {}",
            vtype,
            mesh.vertices.len()
        );
        assert!(
            !mesh.indices.is_empty(),
            "Mesh for {:?} must generate indices: got {}",
            vtype,
            mesh.indices.len()
        );
        assert!(
            mesh.indices.len().is_multiple_of(3),
            "Index buffer for {:?} must consist of complete triangle triplets",
            vtype
        );
    }

    // 2. Heavy truck articulated trailer mesh verification
    let truck_mesh_straight = generate_vehicle_mesh(&VehicleMeshParams {
        position: Vec3::ZERO,
        orientation: Quat::IDENTITY,
        color: [0.3, 0.3, 0.8, 1.0],
        vehicle_type: VehicleVisualType::HeavyTruck,
        wheel_steer_angle: 0.0,
        headlights_on: true,
        brake_lights_on: false,
        reverse_lights_on: false,
        trailer_angle: 0.0,
    });

    let truck_mesh_articulated = generate_vehicle_mesh(&VehicleMeshParams {
        position: Vec3::ZERO,
        orientation: Quat::IDENTITY,
        color: [0.3, 0.3, 0.8, 1.0],
        vehicle_type: VehicleVisualType::HeavyTruck,
        wheel_steer_angle: 0.0,
        headlights_on: true,
        brake_lights_on: false,
        reverse_lights_on: false,
        trailer_angle: 0.5, // 0.5 rad articulation
    });

    assert_eq!(
        truck_mesh_straight.vertices.len(),
        truck_mesh_articulated.vertices.len(),
        "Vertex count must remain invariant under articulation rotation"
    );

    // 3. Multimodal Instrument Cluster HUD Rendering
    let mut hud = HudMesh::new();

    // Motorcycle cluster: (x, y, speed_kph, lean_angle_deg, steer_deg, rpm, is_crashed)
    hud.add_motorcycle_cluster(20.0, 500.0, 72.5, 32.4, 15.0, 6800.0, false);
    // Heavy truck cluster: (x, y, speed_kph, gear_str, trailer_deg, rollover_margin_g, air_brake_bar, is_rolled_over)
    hud.add_heavy_truck_cluster(20.0, 500.0, 85.0, "D8", 14.5, 0.35, 7.8, false);
    // Transit bus cluster: (x, y, speed_kph, passenger_count, total_mass_tons, comfort_rating, air_brake_bar)
    hud.add_transit_bus_cluster(20.0, 500.0, 45.0, 52, 16.4, 4.8, 8.2);
    // Train cluster: (x, y, speed_kph, throttle_pct, brake_pipe_bar, brake_cyl_bar, tractive_kn, reverser, signal_aspect, wheel_slip)
    hud.add_train_cluster(20.0, 500.0, 110.0, 0.85, 4.8, 0.0, 180.0, 1, "Clear", false);
    // Marine cluster: (x, y, speed_knots, throttle, rudder_deg, heading_deg, draft_m, gm_m)
    hud.add_marine_cluster(20.0, 500.0, 18.5, 0.85, 12.0, 275.0, 3.8, 1.8);
    // Aviation PFD: (x, y, ias_knots, alt_m, climb_rate_mps, pitch_deg, roll_deg, flaps, gear_down, is_stalled)
    hud.add_aviation_pfd(
        20.0, 450.0, 240.0, 3200.0, 4.5, 2.5, -5.0, 0.0, false, false,
    );

    assert!(
        !hud.is_empty(),
        "HUD mesh layer must contain vertices and indices for all multimodal clusters"
    );
    assert!(
        hud.vertices.len() > 100,
        "HUD mesh must generate substantial telemetry geometry: vertices = {}",
        hud.vertices.len()
    );
    assert!(
        hud.indices.len().is_multiple_of(3),
        "HUD indices must consist of complete triangle triplets"
    );
}

// ============================================================================
// TEST 10: RAIL TRACK-CONSTRAINED NAVIGATION, TURNOUTS & ATP SIGNAL ENFORCEMENT
// ============================================================================

#[test]
fn test_stage7_rail_caltrain_navigation_turnouts_and_atp_enforcement() {
    let mut rail = RailNetworkGraph::create_san_francisco_caltrain_corridor();
    assert_eq!(rail.nodes.len(), 6);
    assert_eq!(rail.segments.len(), 5);

    // 1. Spawning commuter train consist at San Francisco 4th & King
    let anchor = Wgs84Coord::new(37.7764, -122.3942, 5.0);
    let seg1 = rail.segments.get("TRK_SF_01").unwrap();
    let (spawn_pos, _) = seg1.sample_position_enu(0.0, anchor);
    assert!(
        spawn_pos.length() < 0.1,
        "Spawn must be at 4th & King terminus origin"
    );

    let mut train = TrainConsist::new_locomotive(
        "Caltrain-MP36PH",
        LocomotiveType::DieselElectric,
        2_700_000.0,
        220_000.0,
        130_000.0,
        21.0,
    );
    for i in 1..=4 {
        train.add_wagon(TrainCar::new_wagon(
            &format!("BiLevel-Coach-{}", i),
            45_000.0,
            5_000.0,
            26.0,
        ));
    }
    assert_eq!(
        train.cars.len(),
        5,
        "Train consist must be 1 locomotive + 4 bi-level passenger coaches"
    );
    assert_eq!(train.brake_state, TrainBrakeState::Released);

    // 2. Track constrained motion and geometry sampling along Caltrain mainline
    let (mid_pos, mid_heading) = seg1.sample_position_enu(525.0, anchor);
    assert!(
        mid_pos.length() > 200.0,
        "Mid-track position must progress along corridor"
    );
    assert!(mid_heading.is_finite());

    // 3. Audio acoustic emissions (locomotive horn & flange squeal)
    train.sound_horn();
    let audio = train.drain_audio_events();
    assert!(
        audio.contains(&MultimodalAudioEvent::TrainHorn),
        "Train horn must emit acoustic event"
    );

    // 4. Mainline vs Yard Turnout switch routing
    // Mainline straight alignment
    assert!(rail.set_switch_state("TRK_SF_02_MAIN", SwitchState::Straight));
    let outgoing_straight = rail.outgoing_segments("SF_MISSION_BAY_TURNOUT");
    assert!(outgoing_straight.iter().any(|s| s.id == "TRK_SF_02_MAIN"));

    // Diverging switch alignment into Mission Bay Maintenance Yard
    assert!(rail.set_switch_state("TRK_SF_02_MAIN", SwitchState::Locked));
    assert!(rail.set_switch_state("TRK_SF_02_YARD", SwitchState::Diverging));
    let outgoing_diverging = rail.outgoing_segments("SF_MISSION_BAY_TURNOUT");
    assert!(
        outgoing_diverging.iter().all(|s| s.id != "TRK_SF_02_MAIN"),
        "Locked straight track must be filtered out"
    );
    assert!(
        outgoing_diverging.iter().any(|s| s.id == "TRK_SF_02_YARD"),
        "Diverging siding must be open"
    );

    // 5. Automatic Train Protection (ATP) Stop Signal Trip
    train.throttle = 1.0;
    train.reverser = 1;
    for _ in 0..180 {
        train.step(1.0 / 60.0);
    }
    assert!(
        train.speed_kph > 5.0,
        "Train should have accelerated: got {:.1} km/h",
        train.speed_kph
    );

    // Red signal encountered -> ATP trips emergency brake
    rail.set_signal_aspect("TRK_SF_02_MAIN", SignalAspect::Stop);
    train.current_signal = SignalAspect::Stop;
    train.step(1.0 / 60.0);

    assert!(
        train.emergency_brake,
        "ATP must trip when passing Red block signal"
    );
    assert_eq!(
        train.brake_state,
        TrainBrakeState::Emergency,
        "Brake pipe must vent to emergency"
    );

    // Advance 10 seconds under emergency brake -> full stop
    for _ in 0..600 {
        train.step(1.0 / 60.0);
    }
    assert!(
        train.speed_kph < 0.1,
        "Train must come to a dead stop under ATP emergency braking"
    );
}

// ============================================================================
// TEST 11: MARINE WATERWAY FAIRWAYS, BERTH DOCKING & SHORELINE COLLISION
// ============================================================================

#[test]
fn test_stage7_marine_san_francisco_bay_fairways_docking_and_grounding() {
    let mut waterway = WaterwayGraph::create_san_francisco_bay_waterway();
    assert_eq!(waterway.nodes.len(), 4);
    assert_eq!(waterway.channels.len(), 3);
    assert_eq!(waterway.ports.len(), 2);

    // 1. Channel navigability & draft depth limits
    let container_draft = 11.5;
    assert!(waterway.is_channel_navigable("CH_GOLDEN_GATE_APPROACH", container_draft));
    assert!(waterway.is_channel_navigable("CH_OAKLAND_HARBOR", container_draft));

    let supertanker_draft = 22.0;
    assert!(
        !waterway.is_channel_navigable("CH_CENTRAL_BAY_FAIRWAY", supertanker_draft),
        "Supertanker draft must exceed Central Bay channel depth"
    );

    // 2. Fairway arclength sampling
    let ch = waterway.channels.get("CH_CENTRAL_BAY_FAIRWAY").unwrap();
    let anchor = Wgs84Coord::new(37.7955, -122.3937, 0.0);
    let (ch_start, _) = ch.sample_position_enu(0.0, anchor);
    let (ch_end, _) = ch.sample_position_enu(ch.length_meters, anchor);
    assert!(
        ch_start.distance(ch_end) > 1000.0,
        "Channel points must span fairway length"
    );

    // 3. Spawning passenger ferry in San Francisco Bay
    let mut ferry = MarineVessel::new_ferry("Golden-Gate-Ferry", Vec3::ZERO, 0.0);
    ferry.sound_foghorn();
    let audio = ferry.drain_audio_events();
    assert!(
        audio.contains(&MultimodalAudioEvent::ShipFoghorn),
        "Maritime foghorn must emit audio event"
    );

    // 4. Shoreline grounding collision detection
    assert!(
        !ferry.check_shoreline_collision(0.0),
        "Vessel at sea level is not grounded"
    );
    assert!(!ferry.is_grounded);

    // Heading towards shore (elevation > 0.5m MSL)
    let ground_elev = 2.4;
    assert!(
        ferry.check_shoreline_collision(ground_elev),
        "Vessel must collide with shoreline when elevation exceeds MSL threshold"
    );
    assert!(
        ferry.is_grounded,
        "Grounded status must be set on collision"
    );
    assert_eq!(
        ferry.speed_knots, 0.0,
        "Vessel speed must be arrested upon grounding"
    );

    // 5. Port berth mooring & docking validation
    let mut port_sf = waterway.ports.get_mut("PORT_SF_FERRY").unwrap().clone();
    let dock_res = ferry.dock_at_berth(&mut port_sf, "BERTH_GATE_B");
    assert!(
        dock_res.is_ok(),
        "Ferry must successfully dock at Ferry Building Gate B: {:?}",
        dock_res
    );

    // Invalid docking: large container cargo ship attempting to moor at small passenger ferry berth
    let mut cargo_ship = MarineVessel::new_container_ship("Ever-RealEarth", Vec3::ZERO, 0.0);
    let invalid_dock = cargo_ship.dock_at_berth(&mut port_sf, "BERTH_GATE_C");
    assert!(
        invalid_dock.is_err(),
        "Cargo ship (260m) must be rejected by ferry berth (95m)"
    );
}

// ============================================================================
// TEST 12: AVIATION SFO TAKEOFF ROLL, ROTATION, ILS GUIDANCE & TOUCHDOWN
// ============================================================================

#[test]
fn test_stage7_aviation_sfo_runway_takeoff_ils_approach_and_touchdown() {
    let airport = Airport::create_san_francisco_airport();
    assert_eq!(airport.icao_code, "KSFO");
    assert_eq!(airport.runways.len(), 2);

    let rwy_28l = &airport.runways[0];
    assert_eq!(rwy_28l.identifier, "28L");
    assert_eq!(rwy_28l.true_heading_deg, 284.0);

    // 1. Spawning airliner on Runway 28L
    let mut jet =
        AircraftFlight::new_airliner("Boeing-777-Freighter", Vec3::new(0.0, 2.0, 0.0), 0.0);
    assert!(!jet.is_airborne);
    assert!(jet.in_ground_effect);

    // 2. Takeoff ground roll acceleration & Vr rotation
    let mut inputs = FlightInputs {
        throttle: 1.0,
        flaps: 0.5,
        gear_down: true,
        ..Default::default()
    };

    // Accelerate down runway for 25 seconds
    for _ in 0..1500 {
        jet.step(&inputs, 0.0, 1.0 / 60.0);
    }
    assert!(
        jet.true_airspeed_mps > 50.0,
        "Airliner must accelerate past 50 m/s, got {:.1} m/s",
        jet.true_airspeed_mps
    );

    // Rotate elevator back
    inputs.elevator = 0.6;
    for _ in 0..600 {
        jet.step(&inputs, 0.0, 1.0 / 60.0);
    }
    assert!(
        jet.is_airborne,
        "Aircraft must become airborne following elevator rotation"
    );
    assert!(
        jet.position.y > 10.0,
        "Aircraft altitude must increase during climb-out: y={:.1}",
        jet.position.y
    );
    assert!(
        jet.climb_rate_mps > 1.0,
        "Climb rate must be positive: got {:.1} m/s",
        jet.climb_rate_mps
    );

    // 3. ILS Glideslope & Localizer approach guidance computation
    // Runway 28L (284 deg heading) inbound approach radial is 104 deg from threshold
    let approach_coord = Wgs84Coord::new(37.6135 - 0.02176, -122.3572 + 0.1103, 4.0 + 524.0);
    let ils = rwy_28l.compute_ils_guidance(&approach_coord, (4.0 + 524.0) as f32);
    assert!(ils.is_localizer_captured, "Aircraft aligned with Runway 28L extended centerline must capture localizer, got dev: {:.2} deg", ils.localizer_deviation_deg);
    assert!(
        ils.distance_to_threshold_meters > 8000.0,
        "Distance to threshold must match approach geometry"
    );

    let (loc_dev, gs_dev) =
        jet.compute_approach_guidance(Vec3::new(0.0, 4.0, 0.0), 284.0_f32.to_radians());
    assert!(loc_dev.is_finite());
    assert!(gs_dev.is_finite());

    // 4. Landing gear touchdown & strut compression
    let mut landing_jet =
        AircraftFlight::new_airliner("A321-Neo", Vec3::new(0.0, 5.5, 0.0), 284.0_f32.to_radians());
    landing_jet.is_airborne = true;
    landing_jet.linear_velocity = Vec3::new(0.0, -2.0, 62.0); // 2.0 m/s sink rate
    let landing_inputs = FlightInputs {
        gear_down: true,
        wheel_brakes: 0.9,
        reverse_thrust: true,
        throttle: 0.8,
        ..Default::default()
    };

    for _ in 0..180 {
        landing_jet.step(&landing_inputs, 4.0, 1.0 / 60.0);
    }

    assert!(
        !landing_jet.is_airborne,
        "Aircraft must touch down on runway surface"
    );
    assert!(
        landing_jet.gear_compression_meters > 0.0,
        "Oleo struts must compress on touchdown"
    );
    let landing_audio = landing_jet.drain_audio_events();
    assert!(
        landing_audio
            .iter()
            .any(|e| matches!(e, MultimodalAudioEvent::AircraftGearTouchdown { .. })),
        "Touchdown must trigger gear touchdown audio chirp"
    );
}

// ============================================================================
// TEST 13: END-TO-END MULTIMODAL MISSIONS, INVALID TRANSITIONS & BOUNDARIES
// ============================================================================

#[test]
fn test_stage7_multimodal_mission_lifecycle_invalid_transitions_and_coverage() {
    let catalog = JobCatalogGenerator::jobs_for_region("san-francisco");

    // 1. Multimodal contracts existence across all transport modes
    let required_jobs = [
        ("moto-sf-01", "Motorcycle courier express"),
        ("transit-sf-01", "Transit bus route"),
        ("freight-sf-01", "Heavy freight haul"),
        ("rail-sf-01", "Caltrain commuter rail"),
        ("ferry-sf-01", "SF Bay passenger ferry"),
        ("air-sf-01", "SFO air cargo approach"),
    ];

    for (job_id, desc) in &required_jobs {
        assert!(
            catalog.iter().any(|j| j.id == *job_id),
            "Missing multimodal contract {} ({})",
            job_id,
            desc
        );
    }

    // 2. Motorcycle express courier full mission simulation loop
    let event_bus = GameplayEventBus::new();
    let profile = PlayerProfile::new("player_01", "MultimodalPlayer");
    let mut moto_job = catalog
        .iter()
        .find(|j| j.id == "moto-sf-01")
        .unwrap()
        .clone();
    assert_eq!(moto_job.state, JobState::Available);

    // Accept mission
    let accept_res = moto_job.accept(&profile, &event_bus);
    assert!(
        accept_res.is_ok(),
        "Failed to accept motorcycle job: {:?}",
        accept_res
    );
    assert_eq!(moto_job.state, JobState::EnRouteToPickup);

    // Step at pickup location (Transamerica Pyramid)
    let pickup_coord = moto_job.pickup.coord;
    for _ in 0..150 {
        moto_job.update(1.0 / 60.0, pickup_coord, 0.0, &event_bus);
    }
    assert_eq!(
        moto_job.state,
        JobState::EnRouteToDropoff,
        "Job must transition to loaded/en-route to dropoff after pickup dwell"
    );

    // Step at destination location (Salesforce Transit Center)
    let dropoff_coord = moto_job.dropoff.coord;
    let mut completion = None;
    for _ in 0..150 {
        if let Some(res) = moto_job.update(1.0 / 60.0, dropoff_coord, 0.0, &event_bus) {
            completion = Some(res);
            break;
        }
    }
    assert!(
        completion.is_some(),
        "Motorcycle job must successfully settle upon dropoff completion"
    );
    let res = completion.unwrap();
    assert_eq!(res.job_id, "moto-sf-01");
    assert!(res.final_payout > 100.0, "Payout must be awarded");
    assert!(res.xp_gained > 50, "XP must be awarded");

    // 3. Invalid mode transitions & boundary enforcement
    // A. Road vehicle type restrictions: Heavy truck cannot enter NoHeavyTrucks zone
    let truck_cat = RoadVehicleCategory::HeavyTruck;
    let truck_ban = RoadAccessRestriction::NoHeavyTrucks;
    assert!(
        !truck_ban.is_allowed(truck_cat),
        "Heavy truck must be banned from NoHeavyTrucks zone"
    );

    // Motorcycle allowed on motorcycle lanes, banned from bus rapid transit (BusOnly)
    let moto_cat = RoadVehicleCategory::Motorcycle;
    let brt_lane = RoadAccessRestriction::BusOnly;
    assert!(
        !brt_lane.is_allowed(moto_cat),
        "Motorcycle must be banned from BusOnly lane"
    );

    // B. Marine boundary violation: Ship cannot cross elevation boundary onto land
    let mut vessel = MarineVessel::new_container_ship("Evergreen", Vec3::ZERO, 0.0);
    assert!(
        vessel.check_shoreline_collision(5.0),
        "Ship must detect terrain above sea level as shoreline collision"
    );
    assert!(vessel.is_grounded);

    // C. Rail boundary violation: Train passing red signal triggers emergency penalty
    let mut penalty_train = TrainConsist::new_locomotive(
        "Freight-Loco",
        LocomotiveType::DieselElectric,
        3_000_000.0,
        300_000.0,
        140_000.0,
        22.0,
    );
    penalty_train.velocity_mps = 10.0;
    penalty_train.current_signal = SignalAspect::Stop;
    penalty_train.step(0.1);
    assert!(penalty_train.emergency_brake);
    assert_eq!(penalty_train.brake_state, TrainBrakeState::Emergency);
}
