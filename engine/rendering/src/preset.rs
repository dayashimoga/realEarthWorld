//! Graphics quality presets and dynamic adaptive frame-time regulation.

use serde::{Deserialize, Serialize};

/// Graphics quality presets tailored for different hardware capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphicsPresetLevel {
    Low,
    Medium,
    High,
    Adaptive,
}

/// Dynamic graphics settings governed by preset or adaptive regulator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub level: GraphicsPresetLevel,
    /// Maximum rendering view distance in meters.
    pub view_distance_meters: f64,
    /// Distance threshold where LOD0 simplifies to LOD1.
    pub lod1_distance_meters: f64,
    /// Atmospheric fog density factor.
    pub fog_density: f32,
    /// Specular reflection intensity for water and glass roofs.
    pub specular_intensity: f32,
    /// Enable detailed road lane markings in fragment shader.
    pub road_markings_enabled: bool,
    /// Target frame time in milliseconds (default: 16.67 ms = 60 FPS).
    pub target_frame_time_ms: f32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self::from_preset(GraphicsPresetLevel::High)
    }
}

impl GraphicsSettings {
    /// Construct settings from a predefined preset level.
    pub fn from_preset(level: GraphicsPresetLevel) -> Self {
        match level {
            GraphicsPresetLevel::Low => Self {
                level,
                view_distance_meters: 1500.0,
                lod1_distance_meters: 800.0,
                fog_density: 0.0004,
                specular_intensity: 0.1,
                road_markings_enabled: false,
                target_frame_time_ms: 16.67,
            },
            GraphicsPresetLevel::Medium => Self {
                level,
                view_distance_meters: 2500.0,
                lod1_distance_meters: 1500.0,
                fog_density: 0.0002,
                specular_intensity: 0.45,
                road_markings_enabled: true,
                target_frame_time_ms: 16.67,
            },
            GraphicsPresetLevel::High => Self {
                level,
                view_distance_meters: 4000.0,
                lod1_distance_meters: 2200.0,
                fog_density: 0.00015,
                specular_intensity: 0.8,
                road_markings_enabled: true,
                target_frame_time_ms: 16.67,
            },
            GraphicsPresetLevel::Adaptive => Self {
                level,
                view_distance_meters: 3000.0,
                lod1_distance_meters: 1800.0,
                fog_density: 0.0002,
                specular_intensity: 0.6,
                road_markings_enabled: true,
                target_frame_time_ms: 16.67,
            },
        }
    }

    /// Dynamically adjusts view distance and LOD thresholds to maintain target 60 FPS.
    pub fn adapt_to_frame_time(&mut self, rolling_avg_frame_time_ms: f32) {
        if self.level != GraphicsPresetLevel::Adaptive {
            return;
        }

        if rolling_avg_frame_time_ms > 16.0 {
            // Frame time too high: degrade view distance and LOD to sustain 60 FPS
            self.view_distance_meters = (self.view_distance_meters - 100.0).max(1200.0);
            self.lod1_distance_meters = (self.lod1_distance_meters - 50.0).max(600.0);
        } else if rolling_avg_frame_time_ms < 12.0 {
            // Surplus frame time: scale up fidelity
            self.view_distance_meters = (self.view_distance_meters + 50.0).min(4500.0);
            self.lod1_distance_meters = (self.lod1_distance_meters + 25.0).min(2500.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_definitions() {
        let low = GraphicsSettings::from_preset(GraphicsPresetLevel::Low);
        let high = GraphicsSettings::from_preset(GraphicsPresetLevel::High);

        assert!(low.view_distance_meters < high.view_distance_meters);
        assert!(low.lod1_distance_meters < high.lod1_distance_meters);
        assert!(!low.road_markings_enabled);
        assert!(high.road_markings_enabled);
    }

    #[test]
    fn test_adaptive_frame_time_regulator() {
        let mut adaptive = GraphicsSettings::from_preset(GraphicsPresetLevel::Adaptive);
        let initial_dist = adaptive.view_distance_meters;

        // Simulate frame stall (22 ms > 16 ms)
        adaptive.adapt_to_frame_time(22.0);
        assert!(adaptive.view_distance_meters < initial_dist);

        // Simulate fast frames (8 ms < 12 ms)
        adaptive.adapt_to_frame_time(8.0);
        adaptive.adapt_to_frame_time(8.0);
        assert!(adaptive.view_distance_meters > initial_dist - 100.0);
    }
}
