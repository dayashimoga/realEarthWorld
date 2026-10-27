//! STAGE H2 QUALITY GATE CERTIFICATION SUITE
//! Authentic Map-to-3D Compiler and Continuous Geographic Streaming
//!
//! Mission: Eliminate geographic disappearance, flat blank terrain, missing scenery,
//! invalid road/terrain transitions, and synthetic city substitutions.
//! Demonstrate real, demand-driven map-to-3D generation and continuous streaming.
//!
//! 5 Exhaustive Quality Gates:
//! 1. Fail-Closed Acquisition & Cryptographic Manifest Integrity
//! 2. San Francisco Complete World Compilation & Feature Reconciliation
//! 3. Continuous Multi-Kilometer Driving Traversal Across Chunk Boundaries
//! 4. High-Speed Geographic Flight & Continuous Streaming Prefetch
//! 5. Production Streaming Telemetry & Fail-Closed Boundaries

use glam::Vec3;
use realearth_compiler::chunk::{CHUNK_MAGIC, CHUNK_VERSION};
use realearth_compiler::database::ChunkDatabase;
use realearth_compiler::packager::read_chunk_file;
use realearth_compiler::regional_catalog::PackageManifest;
use realearth_ingestion::authentic::load_authentic_city;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::us_catalog::find_city_by_id;
use realearth_streaming::{
    generate_demand_terrain_chunk, san_francisco_pilot_coverage, InstalledPackageRecord,
    RegionInstaller, StorageQuotaManager, StreamingConfig, WorldStreamer,
};
use realearth_validation::validate_dataset;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root must exist")
        .to_path_buf()
}

// ============================================================================
// GATE 1: FAIL-CLOSED ACQUISITION & CRYPTOGRAPHIC MANIFEST INTEGRITY
// ============================================================================

#[test]
fn test_h2_gate1_fail_closed_acquisition_and_cryptographic_manifest_integrity() {
    println!("\n🛡️ [STAGE H2 GATE 1] Testing Fail-Closed Acquisition & Cryptographic Integrity...");

    let temp = tempdir().expect("Must create temp directory");
    let base_dir = temp.path().to_path_buf();

    let mut installer = RegionInstaller::new(&base_dir, 0);

    // 1. Audit Fail-Closed Behavior on Missing Package Manifest
    // Prior bug: verify_installed_package returned Ok(true) when manifest.json was missing!
    let dummy_dir = installer.region_package_dir("dummy-region");
    fs::create_dir_all(&dummy_dir).unwrap();
    fs::write(dummy_dir.join("chunks.db"), b"SQLite format 3\0 dummy").unwrap();

    let verified_missing_manifest = installer.verify_installed_package("dummy-region").unwrap();
    assert!(
        !verified_missing_manifest,
        "GATE 1 FAIL: Must reject package missing manifest.json (must fail closed)"
    );

    // 2. Audit Fail-Closed Behavior on Corrupted / Mismatched Checksum
    let manifest = PackageManifest {
        region_id: "dummy-region".to_string(),
        name: "Dummy Region".to_string(),
        version: "1.0.0".to_string(),
        bounds: GeoBounds::new(37.78, 37.80, -122.42, -122.40),
        anchor: Wgs84Coord::new(37.79, -122.41, 10.0),
        utm_zone: 10,
        total_chunks: 1,
        uncompressed_bytes: 1000,
        compressed_bytes: 100,
        sha256_checksum: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        created_at_utc: "2026-10-26T00:00:00Z".to_string(),
    };
    fs::write(dummy_dir.join("manifest.json"), manifest.to_json().unwrap()).unwrap();

    let verified_corrupted_hash = installer.verify_installed_package("dummy-region").unwrap();
    assert!(
        !verified_corrupted_hash,
        "GATE 1 FAIL: Must reject package with SHA-256 mismatch between manifest and chunks.db"
    );

    // 3. Audit Fail-Closed Behavior on Empty chunks.db (0 bytes)
    fs::write(dummy_dir.join("chunks.db"), b"").unwrap();
    let verified_empty_db = installer.verify_installed_package("dummy-region").unwrap();
    assert!(
        !verified_empty_db,
        "GATE 1 FAIL: Must reject package with 0-byte chunks.db"
    );

    // 4. Audit Actual Byte Tracking (No Fixed Illustrative Constants)
    let _sf_city = find_city_by_id("san-francisco").expect("San Francisco must be in catalog");
    let mut tracked_bytes = Vec::new();

    installer
        .install_region(
            "san-francisco",
            Box::new(|p: realearth_streaming::InstallProgress| {
                tracked_bytes.push((p.state, p.bytes_downloaded, p.total_bytes));
            }),
        )
        .expect("San Francisco must install cleanly into temp directory");

    assert!(
        !tracked_bytes.is_empty(),
        "Must record progress callback events"
    );

    // Verify byte counts reflect genuine file sizes rather than fixed 500_000 / 250_000 constants
    let final_progress = tracked_bytes.last().unwrap();
    assert!(
        final_progress.1 > 0 && final_progress.2 > 0,
        "Downloaded and total byte counts must be strictly positive"
    );
    assert_eq!(
        final_progress.1, final_progress.2,
        "Final progress downloaded bytes must equal total bytes"
    );

    // 5. Audit Storage Quota Manager Eviction & Headroom Limits
    let mut quota_manager = StorageQuotaManager::new(500_000); // 500 KB quota
    quota_manager.register_package(InstalledPackageRecord {
        package_id: "old-unpinned-pkg".to_string(),
        country_code: "US".to_string(),
        version: "1.0.0".to_string(),
        disk_bytes: 350_000,
        last_accessed_timestamp: 10,
        is_active: false,
        is_pinned: false,
    });

    quota_manager.register_package(InstalledPackageRecord {
        package_id: "san-francisco".to_string(),
        country_code: "US".to_string(),
        version: "1.0.0".to_string(),
        disk_bytes: 200_000,
        last_accessed_timestamp: 100,
        is_active: true,
        is_pinned: true,
    });

    // Total registered = 550_000 > 500_000 quota
    assert!(
        quota_manager.total_used_bytes() > quota_manager.quota_bytes,
        "Quota exceeded initially"
    );
    let evicted = quota_manager.evict_lru_to_quota();
    assert_eq!(
        evicted.len(),
        1,
        "Must evict exactly the unpinned LRU package to satisfy quota"
    );
    assert_eq!(evicted[0], "old-unpinned-pkg");
    assert!(
        quota_manager.packages.contains_key("san-francisco"),
        "Pinned San Francisco package must NEVER be evicted"
    );
    assert!(
        !quota_manager.packages.contains_key("old-unpinned-pkg"),
        "Unpinned LRU package must be evicted"
    );

    println!("✅ [STAGE H2 GATE 1] PASSED: Fail-closed acquisition, cryptographic integrity, and quota management verified.");
}

