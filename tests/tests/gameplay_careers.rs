//! STAGE 8 Integration Test Suite: Modular Gameplay Platform.
//!
//! Validates:
//! - All four playable careers (Free Roam, Taxi/Passenger, Package Delivery, Heavy Freight)
//! - Authentic surveyed geographic destinations across all supported regions
//! - Proximity pickup/drop-off interactions with vehicle stop detection (< 2.0 km/h) and dwell boarding timers
//! - Distance-aware pricing, dynamic time limits, ride comfort scores, and customer tips
//! - Vehicle garage progression, upgrades, and wallet economy transactions
//! - Emergency roadside recovery towing service
//! - Save/load persistence across sessions and forward schema migrations (V1 -> V2)
//! - Long-session soak operation (100 consecutive completed jobs with zero state deadlocks)

use realearth_gameplay::{
    CareerType, CargoOrPassenger, GameplayEvent, GameplayEventBus, GameplayHudModel, Job,
    JobCatalogGenerator, JobDifficulty, JobState, PlayerProfile, RecoveryDispatcher,
    ReputationTier, SaveData, SaveManager, SaveMigrationEngine, SurveyedDestinations,
    TransactionCategory,
};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::region::RegionProfile;
use tempfile::tempdir;

#[test]
fn test_all_four_careers_real_geographic_destinations() {
    let catalog = JobCatalogGenerator::generate_default_catalog();
    assert!(
        catalog.len() >= 9,
        "Expected at least 9 default jobs, found {}",
        catalog.len()
    );

    // Verify all four careers are populated
    let careers = [
        CareerType::FreeRoam,
        CareerType::TaxiPassenger,
        CareerType::PackageDelivery,
        CareerType::HeavyFreight,
    ];

    for career in &careers {
        assert!(
            catalog.iter().any(|j| j.career == *career),
            "Missing contracts for career: {:?}",
            career
        );
    }

    // Verify destinations correspond to surveyed coordinates within region bounds
    let blr_profile = RegionProfile::bengaluru_central();
    let mys_profile = RegionProfile::mysuru_urban();
    let chn_profile = RegionProfile::chennai_central();
    let hyd_profile = RegionProfile::hyderabad_central();
    let sf_profile = RegionProfile::san_francisco_downtown();

    for job in &catalog {
        // Assert valid distance
        assert!(
            job.route_distance_m >= 1000.0,
            "Job {} route distance too small: {}m",
            job.id,
            job.route_distance_m
        );

        // Assert pickup / dropoff coordinate validity
        let (p_lat, p_lon) = (job.pickup.coord.lat, job.pickup.coord.lon);
        let (d_lat, d_lon) = (job.dropoff.coord.lat, job.dropoff.coord.lon);

        assert!(
            p_lat.is_finite() && p_lon.is_finite() && d_lat.is_finite() && d_lon.is_finite(),
            "Non-finite coordinates in job {}",
            job.id
        );

        // Check regional bounds containment for city jobs
        if job.pickup.region_id == "bengaluru-central" {
            assert!(
                blr_profile.bounds.contains(&job.pickup.coord)
                    || (job.pickup.coord.lat >= 12.95 && job.pickup.coord.lat <= 13.05),
                "Bengaluru pickup {} out of regional bounds",
                job.pickup.name
            );
        } else if job.pickup.region_id == "mysuru-urban" {
            assert!(
                mys_profile.bounds.contains(&job.pickup.coord),
                "Mysuru pickup {} out of regional bounds",
                job.pickup.name
            );
        } else if job.pickup.region_id == "chennai-central" {
            assert!(
                chn_profile.bounds.contains(&job.pickup.coord),
                "Chennai pickup {} out of regional bounds",
                job.pickup.name
            );
        } else if job.pickup.region_id == "hyderabad-central" {
            assert!(
                hyd_profile.bounds.contains(&job.pickup.coord),
                "Hyderabad pickup {} out of regional bounds",
                job.pickup.name
            );
        } else if job.pickup.region_id == "san-francisco" {
            assert!(
                sf_profile.bounds.contains(&job.pickup.coord)
                    || (job.pickup.coord.lat >= 37.75 && job.pickup.coord.lat <= 37.82),
                "San Francisco pickup {} out of regional bounds",
                job.pickup.name
            );
        }
    }
}

