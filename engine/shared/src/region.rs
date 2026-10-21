//! Configurable Geographic Region Profiles for Real Earth.
//! Supports any US metropolitan area or global bounding box.

use crate::bounds::GeoBounds;
use crate::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Metadata and spatial bounds configuration for a bounded real-world simulation region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionProfile {
    /// Unique identifier for this region (e.g. "san-francisco-downtown").
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// ISO 3166-1 alpha-2 country code (e.g. "US", "IN").
    pub country: String,
    /// Sub-division or state (e.g. "CA", "KA").
    pub state_or_province: String,
    /// Outer geographic bounding box in WGS84 degrees.
    pub bounds: GeoBounds,
    /// Primary reference anchor origin.
    pub anchor: Wgs84Coord,
    /// Recommended UTM Projection Zone (e.g. 10 for San Francisco, 43 for Bengaluru).
    pub utm_zone: u8,
    /// Recommended vertical datum standard (e.g. "NAVD88", "EGM96").
    pub vertical_datum: String,
    /// Estimated minimum elevation above MSL in meters.
    pub min_elevation_msl: f32,
    /// Estimated maximum elevation above MSL in meters.
    pub max_elevation_msl: f32,
}

impl RegionProfile {
    /// Creates a generic region profile.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        country: impl Into<String>,
        state: impl Into<String>,
        bounds: GeoBounds,
        anchor: Wgs84Coord,
        utm_zone: u8,
        vertical_datum: impl Into<String>,
        min_elev: f32,
        max_elev: f32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            country: country.into(),
            state_or_province: state.into(),
            bounds,
            anchor,
            utm_zone,
            vertical_datum: vertical_datum.into(),
            min_elevation_msl: min_elev,
            max_elevation_msl: max_elev,
        }
    }

    /// Authoritative San Francisco Downtown & Waterfront Pilot Region (US).
    ///
    /// Extends from Market St / Mission District to North Beach & Embarcadero Piers.
    /// Anchor: Transamerica Pyramid (37.7952°N, -122.4028°W, 10.0m MSL).
    pub fn san_francisco_downtown() -> Self {
        Self {
            id: "san-francisco-downtown".to_string(),
            name: "San Francisco Downtown & Waterfront".to_string(),
            country: "US".to_string(),
            state_or_province: "CA".to_string(),
            bounds: GeoBounds::new(37.7800, 37.8050, -122.4150, -122.3880),
            anchor: Wgs84Coord::new(37.7952, -122.4028, 10.0),
            utm_zone: 10,
            vertical_datum: "NAVD88".to_string(),
            min_elevation_msl: 0.0,
            max_elevation_msl: 115.0,
        }
    }

    /// Bengaluru Central Pilot Region (India).
    /// Anchor: Vidhana Soudha (12.9797°N, 77.5907°E, 920.0m MSL).
    pub fn bengaluru_central() -> Self {
        Self {
            id: "bengaluru-central".to_string(),
            name: "Bengaluru Central".to_string(),
            country: "IN".to_string(),
            state_or_province: "KA".to_string(),
            bounds: GeoBounds::new(12.9600, 12.9850, 77.5800, 77.6150),
            anchor: Wgs84Coord::new(12.9797, 77.5907, 920.0),
            utm_zone: 43,
            vertical_datum: "EGM96".to_string(),
            min_elevation_msl: 900.0,
            max_elevation_msl: 935.0,
        }
    }
}
