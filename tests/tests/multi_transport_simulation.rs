//! Multi-transport end-to-end integration and certification tests for Stage 9.
//!
//! Validates:
//! 1. Phase 9.1 — Road Transport (Motorcycles, Transit Buses, Heavy Articulated Trucks, Access Policies)
//! 2. Phase 9.2 — Railway & Marine (Track Graphs, Turnouts, Block Signaling, Level Crossings, Train Consist, Waterway Graphs, Archimedes Buoyancy, Metacentric Stability, Rudder Dynamics)
//! 3. Phase 9.3 — Aviation (Airport Runways, ILS Glideslope, 6-DOF Aerodynamic Polar, Ground Effect, Oleo Struts)
//! 4. Multi-Modal Concurrent Execution & Physics Independence Invariants.

use glam::Vec3;
use realearth_navigation::air_navigation::{AirNavigationDatabase, Airport, AirportRunway};
use realearth_navigation::rail_graph::{
    CrossingState, RailGauge, RailLevelCrossing, RailNetworkGraph, RailNode, RailTrackSegment,
    SignalAspect, SwitchState,
};
use realearth_navigation::waterway::{
    MarinePort, NavigationAids, PortBerth, ShippingChannel, WaterwayCategory, WaterwayGraph,
    WaterwayNode,
};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_vehicles::dynamics::VehicleInputs;
use realearth_vehicles::ground::FlatGround;
use realearth_vehicles::multimodal::aviation::{AircraftFlight, AviationCategory, FlightInputs};
use realearth_vehicles::multimodal::marine::{MarineInputs, MarineVessel, MarineVesselType};
use realearth_vehicles::multimodal::road_transport::{
    HeavyTruckModel, MotorcycleModel, RoadAccessRestriction, RoadVehicleCategory, TransitBusModel,
};
use realearth_vehicles::multimodal::train::{LocomotiveType, TrainCar, TrainConsist};
use std::time::Instant;

// ============================================================================
// PHASE 9.1: ROAD TRANSPORT (Motorcycles, Buses, Trucks)
// ============================================================================

#[test]
fn test_stage9_1_road_multimodal_dynamics_and_access_restrictions() {
    let ground = FlatGround::new(0.0);

    // 1. Road Access Policies
    let brt_lane = RoadAccessRestriction::BusOnly;
    assert!(brt_lane.is_allowed(RoadVehicleCategory::TransitBus));
    assert!(!brt_lane.is_allowed(RoadVehicleCategory::CitySedan));
    assert!(!brt_lane.is_allowed(RoadVehicleCategory::HeavyTruck));
    assert!(!brt_lane.is_allowed(RoadVehicleCategory::Motorcycle));

    let heritage_sector = RoadAccessRestriction::NoHeavyTrucks;
    assert!(heritage_sector.is_allowed(RoadVehicleCategory::Motorcycle));
    assert!(heritage_sector.is_allowed(RoadVehicleCategory::TransitBus));
    assert!(!heritage_sector.is_allowed(RoadVehicleCategory::HeavyTruck));

    // 2. Motorcycle 2-Wheel Lean Dynamics
    let mut bike = MotorcycleModel::new(Vec3::ZERO, 0.0);
    assert_eq!(bike.mass_kg, 220.0);
    assert_eq!(bike.lean_angle, 0.0);

    // Accelerate into right turn
    let bike_inputs = VehicleInputs {
        throttle: 0.85,
        steer: 0.6,
        ..Default::default()
    };
    for _ in 0..180 {
        bike.step(&bike_inputs, &ground, 1.0 / 60.0);
    }

    assert!(
        bike.speed_kph > 15.0,
        "Motorcycle should accelerate, speed: {} kph",
        bike.speed_kph
    );
    assert!(
        bike.lean_angle > 0.05,
        "Motorcycle must lean into the corner: lean={:.3} rad",
        bike.lean_angle
    );
    assert!(!bike.is_crashed);

    // 3. Transit Bus Payload, Swept Path & Pneumatic Air Suspension
    let mut bus = TransitBusModel::new(Vec3::ZERO, 0.0);
    bus.set_passenger_count(50);
    assert_eq!(bus.total_mass_kg, 12500.0 + (50.0 * 75.0));

    let tail_pos = bus.rear_tail_position();
    let tail_dist = (tail_pos - bus.position).length();
    assert!(
        tail_dist >= 3.0,
        "Rear tail overhang must extend >= 3.0m behind center: dist={:.2}m",
        tail_dist
    );

    let bus_inputs = VehicleInputs {
        throttle: 0.8,
        ..Default::default()
    };
    for _ in 0..240 {
        bus.step(&bus_inputs, &ground, 1.0 / 60.0);
    }
    assert!(
        bus.speed_kph > 12.0,
        "Transit bus should reach line speed: {} kph",
        bus.speed_kph
    );

    // 4. Heavy Articulated Truck Rollover Stability Threshold
    let mut truck = HeavyTruckModel::new(Vec3::ZERO, 0.0);
    assert!(truck.gross_mass_kg() >= 30_000.0);
    assert!(truck.rollover_threshold_accel > 3.0);

    let truck_inputs = VehicleInputs {
        throttle: 0.9,
        ..Default::default()
    };
    for _ in 0..360 {
        truck.step(&truck_inputs, &ground, 1.0 / 60.0);
    }
    assert!(truck.speed_kph > 18.0);
    assert!(!truck.is_rolled_over);
}

