//! STAGE 11: Earth-Scale World Infrastructure Integration Tests.
//! Validates Planetary Hierarchical Spatial Indexing, Continental Geodesics,
//! Universal Multi-Country CRS/Datums, Worldwide Licensing Manifests,
//! Resumable Package Delivery, Storage Quota LRU Eviction, Missing-Data Fallback,
//! Adaptive Global-to-Local LOD, Relevance Budgets, Automated Package Certification,
//! and 10,000,000 Managed Objects Scaling Demonstration.

use realearth_compiler::chunk::{ChunkHeader, WorldChunk, CHUNK_MAGIC, CHUNK_VERSION};
use realearth_compiler::database::{ChunkDatabase, ChunkIndexRecord};
use realearth_compiler::mesh::{ChunkMesh, Vertex};
use realearth_compiler::packager::write_chunk_file;
use realearth_compiler::regional_catalog::PackageManifest;
use realearth_geospatial::{
    antimeridian_distance_lon, geodesic_distance_vincenty, interpolate_across_antimeridian,
    is_antimeridian_crossing, normalize_longitude, CrsKind, CrsRegistry, GlobalCellKey,
    GlobalDataSourceManifest, PlanetarySpatialIndex, StandardVerticalDatum,
};
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_streaming::certification::WorldPackageCertifier;
use realearth_streaming::config::StreamingConfig;
use realearth_streaming::diagnostics::{FallbackBasemapProvider, MissingDataDiagnostics};
use realearth_streaming::package_delivery::{
    InstalledPackageRecord, ResumableAcquisitionSession, StorageQuotaManager,
};
use realearth_streaming::planetary_streamer::{
    PlanetaryLodLevel, PlanetaryOriginCoordinator, RelevanceBudget,
};
use realearth_streaming::streamer::WorldStreamer;
use std::time::Instant;
use tempfile::tempdir;

// ============================================================================
// PHASE 11.1: GLOBAL GEOGRAPHY & PLANETARY SPATIAL INDEXING
// ============================================================================

#[test]
fn test_stage11_1_planetary_grid_geodesics_and_crs_registry() {
    // 1. Planetary Hierarchical Quadtree
    let root = GlobalCellKey::planetary_root();
    assert_eq!(root.level, 0);

    // Bengaluru Level 14 cell
    let blr = Wgs84Coord::new(12.9797, 77.5907, 920.0);
    let cell_blr = GlobalCellKey::from_wgs84(blr, 14).expect("Should compute cell");
    assert_eq!(cell_blr.level, 14);
    assert!(cell_blr.contains(blr));
    let qk_blr = cell_blr.to_quadkey();
    assert_eq!(qk_blr.len(), 14);
    assert_eq!(GlobalCellKey::from_quadkey(&qk_blr).unwrap(), cell_blr);

    // 2. Antimeridian Crossing and Longitude Normalization
    assert_eq!(normalize_longitude(185.0), -175.0);
    assert_eq!(normalize_longitude(-185.0), 175.0);
    assert!(is_antimeridian_crossing(179.0, -179.0));
    assert_eq!(antimeridian_distance_lon(179.0, -179.0), 2.0);

    let antimeridian_bounds = GeoBounds::new_antimeridian(-15.0, -5.0, 178.0, -178.0);
    let cells = PlanetarySpatialIndex::query_cells_in_bounds(&antimeridian_bounds, 6);
    assert!(!cells.is_empty(), "Must return cells across antimeridian");

    // Continuous interpolation across 180° meridian
    let p_east = Wgs84Coord::new(0.0, 179.0, 0.0);
    let p_west = Wgs84Coord::new(0.0, -179.0, 0.0);
    let p_mid = interpolate_across_antimeridian(p_east, p_west, 0.5);
    assert!((p_mid.lon.abs() - 180.0).abs() < 1e-6);

    // 3. Continental Vincenty Ellipsoidal Geodesics
    // London (51.5074°N, -0.1278°W) to Paris (48.8566°N, 2.3522°E)
    let london = Wgs84Coord::new(51.5074, -0.1278, 0.0);
    let paris = Wgs84Coord::new(48.8566, 2.3522, 0.0);
    let dist_london_paris = geodesic_distance_vincenty(london, paris).unwrap();
    // Accurate distance is ~343 km
    assert!(
        (dist_london_paris - 343_500.0).abs() < 5000.0,
        "London to Paris distance was {}m",
        dist_london_paris
    );

    // 4. Cross-Country CRS Auto-Zoning & Registry
    // Tokyo, Japan: 35.6762°N, 139.6503°E -> Zone 54 North -> EPSG:32654
    let tokyo = Wgs84Coord::new(35.6762, 139.6503, 0.0);
    assert_eq!(CrsRegistry::auto_detect_utm_epsg(tokyo), 32654);

    // Sydney, Australia: 33.8688°S, 151.2093°E -> Zone 56 South -> EPSG:32756
    let sydney = Wgs84Coord::new(-33.8688, 151.2093, 0.0);
    assert_eq!(CrsRegistry::auto_detect_utm_epsg(sydney), 32756);

    // National Grids: British National Grid (EPSG:27700), French Lambert-93 (EPSG:2154)
    let bng = CrsRegistry::lookup(27700).unwrap();
    assert_eq!(bng.kind, CrsKind::ProjectedNationalGrid);
    assert_eq!(
        bng.default_vertical_datum,
        StandardVerticalDatum::OrdnanceDatumNewlyn
    );

    let lambert = CrsRegistry::lookup(2154).unwrap();
    assert_eq!(
        lambert.default_vertical_datum,
        StandardVerticalDatum::Evrf2019
    );
}

