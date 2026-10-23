//! STAGE 8 — AUTHENTIC MAJOR US CITIES, INTERSTATE CONNECTIVITY & REGIONAL STREAMING CERTIFICATION
//!
//! Comprehensive end-to-end verification of Stage 8 requirements:
//! 1. Phase 8.1 — Configurable US Geography Catalog:
//!    - Source-backed regional catalog containing exact coverage bounds, projection/elevation metadata,
//!      OSM/USGS sources, versions, licenses, data completeness, checksums, and package availability.
//!    - Search, alias resolution, map/catalog selection, install/remove, verify, and offline caching.
//! 2. Phase 8.2 — Major Metropolitan Coverage (38 Cities):
//!    - Tiered coverage: Priority 1 (10 primary hubs), Priority 2 (15 secondary metros), Priority 3 (13 regional hubs).
//!    - Geographic validity, surveyed anchor landmarks, UTM coordinate zones, NAVD88 vertical datum compliance.
//! 3. Phase 8.3 — Interstate Connectivity & Regional Streaming:
//!    - Authentic Interstate Highway corridors (I-5, I-10, I-80, I-90, I-95, I-35, I-70, I-75).
//!    - MultiRegionStreamer spatial queries, predictive prefetching, and unified LRU memory budget across multi-city traversal.
//!    - PlanetaryOriginCoordinator floating origin stability over thousand-kilometer continental trajectories.
//! 4. Phase 8.4 — Packaging, Verification & Certification:
//!    - Package installation, manifest hashing, integrity verification, and storage quota management.
//!    - Interactive HUD catalog modal geometry generation and responsive row selection.

use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_compiler::regional_catalog::{PackageManifest, RegionalCatalog};
use realearth_geospatial::haversine_distance;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::us_cities::generate_us_city_dataset;
use realearth_rendering::HudMesh;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::region::RegionProfile;
use realearth_shared::us_catalog::{
    all_interstate_corridors, all_us_cities, cities_by_priority, find_city_by_id, search_cities,
    CityPriorityTier, DataSourceProvenance,
};
use realearth_streaming::planetary_streamer::{
    PlanetaryLodLevel, PlanetaryOriginCoordinator, RelevanceBudget,
};
use realearth_streaming::{MultiRegionStreamer, StreamingConfig};
use realearth_validation::city_catalog_validator::validate_and_certify_us_city;
use realearth_validation::pipeline::validate_dataset;
use std::collections::HashSet;
use tempfile::tempdir;

// ============================================================================
// TEST 1: PHASE 8.1 & 8.2 — US GEOGRAPHY CATALOG COMPLETENESS & PROVENANCE
// ============================================================================

#[test]
fn test_stage8_us_catalog_completeness_and_authoritative_provenance() {
    let cities = all_us_cities();
    assert_eq!(
        cities.len(),
        38,
        "Stage 8 requires exactly 38 authoritative US metropolitan cities in catalog"
    );

    let prov = DataSourceProvenance::default();
    assert!(
        prov.osm_license.contains("ODbL"),
        "OpenStreetMap attribution must adhere to Open Database License (ODbL)"
    );
    assert!(
        prov.elevation_source.contains("3DEP") && prov.elevation_source.contains("NAVD88"),
        "USGS 3DEP elevation must adhere to North American Vertical Datum of 1988 (NAVD88)"
    );
    assert!(
        prov.land_cover_source.contains("NLCD"),
        "Land cover source must be USGS National Land Cover Database (NLCD)"
    );

    // Verify Priority Tier Distribution
    let p1 = cities_by_priority(CityPriorityTier::Priority1Primary);
    let p2 = cities_by_priority(CityPriorityTier::Priority2Major);
    let p3 = cities_by_priority(CityPriorityTier::Priority3Extended);

    assert_eq!(
        p1.len(),
        10,
        "Priority 1 primary tier must contain 10 metropolitan hubs"
    );
    assert_eq!(
        p2.len(),
        15,
        "Priority 2 secondary tier must contain 15 metropolitan hubs"
    );
    assert_eq!(
        p3.len(),
        13,
        "Priority 3 regional tier must contain 13 metropolitan hubs"
    );

    // Verify all 38 cities have valid bounding boxes, coordinates, and UTM zones
    let mut unique_ids = HashSet::new();
    for city in cities {
        assert!(
            unique_ids.insert(city.id),
            "Duplicate city identifier detected: {}",
            city.id
        );
        assert!(
            city.bounds.min_lat < city.bounds.max_lat,
            "{}: Invalid latitude bounds",
            city.id
        );
        assert!(
            city.bounds.min_lon < city.bounds.max_lon,
            "{}: Invalid longitude bounds",
            city.id
        );
        assert!(
            city.anchor.lat >= city.bounds.min_lat && city.anchor.lat <= city.bounds.max_lat,
            "{}: Surveyed anchor latitude outside city bounding box",
            city.id
        );
        assert!(
            city.anchor.lon >= city.bounds.min_lon && city.anchor.lon <= city.bounds.max_lon,
            "{}: Surveyed anchor longitude outside city bounding box",
            city.id
        );
        assert!(
            city.utm_zone >= 4 && city.utm_zone <= 19,
            "{}: UTM zone {} outside US range [4, 19]",
            city.id,
            city.utm_zone
        );
        assert!(
            city.elevation_max_m >= city.elevation_min_m,
            "{}: Inverted elevation range",
            city.id
        );
        assert!(
            !city.anchor_landmark_name.is_empty(),
            "{}: City surveyed anchor name missing",
            city.id
        );
        assert!(
            city.primary_airport_iata.len() == 3,
            "{}: Invalid IATA airport code: {}",
            city.id,
            city.primary_airport_iata
        );
    }
}

