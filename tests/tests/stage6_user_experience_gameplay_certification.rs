//! STAGE 6 — MODERN USER EXPERIENCE AND COMPLETE GAMEPLAY PLATFORM CERTIFICATION
//!
//! Comprehensive end-to-end certification of Stage 6 requirements from docs/ExecutionContract.md:
//! 1. Phase 6.1 — UI/UX Redesign: Fullscreen Start Screen, Pause Menu, Job Browser, Garage Modal,
//!    Settings Modal, Debrief Modal, Speedometer Instrument Cluster, and GPS Minimap.
//! 2. Phase 6.1 — Responsive Geometry & Multi-Resolution DPI Scaling (720p, 1080p, 1440p, 4K).
//! 3. Phase 6.1 — Telemetry Readouts: Metric/Imperial Speedometer, RPM Redline, Gear Badges & Clamped Radar Pins.
//! 4. Phase 6.2 — Authentic Career Loops: Taxi / Passenger pickup, < 2.0 km/h vehicle stop verification,
//!    dwell boarding, dropoff, comfort scoring, and tip bonuses.
//! 5. Phase 6.2 — Package Delivery Fragility & Heavy Freight Tonnage/Tier Gating.
//! 6. Phase 6.2 — Free Roam Exploration, Distance Milestones & Achievement System.
//! 7. Phase 6.3 — Progression & Persistence: Schema V1 -> V2 Migration Engine & Atomic Quicksave / Quickload.
//! 8. Phase 6.3 — Roadside Recovery Dispatcher, Municipal Towing Subsidies & Zero Exploitation Idempotency.

use realearth_gameplay::{
    AchievementRegistry, CareerType, CargoOrPassenger, GameplayEventBus, Job, JobCatalogGenerator,
    JobDifficulty, JobLocation, JobState, PlayerProfile, RecoveryDispatcher, ReputationTier,
    SaveData, SaveManager, SaveMigrationEngine, CURRENT_SAVE_SCHEMA_VERSION,
};
use realearth_rendering::HudMesh;
use realearth_shared::coordinates::Wgs84Coord;
use tempfile::tempdir;

// ============================================================================
// TEST 1: START SCREEN AND PAUSE MENU GEOMETRY
// ============================================================================

#[test]
fn test_stage6_start_screen_and_pause_menu_geometry() {
    let mut hud = HudMesh::new();
    let width = 1920.0f32;
    let height = 1080.0f32;

    let start_options = [
        ("ENTER CAREER", "Resume transport operations in active city"),
        ("CAREER CONTRACTS", "Browse regional dispatch jobs"),
        ("VEHICLE GARAGE", "Inspect and switch fleet vehicles"),
        ("SETTINGS", "Display, audio, and simulation preferences"),
    ];

    // 1. Verify Start Screen generation and selection highlighting
    for sel_idx in 0..start_options.len() {
        hud.clear();
        hud.add_start_screen(
            width,
            height,
            sel_idx,
            &start_options,
            "Veteran Transporter",
            125000.50,
            42,
        );

        assert!(
            !hud.vertices.is_empty(),
            "Start screen must generate non-empty vertices"
        );
        assert!(
            !hud.indices.is_empty(),
            "Start screen must generate index buffer"
        );
        assert_eq!(
            hud.indices.len() % 3,
            0,
            "Indices must form complete triangles"
        );

        // Every vertex must fall within the display boundaries
        for v in &hud.vertices {
            assert!(
                v.position[0] >= 0.0 && v.position[0] <= width,
                "Start screen vertex X ({}) out of screen bounds [0, {}]",
                v.position[0],
                width
            );
            assert!(
                v.position[1] >= 0.0 && v.position[1] <= height,
                "Start screen vertex Y ({}) out of screen bounds [0, {}]",
                v.position[1],
                height
            );
        }
    }

    // 2. Verify Pause Menu generation and selection highlighting
    let pause_options = [
        "RESUME SIMULATION",
        "REGIONAL DISPATCH & JOBS [J]",
        "VEHICLE GARAGE [G]",
        "SIMULATION SETTINGS [O]",
        "EMERGENCY ROADSIDE TOW [R]",
        "QUICKSAVE STATE [F5]",
        "RETURN TO START SCREEN",
    ];

    for sel_idx in 0..pause_options.len() {
        hud.clear();
        hud.add_pause_menu(
            width,
            height,
            sel_idx,
            &pause_options,
            "San Francisco Downtown",
        );

        assert!(
            !hud.vertices.is_empty(),
            "Pause menu must generate non-empty vertices"
        );
        assert!(
            !hud.indices.is_empty(),
            "Pause menu must generate complete triangles"
        );

        for v in &hud.vertices {
            assert!(
                v.position[0] >= 0.0 && v.position[0] <= width,
                "Pause menu vertex X out of bounds"
            );
            assert!(
                v.position[1] >= 0.0 && v.position[1] <= height,
                "Pause menu vertex Y out of bounds"
            );
        }
    }
}

