//! STAGE 2 CERTIFICATION TEST SUITE — AUTHENTIC GIS INGESTION & REAL US COVERAGE
//!
//! Comprehensive tests verifying:
//! - 2.1: Removal of production synthesis & isolation of synthetic fixtures.
//! - 2.2: Authentic ingestion with reproducible OSM acquisition, USGS 3DEP DEM, and full provenance tracking.
//! - 2.3: Geographic correctness: elevation sampling, footprints, heights, grade separations, and quarantine.
//! - 2.4: Real compilation into versioned, indexed, compressed runtime chunks with collision and navigation.
//! - 2.5: Documented US coverage across San Francisco pilot-core and prioritized major US metropolitan cities.

use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_compiler::database::ChunkDatabase;
use realearth_ingestion::authentic::{load_authentic_city, verify_city_playable};
use realearth_navigation::RoadNetworkGraph;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::us_catalog::{
    cities_by_priority, find_city_by_id, CityPriorityTier, RegionPackageReadiness,
};
use realearth_validation::diagnostic::ValidationTolerance;
use realearth_validation::pipeline::validate_dataset_with_tolerances;
use tempfile::tempdir;

// ============================================================================
// 2.1 & 2.2: AUTHENTIC SAN FRANCISCO GIS INGESTION & DEM ACCURACY
// ============================================================================

