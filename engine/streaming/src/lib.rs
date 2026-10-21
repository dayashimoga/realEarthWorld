//! High-performance asynchronous spatial world streaming and memory management.

pub mod config;
pub mod lod;
pub mod lru_cache;
pub mod streamer;

pub use config::StreamingConfig;
pub use lod::select_lod;
pub use lru_cache::{CachedChunk, ChunkKey, LruChunkCache};
pub use streamer::WorldStreamer;

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
        let mem = cache.total_memory_bytes();
        assert!(mem > 0);

        // Evict to half of current memory
        let target = mem / 2;
        let evicted = cache.evict_to_budget(target, 9, 0);

        assert!(evicted > 0, "Should have evicted chunks");
        assert!(cache.total_memory_bytes() <= target);
        assert_eq!(cache.evictions, evicted as u64);
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
        let (count, mem_mb, _hits, misses, _) = streamer.telemetry();
        assert_eq!(count, active.len());
        assert!(mem_mb > 0.0);
        assert!(misses > 0);
    }
}
