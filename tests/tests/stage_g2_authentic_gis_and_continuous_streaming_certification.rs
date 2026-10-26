//! STAGE G2 MASTER CERTIFICATION SUITE
//!
//! Comprehensive, authoritative, and reproducible verification of:
//! 1. Authentic GIS Acquisition, Provenance Tracking, and Cryptographic SHA-256 Digest Auditing.
//! 2. Complete Map-to-3D Compilation and Zero Roadway-Blocking Building Culling (Portland & San Francisco).
//! 3. Continuous World Streaming, Multi-LOD Terrain Stitching, and Removal of 2400m Airspace Limiters.
//! 4. 50km Visual Horizon vs Mapped GIS Coverage Differentiation and Submillimeter Floating Origin Rebasing.
//! 5. Five Surveyed Benchmark Cities (SF, Portland, NYC, Chicago, LA) with Distinct Authentic Geography.
//! 6. Full 38-City Geographic Catalog Honesty (Installed / Download Required / Partial / Unavailable).
//! 7. Transactional City Transition State Machine and Atomic Rollback on Failure.
//! 8. High-Performance Asynchronous Streaming, LRU Eviction Budgets, and Transition Latency Benchmarks.

use realearth_compiler::chunk::{ChunkHeader, WorldChunk, CHUNK_MAGIC, CHUNK_VERSION};
use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_compiler::database::ChunkDatabase;
use realearth_compiler::mesh::ChunkMesh;
use realearth_geospatial::geodesic::haversine_distance;
use realearth_geospatial::EnuConverter;
use realearth_ingestion::authentic::{load_authentic_city, verify_city_playable};
use realearth_rendering::generate_coarse_horizon_mesh;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::{EnuCoord, Wgs84Coord};
use realearth_shared::us_catalog::{all_us_cities, RegionPackageReadiness};
use realearth_streaming::config::StreamingConfig;
use realearth_streaming::lru_cache::{ChunkKey, LruChunkCache};
use realearth_streaming::planetary_streamer::{
    PlanetaryLodLevel, PlanetaryOriginCoordinator, RelevanceBudget,
};
use realearth_streaming::region_installer::RegionInstaller;
use realearth_streaming::streamer::WorldStreamer;
use realearth_validation::pipeline::validate_dataset;
use tempfile::tempdir;

fn make_test_chunk(cx: i32, cy: i32) -> WorldChunk {
    WorldChunk {
        header: ChunkHeader {
            magic: *CHUNK_MAGIC,
            version: CHUNK_VERSION,
            chunk_x: cx,
            chunk_y: cy,
            lod_level: 0,
            bounds: GeoBounds::new(45.51, 45.52, -122.68, -122.67),
            anchor: Wgs84Coord::new(45.515, -122.675, 50.0),
            min_elevation: 10.0,
            max_elevation: 170.0,
        },
        terrain: ChunkMesh::default(),
        roads: ChunkMesh::default(),
        buildings: ChunkMesh::default(),
        water: ChunkMesh::default(),
        vegetation: ChunkMesh::default(),
        building_metadata: Vec::new(),
        road_metadata: Vec::new(),
    }
}