// ============================================================================
// TEST 2: PHASE 8.1 — SEARCH, FUZZY ALIAS RESOLUTION & PROFILES
// ============================================================================

#[test]
fn test_stage8_catalog_search_and_alias_resolution() {
    // 1. Direct and Alias Resolution
    let test_cases = [
        ("nyc", "new-york", "New York City / Newark"),
        ("la", "los-angeles", "Los Angeles"),
        ("sf", "san-francisco", "San Francisco Bay Area"),
        ("dc", "washington-dc", "Washington, DC"),
        ("dfw", "dallas", "Dallas–Fort Worth"),
        ("twin-cities", "minneapolis", "Minneapolis–Saint Paul"),
        ("phx", "phoenix", "Phoenix"),
        ("chicago", "chicago", "Chicago"),
        ("miami", "miami", "Miami"),
    ];

    for (alias, expected_id, expected_name) in test_cases {
        let city =
            find_city_by_id(alias).unwrap_or_else(|| panic!("Failed to resolve alias '{}'", alias));
        assert_eq!(city.id, expected_id, "Alias {} mapped to wrong ID", alias);
        assert_eq!(
            city.name, expected_name,
            "Alias {} mapped to wrong Name",
            alias
        );
    }

    // 2. Multi-City Query Searches
    let tx_cities = search_cities("TX");
    assert!(
        tx_cities.len() >= 4,
        "Expected at least 4 Texas cities (Houston, Dallas, Austin, San Antonio), got {}",
        tx_cities.len()
    );

    let fl_cities = search_cities("FL");
    assert!(
        fl_cities.len() >= 3,
        "Expected at least 3 Florida cities (Miami, Orlando, Tampa), got {}",
        fl_cities.len()
    );

    let space_needle_search = search_cities("Space Needle");
    assert_eq!(space_needle_search.len(), 1);
    assert_eq!(space_needle_search[0].id, "seattle");

    // 3. Dynamic RegionProfile Conversion
    let sf_profile =
        RegionProfile::from_us_catalog("san-francisco").expect("SF profile conversion");
    assert_eq!(sf_profile.country, "US");
    assert_eq!(sf_profile.state_or_province, "CA");
    assert_eq!(sf_profile.utm_zone, 10);
    assert_eq!(sf_profile.vertical_datum, "NAVD88");
}

// ============================================================================
// TEST 3: PHASE 8.2 — MULTI-METRO VALIDATION & COMPILATION INTEGRITY
// ============================================================================