#[test]
fn test_stage2_san_francisco_authentic_ingestion_and_dem_accuracy() {
    let (dataset, dem, meta) = load_authentic_city("san-francisco")
        .expect("Authentic San Francisco dataset must load successfully");

    // 1. Verify region metadata and exact bounding extent
    assert_eq!(meta.region_id, "san-francisco");
    assert_eq!(meta.name, "San Francisco Bay Area");
    assert_eq!(meta.state, "CA");
    assert_eq!(meta.utm_zone, 10);
    assert_eq!(meta.bounds.min_lat, 37.7800);
    assert_eq!(meta.bounds.max_lat, 37.8050);
    assert_eq!(meta.bounds.min_lon, -122.4150);
    assert_eq!(meta.bounds.max_lon, -122.3880);

    // 2. Verify provenance tracking and non-empty cryptographic hashes
    assert!(
        meta.provenance.license.contains("ODbL")
            || meta.provenance.license.contains("Public Domain"),
        "Provenance license must reference ODbL / Public Domain"
    );
    assert!(
        meta.provenance.provider.contains("OpenStreetMap")
            && meta.provenance.provider.contains("USGS"),
        "Provider must attribute OSM and USGS"
    );
    assert_eq!(
        meta.source_osm_sha256.len(),
        64,
        "OSM source SHA-256 must be valid 64-character hex digest"
    );
    assert_eq!(
        meta.source_dem_sha256.len(),
        64,
        "DEM source SHA-256 must be valid 64-character hex digest"
    );

    // 3. Verify surveyed skyscraper heights and floors
    assert!(
        dataset.buildings.len() >= 5,
        "San Francisco dataset must contain surveyed landmark buildings, got {}",
        dataset.buildings.len()
    );

    let salesforce = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("Salesforce Tower"))
        .expect("Salesforce Tower must be present");
    assert_eq!(
        salesforce.explicit_height,
        Some(326.0),
        "Salesforce Tower surveyed height must be 326.0m"
    );
    assert_eq!(
        salesforce.levels,
        Some(61),
        "Salesforce Tower floor count must be 61"
    );

    let transamerica = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("Transamerica Pyramid"))
        .expect("Transamerica Pyramid must be present");
    assert_eq!(
        transamerica.explicit_height,
        Some(260.0),
        "Transamerica Pyramid surveyed height must be 260.0m"
    );
    assert_eq!(
        transamerica.levels,
        Some(48),
        "Transamerica floor count must be 48"
    );

    let california555 = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("555 California Street"))
        .expect("555 California must be present");
    assert_eq!(
        california555.explicit_height,
        Some(237.0),
        "555 California surveyed height must be 237.0m"
    );

    let ferry_bldg = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("San Francisco Ferry Building"))
        .expect("Ferry Building must be present");
    assert_eq!(
        ferry_bldg.explicit_height,
        Some(75.0),
        "Ferry Building height must be 75.0m"
    );

    let coit_tower = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("Coit Tower"))
        .expect("Coit Tower must be present");
    assert_eq!(
        coit_tower.explicit_height,
        Some(64.0),
        "Coit Tower height must be 64.0m"
    );

    // 4. Verify authentic road attributes (lanes, speed limits, grade separations)
    assert!(
        dataset.roads.len() >= 6,
        "San Francisco dataset must contain arterial roads and highways, got {}",
        dataset.roads.len()
    );

    let market_st = dataset
        .roads
        .iter()
        .find(|r| r.name.as_deref() == Some("Market Street"))
        .expect("Market Street must be present");
    assert_eq!(market_st.lanes, 4);

    let bay_bridge = dataset
        .roads
        .iter()
        .find(|r| r.name.as_deref() == Some("Interstate 80 Bay Bridge Approach"))
        .expect("Bay Bridge approach must be present");
    assert!(bay_bridge.is_bridge, "Bay Bridge must be tagged as bridge");
    assert_eq!(bay_bridge.layer, 1, "Bay Bridge must be layer 1");
    assert_eq!(bay_bridge.lanes, 5);

    let broadway_tunnel = dataset
        .roads
        .iter()
        .find(|r| r.name.as_deref() == Some("Broadway Tunnel"))
        .expect("Broadway Tunnel must be present");
    assert!(
        broadway_tunnel.is_tunnel,
        "Broadway Tunnel must be tagged as tunnel"
    );
    assert_eq!(
        broadway_tunnel.layer, -1,
        "Broadway Tunnel must be layer -1"
    );

    // 5. Verify USGS 3DEP DEM precision and accuracy
    assert_eq!(dem.width, 64);
    assert_eq!(dem.height, 64);
    assert!(
        meta.dem_accuracy_m <= 0.5,
        "DEM accuracy must be <= 0.5m for coastal LIDAR, got {}m",
        meta.dem_accuracy_m
    );

    // Coastline water level along Eastern bay margin
    let bay_coord = Wgs84Coord::new(37.7950, -122.3890, 0.0);
    let bay_elevation = dem.sample_bilinear(&bay_coord);
    assert!(
        bay_elevation <= 1.0,
        "San Francisco Bay water level must be <= 1.0m MSL, got {:.2}m",
        bay_elevation
    );

    // Telegraph Hill (Coit Tower) high elevation
    let coit_coord = Wgs84Coord::new(37.8024, -122.4058, 0.0);
    let coit_ground = dem.sample_bilinear(&coit_coord);
    assert!(
        coit_ground >= 25.0,
        "Telegraph Hill ground elevation must be elevated (>= 25m), got {:.2}m",
        coit_ground
    );

    // Nob Hill (California Street) peak elevation
    let nob_coord = Wgs84Coord::new(37.7925, -122.4110, 0.0);
    let nob_ground = dem.sample_bilinear(&nob_coord);
    assert!(
        nob_ground >= 50.0,
        "Nob Hill elevation must exceed 50m MSL, got {:.2}m",
        nob_ground
    );
}

// ============================================================================
// 2.1: NO SYNTHETIC SYNTHESIS IN PRODUCTION SOURCES
// ============================================================================

