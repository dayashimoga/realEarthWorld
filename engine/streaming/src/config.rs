//! Configuration settings for world streaming and memory budgets.

use serde::{Deserialize, Serialize};

/// Configuration parameters governing chunk streaming and cache budgets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Maximum resident memory budget in Megabytes (default: 1024 MB).
    pub max_memory_mb: usize,
    /// Memory threshold ratio at which eviction begins (default: 0.85 = 85%).
    pub eviction_threshold_ratio: f32,
    /// Maximum horizontal viewing distance in meters (default: 3500.0 m).
    pub view_distance_meters: f64,
    /// Distance threshold for transitioning from LOD0 to LOD1 (default: 1500.0 m).
    pub lod1_distance_meters: f64,
    /// Number of worker threads for decompression and background loading.
    pub worker_threads: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024,
            eviction_threshold_ratio: 0.85,
            view_distance_meters: 3500.0,
            lod1_distance_meters: 1500.0,
            worker_threads: 4,
        }
    }
}

impl StreamingConfig {
    pub fn eviction_threshold_bytes(&self) -> usize {
        ((self.max_memory_mb as f64 * 1024.0 * 1024.0) * self.eviction_threshold_ratio as f64)
            as usize
    }

    pub fn max_memory_bytes(&self) -> usize {
        self.max_memory_mb * 1024 * 1024
    }
}