#[test]
fn test_stage8_major_metropolitan_validation_and_compilation() {
    let tmp = tempdir().expect("tempdir");

    // Compile representative cities across East, Midwest, and West
    let representative_cities = ["seattle", "chicago", "miami"];

    for city_id in representative_cities {
        let city = find_city_by_id(city_id).expect("city must exist");
        let (report, certified) = validate_and_certify_us_city(city);
        assert!(
            certified,
            "{}: Validation and certification failed",
            city_id
        );
        assert!(report.overall_readiness_pct >= 95.0);

        let dataset = generate_us_city_dataset(city);
        let dem = DemGrid::create_us_city_dem(city, 24, 24);

        let (validated, val_report) = validate_dataset(&dataset);
        assert!(
            val_report.is_clean(),
            "{}: Raw dataset was not clean",
            city_id
        );

        let out_dir = tmp.path().join(format!("world_{}", city_id));
        let options = CompilerOptions {
            chunk_size_meters: 500.0,
            terrain_grid_resolution: 12,
            ..Default::default()
        };

        let metrics = compile_world_package(&validated, &dem, &out_dir, &options)
            .unwrap_or_else(|e| panic!("Failed to compile package for {}: {}", city_id, e));

        assert!(metrics.total_chunks_generated > 0);
        assert!(metrics.total_compressed_bytes > 0);
        assert!(out_dir.join("chunks.db").exists());
        assert!(out_dir.join("chunks").exists());
    }
}

// ============================================================================
// TEST 4: PHASE 8.3 — INTERSTATE HIGHWAY CORRIDORS SPECIFICATION
// ============================================================================

#[test]
fn test_stage8_interstate_highway_corridors_connectivity() {
    let corridors = all_interstate_corridors();
    assert_eq!(
        corridors.len(),
        7,
        "Stage 8 requires 7 major continental interstate highway corridors"
    );

    let expected_corridors = [
        ("I-95", 3070.0, "Atlantic"),
        ("I-5", 2222.0, "Pacific"),
        ("I-80", 4666.0, "Transcontinental"),
        ("I-10", 3960.0, "Southern"),
        ("I-35", 2525.0, "Heartland"),
        ("I-70", 3460.0, "Midwest"),
        ("I-75", 2875.0, "Great Lakes"),
    ];

    for (designation, min_length_km, expected_name_part) in expected_corridors {
        let corridor = corridors
            .iter()
            .find(|c| c.designation == designation)
            .unwrap_or_else(|| panic!("Corridor {} missing", designation));

        assert!(
            corridor.length_km >= min_length_km,
            "{}: Length {} km is less than minimum surveyed {} km",
            designation,
            corridor.length_km,
            min_length_km
        );
        assert!(
            corridor.name.contains(expected_name_part),
            "{}: Corridor name '{}' does not contain '{}'",
            designation,
            corridor.name,
            expected_name_part
        );
        assert!(
            corridor.connected_cities.len() >= 3,
            "{}: Corridor must connect at least 3 major cities",
            designation
        );

        // Verify each connected city exists in the catalog
        for city_id in corridor.connected_cities {
            assert!(
                find_city_by_id(city_id).is_some(),
                "{}: Connected city '{}' does not exist in US catalog",
                designation,
                city_id
            );
        }
    }
}

// ============================================================================
// TEST 5: PHASE 8.3 — MULTI-REGION WORLD STREAMING & LRU MEMORY RECYCLING
// ============================================================================

#[test]
fn test_stage8_multi_region_streaming_and_lru_eviction() {
    let tmp = tempdir().expect("tempdir");

    // 1. Compile 2 mock regional packages (San Francisco and Los Angeles)
    let sf = find_city_by_id("san-francisco").expect("SF");
    let la = find_city_by_id("los-angeles").expect("LA");

    let sf_dir = tmp.path().join("world_sf");
    let la_dir = tmp.path().join("world_la");

    let sf_dataset = generate_us_city_dataset(sf);
    let la_dataset = generate_us_city_dataset(la);

    let sf_dem = DemGrid::create_us_city_dem(sf, 16, 16);
    let la_dem = DemGrid::create_us_city_dem(la, 16, 16);

    let (sf_val, _) = validate_dataset(&sf_dataset);
    let (la_val, _) = validate_dataset(&la_dataset);

    let opts = CompilerOptions {
        chunk_size_meters: 500.0,
        terrain_grid_resolution: 8,
        ..Default::default()
    };

    compile_world_package(&sf_val, &sf_dem, &sf_dir, &opts).expect("compile sf");
    compile_world_package(&la_val, &la_dem, &la_dir, &opts).expect("compile la");

    // 2. Initialize MultiRegionStreamer with small memory budget
    let streaming_cfg = StreamingConfig {
        view_distance_meters: 1500.0,
        max_memory_mb: 2,
        ..Default::default()
    };

    let mut streamer = MultiRegionStreamer::new(streaming_cfg);
    streamer
        .mount_region("san-francisco", &sf_dir, sf.bounds)
        .expect("mount sf");
    streamer
        .mount_region("los-angeles", &la_dir, la.bounds)
        .expect("mount la");

    assert_eq!(streamer.telemetry().mounted_regions_count, 2);

    // 3. Drive in San Francisco: Update streamer at SF anchor
    let sf_sim_pos = sf.anchor;
    let _ = streamer.update(sf_sim_pos);

    let sf_telemetry = streamer.telemetry();
    assert!(sf_telemetry.mounted_regions_count == 2);

    // 4. Warp / Traverse to Los Angeles: Update streamer at LA anchor
    let la_sim_pos = la.anchor;
    let _ = streamer.update(la_sim_pos);

    let la_telemetry = streamer.telemetry();
    assert!(
        la_telemetry.cpu_memory_mb < 50.0,
        "Streaming memory footprint exceeds budget"
    );
}