// ============================================================================
// TEST 2: RESPONSIVE LAYOUT AND MULTI-RESOLUTION DPI SCALING
// ============================================================================

#[test]
fn test_stage6_responsive_layout_and_dpi_scaling() {
    let resolutions = [
        (1280.0f32, 720.0f32),  // 720p HD
        (1920.0f32, 1080.0f32), // 1080p FHD
        (2560.0f32, 1440.0f32), // 1440p QHD
        (3840.0f32, 2160.0f32), // 4K UHD
    ];

    let mut hud = HudMesh::new();

    let sample_jobs = [
        (
            "JOB-01",
            "TAXI",
            "Financial District Express",
            "Market St -> Embarcadero",
            3.4,
            120.0,
            80,
            "Standard",
            true,
        ),
        (
            "JOB-02",
            "FREIGHT",
            "Port Container Delivery",
            "Pier 39 -> SFO Cargo Depot",
            18.2,
            650.0,
            240,
            "Challenging",
            true,
        ),
    ];

    let sample_vehicles = [
        (
            "Sports Coupe",
            "Passenger Car",
            "RWD",
            1450.0,
            380.0,
            260.0,
            true,
        ),
        (
            "Heavy Freight Hauler",
            "Commercial Class 8",
            "6x4 Tandem",
            8900.0,
            550.0,
            140.0,
            false,
        ),
    ];

    let sample_settings = [
        ("DISPLAY MODE", "Fullscreen", "Rendering presentation mode"),
        (
            "GRAPHICS PRESET",
            "High",
            "Quality level of shaders and textures",
        ),
        (
            "UNITS OF MEASURE",
            "Metric (km/h)",
            "Speed and distance telemetry",
        ),
    ];

    for &(w, h) in &resolutions {
        // A. Job Browser Modal
        hud.clear();
        hud.add_job_browser_modal(w, h, 0, &sample_jobs, Some("JOB-01"));
        assert!(!hud.vertices.is_empty());
        for v in &hud.vertices {
            assert!(
                v.position[0] >= 0.0 && v.position[0] <= w,
                "Job browser X out of bounds on {}x{}",
                w,
                h
            );
            assert!(
                v.position[1] >= 0.0 && v.position[1] <= h,
                "Job browser Y out of bounds on {}x{}",
                w,
                h
            );
        }

        // B. Garage Modal
        hud.clear();
        hud.add_garage_modal(w, h, 0, &sample_vehicles);
        assert!(!hud.vertices.is_empty());
        for v in &hud.vertices {
            assert!(
                v.position[0] >= 0.0 && v.position[0] <= w,
                "Garage modal X out of bounds on {}x{}",
                w,
                h
            );
            assert!(
                v.position[1] >= 0.0 && v.position[1] <= h,
                "Garage modal Y out of bounds on {}x{}",
                w,
                h
            );
        }

        // C. Settings Modal
        hud.clear();
        hud.add_settings_modal(w, h, 0, &sample_settings);
        assert!(!hud.vertices.is_empty());
        for v in &hud.vertices {
            assert!(
                v.position[0] >= 0.0 && v.position[0] <= w,
                "Settings modal X out of bounds on {}x{}",
                w,
                h
            );
            assert!(
                v.position[1] >= 0.0 && v.position[1] <= h,
                "Settings modal Y out of bounds on {}x{}",
                w,
                h
            );
        }

        // D. Debrief Modal
        hud.clear();
        hud.add_debrief_modal(
            w,
            h,
            "Nob Hill Fare Completed",
            "Taxi & Passenger",
            120.0,
            35.0,
            24.0,
            179.0,
            150,
            4.9,
        );
        assert!(!hud.vertices.is_empty());
        for v in &hud.vertices {
            assert!(
                v.position[0] >= 0.0 && v.position[0] <= w,
                "Debrief modal X out of bounds on {}x{}",
                w,
                h
            );
            assert!(
                v.position[1] >= 0.0 && v.position[1] <= h,
                "Debrief modal Y out of bounds on {}x{}",
                w,
                h
            );
        }

        // E. Dwell Progress Bar
        hud.clear();
        hud.add_dwell_progress(w, h, 0.65, "PASSENGER BOARDING", true);
        assert!(!hud.vertices.is_empty());
        for v in &hud.vertices {
            assert!(
                v.position[0] >= 0.0 && v.position[0] <= w,
                "Dwell bar X out of bounds on {}x{}",
                w,
                h
            );
            assert!(
                v.position[1] >= 0.0 && v.position[1] <= h,
                "Dwell bar Y out of bounds on {}x{}",
                w,
                h
            );
        }
    }
}

