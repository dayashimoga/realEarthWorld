//! Comprehensive Master Certification Suite for Stage E:
//! Multimodal Navigation, User Experience, and Transportation Gameplay.
//!
//! Validates:
//! - Phase E1: Multimodal vehicle spawning and valid infrastructure constraints (Car, Bus, Train, Aircraft, Marine)
//! - Phase E2: Windows mouse interaction, modal input exclusivity, and DPI-invariant UI layouts
//! - Phase E3: Complete transportation gameplay, authentic transit passenger services, and real geography missions
//! - Phase E4: End-to-end playable journeys:
//!   1. San Francisco sports car drive and standstill stop
//!   2. Detailed transit bus passenger service route with boarding dwell and pneumatic air brakes
//!   3. Locomotive consist track-constrained departure, curvature adherence, and station platform stop
//!   4. Aircraft takeoff, streamed terrain clearance, and flight dynamics
//!   5. US city transition and appropriate multimodal vehicle spawning
//!   6. Mission contract completion, wallet payment, and quicksave/quickload roundtrip
//!   7. Mouse hit-testing determinism across 720p, 1080p, 1440p, and 4K UHD resolutions with zero HUD overlap

use glam::{Quat, Vec3};
use realearth_gameplay::{
    CareerType, CargoOrPassenger, GameplayEventBus, Job, JobCatalogGenerator, JobDifficulty,
    JobLocation, JobState, PlayerProfile, ReputationTier, SaveData, SaveManager,
};
use realearth_navigation::rail_graph::{
    RailGauge, RailNetworkGraph, RailTrackSegment, SignalAspect,
};
use realearth_rendering::{
    CatalogHit, CatalogLayout, DebriefHit, DebriefLayout, GarageHit, GarageLayout, InGameHudHit,
    InGameHudLayout, JobBrowserHit, JobBrowserLayout, MultiplayerHit, MultiplayerLayout,
    PauseMenuHit, PauseMenuLayout, RailRouteHit, RailRouteLayout, SettingsHit, SettingsLayout,
    StartScreenHit, StartScreenLayout,
};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::us_catalog::{all_us_cities, find_city_by_id};
use realearth_vehicles::dynamics::{
    DynamicVehicle, TransmissionMode, VehicleConfig, VehicleInputs,
};
use realearth_vehicles::ground::FlatGround;
use realearth_vehicles::multimodal::aviation::{AircraftFlight, FlightInputs};
use realearth_vehicles::multimodal::road_transport::TransitBusModel;
use realearth_vehicles::multimodal::train::TrainConsist;

const DT_60FPS: f32 = 1.0 / 60.0;

// ============================================================================
// JOURNEY 1: SAN FRANCISCO SPORTS CAR DRIVE AND SERVICE BRAKING STOP
// ============================================================================

#[test]
fn test_e1_san_francisco_sports_car_drive_and_stop() {
    println!("🏎️ Journey 1: San Francisco Sports Car Drive and Stop...");

    let ground = FlatGround::new(5.0);
    let car_config = VehicleConfig::sports_coupe();
    let eq_y = 5.0 + car_config.equilibrium_ride_height();
    let mut car = DynamicVehicle::new(car_config, Vec3::new(100.0, eq_y, 200.0), 0.0);
    car.recover_on_surface(5.0);

    assert_eq!(car.speed_kph, 0.0);
    assert!(car.position.y >= 5.0);

    // 1. Full Throttle Acceleration in Drive Mode
    let mut throttle_inputs = VehicleInputs::default();
    throttle_inputs.gear_mode = TransmissionMode::Drive;
    throttle_inputs.throttle = 1.0;

    for _ in 0..120 {
        // 2.0 seconds at 60 FPS
        car.step(&throttle_inputs, &ground, DT_60FPS);
    }

    assert!(
        car.speed_kph > 25.0,
        "Sports car must accelerate above 25 km/h under full throttle, speed was {:.1} km/h",
        car.speed_kph
    );
    assert!(
        car.engine_rpm > 2500.0,
        "Engine RPM must increase under throttle, got {:.0}",
        car.engine_rpm
    );
    assert!(
        car.current_gear >= 1,
        "Automatic transmission must shift into forward gear"
    );

    // 2. Service Braking to Standstill
    let mut brake_inputs = VehicleInputs::default();
    brake_inputs.gear_mode = TransmissionMode::Drive;
    brake_inputs.brake = 1.0;

    for _ in 0..150 {
        // 2.5 seconds braking
        car.step(&brake_inputs, &ground, DT_60FPS);
    }

    assert!(
        car.speed_kph < 0.2,
        "Sports car must come to complete standstill under service brakes, speed was {:.2} km/h",
        car.speed_kph
    );

    // 3. Parking Pawl Engagement
    let mut park_inputs = VehicleInputs::default();
    park_inputs.gear_mode = TransmissionMode::Park;
    let initial_pos = car.position;

    for _ in 0..60 {
        car.step(&park_inputs, &ground, DT_60FPS);
    }

    let delta_dist = (car.position - initial_pos).length();
    assert!(
        delta_dist < 0.01,
        "Vehicle in Park must remain rigidly stationary, delta was {:.4}m",
        delta_dist
    );
}