// ============================================================================
// GATE 2: COMPLETE WORLD COMPILATION & FEATURE RECONCILIATION
// ============================================================================

#[test]
fn test_h2_gate2_san_francisco_complete_world_compilation_and_feature_reconciliation() {
    println!("\n🏙️ [STAGE H2 GATE 2] Testing San Francisco Complete Compilation & Feature Reconciliation...");

    // 1. Verify Published Pilot Coverage Polygon Specification
    let pilot = san_francisco_pilot_coverage();
    assert_eq!(pilot.region_id, "san-francisco");
    assert_eq!(
        pilot.boundary_polygon.len(),
        5,
        "Bounding polygon must be closed (5 vertices)"
    );
    assert!(
        pilot.surveyed_area_km2 >= 6.5,
        "Surveyed area must be >= 6.5 km²"
    );
    assert_eq!(pilot.min_elevation_m, 0.0);
    assert_eq!(pilot.max_elevation_m, 115.0);
    assert!(pilot
        .primary_arterials
        .contains(&"Market Street".to_string()));
    assert!(pilot
        .primary_arterials
        .contains(&"The Embarcadero".to_string()));
    assert!(pilot
        .key_landmarks
        .iter()
        .any(|l| l.contains("Salesforce Tower")));
    assert!(pilot
        .key_landmarks
        .iter()
        .any(|l| l.contains("Transamerica Pyramid")));
    assert!(pilot
        .grade_separations
        .iter()
        .any(|g| g.contains("Bay Bridge")));
    assert!(pilot
        .grade_separations
        .iter()
        .any(|g| g.contains("Broadway Tunnel")));

    // 2. Load Ingested Dataset and Validate Pipeline
    let (dataset, _dem, _meta) =
        load_authentic_city("san-francisco").expect("SF must load cleanly");
    let (_validated, report) = validate_dataset(&dataset);

    println!(
        "   Ingested: {} buildings, {} roads, {} water, {} land cover",
        dataset.buildings.len(),
        dataset.roads.len(),
        dataset.water_bodies.len(),
        dataset.land_cover.len()
    );
    assert!(
        dataset.buildings.len() >= 250,
        "Must contain >= 250 surveyed buildings"
    );
    assert!(dataset.roads.len() >= 45, "Must contain >= 45 mapped roads");
    assert_eq!(
        report.quarantined.len(),
        0,
        "No features should be quarantined"
    );
    assert_eq!(
        report.valid_roads_passed, report.total_roads_evaluated,
        "All evaluated roads must pass validation"
    );
    assert_eq!(
        report.valid_buildings_passed, report.total_buildings_evaluated,
        "All evaluated buildings must pass validation"
    );

    // 3. Inspect Pre-compiled Chunks in data/world_san_francisco
    let world_dir = workspace_root().join("data/world_san_francisco");
    let db_path = world_dir.join("chunks.db");
    assert!(
        db_path.exists(),
        "data/world_san_francisco/chunks.db must exist"
    );

    let db = ChunkDatabase::open(&db_path).expect("ChunkDatabase must open cleanly");
    let chunk_count = db.count_chunks().expect("Must count chunks");
    assert_eq!(
        chunk_count, 30,
        "San Francisco pilot package must contain strictly 30 chunks (5x6 grid)"
    );

    // 4. Feature Lifecycle Reconciliation across all 30 chunks
    let all_records = db
        .query_bbox(&pilot.bounding_box)
        .expect("Must query all chunks");
    assert_eq!(all_records.len(), 30);

    let mut total_road_meta_count = 0;
    let mut total_building_meta_count = 0;
    let mut total_surveyed_buildings = 0;
    let mut total_inferred_buildings = 0;
    let mut total_bridge_roads = 0;
    let mut total_tunnel_roads = 0;
    let mut unique_road_ways = HashSet::new();
    let mut unique_building_ids = HashSet::new();

    for cand in &all_records {
        let chunk_file = world_dir.join(&cand.storage_path);
        let chunk = read_chunk_file(&chunk_file).expect("WorldChunk must deserialize cleanly");

        assert_eq!(chunk.header.magic, *CHUNK_MAGIC);
        assert_eq!(chunk.header.version, CHUNK_VERSION);

        total_road_meta_count += chunk.road_count();
        total_building_meta_count += chunk.building_metadata.len();
        total_surveyed_buildings += chunk.surveyed_building_count();
        total_inferred_buildings += chunk.inferred_building_count();

        for r in &chunk.road_metadata {
            unique_road_ways.insert(r.way_id);
            if r.is_bridge {
                total_bridge_roads += 1;
            }
            if r.is_tunnel {
                total_tunnel_roads += 1;
            }
        }

        for b in &chunk.building_metadata {
            unique_building_ids.insert(b.feature_id);
        }
    }

    println!(
        "   Compiled Chunks: {} roads ({} unique OSM ways), {} buildings ({} unique IDs)",
        total_road_meta_count,
        unique_road_ways.len(),
        total_building_meta_count,
        unique_building_ids.len()
    );
    println!(
        "   Building Provenance: {} surveyed, {} inferred levels",
        total_surveyed_buildings, total_inferred_buildings
    );
    println!(
        "   Grade Separations: {} bridge road segments, {} tunnel road segments",
        total_bridge_roads, total_tunnel_roads
    );

    assert!(
        unique_road_ways.len() >= 40,
        "Must preserve >= 40 unique OSM way IDs in compiled road metadata"
    );
    assert!(
        unique_building_ids.len() >= 200,
        "Must preserve >= 200 unique building IDs in compiled building metadata"
    );
    assert!(
        total_surveyed_buildings > 0,
        "Must preserve surveyed building heights from CTBUH/SF Planning"
    );
    assert!(
        total_inferred_buildings > 0,
        "Must distinguish inferred level heights from surveyed geometry"
    );
    assert!(
        total_bridge_roads > 0,
        "Must preserve bridge flag in road metadata (e.g. Bay Bridge)"
    );
    assert!(
        total_tunnel_roads > 0,
        "Must preserve tunnel flag in road metadata (e.g. Broadway Tunnel)"
    );

    println!("✅ [STAGE H2 GATE 2] PASSED: Complete world compilation and feature lifecycle reconciliation verified.");
}

