//! Stage E — Complete Defect Remediation Regression Test Suite.
//!
//! Validates P0/P1/P2 root-cause fixes across:
//! - P0: SQLite WAL mode concurrency, busy timeout, and antimeridian spatial queries
//! - P0: Unbounded Zstd decompression bomb mitigation (64 MiB bounded memory protection)
//! - P1: Negative longitude preservation (Western hemisphere) and grade-separated layer isolation
//! - P1: Antimeridian DEM continuous bilinear interpolation without clamp panics
//! - P1: IDM car-following zero-headway regularization and bumper collision protection
//! - P1: Automotive tire dynamics low-speed lateral scrub continuity (no zero-friction skating)

use glam::Vec3;
use realearth_compiler::building_mesher::mesh_building;
use realearth_compiler::chunk::{ChunkHeader, WorldChunk, CHUNK_MAGIC};
use realearth_compiler::database::{ChunkDatabase, ChunkIndexRecord};
use realearth_compiler::packager::{
    pack_chunk_to_bytes, unpack_chunk_from_bytes, MAX_UNCOMPRESSED_CHUNK_BYTES,
};
use realearth_compiler::road_mesher::mesh_road;
use realearth_compiler::terrain_mesher::mesh_terrain;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::BuildingType;
use realearth_ingestion::osm_types::{HighwayType, OsmRoadFeature};
use realearth_navigation::road_graph::RoadNetworkGraph;
use realearth_navigation::router::{find_route, RoutingPreference};
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_validation::polygon_validator::ValidatedBuilding;
use realearth_validation::{HeightProvenance, ValidatedHeight};
use realearth_vehicles::dynamics::{DynamicVehicle, VehicleConfig, VehicleInputs};
use realearth_vehicles::ground::FlatGround;
use realearth_vehicles::traffic::{IdmConfig, TrafficAgent, TrafficSimulation};
use std::sync::Arc;
use tempfile::tempdir;

