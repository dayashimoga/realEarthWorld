//! Multi-Region World Streamer coordinating spatial queries, predictive prefetching,
//! and unified LRU memory budget across multiple decentralized regional packages.

use crate::async_loader::{read_chunk_with_corruption_recovery, ChunkLoadQueue, ChunkLoadRequest};
use crate::config::StreamingConfig;
use crate::lod::select_lod_with_hysteresis;
use crate::lru_cache::{ChunkKey, LruChunkCache};
use crate::prefetch::{calculate_prefetch_bounds, evaluate_chunk_priority, ChunkPriority};
use rayon::prelude::*;
use realearth_compiler::chunk::WorldChunk;
use realearth_compiler::database::ChunkDatabase;
use realearth_compiler::regional_catalog::RegionalCatalog;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::{RealEarthError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A mounted regional database inside the multi-region streaming system.
pub struct MountedRegion {
    pub id: String,
    pub world_dir: PathBuf,
    pub bounds: GeoBounds,
    pub db: ChunkDatabase,
}

/// Telemetry metrics for multi-region streaming runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiRegionTelemetry {
    pub mounted_regions_count: usize,
    pub active_chunks: usize,
    pub cpu_memory_mb: f64,
    pub gpu_memory_mb: f64,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub pinned_chunks: usize,
    pub pending_queue: usize,
    pub active_region_ids: Vec<String>,
}

/// Production multi-region streamer coordinating continuous vehicle navigation
/// and world streaming across multiple independent regional databases.
pub struct MultiRegionStreamer {
    pub config: StreamingConfig,
    regions: HashMap<String, MountedRegion>,
    cache: LruChunkCache,
    load_queue: ChunkLoadQueue,
    current_frame: u64,
    last_camera_pos: Option<Wgs84Coord>,
    last_velocity: glam::Vec3,
    chunk_to_region: HashMap<(i32, i32, u32), String>,
}

impl MultiRegionStreamer {
    /// Creates a new empty MultiRegionStreamer with the specified streaming configuration.
    pub fn new(config: StreamingConfig) -> Self {
        let load_queue = ChunkLoadQueue::new(&config);
        Self {
            config,
            regions: HashMap::new(),
            cache: LruChunkCache::new(),
            load_queue,
            current_frame: 0,
            last_camera_pos: None,
            last_velocity: glam::Vec3::ZERO,
            chunk_to_region: HashMap::new(),
        }
    }

    /// Initializes a MultiRegionStreamer from all installed packages in a RegionalCatalog.
    pub fn from_catalog(catalog: &RegionalCatalog, config: StreamingConfig) -> Result<Self> {
        let mut streamer = Self::new(config);
        for manifest in catalog.list_installed() {
            let pkg_dir = catalog.package_dir(&manifest.region_id);
            streamer.mount_region(&manifest.region_id, &pkg_dir, manifest.bounds)?;
        }
        Ok(streamer)
    }

    /// Mounts an individual regional package into the multi-region streaming coordinator.
    pub fn mount_region(
        &mut self,
        region_id: &str,
        world_dir: &Path,
        bounds: GeoBounds,
    ) -> Result<()> {
        let db_path = world_dir.join("chunks.db");
        if !db_path.exists() {
            return Err(RealEarthError::Streaming(format!(
                "chunks.db not found for region '{}' in {:?}",
                region_id, world_dir
            )));
        }

        let db = ChunkDatabase::open(&db_path)?;
        self.regions.insert(
            region_id.to_string(),
            MountedRegion {
                id: region_id.to_string(),
                world_dir: world_dir.to_path_buf(),
                bounds,
                db,
            },
        );

        Ok(())
    }

    /// Unmounts a region from active streaming.
    pub fn unmount_region(&mut self, region_id: &str) {
        self.regions.remove(region_id);
    }

