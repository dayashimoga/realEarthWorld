//! Least Recently Used (LRU) memory-budgeted chunk cache.

use realearth_compiler::chunk::WorldChunk;
use std::collections::HashMap;

/// Unique coordinate key for a chunk in the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkKey {
    pub x: i32,
    pub y: i32,
    pub lod: u32,
}

impl ChunkKey {
    pub const fn new(x: i32, y: i32, lod: u32) -> Self {
        Self { x, y, lod }
    }
}

/// An entry residing in the active chunk cache.
#[derive(Debug, Clone)]
pub struct CachedChunk {
    pub chunk: WorldChunk,
    pub last_accessed_frame: u64,
    pub estimated_bytes: usize,
}

/// Memory-governed LRU chunk cache.
#[derive(Debug, Default)]
pub struct LruChunkCache {
    entries: HashMap<ChunkKey, CachedChunk>,
    total_memory_bytes: usize,
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

    pub fn total_memory_bytes(&self) -> usize {
        self.total_memory_bytes
    }

    pub fn total_memory_mb(&self) -> f64 {
        self.total_memory_bytes as f64 / (1024.0 * 1024.0)
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

    /// Inserts a chunk into the cache, estimating its heap memory consumption.
    pub fn insert(&mut self, key: ChunkKey, chunk: WorldChunk, current_frame: u64) {
        // Approximate memory: vertex buffers + index buffers + metadata
        let vertex_bytes =
            chunk.total_vertices() * std::mem::size_of::<realearth_compiler::mesh::Vertex>();
        let index_bytes = chunk.total_triangles() * 3 * std::mem::size_of::<u32>();
        let meta_bytes = chunk.building_metadata.len() * 128;
        let estimated_bytes = vertex_bytes + index_bytes + meta_bytes + 2048;

        if let Some(old) = self.entries.insert(
            key,
            CachedChunk {
                chunk,
                last_accessed_frame: current_frame,
                estimated_bytes,
            },
        ) {
            self.total_memory_bytes = self.total_memory_bytes.saturating_sub(old.estimated_bytes);
        }

        self.total_memory_bytes += estimated_bytes;
    }

    /// Evicts chunks until memory is below `target_bytes`, prioritizing oldest and farthest chunks.
    pub fn evict_to_budget(
        &mut self,
        target_bytes: usize,
        camera_chunk_x: i32,
        camera_chunk_y: i32,
    ) -> usize {
        if self.total_memory_bytes <= target_bytes {
            return 0;
        }

        // Collect keys and sort by access age and distance from camera
        let mut candidates: Vec<(ChunkKey, u64, i64)> = self
            .entries
            .iter()
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
            if self.total_memory_bytes <= target_bytes {
                break;
            }
            if let Some(removed) = self.entries.remove(&key) {
                self.total_memory_bytes = self
                    .total_memory_bytes
                    .saturating_sub(removed.estimated_bytes);
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