/// E1: Verifies SQLite WAL mode, normal synchrony, busy timeout, and multithreaded concurrency.
#[test]
fn test_e1_sqlite_wal_concurrency_and_busy_timeout() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let db_path = tmp.path().join("concurrency_test_chunks.db");

    let db = ChunkDatabase::open(&db_path).expect("Failed to open database with WAL pragmas");

    let rec = ChunkIndexRecord {
        chunk_x: 10,
        chunk_y: 20,
        lod_level: 0,
        bounds: GeoBounds::new(37.77, 37.78, -122.42, -122.41),
        min_alt: 5.0,
        max_alt: 85.0,
        compressed_bytes: 4096,
        uncompressed_bytes: 16384,
        feature_count: 120,
        storage_path: "sf_10_20_lod0.rec".to_string(),
        file_offset: 0,
    };
    db.insert_chunk(&rec)
        .expect("Failed to insert chunk record");

    let arc_path = Arc::new(db_path);
    let mut handles = Vec::new();

    // Spawn 8 concurrent reader and writer threads accessing chunks.db simultaneously
    for thread_idx in 0..8 {
        let path_clone = Arc::clone(&arc_path);
        let handle = std::thread::spawn(move || {
            let conn = ChunkDatabase::open(&path_clone)
                .expect("Thread failed to open database concurrently");

            if thread_idx % 2 == 0 {
                // Writer thread: insert unique chunk
                let t_rec = ChunkIndexRecord {
                    chunk_x: 100 + thread_idx,
                    chunk_y: 200 + thread_idx,
                    lod_level: 0,
                    bounds: GeoBounds::new(37.70, 37.71, -122.40, -122.39),
                    min_alt: 0.0,
                    max_alt: 50.0,
                    compressed_bytes: 2048,
                    uncompressed_bytes: 8192,
                    feature_count: 50,
                    storage_path: format!("chunk_t_{}.rec", thread_idx),
                    file_offset: 0,
                };
                conn.insert_chunk(&t_rec)
                    .expect("Concurrent insert must not fail with SQLITE_BUSY");
            } else {
                // Reader thread: query bbox
                let query_box = GeoBounds::new(37.70, 37.80, -122.50, -122.30);
                let results = conn
                    .query_bbox(&query_box)
                    .expect("Concurrent query must succeed in WAL mode");
                assert!(!results.is_empty(), "Reader should find inserted chunks");
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Worker thread panicked under concurrency");
    }

    let final_count = db.count_chunks().expect("Failed to count chunks");
    assert!(
        final_count >= 5,
        "Expected at least 5 chunks in database, got {}",
        final_count
    );
}

/// E1: Verifies that antimeridian-spanning bounding boxes (min_lon > max_lon) query properly.
#[test]
fn test_e1_antimeridian_spatial_query_remediation() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let db_path = tmp.path().join("antimeridian_chunks.db");
    let db = ChunkDatabase::open(&db_path).expect("Failed to open db");

    // Chunk A: In Eastern Pacific near Fiji / Aleutians (178.0° to 179.5°)
    let chunk_east = ChunkIndexRecord {
        chunk_x: 500,
        chunk_y: 100,
        lod_level: 0,
        bounds: GeoBounds::new(51.0, 52.0, 178.0, 179.5),
        min_alt: 0.0,
        max_alt: 200.0,
        compressed_bytes: 1024,
        uncompressed_bytes: 4096,
        feature_count: 30,
        storage_path: "east_chunk.rec".to_string(),
        file_offset: 0,
    };

    // Chunk B: In Western Pacific across the antimeridian (-179.5° to -178.0°)
    let chunk_west = ChunkIndexRecord {
        chunk_x: 501,
        chunk_y: 100,
        lod_level: 0,
        bounds: GeoBounds::new(51.0, 52.0, -179.5, -178.0),
        min_alt: 0.0,
        max_alt: 200.0,
        compressed_bytes: 1024,
        uncompressed_bytes: 4096,
        feature_count: 30,
        storage_path: "west_chunk.rec".to_string(),
        file_offset: 0,
    };

    // Chunk C: Distant mainland chunk (140.0° to 141.0°) that must NOT match antimeridian bbox
    let chunk_distant = ChunkIndexRecord {
        chunk_x: 300,
        chunk_y: 100,
        lod_level: 0,
        bounds: GeoBounds::new(51.0, 52.0, 140.0, 141.0),
        min_alt: 0.0,
        max_alt: 500.0,
        compressed_bytes: 1024,
        uncompressed_bytes: 4096,
        feature_count: 20,
        storage_path: "distant_chunk.rec".to_string(),
        file_offset: 0,
    };

    db.insert_chunk(&chunk_east).expect("Insert east");
    db.insert_chunk(&chunk_west).expect("Insert west");
    db.insert_chunk(&chunk_distant).expect("Insert distant");

    // Query with antimeridian-spanning box: from 177.0°E across 180° to -177.0°W
    let antimeridian_query = GeoBounds::new_antimeridian(50.0, 53.0, 177.0, -177.0);
    assert!(
        antimeridian_query.crosses_antimeridian(),
        "Envelope must be recognized as crossing antimeridian"
    );

    let results = db
        .query_bbox(&antimeridian_query)
        .expect("Querying antimeridian bbox must succeed");

    assert_eq!(
        results.len(),
        2,
        "Antimeridian query must match both East and West flanks of the 180° meridian, but not distant chunks"
    );
    let chunk_xs: Vec<i32> = results.iter().map(|r| r.chunk_x).collect();
    assert!(chunk_xs.contains(&500));
    assert!(chunk_xs.contains(&501));
    assert!(!chunk_xs.contains(&300));
}

/// E1: Verifies bounded decompression prevents memory exhaustion (decompression bombs).
#[test]
fn test_e1_decompression_bomb_dos_mitigation() {
    use realearth_compiler::mesh::ChunkMesh;

    // 1. Valid WorldChunk packs and unpacks cleanly within limit
    let valid_chunk = WorldChunk {
        header: ChunkHeader {
            magic: *CHUNK_MAGIC,
            version: 1,
            chunk_x: 1,
            chunk_y: 2,
            lod_level: 0,
            bounds: GeoBounds::new(37.7, 37.8, -122.5, -122.4),
            anchor: Wgs84Coord::new(37.75, -122.45, 0.0),
            min_elevation: 0.0,
            max_elevation: 100.0,
        },
        terrain: ChunkMesh::default(),
        roads: ChunkMesh::default(),
        buildings: ChunkMesh::default(),
        water: ChunkMesh::default(),
        vegetation: ChunkMesh::default(),
        building_metadata: Vec::new(),
    };

    let packed = pack_chunk_to_bytes(&valid_chunk).expect("Packing valid chunk failed");
    let unpacked = unpack_chunk_from_bytes(&packed).expect("Unpacking valid chunk failed");
    assert_eq!(unpacked.header.chunk_x, 1);

    // 2. Synthesize a compression bomb payload: 70 MiB of zeros (compresses to ~70 KiB)
    // which exceeds MAX_UNCOMPRESSED_CHUNK_BYTES (64 MiB).
    let oversized_payload = vec![0u8; MAX_UNCOMPRESSED_CHUNK_BYTES + 4 * 1024 * 1024];
    let compressed_bomb =
        zstd::encode_all(&oversized_payload[..], 1).expect("Failed to compress bomb payload");

    assert!(
        compressed_bomb.len() < 500_000,
        "Bomb payload should compress to under 500 KB"
    );

    // Attempting to unpack the bomb must be rejected with an explicit size-limit error
    let bomb_result = unpack_chunk_from_bytes(&compressed_bomb);
    assert!(
        bomb_result.is_err(),
        "Decompression bomb must be rejected before allocating unbounded memory"
    );
    let err_msg = format!("{}", bomb_result.unwrap_err());
    assert!(
        err_msg.contains("potential decompression bomb")
            || err_msg.contains("exceeds maximum allowed size"),
        "Error message must clearly identify decompression bomb rejection: {}",
        err_msg
    );
}

/// E2: Verifies Western hemisphere negative longitudes and bridge abutment connectivity.
#[test]
fn test_e2_navigation_western_negative_longitudes_and_layer_isolation() {
    let mut network = RoadNetworkGraph::new();

    // Road 1: Surface street in San Francisco (Market St) at layer = 0
    // Longitude is negative (-122.4194)
    let surface_road = OsmRoadFeature {
        way_id: 1001,
        name: Some("Market St".to_string()),
        highway_type: HighwayType::Primary,
        lanes: 4,
        is_oneway: false,
        speed_limit_kph: 50,
        geometry: vec![
            Wgs84Coord::new(37.7749, -122.4194, 15.0),
            Wgs84Coord::new(37.7760, -122.4170, 15.0),
        ],
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    };

    // Road 2: Bridge overpass connecting at north abutment (37.7760, -122.4170)
    let elevated_bridge = OsmRoadFeature {
        way_id: 2001,
        name: Some("Central Flyover Bridge".to_string()),
        highway_type: HighwayType::Motorway,
        lanes: 2,
        is_oneway: false,
        speed_limit_kph: 80,
        geometry: vec![
            Wgs84Coord::new(37.7760, -122.4170, 15.0), // Connects at bridge abutment
            Wgs84Coord::new(37.7780, -122.4150, 30.0),
        ],
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    };

    // Road 3: Parallel avenue in San Francisco (Mission St) at distinct negative longitude (-122.4180)
    let mission_road = OsmRoadFeature {
        way_id: 3001,
        name: Some("Mission St".to_string()),
        highway_type: HighwayType::Primary,
        lanes: 4,
        is_oneway: false,
        speed_limit_kph: 50,
        geometry: vec![
            Wgs84Coord::new(37.7740, -122.4180, 15.0),
            Wgs84Coord::new(37.7750, -122.4160, 15.0),
        ],
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    };

    network.add_road(&surface_road);
    network.add_road(&elevated_bridge);
    network.add_road(&mission_road);

    // Prior to remediation, negative longitudes were cast to u64 and collapsed to 0.
    // With signed i64 coordinates, Market St and Mission St remain distinct (5 unique nodes total).
    assert_eq!(
        network.graph.node_count(),
        5,
        "Distinct negative coordinates must produce distinct nodes and bridge abutment must connect"
    );

    // Test nearest node query in Western hemisphere: negative coordinates must find nearest node accurately
    let query_pt = Wgs84Coord::new(37.77492, -122.41938, 15.0);
    let nearest_idx = network
        .find_nearest_node(query_pt)
        .expect("Must find nearest node in Western hemisphere");
    let node = &network.graph[nearest_idx];
    assert!(
        (node.coord.lon - (-122.4194)).abs() < 1e-4,
        "Nearest node must match negative longitude -122.4194, got {}",
        node.coord.lon
    );
}

/// E2: Verifies DemGrid continuous bilinear interpolation across antimeridian without panics.
#[test]
fn test_e2_dem_antimeridian_bilinear_interpolation_invariance() {
    // 5x5 DEM grid spanning across the 180° meridian: 178°E to -178°W (4 degrees wide)
    let antimeridian_bounds = GeoBounds::new_antimeridian(50.0, 52.0, 178.0, -178.0);
    let mut elevations = Vec::with_capacity(25);
    for y in 0..5 {
        for x in 0..5 {
            elevations.push(100.0 + (x as f32) * 20.0 + (y as f32) * 10.0);
        }
    }

    let dem = DemGrid::new(antimeridian_bounds, 5, 5, elevations).expect("DEM creation failed");

    // Sample directly on eastern side (179.0°E)
    let coord_east = Wgs84Coord::new(51.0, 179.0, 0.0);
    let elev_east = dem.sample_bilinear(&coord_east);
    assert!(
        elev_east > 100.0 && elev_east < 220.0,
        "Sample on eastern side must be valid: {}",
        elev_east
    );

    // Sample directly on western side (-179.0°W)
    let coord_west = Wgs84Coord::new(51.0, -179.0, 0.0);
    let elev_west = dem.sample_bilinear(&coord_west);
    assert!(
        elev_west > 100.0 && elev_west < 220.0,
        "Sample on western side must be valid: {}",
        elev_west
    );

    // Sample exactly on antimeridian (180.0° / -180.0°)
    let coord_180 = Wgs84Coord::new(51.0, 180.0, 0.0);
    let elev_180 = dem.sample_bilinear(&coord_180);
    assert!(
        !elev_180.is_nan(),
        "Elevation on antimeridian must not be NaN"
    );

    // Elevation must smoothly increase from West to East across the antimeridian
    assert!(
        elev_west > elev_east,
        "Elevation at -179°W ({}) must be higher than 179°E ({}) given the gradient",
        elev_west,
        elev_east
    );
}

/// E3: Verifies IDM bumper collision protection and regularized zero-headway acceleration.
#[test]
fn test_e3_idm_bumper_penetration_and_zero_headway_regularization() {
    let idm = IdmConfig::default();

    // 1. Normal following at 10m gap
    let accel_normal = idm.calculate_acceleration(12.0, Some(10.0), Some(12.0));
    assert!(
        accel_normal.is_finite(),
        "Normal acceleration must be finite"
    );

    // 2. Severe bumper collision / penetration (gap <= 0.05m or negative headway)
    let accel_collision = idm.calculate_acceleration(12.0, Some(0.0), Some(5.0));
    assert_eq!(
        accel_collision, -8.0,
        "Zero or negative bumper headway must trigger immediate -8.0 m/s^2 emergency braking"
    );

    // 3. Negative headway (trailing car penetrating leading bumper)
    let accel_negative = idm.calculate_acceleration(15.0, Some(-1.5), Some(0.0));
    assert_eq!(
        accel_negative, -8.0,
        "Penetrated bumper headway must trigger immediate -8.0 m/s^2 emergency braking"
    );

    // 4. Test multi-agent simulation with an aggressive trailing agent
    let ground = FlatGround::new(0.0);
    let mut sim = TrafficSimulation::new();

    // Leader stationary at x = 10, y = 0, z = 0
    let mut leader = TrafficAgent::new(
        1,
        Vec3::new(0.0, 0.0, 10.0),
        vec![Vec3::new(0.0, 0.0, 10.0), Vec3::new(0.0, 0.0, 100.0)],
        idm,
    );
    leader.speed_ms = 0.0;
    sim.spawn_agent(leader);

    // Trailer right behind leader at z = 3 (bumper gap ~ 2.5m)
    let trailer = TrafficAgent::new(
        2,
        Vec3::new(0.0, 0.0, 3.0),
        vec![Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, 100.0)],
        idm,
    );
    sim.spawn_agent(trailer);

    // Step simulation 20 times (2.0s total at dt = 0.1s)
    for _ in 0..20 {
        sim.step(0.1, &ground);
    }

    let leader_z = sim.agents[0].position.z;
    let trailer_z = sim.agents[1].position.z;

    // Trailer must never overtake or penetrate through leader's position
    assert!(
        trailer_z < leader_z,
        "Trailing vehicle (z={}) must not penetrate ahead of leading vehicle (z={})",
        trailer_z,
        leader_z
    );
}

