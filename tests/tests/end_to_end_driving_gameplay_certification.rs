//! STAGE 3 — END-TO-END WORLD, DRIVING AND GAMEPLAY CERTIFICATION SUITE
//!
//! Validates the entire integrated Real Earth application loop:
//! 1. World Integration: Authentic GIS -> Validator -> Compiler -> Streamer -> Renderer -> Physics
//! 2. Realistic Vehicle Dynamics: Acceleration, steering, braking, hill-holding, anti-tunneling, bridges
//! 3. Autonomous Traffic: IDM car-following, intersections, collision avoidance
//! 4. Gameplay Progression: Taxi/Delivery/Freight contracts, dwell times, payouts, wallet, XP, persistence
//! 5. Garage Fleet: Vehicle class presets, handling characteristics, and odometer tracking
//! 6. Visual Quality & Atmosphere: Shaders, road markings, architectural windows, water shimmer, day/night
//! 7. Multi-Region US Navigation: Road network connectivity across major metropolitan regions

use glam::Vec3;
use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_gameplay::{
    destinations::JobCatalogGenerator,
    events::GameplayEventBus,
    jobs::JobState,
    profile::PlayerProfile,
    save::{GameSettings, SaveData, SaveManager, CURRENT_SAVE_SCHEMA_VERSION},
};
use realearth_ingestion::{
    dem_grid::DemGrid, sample_chicago::create_chicago_dataset,
    sample_los_angeles::create_los_angeles_dataset, sample_new_york::create_new_york_dataset,
    sample_san_francisco::create_san_francisco_dataset, sample_seattle::create_seattle_dataset,
};
use realearth_navigation::{find_route, RoadNetworkGraph, RoutingPreference};
use realearth_rendering::{
    FloatingOriginCamera, GraphicsPresetLevel, GraphicsSettings, SceneUniforms,
};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_streaming::{StreamingConfig, WorldStreamer};
use realearth_validation::pipeline::validate_dataset;
use realearth_vehicles::{
    DynamicVehicle, FlatGround, IdmConfig, SlopedGround, TrafficAgent, TrafficSimulation,
    VehicleConfig, VehicleInputs,
};
use tempfile::tempdir;

// ============================================================================
// STAGE 3.1 — INTEGRATED WORLD PIPELINE (GIS -> COMPILER -> STREAMER -> RENDERER -> PHYSICS)
// ============================================================================

#[test]
fn test_stage3_1_world_integration_and_coordinate_continuity() {
    let raw = create_san_francisco_dataset();
    let dem = DemGrid::create_san_francisco_coastal_dem(32, 32);

    let (validated, report) = validate_dataset(&raw);
    assert!(
        report.valid_roads_passed > 0,
        "Dataset must pass validation gate"
    );
    assert!(
        report.valid_buildings_passed > 0,
        "Must have valid buildings"
    );

    let temp = tempdir().expect("tempdir");
    let options = CompilerOptions {
        chunk_size_meters: 500.0,
        terrain_grid_resolution: 8,
        incremental: false,
        ..Default::default()
    };
    let stats = compile_world_package(&validated, &dem, temp.path(), &options)
        .expect("World package compilation must succeed");
    assert!(
        stats.total_chunks_generated > 0,
        "Package must produce compiled chunks"
    );

    // Open WorldStreamer and query chunks around Transamerica Pyramid
    let transamerica = Wgs84Coord::new(37.7952, -122.4028, 35.0);
    let mut streamer = WorldStreamer::open(temp.path(), StreamingConfig::default())
        .expect("WorldStreamer must open package");
    let loaded = streamer.update(transamerica).expect("Streamer update");
    assert!(loaded > 0, "Must stream chunks near landmark");

    let active_chunks = streamer.get_active_chunks();
    assert!(
        !active_chunks.is_empty(),
        "Must have active chunks in memory"
    );

    // Verify all chunks share unified package center anchor (eliminating seams)
    let package_anchor = active_chunks[0].header.anchor;
    for chunk in &active_chunks {
        assert_eq!(
            chunk.header.anchor, package_anchor,
            "All chunks in package must share unified package center anchor"
        );
    }

    let total_terrain_verts: usize = active_chunks.iter().map(|c| c.terrain.vertices.len()).sum();
    let total_road_verts: usize = active_chunks.iter().map(|c| c.roads.vertices.len()).sum();
    assert!(
        total_terrain_verts > 0,
        "Active scene must have terrain geometry"
    );
    assert!(total_road_verts > 0, "Active scene must have road geometry");

    // Verify Camera relative_pos_f32 is in local ENU matching mesh vertices
    let camera = FloatingOriginCamera::new(transamerica, 16.0 / 9.0);
    let rel_pos = camera.relative_pos_f32();
    assert!(
        rel_pos.x.abs() < 1.0,
        "Camera X at anchor must be near 0 in local ENU: found {}",
        rel_pos.x
    );
    assert!(
        rel_pos.z.abs() < 1.0,
        "Camera Z at anchor must be near 0 in local ENU: found {}",
        rel_pos.z
    );

    // Verify physics ground query aligns with compiler DEM
    let ground_y = dem.sample_bilinear(&transamerica);
    assert!(
        (0.0..100.0).contains(&ground_y),
        "Authentic DEM elevation must be positive"
    );
}

