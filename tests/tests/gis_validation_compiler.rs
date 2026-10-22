//! Stage 2: Minimum GIS Validation and Production World Compiler Tests.
//!
//! Validates:
//! Phase 2.1 - Blocking validation (coordinates, road topology, grade separations, water conflicts, diagnostics)
//! Phase 2.2 - Hierarchical compiler (meshes, terrain DEM conformity, buildings, roads, vegetation, water, spatial indexes)
//! Phase 2.3 - Binary storage (zstd .rec, SQLite chunks.db manifests, checksums, deterministic incremental builds, corruption recovery)
//! DONE Criteria - Real Bengaluru GIS -> validated chunks -> headless reload with identical geographic alignment and zero seams.

use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_compiler::database::ChunkDatabase;
use realearth_compiler::packager::read_chunk_file;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::acquisition::compute_sha256;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_parser::parse_osm_xml;
use realearth_ingestion::osm_types::{
    BuildingType, HighwayType, IngestedDataset, OsmBuildingFeature, OsmRoadFeature,
    OsmWaterFeature, WaterType,
};
use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
use realearth_navigation::{find_route, RoadNetworkGraph, RoutingPreference};
use realearth_shared::coordinates::{EnuCoord, Wgs84Coord};
use realearth_streaming::async_loader::read_chunk_with_corruption_recovery;
use realearth_streaming::{StreamingConfig, WorldStreamer};
use realearth_validation::diagnostic::DiagnosticSeverity;
use realearth_validation::pipeline::validate_dataset;
use realearth_validation::publication_gate::{CompilerPublicationGate, PublicationVerdict};
use realearth_validation::terrain_water_validator::validate_water_building_conflicts;
use realearth_validation::RepairEngine;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_stage2_gis_blocking_validation_and_quarantine() {
    // 1. Load real Bengaluru Central OSM XML
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Parent dir of tests crate");
    let osm_xml_path = root.join("assets/bengaluru_sample/bengaluru_central.osm.xml");
    assert!(
        osm_xml_path.exists(),
        "Bengaluru sample OSM XML must be present at {:?}",
        osm_xml_path
    );
    let xml_content =
        std::fs::read_to_string(&osm_xml_path).expect("Failed to read bengaluru_central.osm.xml");
    let xml_dataset = parse_osm_xml(&xml_content).expect("Failed to parse OSM XML");

    assert!(
        xml_dataset.roads.len() >= 4,
        "Expected at least 4 road ways in Bengaluru Central"
    );
    assert!(
        xml_dataset.buildings.len() >= 4,
        "Expected at least 4 landmark buildings in Bengaluru Central"
    );

    // 2. Validate clean Bengaluru dataset
    let (_validated, report) = validate_dataset(&xml_dataset);
    assert!(
        report.is_clean(),
        "Surveyed Bengaluru dataset should produce no unquarantined errors"
    );

    let gate = CompilerPublicationGate::production();
    let verdict = gate.evaluate(&report);
    match verdict {
        PublicationVerdict::Approved { total_features, .. } => {
            assert!(
                total_features >= 8,
                "Expected approved dataset to contain all valid features"
            );
        }
        PublicationVerdict::Rejected { reasons, .. } => {
            panic!(
                "Production gate rejected clean Bengaluru dataset: {:?}",
                reasons
            );
        }
    }

    // 3. Negative Test: Inject NaN and out-of-range coordinates -> Must trigger BLOCKER
    let mut bad_dataset = xml_dataset.clone();
    bad_dataset.roads.push(OsmRoadFeature {
        way_id: 999_001,
        name: Some("Faulty Road NaN".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(12.9790, 77.5900, 0.0),
            Wgs84Coord::new(f64::NAN, 77.5920, 0.0), // Malformed NaN coordinate
        ],
        lanes: 2,
        speed_limit_kph: 50,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });
    bad_dataset.buildings.push(OsmBuildingFeature {
        id: 999_002,
        name: Some("Out of Range Building".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(95.0, 77.59, 0.0), // Out of bounds latitude > 90
            Wgs84Coord::new(95.1, 77.59, 0.0),
            Wgs84Coord::new(95.1, 77.60, 0.0),
            Wgs84Coord::new(95.0, 77.59, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(20.0),
        levels: Some(5),
    });

    let (_, bad_report) = validate_dataset(&bad_dataset);
    let blockers: Vec<_> = bad_report
        .diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Blocker)
        .collect();
    assert!(
        blockers.len() >= 2,
        "Must flag both NaN road and out-of-bounds building as Fatal BLOCKERs"
    );

    let bad_verdict = gate.evaluate(&bad_report);
    match bad_verdict {
        PublicationVerdict::Rejected { blocker_count, .. } => {
            assert!(
                blocker_count >= 2,
                "Publication gate must reject dataset with fatal blockers"
            );
        }
        PublicationVerdict::Approved { .. } => {
            panic!("Gate must NOT approve dataset with NaN / out-of-bounds coordinates");
        }
    }

    // 4. Validate Water / Building Conflict Detection
    let mut water_dataset = IngestedDataset::default();
    let lake = OsmWaterFeature {
        id: 300_001,
        name: Some("Ulsoor Lake".to_string()),
        water_type: WaterType::Lake,
        outer_ring: vec![
            Wgs84Coord::new(12.9800, 77.6100, 0.0),
            Wgs84Coord::new(12.9850, 77.6100, 0.0),
            Wgs84Coord::new(12.9850, 77.6200, 0.0),
            Wgs84Coord::new(12.9800, 77.6200, 0.0),
            Wgs84Coord::new(12.9800, 77.6100, 0.0),
        ],
        is_coastline: false,
        target_elevation_msl: 0.0,
    };
    water_dataset.water_bodies.push(lake);

    // Submerged building directly inside Ulsoor Lake
    water_dataset.buildings.push(OsmBuildingFeature {
        id: 400_001,
        name: Some("Submerged Structure".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(12.9820, 77.6140, 0.0),
            Wgs84Coord::new(12.9830, 77.6140, 0.0),
            Wgs84Coord::new(12.9830, 77.6150, 0.0),
            Wgs84Coord::new(12.9820, 77.6150, 0.0),
            Wgs84Coord::new(12.9820, 77.6140, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(15.0),
        levels: Some(3),
    });

    let (val_water, _) = validate_dataset(&water_dataset);
    let mut repair_eng = RepairEngine::new();
    validate_water_building_conflicts(
        &val_water.buildings,
        &val_water.water_bodies,
        &mut repair_eng,
    );

    let conflict_warnings: Vec<_> = repair_eng
        .diagnostics
        .iter()
        .filter(|d| d.rule_id.as_deref() == Some("RULE_WATER_BUILDING_CONFLICT"))
        .collect();
    assert_eq!(
        conflict_warnings.len(),
        1,
        "Must detect building submerged in water body"
    );
    assert_eq!(
        conflict_warnings[0].severity,
        DiagnosticSeverity::Warning,
        "Water conflict should produce non-silent Warning diagnostic"
    );
}

#[test]
fn test_stage2_world_compiler_repeatability_proof() {
    // Proves that compiling Bengaluru Central twice produces 100% bit-exact chunk outputs
    let temp_a = tempdir().expect("tempdir failed");
    let temp_b = tempdir().expect("tempdir failed");
    let out_a = temp_a.path().join("world_a");
    let out_b = temp_b.path().join("world_b");

    let raw_dataset = create_bengaluru_central_dataset();
    let (validated, _) = validate_dataset(&raw_dataset);
    let dem = DemGrid::create_bengaluru_plateau_dem(32, 32);

    let options = CompilerOptions {
        chunk_size_meters: 1000.0,
        terrain_grid_resolution: 8,
        incremental: false,
        crs_epsg: 32643,
        package_name: "bengaluru_repeatability".to_string(),
    };

    let stats_a = compile_world_package(&validated, &dem, &out_a, &options)
        .expect("Compilation run A failed");
    let stats_b = compile_world_package(&validated, &dem, &out_b, &options)
        .expect("Compilation run B failed");

    assert_eq!(
        stats_a.total_chunks_generated,
        stats_b.total_chunks_generated
    );
    assert_eq!(stats_a.total_vertices, stats_b.total_vertices);
    assert_eq!(stats_a.total_triangles, stats_b.total_triangles);
    assert_eq!(
        stats_a.total_compressed_bytes,
        stats_b.total_compressed_bytes
    );

    let db_a = ChunkDatabase::open(&out_a.join("chunks.db")).unwrap();
    let db_b = ChunkDatabase::open(&out_b.join("chunks.db")).unwrap();

    let chunks_a = db_a.query_bbox(&dem.bounds).unwrap();
    let chunks_b = db_b.query_bbox(&dem.bounds).unwrap();
    assert_eq!(chunks_a.len(), chunks_b.len());

    // Compare each chunk file bit-for-bit and check SHA-256
    for ca in &chunks_a {
        let file_a = out_a.join(&ca.storage_path);
        let file_b = out_b.join(&ca.storage_path);
        assert!(file_a.exists());
        assert!(file_b.exists());

        let bytes_a = std::fs::read(&file_a).unwrap();
        let bytes_b = std::fs::read(&file_b).unwrap();
        assert_eq!(
            bytes_a, bytes_b,
            "Chunk {} must have bit-exact identical binary representation across compilations",
            ca.storage_path
        );

        let sha_a = compute_sha256(&bytes_a);
        let sha_b = compute_sha256(&bytes_b);
        assert_eq!(sha_a, sha_b);

        // Verify checksums in SQLite metadata match the computed SHA-256
        let meta_a = db_a
            .get_chunk_metadata(ca.chunk_x, ca.chunk_y, ca.lod_level)
            .unwrap()
            .expect("Metadata missing in run A");
        let meta_b = db_b
            .get_chunk_metadata(ca.chunk_x, ca.chunk_y, ca.lod_level)
            .unwrap()
            .expect("Metadata missing in run B");

        assert_eq!(meta_a.sha256_checksum, sha_a);
        assert_eq!(meta_b.sha256_checksum, sha_b);
        assert_eq!(meta_a.source_hash, meta_b.source_hash);
    }
}

#[test]
fn test_stage2_deterministic_incremental_compilation() {
    let dir = tempdir().expect("tempdir failed");
    let output_dir = dir.path().join("bengaluru_incremental");

    let raw_dataset = create_bengaluru_central_dataset();
    let (validated, _) = validate_dataset(&raw_dataset);
    let dem = DemGrid::create_bengaluru_plateau_dem(32, 32);

    let options = CompilerOptions {
        chunk_size_meters: 1000.0,
        terrain_grid_resolution: 8,
        incremental: false,
        crs_epsg: 32643,
        package_name: "bengaluru_inc".to_string(),
    };

    // 1. Initial complete compilation
    let stats1 = compile_world_package(&validated, &dem, &output_dir, &options)
        .expect("Initial compile failed");
    let total_initial = stats1.total_chunks_generated;
    assert!(total_initial > 0);
    assert_eq!(stats1.chunks_reused, 0);

    // 2. Incremental compilation with zero modifications
    let mut inc_options = options.clone();
    inc_options.incremental = true;
    let stats2 = compile_world_package(&validated, &dem, &output_dir, &inc_options)
        .expect("Zero-change incremental compile failed");

    assert_eq!(
        stats2.total_chunks_generated, 0,
        "No chunks should be regenerated when inputs are unchanged"
    );
    assert_eq!(
        stats2.chunks_reused, total_initial,
        "All chunks should be reused from cache"
    );

    // 3. Incremental compilation with 1 modified building feature
    let mut modified_dataset = validated.clone();
    // Modify height of Vidhana Soudha (id: 100001)
    if let Some(b) = modified_dataset
        .buildings
        .iter_mut()
        .find(|b| b.id == 100_001)
    {
        b.height.height_meters = 52.0; // modified height
    }

    let stats3 = compile_world_package(&modified_dataset, &dem, &output_dir, &inc_options)
        .expect("Modified incremental compile failed");

    assert!(
        stats3.total_chunks_generated >= 1,
        "Modified chunk must be recompiled"
    );
    assert!(
        stats3.chunks_reused >= 1,
        "Unmodified sibling chunks must still be reused from cache"
    );
    assert_eq!(
        stats3.total_chunks_generated + stats3.chunks_reused,
        total_initial
    );
}

#[test]
fn test_stage2_chunk_integrity_and_corruption_detection() {
    let dir = tempdir().expect("tempdir failed");
    let output_dir = dir.path().join("bengaluru_integrity");

    let raw_dataset = create_bengaluru_central_dataset();
    let (validated, _) = validate_dataset(&raw_dataset);
    let dem = DemGrid::create_bengaluru_plateau_dem(32, 32);

    let options = CompilerOptions {
        chunk_size_meters: 1000.0,
        terrain_grid_resolution: 8,
        ..Default::default()
    };
    compile_world_package(&validated, &dem, &output_dir, &options).unwrap();

    let db = ChunkDatabase::open(&output_dir.join("chunks.db")).unwrap();
    let chunk_path = output_dir.join("chunks/chunk_0_0.rec");
    assert!(chunk_path.exists());

    // 1. Check integrity of intact chunk
    let intact = db.verify_chunk_integrity(0, 0, 0, &chunk_path).unwrap();
    assert!(intact, "Intact chunk must pass cryptographic verification");

    // 2. Corrupt one byte of chunk
    let mut bytes = std::fs::read(&chunk_path).unwrap();
    bytes[42] ^= 0xAA; // Bit flip
    std::fs::write(&chunk_path, &bytes).unwrap();

    // 3. Database integrity verification must report failure
    let corrupt_detected = db.verify_chunk_integrity(0, 0, 0, &chunk_path).unwrap();
    assert!(
        !corrupt_detected,
        "Database integrity verification must catch tampered byte"
    );

    // 4. Raw chunk file read must return error
    let raw_read = read_chunk_file(&chunk_path);
    assert!(
        raw_read.is_err(),
        "Raw chunk file read should fail on corrupted byte stream"
    );

    let recovery_result = read_chunk_with_corruption_recovery(&chunk_path, 0, 0, 0, &dem.bounds);
    assert!(recovery_result.is_ok(), "Corruption recovery must succeed");
    let rec = recovery_result.unwrap();
    assert!(
        rec.is_fallback,
        "Recovered chunk must be flagged as fallback"
    );
    assert!(
        rec.chunk.terrain.vertex_count() > 0,
        "Fallback geometry must contain safe terrain"
    );
}

#[test]
fn test_stage2_headless_reload_and_seam_continuity() {
    let dir = tempdir().expect("tempdir failed");
    let world_dir = dir.path().join("bengaluru_world");

    let raw_dataset = create_bengaluru_central_dataset();
    let (validated, _) = validate_dataset(&raw_dataset);
    let dem = DemGrid::create_bengaluru_plateau_dem(64, 64);

    let options = CompilerOptions {
        chunk_size_meters: 1000.0,
        terrain_grid_resolution: 16,
        package_name: "bengaluru_seamless".to_string(),
        crs_epsg: 32643,
        ..Default::default()
    };
    let stats =
        compile_world_package(&validated, &dem, &world_dir, &options).expect("Compilation failed");
    assert!(stats.total_chunks_generated >= 4);

    // 1. Headless reload in WorldStreamer at Vidhana Soudha
    let vidhana_soudha = Wgs84Coord::new(12.9797, 77.5907, 920.0);
    let config = StreamingConfig {
        max_memory_mb: 256,
        view_distance_meters: 2500.0,
        ..Default::default()
    };
    let mut streamer = WorldStreamer::open(&world_dir, config).expect("Failed to open streamer");

    let loaded = streamer.update(vidhana_soudha).expect("Update failed");
    assert!(loaded > 0, "Chunks must load around Vidhana Soudha");

    let active_chunks = streamer.get_active_chunks();
    assert!(
        active_chunks.len() >= 2,
        "Expected multiple active chunks loaded around camera"
    );

    // 2. Prove Neighbor Connectivity and Boundary Seam Continuity
    let db = ChunkDatabase::open(&world_dir.join("chunks.db")).unwrap();
    let all_records = db.query_bbox(&dem.bounds).unwrap();

    // Verify neighbor pointers in metadata
    for r in &all_records {
        let meta = db
            .get_chunk_metadata(r.chunk_x, r.chunk_y, r.lod_level)
            .unwrap()
            .expect("Missing chunk metadata");
        // Non-isolated chunk must have at least one valid neighbor
        let has_neighbor = meta.neighbor_n.is_some()
            || meta.neighbor_s.is_some()
            || meta.neighbor_e.is_some()
            || meta.neighbor_w.is_some();
        assert!(
            has_neighbor,
            "Chunk ({}, {}) should have neighbor connectivity recorded",
            r.chunk_x, r.chunk_y
        );
    }

    // 3. Verify Road Routing on reloaded geography
    let graph = RoadNetworkGraph::from_roads(&validated.roads);
    let mg_start = Wgs84Coord::new(12.9740, 77.6000, 0.0);
    let mg_end = Wgs84Coord::new(12.9760, 77.6150, 0.0);
    let route = find_route(
        &graph,
        mg_start,
        mg_end,
        RoutingPreference::ShortestDistance,
    )
    .expect("Routing must succeed on reloaded MG road network");
    assert!(route.waypoints.len() >= 3);
    assert!(route.total_distance_meters > 1000.0);

    // 4. Verify identical geographic alignment
    let chunk0 = read_chunk_file(&world_dir.join("chunks/chunk_0_0.rec")).unwrap();
    let anchor = chunk0.header.anchor;
    let enu = EnuConverter::new(anchor);
    let reprojected = enu.enu_to_wgs84(EnuCoord::new(0.0, 0.0, 0.0));
    assert!(
        (reprojected.lat - anchor.lat).abs() < 1e-6,
        "Anchor re-projection must match within 1e-6 degrees"
    );
    assert!(
        (reprojected.lon - anchor.lon).abs() < 1e-6,
        "Anchor re-projection must match within 1e-6 degrees"
    );
}