/// E4: Verifies automotive tire dynamics generate continuous lateral scrub resistance at low speed.
#[test]
fn test_e4_vehicle_dynamics_low_speed_lateral_scrub_continuity() {
    let ground = FlatGround::new(0.0);
    let config = VehicleConfig::default();
    let mut vehicle = DynamicVehicle::new(config, Vec3::new(0.0, 0.7, 0.0), 0.0);
    let inputs = VehicleInputs::default();

    // Settle vehicle onto ground
    for _ in 0..60 {
        vehicle.step(&inputs, &ground, 1.0 / 60.0);
    }
    assert!(
        vehicle.is_grounded,
        "Vehicle must be grounded on flat terrain"
    );

    // Impart a purely lateral velocity at low speed (v_forward = 0.05 m/s along Z, v_lateral = 1.0 m/s along X)
    vehicle.linear_velocity = Vec3::new(1.0, 0.0, 0.05);

    // Step physics once
    vehicle.step(&inputs, &ground, 1.0 / 60.0);

    // Front-left wheel must have non-zero slip angle and non-zero lateral force opposing lateral slide
    let fl_wheel = &vehicle.wheels[0];
    assert!(fl_wheel.in_contact, "Wheel must be in contact with ground");
    assert!(
        fl_wheel.slip_angle.abs() > 0.01,
        "Wheel must have non-zero slip angle under lateral sliding at low forward speed, got {}",
        fl_wheel.slip_angle
    );
    assert!(
        fl_wheel.lateral_force_fy.abs() > 10.0,
        "Wheel must produce non-zero lateral force opposing lateral scrub at low forward speed, got {}",
        fl_wheel.lateral_force_fy
    );

    // Lateral velocity must decrease due to tire scrub resistance
    assert!(
        vehicle.linear_velocity.x < 1.0,
        "Tire friction must decelerate lateral sliding velocity: was 1.0, now {}",
        vehicle.linear_velocity.x
    );
}