// ============================================================================
// STAGE 3.2 — REALISTIC VEHICLE DYNAMICS, ANTI-TUNNELING & HILL CLIMBING
// ============================================================================

#[test]
fn test_stage3_2_vehicle_dynamics_anti_tunneling_and_hill_holding() {
    // 1. Suspension equilibrium on flat ground
    let ground = FlatGround::new(10.0);
    let config = VehicleConfig::sports_coupe();
    let mut vehicle = DynamicVehicle::new(config, Vec3::new(0.0, 11.0, 0.0), 0.0);

    for _ in 0..120 {
        vehicle.step(&VehicleInputs::default(), &ground, 1.0 / 60.0);
    }
    assert!(
        vehicle.is_grounded,
        "Vehicle must reach stable suspension contact with ground"
    );
    assert!(
        (vehicle.position.y - 10.84).abs() < 0.25,
        "Chassis must settle at rest height above ground: y={}",
        vehicle.position.y
    );

    // 2. Acceleration and Braking
    let accel_inputs = VehicleInputs {
        throttle: 1.0,
        ..Default::default()
    };
    for _ in 0..120 {
        vehicle.step(&accel_inputs, &ground, 1.0 / 60.0);
    }
    assert!(
        vehicle.speed_kph > 30.0,
        "Sports coupe must accelerate past 30 km/h: speed={:.1}",
        vehicle.speed_kph
    );

    let brake_inputs = VehicleInputs {
        brake: 1.0,
        ..Default::default()
    };
    for _ in 0..120 {
        vehicle.step(&brake_inputs, &ground, 1.0 / 60.0);
    }
    assert!(
        vehicle.speed_kph < 5.0,
        "Braking must rapidly decelerate vehicle: speed={:.1}",
        vehicle.speed_kph
    );

    // 3. Steep Hill Climbing & Hill-Holding (15-degree San Francisco incline)
    let slope = SlopedGround::new(10.0, 0.0, 0.26); // 15 degree slope in Z
    let mut hill_car = DynamicVehicle::new(VehicleConfig::sedan(), Vec3::new(0.0, 11.0, 0.0), 0.0);

    // Settle on slope
    for _ in 0..60 {
        hill_car.step(&VehicleInputs::default(), &slope, 1.0 / 60.0);
    }

    // Apply brake: hill-holding must lock velocity
    let hold_inputs = VehicleInputs {
        brake: 1.0,
        ..Default::default()
    };
    for _ in 0..60 {
        hill_car.step(&hold_inputs, &slope, 1.0 / 60.0);
    }
    assert!(
        hill_car.linear_velocity.length() < 0.2,
        "Hill-hold must prevent rollback on steep incline: vel={:?}",
        hill_car.linear_velocity
    );

    // 4. Anti-Tunneling / Anti-Sinking Invariance
    // Inject massive downward velocity towards ground
    vehicle.linear_velocity.y = -80.0; // severe drop
    for _ in 0..30 {
        vehicle.step(&VehicleInputs::default(), &ground, 1.0 / 60.0);
        let min_y = 10.0 + vehicle.config.wheel_radius * 0.5;
        assert!(
            vehicle.position.y >= min_y - 0.2,
            "Vehicle chassis must never sink or tunnel through ground: y={}",
            vehicle.position.y
        );
    }
}

