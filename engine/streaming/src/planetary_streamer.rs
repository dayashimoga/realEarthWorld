//! Adaptive Global-to-Local LOD Streaming, Simulation Relevance Budgets,
//! and Planetary Floating Origin Dynamic Re-anchoring.

use realearth_geospatial::should_reanchor;
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Hierarchical Level of Detail (LOD) scale from global planetary orbit to local street level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlanetaryLodLevel {
    /// LOD 0: Planetary Global Ellipsoid (Altitude > 100 km).
    GlobalOrbit = 0,
    /// LOD 1: Continental / Sub-continental scale (20 km - 100 km).
    Continental = 1,
    /// LOD 2: Regional Intercity Corridors (5 km - 20 km).
    Regional = 2,
    /// LOD 3: Metropolitan / City Scale (1 km - 5 km).
    City = 3,
    /// LOD 4: Street & High-detail building level (< 1 km).
    LocalStreet = 4,
}

impl PlanetaryLodLevel {
    /// Evaluates appropriate hierarchical LOD level based on camera altitude above MSL.
    pub fn from_altitude_msl(alt_meters: f64) -> Self {
        if alt_meters >= 100_000.0 {
            Self::GlobalOrbit
        } else if alt_meters >= 20_000.0 {
            Self::Continental
        } else if alt_meters >= 5_000.0 {
            Self::Regional
        } else if alt_meters >= 1_000.0 {
            Self::City
        } else {
            Self::LocalStreet
        }
    }

    /// Corresponding planetary quadtree level for this LOD level.
    pub fn quadtree_level(&self) -> u8 {
        match self {
            Self::GlobalOrbit => 4,  // ~2500 km cell size
            Self::Continental => 7,  // ~300 km cell size
            Self::Regional => 10,    // ~40 km cell size
            Self::City => 13,        // ~5 km cell size
            Self::LocalStreet => 16, // ~600 m cell size
        }
    }
}

/// Simulation relevance budget adapting lookahead and radius based on velocity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelevanceBudget {
    pub max_lookahead_meters: f64,
    pub active_radius_meters: f64,
    pub target_lod: PlanetaryLodLevel,
    pub refresh_rate_hz: f32,
}

impl RelevanceBudget {
    /// Computes dynamic relevance budget adapted to transport velocity and altitude.
    pub fn compute(velocity_mps: f64, altitude_msl: f64) -> Self {
        let target_lod = PlanetaryLodLevel::from_altitude_msl(altitude_msl);

        if velocity_mps >= 150.0 || altitude_msl >= 10_000.0 {
            // High-speed aviation regime (Commercial Jet / High Altitude)
            Self {
                max_lookahead_meters: 25_000.0,
                active_radius_meters: 35_000.0,
                target_lod,
                refresh_rate_hz: 30.0,
            }
        } else if velocity_mps >= 25.0 {
            // Highway ground transport / Train regime (90 - 250 km/h)
            Self {
                max_lookahead_meters: 3_000.0,
                active_radius_meters: 4_500.0,
                target_lod,
                refresh_rate_hz: 60.0,
            }
        } else {
            // Urban ground vehicle / pedestrian regime
            Self {
                max_lookahead_meters: 1_000.0,
                active_radius_meters: 2_000.0,
                target_lod,
                refresh_rate_hz: 60.0,
            }
        }
    }
}

/// Planetary Floating Origin coordinator maintaining continuous sub-millimeter
/// coordinate precision during long-distance continental traversal.
#[derive(Debug, Clone)]
pub struct PlanetaryOriginCoordinator {
    pub current_anchor: Wgs84Coord,
    pub reanchor_threshold_meters: f64,
    pub total_reanchor_count: usize,
}

impl PlanetaryOriginCoordinator {
    /// Creates a coordinator initialized at a specific WGS84 origin.
    pub fn new(initial_anchor: Wgs84Coord, reanchor_threshold_meters: f64) -> Self {
        Self {
            current_anchor: initial_anchor,
            reanchor_threshold_meters,
            total_reanchor_count: 0,
        }
    }

    /// Evaluates camera position and shifts anchor origin if distance exceeds threshold.
    /// Returns true if a re-anchoring shift occurred.
    pub fn update_camera_position(&mut self, camera: Wgs84Coord) -> bool {
        if should_reanchor(camera, self.current_anchor, self.reanchor_threshold_meters) {
            self.current_anchor = camera;
            self.total_reanchor_count += 1;
            true
        } else {
            false
        }
    }

    /// Computes local metric vector from current anchor to a target point.
    pub fn local_metric_offset(&self, target: Wgs84Coord) -> glam::DVec3 {
        let lat_rad = self.current_anchor.lat.to_radians();
        let meters_per_deg_lat = 111_132.954 - 559.822 * (2.0 * lat_rad).cos();
        let meters_per_deg_lon = 111_412.84 * lat_rad.cos();

        let dx = (target.lon - self.current_anchor.lon) * meters_per_deg_lon;
        let dy = (target.lat - self.current_anchor.lat) * meters_per_deg_lat;
        let dz = target.alt - self.current_anchor.alt;

        glam::DVec3::new(dx, dy, dz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planetary_lod_altitude_mapping() {
        assert_eq!(
            PlanetaryLodLevel::from_altitude_msl(250_000.0),
            PlanetaryLodLevel::GlobalOrbit
        );
        assert_eq!(
            PlanetaryLodLevel::from_altitude_msl(35_000.0),
            PlanetaryLodLevel::Continental
        );
        assert_eq!(
            PlanetaryLodLevel::from_altitude_msl(10_000.0),
            PlanetaryLodLevel::Regional
        );
        assert_eq!(
            PlanetaryLodLevel::from_altitude_msl(2_500.0),
            PlanetaryLodLevel::City
        );
        assert_eq!(
            PlanetaryLodLevel::from_altitude_msl(250.0),
            PlanetaryLodLevel::LocalStreet
        );
    }

    #[test]
    fn test_simulation_relevance_budgets() {
        // High-speed airliner at 250 m/s (~900 km/h) at 11,000m altitude
        let flight_budget = RelevanceBudget::compute(250.0, 11_000.0);
        assert_eq!(flight_budget.max_lookahead_meters, 25_000.0);
        assert_eq!(flight_budget.target_lod, PlanetaryLodLevel::Regional);

        // Ground sedan driving at 30 m/s (~108 km/h) on highway
        let road_budget = RelevanceBudget::compute(30.0, 100.0);
        assert_eq!(road_budget.max_lookahead_meters, 3_000.0);
        assert_eq!(road_budget.target_lod, PlanetaryLodLevel::LocalStreet);
    }

    #[test]
    fn test_planetary_origin_reanchor_lifecycle() {
        let blr = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let mut origin = PlanetaryOriginCoordinator::new(blr, 50_000.0);

        // Move 10 km within Bengaluru: no re-anchor
        let near = Wgs84Coord::new(13.05, 77.59, 920.0);
        assert!(!origin.update_camera_position(near));
        assert_eq!(origin.total_reanchor_count, 0);

        // Travel 140 km to Mysuru: triggers re-anchor
        let mysuru = Wgs84Coord::new(12.3051, 76.6551, 760.0);
        assert!(origin.update_camera_position(mysuru));
        assert_eq!(origin.total_reanchor_count, 1);
        assert_eq!(origin.current_anchor, mysuru);
    }
}