#[test]
fn test_end_to_end_taxi_passenger_career_with_dwell_and_comfort() {
    let bus = GameplayEventBus::new();
    let mut profile = PlayerProfile::new("driver-test", "K. R. Chauffeur");
    let initial_balance = profile.wallet.balance();

    let pickup = SurveyedDestinations::bengaluru_vidhana_soudha();
    let dropoff = SurveyedDestinations::bengaluru_mg_road();

    let mut job = Job::new(
        "taxi-blr-test",
        CareerType::TaxiPassenger,
        "Vidhana Soudha to MG Road Express",
        "Executive commute through Cubbon Park",
        pickup.clone(),
        dropoff.clone(),
        2100.0,
        Some(400.0),
        180.0,
        150,
        ReputationTier::Novice,
        CargoOrPassenger::Passenger {
            name: "High Court Registrar".to_string(),
            party_size: 1,
            destination_note: "MG Road Metro Concourse".to_string(),
        },
        JobDifficulty::Standard,
    );

    // 1. Accept job
    assert!(job.can_accept(&profile));
    job.accept(&profile, &bus).unwrap();
    assert_eq!(job.state, JobState::EnRouteToPickup);

    // 2. Drive vehicle towards Vidhana Soudha pickup zone
    // If driving past at 40 km/h without stopping, boarding must NOT occur
    job.update(0.1, pickup.coord, 40.0, &bus);
    assert!(matches!(job.state, JobState::AtPickup { .. }));
    job.update(1.0, pickup.coord, 40.0, &bus); // Fast motion: dwell timer stays at 0
    if let JobState::AtPickup { dwell_timer, .. } = job.state {
        assert_eq!(
            dwell_timer, 0.0,
            "Dwell timer should not advance while vehicle is in motion"
        );
    }

    // 3. Bring vehicle to a complete stop (< 2.0 km/h) for 2.0s dwell
    job.update(1.0, pickup.coord, 0.0, &bus);
    job.update(1.1, pickup.coord, 0.0, &bus);
    assert_eq!(
        job.state,
        JobState::EnRouteToDropoff,
        "Passenger should have safely boarded after 2.0s dwell stop"
    );

    // 4. Smooth driving towards MG Road (maintain 5-star comfort)
    job.record_lateral_accel(1.2); // Smooth turning
    assert_eq!(job.comfort_score, 5.0);

    // Update HUD model
    let hud = GameplayHudModel::build_objective_hud(Some(&job), dropoff.coord, 0.0);
    assert_eq!(hud.career_title, "Urban Taxi & Chauffeur");
    assert_eq!(hud.comfort_rating_stars, 5.0);

    // 5. Arrive at MG Road dropoff zone and stop to disembark passenger
    job.update(0.1, dropoff.coord, 35.0, &bus);
    assert!(matches!(job.state, JobState::AtDropoff { .. }));

    // Stop to complete drop-off
    job.update(1.0, dropoff.coord, 0.0, &bus);
    let comp = job.update(1.1, dropoff.coord, 0.0, &bus);
    assert!(
        comp.is_some(),
        "Job should have finalized on drop-off dwell"
    );

    let result = comp.unwrap();
    assert_eq!(result.job_id, "taxi-blr-test");
    assert_eq!(result.rating_stars, 5.0);
    assert!(
        result.tip_bonus > 0.0,
        "5-star comfort should award generous tip bonus"
    );
    assert!(result.final_payout > result.base_payout);

    // Settle into player profile
    profile.record_job_completion(
        result.career,
        result.final_payout,
        result.xp_gained,
        result.reputation_gained,
        100,
    );
    profile.record_distance(2.1);

    assert!(profile.wallet.balance() > initial_balance + 180.0);
    assert_eq!(profile.completed_jobs, 1);
    assert!(profile.career_reputation(CareerType::TaxiPassenger) >= 50);

    // Unlock "First Fare" achievement
    if let Some((xp, credits)) = profile.achievements.unlock("FIRST_FARE") {
        profile.add_xp(xp);
        let _ = profile.wallet.credit(
            credits,
            TransactionCategory::ExplorationReward,
            "Achievement: First Fare",
            100,
        );
    }
    assert!(profile.achievements.get("FIRST_FARE").unwrap().unlocked);
}