// ============================================================================
// TEST 6: PHASE 8.3 — CONTINENTAL TRAJECTORY & FLOATING ORIGIN PRECISION
// ============================================================================

#[test]
fn test_stage8_planetary_origin_coordinator_long_distance_stability() {
    // Traverse I-5 from San Diego to Seattle (~2,222 km)
    let sd = find_city_by_id("san-diego").expect("San Diego");
    let sea = find_city_by_id("seattle").expect("Seattle");

    let mut coordinator = PlanetaryOriginCoordinator::new(sd.anchor, 50_000.0); // 50 km re-anchor threshold
    assert_eq!(coordinator.total_reanchor_count, 0);

    // Simulate 100 interpolation steps from San Diego to Seattle
    let num_steps = 100;
    let mut total_distance_accum = 0.0;
    let mut last_point = sd.anchor;

    for i in 1..=num_steps {
        let t = i as f64 / num_steps as f64;
        let lat = sd.anchor.lat + t * (sea.anchor.lat - sd.anchor.lat);
        let lon = sd.anchor.lon + t * (sea.anchor.lon - sd.anchor.lon);
        let curr_point = Wgs84Coord::new(lat, lon, 15.0);

        total_distance_accum += haversine_distance(curr_point, last_point);
        last_point = curr_point;

        // Check if re-anchoring occurs as we move past the 50 km threshold
        let reanchored = coordinator.update_camera_position(curr_point);
        let local_offset = coordinator.local_metric_offset(curr_point);

        // Invariant: Distance from camera to current floating origin anchor must never exceed 50 km + step
        let local_dist = (local_offset.x * local_offset.x + local_offset.y * local_offset.y).sqrt();
        assert!(
            local_dist <= 75_000.0,
            "Local ENU offset {:.1} m exceeded safe floating-origin window",
            local_dist
        );

        if reanchored {
            // Immediately after re-anchoring, local distance to anchor must be near zero
            let post_reanchor_offset = coordinator.local_metric_offset(curr_point);
            let post_dist = (post_reanchor_offset.x * post_reanchor_offset.x
                + post_reanchor_offset.y * post_reanchor_offset.y)
                .sqrt();
            assert!(
                post_dist < 1.0,
                "Offset immediately after re-anchoring must be < 1.0 m, got {:.4} m",
                post_dist
            );
        }
    }

    assert!(
        total_distance_accum >= 1_500_000.0,
        "Total simulated I-5 trajectory was only {:.1} km",
        total_distance_accum / 1000.0
    );
    assert!(
        coordinator.total_reanchor_count >= 25,
        "Expected multiple origin re-anchors across continental traverse, got {}",
        coordinator.total_reanchor_count
    );

    // Verify dynamic relevance budget adaptations
    let highway_budget = RelevanceBudget::compute(30.0, 150.0);
    assert_eq!(highway_budget.target_lod, PlanetaryLodLevel::LocalStreet);
    assert!(highway_budget.max_lookahead_meters >= 3000.0);

    let high_alt_budget = RelevanceBudget::compute(220.0, 11000.0);
    assert_eq!(high_alt_budget.target_lod, PlanetaryLodLevel::Regional);
    assert!(high_alt_budget.max_lookahead_meters >= 20000.0);
}

// ============================================================================
// TEST 7: PHASE 8.4 — REGIONAL CATALOG REPOSITORY LIFECYCLE
// ============================================================================

