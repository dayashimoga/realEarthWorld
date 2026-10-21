//! Configuration settings for world streaming and memory budgets.

use serde::{Deserialize, Serialize};

/// Configuration parameters governing chunk streaming and cache budgets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Maximum resident memory budget in Megabytes (default: 1024 MB).
    pub max_memory_mb: usize,
    /// Maximum GPU buffer budget in Megabytes (default: 512 MB).
    pub max_gpu_memory_mb: usize,
    /// Memory threshold ratio at which eviction begins (default: 0.85 = 85%).
    pub eviction_threshold_ratio: f32,
    /// Maximum horizontal viewing distance in meters (default: 3500.0 m).
    pub view_distance_meters: f64,
    /// Distance threshold for transitioning from LOD0 to LOD1 (default: 1500.0 m).
    pub lod1_distance_meters: f64,
    /// Distance threshold for transitioning from LOD1 to LOD2 (default: 3000.0 m).
    pub lod2_distance_meters: f64,
    /// Deadband hysteresis distance buffer to eliminate LOD boundary popping (default: 100.0 m).
    pub lod_hysteresis_meters: f64,
    /// Forward lookahead time along velocity vector in seconds (default: 2.0 s).
    pub lookahead_seconds: f64,
    /// Bounded in-flight decompression queue limit for backpressure control (default: 16).
    pub max_in_flight_tasks: usize,
    /// Immediate radius around camera/vehicle where chunks are pinned for safe collision (default: 250.0 m).
    pub collision_critical_radius_meters: f64,
    /// Number of worker threads for decompression and background loading.
    pub worker_threads: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024,
            max_gpu_memory_mb: 512,
            eviction_threshold_ratio: 0.85,
            view_distance_meters: 3500.0,
            lod1_distance_meters: 1500.0,
            lod2_distance_meters: 3000.0,
            lod_hysteresis_meters: 100.0,
            lookahead_seconds: 2.0,
            max_in_flight_tasks: 16,
            collision_critical_radius_meters: 250.0,
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

    pub fn max_gpu_memory_bytes(&self) -> usize {
        self.max_gpu_memory_mb * 1024 * 1024
    }
}
