//! World Streamer orchestrating SQLite spatial queries, predictive prefetching,
//! Rayon parallel decoding, bounded backpressure, and collision-critical pinning.

use crate::async_loader::{
    generate_demand_terrain_chunk, read_chunk_with_corruption_recovery, ChunkLoadQueue,
    ChunkLoadRequest,
};
use crate::config::StreamingConfig;
use crate::lod::select_lod_with_hysteresis;
use crate::lru_cache::{ChunkKey, LruChunkCache};
use crate::prefetch::{calculate_prefetch_bounds, evaluate_chunk_priority, ChunkPriority};
use rayon::prelude::*;
use realearth_compiler::chunk::WorldChunk;
use realearth_compiler::database::ChunkDatabase;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::{RealEarthError, Result};
use std::path::{Path, PathBuf};

/// Detailed streaming runtime metrics and telemetry snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamingTelemetry {
    pub active_chunks: usize,
    pub cpu_memory_mb: f64,
    pub gpu_memory_mb: f64,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub pinned_chunks: usize,
    pub pending_queue: usize,
    pub cancellations: u64,
    pub fallbacks: u64,
    pub starvations: u64,
    pub missing_sources: u64,
    pub corrupted_chunks: u64,
    pub certified_bounds: Option<GeoBounds>,
}

/// Production spatial world streamer maintaining active chunks around the observer.
pub struct WorldStreamer {
    pub config: StreamingConfig,
    world_dir: PathBuf,
    db: ChunkDatabase,
    cache: LruChunkCache,
    load_queue: ChunkLoadQueue,
    current_frame: u64,
    last_camera_pos: Option<Wgs84Coord>,
    last_velocity: glam::Vec3,
    pub total_starvations: u64,
    pub total_missing_sources: u64,
    pub total_corrupted_chunks: u64,
    pub certified_bounds: Option<GeoBounds>,
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
        let load_queue = ChunkLoadQueue::new(&config);

        let certified_bounds = match db.get_first_package_manifest() {
            Ok(Some(manifest)) => Some(manifest.bounds),
            _ => None,
        };