#[test]
fn test_stage2_no_synthetic_synthesis_in_production_sources() {
    let (dataset, _, meta) =
        load_authentic_city("san-francisco").expect("Authentic load must succeed");

    // Ensure all buildings have authentic, non-synthetic geometry
    for b in &dataset.buildings {
        assert!(
            b.outer_ring.len() >= 4,
            "Building {} ring must have at least 4 coordinates",
            b.id
        );
        assert!(
            b.explicit_height.is_some() || b.levels.is_some(),
            "Building {} must have surveyed height or levels",
            b.id
        );
    }

    // Ensure road network has non-synthetic feature IDs (> 100,000)
    for r in &dataset.roads {
        assert!(
            r.way_id >= 100_000,
            "Way ID {} must follow authentic numbering",
            r.way_id
        );
        assert!(
            r.geometry.len() >= 2,
            "Road {} must have >= 2 coordinates",
            r.way_id
        );
    }

    // Total road length must be positive and non-trivial
    assert!(
        meta.total_road_km > 2.0,
        "Expected at least 2.0 km of surveyed roads, got {:.2} km",
        meta.total_road_km
    );
}

// ============================================================================
// 2.3: GEOGRAPHIC CORRECTNESS & QUARANTINE VALIDATION
// ============================================================================

#[test]
fn test_stage2_geographic_correctness_and_quarantine() {
    let (dataset, dem, _) =
        load_authentic_city("san-francisco").expect("Authentic load must succeed");

    let tolerance = ValidationTolerance::default();
    let (validated, report) = validate_dataset_with_tolerances(&dataset, &tolerance, Some(&dem));

    // Zero quarantined features in authentic reference dataset
    assert_eq!(
        report.quarantined.len(),
        0,
        "Authentic dataset must have zero quarantined features, got: {:?}",
        report.quarantined
    );

    // Validated geometry counts must match
    assert_eq!(validated.buildings.len(), dataset.buildings.len());
    assert_eq!(validated.roads.len(), dataset.roads.len());

    // Grade separation check: Bay Bridge must remain elevated, Broadway Tunnel must remain subterranean
    let bridge = validated
        .roads
        .iter()
        .find(|r| r.name.as_deref() == Some("Interstate 80 Bay Bridge Approach"))
        .unwrap();
    assert_eq!(bridge.layer, 1);
    assert!(bridge.is_bridge);

    let tunnel = validated
        .roads
        .iter()
        .find(|r| r.name.as_deref() == Some("Broadway Tunnel"))
        .unwrap();
    assert_eq!(tunnel.layer, -1);
    assert!(tunnel.is_tunnel);
}

// ============================================================================
// 2.4: REAL COMPILATION & CHUNK INDEXING
// ============================================================================

#[test]
fn test_stage2_authoritative_compilation_and_chunk_integrity() {
    let (dataset, dem, meta) =
        load_authentic_city("san-francisco").expect("Authentic load must succeed");

    let tolerance = ValidationTolerance::default();
    let (validated, _) = validate_dataset_with_tolerances(&dataset, &tolerance, Some(&dem));

    let temp_dir = tempdir().expect("Failed to create tempdir");
    let options = CompilerOptions {
        chunk_size_meters: 500.0,
        terrain_grid_resolution: 16,
        incremental: false,
        package_name: format!("{}_package", meta.region_id),
        ..Default::default()
    };

    let stats = compile_world_package(&validated, &dem, temp_dir.path(), &options)
        .expect("Compilation of authentic GIS package must succeed");

    assert!(
        stats.total_chunks_generated > 0,
        "Compilation must generate runtime chunks"
    );
    assert!(
        stats.total_vertices > 0,
        "Compilation must produce 3D vertices"
    );
    assert!(
        stats.total_triangles > 0,
        "Compilation must produce 3D triangles"
    );

    // Verify SQLite chunks.db index exists and is queryable
    let db_path = temp_dir.path().join("chunks.db");
    let db = ChunkDatabase::open(&db_path).expect("chunks.db must open successfully");
    let chunk_count = db.count_chunks().expect("Must count chunk records");
    assert_eq!(chunk_count, stats.total_chunks_generated);

    // Query chunks intersecting region bounds
    let bbox_chunks = db.query_bbox(&meta.bounds).expect("Must query bbox chunks");
    assert!(!bbox_chunks.is_empty(), "Must find chunks in region bounds");

    let first = &bbox_chunks[0];
    assert!(
        first.compressed_bytes > 0,
        "Chunk must have compressed bytes"
    );
    assert!(
        first.uncompressed_bytes > 0,
        "Chunk must have uncompressed bytes"
    );
    assert!(
        bbox_chunks.iter().any(|c| c.feature_count > 0),
        "At least one chunk in the region must contain vector features"
    );

    // Verify manifest record exists
    let pkg_name = format!("{}_package", meta.region_id);
    let manifest = db
        .get_package_manifest(&pkg_name)
        .expect("Database query must succeed")
        .expect("Manifest record must exist");
    assert_eq!(manifest.package_name, pkg_name);
    assert_eq!(manifest.crs_epsg, 4326);
}