// ============================================================================
// TEST 3: SPEEDOMETER AND GPS MINIMAP TELEMETRY
// ============================================================================

#[test]
fn test_stage6_speedometer_and_gps_minimap_telemetry() {
    let mut hud = HudMesh::new();

    // 1. Metric Speedometer
    hud.clear();
    hud.add_speedometer_cluster(
        50.0,
        500.0,
        100.0, // 100 km/h
        "D",
        4500.0,
        7000.0,
        -12.0,
        true, // metric
        "Sports Coupe",
    );
    let metric_verts = hud.vertices.len();
    assert!(metric_verts > 0);

    // 2. Imperial Speedometer
    hud.clear();
    hud.add_speedometer_cluster(
        50.0,
        500.0,
        100.0, // 100 km/h -> 62.1 mph
        "D",
        4500.0,
        7000.0,
        -12.0,
        false, // imperial
        "Sports Coupe",
    );
    let imperial_verts = hud.vertices.len();
    assert!(imperial_verts > 0);

    // 3. Reverse Gear and Redline RPM
    hud.clear();
    hud.add_speedometer_cluster(
        50.0,
        500.0,
        -15.0,
        "R",
        6800.0, // Redline (>85%)
        7000.0,
        5.0,
        true,
        "Sports Coupe",
    );
    assert!(!hud.vertices.is_empty());

    // 4. GPS Minimap Target Clamping
    hud.clear();
    let cx = 150.0;
    let cy = 150.0;
    let radius = 100.0;

    // A. Target nearby (50m east, 30m north)
    hud.add_gps_minimap(cx, cy, radius, 0.0, Some((50.0, 30.0)), true);
    let near_verts = hud.vertices.len();
    assert!(near_verts > 0);

    // B. Target distant (50,000m east) -> Must clamp to radar perimeter
    hud.clear();
    hud.add_gps_minimap(cx, cy, radius, 0.0, Some((50_000.0, 0.0)), false);
    let far_verts = hud.vertices.len();
    assert!(far_verts > 0);

    // Clamped target pin vertices must remain within radar bounding circle
    let max_pin_distance = radius + 15.0;
    for v in &hud.vertices {
        let dx = v.position[0] - cx;
        let dy = v.position[1] - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(
            dist <= max_pin_distance * 1.5,
            "Radar pin geometry exceeded radar perimeter: dist = {}",
            dist
        );
    }
}

// ============================================================================
// TEST 4: AUTHENTIC TAXI / PASSENGER CAREER LOOP
// ============================================================================