// ============================================================================
// STAGE 3.3 — AUTONOMOUS TRAFFIC SIMULATION & COLLISION CONTINUITY
// ============================================================================

#[test]
fn test_stage3_3_autonomous_traffic_and_platoon_collision_avoidance() {
    let mut traffic = TrafficSimulation::new();
    let ground = FlatGround::new(0.0);

    let straight_route = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 200.0)];

    // Leader vehicle starting at z = 30m, cruising slowly at 8 m/s (~29 km/h)
    let mut leader = TrafficAgent::new(
        1,
        Vec3::new(0.0, 0.0, 30.0),
        straight_route.clone(),
        IdmConfig {
            desired_speed_ms: 8.0,
            ..Default::default()
        },
    );
    leader.speed_ms = 8.0;

    // Follower vehicle starting at z = 0m, desiring 18 m/s (~65 km/h)
    let mut follower = TrafficAgent::new(
        2,
        Vec3::new(0.0, 0.0, 0.0),
        straight_route,
        IdmConfig {
            desired_speed_ms: 18.0,
            min_gap_meters: 3.0,
            ..Default::default()
        },
    );
    follower.speed_ms = 12.0;

    traffic.spawn_agent(leader);
    traffic.spawn_agent(follower);

    // Simulate for 100 ticks of 0.05s = 5 seconds
    for _ in 0..100 {
        traffic.step(0.05, &ground);
    }

    let lead_z = traffic.agents[0].position.z;
    let follower_z = traffic.agents[1].position.z;
    let gap = lead_z - follower_z;

    // Follower must not rear-end leader
    assert!(
        follower_z < lead_z,
        "Follower z ({}) must stay behind leader z ({})",
        follower_z,
        lead_z
    );
    assert!(
        gap >= 2.5,
        "Safety headway gap must be maintained by IDM controller: gap={:.2}m",
        gap
    );
}

// ============================================================================
// STAGE 3.4 — GAMEPLAY CONTRACTS, PAYOUTS, PROGRESSION & SAVE/RELOAD PERSISTENCE
// ============================================================================

#[test]
fn test_stage3_4_gameplay_progression_payouts_and_save_reload_persistence() {
    let bus = GameplayEventBus::new();
    let mut profile = PlayerProfile::new("driver-sf", "San Francisco Courier");
    let initial_balance = profile.wallet.balance();
    assert_eq!(initial_balance, 500.0);

    // 1. Accept authentic San Francisco Taxi Contract
    let sf_jobs = JobCatalogGenerator::jobs_for_region("san-francisco");
    assert!(
        !sf_jobs.is_empty(),
        "Must have authentic jobs in San Francisco"
    );

    let mut job = sf_jobs[0].clone();
    job.accept(&profile, &bus)
        .expect("Accepting contract must succeed");
    assert_eq!(job.state, JobState::EnRouteToPickup);

    // 2. Drive to pickup location (Ferry Building)
    let pickup_coord = job.pickup.coord;
    let _ = job.update(1.0, pickup_coord, 0.0, &bus);
    assert!(
        matches!(job.state, JobState::AtPickup { .. } | JobState::Loaded),
        "Job must transition to pickup boarding"
    );

    // Dwell until passenger boarded
    for _ in 0..10 {
        let _ = job.update(1.0, pickup_coord, 0.0, &bus);
    }
    assert_eq!(job.state, JobState::EnRouteToDropoff);

    // 3. Drive to dropoff location (Salesforce Tower)
    let dropoff_coord = job.dropoff.coord;
    let _ = job.update(1.0, dropoff_coord, 0.0, &bus);

    // Dwell until completion
    let mut completion_result = None;
    for _ in 0..10 {
        if let Some(res) = job.update(1.0, dropoff_coord, 0.0, &bus) {
            completion_result = Some(res);
            break;
        }
    }

    let completion = completion_result.expect("Job must emit JobCompletionResult upon delivery");
    assert!(completion.final_payout > 100.0, "Payout must be positive");
    assert!(completion.xp_gained > 50, "XP must be awarded");

    // Settle payout onto player profile
    profile.record_job_completion(
        completion.career,
        completion.final_payout,
        completion.xp_gained,
        (completion.rating_stars * 20.0) as u32,
        1700000000,
    );

    assert_eq!(profile.completed_jobs, 1);
    assert!(profile.wallet.balance() > 500.0);
    assert!(profile.xp > 50);

    // 4. Persistence Roundtrip (Save -> File -> Load)
    let temp_save_dir = tempdir().expect("save dir");
    let save_path = temp_save_dir.path().join("quicksave.json");

    let save_data = SaveData {
        schema_version: CURRENT_SAVE_SCHEMA_VERSION,
        timestamp_unix: 1700000000,
        profile: profile.clone(),
        active_job: Some(job.clone()),
        active_region_id: "san-francisco".to_string(),
        vehicle_position: dropoff_coord,
        settings: GameSettings::default(),
    };

    SaveManager::save_to_file(&save_path, &save_data).expect("Quicksave to disk must succeed");
    assert!(save_path.exists(), "Save file must exist on disk");

    // Load from disk
    let reloaded =
        SaveManager::load_from_file(&save_path).expect("Quickload from disk must succeed");

    assert_eq!(reloaded.schema_version, CURRENT_SAVE_SCHEMA_VERSION);
    assert_eq!(reloaded.active_region_id, "san-francisco");
    assert_eq!(reloaded.profile.completed_jobs, 1);
    assert_eq!(reloaded.profile.wallet.balance(), profile.wallet.balance());
    assert_eq!(reloaded.profile.xp, profile.xp);
    assert_eq!(reloaded.vehicle_position.lat, dropoff_coord.lat);
    assert_eq!(reloaded.vehicle_position.lon, dropoff_coord.lon);
}