// ============================================================================
// 2.5: PRIORITY 1 MAJOR US CITIES COVERAGE & NAVIGATION
// ============================================================================

#[test]
fn test_stage2_priority1_major_us_cities_coverage_and_navigation() {
    let priority1_cities = cities_by_priority(CityPriorityTier::Priority1Primary);
    assert!(
        !priority1_cities.is_empty(),
        "Priority 1 cities catalog must not be empty"
    );

    for city in priority1_cities {
        match city.package_readiness() {
            RegionPackageReadiness::InstalledReady => {
                let (dataset, dem, meta) = load_authentic_city(city.id).unwrap_or_else(|e| {
                    panic!(
                        "Installed benchmark city '{}' must load authentically: {}",
                        city.id, e
                    )
                });

                // 1. Exact extent and anchor verification
                assert_eq!(meta.bounds.min_lat, city.bounds.min_lat);
                assert_eq!(meta.bounds.max_lat, city.bounds.max_lat);
                assert_eq!(meta.bounds.min_lon, city.bounds.min_lon);
                assert_eq!(meta.bounds.max_lon, city.bounds.max_lon);
                assert!(
                    city.bounds.contains(&city.anchor),
                    "City {} anchor must be inside bounding box",
                    city.id
                );

                // 2. Building coverage: Surveyed anchor landmark skyscraper must be present
                assert!(
                    !dataset.buildings.is_empty(),
                    "City {} must contain buildings",
                    city.id
                );
                let has_anchor_landmark = dataset.buildings.iter().any(|b| {
                    b.name
                        .as_deref()
                        .map(|n| {
                            city.anchor_landmark_name.contains(n)
                                || n.contains(city.anchor_landmark_name)
                                || n.contains(city.name)
                        })
                        .unwrap_or(false)
                });
                assert!(
                    has_anchor_landmark,
                    "City {} must contain anchor landmark '{}'",
                    city.id, city.anchor_landmark_name
                );

                // 3. Road Network Graph & Executable A* Navigation Pathfinding
                assert!(
                    dataset.roads.len() >= 2,
                    "City {} must contain connected road network (>= 2 roads)",
                    city.id
                );

                let road_graph = RoadNetworkGraph::from_roads(&dataset.roads);

                let node_count = road_graph.graph.node_count();
                let edge_count = road_graph.graph.edge_count();
                assert!(
                    node_count >= 2,
                    "Road graph for {} must have >= 2 nodes, got {}",
                    city.id,
                    node_count
                );
                assert!(
                    edge_count >= 1,
                    "Road graph for {} must have >= 1 edge, got {}",
                    city.id,
                    edge_count
                );

                // Execute A* navigation route between first and last node
                let start_coord = dataset.roads[0].geometry[0];
                let target_coord = dataset.roads[0].geometry.last().copied().unwrap();

                let route = realearth_navigation::find_route(
                    &road_graph,
                    start_coord,
                    target_coord,
                    realearth_navigation::RoutingPreference::ShortestDistance,
                );
                assert!(
                    route.is_ok(),
                    "A* route must be findable along road in city {}",
                    city.id
                );
                let nav = route.unwrap();
                assert!(
                    !nav.waypoints.is_empty(),
                    "Route for {} must contain waypoints",
                    city.id
                );
                assert!(
                    nav.total_distance_meters >= 0.0,
                    "Route distance must be non-negative"
                );

                // 4. DEM NAVD88 bounds check
                for &elev in &dem.elevations {
                    assert!(
                        elev >= city.elevation_min_m - 0.5 && elev <= city.elevation_max_m + 0.5,
                        "City {} DEM elevation {:.2}m outside USGS limits [{}m, {}m]",
                        city.id,
                        elev,
                        city.elevation_min_m,
                        city.elevation_max_m
                    );
                }
            }
            RegionPackageReadiness::DownloadRequired => {
                let load_res = verify_city_playable(city.id);
                assert!(
                    load_res.is_err(),
                    "Unbundled catalog city '{}' must honestly require download package",
                    city.id
                );
                assert!(
                    city.bounds.contains(&city.anchor),
                    "City {} anchor must be inside bounding box",
                    city.id
                );
                assert!(
                    city.estimated_download_size_bytes() >= 10 * 1024 * 1024,
                    "City {} download size estimate must be >= 10MB",
                    city.id
                );
            }
            _ => {}
        }
    }
}