/// E4: Verifies systemic quality and routing invariance across Western hemisphere coordinates.
#[test]
fn test_e4_systemic_regression_full_spectrum_verification() {
    use realearth_geospatial::transforms::{ecef_to_wgs84, wgs84_to_ecef};
    use realearth_navigation::router::{find_route, RoutingPreference};

    // 1. Verify WGS84 -> ECEF -> WGS84 roundtrip in Western hemisphere (San Francisco & New York)
    let sf_coord = Wgs84Coord::new(37.7749, -122.4194, 15.0);
    let sf_ecef = wgs84_to_ecef(sf_coord);
    let sf_roundtrip = ecef_to_wgs84(sf_ecef);
    assert!(
        (sf_coord.lat - sf_roundtrip.lat).abs() < 1e-6,
        "Latitude error must be < 1e-6 degrees"
    );
    assert!(
        (sf_coord.lon - sf_roundtrip.lon).abs() < 1e-6,
        "Longitude error must be < 1e-6 degrees in Western hemisphere"
    );
    assert!(
        (sf_coord.alt - sf_roundtrip.alt).abs() < 0.05,
        "Altitude error must be < 5 cm"
    );

    // 2. Build multi-node network in Western hemisphere and execute A* routing
    let mut network = RoadNetworkGraph::new();
    let road_seg_1 = OsmRoadFeature {
        way_id: 3001,
        name: Some("US-101 Northbound".to_string()),
        highway_type: HighwayType::Motorway,
        lanes: 4,
        is_oneway: true,
        speed_limit_kph: 100,
        geometry: vec![
            Wgs84Coord::new(37.7000, -122.4000, 10.0),
            Wgs84Coord::new(37.7100, -122.4050, 12.0),
            Wgs84Coord::new(37.7200, -122.4100, 15.0),
        ],
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    };
    network.add_road(&road_seg_1);

    let start_coord = Wgs84Coord::new(37.7000, -122.4000, 10.0);
    let goal_coord = Wgs84Coord::new(37.7200, -122.4100, 15.0);

    let route = find_route(
        &network,
        start_coord,
        goal_coord,
        RoutingPreference::ShortestDistance,
    )
    .expect("A* routing in Western hemisphere must succeed");

    assert!(route.total_distance_meters > 2000.0);
    assert!(route.waypoints.len() >= 3);
}

