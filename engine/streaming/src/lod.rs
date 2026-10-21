//! Level of Detail (LOD) selection heuristics.

use crate::config::StreamingConfig;

/// Determines the target Level of Detail (LOD) for a given distance from the camera in meters.
pub fn select_lod(distance_meters: f64, config: &StreamingConfig) -> u32 {
    if distance_meters <= config.lod1_distance_meters {
        0 // Highest detail
    } else {
        1 // Coarse/simplified
    }
}