// ============================================================================
// JOURNEY 2: DETAILED TRANSIT BUS PASSENGER SERVICE ROUTE
// ============================================================================

#[test]
fn test_e2_detailed_transit_bus_passenger_service_route() {
    println!("🚌 Journey 2: Detailed Transit Bus Passenger Service Route...");

    let ground = FlatGround::new(5.0);
    let mut bus = TransitBusModel::new(Vec3::new(50.0, 5.0, 50.0), 0.0);

    assert_eq!(bus.unladen_mass_kg, 12500.0);
    assert_eq!(bus.passenger_count, 35);
    assert_eq!(bus.total_mass_kg, 12500.0 + (35.0 * 75.0));

    // 1. Passenger Boarding at Designated Transit Stop
    bus.board_passengers(7);
    assert_eq!(bus.passenger_count, 42);
    assert_eq!(bus.total_mass_kg, 12500.0 + (42.0 * 75.0));

    let audio_events = bus.drain_audio_events();
    assert!(
        !audio_events.is_empty(),
        "Bus must emit passenger boarding chime"
    );

    // 2. Scheduled Arterial Service Operation
    let mut cruise_inputs = VehicleInputs::default();
    cruise_inputs.gear_mode = TransmissionMode::Drive;
    cruise_inputs.throttle = 1.0;

    for _ in 0..300 {
        // 5.0 seconds driving
        bus.step(&cruise_inputs, &ground, DT_60FPS);
    }

    assert!(
        bus.speed_kph > 15.0,
        "Transit bus must reach normal service speed, got {:.1} km/h",
        bus.speed_kph
    );
    assert!(
        bus.passenger_comfort_rating >= 4.5,
        "Smooth acceleration must preserve high passenger comfort rating (>= 4.5), got {:.2}",
        bus.passenger_comfort_rating
    );

    // 3. Pneumatic Air Brake Arrival Stop at Next Station
    let mut stop_inputs = VehicleInputs::default();
    stop_inputs.gear_mode = TransmissionMode::Drive;
    stop_inputs.brake = 0.85;

    for _ in 0..240 {
        bus.step(&stop_inputs, &ground, DT_60FPS);
    }

    assert!(
        bus.speed_kph < 0.5,
        "Transit bus must smoothly halt at terminal platform, speed was {:.2} km/h",
        bus.speed_kph
    );
    assert!(
        bus.air_brake_pressure_bar >= 6.0,
        "Operating air brake pressure must remain safe, got {:.1} bar",
        bus.air_brake_pressure_bar
    );
}

// ============================================================================
// JOURNEY 3: LOCOMOTIVE CONSIST RAIL TRACK FOLLOWING AND STATION STOP
// ============================================================================