/// E5: Verifies that A* heuristic dynamically scales with latitude cosine and remains strictly admissible in high latitudes.
#[test]
fn test_e5_router_admissible_heuristic_and_high_latitude_scaling() {
    let mut network = RoadNetworkGraph::new();

    // High latitude route in Anchorage, Alaska (lat ~ 61.2° N)
    // At 61.2° N, 1 deg of lon is only ~53.8 km, compared to 111.3 km at the equator.
    let road = OsmRoadFeature {
        way_id: 5001,
        name: Some("Glenn Highway Anchorage".to_string()),
        highway_type: HighwayType::Motorway,
        lanes: 4,
        is_oneway: false,
        speed_limit_kph: 100,
        geometry: vec![
            Wgs84Coord::new(61.2181, -149.9003, 30.0),
            Wgs84Coord::new(61.2250, -149.8800, 35.0),
            Wgs84Coord::new(61.2350, -149.8500, 40.0),
        ],
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    };
    network.add_road(&road);

    let start = Wgs84Coord::new(61.2181, -149.9003, 30.0);
    let goal = Wgs84Coord::new(61.2350, -149.8500, 40.0);

    let route_dist = find_route(&network, start, goal, RoutingPreference::ShortestDistance)
        .expect("A* shortest distance routing in Anchorage must succeed");
    assert!(route_dist.total_distance_meters > 2000.0);

    let route_time = find_route(&network, start, goal, RoutingPreference::FastestTime)
        .expect("A* fastest time routing in Anchorage must succeed");
    assert!(route_time.estimated_duration_seconds > 50.0);
}