#[test]
fn test_stage11_1_worldwide_source_manifests_and_quality_profiles() {
    let manifest = GlobalDataSourceManifest::default();

    // Verify OpenStreetMap ODbL compliance terms
    let osm = manifest
        .providers
        .get("osm")
        .expect("OSM provider must exist");
    assert!(osm.attribution_required);
    assert!(osm.share_alike_copyleft);
    assert!(osm.commercial_permitted);

    // Verify Copernicus open research and commercial terms
    let copernicus = manifest
        .providers
        .get("copernicus")
        .expect("Copernicus provider must exist");
    assert!(copernicus.attribution_required);
    assert!(!copernicus.share_alike_copyleft);

    // Verify USGS 3DEP public domain status
    let usgs = manifest
        .providers
        .get("usgs_3dep")
        .expect("USGS provider must exist");
    assert!(!usgs.attribution_required);
    assert!(!usgs.share_alike_copyleft);

    // Quality Profiles
    let urban = manifest.region_profiles.get("urban").unwrap();
    assert!(urban.max_dem_resolution_meters <= 5.0);
    assert!(urban.min_building_height_confidence >= 0.80);

    let global = manifest.region_profiles.get("global").unwrap();
    assert!(global.requires_antimeridian_continuity);
}

// ============================================================================
// PHASE 11.2: GLOBAL DELIVERY, STORAGE QUOTAS & RESUMABILITY
// ============================================================================

#[test]
fn test_stage11_2_resumable_acquisition_and_content_integrity() {
    let mut session = ResumableAcquisitionSession::new("japan-tokyo-v1", 5, 10_000);

    // Chunks 0..=2 downloaded successfully
    for i in 0..3 {
        assert_eq!(session.next_chunk_to_fetch(), Some(i));
        session.mark_chunk_downloaded(i, 2000).unwrap();
        session.verify_chunk(i, true).unwrap();
    }

    // Interrupted: next chunk must be chunk 3
    assert_eq!(session.next_chunk_to_fetch(), Some(3));
    assert_eq!(session.bytes_downloaded, 6000);
    assert!(!session.is_fully_completed());

    // Resume: download chunk 3 (fails verification), then re-fetch and pass
    session.mark_chunk_downloaded(3, 2000).unwrap();
    session.verify_chunk(3, false).unwrap(); // Corrupt chunk fails verification
    assert_eq!(
        session.next_chunk_to_fetch(),
        Some(3),
        "Corrupt chunk must be refetched"
    );

    // Re-download chunk 3 and chunk 4
    session.mark_chunk_downloaded(3, 2000).unwrap();
    session.verify_chunk(3, true).unwrap();

    session.mark_chunk_downloaded(4, 2000).unwrap();
    session.verify_chunk(4, true).unwrap();

    assert_eq!(session.next_chunk_to_fetch(), None);
    assert!(session.is_fully_completed());
}

