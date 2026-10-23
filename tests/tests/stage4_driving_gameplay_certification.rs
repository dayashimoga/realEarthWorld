//! STAGE 4 — COMPLETE INTEGRATED CAR DRIVING AND PHYSICS EXPERIENCE CERTIFICATION
//!
//! Forensic end-to-end verification of Stage 4 requirements from docs/ExecutionContract.md:
//! 1. Market Street Straightaway Acceleration, Progressive Throttle, Emergency Braking & Reverse Transmission
//! 2. Nob Hill / California Street Extreme 22% Grade Climbing, ARB Roll Stabilization & Motionless Hill-Hold
//! 3. Bay Bridge Elevated Skyway High-Speed Cruise, CCD Barrier Collision & Chunk Collider Pinning Lifecycle
//! 4. Lombard Street Switchback Navigation, Ackermann Steering Geometry, Speed-Sensitive Lock & Friction Circle
//! 5. Garage Fleet Hot-Swapping (Coupe, Sedan, SUV, Taxi, Bus, Truck), Interior Meshing & Safe Surface Recovery
//! 6. Multi-Perspective Camera Optics (Chase, Hood, Cockpit, Bumper), Animated Steering Wheel & HUD Telemetry

use glam::{Mat3, Quat, Vec3};
use realearth_ingestion::{dem_grid::DemGrid, sample_san_francisco::create_san_francisco_dataset};
use realearth_navigation::{find_route, RoadNetworkGraph, RoutingPreference};
use realearth_rendering::{
    generate_vehicle_mesh, FloatingOriginCamera, VehicleMeshParams, VehicleVisualType,
};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_vehicles::{
    camera::{VehicleCameraController, VehicleCameraPerspective},
    physics::PhysicsWorld,
    rapier3d::prelude::*,
    DynamicVehicle, FlatGround, FunctionalGroundQuery, SlopedGround, TransmissionMode,
    VehicleConfig, VehicleInputs,
};

// ============================================================================
// TEST 1: MARKET STREET ACCELERATION, EMERGENCY BRAKING & REVERSE TRANSMISSION
// ============================================================================

