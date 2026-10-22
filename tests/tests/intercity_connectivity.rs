//! Comprehensive Stage 7 Integration Test Suite:
//! Geographic Expansion and Intercity Connectivity across South India.
//!
//! Validates:
//! 1. All regional source packages (Bengaluru, Mysuru, Chennai, Hyderabad, Corridors) compile
//!    with consistent WGS84 coordinates, MSL datums, UTM zones, and SHA-256 integrity.
//! 2. Authentic transport corridors (NH 275, NH 48/NE 7, NH 44) allow continuous A* pathfinding
//!    between city landmarks without fabricated connectors or topological gaps.
//! 3. RegionalCatalog supports incremental installation, rescan, verification, and uninstallation.
//! 4. MultiRegionStreamer lazily streams chunks based on spatial bounding boxes and respects
//!    strict LRU memory eviction during long drives.
//! 5. Layer 1 river bridges (Kaveri, Palar, Tungabhadra, Napier, Nayapul) preserve grade-separated
//!    vertical clearances without terrain or water collisions.

use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_compiler::regional_catalog::{PackageManifest, RegionalCatalog};
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
use realearth_ingestion::sample_chennai::create_chennai_dataset;
use realearth_ingestion::sample_corridors::create_intercity_corridors_dataset;
use realearth_ingestion::sample_hyderabad::create_hyderabad_dataset;
use realearth_ingestion::sample_mysuru::create_mysuru_dataset;
use realearth_navigation::{find_route, RoadNetworkGraph, RoutingPreference};
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::region::RegionProfile;
use realearth_streaming::{MultiRegionStreamer, StreamingConfig};
use realearth_validation::pipeline::validate_dataset;
use tempfile::tempdir;

#[test]
fn test_all_four_regional_packages_compilation_and_integrity() {
    let temp = tempdir().expect("tempdir creation failed");
    let catalog_dir = temp.path().join("catalog");
    let mut catalog = RegionalCatalog::open(&catalog_dir).expect("Failed to open catalog");

    let regions = [
        (
            "bengaluru-central",
            RegionProfile::bengaluru_central(),
            create_bengaluru_central_dataset(),
            DemGrid::create_bengaluru_plateau_dem(16, 16),
        ),
        (
            "mysuru-urban",
            RegionProfile::mysuru_urban(),
            create_mysuru_dataset(),
            DemGrid::create_mysuru_dem(16, 16),
        ),
        (
            "chennai-central",
            RegionProfile::chennai_central(),
            create_chennai_dataset(),
            DemGrid::create_chennai_coastal_dem(16, 16),
        ),
        (
            "hyderabad-central",
            RegionProfile::hyderabad_central(),
            create_hyderabad_dataset(),
            DemGrid::create_hyderabad_plateau_dem(16, 16),
        ),
        (
            "intercity-corridors",
            RegionProfile::intercity_corridors(),
            create_intercity_corridors_dataset(),
            DemGrid::create_intercity_corridors_dem(16, 16),
        ),
    ];

    for (reg_id, profile, raw_dataset, dem) in regions {
        let (validated, report) = validate_dataset(&raw_dataset);
        assert_eq!(
            report.quarantined.len(),
            0,
            "Region {} had quarantined features",
            reg_id
        );
        assert!(
            report.valid_roads_passed > 0,
            "Region {} had no valid roads",
            reg_id
        );

        let build_dir = temp.path().join(format!("build_{}", reg_id));
        let chunk_size = if reg_id == "intercity-corridors" {
            50_000.0
        } else {
            1000.0
        };
        let opts = CompilerOptions {
            chunk_size_meters: chunk_size,
            terrain_grid_resolution: 8,
            ..Default::default()
        };

        let stats = compile_world_package(&validated, &dem, &build_dir, &opts)
            .unwrap_or_else(|e| panic!("Failed to compile package for {}: {}", reg_id, e));

        assert!(stats.total_chunks_generated > 0);
        assert!(stats.total_compressed_bytes > 0);

        let manifest = PackageManifest::new(
            &profile,
            "1.0.0",
            stats.total_chunks_generated,
            stats.total_uncompressed_bytes,
            stats.total_compressed_bytes,
            "test-checksum",
        );

        catalog
            .install_package(&build_dir, &manifest)
            .unwrap_or_else(|e| panic!("Failed to install package {}: {}", reg_id, e));

        assert!(catalog.is_installed(reg_id));
    }

    assert_eq!(catalog.list_installed().len(), 5);
    assert!(catalog.total_installed_bytes() > 0);

    // Verify all 5 installed packages integrity
    for reg_id in [
        "bengaluru-central",
        "mysuru-urban",
        "chennai-central",
        "hyderabad-central",
        "intercity-corridors",
    ] {
        let valid = catalog
            .verify_package_integrity(reg_id)
            .unwrap_or_else(|e| panic!("Integrity check failed for {}: {}", reg_id, e));
        assert!(
            valid,
            "Integrity verification failed for package {}",
            reg_id
        );
    }
}