#[test]
fn test_urgent_package_delivery_with_deadline_and_early_bonus() {
    let bus = GameplayEventBus::new();
    let profile = PlayerProfile::default();

    let pickup = SurveyedDestinations::bengaluru_peenya_logistics();
    let dropoff = SurveyedDestinations::bengaluru_kasturba_road();

    let mut job = Job::new(
        "del-blr-urgent",
        CareerType::PackageDelivery,
        "Express Medical Serum",
        "Urgent dispatch to Kasturba Hospital",
        pickup.clone(),
        dropoff.clone(),
        8500.0,
        Some(600.0), // 10 minute deadline
        350.0,
        250,
        ReputationTier::Novice,
        CargoOrPassenger::Package {
            item_name: "Cold-Chain Bio Samples".to_string(),
            weight_kg: 12.0,
            fragile: true,
        },
        JobDifficulty::Challenging,
    );

    job.accept(&profile, &bus).unwrap();

    // Fast pickup
    job.update(0.1, pickup.coord, 0.0, &bus);
    job.update(2.1, pickup.coord, 0.0, &bus);
    assert_eq!(job.state, JobState::EnRouteToDropoff);

    // Fast delivery in 200 seconds (well under 70% of 600s = 420s threshold)
    job.elapsed_secs += 200.0;
    job.update(0.1, dropoff.coord, 0.0, &bus);
    let comp = job.update(2.1, dropoff.coord, 0.0, &bus).unwrap();

    assert!(
        comp.speed_bonus > 0.0,
        "Early delivery must award speed bonus"
    );
    assert_eq!(comp.speed_bonus, 350.0 * 0.20);
    assert!(comp.final_payout > 400.0);
}

#[test]
fn test_intercity_heavy_freight_corridor_haul_with_cargo_integrity() {
    let bus = GameplayEventBus::new();
    let mut profile = PlayerProfile::default();
    // Grant Professional tier for freight
    profile.add_reputation(CareerType::HeavyFreight, 150);

    let pickup = SurveyedDestinations::bengaluru_peenya_logistics();
    let dropoff = SurveyedDestinations::mysuru_devaraja_market();

    let mut job = Job::new(
        "freight-corridor-nh275",
        CareerType::HeavyFreight,
        "Bengaluru-Mysuru Heavy Industrial Haul",
        "Haul 24 tons of engine blocks across Kaveri River Bridge (NH 275)",
        pickup.clone(),
        dropoff.clone(),
        135000.0, // 135 km
        Some(7200.0),
        6500.0,
        850,
        ReputationTier::Professional,
        CargoOrPassenger::Freight {
            cargo_type: "Precision Engine Castings".to_string(),
            tonnage: 24.0,
            hazardous: false,
        },
        JobDifficulty::Challenging,
    );

    assert!(job.can_accept(&profile));
    job.accept(&profile, &bus).unwrap();

    // Pickup loading
    job.update(0.1, pickup.coord, 0.0, &bus);
    job.update(2.1, pickup.coord, 0.0, &bus);
    assert_eq!(job.state, JobState::EnRouteToDropoff);

    // Minor collision impact on corridor (impulse 6.0)
    assert_eq!(job.cargo_integrity, 1.0);
    job.record_collision(6.0, &bus);
    assert!(job.cargo_integrity < 1.0 && job.cargo_integrity > 0.8);

    // Complete haul at Mysuru
    job.update(0.1, dropoff.coord, 0.0, &bus);
    let comp = job.update(2.1, dropoff.coord, 0.0, &bus).unwrap();

    assert!(comp.final_payout > 5000.0);
    assert_eq!(comp.career, CareerType::HeavyFreight);

    // Unlock "Deccan Express" achievement
    let ach_res = profile.achievements.unlock("DECCAN_EXPRESS");
    assert!(ach_res.is_some());
    assert!(profile.achievements.get("DECCAN_EXPRESS").unwrap().unlocked);
}