// ============================================================================
// PHASE 9.2: RAILWAY & MARINE
// ============================================================================

#[test]
fn test_stage9_2_railway_track_network_and_train_consist() {
    // 1. Rail Network Graph with Turnouts, Signals, and Crossings
    let mut rail_net = RailNetworkGraph::new();

    let n_sta = RailNode {
        id: "STA_SBC".to_string(),
        coord: Wgs84Coord::new(12.9780, 77.5680, 920.0),
        is_station: true,
        station_name: Some("KSR Bengaluru City".to_string()),
    };
    let n_junc = RailNode {
        id: "JUNC_BYPL".to_string(),
        coord: Wgs84Coord::new(12.9900, 77.6500, 910.0),
        is_station: false,
        station_name: None,
    };
    let n_cpt = RailNode {
        id: "STA_WFD".to_string(),
        coord: Wgs84Coord::new(12.9950, 77.7500, 890.0),
        is_station: true,
        station_name: Some("Whitefield Satellite".to_string()),
    };

    rail_net.add_node(n_sta);
    rail_net.add_node(n_junc);
    rail_net.add_node(n_cpt);

    let seg1 = RailTrackSegment {
        id: "TRK_SBC_BYPL".to_string(),
        from_node: "STA_SBC".to_string(),
        to_node: "JUNC_BYPL".to_string(),
        length_meters: 8900.0,
        speed_limit_kph: 110.0,
        gauge: RailGauge::Broad,
        is_electrified: true,
        switch_state: Some(SwitchState::Straight),
        signal_aspect: SignalAspect::Clear,
        geometry: Vec::new(),
    };
    rail_net.add_segment(seg1);

    let crossing = RailLevelCrossing {
        id: "LC_77".to_string(),
        road_name: "Old Madras Road Crossing".to_string(),
        coord: Wgs84Coord::new(12.9850, 77.6200, 915.0),
        state: CrossingState::Open,
        timer: 0.0,
    };
    rail_net.add_crossing(crossing);

    // Trigger crossing approach closure
    rail_net.trigger_crossing_approach("LC_77");
    assert_eq!(
        rail_net.crossings.get("LC_77").unwrap().state,
        CrossingState::Closed
    );

    // 2. Train Consist Physics: Tractive Effort & Westinghouse Air Brake
    let mut train = TrainConsist::new_locomotive(
        "WAP-7",
        LocomotiveType::Electric,
        4_500_000.0,
        450_000.0,
        123_000.0,
        20.5,
    );

    // Add 12 high-capacity passenger coaches
    for i in 1..=12 {
        train.add_wagon(TrainCar::new_wagon(
            &format!("LHB-{}", i),
            41_500.0,
            4_500.0,
            24.0,
        ));
    }

    assert_eq!(train.cars.len(), 13);
    assert_eq!(train.total_mass_kg(), 123_000.0 + (12.0 * 46_000.0));
    assert!(train.total_length_meters() > 300.0);

    // Accelerate under notch 1.0 (15 seconds)
    train.throttle = 1.0;
    for _ in 0..900 {
        train.step(1.0 / 60.0);
    }
    assert!(
        train.speed_kph > 20.0,
        "Train should accelerate under 4.5 MW electric traction: {} kph",
        train.speed_kph
    );
    assert!(train.position_s > 30.0);

    // Apply emergency brake
    train.throttle = 0.0;
    train.trigger_emergency_brake();
    for _ in 0..1500 {
        train.step(1.0 / 60.0);
    }
    assert_eq!(train.brake_pipe_pressure_bar, 0.0);
    assert_eq!(
        train.velocity_mps, 0.0,
        "Train must stop completely under full Westinghouse emergency venting"
    );
}