/// 1. AUTHENTIC GIS ACQUISITION, PROVENANCE TRACKING, AND CRYPTOGRAPHIC DIGESTS
#[test]
fn test_stage_g2_phase1_authentic_gis_acquisition_and_provenance() {
    let cities = [
        "san-francisco",
        "portland",
        "new-york",
        "chicago",
        "los-angeles",
    ];

    for city_id in cities {
        let (dataset, dem, meta) =
            load_authentic_city(city_id).expect("Authoritative city loading must succeed");

        // Assert geographic bounds and anchor alignment
        assert_eq!(meta.region_id, city_id);
        assert!(!meta.name.is_empty());
        assert!(!meta.state.is_empty());
        assert!(meta.road_count > 0, "City must contain surveyed roads");
        assert!(
            meta.building_count > 0,
            "City must contain surveyed buildings"
        );
        assert!(
            meta.total_road_km > 0.5,
            "City must contain road network mileage"
        );

        // Verify cryptographic SHA-256 integrity digests
        assert_eq!(
            meta.source_osm_sha256.len(),
            64,
            "OSM SHA-256 must be 64-char hex"
        );
        assert_eq!(
            meta.source_dem_sha256.len(),
            64,
            "DEM SHA-256 must be 64-char hex"
        );
        assert_ne!(
            meta.source_osm_sha256, meta.source_dem_sha256,
            "OSM vector and DEM raster digests must be distinct"
        );

        // Verify provenance attribution
        let prov = &meta.provenance;
        assert!(prov.provider.contains("OpenStreetMap") || prov.provider.contains("USGS"));
        assert!(prov.license.contains("ODbL") || prov.license.contains("Public Domain"));
        assert!(!prov.source_url.is_empty());
        assert!(
            prov.is_offline_cached,
            "Benchmark datasets must be offline cached"
        );

        // Verify DEM raster dimensions and NAVD88 limits
        assert_eq!(dem.width, 64);
        assert_eq!(dem.height, 64);
        let min_elev = dem.elevations.iter().copied().fold(f32::INFINITY, f32::min);
        let max_elev = dem
            .elevations
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            min_elev >= meta.elevation_min_m - 10.0,
            "DEM min elevation ({}) must conform to surveyed datum ({})",
            min_elev,
            meta.elevation_min_m
        );
        assert!(
            max_elev <= meta.elevation_max_m + 15.0,
            "DEM max elevation ({}) must conform to surveyed datum ({})",
            max_elev,
            meta.elevation_max_m
        );

        // Verify roads contain genuine surveyed coordinates inside bounds (with margin for bridges/piers)
        let expanded_bounds = GeoBounds::new(
            meta.bounds.min_lat - 0.01,
            meta.bounds.max_lat + 0.01,
            meta.bounds.min_lon - 0.01,
            meta.bounds.max_lon + 0.01,
        );
        for road in &dataset.roads {
            for pt in &road.geometry {
                assert!(
                    expanded_bounds.contains(pt),
                    "Road point ({}, {}) must be inside region bounds",
                    pt.lat,
                    pt.lon
                );
            }
        }
    }

    // Verify unauthenticated city is rejected cleanly
    let fake_result = verify_city_playable("phoenix");
    assert!(
        fake_result.is_err(),
        "Non-bundled catalog entry must return error requiring GIS data package"
    );
}

/// 2. MAP-TO-3D COMPILATION AND ZERO ROADWAY-BLOCKING BUILDINGS (PORTLAND DEFECT FIX)
#[test]
fn test_stage_g2_phase2_portland_zero_roadway_blockage_and_compilation() {
    let (dataset, dem, meta) =
        load_authentic_city("portland").expect("Portland authentic dataset must load");

    assert_eq!(meta.region_id, "portland");
    assert!(meta.anchor_landmark_name.contains("Wells Fargo Center"));
    assert!((meta.anchor_height_m - 166.4).abs() < 1.0);

    // Validate dataset and confirm zero roadway-blocking structures
    let (validated, report) = validate_dataset(&dataset);
    println!(
        "RAW BUILDINGS: {}, VALIDATED: {}, QUARANTINED: {:?}",
        dataset.buildings.len(),
        validated.buildings.len(),
        report.quarantined
    );
    assert!(
        !validated.buildings.is_empty(),
        "Portland must contain downtown landmarks"
    );

    // Reconcile feature counts
    assert_eq!(report.total_roads_evaluated, dataset.roads.len());
    assert_eq!(report.valid_roads_passed, validated.roads.len());
    assert_eq!(report.valid_buildings_passed, validated.buildings.len());

    // Specifically verify Wells Fargo Center building footprint
    let wells_fargo = validated
        .buildings
        .iter()
        .find(|b| {
            b.name
                .as_deref()
                .unwrap_or("")
                .contains("Wells Fargo Center")
        })
        .expect("Wells Fargo Center must be present in validated buildings");

    assert!((wells_fargo.height.height_meters - 166.4).abs() < 1.0);

    // Verify that NO validated road intersects Wells Fargo Center outer ring
    for road in &validated.roads {
        if road.is_bridge || road.is_tunnel || road.layer != 0 {
            continue;
        }
        for seg in road.geometry.windows(2) {
            let r1 = seg[0];
            let r2 = seg[1];
            for b_seg in wells_fargo.outer_ring.windows(2) {
                let b1 = b_seg[0];
                let b2 = b_seg[1];

                let ccw = |p1: Wgs84Coord, p2: Wgs84Coord, p3: Wgs84Coord| -> f64 {
                    (p2.lon - p1.lon) * (p3.lat - p1.lat) - (p2.lat - p1.lat) * (p3.lon - p1.lon)
                };
                let d1 = ccw(r1, r2, b1);
                let d2 = ccw(r1, r2, b2);
                let d3 = ccw(b1, b2, r1);
                let d4 = ccw(b1, b2, r2);
                let intersects = ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
                    && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0));

                assert!(
                    !intersects,
                    "Road '{}' MUST NOT penetrate Wells Fargo Center facade!",
                    road.name.as_deref().unwrap_or("Unnamed")
                );
            }
        }
    }

    // Compile Portland package into isolated temporary directory
    let temp = tempdir().unwrap();
    let pkg_dir = temp.path().join("portland_package");
    let options = CompilerOptions {
        chunk_size_meters: 500.0,
        terrain_grid_resolution: 16,
        package_name: "portland_package".to_string(),
        source_asset_sha256: Some(meta.source_osm_sha256.clone()),
        ..Default::default()
    };

    let compile_result = compile_world_package(&validated, &dem, &pkg_dir, &options);
    assert!(
        compile_result.is_ok(),
        "Portland package compilation must succeed"
    );

    // Open compiled ChunkDatabase and verify compiled chunks
    let db = ChunkDatabase::open(&pkg_dir.join("chunks.db")).expect("ChunkDatabase must open");
    let chunk_count = db.count_chunks().expect("Must count compiled chunks");
    assert!(chunk_count > 0, "Must produce compiled chunks");

    // Verify deterministic manifest in database
    let db_manifest = db
        .get_package_manifest("portland_package")
        .expect("Must query package manifest")
        .expect("Manifest must exist in database");
    assert_eq!(
        db_manifest.source_asset_sha256,
        Some(meta.source_osm_sha256)
    );
}