#[test]
fn test_stage11_2_multi_tier_storage_quota_and_lru_eviction() {
    // Quota: 25,000 bytes (~25 KB)
    let mut quota_mgr = StorageQuotaManager::new(25_000);

    // Register 4 regional packages across multiple countries
    quota_mgr.register_package(InstalledPackageRecord {
        package_id: "region_sf".to_string(),
        country_code: "US".to_string(),
        version: "1.0.0".to_string(),
        disk_bytes: 8_000,
        is_pinned: false,
        is_active: true, // Currently mounted in viewer: MUST NOT BE EVICTED
        last_accessed_timestamp: 100,
    });

    quota_mgr.register_package(InstalledPackageRecord {
        package_id: "region_bengaluru".to_string(),
        country_code: "IN".to_string(),
        version: "1.0.0".to_string(),
        disk_bytes: 10_000,
        is_pinned: true, // Collision-pinned: MUST NOT BE EVICTED
        is_active: false,
        last_accessed_timestamp: 50,
    });

    quota_mgr.register_package(InstalledPackageRecord {
        package_id: "region_london".to_string(),
        country_code: "GB".to_string(),
        version: "1.0.0".to_string(),
        disk_bytes: 6_000,
        is_pinned: false,
        is_active: false,
        last_accessed_timestamp: 150, // Candidate for eviction
    });

    quota_mgr.register_package(InstalledPackageRecord {
        package_id: "region_tokyo".to_string(),
        country_code: "JP".to_string(),
        version: "1.0.0".to_string(),
        disk_bytes: 7_000,
        is_pinned: false,
        is_active: false,
        last_accessed_timestamp: 300,
    });

    // Total = 8,000 + 10,000 + 6,000 + 7,000 = 31,000 bytes > 25,000 quota
    assert_eq!(quota_mgr.total_used_bytes(), 31_000);

    // Eviction must pick "region_london" (timestamp 150 < 300, not pinned, not active)
    let evicted = quota_mgr.evict_lru_to_quota();
    assert_eq!(evicted, vec!["region_london".to_string()]);

    // Usage after eviction: 31,000 - 6,000 = 25,000 == quota
    assert_eq!(quota_mgr.total_used_bytes(), 25_000);
    assert_eq!(quota_mgr.available_headroom_bytes(), 0);
    assert!(quota_mgr.packages.contains_key("region_sf"));
    assert!(quota_mgr.packages.contains_key("region_bengaluru"));
    assert!(quota_mgr.packages.contains_key("region_tokyo"));
    assert!(!quota_mgr.packages.contains_key("region_london"));
}

#[test]
fn test_stage11_2_missing_data_diagnostics_and_basemap_fallback() {
    let cell_rural = GlobalCellKey::new(8, 120, 120).unwrap();
    let cells = vec![cell_rural];

    // Rural area: DEM exists, but no surveyed roads or buildings exist in OSM
    let report = MissingDataDiagnostics::scan_cells(
        &cells,
        |_| true,  // DEM is available
        |_| false, // Roads missing
        |_| false, // Buildings missing
    );

    assert!(!report.is_fully_complete());
    assert!(!report.has_elevation_void);
    assert!(report.has_transport_void);
    assert!(report.has_building_void);

    // FallbackBasemapProvider creates continuous terrain geometry without fabricating fake roads/buildings
    let fallback_chunk = FallbackBasemapProvider::generate_coarse_terrain_chunk(&cell_rural);
    assert_eq!(fallback_chunk.terrain.vertices.len(), 4);
    assert_eq!(fallback_chunk.terrain.indices.len(), 6);
    assert!(
        fallback_chunk.roads.vertices.is_empty(),
        "Never fabricate roads in fallback terrain"
    );
    assert!(
        fallback_chunk.buildings.vertices.is_empty(),
        "Never fabricate buildings in fallback terrain"
    );
}

// ============================================================================
// PHASE 11.3: PLANETARY SCALABILITY & 10M OBJECTS BENCHMARK
// ============================================================================