#[test]
fn test_stage9_2_marine_hydrodynamics_and_fairway_navigation() {
    // 1. Waterway Graph (Chennai Harbor Fairway)
    let mut waterway = WaterwayGraph::new();

    waterway.add_node(WaterwayNode {
        id: "N_PILOT".to_string(),
        coord: Wgs84Coord::new(13.1000, 80.3500, 0.0),
        water_depth_meters: 24.0,
        aid_marker: Some(NavigationAids::SafeWater),
    });
    waterway.add_node(WaterwayNode {
        id: "N_BERTH".to_string(),
        coord: Wgs84Coord::new(13.0850, 80.3000, 0.0),
        water_depth_meters: 16.5,
        aid_marker: Some(NavigationAids::PortLateral),
    });

    waterway.add_channel(ShippingChannel {
        id: "CH_MAIN_FAIRWAY".to_string(),
        from_node: "N_PILOT".to_string(),
        to_node: "N_BERTH".to_string(),
        channel_width_meters: 250.0,
        minimum_depth_meters: 16.5,
        speed_limit_knots: 10.0,
        category: WaterwayCategory::DredgedChannel,
        length_meters: 4200.0,
        geometry: Vec::new(),
    });

    waterway.add_port(MarinePort {
        id: "PORT_MADRAS".to_string(),
        name: "Chennai Seaport".to_string(),
        basin_coord: Wgs84Coord::new(13.0850, 80.3000, 0.0),
        max_draft_meters: 15.0,
        berth_count: 1,
        berths: vec![PortBerth {
            berth_id: "BERTH_1".to_string(),
            length_meters: 300.0,
            depth_meters: 16.0,
            is_occupied: false,
        }],
    });

    // Verify channel navigability against draft
    assert!(waterway.is_channel_navigable("CH_MAIN_FAIRWAY", 12.0));
    assert!(!waterway.is_channel_navigable("CH_MAIN_FAIRWAY", 18.0));

    // 2. Hydrodynamic Vessel Physics
    let mut ship = MarineVessel::new_ferry("Kavveri-Express", Vec3::ZERO, 0.0);
    assert_eq!(ship.vessel_type, MarineVesselType::PassengerFerry);
    assert!(ship.metacentric_height_gm > 1.0);

    // Heave settling to equilibrium draft
    let neutral_inputs = MarineInputs::default();
    for tick in 0..600 {
        ship.step(&neutral_inputs, (tick as f32) / 60.0, 1.0 / 60.0);
    }
    assert!(
        ship.position.y.abs() < 2.0,
        "Ship must float stably at water surface datum: y={}",
        ship.position.y
    );

    // Propulsion and rudder steering
    let steer_inputs = MarineInputs {
        throttle: 1.0,
        rudder: 0.7,
        ..Default::default()
    };
    for tick in 0..1200 {
        ship.step(&steer_inputs, (tick as f32) / 60.0, 1.0 / 60.0);
    }
    assert!(ship.speed_knots > 4.0);
    assert!(ship.heading_radians.abs() > 0.02);
}

// ============================================================================
// PHASE 9.3: AVIATION (Airports, ILS Guidance, 6-DOF Flight Dynamics)
// ============================================================================

#[test]
fn test_stage9_3_aviation_airports_and_aerodynamic_flight() {
    // 1. Airport Runway & ILS Guidance
    let mut air_db = AirNavigationDatabase::new();
    let rwy_vobl = AirportRunway {
        identifier: "09L".to_string(),
        threshold: Wgs84Coord::new(13.2000, 77.7000, 915.0),
        reciprocal_threshold: Wgs84Coord::new(13.2000, 77.7400, 915.0),
        true_heading_deg: 90.0,
        length_meters: 4000.0,
        width_meters: 45.0,
        glideslope_angle_deg: 3.0,
        ils_frequency_mhz: Some(109.5),
    };

    let airport = Airport {
        icao_code: "VOBL".to_string(),
        name: "Kempegowda International Airport".to_string(),
        reference_coord: Wgs84Coord::new(13.2000, 77.7000, 915.0),
        elevation_msl_meters: 915.0,
        runways: vec![rwy_vobl.clone()],
    };
    air_db.add_airport(airport);

    // Test ILS guidance along 15 km final approach
    let aircraft_coord = Wgs84Coord::new(13.2000, 77.7000 - 0.136, 915.0 + 785.0);
    let ils = rwy_vobl.compute_ils_guidance(&aircraft_coord, (915.0 + 785.0) as f32);
    assert!(ils.localizer_deviation_deg.abs() < 1.0);
    assert!(ils.glideslope_deviation_deg.abs() < 0.6);
    assert!(ils.is_localizer_captured);

    // 2. 6-DOF Aircraft Aerodynamic Flight
    let mut airliner = AircraftFlight::new_airliner("Airbus-A320", Vec3::new(0.0, 2.0, 0.0), 0.0);
    assert_eq!(airliner.category, AviationCategory::CommercialAirliner);
    assert!(airliner.in_ground_effect);

    // Takeoff ground roll with flaps 1+F and full TOGA thrust
    let mut inputs = FlightInputs {
        throttle: 1.0,
        flaps: 0.5,
        gear_down: true,
        ..Default::default()
    };

    for _ in 0..1500 {
        airliner.step(&inputs, 0.0, 1.0 / 60.0);
    }
    assert!(
        airliner.true_airspeed_mps > 50.0,
        "Airliner must accelerate along runway: speed={:.1} m/s",
        airliner.true_airspeed_mps
    );

    // Rotate elevator stick back for liftoff
    inputs.elevator = 0.6;
    for _ in 0..600 {
        airliner.step(&inputs, 0.0, 1.0 / 60.0);
    }

    assert!(airliner.is_airborne, "Aircraft must lift off runway");
    assert!(
        airliner.position.y > 5.0,
        "Aircraft must climb into 3D airspace: alt={:.1}m",
        airliner.position.y
    );
    assert!(airliner.climb_rate_mps > 0.5);

    // Verify independent flight mechanics (not road wheels)
    assert!(
        airliner.wingspan_meters > 30.0,
        "Aviation model uses true wing planform aerodynamics"
    );
}

