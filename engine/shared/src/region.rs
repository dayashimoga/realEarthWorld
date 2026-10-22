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

    /// Mysuru Urban & Heritage Region (India).
    /// Anchor: Mysore Palace (12.3051°N, 76.6551°E, 760.0m MSL).
    pub fn mysuru_urban() -> Self {
        Self {
            id: "mysuru-urban".to_string(),
            name: "Mysuru Urban & Heritage".to_string(),
            country: "IN".to_string(),
            state_or_province: "KA".to_string(),
            bounds: GeoBounds::new(12.2600, 12.3600, 76.6000, 76.7200),
            anchor: Wgs84Coord::new(12.3051, 76.6551, 760.0),
            utm_zone: 43,
            vertical_datum: "EGM96".to_string(),
            min_elevation_msl: 740.0,
            max_elevation_msl: 1065.0,
        }
    }

    /// Chennai Central & Coastal Region (India).
    /// Anchor: Chennai Central Railway Station (13.0827°N, 80.2755°E, 6.0m MSL).
    pub fn chennai_central() -> Self {
        Self {
            id: "chennai-central".to_string(),
            name: "Chennai Central & Coastal".to_string(),
            country: "IN".to_string(),
            state_or_province: "TN".to_string(),
            bounds: GeoBounds::new(13.0400, 13.1200, 80.2400, 80.3000),
            anchor: Wgs84Coord::new(13.0827, 80.2755, 6.0),
            utm_zone: 44,
            vertical_datum: "EGM96".to_string(),
            min_elevation_msl: 0.0,
            max_elevation_msl: 35.0,
        }
    }

    /// Hyderabad Central & HITEC City Region (India).
    /// Anchor: Charminar (17.3616°N, 78.4747°E, 536.0m MSL).
    pub fn hyderabad_central() -> Self {
        Self {
            id: "hyderabad-central".to_string(),
            name: "Hyderabad Central & HITEC City".to_string(),
            country: "IN".to_string(),
            state_or_province: "TG".to_string(),
            bounds: GeoBounds::new(17.3400, 17.4400, 78.4300, 78.5200),
            anchor: Wgs84Coord::new(17.3616, 78.4747, 536.0),
            utm_zone: 44,
            vertical_datum: "EGM96".to_string(),
            min_elevation_msl: 490.0,
            max_elevation_msl: 615.0,
        }
    }

    /// South India Intercity Corridors connecting Bengaluru, Mysuru, Chennai, and Hyderabad.
    pub fn intercity_corridors() -> Self {
        Self {
            id: "intercity-corridors".to_string(),
            name: "South India Intercity Corridors (NH 275, NH 48, NH 44)".to_string(),
            country: "IN".to_string(),
            state_or_province: "KA-TN-TG".to_string(),
            bounds: GeoBounds::new(12.2000, 17.5000, 76.5000, 80.3500),
            anchor: Wgs84Coord::new(12.9797, 77.5907, 920.0),
            utm_zone: 43,
            vertical_datum: "EGM96".to_string(),
            min_elevation_msl: 0.0,
            max_elevation_msl: 1100.0,
        }
    }
}