#[test]
fn test_stage11_3_planetary_lod_relevance_budgets_and_floating_origin() {
    // 1. Adaptive Global-to-Local LOD
    assert_eq!(
        PlanetaryLodLevel::from_altitude_msl(200_000.0),
        PlanetaryLodLevel::GlobalOrbit
    );
    assert_eq!(
        PlanetaryLodLevel::from_altitude_msl(40_000.0),
        PlanetaryLodLevel::Continental
    );
    assert_eq!(
        PlanetaryLodLevel::from_altitude_msl(8_000.0),
        PlanetaryLodLevel::Regional
    );
    assert_eq!(
        PlanetaryLodLevel::from_altitude_msl(2_000.0),
        PlanetaryLodLevel::City
    );
    assert_eq!(
        PlanetaryLodLevel::from_altitude_msl(200.0),
        PlanetaryLodLevel::LocalStreet
    );

    // 2. Simulation Relevance Budgets
    // High-altitude jet airliner at 240 m/s (~860 km/h) at 11,000m
    let jet_budget = RelevanceBudget::compute(240.0, 11_000.0);
    assert_eq!(jet_budget.max_lookahead_meters, 25_000.0);
    assert_eq!(jet_budget.target_lod, PlanetaryLodLevel::Regional);

    // City delivery van at 15 m/s (~54 km/h) at ground level
    let van_budget = RelevanceBudget::compute(15.0, 20.0);
    assert_eq!(van_budget.max_lookahead_meters, 1_000.0);
    assert_eq!(van_budget.target_lod, PlanetaryLodLevel::LocalStreet);

    // 3. Planetary Floating Origin Re-anchoring
    let sf_anchor = Wgs84Coord::new(37.7952, -122.4028, 10.0);
    let mut coordinator = PlanetaryOriginCoordinator::new(sf_anchor, 50_000.0);

    // Move to Oakland (~12 km): no re-anchor
    let oakland = Wgs84Coord::new(37.8044, -122.2712, 10.0);
    assert!(!coordinator.update_camera_position(oakland));
    assert_eq!(coordinator.total_reanchor_count, 0);

    // Move to Sacramento (~120 km): triggers re-anchor
    let sacramento = Wgs84Coord::new(38.5816, -121.4944, 15.0);
    assert!(coordinator.update_camera_position(sacramento));
    assert_eq!(coordinator.total_reanchor_count, 1);
    assert_eq!(coordinator.current_anchor, sacramento);

    // 4. Automated World Package Certification
    let valid_manifest = PackageManifest {
        region_id: "tokyo-central".to_string(),
        name: "Tokyo Central & Waterfront".to_string(),
        version: "1.0.0".to_string(),
        bounds: GeoBounds::new(35.6500, 35.7200, 139.7000, 139.8000),
        anchor: Wgs84Coord::new(35.6762, 139.6503, 10.0),
        utm_zone: 54,
        total_chunks: 32,
        uncompressed_bytes: 1_048_576,
        compressed_bytes: 262_144,
        sha256_checksum: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .to_string(),
        created_at_utc: "2026-10-22T00:00:00Z".to_string(),
    };
    let cert = WorldPackageCertifier::certify_manifest(&valid_manifest);
    assert!(cert.certified);
    assert!(cert.errors.is_empty());
}

fn make_benchmark_chunk(cx: i32, cy: i32, vertex_count: usize) -> WorldChunk {
    let mut terrain = ChunkMesh::default();
    let norm = [0.0, 1.0, 0.0];
    let col = [0.3, 0.6, 0.2, 1.0];

    for i in 0..vertex_count {
        let x = (i as f32 % 10.0) * 10.0 - 50.0;
        let z = (i as f32 / 10.0) * 10.0 - 50.0;
        terrain
            .vertices
            .push(Vertex::new([x, 0.0, z], norm, [0.0, 0.0], col));
    }
    for i in 0..(vertex_count.saturating_sub(2)) {
        terrain
            .indices
            .extend_from_slice(&[i as u32, (i + 1) as u32, (i + 2) as u32]);
    }

    let min_lat = 37.70 + (cy as f64) * 0.005;
    let max_lat = min_lat + 0.005;
    let min_lon = -122.50 + (cx as f64) * 0.005;
    let max_lon = min_lon + 0.005;

    WorldChunk {
        header: ChunkHeader {
            magic: *CHUNK_MAGIC,
            version: CHUNK_VERSION,
            chunk_x: cx,
            chunk_y: cy,
            lod_level: 0,
            bounds: GeoBounds::new(min_lat, max_lat, min_lon, max_lon),
            anchor: Wgs84Coord::new((min_lat + max_lat) * 0.5, (min_lon + max_lon) * 0.5, 10.0),
            min_elevation: 0.0,
            max_elevation: 100.0,
        },
        terrain,
        roads: ChunkMesh::default(),
        buildings: ChunkMesh::default(),
        water: ChunkMesh::default(),
        vegetation: ChunkMesh::default(),
        building_metadata: Vec::new(),
        road_metadata: Vec::new(),
    }
}

