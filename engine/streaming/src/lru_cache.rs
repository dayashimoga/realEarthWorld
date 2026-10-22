//! Least Recently Used (LRU) memory-budgeted chunk cache with dual CPU/GPU accounting
//! and safe collision-critical chunk pinning.

use realearth_compiler::chunk::WorldChunk;
use std::collections::HashMap;

/// Unique coordinate key for a chunk in the cache, optionally namespaced by region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkKey {
    pub x: i32,
    pub y: i32,
    pub lod: u32,
    pub region_hash: u32,
}

impl ChunkKey {
    pub const fn new(x: i32, y: i32, lod: u32) -> Self {
        Self {
            x,
            y,
            lod,
            region_hash: 0,
        }
    }

    /// Creates a ChunkKey namespaced to a specific region ID to avoid inter-region chunk coordinate collisions.
    pub fn for_region(region_id: &str, x: i32, y: i32, lod: u32) -> Self {
        let mut h = 0x811c_9dc5u32;
        for byte in region_id.as_bytes() {
            h ^= *byte as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
        Self {
            x,
            y,
            lod,
            region_hash: h,
        }
    }
}

/// An entry residing in the active chunk cache.
#[derive(Debug, Clone)]
pub struct CachedChunk {
    pub chunk: WorldChunk,
    pub last_accessed_frame: u64,
    /// Estimated CPU memory consumption in bytes (RAM).
    pub cpu_bytes: usize,
    /// Estimated GPU memory consumption in bytes (VRAM).
    pub gpu_bytes: usize,
    /// Collision-critical pinning flag. If true, eviction MUST NEVER unload this chunk.
    pub pinned: bool,
    /// Flag indicating whether this chunk was generated as a safe fallback upon corruption.
    pub is_fallback: bool,
}

/// Memory-governed LRU chunk cache with dual CPU/GPU budgets and pinning guarantees.
#[derive(Debug, Default)]
pub struct LruChunkCache {
    entries: HashMap<ChunkKey, CachedChunk>,
    total_cpu_bytes: usize,
    total_gpu_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl LruChunkCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn total_cpu_bytes(&self) -> usize {
        self.total_cpu_bytes
    }

    pub fn total_cpu_mb(&self) -> f64 {
        self.total_cpu_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn total_gpu_bytes(&self) -> usize {
        self.total_gpu_bytes
    }

    pub fn total_gpu_mb(&self) -> f64 {
        self.total_gpu_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn total_memory_bytes(&self) -> usize {
        self.total_cpu_bytes + self.total_gpu_bytes
    }

    pub fn total_memory_mb(&self) -> f64 {
        self.total_memory_bytes() as f64 / (1024.0 * 1024.0)
    }

    /// Fetches a reference to a cached chunk, marking it accessed at `current_frame`.
    pub fn get_mut(&mut self, key: &ChunkKey, current_frame: u64) -> Option<&WorldChunk> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_accessed_frame = current_frame;
            self.hits += 1;
            Some(&entry.chunk)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Fetches a read-only reference without marking access.
    pub fn get(&self, key: &ChunkKey) -> Option<&WorldChunk> {
        self.entries.get(key).map(|e| &e.chunk)
    }

    /// Checks if a chunk is pinned.
    pub fn is_pinned(&self, key: &ChunkKey) -> bool {
        self.entries.get(key).map(|e| e.pinned).unwrap_or(false)
    }

    /// Sets or clears the collision-critical pinning status for a chunk.
    pub fn set_pinned(&mut self, key: &ChunkKey, pinned: bool) -> bool {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.pinned = pinned;
            true
        } else {
            false
        }
    }

    /// Number of currently pinned chunks.
    pub fn pinned_count(&self) -> usize {
        self.entries.values().filter(|e| e.pinned).count()
    }

    /// Inserts a chunk into the cache, estimating its CPU and GPU memory consumption.
    pub fn insert(&mut self, key: ChunkKey, chunk: WorldChunk, current_frame: u64) {
        self.insert_with_options(key, chunk, current_frame, false, false);
    }

    /// Inserts a chunk with explicit pinning and corruption fallback flags.
    pub fn insert_with_options(
        &mut self,
        key: ChunkKey,
        chunk: WorldChunk,
        current_frame: u64,
        pinned: bool,
        is_fallback: bool,
    ) {
        // Vertex memory (Float32x3 pos + Float32x3 normal + Float32x2 uv + Float32x4 color = 48 bytes)
        let vertex_bytes =
            chunk.total_vertices() * std::mem::size_of::<realearth_compiler::mesh::Vertex>();
        // Index memory (u32 indices = 4 bytes per index)
        let index_bytes = chunk.total_triangles() * 3 * std::mem::size_of::<u32>();
        // CPU Metadata and road structures
        let meta_bytes = chunk.building_metadata.len() * 128;
        let cpu_bytes = vertex_bytes + index_bytes + meta_bytes + 2048;

        // GPU VRAM allocation: vertex buffers + index buffers mirrored in device memory
        let gpu_bytes = vertex_bytes + index_bytes;

        if let Some(old) = self.entries.insert(
            key,
            CachedChunk {
                chunk,
                last_accessed_frame: current_frame,
                cpu_bytes,
                gpu_bytes,
                pinned,
                is_fallback,
            },
        ) {
            self.total_cpu_bytes = self.total_cpu_bytes.saturating_sub(old.cpu_bytes);
            self.total_gpu_bytes = self.total_gpu_bytes.saturating_sub(old.gpu_bytes);
        }

        self.total_cpu_bytes += cpu_bytes;
        self.total_gpu_bytes += gpu_bytes;
    }

    /// Evicts unpinned chunks until CPU memory is below `target_bytes`, prioritizing oldest and farthest chunks.
    /// Pinned collision-critical chunks are guaranteed NEVER to be evicted.
    pub fn evict_to_budget(
        &mut self,
        target_bytes: usize,
        camera_chunk_x: i32,
        camera_chunk_y: i32,
    ) -> usize {
        if self.total_cpu_bytes <= target_bytes {
            return 0;
        }

        // Collect keys of UNPINNED candidates sorted by access age and distance from camera
        let mut candidates: Vec<(ChunkKey, u64, i64)> = self
            .entries
            .iter()
            .filter(|(_, entry)| !entry.pinned)
            .map(|(key, entry)| {
                let dx = (key.x - camera_chunk_x) as i64;
                let dy = (key.y - camera_chunk_y) as i64;
                let dist_sq = dx * dx + dy * dy;
                (*key, entry.last_accessed_frame, dist_sq)
            })
            .collect();

        // Sort: oldest frame first, then farthest distance
        candidates.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| b.2.cmp(&a.2)));

        let mut evicted_count = 0;
        for (key, _, _) in candidates {
            if self.total_cpu_bytes <= target_bytes {
                break;
            }
            if let Some(removed) = self.entries.remove(&key) {
                self.total_cpu_bytes = self.total_cpu_bytes.saturating_sub(removed.cpu_bytes);
                self.total_gpu_bytes = self.total_gpu_bytes.saturating_sub(removed.gpu_bytes);
                self.evictions += 1;
                evicted_count += 1;
            }
        }

        evicted_count
    }

    /// Returns an iterator over all currently loaded chunks.
    pub fn iter_chunks(&self) -> impl Iterator<Item = (&ChunkKey, &WorldChunk)> {
        self.entries.iter().map(|(k, v)| (k, &v.chunk))
    }
}