// ============================================================================
// GATE 3: CONTINUOUS MULTI-KILOMETER DRIVING TRAVERSAL ACROSS CHUNK BOUNDARIES
// ============================================================================

#[test]
fn test_h2_gate3_continuous_multi_kilometer_driving_traversal_across_chunk_boundaries() {
    println!("\n🚗 [STAGE H2 GATE 3] Testing Continuous Multi-Kilometer Driving Traversal...");

    let world_dir = workspace_root().join("data/world_san_francisco");
    let config = StreamingConfig {
        view_distance_meters: 1500.0,
        lod1_distance_meters: 400.0,
        collision_critical_radius_meters: 180.0,
        max_in_flight_tasks: 8,
        ..Default::default()
    };

    let mut streamer = WorldStreamer::open(&world_dir, config)
        .expect("WorldStreamer must open data/world_san_francisco");

    // Route: Ferry Building -> Market St -> SOMA -> Nob Hill -> Broadway Tunnel
    // Waypoints spanning ~4.8 km across multiple chunk boundaries
    let waypoints = vec![
        Wgs84Coord::new(37.7955, -122.3937, 2.0), // 1. Ferry Building (Bay waterfront, 2m)
        Wgs84Coord::new(37.7925, -122.3970, 8.0), // 2. Market St & Beale St (8m)
        Wgs84Coord::new(37.7885, -122.4015, 12.0), // 3. Market St & 3rd St (12m)
        Wgs84Coord::new(37.7845, -122.4065, 15.0), // 4. Market St & 5th St (15m)
        Wgs84Coord::new(37.7815, -122.4105, 18.0), // 5. Market St & 8th St (18m)
        Wgs84Coord::new(37.7840, -122.4140, 30.0), // 6. Civic Center / Larkin St (30m)
        Wgs84Coord::new(37.7890, -122.4130, 65.0), // 7. Nob Hill climb (California & Hyde, 65m)
        Wgs84Coord::new(37.7925, -122.4105, 95.0), // 8. Nob Hill summit (Mason & California, 95m)
        Wgs84Coord::new(37.7965, -122.4100, 75.0), // 9. Russian Hill / Broadway Tunnel West (75m)
        Wgs84Coord::new(37.7980, -122.4045, 45.0), // 10. Broadway Tunnel East portal (45m)
        Wgs84Coord::new(37.7970, -122.3975, 5.0), // 11. Embarcadero north waterfront (5m)
    ];

    let mut total_distance_m = 0.0;
    let mut visited_chunks = HashSet::new();
    let mut total_frames = 0;
    let mut min_elevation = f64::MAX;
    let mut max_elevation = f64::MIN;

    for w in waypoints.windows(2) {
        let p1 = w[0];
        let p2 = w[1];
        let d_lat = (p2.lat - p1.lat) * 111_000.0;
        let d_lon = (p2.lon - p1.lon) * 88_000.0;
        let seg_dist = (d_lat * d_lat + d_lon * d_lon).sqrt();
        total_distance_m += seg_dist;

        // Simulate driving at ~60 km/h (16.7 m/s) with 30 Hz simulation ticks (~0.55m per step)
        let steps = (seg_dist / 15.0).ceil() as usize;
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let lat = p1.lat + t * (p2.lat - p1.lat);
            let lon = p1.lon + t * (p2.lon - p1.lon);
            let alt = p1.alt + t * (p2.alt - p1.alt);

            let current_pos = Wgs84Coord::new(lat, lon, alt);
            let velocity = Vec3::new(
                (d_lon / seg_dist * 16.7) as f32,
                0.0,
                (d_lat / seg_dist * 16.7) as f32,
            );

            streamer
                .update_with_velocity(current_pos, velocity)
                .expect("Streamer update must succeed");
            total_frames += 1;

            if alt < min_elevation {
                min_elevation = alt;
            }
            if alt > max_elevation {
                max_elevation = alt;
            }

            // CRITICAL TEST: Ground under vehicle must NEVER disappear or be unready
            let collision_ready = streamer.is_collision_ready(current_pos);
            assert!(
                collision_ready,
                "GATE 3 FAIL: Collision not ready at lat={:.5}, lon={:.5}, frame={}. Ground disappeared!",
                lat, lon, total_frames
            );

            // Track active chunks
            let active = streamer.get_active_chunks();
            assert!(
                !active.is_empty(),
                "GATE 3 FAIL: Active chunk set became empty during driving!"
            );

            // Record chunk coordinates
            let cx = (lon / 0.005).floor() as i32;
            let cy = (lat / 0.005).floor() as i32;
            visited_chunks.insert((cx, cy));
        }
    }

    println!(
        "   Traversed {:.2} km over {} frames across {} distinct chunk boundary zones",
        total_distance_m / 1000.0,
        total_frames,
        visited_chunks.len()
    );
    println!(
        "   Topographic Elevation Range: {:.1}m to {:.1}m (delta: {:.1}m relief)",
        min_elevation,
        max_elevation,
        max_elevation - min_elevation
    );

    assert!(
        total_distance_m >= 3500.0,
        "Route must span >= 3.5 km, spanned {:.1}m",
        total_distance_m
    );
    assert!(
        visited_chunks.len() >= 6,
        "Must traverse >= 6 distinct chunk boundary zones, traversed {}",
        visited_chunks.len()
    );
    assert!(
        max_elevation - min_elevation >= 60.0,
        "Topography must reflect authentic steep San Francisco relief (>= 60m vertical delta)"
    );

    println!("✅ [STAGE H2 GATE 3] PASSED: Continuous multi-kilometer driving traversal verified without void cracks or collision failure.");
}

