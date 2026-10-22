//! High-performance asynchronous spatial world streaming and memory management.

pub mod async_loader;
pub mod certification;
pub mod config;
pub mod diagnostics;
pub mod lod;
pub mod lru_cache;
pub mod multi_region;
pub mod package_delivery;
pub mod planetary_streamer;
pub mod prefetch;
pub mod streamer;

pub use async_loader::{
    generate_fallback_chunk, read_chunk_with_corruption_recovery, ChunkLoadQueue, ChunkLoadRequest,
    LoadedChunkResult,
};
pub use certification::{CertificationResult, WorldPackageCertifier};
pub use config::StreamingConfig;
pub use diagnostics::{
    FallbackBasemapProvider, MissingDataCategory, MissingDataDiagnostics, MissingDataRecord,
    MissingDataReport,
};
pub use lod::{select_lod, select_lod_with_hysteresis};
pub use lru_cache::{CachedChunk, ChunkKey, LruChunkCache};
pub use multi_region::{MountedRegion, MultiRegionStreamer, MultiRegionTelemetry};
pub use package_delivery::{
    ChunkTransferState, CountryPackageManifest, InstalledPackageRecord,
    ResumableAcquisitionSession, StorageQuotaManager,
};
pub use planetary_streamer::{PlanetaryLodLevel, PlanetaryOriginCoordinator, RelevanceBudget};
pub use prefetch::{calculate_prefetch_bounds, evaluate_chunk_priority, ChunkPriority};
pub use streamer::{StreamingTelemetry, WorldStreamer};

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_compiler::chunk::{ChunkHeader, WorldChunk, CHUNK_MAGIC, CHUNK_VERSION};
    use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
    use realearth_compiler::mesh::ChunkMesh;
    use realearth_ingestion::dem_grid::DemGrid;
    use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
    use realearth_shared::bounds::GeoBounds;
    use realearth_shared::coordinates::Wgs84Coord;
    use realearth_validation::pipeline::validate_dataset;
    use tempfile::tempdir;

    fn make_dummy_chunk(cx: i32, cy: i32) -> WorldChunk {
        WorldChunk {
            header: ChunkHeader {
                magic: *CHUNK_MAGIC,
                version: CHUNK_VERSION,
                chunk_x: cx,
                chunk_y: cy,
                lod_level: 0,
                bounds: GeoBounds::new(12.97, 12.98, 77.59, 77.60),
                anchor: Wgs84Coord::new(12.975, 77.595, 920.0),
                min_elevation: 900.0,
                max_elevation: 950.0,
            },
            terrain: ChunkMesh::default(),
            roads: ChunkMesh::default(),
            buildings: ChunkMesh::default(),
            water: ChunkMesh::default(),
            vegetation: ChunkMesh::default(),
            building_metadata: Vec::new(),
        }
    }

    #[test]
    fn test_lru_cache_budget_eviction() {
        let mut cache = LruChunkCache::new();

        // Insert 10 chunks
        for i in 0..10 {
            let key = ChunkKey::new(i, 0, 0);
            let chunk = make_dummy_chunk(i, 0);
            cache.insert(key, chunk, i as u64);
        }

        assert_eq!(cache.len(), 10);
        let mem = cache.total_cpu_bytes();
        assert!(mem > 0);

        // Evict to half of current memory
        let target = mem / 2;
        let evicted = cache.evict_to_budget(target, 9, 0);

        assert!(evicted > 0, "Should have evicted chunks");
        assert!(cache.total_cpu_bytes() <= target);
        assert_eq!(cache.evictions, evicted as u64);
    }

    #[test]
    fn test_collision_critical_pinning_prevents_eviction() {
        let mut cache = LruChunkCache::new();

        for i in 0..6 {
            let key = ChunkKey::new(i, 0, 0);
            let chunk = make_dummy_chunk(i, 0);
            let pinned = i == 0; // Chunk 0 is pinned for collision
            cache.insert_with_options(key, chunk, i as u64, pinned, false);
        }

        assert_eq!(cache.pinned_count(), 1);
        let key0 = ChunkKey::new(0, 0, 0);
        assert!(cache.is_pinned(&key0));

        // Evict with extreme budget of 0 bytes
        cache.evict_to_budget(0, 5, 0);

        // All unpinned chunks evicted, but chunk 0 MUST remain in cache!
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&key0).is_some());
    }

    #[test]
    fn test_chunk_corruption_recovery() {
        let dir = tempdir().expect("Failed to create tempdir");
        let corrupt_file = dir.path().join("corrupt.rec");
        // Write invalid corrupted bytes
        std::fs::write(&corrupt_file, b"GARBAGE_NOT_A_VALID_ZSTD_RECH_CHUNK").unwrap();

        let bounds = GeoBounds::new(12.0, 12.1, 77.0, 77.1);
        let res = read_chunk_with_corruption_recovery(&corrupt_file, 0, 0, 0, &bounds)
            .expect("Should safely recover with fallback chunk");

        assert!(res.is_fallback, "Expected fallback flag to be true");
        assert!(
            !res.chunk.terrain.is_empty(),
            "Fallback should supply non-empty safe terrain"
        );
        assert_eq!(res.chunk.terrain.vertices.len(), 4);
        assert_eq!(res.chunk.terrain.indices.len(), 6);
    }

    #[test]
    fn test_world_streamer_bengaluru_pipeline() {
        let dir = tempdir().expect("Failed to create tempdir");
        let world_dir = dir.path().join("world");

        // 1. Compile test world
        let raw = create_bengaluru_central_dataset();
        let (val, _) = validate_dataset(&raw);
        let dem = DemGrid::create_bengaluru_plateau_dem(32, 32);
        let opts = CompilerOptions {
            chunk_size_meters: 1000.0,
            terrain_grid_resolution: 8,
            ..Default::default()
        };
        compile_world_package(&val, &dem, &world_dir, &opts).expect("Compilation failed");

        // 2. Initialize WorldStreamer
        let config = StreamingConfig {
            max_memory_mb: 64, // Small budget for testing
            view_distance_meters: 2500.0,
            ..Default::default()
        };
        let mut streamer =
            WorldStreamer::open(&world_dir, config).expect("Failed to open streamer");

        // 3. Update camera position at Vidhana Soudha, Bengaluru
        let vidhana_soudha = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let loaded = streamer
            .update(vidhana_soudha)
            .expect("Streamer update failed");

        assert!(
            loaded > 0,
            "Expected chunks to be loaded around Vidhana Soudha"
        );
        let active = streamer.get_active_chunks();
        assert!(!active.is_empty(), "Expected active chunks in memory");

        // Check telemetry
        let telem = streamer.telemetry_details();
        assert_eq!(telem.active_chunks, active.len());
        assert!(telem.cpu_memory_mb > 0.0);
        assert!(telem.misses > 0);

        // Check collision readiness
        assert!(streamer.is_collision_ready(vidhana_soudha));

        // Test teleportation
        let distant_spot = Wgs84Coord::new(13.10, 77.70, 900.0);
        streamer.teleport(distant_spot);
        let telem2 = streamer.telemetry_details();
        assert_eq!(telem2.pending_queue, 0);
    }

    #[test]
    fn test_multi_region_streamer_dual_mount_and_cross_region_update() {
        let temp = tempdir().expect("tempdir failed");

        // 1. Compile Region A (Bengaluru)
        let dir_a = temp.path().join("bengaluru");
        let (dataset_a, _) = validate_dataset(&create_bengaluru_central_dataset());
        let dem_a = DemGrid::create_bengaluru_plateau_dem(16, 16);
        let opts = CompilerOptions {
            chunk_size_meters: 1000.0,
            terrain_grid_resolution: 8,
            ..Default::default()
        };
        compile_world_package(&dataset_a, &dem_a, &dir_a, &opts).unwrap();

        // 2. Initialize MultiRegionStreamer and mount Bengaluru
        let config = StreamingConfig {
            max_memory_mb: 64,
            view_distance_meters: 2500.0,
            ..Default::default()
        };
        let mut multi_streamer = MultiRegionStreamer::new(config);
        multi_streamer
            .mount_region("bengaluru-central", &dir_a, dem_a.bounds)
            .unwrap();

        // 3. Update at Vidhana Soudha
        let vidhana = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let count = multi_streamer.update(vidhana).unwrap();
        assert!(count > 0, "Expected loaded chunks in multi-region streamer");
        assert_eq!(multi_streamer.loaded_chunk_count(), count);

        let telem = multi_streamer.telemetry();
        assert_eq!(telem.mounted_regions_count, 1);
        assert!(telem.active_chunks > 0);
        assert_eq!(telem.active_region_ids, vec!["bengaluru-central"]);
    }
}