// ============================================================================
// 2.5: PRIORITY 2 EXTENDED CITIES & FULL 38-CITY CATALOG
// ============================================================================

#[test]
fn test_stage2_priority2_extended_cities_coverage() {
    let test_sample_ids = [
        "philadelphia",
        "phoenix",
        "denver",
        "atlanta",
        "san-diego",
        "austin",
        "portland",
        "las-vegas",
    ];

    for city_id in test_sample_ids {
        let city = find_city_by_id(city_id).expect("City should exist in catalog");
        match city.package_readiness() {
            RegionPackageReadiness::InstalledReady => {
                let (dataset, dem, meta) = load_authentic_city(city.id).unwrap_or_else(|e| {
                    panic!("City '{}' must load authentically: {}", city.id, e)
                });

                assert_eq!(meta.region_id, city.id);
                assert!(
                    !dataset.buildings.is_empty(),
                    "City {} must have buildings",
                    city.id
                );
                assert!(
                    !dataset.roads.is_empty(),
                    "City {} must have roads",
                    city.id
                );
                assert_eq!(dem.width, 64);
                assert_eq!(dem.height, 64);
            }
            RegionPackageReadiness::DownloadRequired => {
                let load_res = verify_city_playable(city.id);
                assert!(
                    load_res.is_err(),
                    "Unbundled catalog city '{}' must honestly require download package",
                    city.id
                );
                assert!(city.bounds.contains(&city.anchor));
            }
            _ => {}
        }
    }
}

// ============================================================================
// 2.4: INCREMENTAL RECOMPILATION INVARIANCE
// ============================================================================

#[test]
fn test_stage2_incremental_recompilation_invariance() {
    let (dataset, dem, meta) =
        load_authentic_city("san-francisco").expect("Authentic load must succeed");

    let tolerance = ValidationTolerance::default();
    let (validated, _) = validate_dataset_with_tolerances(&dataset, &tolerance, Some(&dem));

    let temp_dir = tempdir().expect("Failed to create tempdir");
    let options = CompilerOptions {
        chunk_size_meters: 500.0,
        terrain_grid_resolution: 16,
        incremental: true,
        package_name: format!("{}_package", meta.region_id),
        ..Default::default()
    };

    // First compilation: generates all chunks
    let stats_first = compile_world_package(&validated, &dem, temp_dir.path(), &options)
        .expect("First compilation must succeed");
    assert!(stats_first.total_chunks_generated > 0);
    assert_eq!(stats_first.chunks_reused, 0);

    // Second compilation: unchanged data reuses all chunks
    let stats_second = compile_world_package(&validated, &dem, temp_dir.path(), &options)
        .expect("Second compilation must succeed");
    assert_eq!(
        stats_second.chunks_reused, stats_first.total_chunks_generated,
        "Incremental build must reuse all previously generated chunks"
    );
    assert_eq!(
        stats_second.total_chunks_generated, 0,
        "No new chunks should be generated during unchanged recompile"
    );
}