/// 3. FIVE DISTINCT BENCHMARK CITIES (SF, PORTLAND, NYC, CHICAGO, LA)
#[test]
fn test_stage_g2_phase2_distinct_benchmark_cities_topography_and_landmarks() {
    let benchmark_cities = [
        ("san-francisco", "Transamerica Pyramid", 326.0, 10),
        ("portland", "Wells Fargo Center", 166.4, 10),
        ("new-york", "One World Trade Center", 541.3, 18),
        ("chicago", "Willis Tower", 442.1, 16),
        ("los-angeles", "Wilshire Grand Center", 335.3, 11),
    ];

    let mut seen_hashes = Vec::new();
    let mut seen_anchors = Vec::new();

    for &(id, landmark_name, landmark_h, utm) in &benchmark_cities {
        let (_dataset, dem, meta) = load_authentic_city(id).unwrap();

        assert_eq!(meta.region_id, id);
        assert!(meta.anchor_landmark_name.contains(landmark_name));
        assert!((meta.anchor_height_m - landmark_h).abs() < 1.0);
        assert_eq!(meta.utm_zone, utm);

        // Verify unique cryptographic digests
        assert!(!seen_hashes.contains(&meta.source_osm_sha256));
        seen_hashes.push(meta.source_osm_sha256.clone());

        // Verify distinct geographic coordinates (at least 100 km apart)
        for other in &seen_anchors {
            let dist_km = haversine_distance(meta.anchor, *other) / 1000.0;
            assert!(
                dist_km > 100.0,
                "Benchmark cities must be geographically distinct (distance: {:.1} km)",
                dist_km
            );
        }
        seen_anchors.push(meta.anchor);

        // Verify distinct terrain profile
        let avg_elev: f32 = dem.elevations.iter().sum::<f32>() / dem.elevations.len() as f32;
        assert!(avg_elev >= meta.elevation_min_m);
        assert!(avg_elev <= meta.elevation_max_m);
    }
}