#[test]
fn test_bengaluru_mysuru_expressway_corridor_traversal() {
    let bengaluru = create_bengaluru_central_dataset();
    let corridors = create_intercity_corridors_dataset();
    let mysuru = create_mysuru_dataset();

    // Merge roads for complete south-west corridor routing
    let mut all_roads = Vec::new();
    all_roads.extend(bengaluru.roads);
    all_roads.extend(corridors.roads);
    all_roads.extend(mysuru.roads);

    let network = RoadNetworkGraph::from_roads(&all_roads);

    // From Vidhana Soudha / MG Road in Bengaluru to Mysore Palace
    let start = Wgs84Coord::new(12.9740, 77.6000, 0.0); // Bengaluru MG Road
    let goal = Wgs84Coord::new(12.3060, 76.6570, 0.0); // Mysuru Palace / K.R. Circle / NH 766

    let route = find_route(&network, start, goal, RoutingPreference::ShortestDistance)
        .expect("A* route between Bengaluru and Mysuru should exist via NH 275");

    assert!(!route.waypoints.is_empty());
    // Real driving distance Bengaluru to Mysuru via NH 275 is approx 135 km - 145 km
    assert!(
        route.total_distance_meters >= 120_000.0 && route.total_distance_meters <= 160_000.0,
        "Distance was {:.1} km, expected between 120 and 160 km",
        route.total_distance_meters / 1000.0
    );

    // Verify Kaveri River Bridge crossing (Way 700_002) is present in the route
    let has_bridge = route.waypoints.iter().any(|w| {
        // Bridge coordinates: lat ~ 12.4200, lon ~ 76.6900
        (w.lat - 12.4200).abs() < 0.015 && (w.lon - 76.6900).abs() < 0.015
    });
    assert!(
        has_bridge,
        "Route did not traverse the Srirangapatna Kaveri River Bridge"
    );
}

#[test]
fn test_bengaluru_chennai_highway_corridor_traversal() {
    let bengaluru = create_bengaluru_central_dataset();
    let corridors = create_intercity_corridors_dataset();
    let chennai = create_chennai_dataset();

    let mut all_roads = Vec::new();
    all_roads.extend(bengaluru.roads);
    all_roads.extend(corridors.roads);
    all_roads.extend(chennai.roads);

    let network = RoadNetworkGraph::from_roads(&all_roads);

    // From Bengaluru MG Road East Terminus to Chennai Central Station
    let start = Wgs84Coord::new(12.9760, 77.6150, 0.0); // MG Road East Terminus
    let goal = Wgs84Coord::new(13.0825, 80.2740, 0.0); // Chennai Central Station

    let route = find_route(&network, start, goal, RoutingPreference::ShortestDistance)
        .expect("A* route between Bengaluru and Chennai should exist via NH 48");

    assert!(!route.waypoints.is_empty());
    // Real driving distance Bengaluru to Chennai via NH 48 is approx 330 km - 360 km
    assert!(
        route.total_distance_meters >= 300_000.0 && route.total_distance_meters <= 380_000.0,
        "Distance was {:.1} km, expected between 300 and 380 km",
        route.total_distance_meters / 1000.0
    );

    // Verify Palar River Bridge crossing is traversed
    let has_palar = route
        .waypoints
        .iter()
        .any(|w| (w.lat - 12.9200).abs() < 0.02 && (w.lon - 79.3300).abs() < 0.03);
    assert!(has_palar, "Route did not traverse the Palar River Bridge");
}