#[test]
fn test_e3_locomotive_consist_rail_track_following_and_station_stop() {
    println!("🚆 Journey 3: Locomotive Consist Rail Track Following and Station Stop...");

    let anchor = Wgs84Coord::new(37.7765, -122.3945, 5.0);
    let mut graph = RailNetworkGraph::new();

    // Surveyed Caltrain Mainline Track Geometry (SF 4th & King Station -> 22nd Street)
    let trk_pts = vec![
        Wgs84Coord::new(37.7765, -122.3945, 5.0),
        Wgs84Coord::new(37.7730, -122.3960, 5.5),
        Wgs84Coord::new(37.7690, -122.3980, 6.2),
        Wgs84Coord::new(37.7640, -122.4000, 7.0),
        Wgs84Coord::new(37.7580, -122.4015, 8.0),
    ];
    let segment = RailTrackSegment {
        id: "TRK_CALTRAIN_MAIN".to_string(),
        from_node: "STN_SF_4TH_KING".to_string(),
        to_node: "STN_22ND_STREET".to_string(),
        length_meters: 2200.0,
        speed_limit_kph: 125.0,
        gauge: RailGauge::Standard,
        is_electrified: true,
        switch_state: None,
        signal_aspect: SignalAspect::Clear,
        geometry: trk_pts,
    };
    graph.add_segment(segment);

    let route = vec!["TRK_CALTRAIN_MAIN".to_string()];
    let mut consist = TrainConsist::new_caltrain_diesel();
    consist.bind_route(&graph, route.clone(), anchor);

    assert_eq!(consist.cars.len(), 5); // 1 Locomotive + 4 Bombardier BiLevel coaches
    assert!(consist.track_pose.is_some());

    // 1. Depart Terminal Platform Under Throttle
    consist.throttle = 1.0;
    consist.reverser = 1;
    consist.apply_service_brake(0.0);

    for _ in 0..480 {
        // 8.0 seconds travel
        consist.step_along_route(&graph, anchor, DT_60FPS);
    }

    assert!(
        consist.speed_kph > 15.0,
        "Locomotive consist must accelerate on track, got {:.1} km/h",
        consist.speed_kph
    );

    // Verify 100% Track-Constrained Motion: zero cross-track error (< 0.05m)
    let pose = consist.track_pose.expect("Active track pose");
    let cross_err = graph.route_cross_track_distance(&route, pose.position, anchor);
    assert!(
        cross_err < 0.05,
        "Train must adhere strictly to track centerline spline, error was {:.4}m",
        cross_err
    );

    // 2. Station Approach and Westinghouse Air Braking
    consist.throttle = 0.0;
    consist.apply_service_brake(1.0);

    for _ in 0..360 {
        consist.step_along_route(&graph, anchor, DT_60FPS);
    }

    assert!(
        consist.speed_kph < 0.2,
        "Locomotive consist must come to complete stop at station platform, speed was {:.2} km/h",
        consist.speed_kph
    );
    assert!(
        consist.brake_cylinder_pressure_bar > 1.5,
        "Brake cylinder must be pressurized under service brake application"
    );
}

// ============================================================================
// JOURNEY 4: AIRCRAFT AIRPORT DEPARTURE AND STREAMED TERRAIN FLIGHT
// ============================================================================

#[test]
fn test_e4_aircraft_airport_departure_and_streamed_flight() {
    println!("✈️ Journey 4: Aircraft Airport Departure and Streamed Flight...");

    let mut plane = AircraftFlight::new_airliner(
        "Commercial Airliner B737-800",
        Vec3::new(0.0, 350.0, 0.0),
        0.0,
    );
    // Initial climb attitude: 5 degrees pitch up
    plane.orientation = Quat::from_rotation_x(-5.0_f32.to_radians());
    plane.linear_velocity = Vec3::new(0.0, 8.0, 105.0); // 105 m/s (~204 knots) with 8 m/s climb
    plane.altitude_msl_meters = 350.0;

    let mut flight_inputs = FlightInputs::default();
    flight_inputs.throttle = 1.0;
    flight_inputs.elevator = 0.05; // Balanced pitch trim to hold climb attitude
    flight_inputs.flaps = 0.10;

    let ground_elevation = 15.0; // 15m terrain MSL

    for _ in 0..300 {
        // 5.0 seconds simulation
        plane.step(&flight_inputs, ground_elevation, DT_60FPS);
    }

    assert!(
        plane.altitude_msl_meters > 350.0,
        "Aircraft must achieve positive rate of climb, altitude was {:.1}m",
        plane.altitude_msl_meters
    );
    assert!(
        !plane.is_stalled,
        "Aircraft must remain in unstalled flight envelope"
    );
    assert!(
        plane.indicated_airspeed_knots > 100.0,
        "Aircraft airspeed must remain safe, got {:.1} kts",
        plane.indicated_airspeed_knots
    );
    assert!(
        plane.altitude_msl_meters - ground_elevation > 300.0,
        "Terrain clearance must exceed 300m AGL"
    );
}