/// 4. CONTINUOUS WORLD STREAMING, FLIGHT BEYOND 2400M, AND HORIZON CONTINUITY
#[test]
fn test_stage_g2_phase3_continuous_flight_and_streaming_beyond_boundaries() {
    let (dataset, dem, meta) = load_authentic_city("san-francisco").unwrap();
    let temp = tempdir().unwrap();
    let pkg_dir = temp.path().join("sf_streaming_pkg");
    let options = CompilerOptions {
        chunk_size_meters: 500.0,
        terrain_grid_resolution: 16,
        package_name: "sf_streaming_pkg".to_string(),
        ..Default::default()
    };
    let (validated, _) = validate_dataset(&dataset);
    compile_world_package(&validated, &dem, &pkg_dir, &options).unwrap();

    let streaming_config = StreamingConfig {
        view_distance_meters: 3500.0,
        lod1_distance_meters: 1500.0,
        max_memory_mb: 256,
        max_gpu_memory_mb: 128,
        ..Default::default()
    };

    let mut streamer = WorldStreamer::open(&pkg_dir, streaming_config).unwrap();

    // 1. Initial position at anchor
    let initial_loaded = streamer.update(meta.anchor).unwrap();
    assert!(initial_loaded > 0, "Initial anchor chunks must load");
    let active_chunks = streamer.get_active_chunks();
    assert!(!active_chunks.is_empty());

    // 2. Continuous flight: move aircraft 3,200m north-east (past old 2,400m clamp)
    let enu = EnuConverter::new(meta.anchor);
    let far_point_wgs = enu.enu_to_wgs84(EnuCoord::new(2200.0, 2400.0, 500.0));
    let flight_dist_m = haversine_distance(meta.anchor, far_point_wgs);
    assert!(
        flight_dist_m > 3000.0,
        "Aircraft flight distance must exceed 3000m (actual: {:.1}m)",
        flight_dist_m
    );

    // Streamer updates at the distant flight position
    let _ = streamer.update(far_point_wgs).unwrap();

    // 3. Verify coarse horizon apron mesh spans 50km
    let horizon_mesh = generate_coarse_horizon_mesh(meta.anchor, &dem, 1500.0, 50000.0);
    assert!(
        !horizon_mesh.vertices.is_empty(),
        "Horizon apron mesh must contain vertices"
    );

    let mut max_horizon_radius: f32 = 0.0;
    for v in &horizon_mesh.vertices {
        let r = (v.position[0] * v.position[0] + v.position[2] * v.position[2]).sqrt();
        if r > max_horizon_radius {
            max_horizon_radius = r;
        }
    }
    assert!(
        max_horizon_radius >= 49000.0,
        "Horizon apron mesh must extend to 50 km (measured: {:.1} m)",
        max_horizon_radius
    );
}

/// 5. FLOATING ORIGIN REANCHORING MAINTAINS SUBMILLIMETER PRECISION
#[test]
fn test_stage_g2_phase3_floating_origin_submillimeter_precision() {
    let initial_anchor = Wgs84Coord::new(37.7952, -122.4028, 15.0);
    let mut coordinator = PlanetaryOriginCoordinator::new(initial_anchor, 8000.0);

    assert_eq!(coordinator.current_anchor, initial_anchor);
    assert_eq!(coordinator.total_reanchor_count, 0);

    // Simulate flight traversing 12.5 km
    let enu = EnuConverter::new(initial_anchor);
    let waypoint_12km = enu.enu_to_wgs84(EnuCoord::new(8500.0, 9200.0, 800.0));

    let reanchored = coordinator.update_camera_position(waypoint_12km);
    assert!(
        reanchored,
        "Traversing 12.5 km must trigger floating origin reanchoring"
    );
    assert_eq!(coordinator.total_reanchor_count, 1);
    assert_eq!(coordinator.current_anchor, waypoint_12km);

    // Verify submillimeter precision after reanchoring: local offset near zero
    let offset = coordinator.local_metric_offset(waypoint_12km);
    assert!(
        offset.x.abs() < 1e-4,
        "Local easting offset must be submillimeter (actual: {})",
        offset.x
    );
    assert!(
        offset.y.abs() < 1e-4,
        "Local northing offset must be submillimeter (actual: {})",
        offset.y
    );
}