    /// Primary multi-region update step called every frame or simulation tick.
    /// Evaluates which regions intersect the observer lookahead cone and streams chunks from all of them.
    pub fn update(&mut self, camera_pos: Wgs84Coord) -> Result<usize> {
        let velocity = if let Some(last_pos) = self.last_camera_pos {
            let lat_rad = camera_pos.lat.to_radians();
            let m_per_deg_lat = 111_000.0;
            let m_per_deg_lon = 111_320.0 * lat_rad.cos().abs().max(0.1);

            let dx = ((camera_pos.lon - last_pos.lon) * m_per_deg_lon) * 60.0;
            let dz = ((camera_pos.lat - last_pos.lat) * m_per_deg_lat) * 60.0;
            let dy = (camera_pos.alt - last_pos.alt) * 60.0;
            glam::Vec3::new(dx as f32, dy as f32, dz as f32)
        } else {
            self.last_velocity
        };

        self.last_camera_pos = Some(camera_pos);
        self.last_velocity = velocity;
        self.current_frame = self.current_frame.wrapping_add(1);

        // 1. Calculate anisotropic prefetch bounding box
        let prefetch_bounds = calculate_prefetch_bounds(camera_pos, velocity, &self.config);

        // 2. Identify all mounted regions overlapping the prefetch bounding box
        for region in self.regions.values() {
            if !(region.bounds.max_lat < prefetch_bounds.min_lat
                || region.bounds.min_lat > prefetch_bounds.max_lat
                || region.bounds.max_lon < prefetch_bounds.min_lon
                || region.bounds.min_lon > prefetch_bounds.max_lon)
            {
                if let Ok(candidates) = region.db.query_bbox(&prefetch_bounds) {
                    for cand in candidates {
                        let center = cand.bounds.center();
                        let dist_lat = (center.lat - camera_pos.lat) * 111_000.0;
                        let dist_lon = (center.lon - camera_pos.lon) * 108_000.0;
                        let dist_meters = (dist_lat * dist_lat + dist_lon * dist_lon).sqrt();

                        let target_lod =
                            select_lod_with_hysteresis(dist_meters, None, &self.config);
                        let key = ChunkKey::for_region(
                            &region.id,
                            cand.chunk_x,
                            cand.chunk_y,
                            target_lod,
                        );

                        if self.cache.get_mut(&key, self.current_frame).is_some() {
                            continue;
                        }

                        let priority = evaluate_chunk_priority(
                            &cand.bounds,
                            camera_pos,
                            velocity,
                            &self.config,
                        );
                        let storage_path = region.world_dir.join(&cand.storage_path);
                        let req = ChunkLoadRequest {
                            priority,
                            chunk_x: cand.chunk_x,
                            chunk_y: cand.chunk_y,
                            lod_level: target_lod,
                            storage_path,
                            bounds: cand.bounds,
                            request_frame: self.current_frame,
                        };

                        self.chunk_to_region
                            .insert((cand.chunk_x, cand.chunk_y, target_lod), region.id.clone());
                        self.load_queue.push(req);
                    }
                }
            }
        }

        // 3. Batch parallel decompression with Rayon
        let mut batch = Vec::new();
        while batch.len() < self.config.max_in_flight_tasks {
            if let Some(req) = self.load_queue.pop() {
                let region_id = self
                    .chunk_to_region
                    .get(&(req.chunk_x, req.chunk_y, req.lod_level))
                    .cloned()
                    .unwrap_or_default();
                let key = ChunkKey::for_region(&region_id, req.chunk_x, req.chunk_y, req.lod_level);
                if self.cache.get(&key).is_none() {
                    batch.push((region_id, req));
                }
            } else {
                break;
            }
        }

        let mut loaded_this_frame = 0;
        if !batch.is_empty() {
            let results: Vec<_> = batch
                .par_iter()
                .filter_map(|(reg_id, req)| {
                    match read_chunk_with_corruption_recovery(
                        &req.storage_path,
                        req.chunk_x,
                        req.chunk_y,
                        req.lod_level,
                        &req.bounds,
                    ) {
                        Ok(res) => Some((reg_id.clone(), req.priority, res)),
                        Err(e) => {
                            tracing::warn!("Failed to read chunk: {}", e);
                            None
                        }
                    }
                })
                .collect();

            for (reg_id, priority, res) in results {
                let key = ChunkKey::for_region(&reg_id, res.chunk_x, res.chunk_y, res.lod_level);
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
                }
            }
        }

        // 4. Global memory budget eviction across all loaded regions
        let threshold_bytes = self.config.eviction_threshold_bytes();
        if self.cache.total_cpu_bytes() > threshold_bytes {
            let cam_x = (camera_pos.lon / 0.005).floor() as i32;
            let cam_y = (camera_pos.lat / 0.005).floor() as i32;
            self.cache.evict_to_budget(threshold_bytes, cam_x, cam_y);
        }

        Ok(loaded_this_frame)
    }

    /// Sets or clears collision-critical pinning on a chunk.
    pub fn set_pinned(
        &mut self,
        region_id: &str,
        cx: i32,
        cy: i32,
        lod: u32,
        pinned: bool,
    ) -> bool {
        let key = ChunkKey::for_region(region_id, cx, cy, lod);
        self.cache.set_pinned(&key, pinned)
    }

    /// Checks whether a specific chunk is resident in the LRU cache.
    pub fn is_chunk_cached(&self, region_id: &str, cx: i32, cy: i32, lod: u32) -> bool {
        let key = ChunkKey::for_region(region_id, cx, cy, lod);
        self.cache.get(&key).is_some()
    }

    /// Returns references to all currently loaded chunks across all regions in RAM.
    pub fn active_chunks(&self) -> Vec<&WorldChunk> {
        self.cache.iter_chunks().map(|(_, chunk)| chunk).collect()
    }

    /// Current count of loaded chunks across all active regions.
    pub fn loaded_chunk_count(&self) -> usize {
        self.cache.len()
    }

    /// Extracts real-time multi-region streaming telemetry snapshot.
    pub fn telemetry(&self) -> MultiRegionTelemetry {
        let mut active_region_ids: Vec<String> = self.regions.keys().cloned().collect();
        active_region_ids.sort();
        MultiRegionTelemetry {
            mounted_regions_count: self.regions.len(),
            active_chunks: self.cache.len(),
            cpu_memory_mb: self.cache.total_cpu_mb(),
            gpu_memory_mb: self.cache.total_gpu_mb(),
            hits: self.cache.hits,
            misses: self.cache.misses,
            evictions: self.cache.evictions,
            pinned_chunks: self.cache.pinned_count(),
            pending_queue: self.load_queue.pending_count(),
            active_region_ids,
        }
    }
}