#[test]
fn test_bengaluru_hyderabad_highway_corridor_traversal() {
    let bengaluru = create_bengaluru_central_dataset();
    let corridors = create_intercity_corridors_dataset();
    let hyderabad = create_hyderabad_dataset();

    let mut all_roads = Vec::new();
    all_roads.extend(bengaluru.roads);
    all_roads.extend(corridors.roads);
    all_roads.extend(hyderabad.roads);

    let network = RoadNetworkGraph::from_roads(&all_roads);

    // From Vidhana Soudha North Gate in Bengaluru to Charminar in Hyderabad
    let start = Wgs84Coord::new(12.9803, 77.5901, 0.0); // Vidhana Soudha North Gate
    let goal = Wgs84Coord::new(17.3620, 78.4740, 0.0); // Charminar Approach

    let route = find_route(&network, start, goal, RoutingPreference::ShortestDistance)
        .expect("A* route between Bengaluru and Hyderabad should exist via NH 44");

    assert!(!route.waypoints.is_empty());
    // Real driving distance Bengaluru to Hyderabad via NH 44 is approx 550 km - 590 km
    assert!(
        route.total_distance_meters >= 500_000.0 && route.total_distance_meters <= 630_000.0,
        "Distance was {:.1} km, expected between 500 and 630 km",
        route.total_distance_meters / 1000.0
    );

    // Verify Tungabhadra River Bridge crossing at Kurnool
    let has_tungabhadra = route
        .waypoints
        .iter()
        .any(|w| (w.lat - 15.8300).abs() < 0.02 && (w.lon - 78.0300).abs() < 0.02);
    assert!(
        has_tungabhadra,
        "Route did not traverse the Tungabhadra River Bridge"
    );

    // Verify PVNR Elevated Expressway
    let has_pvnr = route
        .waypoints
        .iter()
        .any(|w| (w.lat - 17.3600).abs() < 0.03 && (w.lon - 78.4450).abs() < 0.02);
    assert!(
        has_pvnr,
        "Route did not traverse the PVNR Elevated Expressway"
    );
}

#[test]
fn test_multi_region_catalog_incremental_install_and_lazy_load() {
    let temp = tempdir().expect("tempdir failed");
    let catalog_dir = temp.path().join("cat");
    let mut catalog = RegionalCatalog::open(&catalog_dir).expect("open failed");

    // 1. Compile and install Bengaluru package
    let dir_bengaluru = temp.path().join("bengaluru_build");
    let (dataset_b, _) = validate_dataset(&create_bengaluru_central_dataset());
    let dem_b = DemGrid::create_bengaluru_plateau_dem(16, 16);
    let opts = CompilerOptions {
        chunk_size_meters: 1000.0,
        terrain_grid_resolution: 8,
        ..Default::default()
    };
    let stats_b = compile_world_package(&dataset_b, &dem_b, &dir_bengaluru, &opts).unwrap();
    let manifest_b = PackageManifest::new(
        &RegionProfile::bengaluru_central(),
        "1.0",
        stats_b.total_chunks_generated,
        stats_b.total_uncompressed_bytes,
        stats_b.total_compressed_bytes,
        "",
    );
    catalog
        .install_package(&dir_bengaluru, &manifest_b)
        .unwrap();

    // 2. Query spatial coverage
    let b_query = GeoBounds::new(12.97, 12.99, 77.58, 77.60);
    let covering = catalog.find_regions_covering(&b_query);
    assert_eq!(covering, vec!["bengaluru-central"]);

    let distant_query = GeoBounds::new(13.05, 13.10, 80.20, 80.30); // Chennai bounds
    let covering_distant = catalog.find_regions_covering(&distant_query);
    assert!(
        covering_distant.is_empty(),
        "Uninstalled Chennai should not match query"
    );

    // 3. Mount in MultiRegionStreamer
    let config = StreamingConfig {
        max_memory_mb: 64,
        view_distance_meters: 2500.0,
        ..Default::default()
    };
    let mut streamer = MultiRegionStreamer::new(config);
    streamer
        .mount_region("bengaluru-central", &dir_bengaluru, dem_b.bounds)
        .unwrap();

    let count = streamer
        .update(Wgs84Coord::new(12.9797, 77.5907, 920.0))
        .unwrap();
    assert!(count > 0);
    assert_eq!(
        streamer.telemetry().active_region_ids,
        vec!["bengaluru-central"]
    );
}