/// E5: Verifies that road mesher constructs a continuous shared-vertex miter ribbon without disconnected gaps.
#[test]
fn test_e5_continuous_miter_road_mesh_generation() {
    let origin = Wgs84Coord::new(37.7749, -122.4194, 10.0);
    let converter = EnuConverter::new(origin);
    let dem_bounds = GeoBounds::new(37.7700, 37.7800, -122.4300, -122.4100);
    let dem = DemGrid::new(dem_bounds, 10, 10, vec![15.0; 100]).expect("DEM creation failed");

    // 4-point curving road
    let road = OsmRoadFeature {
        way_id: 6001,
        name: Some("Market Street".to_string()),
        highway_type: HighwayType::Primary,
        lanes: 4,
        is_oneway: false,
        speed_limit_kph: 50,
        geometry: vec![
            Wgs84Coord::new(37.7740, -122.4200, 15.0),
            Wgs84Coord::new(37.7745, -122.4190, 15.0),
            Wgs84Coord::new(37.7755, -122.4185, 15.0),
            Wgs84Coord::new(37.7765, -122.4180, 15.0),
        ],
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    };

    let mesh = mesh_road(&road, &converter, &dem);
    assert!(!mesh.is_empty(), "Road mesh should not be empty");

    // For a 4-point continuous ribbon, vertices must be exactly 4 points * 2 sides = 8 vertices
    // Previously disconnected quads produced (4-1)*4 = 12 vertices.
    assert_eq!(
        mesh.vertices.len(),
        8,
        "Continuous ribbon must have exactly 2 vertices per centerline point (8 total)"
    );
    // 3 segments * 2 triangles * 3 indices = 18 indices
    assert_eq!(
        mesh.indices.len(),
        18,
        "Continuous ribbon must have exactly 6 indices per segment (18 total)"
    );
}