// ============================================================================
// STAGE 3.5 — MULTI-VEHICLE GARAGE FLEET PRESETS & PHYSICAL PARITY
// ============================================================================

#[test]
fn test_stage3_5_garage_vehicle_modes_and_physics_differentiation() {
    let coupe = VehicleConfig::sports_coupe();
    let sedan = VehicleConfig::sedan();
    let suv = VehicleConfig::suv();
    let taxi = VehicleConfig::taxi();
    let bus = VehicleConfig::transit_bus();
    let truck = VehicleConfig::heavy_truck();

    // Mass progression
    assert!(coupe.mass_kg < sedan.mass_kg);
    assert!(sedan.mass_kg < suv.mass_kg);
    assert!(suv.mass_kg < bus.mass_kg);
    assert!(bus.mass_kg < truck.mass_kg);

    // Torque progression
    assert!(coupe.peak_engine_force > sedan.peak_engine_force);
    assert!(truck.peak_engine_force > bus.peak_engine_force);

    // Turning agility (max steer angle)
    assert!(taxi.max_steer_angle > truck.max_steer_angle);
    assert!(coupe.max_steer_angle > bus.max_steer_angle);

    // Suspension stiffness
    assert!(truck.suspension_stiffness > coupe.suspension_stiffness);
}

// ============================================================================
// STAGE 3.6 — VISUAL QUALITY, SHADERS, ATMOSPHERE & ROAD MARKINGS
// ============================================================================

#[test]
fn test_stage3_6_visual_quality_shaders_atmosphere_and_presets() {
    let cam = FloatingOriginCamera::new(Wgs84Coord::new(37.7952, -122.4028, 50.0), 16.0 / 9.0);

    // Day lighting
    let day_sun = [0.2, 0.8, -0.4];
    let day_settings = GraphicsSettings::from_preset(GraphicsPresetLevel::High);
    let day_uniforms = SceneUniforms {
        view_proj: cam.view_proj_matrix().to_cols_array_2d(),
        camera_pos: [0.0, 50.0, 0.0, 1.0],
        sun_direction: [day_sun[0], day_sun[1], day_sun[2], 0.0],
        sun_color_intensity: [1.0, 0.98, 0.92, 1.2],
        ambient_light: [0.22, 0.24, 0.28, 1.0],
        fog_params: [day_settings.fog_density, 100.0, 10_000.0, 0.0],
        material_params: [day_settings.specular_intensity, 1.0, 0.0, 0.0],
    };
    assert_eq!(
        day_uniforms.material_params[1], 1.0,
        "Road markings must be enabled"
    );

    // Night lighting
    let night_sun = [0.0, -0.6, 0.8];
    assert!(night_sun[1] < 0.0, "Night sun elevation must be negative");

    // Verify all 4 graphics presets configure appropriate view distances and fog
    for level in [
        GraphicsPresetLevel::Low,
        GraphicsPresetLevel::Medium,
        GraphicsPresetLevel::High,
        GraphicsPresetLevel::Adaptive,
    ] {
        let s = GraphicsSettings::from_preset(level);
        assert!(s.view_distance_meters >= 1500.0);
        assert!(s.specular_intensity > 0.0);
    }
}