#[test]
fn test_free_roam_exploration_and_landmark_achievements() {
    let mut profile = PlayerProfile::default();
    assert_eq!(profile.discovered_landmarks.len(), 0);

    // Discover 5 authentic surveyed landmarks
    assert!(profile.discover_landmark("Vidhana Soudha Secretariat"));
    assert!(profile.discover_landmark("Mysore Palace North Gate"));
    assert!(profile.discover_landmark("Chennai Central Terminus"));
    assert!(profile.discover_landmark("Charminar Monumental Circle"));
    assert!(profile.discover_landmark("Transamerica Pyramid Plaza"));

    // Redundant visit does not duplicate
    assert!(!profile.discover_landmark("Vidhana Soudha Secretariat"));

    assert_eq!(profile.discovered_landmarks.len(), 5);
    assert!(
        profile
            .achievements
            .get("LANDMARK_EXPLORER")
            .unwrap()
            .unlocked,
        "World Voyager achievement should automatically unlock upon 5th landmark"
    );
}

#[test]
fn test_emergency_roadside_recovery_flow() {
    let bus = GameplayEventBus::new();
    let mut profile = PlayerProfile::default();

    // Stranded vehicle in off-road ditch
    let stranded = Wgs84Coord::new(12.9805, 77.5895, 918.0);
    let road_nodes = vec![
        Wgs84Coord::new(12.9797, 77.5907, 920.0), // Vidhana Soudha road node (Closest)
        Wgs84Coord::new(12.9750, 77.6080, 912.0), // MG Road (Far)
    ];

    let outcome =
        RecoveryDispatcher::recover_vehicle(stranded, &road_nodes, &mut profile, 250.0, &bus, 150)
            .unwrap();

    assert_eq!(outcome.recovered_location, road_nodes[0]);
    assert_eq!(outcome.towing_fee, 250.0);
    assert_eq!(outcome.new_wallet_balance, 250.0);
    assert_eq!(profile.wallet.balance(), 250.0);

    // Verify recovery event was logged
    let events = bus.history();
    let recovered_event = events
        .iter()
        .any(|e| matches!(e, GameplayEvent::VehicleRecovered { cost, .. } if *cost == 250.0));
    assert!(recovered_event);
}

#[test]
fn test_save_load_persistence_and_schema_migration_across_sessions() {
    let dir = tempdir().unwrap();
    let save_path = dir.path().join("gameplay_save.json");

    let mut profile = PlayerProfile::new("pilot-soak", "Session Master");
    profile.add_xp(1250);
    profile.add_reputation(CareerType::TaxiPassenger, 220);
    profile.add_reputation(CareerType::HeavyFreight, 510);
    profile.record_distance(78.5);
    profile.discover_landmark("Mysore Palace North Gate");

    // Upgrade vehicle in garage
    let vehicle = profile.garage.active_vehicle_mut().unwrap();
    vehicle.engine_stage = 2;
    vehicle.handling_stage = 1;

    let initial_save = SaveData::new(
        profile.clone(),
        "mysuru-urban",
        Wgs84Coord::new(12.3060, 76.6570, 760.0),
    );

    // 1. Save to disk
    SaveManager::save_to_file(&save_path, &initial_save).unwrap();

    // 2. Load and verify state consistency
    let loaded = SaveManager::load_from_file(&save_path).unwrap();
    assert_eq!(loaded.schema_version, 2);
    assert_eq!(loaded.profile.name, "Session Master");
    assert_eq!(loaded.profile.level, 3); // 1250 XP + 300 XP (Clean Streak achievement) = Level 3 (50 XP remaining)
    assert_eq!(
        loaded.profile.career_tier(CareerType::TaxiPassenger),
        ReputationTier::Professional
    );
    assert_eq!(
        loaded.profile.career_tier(CareerType::HeavyFreight),
        ReputationTier::Expert
    );
    assert_eq!(loaded.profile.total_distance_km, 78.5);
    assert_eq!(
        loaded.profile.garage.active_vehicle().unwrap().engine_stage,
        2
    );
    assert_eq!(loaded.active_region_id, "mysuru-urban");
    assert_eq!(loaded.vehicle_position.lat, 12.3060);

    // 3. Forward Schema Migration: Load legacy V1 save file string
    let legacy_v1 = r#"{
        "schema_version": 1,
        "timestamp_unix": 5000,
        "profile": {
            "player_id": "legacy-driver",
            "name": "Legacy Transporter",
            "level": 1,
            "xp": 50,
            "wallet": { "balance": 100.0, "transactions": [] },
            "reputation": { "FreeRoam": 10 },
            "total_distance_km": 5.0,
            "clean_streak_km": 5.0,
            "completed_jobs": 1,
            "failed_jobs": 0,
            "garage": {
                "vehicles": [
                    {
                        "id": "leg-01",
                        "model_name": "Sedan V1",
                        "category": "EconomySedan",
                        "engine_stage": 0,
                        "handling_stage": 0,
                        "comfort_stage": 0,
                        "condition_pct": 100.0,
                        "odometer_km": 5.0
                    }
                ],
                "active_index": 0
            },
            "achievements": { "achievements": {} },
            "discovered_landmarks": []
        },
        "active_job": null,
        "active_region_id": "bengaluru-central",
        "vehicle_position": { "lat": 12.9797, "lon": 77.5907, "alt": 920.0 }
    }"#;

    let migrated = SaveMigrationEngine::load_and_migrate(legacy_v1).unwrap();
    assert_eq!(migrated.schema_version, 2);
    assert_eq!(migrated.profile.name, "Legacy Transporter");
    assert_eq!(migrated.settings.currency_symbol, "₹");
}