// ============================================================================
// PERFORMANCE & CONCURRENT MULTI-TRANSPORT INTEGRATION
// ============================================================================

#[test]
fn test_stage9_concurrent_multi_transport_performance_and_independence() {
    let ground = FlatGround::new(0.0);

    // Instantiate concurrent entities across all 4 modes
    let mut motorcycle = MotorcycleModel::new(Vec3::new(0.0, 0.0, 0.0), 0.0);
    let mut bus = TransitBusModel::new(Vec3::new(10.0, 0.0, 0.0), 0.0);
    let mut truck = HeavyTruckModel::new(Vec3::new(20.0, 0.0, 0.0), 0.0);
    let mut train = TrainConsist::new_locomotive(
        "WAP-7",
        LocomotiveType::Electric,
        4_500_000.0,
        450_000.0,
        123_000.0,
        20.5,
    );
    let mut ship = MarineVessel::new_ferry("Coastal-Star", Vec3::new(100.0, 0.0, 0.0), 0.0);
    let mut aircraft = AircraftFlight::new_airliner("Boeing-737", Vec3::new(0.0, 3000.0, 0.0), 0.0);
    airliner_cruise_setup(&mut aircraft);

    let road_inputs = VehicleInputs {
        throttle: 0.5,
        ..Default::default()
    };
    let marine_inputs = MarineInputs {
        throttle: 0.6,
        ..Default::default()
    };
    let flight_inputs = FlightInputs {
        throttle: 0.7,
        ..Default::default()
    };

    train.throttle = 0.6;

    let start_time = Instant::now();
    let steps = 600; // 10.0 seconds of multi-modal simulation

    for tick in 0..steps {
        let dt = 1.0 / 60.0;
        let time = (tick as f32) * dt;

        // Step all independent modes concurrently
        motorcycle.step(&road_inputs, &ground, dt);
        bus.step(&road_inputs, &ground, dt);
        truck.step(&road_inputs, &ground, dt);
        train.step(dt);
        ship.step(&marine_inputs, time, dt);
        aircraft.step(&flight_inputs, 0.0, dt);
    }

    let elapsed = start_time.elapsed();
    let tick_rate = (steps as f64) / elapsed.as_secs_f64();

    // Verify all modalities progressed in state
    assert!(motorcycle.position.length() > 0.0);
    assert!(bus.position.length() > 0.0);
    assert!(truck.tractor_position.length() > 0.0);
    assert!(train.position_s > 0.0);
    assert!(ship.position.length() > 0.0);
    assert!(aircraft.position.z > 0.0);

    // Verify headless tick rate >> 60 Hz real-time budget
    assert!(
        tick_rate > 500.0,
        "Multi-transport simulation must exceed 500 Hz headless throughput, achieved: {:.1} Hz",
        tick_rate
    );
}

fn airliner_cruise_setup(aircraft: &mut AircraftFlight) {
    aircraft.linear_velocity = Vec3::new(0.0, 0.0, 220.0); // 220 m/s (~790 km/h cruise)
    aircraft.true_airspeed_mps = 220.0;
    aircraft.is_airborne = true;
    aircraft.in_ground_effect = false;
}