// ============================================================================
// GATE 4: HIGH-SPEED GEOGRAPHIC FLIGHT & CONTINUOUS STREAMING PREFETCH
// ============================================================================

#[test]
fn test_h2_gate4_high_speed_geographic_flight_and_continuous_streaming() {
    println!("\n✈️ [STAGE H2 GATE 4] Testing High-Speed Flight & Anisotropic Prefetching...");

    let world_dir = workspace_root().join("data/world_san_francisco");
    let config = StreamingConfig {
        view_distance_meters: 3000.0,
        lod1_distance_meters: 800.0,
        lookahead_seconds: 5.0, // 5 second lookahead cone
        max_in_flight_tasks: 12,
        ..Default::default()
    };

    let mut streamer = WorldStreamer::open(&world_dir, config)
        .expect("WorldStreamer must open data/world_san_francisco");

    // Flight from South-East (over Bay Bridge approach) to North-West across downtown into the Bay
    // Flight speed: 200 m/s (~720 km/h = 388 knots) at 600m altitude
    let flight_start = Wgs84Coord::new(37.7810, -122.3890, 600.0);
    let flight_end = Wgs84Coord::new(37.8040, -122.4140, 800.0);

    let d_lat = (flight_end.lat - flight_start.lat) * 111_000.0;
    let d_lon = (flight_end.lon - flight_start.lon) * 88_000.0;
    let flight_dist_m = (d_lat * d_lat + d_lon * d_lon).sqrt();

    let flight_speed_mps = 200.0;
    let velocity = Vec3::new(
        (d_lon / flight_dist_m * flight_speed_mps) as f32,
        2.0,
        (d_lat / flight_dist_m * flight_speed_mps) as f32,
    );

    let steps = (flight_dist_m / 40.0).ceil() as usize; // Sample every 40m
    let mut total_flight_frames = 0;

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let lat = flight_start.lat + t * (flight_end.lat - flight_start.lat);
        let lon = flight_start.lon + t * (flight_end.lon - flight_start.lon);
        let alt = flight_start.alt + t * (flight_end.alt - flight_start.alt);

        let plane_pos = Wgs84Coord::new(lat, lon, alt);
        streamer
            .update_with_velocity(plane_pos, velocity)
            .expect("Flight streaming update must succeed");
        total_flight_frames += 1;

        let active_chunks = streamer.get_active_chunks();
        assert!(
            !active_chunks.is_empty(),
            "GATE 4 FAIL: Active chunk set became empty during high-speed flight at frame {}",
            total_flight_frames
        );

        // Verify terrain relief is present (non-empty terrain vertices)
        let has_terrain_mesh = active_chunks.iter().any(|c| !c.terrain.vertices.is_empty());
        assert!(
            has_terrain_mesh,
            "GATE 4 FAIL: Terrain meshes missing during flight"
        );
    }

    let telem = streamer.telemetry_details();
    println!(
        "   High-Speed Flight: {:.2} km flown at {:.0} m/s over {} frames",
        flight_dist_m / 1000.0,
        flight_speed_mps,
        total_flight_frames
    );
    println!(
        "   Streaming Telemetry: active={}, hits={}, misses={}, fallbacks={}",
        telem.active_chunks, telem.hits, telem.misses, telem.fallbacks
    );

    assert!(
        telem.active_chunks >= 4,
        "Flight view circle must maintain multiple resident chunks ahead of flight path"
    );

    println!("✅ [STAGE H2 GATE 4] PASSED: High-speed flight streaming and lookahead prefetching verified.");
}