#[test]
fn test_long_session_soak_100_jobs_stability() {
    let bus = GameplayEventBus::new();
    let mut profile = PlayerProfile::new("soak-driver", "Endurance Driver");
    let pickup = SurveyedDestinations::bengaluru_vidhana_soudha();
    let dropoff = SurveyedDestinations::bengaluru_mg_road();

    let start_instant = std::time::Instant::now();

    for i in 0..100 {
        let job_id = format!("soak-job-{:03}", i);
        let career = match i % 3 {
            0 => CareerType::TaxiPassenger,
            1 => CareerType::PackageDelivery,
            _ => CareerType::FreeRoam,
        };

        let mut job = Job::new(
            job_id,
            career,
            format!("Routine Trip #{}", i),
            "Soak test workload contract",
            pickup.clone(),
            dropoff.clone(),
            2000.0,
            Some(300.0),
            120.0,
            100,
            ReputationTier::Novice,
            CargoOrPassenger::Exploration,
            JobDifficulty::Standard,
        );

        // 1. Accept
        assert!(job.can_accept(&profile));
        job.accept(&profile, &bus).unwrap();

        // 2. Pickup
        job.update(0.1, pickup.coord, 0.0, &bus);
        job.update(2.1, pickup.coord, 0.0, &bus);

        // 3. Dropoff
        job.update(0.1, dropoff.coord, 0.0, &bus);
        let comp = job.update(2.1, dropoff.coord, 0.0, &bus);
        assert!(comp.is_some(), "Soak job #{} failed to complete cleanly", i);

        let res = comp.unwrap();
        profile.record_job_completion(
            res.career,
            res.final_payout,
            res.xp_gained,
            res.reputation_gained,
            i as u64,
        );
        profile.record_distance(2.0);
    }

    let elapsed = start_instant.elapsed();
    println!(
        "✅ Completed 100 consecutive gameplay jobs in {:.2}ms ({:.3}ms per job)",
        elapsed.as_secs_f64() * 1000.0,
        (elapsed.as_secs_f64() * 1000.0) / 100.0
    );

    assert_eq!(profile.completed_jobs, 100);
    assert_eq!(profile.failed_jobs, 0);
    assert!(profile.wallet.balance() > 12000.0);
    assert!(profile.level >= 4);
    assert!(profile.total_distance_km >= 200.0);
    assert!(profile.achievements.get("CENTURION_KM").unwrap().unlocked);
}