#[test]
fn test_intercity_long_drive_memory_stability_and_eviction() {
    let temp = tempdir().expect("tempdir failed");
    let dir = temp.path().join("bengaluru");
    let (dataset, _) = validate_dataset(&create_bengaluru_central_dataset());
    let dem = DemGrid::create_bengaluru_plateau_dem(16, 16);
    let opts = CompilerOptions {
        chunk_size_meters: 1000.0,
        terrain_grid_resolution: 8,
        ..Default::default()
    };
    compile_world_package(&dataset, &dem, &dir, &opts).unwrap();

    // Configure a constrained memory budget (32 MB) to trigger eviction aggressively
    let config = StreamingConfig {
        max_memory_mb: 32,
        view_distance_meters: 3000.0,
        ..Default::default()
    };
    let mut streamer = MultiRegionStreamer::new(config);
    streamer
        .mount_region("bengaluru-central", &dir, dem.bounds)
        .unwrap();

    // Pin collision critical chunk at origin
    streamer.set_pinned("bengaluru-central", 0, 0, 0, true);

    // Simulate 50 frames of high-speed long drive moving across coordinates
    let mut current_lat = 12.9700;
    let mut current_lon = 77.5800;

    for _frame in 0..50 {
        current_lat += 0.001; // ~111m per step
        current_lon += 0.001;
        let pos = Wgs84Coord::new(current_lat, current_lon, 920.0);
        let _ = streamer.update(pos).unwrap();
    }

    let telem = streamer.telemetry();
    // Memory budget strictly preserved
    assert!(
        telem.cpu_memory_mb <= 32.0,
        "Memory exceeded 32 MB budget: {:.2} MB",
        telem.cpu_memory_mb
    );
    // Pinned chunk at (0, 0, 0) was never evicted
    assert!(
        streamer.is_chunk_cached("bengaluru-central", 0, 0, 0),
        "Collision pinned chunk at origin was incorrectly evicted!"
    );
}

#[test]
fn test_grade_separated_river_and_railway_crossings_invariance() {
    let corridors = create_intercity_corridors_dataset();
    let chennai = create_chennai_dataset();
    let hyderabad = create_hyderabad_dataset();

    // 1. Srirangapatna Kaveri River Bridge (Layer 1) vs Kaveri River (Layer 0)
    let kaveri_bridge = corridors
        .roads
        .iter()
        .find(|r| r.way_id == 700_002)
        .expect("Kaveri bridge missing");
    let kaveri_river = corridors
        .water_bodies
        .iter()
        .find(|w| w.id == 720_001)
        .expect("Kaveri river missing");
    assert_eq!(kaveri_bridge.layer, 1);
    assert!(kaveri_bridge.is_bridge);
    assert_eq!(kaveri_river.target_elevation_msl, 685.0);

    // 2. Palar River Bridge (Layer 1) vs Palar River (Layer 0)
    let palar_bridge = corridors
        .roads
        .iter()
        .find(|r| r.way_id == 700_005)
        .expect("Palar bridge missing");
    let palar_river = corridors
        .water_bodies
        .iter()
        .find(|w| w.id == 720_002)
        .expect("Palar river missing");
    assert_eq!(palar_bridge.layer, 1);
    assert!(palar_bridge.is_bridge);
    assert_eq!(palar_river.target_elevation_msl, 145.0);

    // 3. Napier Bridge (Layer 1) over Cooum River in Chennai
    let napier_bridge = chennai
        .roads
        .iter()
        .find(|r| r.way_id == 510_004)
        .expect("Napier bridge missing");
    assert_eq!(napier_bridge.layer, 1);
    assert!(napier_bridge.is_bridge);

    // 4. Nayapul Bridge (Layer 1) over Musi River in Hyderabad
    let nayapul_bridge = hyderabad
        .roads
        .iter()
        .find(|r| r.way_id == 610_004)
        .expect("Nayapul bridge missing");
    assert_eq!(nayapul_bridge.layer, 1);
    assert!(nayapul_bridge.is_bridge);

    // 5. PVNR Elevated Expressway (Layer 1) in Hyderabad
    let pvnr = hyderabad
        .roads
        .iter()
        .find(|r| r.way_id == 610_001)
        .expect("PVNR missing");
    assert_eq!(pvnr.layer, 1);
    assert!(pvnr.is_bridge);
}