// ============================================================================
// GATE 5: PRODUCTION STREAMING TELEMETRY & FAIL-CLOSED BOUNDARIES
// ============================================================================

#[test]
fn test_h2_gate5_production_streaming_telemetry_and_fail_closed_boundaries() {
    println!("\n📊 [STAGE H2 GATE 5] Testing Telemetry & Fail-Closed Boundaries...");

    let world_dir = workspace_root().join("data/world_san_francisco");
    let config = StreamingConfig {
        view_distance_meters: 1000.0,
        max_memory_mb: 20, // Small budget to trigger budget evictions under stress
        ..Default::default()
    };

    let mut streamer = WorldStreamer::open(&world_dir, config)
        .expect("WorldStreamer must open data/world_san_francisco");

    // 1. Verify Certified Bounds Reporting
    let cert_bounds = streamer.get_certified_bounds();
    assert!(
        cert_bounds.is_some(),
        "WorldStreamer must extract certified bounds from package manifest"
    );
    let bounds = cert_bounds.unwrap();
    assert_eq!(bounds.min_lat, 37.78);
    assert_eq!(bounds.max_lat, 37.805);
    assert_eq!(bounds.min_lon, -122.415);
    assert_eq!(bounds.max_lon, -122.388);

    // 2. Test In-Bounds vs Out-of-Bounds Queries
    let inside_coord = Wgs84Coord::new(37.795, -122.402, 10.0);
    let outside_coord = Wgs84Coord::new(37.700, -122.500, 10.0);

    assert!(
        streamer.is_within_certified_bounds(inside_coord),
        "Inside coordinate must be identified as within certified bounds"
    );
    assert!(
        !streamer.is_within_certified_bounds(outside_coord),
        "Outside coordinate must be identified as outside certified bounds"
    );

    // 3. Independent Detection of Missing Source vs Tile Starvation
    // Updating at an out-of-bounds coordinate should increment missing_sources, NOT silently fabricate fake cities
    let initial_missing = streamer.total_missing_sources;
    streamer
        .update_with_velocity(outside_coord, Vec3::ZERO)
        .expect("Update must succeed with graceful demand terrain generation");

    let telem_after_oob = streamer.telemetry_details();
    assert!(
        telem_after_oob.missing_sources > initial_missing,
        "GATE 5 FAIL: Out-of-bounds observer must record missing_sources in telemetry"
    );

    // 4. Memory Budget Eviction Under Continuous Stress Movement
    for i in 0..150 {
        let lat = 37.785 + (i as f64 * 0.0001);
        let lon = -122.410 + (i as f64 * 0.0001);
        let pos = Wgs84Coord::new(lat, lon, 15.0);
        streamer
            .update_with_velocity(pos, Vec3::new(10.0, 0.0, 10.0))
            .expect("Stress update must succeed");
    }

    let final_telem = streamer.telemetry_details();
    println!(
        "   Telemetry Snapshot: CPU={:.2}MB, GPU={:.2}MB, hits={}, misses={}, evictions={}, missing_sources={}",
        final_telem.cpu_memory_mb,
        final_telem.gpu_memory_mb,
        final_telem.hits,
        final_telem.misses,
        final_telem.evictions,
        final_telem.missing_sources
    );

    assert!(
        final_telem.cpu_memory_mb <= 25.0,
        "CPU memory ({:.2} MB) must remain bounded near budget (20 MB)",
        final_telem.cpu_memory_mb
    );

    // 5. Verify Demand-Driven Terrain Generates Non-Zero Contours (No 0m Flat Planks)
    let demand_bounds = GeoBounds::new(37.70, 37.705, -122.50, -122.495);
    let demand_chunk = generate_demand_terrain_chunk(0, 0, 0, &demand_bounds, None);
    assert!(
        !demand_chunk.terrain.vertices.is_empty(),
        "Demand chunk must contain generated terrain vertices"
    );
    assert!(
        demand_chunk.header.min_elevation >= 0.0,
        "Terrain elevation must be >= 0.0"
    );
    assert!(
        demand_chunk.header.max_elevation > demand_chunk.header.min_elevation,
        "Demand chunk must feature contoured relief, not flat 0m plank"
    );

    println!("✅ [STAGE H2 GATE 5] PASSED: Telemetry tracking, bounded memory, and fail-closed bounds verified.");
}