        Ok(Self {
            config,
            world_dir: world_dir.to_path_buf(),
            db,
            cache,
            load_queue,
            current_frame: 0,
            last_camera_pos: None,
            last_velocity: glam::Vec3::ZERO,
            total_starvations: 0,
            total_missing_sources: 0,
            total_corrupted_chunks: 0,
            certified_bounds,
        })
    }

    /// Primary streaming update step called every frame or simulation tick.
    pub fn update(&mut self, camera_pos: Wgs84Coord) -> Result<usize> {
        // Derive velocity automatically from position delta if available
        let velocity = if let Some(last_pos) = self.last_camera_pos {
            let lat_rad = camera_pos.lat.to_radians();
            let m_per_deg_lat = 111_000.0;
            let m_per_deg_lon = 111_320.0 * lat_rad.cos().abs().max(0.1);

            let dx = ((camera_pos.lon - last_pos.lon) * m_per_deg_lon) * 60.0;
            let dz = ((camera_pos.lat - last_pos.lat) * m_per_deg_lat) * 60.0;
            let dy = (camera_pos.alt - last_pos.alt) * 60.0;

            glam::Vec3::new(dx as f32, dy as f32, dz as f32)
        } else {
            glam::Vec3::ZERO
        };

        self.update_with_velocity(camera_pos, velocity)
    }

    /// Explicit velocity- and direction-aware streaming update.
    pub fn update_with_velocity(
        &mut self,
        camera_pos: Wgs84Coord,
        velocity_mps: glam::Vec3,
    ) -> Result<usize> {
        self.current_frame += 1;
        self.last_velocity = velocity_mps;

        // Check for teleportation (> 500m jump in one frame)
        if let Some(prev) = self.last_camera_pos {
            let d_lat = (camera_pos.lat - prev.lat) * 111_000.0;
            let d_lon = (camera_pos.lon - prev.lon) * 108_000.0;
            let dist_jump = (d_lat * d_lat + d_lon * d_lon).sqrt();

            if dist_jump > 500.0 {
                // Teleportation detected: cancel non-critical stale prefetch requests
                let cancelled = self.load_queue.cancel_non_critical();
                tracing::info!(
                    "Camera teleportation detected ({:.1}m jump). Cancelled {} pending prefetch requests.",
                    dist_jump,
                    cancelled
                );
            }
        }
        self.last_camera_pos = Some(camera_pos);

        // 1. Calculate anisotropic prefetch bounding box (view circle + forward velocity cone)
        let query_box = calculate_prefetch_bounds(camera_pos, velocity_mps, &self.config);

        // 2. Query SQLite spatial index
        let candidates = self.db.query_bbox(&query_box)?;

        // 3. Evaluate priorities and enqueue missing chunks
        for cand in candidates {
            let chunk_center = cand.bounds.center();
            let dist_lat = (chunk_center.lat - camera_pos.lat) * 111_000.0;
            let dist_lon = (chunk_center.lon - camera_pos.lon) * 108_000.0;
            let dist_meters = (dist_lat * dist_lat + dist_lon * dist_lon).sqrt();

            let target_lod = select_lod_with_hysteresis(dist_meters, None, &self.config);
            let key = ChunkKey::new(cand.chunk_x, cand.chunk_y, target_lod);

            let priority =
                evaluate_chunk_priority(&cand.bounds, camera_pos, velocity_mps, &self.config);

            // If chunk is already loaded in cache
            if self.cache.get_mut(&key, self.current_frame).is_some() {
                // Pin collision-critical chunks
                if priority == ChunkPriority::CriticalCollision {
                    self.cache.set_pinned(&key, true);
                }
            } else {
                // Enqueue for loading
                let storage_path = self.world_dir.join(&cand.storage_path);
                self.load_queue.push(ChunkLoadRequest {
                    priority,
                    chunk_x: cand.chunk_x,
                    chunk_y: cand.chunk_y,
                    lod_level: target_lod,
                    storage_path,
                    bounds: cand.bounds,
                    request_frame: self.current_frame,
                });
            }
        }

        // 4. Batch parallel decompression with Rayon (bounded backpressure)
        let mut batch = Vec::new();
        while batch.len() < self.config.max_in_flight_tasks {
            if let Some(req) = self.load_queue.pop() {
                let key = ChunkKey::new(req.chunk_x, req.chunk_y, req.lod_level);
                if self.cache.get(&key).is_none() {
                    batch.push(req);
                }
            } else {
                break;
            }
        }

        let mut loaded_this_frame = 0;
        if !batch.is_empty() {
            // Decompress in parallel across Rayon worker threads
            let results: Vec<_> = batch
                .par_iter()
                .filter_map(|req| {
                    match read_chunk_with_corruption_recovery(
                        &req.storage_path,
                        req.chunk_x,
                        req.chunk_y,
                        req.lod_level,
                        &req.bounds,
                    ) {
                        Ok(res) => Some((req.priority, res)),
                        Err(e) => {
                            tracing::warn!("Failed to read chunk: {}", e);
                            None
                        }
                    }
                })
                .collect();

            for (priority, res) in results {
                let key = ChunkKey::new(res.chunk_x, res.chunk_y, res.lod_level);
                let pinned = priority == ChunkPriority::CriticalCollision;
                self.cache.insert_with_options(
                    key,
                    res.chunk,
                    self.current_frame,
                    pinned,
                    res.is_fallback,
                );
                loaded_this_frame += 1;
                self.load_queue.total_loaded += 1;
                if res.is_fallback {
                    self.load_queue.total_fallbacks += 1;
                    self.total_corrupted_chunks += 1;
                }
            }
        }

        // 5. Authentic Emergency Pinning & Demand-Driven Terrain Generation
        // If observer position has no chunk covering it (e.g. fast traversal or edge of loaded area),
        // first check SQLite spatial index synchronously for authentic surveyed chunk.
        if !self.is_collision_ready(camera_pos) {
            let margin = 0.002; // ~220m immediate proximity margin
            let near_box = GeoBounds::new(
                camera_pos.lat - margin,
                camera_pos.lat + margin,
                camera_pos.lon - margin,
                camera_pos.lon + margin,
            );
            if let Ok(cands) = self.db.query_bbox(&near_box) {
                for cand in cands {
                    let key = ChunkKey::new(cand.chunk_x, cand.chunk_y, 0);
                    if self.cache.get(&key).is_none() {
                        let path = self.world_dir.join(&cand.storage_path);
                        if let Ok(res) = read_chunk_with_corruption_recovery(
                            &path,
                            cand.chunk_x,
                            cand.chunk_y,
                            0,
                            &cand.bounds,
                        ) {
                            if res.is_fallback {
                                self.total_corrupted_chunks += 1;
                                self.load_queue.total_fallbacks += 1;
                            }
                            self.cache.insert_with_options(
                                key,
                                res.chunk,
                                self.current_frame,
                                true,
                                res.is_fallback,
                            );
                            loaded_this_frame += 1;
                            self.load_queue.total_loaded += 1;
                        }
                    }
                }
            }
        }

        // If observer position still has no collision chunk after synchronous database query:
        if !self.is_collision_ready(camera_pos) {
            // Check if observer is inside certified bounds (starvation) or outside (missing source)
            if self.is_within_certified_bounds(camera_pos) {
                self.total_starvations += 1;
            } else {
                self.total_missing_sources += 1;
            }

            let chunk_deg = 0.005; // ~500m
            let center_cx = (camera_pos.lon / chunk_deg).floor() as i32;
            let center_cy = (camera_pos.lat / chunk_deg).floor() as i32;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let cx = center_cx + dx;
                    let cy = center_cy + dy;
                    let key = ChunkKey::new(cx, cy, 0);
                    if self.cache.get(&key).is_none() {
                        let min_lat = cy as f64 * chunk_deg;
                        let max_lat = (cy + 1) as f64 * chunk_deg;
                        let min_lon = cx as f64 * chunk_deg;
                        let max_lon = (cx + 1) as f64 * chunk_deg;
                        let bounds = GeoBounds::new(min_lat, max_lat, min_lon, max_lon);
                        // Demand-driven terrain generation with relief and slope normals rather than flat 0.0m planes
                        let demand_chunk = generate_demand_terrain_chunk(cx, cy, 0, &bounds, None);
                        self.cache.insert_with_options(
                            key,
                            demand_chunk,
                            self.current_frame,
                            dx == 0 && dy == 0,
                            true,
                        );
                        loaded_this_frame += 1;
                        self.load_queue.total_fallbacks += 1;
                    }
                }
            }
        }

        // 6. Memory budget eviction (CPU & GPU)
        let threshold_bytes = self.config.eviction_threshold_bytes();
        if self.cache.total_cpu_bytes() > threshold_bytes {
            let cam_x = (camera_pos.lon / 0.005).floor() as i32;
            let cam_y = (camera_pos.lat / 0.005).floor() as i32;
            self.cache.evict_to_budget(threshold_bytes, cam_x, cam_y);
        }

        Ok(loaded_this_frame)
    }

    /// Instant teleportation to coordinates: aborts stale prefetch work.
    pub fn teleport(&mut self, target_pos: Wgs84Coord) {
        self.load_queue.cancel_non_critical();
        self.last_camera_pos = Some(target_pos);
        self.last_velocity = glam::Vec3::ZERO;
    }

    /// Checks if terrain and collision data under a coordinate is fully loaded and ready for physics.
    pub fn is_collision_ready(&self, pos: Wgs84Coord) -> bool {
        for (_, chunk) in self.cache.iter_chunks() {
            if chunk.header.bounds.contains(&pos) && !chunk.terrain.is_empty() {
                return true;
            }
        }
        false
    }

    /// Explicitly pins or unpins a chunk.
    pub fn set_pinned(&mut self, key: &ChunkKey, pinned: bool) -> bool {
        self.cache.set_pinned(key, pinned)
    }

    /// Fetches all currently loaded active chunks for rendering or physics.
    pub fn get_active_chunks(&self) -> Vec<&WorldChunk> {
        self.cache.iter_chunks().map(|(_, c)| c).collect()
    }

    /// Checks whether a coordinate lies within the certified bounds of the loaded package.
    pub fn is_within_certified_bounds(&self, coord: Wgs84Coord) -> bool {
        if let Some(bounds) = &self.certified_bounds {
            bounds.contains(&coord)
        } else {
            false
        }
    }

    /// Returns certified bounds of the loaded package, if available in the manifest.
    pub fn get_certified_bounds(&self) -> Option<GeoBounds> {
        self.certified_bounds
    }

    /// Returns comprehensive cache telemetry snapshot.
    pub fn telemetry_details(&self) -> StreamingTelemetry {
        StreamingTelemetry {
            active_chunks: self.cache.len(),
            cpu_memory_mb: self.cache.total_cpu_mb(),
            gpu_memory_mb: self.cache.total_gpu_mb(),
            hits: self.cache.hits,
            misses: self.cache.misses,
            evictions: self.cache.evictions,
            pinned_chunks: self.cache.pinned_count(),
            pending_queue: self.load_queue.pending_count(),
            cancellations: self.load_queue.total_cancellations,
            fallbacks: self.load_queue.total_fallbacks,
            starvations: self.total_starvations,
            missing_sources: self.total_missing_sources,
            corrupted_chunks: self.total_corrupted_chunks,
            certified_bounds: self.certified_bounds,
        }
    }

    /// Returns legacy 5-tuple cache telemetry: (loaded_count, memory_mb, hits, misses, evictions).
    pub fn telemetry(&self) -> (usize, f64, u64, u64, u64) {
        (
            self.cache.len(),
            self.cache.total_memory_mb(),
            self.cache.hits,
            self.cache.misses,
            self.cache.evictions,
        )
    }

    /// Returns total stored geographic features indexed in the regional database.
    pub fn total_stored_features(&self) -> usize {
        self.db.total_feature_count().unwrap_or(0)
    }

    /// Returns total stored chunk records indexed in the regional database.
    pub fn total_stored_chunks(&self) -> usize {
        self.db.count_chunks().unwrap_or(0)
    }
}