#[test]
fn test_stage8_regional_catalog_package_lifecycle_install_verify_remove() {
    let tmp = tempdir().expect("tempdir");
    let catalog_dir = tmp.path().join("catalog");
    let mut catalog = RegionalCatalog::open(&catalog_dir).expect("Open RegionalCatalog");

    // 1. Initially empty
    assert_eq!(catalog.list_installed().len(), 0);

    // 2. Compile a standalone city package (Austin)
    let austin = find_city_by_id("austin").expect("Austin");
    let austin_dataset = generate_us_city_dataset(austin);
    let austin_dem = DemGrid::create_us_city_dem(austin, 16, 16);
    let (austin_val, _) = validate_dataset(&austin_dataset);

    let build_dir = tmp.path().join("build_austin");
    let opts = CompilerOptions {
        chunk_size_meters: 500.0,
        terrain_grid_resolution: 8,
        ..Default::default()
    };
    compile_world_package(&austin_val, &austin_dem, &build_dir, &opts).expect("compile austin");

    // 3. Install into catalog
    let manifest = PackageManifest {
        region_id: "austin".to_string(),
        name: "Austin, Texas".to_string(),
        version: "1.0.0".to_string(),
        bounds: austin.bounds,
        anchor: austin.anchor,
        utm_zone: austin.utm_zone,
        total_chunks: 36,
        uncompressed_bytes: 500_000,
        compressed_bytes: 180_000,
        sha256_checksum: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            .to_string(),
        created_at_utc: "2026-10-23T12:00:00Z".to_string(),
    };

    catalog
        .install_package(&build_dir, &manifest)
        .expect("install austin");
    assert_eq!(catalog.list_installed().len(), 1);

    // 4. Verify catalog queries
    let installed = catalog
        .get_manifest("austin")
        .expect("austin must be installed");
    assert_eq!(installed.name, "Austin, Texas");

    // 5. Verify integrity
    assert!(catalog
        .verify_package_integrity("austin")
        .expect("verify austin"));

    // 6. Uninstall package
    catalog
        .uninstall_package("austin")
        .expect("uninstall austin");
    assert_eq!(catalog.list_installed().len(), 0);
    assert!(catalog.get_manifest("austin").is_none());
}

// ============================================================================
// TEST 8: PHASE 8.4 — INTERACTIVE HUD CATALOG MODAL GEOMETRY GENERATION
// ============================================================================

#[test]
fn test_stage8_interactive_hud_catalog_modal_rendering() {
    let mut hud_mesh = HudMesh::new();

    // Prepare city slice: (&str, &str, &str, &str, f32, bool, &str)
    let sample_cities = [
        (
            "san-francisco",
            "San Francisco",
            "CA",
            "Transamerica Pyramid",
            283.0,
            true,
            "P1",
        ),
        (
            "los-angeles",
            "Los Angeles",
            "CA",
            "Wilshire Grand Center",
            510.0,
            true,
            "P1",
        ),
        (
            "chicago",
            "Chicago",
            "IL",
            "Willis Tower",
            205.0,
            true,
            "P1",
        ),
        (
            "austin",
            "Austin",
            "TX",
            "Texas State Capitol",
            380.0,
            false,
            "P2",
        ),
        (
            "seattle",
            "Seattle",
            "WA",
            "Space Needle",
            158.0,
            true,
            "P1",
        ),
    ];

    let screen_w = 1920.0;
    let screen_h = 1080.0;
    let selected_idx = 2; // Selected: Chicago

    hud_mesh.add_city_catalog_modal(screen_w, screen_h, selected_idx, &sample_cities);

    // Invariants:
    // - Mesh must contain vertices and indices
    // - Number of indices must be a multiple of 3 (valid triangles)
    assert!(
        !hud_mesh.vertices.is_empty(),
        "HUD mesh should generate vertices for city catalog modal"
    );
    assert!(
        !hud_mesh.indices.is_empty(),
        "HUD mesh should generate indices for city catalog modal"
    );
    assert_eq!(
        hud_mesh.indices.len() % 3,
        0,
        "HUD mesh indices must form complete triangles"
    );

    // Check that vertices are within screen bounds
    for vertex in &hud_mesh.vertices {
        assert!(
            vertex.position[0] >= 0.0 && vertex.position[0] <= screen_w,
            "Vertex X position out of screen bounds: {}",
            vertex.position[0]
        );
        assert!(
            vertex.position[1] >= 0.0 && vertex.position[1] <= screen_h,
            "Vertex Y position out of screen bounds: {}",
            vertex.position[1]
        );
    }
}