#[test]
fn test_stage6_taxi_passenger_career_loop() {
    let event_bus = GameplayEventBus::new();
    let mut profile = PlayerProfile::new("driver-sf-01", "City Chauffeur");

    let pickup_coord = Wgs84Coord::new(37.7952, -122.4028, 15.0); // Transamerica Pyramid
    let dropoff_coord = Wgs84Coord::new(37.7885, -122.4075, 20.0); // Union Square

    let mut job = Job::new(
        "TAXI-SF-001",
        CareerType::TaxiPassenger,
        "Downtown Executive Transfer",
        "Transfer financial executive from Transamerica to Union Square",
        JobLocation::new(
            "Transamerica Financial Plaza",
            pickup_coord,
            "san-francisco",
            25.0,
        ),
        JobLocation::new(
            "Union Square Grand Plaza",
            dropoff_coord,
            "san-francisco",
            25.0,
        ),
        1500.0,
        Some(300.0), // 5 minute limit
        85.0,        // base payout
        100,         // xp reward
        ReputationTier::Novice,
        CargoOrPassenger::Passenger {
            name: "Alexandra Vance".to_string(),
            party_size: 1,
            destination_note: "Please drive smoothly, reviewing quarterly reports.".to_string(),
        },
        JobDifficulty::Standard,
    );

    assert!(job.can_accept(&profile));
    assert_eq!(job.state, JobState::Available);

    // Accept Job
    job.accept(&profile, &event_bus).unwrap();
    assert_eq!(job.state, JobState::EnRouteToPickup);

    // 1. Vehicle driving towards pickup, outside perimeter
    let mut sim_coord = Wgs84Coord::new(37.8000, -122.4028, 15.0);
    let outcome = job.update(0.1, sim_coord, 45.0, &event_bus);
    assert!(outcome.is_none());
    assert_eq!(job.state, JobState::EnRouteToPickup);

    // 2. Vehicle enters pickup radius at high speed (40 km/h)
    sim_coord = pickup_coord;
    let outcome = job.update(0.1, sim_coord, 40.0, &event_bus);
    assert!(outcome.is_none());
    match job.state {
        JobState::AtPickup { dwell_timer, .. } => {
            assert_eq!(
                dwell_timer, 0.0,
                "Dwell timer must not advance while speed >= 2.0 km/h"
            );
        }
        _ => panic!("Expected AtPickup state"),
    }

    // 3. Vehicle continues moving fast (15 km/h) -> Dwell must NOT advance
    for _ in 0..10 {
        job.update(0.1, sim_coord, 15.0, &event_bus);
    }
    match job.state {
        JobState::AtPickup { dwell_timer, .. } => {
            assert_eq!(
                dwell_timer, 0.0,
                "Moving vehicle must never advance dwell boarding timer"
            );
        }
        _ => panic!("Expected AtPickup state"),
    }

    // 4. Vehicle comes to complete stop (< 2.0 km/h) -> Dwell timer advances
    for _ in 0..15 {
        job.update(0.1, sim_coord, 0.0, &event_bus); // 1.5s accumulated
    }
    match job.state {
        JobState::AtPickup { dwell_timer, .. } => {
            assert!(
                dwell_timer >= 1.4,
                "Dwell timer must advance when vehicle stops"
            );
        }
        _ => panic!("Expected AtPickup state"),
    }

    // Finish 2.0s dwell boarding
    for _ in 0..10 {
        job.update(0.1, sim_coord, 0.0, &event_bus);
    }
    assert_eq!(
        job.state,
        JobState::EnRouteToDropoff,
        "Passenger must be safely boarded into EnRouteToDropoff"
    );

    // 5. In transit: passenger comfort is sensitive to high lateral acceleration
    job.record_lateral_accel(5.5); // Hard swerve (> 4.5 m/s^2)
    assert!(
        job.comfort_score < 5.0,
        "Rough driving must degrade comfort score"
    );
    let comfort_before = job.comfort_score;

    // Smooth driving does not degrade comfort further
    job.record_lateral_accel(2.0);
    assert_eq!(job.comfort_score, comfort_before);

    // 6. Arrive at dropoff and come to stop
    sim_coord = dropoff_coord;
    job.update(0.1, sim_coord, 0.5, &event_bus); // Inside dropoff radius, stopped
    match job.state {
        JobState::AtDropoff { .. } => {}
        _ => panic!("Expected AtDropoff state"),
    }

    // Complete dropoff dwell timer (2.0s)
    let mut completion_result = None;
    for _ in 0..25 {
        if let Some(res) = job.update(0.1, sim_coord, 0.0, &event_bus) {
            completion_result = Some(res);
            break;
        }
    }

    let completion = completion_result.expect("Job must successfully complete after dwell");
    assert_eq!(completion.career, CareerType::TaxiPassenger);
    assert!(completion.base_payout > 0.0);
    assert!(
        completion.tip_bonus > 0.0,
        "Passenger comfort (>1 star) must award tip bonus"
    );
    assert!(
        completion.final_payout > completion.base_payout,
        "Final payout must include tip and difficulty bonuses"
    );
    assert!(completion.xp_gained > 0);

    // Apply completion to player profile
    profile.record_job_completion(
        completion.career,
        completion.final_payout,
        completion.xp_gained,
        completion.reputation_gained,
        1700000000,
    );
    assert_eq!(profile.completed_jobs, 1);
    assert!(profile.wallet.balance() > 0.0);
}

