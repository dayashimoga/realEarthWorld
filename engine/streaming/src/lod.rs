//! Level of Detail (LOD) selection heuristics and distance hysteresis.

use crate::config::StreamingConfig;

/// Determines the target Level of Detail (LOD 0, 1, or 2) for a given distance from the camera in meters.
pub fn select_lod(distance_meters: f64, config: &StreamingConfig) -> u32 {
    if distance_meters <= config.lod1_distance_meters {
        0 // Highest detail (LOD0)
    } else if distance_meters <= config.lod2_distance_meters {
        1 // Medium detail (LOD1)
    } else {
        2 // Regional/simplified (LOD2)
    }
}

/// Determines the target LOD with hysteresis deadband buffer to eliminate ping-pong popping.
pub fn select_lod_with_hysteresis(
    distance_meters: f64,
    current_lod: Option<u32>,
    config: &StreamingConfig,
) -> u32 {
    let hyst = config.lod_hysteresis_meters;

    match current_lod {
        Some(0) => {
            // Currently LOD0: stay LOD0 unless we comfortably exceed LOD1 threshold + hysteresis
            if distance_meters > config.lod1_distance_meters + hyst {
                if distance_meters > config.lod2_distance_meters + hyst {
                    2
                } else {
                    1
                }
            } else {
                0
            }
        }
        Some(1) => {
            // Currently LOD1: drop to LOD0 if close enough; raise to LOD2 if far enough
            if distance_meters < config.lod1_distance_meters - hyst {
                0
            } else if distance_meters > config.lod2_distance_meters + hyst {
                2
            } else {
                1
            }
        }
        Some(2) => {
            // Currently LOD2: drop to LOD1 or LOD0 if moving closer
            if distance_meters < config.lod1_distance_meters - hyst {
                0
            } else if distance_meters < config.lod2_distance_meters - hyst {
                1
            } else {
                2
            }
        }
        _ => select_lod(distance_meters, config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_selection_and_hysteresis() {
        let config = StreamingConfig {
            lod1_distance_meters: 1500.0,
            lod2_distance_meters: 3000.0,
            lod_hysteresis_meters: 100.0,
            ..Default::default()
        };

        // Standard selection
        assert_eq!(select_lod(500.0, &config), 0);
        assert_eq!(select_lod(2000.0, &config), 1);
        assert_eq!(select_lod(4000.0, &config), 2);

        // Hysteresis test: near the 1500m boundary
        // At 1550m (exceeds 1500m, but within +100m hysteresis): LOD0 should stay LOD0!
        assert_eq!(
            select_lod_with_hysteresis(1550.0, Some(0), &config),
            0,
            "Should stay LOD0 inside hysteresis window"
        );

        // At 1620m (exceeds 1500m + 100m = 1600m): LOD0 switches to LOD1
        assert_eq!(
            select_lod_with_hysteresis(1620.0, Some(0), &config),
            1,
            "Should transition to LOD1 beyond hysteresis window"
        );

        // At 1450m: LOD1 should stay LOD1 (since it hasn't dropped below 1500m - 100m = 1400m)
        assert_eq!(
            select_lod_with_hysteresis(1450.0, Some(1), &config),
            1,
            "Should stay LOD1 inside hysteresis window"
        );

        // At 1380m: LOD1 drops back to LOD0
        assert_eq!(
            select_lod_with_hysteresis(1380.0, Some(1), &config),
            0,
            "Should drop to LOD0 below hysteresis window"
        );
    }
}