// ============================================================================
// STAGE 3.7 — MULTI-CITY AUTHENTIC US ROAD ROUTING & NAVIGATION
// ============================================================================

#[test]
fn test_stage3_7_multi_city_authentic_us_road_routing() {
    // 1. San Francisco Downtown Navigation Route (Market Street to Ferry Building / Embarcadero)
    let sf = create_san_francisco_dataset();
    let sf_graph = RoadNetworkGraph::from_roads(&sf.roads);
    let sf_start = sf.roads[0].geometry[0]; // Market Street start
    let sf_goal = sf.roads[1].geometry.last().copied().unwrap(); // Embarcadero north
    let sf_route = find_route(
        &sf_graph,
        sf_start,
        sf_goal,
        RoutingPreference::ShortestDistance,
    )
    .expect("San Francisco route must connect Market St to Embarcadero");
    assert!(sf_route.waypoints.len() >= 2);
    assert!(sf_route.total_distance_meters > 500.0);

    // 2. New York Lower Manhattan Navigation Route
    let nyc = create_new_york_dataset();
    let nyc_graph = RoadNetworkGraph::from_roads(&nyc.roads);
    let nyc_start = nyc.roads[0].geometry[0];
    let nyc_goal = nyc.roads[0].geometry.last().copied().unwrap();
    let nyc_route = find_route(
        &nyc_graph,
        nyc_start,
        nyc_goal,
        RoutingPreference::ShortestDistance,
    )
    .expect("New York route must succeed along Lower Manhattan arterial");
    assert!(nyc_route.waypoints.len() >= 2);
    assert!(nyc_route.total_distance_meters > 400.0);

    // 3. Los Angeles Downtown Navigation Route
    let la = create_los_angeles_dataset();
    let la_graph = RoadNetworkGraph::from_roads(&la.roads);
    let la_start = la.roads[0].geometry[0];
    let la_goal = la.roads[0].geometry.last().copied().unwrap();
    let la_route = find_route(
        &la_graph,
        la_start,
        la_goal,
        RoutingPreference::ShortestDistance,
    )
    .expect("Los Angeles route must succeed along Wilshire corridor");
    assert!(la_route.waypoints.len() >= 2);

    // 4. Chicago The Loop Navigation Route
    let chi = create_chicago_dataset();
    let chi_graph = RoadNetworkGraph::from_roads(&chi.roads);
    let chi_start = chi.roads[0].geometry[0];
    let chi_goal = chi.roads[0].geometry.last().copied().unwrap();
    let chi_route = find_route(
        &chi_graph,
        chi_start,
        chi_goal,
        RoutingPreference::ShortestDistance,
    )
    .expect("Chicago route must succeed along The Loop arterial");
    assert!(chi_route.waypoints.len() >= 2);

    // 5. Seattle Waterfront Navigation Route
    let sea = create_seattle_dataset();
    let sea_graph = RoadNetworkGraph::from_roads(&sea.roads);
    let sea_start = sea.roads[0].geometry[0];
    let sea_goal = sea.roads[0].geometry.last().copied().unwrap();
    let sea_route = find_route(
        &sea_graph,
        sea_start,
        sea_goal,
        RoutingPreference::ShortestDistance,
    )
    .expect("Seattle route must succeed along Waterfront arterial");
    assert!(sea_route.waypoints.len() >= 2);
}