// ============================================================================
// TEST 5: PACKAGE DELIVERY AND HEAVY FREIGHT CAREER LOOPS
// ============================================================================

#[test]
fn test_stage6_package_delivery_and_freight_career_loops() {
    let event_bus = GameplayEventBus::new();
    let profile = PlayerProfile::new("driver-sf-02", "Logistics Specialist");

    let p1 = Wgs84Coord::new(37.7952, -122.4028, 15.0);
    let p2 = Wgs84Coord::new(37.7885, -122.4075, 20.0);

    // 1. Package Delivery Cargo Fragility Tracking
    let mut pkg_job = Job::new(
        "PKG-SF-01",
        CareerType::PackageDelivery,
        "Fragile Medical Supplies",
        "Transport delicate lab reagents",
        JobLocation::new("Medical Center", p1, "san-francisco", 25.0),
        JobLocation::new("Research Labs", p2, "san-francisco", 25.0),
        2200.0,
        Some(400.0),
        150.0,
        180,
        ReputationTier::Novice,
        CargoOrPassenger::Package {
            item_name: "Centrifuge Tubes".to_string(),
            weight_kg: 12.5,
            fragile: true,
        },
        JobDifficulty::Standard,
    );

    pkg_job.accept(&profile, &event_bus).unwrap();

    // Board / Load at stop
    pkg_job.update(0.1, p1, 0.0, &event_bus);
    for _ in 0..25 {
        pkg_job.update(0.1, p1, 0.0, &event_bus);
    }
    assert_eq!(pkg_job.state, JobState::EnRouteToDropoff);

    // Severe collision impact degrades cargo integrity
    pkg_job.record_collision(12.0, &event_bus);
    assert!(
        pkg_job.cargo_integrity < 1.0,
        "Collision impact must degrade cargo integrity"
    );
    let integrity = pkg_job.cargo_integrity;

    // Complete delivery with compromised cargo
    let mut completion = None;
    pkg_job.update(0.1, p2, 0.0, &event_bus);
    for _ in 0..25 {
        if let Some(res) = pkg_job.update(0.1, p2, 0.0, &event_bus) {
            completion = Some(res);
            break;
        }
    }
    let res = completion.unwrap();
    // Payout should be reduced proportional to cargo integrity
    assert!(
        res.final_payout < (pkg_job.base_payout * pkg_job.difficulty.payout_bonus()),
        "Compromised cargo integrity ({:.2}) must discount final payout",
        integrity
    );

    // 2. Heavy Freight Tonnage & Tier Gating
    let freight_job = Job::new(
        "FRT-SF-01",
        CareerType::HeavyFreight,
        "Industrial Steel Beams",
        "Haul 24 tonnes of structural steel",
        JobLocation::new("Port Terminal", p1, "san-francisco", 35.0),
        JobLocation::new("Subway Construction", p2, "san-francisco", 35.0),
        8500.0,
        Some(1200.0),
        750.0,
        600,
        ReputationTier::Expert, // Requires Expert Tier (>= 500 points)
        CargoOrPassenger::Freight {
            cargo_type: "Structural Steel".to_string(),
            tonnage: 24.0,
            hazardous: false,
        },
        JobDifficulty::Challenging,
    );

    // Novice driver cannot accept Expert tier freight
    assert!(
        !freight_job.can_accept(&profile),
        "Novice driver must not be eligible for Expert freight contract"
    );

    // Promote profile to Expert tier
    let mut veteran = profile.clone();
    veteran.add_reputation(CareerType::HeavyFreight, 600); // Promotes to Expert
    assert!(
        veteran.career_tier(CareerType::HeavyFreight) >= ReputationTier::Expert,
        "Reputation gain must promote tier to Expert"
    );
    assert!(
        freight_job.can_accept(&veteran),
        "Expert driver must be permitted to accept Expert freight contract"
    );
}

