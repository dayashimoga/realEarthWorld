//! World Streamer orchestrating SQLite queries, background loading, and cache budgeting.

use crate::config::StreamingConfig;
use crate::lod::select_lod;
use crate::lru_cache::{ChunkKey, LruChunkCache};
use realearth_compiler::chunk::WorldChunk;
use realearth_compiler::database::ChunkDatabase;
use realearth_compiler::packager::read_chunk_file;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::{RealEarthError, Result};
use std::path::{Path, PathBuf};

/// High-performance spatial streamer maintaining active chunks around the camera.
pub struct WorldStreamer {
    pub config: StreamingConfig,
    world_dir: PathBuf,
    db: ChunkDatabase,
    cache: LruChunkCache,
    current_frame: u64,
}

impl WorldStreamer {
    /// Initializes a WorldStreamer targeting a compiled world package directory containing `chunks.db`.
    pub fn open(world_dir: &Path, config: StreamingConfig) -> Result<Self> {
        let db_path = world_dir.join("chunks.db");
        if !db_path.exists() {
            return Err(RealEarthError::Streaming(format!(
                "chunks.db not found in world directory: {:?}",
                world_dir
            )));
        }

        let db = ChunkDatabase::open(&db_path)?;
        let cache = LruChunkCache::new();

        Ok(Self {
            config,
            world_dir: world_dir.to_path_buf(),
            db,
            cache,
            current_frame: 0,
        })
    }

    /// Primary streaming update step called every frame or simulation tick.
    pub fn update(&mut self, camera_pos: Wgs84Coord) -> Result<usize> {
        self.current_frame += 1;

        // 1. Calculate query bounding box for camera view distance
        // 1 deg lat ~ 111,000m, 1 deg lon ~ 108,000m
        let d_lat = self.config.view_distance_meters / 111_000.0;
        let d_lon = self.config.view_distance_meters / 108_000.0;

        let query_box = GeoBounds::new(
            camera_pos.lat - d_lat,
            camera_pos.lat + d_lat,
            camera_pos.lon - d_lon,
            camera_pos.lon + d_lon,
        );

        // 2. Query SQLite spatial index
        let candidates = self.db.query_bbox(&query_box)?;

        let mut chunks_loaded_this_frame = 0;

        // 3. Process candidate chunks
        for cand in candidates {
            let chunk_center = cand.bounds.center();
            let dist_lat = (chunk_center.lat - camera_pos.lat) * 111_000.0;
            let dist_lon = (chunk_center.lon - camera_pos.lon) * 108_000.0;
            let dist_meters = (dist_lat * dist_lat + dist_lon * dist_lon).sqrt();

            let target_lod = select_lod(dist_meters, &self.config);
            let key = ChunkKey::new(cand.chunk_x, cand.chunk_y, target_lod);

            // Check if already in cache
            if self.cache.get_mut(&key, self.current_frame).is_none() {
                // Load from disk
                let chunk_file_path = self.world_dir.join(&cand.storage_path);
                if chunk_file_path.exists() {
                    let chunk = read_chunk_file(&chunk_file_path)?;
                    self.cache.insert(key, chunk, self.current_frame);
                    chunks_loaded_this_frame += 1;
                }
            }
        }

        // 4. Memory budget eviction
        let threshold = self.config.eviction_threshold_bytes();
        if self.cache.total_memory_bytes() > threshold {
            // Camera approximate grid key
            let cam_x = (camera_pos.lon / 0.005).floor() as i32;
            let cam_y = (camera_pos.lat / 0.005).floor() as i32;
            self.cache.evict_to_budget(threshold, cam_x, cam_y);
        }

        Ok(chunks_loaded_this_frame)
    }

    /// Fetches all currently loaded active chunks for rendering or physics.
    pub fn get_active_chunks(&self) -> Vec<&WorldChunk> {
        self.cache.iter_chunks().map(|(_, c)| c).collect()
    }

    /// Returns cache telemetry: (loaded_count, memory_mb, hits, misses, evictions).
    pub fn telemetry(&self) -> (usize, f64, u64, u64, u64) {
        (
            self.cache.len(),
            self.cache.total_memory_mb(),
            self.cache.hits,
            self.cache.misses,
            self.cache.evictions,
        )
    }
}
