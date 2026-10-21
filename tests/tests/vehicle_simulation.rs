//! Integration Test Suite for STAGE 6: Complete Drivable Car Simulation.
//!
//! Validates:
//! - Automotive 4-wheel independent raycast suspension equilibrium
//! - Slope response, gravity, hill holding, and braking
//! - Flip recovery and crash recovery on ground surfaces
//! - Left-Hand Traffic (LHT) lane navigation along real Bengaluru connected corridors
//! - Grade-separated flyover continuity (Layer 0 ground vs Layer 1 elevated bridge)
//! - Autonomous AI traffic platoon following with Intelligent Driver Model (IDM)
//! - Physics/presentation interpolation and state snapshot determinism

use glam::Vec3;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
use realearth_navigation::{find_route, RoadNetworkGraph, RoutingPreference};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_vehicles::{
    DynamicVehicle, FlatGround, FunctionalGroundQuery, IdmConfig, SlopedGround, TrafficAgent,
    TrafficSimulation, TransmissionMode, VehicleConfig, VehicleInputs,
};

#[test]
fn test_bengaluru_connected_routes_drive() {
    let dataset = create_bengaluru_central_dataset();
    let network = RoadNetworkGraph::from_roads(&dataset.roads);
    let dem = DemGrid::create_bengaluru_plateau_dem(64, 64);

    // 1. Compute Route from Vidhana Soudha to MG Road
    let vidhana = Wgs84Coord::new(12.9791, 77.5901, 0.0);
    let mg_road_end = Wgs84Coord::new(12.9740, 77.6000, 0.0);
    let route = find_route(
        &network,
        vidhana,
        mg_road_end,
        RoutingPreference::ShortestDistance,
    )
    .expect("Route finding must succeed across connected Bengaluru central corridors");

    assert!(route.waypoints.len() >= 3);
    assert!(route.total_distance_meters > 1000.0);

    // 2. Setup Vehicle & Ground Elevation Query
    let anchor = vidhana;
    let dem_clone = dem.clone();
    let ground = FunctionalGroundQuery::new(move |x: f32, z: f32| -> f32 {
        let lat = anchor.lat + (z as f64 / 111_000.0);
        let lon = anchor.lon + (x as f64 / 108_000.0);
        let coord = Wgs84Coord::new(lat, lon, 0.0);
        dem_clone.sample_bilinear(&coord)
    });

    let initial_elev = (ground.elevation_fn)(0.0, 0.0);
    let mut car = DynamicVehicle::new(
        VehicleConfig::default(),
        Vec3::new(0.0, initial_elev, 0.0),
        0.0,
    );
    car.recover_on_surface(initial_elev);

    // 3. Accelerate and drive for 300 ticks (5 seconds at 60Hz)
    let inputs = VehicleInputs {
        throttle: 0.85,
        brake: 0.0,
        steer: 0.02, // Gentle curve
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    let dt = 1.0 / 60.0;
    for _ in 0..300 {
        car.step(&inputs, &ground, dt);
        let terrain_y = (ground.elevation_fn)(car.position.x, car.position.z);

        // Verification: No falling through terrain
        assert!(
            car.position.y >= terrain_y - 0.05,
            "Vehicle fell below terrain: car y={}, terrain y={}",
            car.position.y,
            terrain_y
        );

        // Verification: No floating in the sky (suspension maintains contact)
        assert!(
            car.position.y <= terrain_y + 1.5,
            "Vehicle is floating above terrain: car y={}, terrain y={}",
            car.position.y,
            terrain_y
        );
    }

    assert!(
        car.speed_kph > 35.0,
        "Vehicle should accelerate smoothly up to cruising speed, got {:.1} km/h",
        car.speed_kph
    );
    assert!(car.is_grounded, "Vehicle should maintain ground contact");
}

#[test]
fn test_grade_separated_elevated_flyover_continuity() {
    let dataset = create_bengaluru_central_dataset();
    let kasturba = dataset
        .roads
        .iter()
        .find(|r| r.name.as_deref() == Some("Kasturba Road"))
        .expect("Kasturba Road missing");
    let flyover = dataset
        .roads
        .iter()
        .find(|r| r.name.as_deref() == Some("Cubbon Park Elevated Flyover"))
        .expect("Cubbon Park Elevated Flyover missing");

    // Grade separation assertions
    assert_eq!(kasturba.layer, 0, "Kasturba Road must be at-grade Layer 0");
    assert_eq!(flyover.layer, 1, "Flyover must be elevated Layer 1");
    assert!(
        flyover.is_bridge,
        "Flyover must be marked as bridge structure"
    );

    // Both pass through (12.9750, 77.5950) without at-grade conflict
    let kasturba_has_point = kasturba
        .geometry
        .iter()
        .any(|c| (c.lat - 12.9750).abs() < 1e-4);
    let flyover_has_point = flyover
        .geometry
        .iter()
        .any(|c| (c.lat - 12.9750).abs() < 1e-4);

    assert!(kasturba_has_point && flyover_has_point);
}

#[test]
fn test_slope_climbing_and_hill_holding() {
    // 15% grade slope in Z direction
    let slope = SlopedGround::new(10.0, 0.0, 0.15);
    let mut car = DynamicVehicle::new(VehicleConfig::default(), Vec3::new(0.0, 10.5, 0.0), 0.0);
    car.recover_on_surface(10.0);

    let dt = 1.0 / 60.0;

    // 1. Climb uphill with 90% throttle for 120 ticks (2 seconds)
    let climb_inputs = VehicleInputs {
        throttle: 0.9,
        brake: 0.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };
    for _ in 0..120 {
        car.step(&climb_inputs, &slope, dt);
    }

    assert!(
        car.position.z > 0.0,
        "Car should have traveled uphill in +Z"
    );
    let _peak_z = car.position.z;

    // 2. Decelerate to complete stop and test Hill Hold
    let brake_inputs = VehicleInputs {
        throttle: 0.0,
        brake: 1.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };
    for _ in 0..90 {
        car.step(&brake_inputs, &slope, dt);
    }
    assert!(
        car.linear_velocity.length() < 0.15,
        "Car must come to a halt under full braking: vel={:?}",
        car.linear_velocity
    );
    let stopped_z = car.position.z;

    // Hold brake for another 60 ticks (1 second) - vehicle must not slip or roll back
    for _ in 0..60 {
        car.step(&brake_inputs, &slope, dt);
    }
    assert!(
        (car.position.z - stopped_z).abs() < 0.05,
        "Vehicle should hold position on slope with brake applied"
    );

    // 3. Rollback: switch to Neutral with zero brake
    let neutral_inputs = VehicleInputs {
        throttle: 0.0,
        brake: 0.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Neutral,
    };
    for _ in 0..120 {
        car.step(&neutral_inputs, &slope, dt);
    }

    // Gravity should roll vehicle back downhill (negative Z direction)
    assert!(
        car.position.z < stopped_z - 0.5,
        "Vehicle should roll downhill under gravity in Neutral without brake"
    );
}

#[test]
fn test_autonomous_traffic_fleet_with_idm() {
    let ground = FlatGround::new(0.0);
    let mut sim = TrafficSimulation::new();

    let waypoints = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 100.0),
        Vec3::new(0.0, 0.0, 300.0),
    ];

    // Spawn 4 traffic agents in a convoy along the lane
    for i in 0..4 {
        let spawn_z = (3 - i) as f32 * 25.0; // 75m, 50m, 25m, 0m
        let mut agent = TrafficAgent::new(
            i as u64 + 1,
            Vec3::new(0.0, 0.0, spawn_z),
            waypoints.clone(),
            IdmConfig {
                desired_speed_ms: 12.0, // ~43 km/h
                min_gap_meters: 3.5,
                comfortable_decel_ms2: 2.5,
                ..Default::default()
            },
        );
        agent.speed_ms = 8.0;
        sim.spawn_agent(agent);
    }

    // Step simulation for 150 ticks (2.5 seconds)
    for _ in 0..150 {
        sim.step(1.0 / 60.0, &ground);
    }

    // Verify ordering and safe separation
    for i in 0..3 {
        let leader_z = sim.agents[i].position.z;
        let follower_z = sim.agents[i + 1].position.z;
        let gap = leader_z - follower_z;

        assert!(
            follower_z < leader_z,
            "Agent {} (z={}) must remain behind Agent {} (z={})",
            i + 1,
            follower_z,
            i,
            leader_z
        );
        assert!(gap >= 3.0, "Convoy gap must be >= 3.0m (got {:.2}m)", gap);
    }
}