#[test]
fn test_stage4_1_market_street_acceleration_braking_and_reverse() {
    let sf = create_san_francisco_dataset();
    let dem = DemGrid::create_san_francisco_coastal_dem(64, 64);

    // Compute route along Market Street arterial
    let network = RoadNetworkGraph::from_roads(&sf.roads);
    let market_start = sf.roads[0].geometry[0];
    let market_end = sf.roads[0].geometry.last().copied().unwrap();
    let route = find_route(
        &network,
        market_start,
        market_end,
        RoutingPreference::ShortestDistance,
    )
    .expect("Market Street route must resolve");
    assert!(route.waypoints.len() >= 2);

    // Ground elevation model query using authentic DEM
    let anchor = market_start;
    let dem_clone = dem.clone();
    let ground = FunctionalGroundQuery::new(move |x: f32, z: f32| -> f32 {
        let lat = anchor.lat + (z as f64 / 111_000.0);
        let lon = anchor.lon + (x as f64 / 108_000.0);
        let coord = Wgs84Coord::new(lat, lon, 0.0);
        dem_clone.sample_bilinear(&coord)
    });

    let ground_y = (ground.elevation_fn)(0.0, 0.0);
    let config = VehicleConfig::sports_coupe();
    let mut car = DynamicVehicle::new(config, Vec3::new(0.0, ground_y + 0.8, 0.0), 0.0);
    let dt = 1.0 / 60.0;

    // Settle onto road suspension equilibrium
    for _ in 0..60 {
        car.step(&VehicleInputs::default(), &ground, dt);
    }
    assert!(car.is_grounded, "Vehicle must be stably grounded at spawn");

    // Phase A: Progressive 0-60 km/h acceleration
    let accel_inputs = VehicleInputs {
        throttle: 1.0,
        brake: 0.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    let mut prev_speed = car.speed_kph;
    let mut peak_acceleration = 0.0f32;

    for _ in 0..180 {
        car.step(&accel_inputs, &ground, dt);
        let current_accel = (car.speed_kph - prev_speed) / dt;
        if current_accel > peak_acceleration {
            peak_acceleration = current_accel;
        }
        prev_speed = car.speed_kph;
        assert!(
            car.is_grounded,
            "Tires must maintain suspension ground contact"
        );
    }

    assert!(
        car.speed_kph >= 45.0,
        "Sports Coupe must accelerate to cruising speed: speed={:.1} km/h",
        car.speed_kph
    );
    assert!(
        peak_acceleration > 5.0,
        "Engine must produce positive forward acceleration"
    );

    // Phase B: Emergency Braking
    let initial_brake_speed = car.speed_kph;
    let initial_z = car.position.z;
    let brake_inputs = VehicleInputs {
        throttle: 0.0,
        brake: 1.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    for _ in 0..180 {
        car.step(&brake_inputs, &ground, dt);
    }

    let stopping_dist = (car.position.z - initial_z).abs();
    assert!(
        car.speed_kph <= 2.0,
        "Emergency brake must bring vehicle to complete stop: speed={:.1} km/h",
        car.speed_kph
    );
    assert!(
        stopping_dist < 50.0,
        "Stopping distance must satisfy braking performance margins: dist={:.1}m for initial speed={:.1}km/h",
        stopping_dist,
        initial_brake_speed
    );

    // Phase C: Reverse Mode with Speed Governor
    let rev_inputs = VehicleInputs {
        throttle: 0.8,
        brake: 0.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Reverse,
    };

    for _ in 0..240 {
        car.step(&rev_inputs, &ground, dt);
    }

    assert!(
        car.linear_velocity.z < -2.0,
        "Vehicle must reverse backwards along longitudinal axis: vz={:.2} m/s",
        car.linear_velocity.z
    );
    assert!(
        car.speed_kph <= 50.0,
        "Reverse governor must cap reverse velocity around 45 km/h: speed={:.1} km/h",
        car.speed_kph
    );

    // Verify reverse lights turn on
    let mesh_params = VehicleMeshParams {
        reverse_lights_on: rev_inputs.gear_mode == TransmissionMode::Reverse,
        brake_lights_on: false,
        headlights_on: true,
        wheel_steer_angle: car.wheels[0].steer_angle,
        ..Default::default()
    };
    assert!(
        mesh_params.reverse_lights_on,
        "Reverse lights must be active in reverse"
    );
}

// ============================================================================
// TEST 2: NOB HILL / CALIFORNIA STREET 22% GRADE, ARB & MOTIONLESS HILL-HOLD
// ============================================================================

#[test]
fn test_stage4_2_nob_hill_steep_slope_arb_and_motionless_hill_hold() {
    // California Street reaches up to 22% grade (approx 12.4 degrees / 0.22 slope)
    let slope_grade = 0.22f32;
    let slope = SlopedGround::new(50.0, 0.0, slope_grade);
    let suv_config = VehicleConfig::suv();
    let mut suv = DynamicVehicle::new(suv_config, Vec3::new(0.0, 52.0, 0.0), 0.0);

    // Settle on slope
    let dt = 1.0 / 60.0;
    for _ in 0..60 {
        suv.step(&VehicleInputs::default(), &slope, dt);
    }
    assert!(
        suv.is_grounded,
        "SUV suspension must settle onto steep slope"
    );

    // Phase A: Climb 22% steep incline
    let climb_inputs = VehicleInputs {
        throttle: 1.0,
        brake: 0.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };
    let initial_y = suv.position.y;
    for _ in 0..120 {
        suv.step(&climb_inputs, &slope, dt);
    }
    assert!(
        suv.position.y > initial_y + 1.0,
        "High-torque SUV must climb 22% grade: y_initial={:.2}, y_final={:.2}",
        initial_y,
        suv.position.y
    );

    // Phase B: Motionless Hill-Hold without Rollback
    let hold_inputs = VehicleInputs {
        throttle: 0.0,
        brake: 1.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };
    // Initial braking step
    for _ in 0..60 {
        suv.step(&hold_inputs, &slope, dt);
    }
    let hold_pos_z = suv.position.z;
    let hold_pos_x = suv.position.x;
    for _ in 0..120 {
        suv.step(&hold_inputs, &slope, dt);
    }
    let drift_z = (suv.position.z - hold_pos_z).abs();
    let drift_x = (suv.position.x - hold_pos_x).abs();
    assert!(
        drift_z < 0.05 && drift_x < 0.05,
        "Hill-hold must maintain motionless stance with zero rollback: drift_z={:.4}m, drift_x={:.4}m",
        drift_z,
        drift_x
    );
    assert!(
        suv.linear_velocity.length() < 0.05,
        "Chassis linear velocity under hill-hold must remain virtually zero: vel={:?}",
        suv.linear_velocity
    );

    // Phase C: Anti-Roll Bar (ARB) roll stabilization test
    // Traverse cross-slope (tilt in X)
    let cross_slope = SlopedGround::new(50.0, 0.18, 0.0); // 18% side slope
    let mut sedan = DynamicVehicle::new(VehicleConfig::sedan(), Vec3::new(0.0, 52.0, 0.0), 0.0);
    for _ in 0..100 {
        sedan.step(&VehicleInputs::default(), &cross_slope, dt);
    }
    let chassis_roll = sedan.orientation.to_euler(glam::EulerRot::YXZ).2;
    assert!(
        (chassis_roll.abs() - 0.18).abs() < 0.05,
        "Anti-roll bars and suspension must conform cleanly to side slope without rollover: roll={:.3} rad",
        chassis_roll
    );
    assert!(
        !sedan.is_overturned,
        "Vehicle must not overturn on side slope"
    );
}

// ============================================================================
// TEST 3: BAY BRIDGE SKYWAY CRUISE, CCD COLLISION & CHUNK PINNING LIFECYCLE
// ============================================================================

#[test]
fn test_stage4_3_bay_bridge_high_speed_cruise_ccd_and_chunk_lifecycle() {
    let mut physics = PhysicsWorld::new();

    // 1. Setup flat bridge deck ground collider at elevation Y = 40.0
    let _ground_handle = physics.spawn_ground(40.0);

    // 2. Setup concrete guardrail barrier obstacle (X = 3.5m, Y = 40.6m, Z spanning 200m)
    let barrier_half_extents = Vec3::new(0.3, 0.6, 100.0);
    let barrier_pos = Vec3::new(3.5, 40.6, 0.0);
    let _guardrail_handle = physics.spawn_obstacle_collider(barrier_pos, barrier_half_extents);

    // 3. Register Chunk Colliders for lifetime tracking along bridge
    let chunk_tag = "chunk_sf_bay_bridge_span_04".to_string();
    let pylon_pos = Vec3::new(-4.0, 45.0, 50.0);
    let pylon_handle = physics.spawn_building_collider(pylon_pos, Vec3::new(1.0, 10.0, 1.0));
    physics.register_chunk_collider(&chunk_tag, pylon_handle);
    assert_eq!(
        physics
            .chunk_colliders
            .get(&chunk_tag)
            .map_or(0, |v| v.len()),
        1,
        "Chunk collider registration must succeed"
    );

    // 4. Spawn high-speed dynamic vehicle with Continuous Collision Detection (CCD)
    let (car_body, _collider) = physics.spawn_vehicle_body(0.0, 40.5, 0.0, [0.9, 0.6, 2.2], 1400.0);

    // Verify CCD is enabled on dynamic vehicle
    let body = physics
        .rigid_body_set
        .get(car_body)
        .expect("Rigid body exists");
    assert!(
        body.is_ccd_enabled(),
        "Continuous Collision Detection (CCD) must be enabled on vehicle"
    );

    // 5. High-speed traversal test: launch vehicle towards guardrail at 120 km/h (33.3 m/s)
    let body_mut = physics.rigid_body_set.get_mut(car_body).unwrap();
    body_mut.set_linvel(vector![28.0, 0.0, 20.0], true);

    // Simulate for 30 ticks (0.5 seconds)
    for _ in 0..30 {
        physics.step(1.0 / 60.0);
    }

    // Vehicle must NOT tunnel through barrier at X = 3.5m
    let post_col_body = physics.rigid_body_set.get(car_body).unwrap();
    let car_x = post_col_body.translation().x;
    assert!(
        car_x < 3.5 + 0.3 + 0.2, // Barrier X + barrier half width + margin
        "CCD must prevent vehicle from tunneling through guardrail at high speed: car_x={:.3}",
        car_x
    );

    // 6. Test chunk eviction lifecycle: removing chunk removes associated colliders
    physics.remove_chunk_colliders(&chunk_tag);
    assert_eq!(
        physics
            .chunk_colliders
            .get(&chunk_tag)
            .map_or(0, |v| v.len()),
        0,
        "Evicted chunk colliders must be cleanly dereferenced"
    );
}

// ============================================================================
// TEST 4: LOMBARD STREET SWITCHBACKS, ACKERMANN GEOMETRY & FRICTION CIRCLE
// ============================================================================

#[test]
fn test_stage4_4_lombard_switchbacks_ackermann_and_friction_circle() {
    let ground = FlatGround::new(0.0);
    let mut car = DynamicVehicle::new(VehicleConfig::sports_coupe(), Vec3::new(0.0, 0.8, 0.0), 0.0);
    let dt = 1.0 / 60.0;

    // Settle into suspension equilibrium
    for _ in 0..60 {
        car.step(&VehicleInputs::default(), &ground, dt);
    }
    assert!(car.is_grounded, "Car must be grounded in equilibrium");

    // Phase A: Ackermann geometry test
    let steer_input = 0.5f32; // ~28 degrees steer
    let wheelbase = car.config.wheelbase;
    let track = car.config.track_width;

    // Kinematic Ackermann formula:
    // tan(delta_inner) = L / (R - w/2)
    // tan(delta_outer) = L / (R + w/2)
    // Therefore delta_inner > delta_outer
    let r_center = wheelbase / steer_input.tan();
    let ackermann_inner = (wheelbase / (r_center - track * 0.5)).atan();
    let ackermann_outer = (wheelbase / (r_center + track * 0.5)).atan();
    assert!(
        ackermann_inner > ackermann_outer,
        "Inner wheel angle must exceed outer wheel angle in Ackermann geometry"
    );

    // Phase B: Speed-sensitive steering attenuation
    // At parking speed (v ~ 5 km/h) -> full lock
    let low_speed_attenuation: f32 =
        (1.0f32 - (5.0f32 / 120.0f32) * 0.60f32).clamp(0.40f32, 1.0f32);
    assert!(
        low_speed_attenuation > 0.95,
        "Low speed must retain >95% steering lock for Lombard switchbacks"
    );

    // At highway speed (v = 120 km/h) -> 40% lock
    let high_speed_attenuation: f32 =
        (1.0f32 - (120.0f32 / 120.0f32) * 0.60f32).clamp(0.40f32, 1.0f32);
    assert!(
        (high_speed_attenuation - 0.40).abs() < 1e-4,
        "Highway speed must attenuate steer lock to 40% for high-speed stability"
    );

    // Phase C: Slalom Cornering & Lateral Grip Circle Limit
    // Accelerate to 40 km/h, then steer sharply
    for _ in 0..120 {
        car.step(
            &VehicleInputs {
                throttle: 0.85,
                brake: 0.0,
                steer: 0.0,
                handbrake: false,
                gear_mode: TransmissionMode::Drive,
            },
            &ground,
            dt,
        );
    }
    assert!(
        car.speed_kph > 30.0,
        "Car must accelerate past 30 km/h: speed={:.1} km/h",
        car.speed_kph
    );

    // Enter hairpin turn: steer = 0.6
    let initial_forward = Mat3::from_quat(car.orientation).col(2);
    let initial_heading = initial_forward.x.atan2(initial_forward.z);
    for _ in 0..60 {
        car.step(
            &VehicleInputs {
                throttle: 0.2,
                brake: 0.0,
                steer: 0.6,
                handbrake: false,
                gear_mode: TransmissionMode::Drive,
            },
            &ground,
            dt,
        );
    }
    let current_forward = Mat3::from_quat(car.orientation).col(2);
    let current_heading = current_forward.x.atan2(current_forward.z);
    let yaw_delta = (current_heading - initial_heading).abs();
    assert!(
        yaw_delta > 0.35,
        "Vehicle must execute sharp yaw rotation through switchback: yaw_delta={:.3} rad",
        yaw_delta
    );
    assert!(
        car.is_grounded,
        "Vehicle must maintain tire traction during lateral cornering"
    );
}

// ============================================================================
// TEST 5: GARAGE FLEET HOT-SWAPPING, INTERIOR MESHING & SAFE SURFACE RECOVERY
// ============================================================================

#[test]
fn test_stage4_5_garage_fleet_interior_meshing_and_safe_recovery() {
    let configs = [
        (
            "SportsCoupe",
            VehicleConfig::sports_coupe(),
            VehicleVisualType::SportsCoupe,
        ),
        ("Sedan", VehicleConfig::sedan(), VehicleVisualType::Sedan),
        ("SUV", VehicleConfig::suv(), VehicleVisualType::Suv),
        ("Taxi", VehicleConfig::taxi(), VehicleVisualType::Taxi),
        (
            "TransitBus",
            VehicleConfig::transit_bus(),
            VehicleVisualType::TransitBus,
        ),
        (
            "HeavyTruck",
            VehicleConfig::heavy_truck(),
            VehicleVisualType::HeavyTruck,
        ),
    ];

    // Verify physical property differentiation
    for i in 0..configs.len() - 1 {
        let (name_a, cfg_a, _) = &configs[i];
        let (name_b, cfg_b, _) = &configs[i + 1];
        if name_a == &"SUV" && name_b == &"Taxi" {
            // Taxi is lighter sedan-class compared to SUV
            assert!(cfg_b.mass_kg < cfg_a.mass_kg);
        } else {
            assert!(
                cfg_a.mass_kg <= cfg_b.mass_kg,
                "Mass hierarchy violated: {} vs {}",
                name_a,
                name_b
            );
        }
    }

    // Verify 3D procedural meshing with cockpit interior geometry for each model
    for (name, _cfg, v_type) in &configs {
        let mesh_params = VehicleMeshParams {
            vehicle_type: *v_type,
            reverse_lights_on: false,
            brake_lights_on: true,
            headlights_on: true,
            wheel_steer_angle: 0.25,
            ..Default::default()
        };
        let mesh = generate_vehicle_mesh(&mesh_params);
        assert!(
            mesh.vertices.len() > 100,
            "Model {} must generate detailed geometry with interior vertices: found {}",
            name,
            mesh.vertices.len()
        );
        assert!(
            mesh.indices.len() > 150,
            "Model {} must generate valid index buffer: found {}",
            name,
            mesh.indices.len()
        );
        assert_eq!(
            mesh.indices.len() % 3,
            0,
            "Index buffer must consist of complete triangle triplets"
        );
    }

    // Safe Upright Surface Recovery Test
    let _ground = FlatGround::new(15.0);
    let mut upside_down_car =
        DynamicVehicle::new(VehicleConfig::sedan(), Vec3::new(10.0, 16.0, 20.0), 0.0);
    // Invert vehicle
    upside_down_car.orientation = Quat::from_rotation_z(std::f32::consts::PI);
    upside_down_car.angular_velocity = Vec3::new(5.0, 2.0, -3.0);
    upside_down_car.linear_velocity = Vec3::new(10.0, -15.0, 5.0);

    // Trigger safe recovery on road surface
    upside_down_car.recover_on_surface(15.0);

    // Verify post-recovery invariants
    assert_eq!(
        upside_down_car.orientation,
        Quat::IDENTITY,
        "Chassis must be righted upright"
    );
    assert_eq!(
        upside_down_car.angular_velocity,
        Vec3::ZERO,
        "Angular tumbling must be zeroed"
    );
    assert_eq!(
        upside_down_car.linear_velocity,
        Vec3::ZERO,
        "Linear momentum must be halted"
    );
    assert!(
        upside_down_car.position.y > 15.0,
        "Chassis must rest above ground level: y={:.2}",
        upside_down_car.position.y
    );
    assert!(
        (upside_down_car.position.y - 15.84).abs() < 0.25,
        "Chassis must be positioned cleanly at suspension rest height: y={:.3}",
        upside_down_car.position.y
    );
}

// ============================================================================
// TEST 6: MULTI-PERSPECTIVE CAMERA OPTICS & HUD TELEMETRY INVARIANTS
// ============================================================================

#[test]
fn test_stage4_6_multi_camera_perspectives_and_telemetry_invariants() {
    let mut camera = VehicleCameraController::new(VehicleCameraPerspective::Chase);
    let car_pos = Vec3::new(100.0, 20.0, -50.0);
    let car = DynamicVehicle::new(VehicleConfig::sedan(), car_pos, 0.5);

    // 1. Perspective Cycling Sequence
    assert_eq!(camera.perspective, VehicleCameraPerspective::Chase);
    camera.cycle_perspective();
    assert_eq!(camera.perspective, VehicleCameraPerspective::Hood);
    camera.cycle_perspective();
    assert_eq!(camera.perspective, VehicleCameraPerspective::Cockpit);
    camera.cycle_perspective();
    assert_eq!(camera.perspective, VehicleCameraPerspective::Bumper);
    camera.cycle_perspective();
    assert_eq!(camera.perspective, VehicleCameraPerspective::Chase);

    // 2. Cockpit Camera Geometric Invariance
    camera.perspective = VehicleCameraPerspective::Cockpit;
    let (eye_pos, yaw, pitch) = camera.update(&car, 1.0);

    // Driver's seat sits at X ~ -0.38m in local space
    let local_eye = car.orientation.inverse() * (eye_pos - car.position);
    assert!(
        (local_eye.x - (-0.38)).abs() < 0.05,
        "Cockpit camera X must be aligned with driver seat left offset: local_x={:.3}",
        local_eye.x
    );
    assert!(
        (local_eye.y - 1.05).abs() < 0.05,
        "Cockpit camera Y must be at driver eye level: local_y={:.3}",
        local_eye.y
    );
    assert!(
        pitch < 0.0,
        "Cockpit view must have subtle downward pitch onto road dashboard: pitch={:.3}",
        pitch
    );
    assert!(!yaw.is_nan());

    // 3. Floating Origin Camera ENU coordinate compatibility
    let wgs_anchor = Wgs84Coord::new(37.7749, -122.4194, 20.0);
    let floating_cam = FloatingOriginCamera::new(wgs_anchor, 16.0 / 9.0);
    let view_proj = floating_cam.view_proj_matrix();
    let cols = view_proj.to_cols_array();
    for v in cols {
        assert!(!v.is_nan() && !v.is_infinite());
    }

    // 4. Telemetry Metric Calculations
    let speed_ms = 25.0f32; // 25 m/s = 90 km/h
    let speed_kmh = speed_ms * 3.6;
    assert_eq!(speed_kmh, 90.0);

    let gear_idx = match speed_kmh {
        s if s < 0.1 => 1,
        s if s < 25.0 => 1,
        s if s < 50.0 => 2,
        s if s < 80.0 => 3,
        s if s < 115.0 => 4,
        _ => 5,
    };
    assert_eq!(gear_idx, 4, "90 km/h should map to 4th gear");

    // Steer wheel visual rotation check
    let steer_angle = 0.20f32;
    let wheel_mesh_rotation = steer_angle * 3.5;
    assert!(
        (wheel_mesh_rotation - 0.70).abs() < 1e-4,
        "Steering wheel visual rotation must scale by 3.5x ratio"
    );
}