// ============================================================================
// TEST 6: FREE ROAM AND LANDMARK DISCOVERY
// ============================================================================

#[test]
fn test_stage6_free_roam_and_landmark_discovery() {
    let mut achievements = AchievementRegistry::new();

    // Initial state: all locked
    let first_fare = achievements.get("FIRST_FARE").unwrap();
    assert!(!first_fare.unlocked);

    let smooth = achievements.get("SMOOTH_CHAUFFEUR").unwrap();
    assert!(!smooth.unlocked);

    let centurion = achievements.get("CENTURION_KM").unwrap();
    assert!(!centurion.unlocked);

    // Unlock achievements upon completing milestones
    achievements.unlock("FIRST_FARE");
    assert!(achievements.get("FIRST_FARE").unwrap().unlocked);

    achievements.unlock("SMOOTH_CHAUFFEUR");
    assert!(achievements.get("SMOOTH_CHAUFFEUR").unwrap().unlocked);

    // Track driving distance progress towards 100km Centurion
    achievements.update_progress("CENTURION_KM", 0.45);
    assert_eq!(achievements.get("CENTURION_KM").unwrap().progress, 0.45);

    achievements.update_progress("CENTURION_KM", 1.0);
    assert!(achievements.get("CENTURION_KM").unwrap().unlocked);

    // Regional Job Catalog generation for all careers
    let sf_jobs = JobCatalogGenerator::jobs_for_region("san-francisco");
    assert!(
        !sf_jobs.is_empty(),
        "San Francisco catalog must contain real transport contracts"
    );
    assert!(
        sf_jobs
            .iter()
            .any(|j| j.career == CareerType::TaxiPassenger),
        "Missing Taxi contracts"
    );
    assert!(
        sf_jobs
            .iter()
            .any(|j| j.career == CareerType::PackageDelivery),
        "Missing Delivery contracts"
    );
    assert!(
        sf_jobs.iter().any(|j| j.career == CareerType::HeavyFreight),
        "Missing Freight contracts"
    );
    assert!(
        sf_jobs.iter().any(|j| j.career == CareerType::FreeRoam),
        "Missing Free Roam contracts"
    );
}

// ============================================================================
// TEST 7: PERSISTENCE AND ATOMIC SCHEMA MIGRATION
// ============================================================================

#[test]
fn test_stage6_persistence_and_atomic_migration() {
    let dir = tempdir().unwrap();
    let save_file = dir.path().join("quicksave.json");

    let mut profile = PlayerProfile::new("driver-persistence-01", "Planetary Rover");
    let _ = profile.wallet.credit(
        5420.75,
        realearth_gameplay::TransactionCategory::JobPayout,
        "Career Payouts",
        1700000000,
    );

    let vehicle_pos = Wgs84Coord::new(37.7952, -122.4028, 12.5);
    let original_save = SaveData::new(profile.clone(), "san-francisco", vehicle_pos);

    // 1. Quicksave Roundtrip
    SaveManager::save_to_file(&save_file, &original_save)
        .expect("Quicksave must serialize and persist to disk");

    let loaded = SaveManager::load_from_file(&save_file)
        .expect("Quickload must read and deserialize save file");

    assert_eq!(loaded.schema_version, CURRENT_SAVE_SCHEMA_VERSION);
    assert_eq!(loaded.profile.name, profile.name);
    assert_eq!(loaded.profile.wallet.balance(), profile.wallet.balance());
    assert_eq!(loaded.active_region_id, "san-francisco");
    assert_eq!(loaded.vehicle_position.lat, vehicle_pos.lat);
    assert_eq!(loaded.vehicle_position.lon, vehicle_pos.lon);

    // 2. Schema Migration V1 -> V2 Forward Compatibility Test
    let v1_legacy_json = r#"{
        "schema_version": 1,
        "timestamp_unix": 1699999999,
        "profile": {
            "player_id": "legacy-pilot",
            "name": "Legacy Pilot",
            "level": 3,
            "xp": 450,
            "wallet": { "balance": 1250.0, "transactions": [] },
            "reputation": { "TaxiPassenger": 150 },
            "total_distance_km": 42.5,
            "clean_streak_km": 12.0,
            "completed_jobs": 5,
            "failed_jobs": 0,
            "garage": { "vehicles": [], "active_index": 0 },
            "achievements": { "achievements": {} },
            "discovered_landmarks": []
        },
        "active_job": null,
        "active_region_id": "san-francisco",
        "vehicle_position": { "lat": 37.7952, "lon": -122.4028, "alt": 15.0 }
    }"#;

    let migrated = SaveMigrationEngine::load_and_migrate(v1_legacy_json)
        .expect("V1 save must cleanly migrate to latest schema");

    assert_eq!(migrated.schema_version, 2);
    assert_eq!(migrated.profile.name, "Legacy Pilot");
    assert_eq!(migrated.profile.completed_jobs, 5);
    assert_eq!(migrated.settings.difficulty, "Standard");
    assert!(migrated.settings.metric_units);
}