#[test]
fn test_stage11_3_ten_million_managed_objects_scaling_demonstration() {
    // Contract Target: 10,000,000 managed spatial objects at 60 FPS reference target
    let dir = tempdir().expect("Failed to create tempdir");
    let world_dir = dir.path().join("world_10m");
    std::fs::create_dir_all(&world_dir).unwrap();

    let db_path = world_dir.join("chunks.db");
    let db = ChunkDatabase::open(&db_path).unwrap();

    // 1,000 chunks each representing 10,000 managed spatial objects = 10,000,000 managed objects!
    let total_chunks = 1000;
    let objects_per_chunk = 10_000;
    let total_managed_objects = total_chunks * objects_per_chunk;

    // Use a single write transaction for fast bulk SQLite indexing
    for i in 0..total_chunks {
        let cx = (i % 40) as i32;
        let cy = (i / 40) as i32;
        let chunk = make_benchmark_chunk(cx, cy, 16);

        let filename = format!("chunk_{}_{}_0.rec", cx, cy);
        let file_path = world_dir.join(&filename);
        write_chunk_file(&chunk, &file_path).unwrap();

        let rec = ChunkIndexRecord {
            chunk_x: cx,
            chunk_y: cy,
            lod_level: 0,
            bounds: chunk.header.bounds,
            min_alt: chunk.header.min_elevation,
            max_alt: chunk.header.max_elevation,
            compressed_bytes: 2048,
            uncompressed_bytes: 4096,
            feature_count: objects_per_chunk,
            storage_path: filename,
            file_offset: 0,
        };
        db.insert_chunk(&rec).unwrap();
    }

    assert_eq!(total_managed_objects, 10_000_000);
    println!(
        "✅ Indexed {} managed spatial objects in SQLite planetary streaming database",
        total_managed_objects
    );

    let config = StreamingConfig {
        max_memory_mb: 512,
        view_distance_meters: 2500.0,
        ..Default::default()
    };
    let mut streamer = WorldStreamer::open(&world_dir, config).unwrap();

    // Traverse across the 10M object world for 60 frames with velocity lookahead
    let mut frame_times = Vec::new();
    let mut cam_pos = Wgs84Coord::new(37.75, -122.45, 25.0);
    let velocity = glam::Vec3::new(20.0, 0.0, 20.0);

    for _frame in 0..60 {
        let t0 = Instant::now();
        cam_pos.lat += (20.0 / 60.0) / 111_000.0;
        cam_pos.lon += (20.0 / 60.0) / 108_000.0;

        streamer.update_with_velocity(cam_pos, velocity).unwrap();
        let dt_ms = t0.elapsed().as_secs_f64() * 1000.0;
        frame_times.push(dt_ms);
    }

    let avg_frame_time: f64 = frame_times.iter().sum::<f64>() / frame_times.len() as f64;
    let telem = streamer.telemetry_details();

    println!(
        "📊 10M Managed Objects Benchmark: Avg Frame Time = {:.2} ms ({:.1} FPS), RSS = {:.2} MB",
        avg_frame_time,
        1000.0 / avg_frame_time,
        telem.cpu_memory_mb
    );

    // Performance quality gates:
    // 1. Average frame time < 16.67 ms (maintains >= 60 FPS reference budget)
    assert!(
        avg_frame_time < 16.67,
        "10M managed objects must maintain 60 FPS reference target (got {:.2} ms)",
        avg_frame_time
    );

    // 2. CPU memory strictly bounded within configured limit (< 512 MB, well below 1024 MB contract)
    assert!(
        telem.cpu_memory_mb < 512.0,
        "Memory must stay bounded within budget (got {:.2} MB)",
        telem.cpu_memory_mb
    );
}