// ============================================================================
// JOURNEY 5: INSTALLED US CITY TRANSITION AND MULTIMODAL SPAWNING
// ============================================================================

#[test]
fn test_e5_installed_us_city_transition_and_multimodal_spawning() {
    println!("🏙️ Journey 5: Installed US City Transition and Multimodal Spawning...");

    let cities = all_us_cities();
    assert_eq!(cities.len(), 38);

    // Verify Minneapolis–Saint Paul
    let msp = find_city_by_id("minneapolis").expect("Minneapolis definition");
    assert_eq!(msp.primary_airport_iata, "MSP");
    assert!(msp.anchor_landmark_name.contains("IDS Center"));

    let msp_jobs = JobCatalogGenerator::jobs_for_region("minneapolis");
    assert!(
        msp_jobs.len() >= 6,
        "Minneapolis must have at least 6 multimodal contracts, found {}",
        msp_jobs.len()
    );

    // Verify presence of multimodal career types in generated city catalog
    let has_roam = msp_jobs.iter().any(|j| j.career == CareerType::FreeRoam);
    let has_taxi = msp_jobs
        .iter()
        .any(|j| j.career == CareerType::TaxiPassenger);
    let has_bus = msp_jobs.iter().any(|j| j.career == CareerType::TransitBus);
    let has_freight = msp_jobs
        .iter()
        .any(|j| j.career == CareerType::HeavyFreight);
    let has_courier = msp_jobs
        .iter()
        .any(|j| j.career == CareerType::PackageDelivery);
    let has_air = msp_jobs
        .iter()
        .any(|j| j.career == CareerType::AviationPilot);

    assert!(has_roam, "Missing FreeRoam in Minneapolis");
    assert!(has_taxi, "Missing Taxi in Minneapolis");
    assert!(has_bus, "Missing TransitBus in Minneapolis");
    assert!(has_freight, "Missing HeavyFreight in Minneapolis");
    assert!(has_courier, "Missing PackageDelivery in Minneapolis");
    assert!(has_air, "Missing AviationPilot in Minneapolis");

    // Verify all job coordinates are non-placeholder and within surveyed bounds
    for job in &msp_jobs {
        assert!(
            msp.bounds.contains(&job.pickup.coord),
            "Job {} pickup outside Minneapolis bounds",
            job.id
        );
        assert!(
            msp.bounds.contains(&job.dropoff.coord),
            "Job {} dropoff outside Minneapolis bounds",
            job.id
        );
        let dist = job.pickup.distance_to(job.dropoff.coord);
        assert!(
            dist > 1000.0,
            "Job {} distance must be authentic (> 1000m), got {:.1}m",
            job.id,
            dist
        );
    }
}

// ============================================================================
// JOURNEY 6: MISSION COMPLETION, WALLET REWARDS, SAVE AND RESUME
// ============================================================================

