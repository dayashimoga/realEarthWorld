use crate::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// 2D Geographic Bounding Box in WGS84 coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl GeoBounds {
    pub fn new(min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> Self {
        assert!(min_lat <= max_lat, "min_lat must be <= max_lat");
        assert!(min_lon <= max_lon, "min_lon must be <= max_lon");
        Self {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
        }
    }

    /// Creates a geographic bounding box allowing antimeridian crossing (where min_lon > max_lon, e.g. 175°E to -175°W).
    pub fn new_antimeridian(min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> Self {
        assert!(min_lat <= max_lat, "min_lat must be <= max_lat");
        Self {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
        }
    }

    /// Bengaluru Central reference bounding box (approx. 4km x 4km around Vidhana Soudha / MG Road / Cubbon Park).
    pub fn bengaluru_central() -> Self {
        Self {
            min_lat: 12.9600,
            max_lat: 12.9850,
            min_lon: 77.5800,
            max_lon: 77.6150,
        }
    }

    /// Checks if this bounding envelope crosses the antimeridian (180° longitude).
    pub fn crosses_antimeridian(&self) -> bool {
        self.min_lon > self.max_lon
    }

    pub fn contains(&self, coord: &Wgs84Coord) -> bool {
        let lat_in = coord.lat >= self.min_lat && coord.lat <= self.max_lat;
        if !lat_in {
            return false;
        }
        if self.crosses_antimeridian() {
            coord.lon >= self.min_lon || coord.lon <= self.max_lon
        } else {
            coord.lon >= self.min_lon && coord.lon <= self.max_lon
        }
    }

    pub fn center(&self) -> Wgs84Coord {
        Wgs84Coord::new(
            (self.min_lat + self.max_lat) * 0.5,
            (self.min_lon + self.max_lon) * 0.5,
            0.0,
        )
    }
}