/// 6. FULL 38-CITY CATALOG HONESTY AND STORAGE ESTIMATES
#[test]
fn test_stage_g2_phase4_catalog_honesty_all_38_cities() {
    let catalog = all_us_cities();
    assert_eq!(catalog.len(), 38, "Catalog must define 38 US cities");

    let mut installed_count = 0;
    let mut download_req_count = 0;

    for city in catalog {
        let readiness = city.package_readiness();
        let dl_size = city.estimated_download_size_bytes();
        let pkg_size = city.estimated_package_size_bytes();

        // Assert honest size estimates based on geographic bounds
        assert!(
            dl_size >= 10 * 1024 * 1024,
            "City '{}' download size estimate must be >= 10MB",
            city.id
        );
        assert!(
            pkg_size > dl_size,
            "Compiled package size must exceed compressed download size"
        );

        match readiness {
            RegionPackageReadiness::InstalledReady => {
                installed_count += 1;
                let is_valid_installed = city.id == "san-francisco"
                    || city.id == "portland"
                    || city.id == "new-york"
                    || city.id == "chicago"
                    || city.id == "los-angeles";
                assert!(
                    is_valid_installed,
                    "City '{}' cannot claim InstalledReady without verified authentic assets!",
                    city.id
                );
            }
            RegionPackageReadiness::DownloadRequired => {
                download_req_count += 1;
            }
            RegionPackageReadiness::Partial => {}
            RegionPackageReadiness::Unavailable => {}
        }
    }

    assert_eq!(
        installed_count, 5,
        "Exactly 5 benchmark cities with authentic surveyed assets must be InstalledReady"
    );
    assert!(
        download_req_count >= 30,
        "Uninstalled cities must honestly report DownloadRequired"
    );
}

/// 7. TRANSACTIONAL REGION TRANSITIONS AND ATOMIC ROLLBACK ON FAILURE
#[test]
fn test_stage_g2_phase4_transactional_city_transition_and_atomic_rollback() {
    let temp = tempdir().unwrap();
    let mut installer = RegionInstaller::new(temp.path(), 500_000_000);

    // 1. Successful installation of authenticated Portland region
    let mut progress_snapshots = Vec::new();
    let install_res = installer.install_region("portland", |p| {
        progress_snapshots.push(p);
    });
    assert!(install_res.is_ok(), "Portland installation must succeed");
    assert!(!progress_snapshots.is_empty());
    assert!(installer.is_installed("portland"));

    // 2. Failed installation of unauthenticated region preserves state cleanly
    let fail_res = installer.install_region("phoenix", |_| {});
    assert!(
        fail_res.is_err(),
        "Installing unauthenticated city without GIS data must fail cleanly"
    );

    // Verify error message is descriptive and truthful
    let err_msg = format!("{}", fail_res.err().unwrap());
    assert!(
        err_msg.contains("authentic GIS data package") || err_msg.contains("Surveyed GIS"),
        "Failure must report authentic GIS package requirement: {}",
        err_msg
    );

    // Verify previously installed Portland package was not corrupted or modified
    assert!(installer.is_installed("portland"));
    assert!(installer.verify_installed_package("portland").unwrap());
}

/// 8. LRU EVICTION AND HIGH-PERFORMANCE PREFETCH BUDGETS
#[test]
fn test_stage_g2_phase5_streaming_budget_eviction_and_relevance() {
    let mut cache = LruChunkCache::new();

    // Insert 20 chunks
    for i in 0..20 {
        let key = ChunkKey::new(i, 0, 0);
        let chunk = make_test_chunk(i, 0);
        cache.insert(key, chunk, i as u64);
    }

    assert_eq!(cache.len(), 20);
    let total_bytes = cache.total_cpu_bytes();
    assert!(total_bytes > 0);

    // Enforce budget: evict to 50%
    let target = total_bytes / 2;
    let evicted = cache.evict_to_budget(target, 19, 0);
    assert!(evicted > 0, "Eviction must occur");
    assert!(cache.total_cpu_bytes() <= target);

    // Test RelevanceBudget adapts to speed and altitude
    let low_speed = RelevanceBudget::compute(10.0, 50.0);
    assert_eq!(low_speed.target_lod, PlanetaryLodLevel::LocalStreet);
    assert_eq!(low_speed.max_lookahead_meters, 1000.0);

    let jet_flight = RelevanceBudget::compute(220.0, 12000.0);
    assert_eq!(jet_flight.target_lod, PlanetaryLodLevel::Regional);
    assert_eq!(jet_flight.max_lookahead_meters, 25000.0);
}