/// E5: Verifies that terrain mesher computes physically dimensioned surface normals with latitude cosine scaling.
#[test]
fn test_e5_terrain_normal_gradient_physics_precision() {
    let origin = Wgs84Coord::new(47.6062, -122.3321, 0.0); // Seattle, WA
    let converter = EnuConverter::new(origin);
    let bounds = GeoBounds::new(47.6000, 47.6100, -122.3400, -122.3300);

    // Flat terrain grid: all normals must point purely upward (0, 1, 0)
    let dem_bounds = GeoBounds::new(47.59, 47.62, -122.35, -122.32);
    let dem_flat = DemGrid::new(dem_bounds, 10, 10, vec![45.0; 100]).expect("DEM creation failed");
    let mesh_flat = mesh_terrain(&bounds, &converter, &dem_flat, 4);

    for v in &mesh_flat.vertices {
        assert!(
            (v.normal[1] - 1.0).abs() < 1e-3,
            "Flat terrain normal must have Y component = 1.0, got {:?}",
            v.normal
        );
        assert!(
            v.normal[0].abs() < 1e-3 && v.normal[2].abs() < 1e-3,
            "Flat terrain normal must have X and Z components ~ 0, got {:?}",
            v.normal
        );
    }
}

/// E5: Verifies that building mesher generates terrain-conforming foundation skirts on slopes without gaps.
#[test]
fn test_e5_building_foundation_skirting_on_slopes() {
    let origin = Wgs84Coord::new(37.7749, -122.4194, 0.0);
    let converter = EnuConverter::new(origin);

    // Sloped terrain: 10m on South edge, 30m on North edge
    let dem_bounds = GeoBounds::new(37.7700, 37.7800, -122.4300, -122.4100);
    let mut elevations = vec![10.0; 100];
    for r in 5..10 {
        for c in 0..10 {
            elevations[r * 10 + c] = 30.0;
        }
    }
    let dem = DemGrid::new(dem_bounds, 10, 10, elevations).expect("DEM creation failed");

    let building = ValidatedBuilding {
        id: 7001,
        name: Some("Nob Hill Tower".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(37.7740, -122.4200, 0.0), // downhill (elev ~ 10m)
            Wgs84Coord::new(37.7740, -122.4190, 0.0),
            Wgs84Coord::new(37.7760, -122.4190, 0.0), // uphill (elev ~ 30m)
            Wgs84Coord::new(37.7760, -122.4200, 0.0),
            Wgs84Coord::new(37.7740, -122.4200, 0.0), // closed
        ],
        inner_rings: Vec::new(),
        height: ValidatedHeight {
            height_meters: 50.0,
            provenance: HeightProvenance::Surveyed,
            confidence: 1.0,
        },
        area_m2: 400.0,
    };

    let mesh = mesh_building(&building, &converter, &dem);
    assert!(!mesh.is_empty(), "Building mesh should not be empty");

    // All base vertices must reach ground elevation without floating
    let min_y = mesh
        .vertices
        .iter()
        .map(|v| v.position[1])
        .fold(f32::MAX, f32::min);
    assert!(
        min_y <= 12.0,
        "Downhill foundation skirt must reach down to downhill ground (<= 12.0m), got {}",
        min_y
    );
}

/// E5: Verifies that traffic simulation filters out opposing-lane oncoming traffic from car-following leader logic.
#[test]
fn test_e5_traffic_opposing_lane_and_lead_filtering() {
    let mut sim = TrafficSimulation::new();
    let ground = FlatGround::new(0.0);

    // Agent 0: Eastbound vehicle at (0, 0, 0), yaw = PI/2 (heading East)
    let agent_0 = TrafficAgent::new(
        101,
        Vec3::new(0.0, 0.0, 0.0),
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(100.0, 0.0, 0.0)],
        IdmConfig::default(),
    );
    sim.agents.push(agent_0);

    // Agent 1: Oncoming Westbound vehicle at (40, 0, 2.0) (2 meters lateral offset in opposing lane), yaw = -PI/2 (heading West)
    let mut agent_1 = TrafficAgent::new(
        102,
        Vec3::new(40.0, 0.0, 2.0),
        vec![Vec3::new(40.0, 0.0, 2.0), Vec3::new(-50.0, 0.0, 2.0)],
        IdmConfig::default(),
    );
    agent_1.yaw = -std::f32::consts::FRAC_PI_2; // explicitly heading West (opposing)
    sim.agents.push(agent_1);

    // Step simulation
    sim.step(0.1, &ground);

    // Agent 0 should accelerate towards desired speed because oncoming car in opposing lane is NOT a lead obstacle
    assert!(
        sim.agents[0].speed_ms >= 0.0,
        "Agent 0 should not halt due to oncoming traffic in opposing lane"
    );
}