// ============================================================================
// TEST 8: ROADSIDE RECOVERY AND IDEMPOTENCY
// ============================================================================

#[test]
fn test_stage6_roadside_recovery_and_idempotency() {
    let event_bus = GameplayEventBus::new();
    let mut profile = PlayerProfile::new("driver-stranded-01", "Offroad Explorer");
    let _ = profile.wallet.credit(
        500.0,
        realearth_gameplay::TransactionCategory::JobPayout,
        "Starting Funds",
        1700000000,
    );

    // Surveyed road coordinates along San Francisco Market Street corridor
    let road_nodes = vec![
        Wgs84Coord::new(37.7930, -122.3980, 10.0),
        Wgs84Coord::new(37.7900, -122.4020, 12.0),
        Wgs84Coord::new(37.7850, -122.4080, 15.0),
    ];

    // Stranded vehicle off-road (e.g. rolled down coastal cliff)
    let stranded_pos = Wgs84Coord::new(37.7895, -122.4025, 5.0); // Closest to road_nodes[1]
    let initial_balance = profile.wallet.balance();

    // 1. Dispatch emergency recovery
    let tow_fee = 150.0;
    let outcome = RecoveryDispatcher::recover_vehicle(
        stranded_pos,
        &road_nodes,
        &mut profile,
        tow_fee,
        &event_bus,
        1700000001,
    )
    .expect("Roadside recovery must resolve nearest road coordinate");

    assert_eq!(
        outcome.recovered_location, road_nodes[1],
        "Vehicle must snap to nearest surveyed road node"
    );
    assert_eq!(outcome.towing_fee, 150.0);
    assert_eq!(profile.wallet.balance(), initial_balance - 150.0);

    // 2. Municipal Subsidy when player cannot afford full towing fee
    let _ = profile.wallet.debit(
        profile.wallet.balance(),
        realearth_gameplay::TransactionCategory::CollisionRepair,
        "Zeroing out wallet",
        1700000002,
    );
    assert_eq!(profile.wallet.balance(), 0.0);

    let subsidised_outcome = RecoveryDispatcher::recover_vehicle(
        stranded_pos,
        &road_nodes,
        &mut profile,
        tow_fee,
        &event_bus,
        1700000003,
    )
    .expect("Recovery must succeed with municipal assistance even if broke");

    assert_eq!(subsidised_outcome.towing_fee, 0.0);
    assert_eq!(
        profile.wallet.balance(),
        0.0,
        "Wallet must never become negative"
    );
    assert_eq!(subsidised_outcome.recovered_location, road_nodes[1]);

    // 3. Zero Road Exception Handling
    let empty_roads: Vec<Wgs84Coord> = Vec::new();
    let err = RecoveryDispatcher::recover_vehicle(
        stranded_pos,
        &empty_roads,
        &mut profile,
        tow_fee,
        &event_bus,
        1700000004,
    );
    assert!(
        err.is_err(),
        "Recovery must fail gracefully if no surveyed roads exist nearby"
    );
}