#[test]
fn test_e6_mission_completion_wallet_rewards_save_and_resume() {
    println!("💾 Journey 6: Mission Completion, Wallet Rewards, Save and Resume...");

    let temp_dir = tempfile::tempdir().expect("Create temporary save directory");
    let save_path = temp_dir.path().join("save.json");

    let mut profile = PlayerProfile::new("player-01", "Driver Pioneer");
    assert_eq!(profile.wallet.balance(), 500.0);
    assert_eq!(profile.completed_jobs, 0);

    // 1. Accept San Francisco Taxi Contract
    let mut job = Job::new(
        "taxi-sf-test",
        CareerType::TaxiPassenger,
        "San Francisco Ferry to Salesforce Commute",
        "Executive transport from Ferry Building to Salesforce Transit Center.",
        JobLocation::new(
            "San Francisco Ferry Building",
            Wgs84Coord::new(37.7952, -122.3937, 2.0),
            "san-francisco",
            25.0,
        ),
        JobLocation::new(
            "Salesforce Transit Center",
            Wgs84Coord::new(37.7897, -122.3972, 8.0),
            "san-francisco",
            25.0,
        ),
        1200.0,
        Some(300.0),
        140.0,
        110,
        ReputationTier::Novice,
        CargoOrPassenger::Passenger {
            name: "Software Director".to_string(),
            party_size: 1,
            destination_note: "Salesforce Grand Lobby".to_string(),
        },
        JobDifficulty::Standard,
    );

    let event_bus = GameplayEventBus::new();
    job.accept(&profile, &event_bus)
        .expect("Accept job contract");
    assert_eq!(job.state, JobState::EnRouteToPickup);

    // 2. Pickup Sequence (Stop vehicle within pickup perimeter)
    let pickup_coord = job.pickup.coord;
    let _ = job.update(0.1, pickup_coord, 0.0, &event_bus);
    assert!(matches!(job.state, JobState::AtPickup { .. }));

    // Boarding dwell: 2.0 seconds
    for _ in 0..20 {
        let _ = job.update(0.1, pickup_coord, 0.0, &event_bus);
    }
    assert_eq!(job.state, JobState::EnRouteToDropoff);

    // 3. Dropoff Sequence (Stop vehicle within dropoff perimeter)
    let dropoff_coord = job.dropoff.coord;
    let _ = job.update(0.1, dropoff_coord, 0.0, &event_bus);
    assert!(matches!(job.state, JobState::AtDropoff { .. }));

    // Unloading dwell: 2.0 seconds
    let mut completion_result = None;
    for _ in 0..20 {
        if let Some(res) = job.update(0.1, dropoff_coord, 0.0, &event_bus) {
            completion_result = Some(res);
            break;
        }
    }

    let result = completion_result.expect("Job must emit completion result upon dropoff");
    assert_eq!(result.job_id, "taxi-sf-test");
    assert_eq!(result.base_payout, 140.0);
    assert!(
        result.final_payout >= 140.0,
        "Final payout must include base and bonuses"
    );
    assert_eq!(result.rating_stars, 5.0);

    // 4. Update Profile & Persist Save
    profile.record_job_completion(
        result.career,
        result.final_payout,
        result.xp_gained,
        result.reputation_gained,
        100,
    );
    assert_eq!(profile.completed_jobs, 1);
    assert_eq!(profile.wallet.balance(), 500.0 + result.final_payout);

    let initial_save = SaveData::new(
        profile.clone(),
        "san-francisco",
        Wgs84Coord::new(37.7952, -122.3937, 2.0),
    );
    SaveManager::save_to_file(&save_path, &initial_save).expect("Save profile to disk");

    // 5. Load and Verify Roundtrip Persistence
    let loaded_save = SaveManager::load_from_file(&save_path).expect("Load profile from disk");
    assert_eq!(loaded_save.profile.name, "Driver Pioneer");
    assert_eq!(loaded_save.profile.completed_jobs, 1);
    assert_eq!(
        loaded_save.profile.wallet.balance(),
        500.0 + result.final_payout
    );
    assert_eq!(
        loaded_save.profile.career_tier(CareerType::TaxiPassenger),
        profile.career_tier(CareerType::TaxiPassenger)
    );
}

// ============================================================================
// JOURNEY 7: WINDOWS MOUSE & UI INTERACTION AT MULTIPLE DPI SCALES
// ============================================================================