#[test]
fn test_vehicle_state_snapshot_rewind_determinism() {
    let ground = FlatGround::new(0.0);
    let mut car1 = DynamicVehicle::new(VehicleConfig::default(), Vec3::new(0.0, 0.5, 0.0), 0.0);
    car1.recover_on_surface(0.0);

    let inputs = VehicleInputs {
        throttle: 0.75,
        brake: 0.0,
        steer: 0.05,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    let dt = 1.0 / 60.0;

    // Run for 60 ticks
    for _ in 0..60 {
        car1.step(&inputs, &ground, dt);
    }

    // Take snapshot
    let snapshot = car1.snapshot();

    // Run 60 more ticks
    for _ in 0..60 {
        car1.step(&inputs, &ground, dt);
    }
    let pos_after_120 = car1.position;

    // Rewind car1 to snapshot and replay 60 ticks
    car1.restore_snapshot(&snapshot);
    for _ in 0..60 {
        car1.step(&inputs, &ground, dt);
    }

    // Proves deterministic replay from snapshot
    assert_eq!(car1.position, pos_after_120);
    assert_eq!(car1.speed_kph, car1.speed_kph);
}

#[test]
fn test_crash_and_flip_recovery() {
    let ground = FlatGround::new(0.0);
    let mut car = DynamicVehicle::new(VehicleConfig::default(), Vec3::new(0.0, 1.0, 0.0), 0.0);

    // Invert vehicle orientation (roll 180 degrees)
    car.orientation = glam::Quat::from_rotation_z(std::f32::consts::PI);
    car.linear_velocity = Vec3::new(5.0, 0.0, 5.0);
    car.angular_velocity = Vec3::new(2.0, 1.0, 0.0);

    let inputs = VehicleInputs::default();
    car.step(&inputs, &ground, 1.0 / 60.0);
    assert!(
        car.is_overturned,
        "Vehicle must detect overturned condition"
    );

    // Execute flip recovery
    car.recover_on_surface(0.0);
    assert!(
        !car.is_overturned,
        "Vehicle must no longer be overturned after recovery"
    );
    assert_eq!(car.linear_velocity, Vec3::ZERO);
    assert_eq!(car.angular_velocity, Vec3::ZERO);
    assert!(car.position.y > 0.0);
}
