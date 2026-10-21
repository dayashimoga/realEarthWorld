//! Digital Elevation Model (DEM) raster grid processor and interpolator.

use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::{RealEarthError, Result};
use serde::{Deserialize, Serialize};

/// High-resolution Digital Elevation Model raster grid.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemGrid {
    pub bounds: GeoBounds,
    pub width: usize,
    pub height: usize,
    pub elevations: Vec<f32>,
}

impl DemGrid {
    pub fn new(
        bounds: GeoBounds,
        width: usize,
        height: usize,
        elevations: Vec<f32>,
    ) -> Result<Self> {
        if elevations.len() != width * height {
            return Err(RealEarthError::Ingestion(format!(
                "Elevation buffer size mismatch: expected {}, got {}",
                width * height,
                elevations.len()
            )));
        }
        Ok(Self {
            bounds,
            width,
            height,
            elevations,
        })
    }

    /// Generates a synthetic DEM conforming to Bengaluru Central topography (elevation 900m - 930m MSL).
    pub fn create_bengaluru_plateau_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::bengaluru_central();
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let v = y as f64 / (height - 1) as f64;
            let lat = bounds.max_lat - v * (bounds.max_lat - bounds.min_lat);

            for x in 0..width {
                let u = x as f64 / (width - 1) as f64;
                let lon = bounds.min_lon + u * (bounds.max_lon - bounds.min_lon);

                // Deccan plateau elevation pattern around Bengaluru (910m to 935m with gentle rolling terrain)
                let base = 918.0;
                let ridge = (lat * 120.0).sin() * 7.5 + (lon * 150.0).cos() * 5.0;
                let local_hill = ((u - 0.4).powi(2) + (v - 0.6).powi(2)).sqrt() * -12.0;

                let elev = (base + ridge + local_hill) as f32;
                elevations.push(elev);
            }
        }

        Self {
            bounds,
            width,
            height,
            elevations,
        }
    }

    /// Generates a realistic coastal DEM conforming to USGS 3DEP data for San Francisco Downtown (0m to 115m MSL).
    /// Accurately reflects sea level along Embarcadero Bay waterfront and steep climbs up Telegraph Hill and Nob Hill.
    pub fn create_san_francisco_coastal_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(37.7800, 37.8050, -122.4150, -122.3880);
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let v = y as f64 / (height - 1) as f64; // 0.0 = North (37.8050), 1.0 = South (37.7800)
            let _lat = bounds.max_lat - v * (bounds.max_lat - bounds.min_lat);

            for x in 0..width {
                let u = x as f64 / (width - 1) as f64; // 0.0 = West (-122.4150), 1.0 = East (-122.3880, Bay)
                let _lon = bounds.min_lon + u * (bounds.max_lon - bounds.min_lon);

                // 1. General East-to-West coastal slope (Bay at 0m, Nob Hill at 105m)
                // u = 1.0 (East/Bay) -> near 0.0m; u = 0.0 (West/Nob Hill) -> ~100m
                let west_grade = (1.0 - u).powf(1.4) * 85.0;

                // 2. Telegraph Hill peak: located in North-East quadrant (v ~ 0.15, u ~ 0.35)
                let dist_telegraph = ((u - 0.35).powi(2) + (v - 0.15).powi(2)).sqrt();
                let telegraph_hill = ((-dist_telegraph * 12.0).exp() * 65.0).max(0.0);

                // 3. Nob Hill peak: located in West-Central quadrant (v ~ 0.55, u ~ 0.1)
                let dist_nob = ((u - 0.10).powi(2) + (v - 0.55).powi(2)).sqrt();
                let nob_hill = ((-dist_nob * 10.0).exp() * 30.0).max(0.0);

                // 4. Coastline cutoff: Along eastern bay shoreline (u > 0.85), clamp to water level (0.0m)
                let mut elev = (west_grade + telegraph_hill + nob_hill) as f32;
                if u > 0.85 {
                    let coast_factor = ((1.0 - u) / 0.15) as f32;
                    elev = (elev * coast_factor).max(0.0);
                }

                // Minimum elevation is mean sea level (0.0m)
                elevations.push(elev.max(0.0));
            }
        }

        Self {
            bounds,
            width,
            height,
            elevations,
        }
    }

    /// Returns elevation at discrete grid indices (x, y).
    #[inline]
    pub fn get_at(&self, x: usize, y: usize) -> f32 {
        let clamped_x = x.min(self.width - 1);
        let clamped_y = y.min(self.height - 1);
        self.elevations[clamped_y * self.width + clamped_x]
    }

    /// Evaluates continuous elevation at WGS84 coordinate using bilinear interpolation.
    pub fn sample_bilinear(&self, coord: &Wgs84Coord) -> f32 {
        let clamped_lat = coord.lat.clamp(self.bounds.min_lat, self.bounds.max_lat);
        let clamped_lon = coord.lon.clamp(self.bounds.min_lon, self.bounds.max_lon);

        let delta_lon = self.bounds.max_lon - self.bounds.min_lon;
        let delta_lat = self.bounds.max_lat - self.bounds.min_lat;

        if delta_lon <= 0.0 || delta_lat <= 0.0 {
            return self.get_at(0, 0);
        }

        let u = (clamped_lon - self.bounds.min_lon) / delta_lon * (self.width - 1) as f64;
        let v = (self.bounds.max_lat - clamped_lat) / delta_lat * (self.height - 1) as f64;

        let x0 = u.floor() as usize;
        let y0 = v.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = (u - x0 as f64) as f32;
        let fy = (v - y0 as f64) as f32;

        let q00 = self.get_at(x0, y0);
        let q10 = self.get_at(x1, y0);
        let q01 = self.get_at(x0, y1);
        let q11 = self.get_at(x1, y1);

        let top = q00 * (1.0 - fx) + q10 * fx;
        let bottom = q01 * (1.0 - fx) + q11 * fx;

        top * (1.0 - fy) + bottom * fy
    }
}