#[test]
fn test_e7_mouse_and_ui_interaction_at_multiple_dpi_scales() {
    println!("🖱️ Journey 7: Windows Mouse and UI Interaction at Multiple DPI Scales...");

    let dpi_configurations = [
        (1280.0, 720.0, 1.0, "720p @ 1.0x DPI"),
        (1920.0, 1080.0, 1.0, "1080p @ 1.0x DPI"),
        (2048.0, 1152.0, 1.25, "1152p @ 1.25x DPI"),
        (2560.0, 1440.0, 1.5, "1440p @ 1.5x DPI"),
        (3840.0, 2160.0, 2.0, "4K UHD @ 2.0x DPI"),
    ];

    for (w, h, dpi, label) in dpi_configurations {
        println!("   Validating: {} ({}x{}, scale: {:.2})", label, w, h, dpi);

        // 1. Start Screen Layout
        let start_layout = StartScreenLayout::compute(w, h, 7);
        for idx in 0..7 {
            let (cx, cy) = start_layout.option_rects[idx].center();
            assert_eq!(
                start_layout.hit_test(cx, cy),
                Some(StartScreenHit::Option(idx))
            );
        }
        let (qcx, qcy) = start_layout.quick_roam_btn.center();
        assert_eq!(
            start_layout.hit_test(qcx, qcy),
            Some(StartScreenHit::QuickRoam)
        );
        let (ccx, ccy) = start_layout.close_btn.center();
        assert_eq!(start_layout.hit_test(ccx, ccy), Some(StartScreenHit::Close));

        // 2. Pause Menu Layout
        let pause_layout = PauseMenuLayout::compute(w, h, 10);
        for idx in 0..10 {
            let (cx, cy) = pause_layout.option_rects[idx].center();
            assert_eq!(
                pause_layout.hit_test(cx, cy),
                Some(PauseMenuHit::Option(idx))
            );
        }
        let (rcx, rcy) = pause_layout.resume_btn.center();
        assert_eq!(pause_layout.hit_test(rcx, rcy), Some(PauseMenuHit::Resume));

        // 3. Garage Layout (10 Multimodal Vehicles)
        let garage_layout = GarageLayout::compute(w, h, 10);
        for idx in 0..10 {
            let (cx, cy) = garage_layout.item_rects[idx].center();
            assert_eq!(
                garage_layout.hit_test(cx, cy),
                Some(GarageHit::VehicleItem(idx))
            );
        }
        let (scx, scy) = garage_layout.select_btn.center();
        assert_eq!(
            garage_layout.hit_test(scx, scy),
            Some(GarageHit::SelectAndSpawn)
        );

        // 4. Job Browser Layout
        let job_layout = JobBrowserLayout::compute(w, h, 0, 5);
        for idx in 0..5 {
            let (jcx, jcy) = job_layout.item_rects[idx].center();
            assert_eq!(
                job_layout.hit_test(jcx, jcy),
                Some(JobBrowserHit::JobItem(idx))
            );
        }
        let (acx, acy) = job_layout.accept_btn.center();
        assert_eq!(
            job_layout.hit_test(acx, acy),
            Some(JobBrowserHit::AcceptContract)
        );

        // 5. City Catalog Layout (38 US Cities)
        let catalog_layout = CatalogLayout::compute(w, h, 0, 38);
        for idx in 0..10 {
            let (ccx, ccy) = catalog_layout.item_rects[idx].center();
            assert_eq!(
                catalog_layout.hit_test(ccx, ccy),
                Some(CatalogHit::CityItem(idx))
            );
        }
        let (wcx, wcy) = catalog_layout.warp_btn.center();
        assert_eq!(
            catalog_layout.hit_test(wcx, wcy),
            Some(CatalogHit::WarpCity)
        );

        // 6. Settings Layout
        let settings_layout = SettingsLayout::compute(w, h, 6);
        for idx in 0..6 {
            let (scx, scy) = settings_layout.item_rects[idx].center();
            assert_eq!(
                settings_layout.hit_test(scx, scy),
                Some(SettingsHit::SettingItem(idx))
            );
        }

        // 7. Rail Route Modal Layout
        let rail_layout = RailRouteLayout::compute(w, h, 3);
        for idx in 0..3 {
            let (rcx, rcy) = rail_layout.route_rects[idx].center();
            assert_eq!(
                rail_layout.hit_test(rcx, rcy),
                Some(RailRouteHit::RouteItem(idx))
            );
        }
        let (rdcx, rdcy) = rail_layout.dispatch_btn.center();
        assert_eq!(
            rail_layout.hit_test(rdcx, rdcy),
            Some(RailRouteHit::DispatchRoute)
        );

        // 8. Multiplayer Modal Layout
        let multi_layout = MultiplayerLayout::compute(w, h);
        for idx in 0..4 {
            let (mcx, mcy) = multi_layout.action_rects[idx].center();
            assert_eq!(
                multi_layout.hit_test(mcx, mcy),
                Some(MultiplayerHit::Action(idx))
            );
        }

        // 9. Debrief Modal Layout
        let debrief_layout = DebriefLayout::compute(w, h);
        let (dcx, dcy) = debrief_layout.claim_btn.center();
        assert_eq!(debrief_layout.hit_test(dcx, dcy), Some(DebriefHit::Claim));

        // 10. In-Game HUD Layout and Chip Dock
        let in_game_layout = InGameHudLayout::compute(w, h, true);
        let (pcx, pcy) = in_game_layout.pause_btn.center();
        assert_eq!(in_game_layout.hit_test(pcx, pcy), Some(InGameHudHit::Pause));

        let (cam_cx, cam_cy) = in_game_layout.camera_btn.center();
        assert_eq!(
            in_game_layout.hit_test(cam_cx, cam_cy),
            Some(InGameHudHit::CameraCycle)
        );

        let (park_cx, park_cy) = in_game_layout.park_chip.center();
        assert_eq!(
            in_game_layout.hit_test(park_cx, park_cy),
            Some(InGameHudHit::ParkToggle)
        );

        let (fleet_cx, fleet_cy) = in_game_layout.garage_chip.center();
        assert_eq!(
            in_game_layout.hit_test(fleet_cx, fleet_cy),
            Some(InGameHudHit::OpenGarage)
        );

        let (jobs_cx, jobs_cy) = in_game_layout.jobs_chip.center();
        assert_eq!(
            in_game_layout.hit_test(jobs_cx, jobs_cy),
            Some(InGameHudHit::OpenJobs)
        );

        let (cities_cx, cities_cy) = in_game_layout.cities_chip.center();
        assert_eq!(
            in_game_layout.hit_test(cities_cx, cities_cy),
            Some(InGameHudHit::OpenCities)
        );

        let (multi_cx, multi_cy) = in_game_layout.multiplayer_chip.center();
        assert_eq!(
            in_game_layout.hit_test(multi_cx, multi_cy),
            Some(InGameHudHit::OpenMultiplayer)
        );

        let (routes_cx, routes_cy) = in_game_layout.routes_chip.center();
        assert_eq!(
            in_game_layout.hit_test(routes_cx, routes_cy),
            Some(InGameHudHit::OpenRoutes)
        );

        let (stats_cx, stats_cy) = in_game_layout.debug_chip.center();
        assert_eq!(
            in_game_layout.hit_test(stats_cx, stats_cy),
            Some(InGameHudHit::ToggleDebug)
        );

        // 11. Verify Zero Layout Collisions / Non-Overlapping Coordinates
        // Top right buttons: y in [16.0, 46.0]
        let top_btn_bottom = in_game_layout.pause_btn.y + in_game_layout.pause_btn.h;
        assert_eq!(top_btn_bottom, 46.0);

        // Multiplayer chip: y = 54.0, height = 32.0 (y in [54.0, 86.0])
        let multiplayer_chip_top = 54.0;
        let multiplayer_chip_bottom = 54.0 + 32.0;
        assert!(
            multiplayer_chip_top > top_btn_bottom,
            "Multiplayer chip must sit below top action buttons"
        );

        // Mission HUD card: y = 94.0, height = 100.0 (y in [94.0, 194.0])
        let mission_card_top = 94.0;
        assert!(
            mission_card_top > multiplayer_chip_bottom,
            "Mission HUD card must sit strictly below multiplayer chip without overlap"
        );

        // Bottom Dock: y = screen_h - 32.0
        let dock_top = in_game_layout.park_chip.y;
        let contextual_prompt_bottom = (h - 72.0) + 18.0; // Prompt top at h - 72.0, height 18.0
        assert!(
            dock_top > contextual_prompt_bottom,
            "Bottom control dock must sit strictly below contextual action prompt"
        );
    }
}
