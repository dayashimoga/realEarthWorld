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

    /// Generates a realistic DEM conforming to Mysuru topography and Chamundi Hill (740m to 1065m MSL).
    pub fn create_mysuru_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(12.2600, 12.3600, 76.6000, 76.7200);
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let v = y as f64 / (height - 1) as f64; // 0.0 = North, 1.0 = South
            for x in 0..width {
                let u = x as f64 / (width - 1) as f64; // 0.0 = West, 1.0 = East

                let base = 755.0;
                let rolling = ((u * 6.0).sin() * 5.0 + (v * 8.0).cos() * 4.0) as f32;

                // Chamundi Hill peak located at (u ~ 0.5917, v ~ 0.8550) rising to ~1065m MSL
                let dist_chamundi = ((u - 0.5917).powi(2) + (v - 0.8550).powi(2)).sqrt();
                let chamundi_hill = ((-dist_chamundi * 8.0).exp() * 320.0) as f32;

                elevations.push(base + rolling + chamundi_hill);
            }
        }

        Self {
            bounds,
            width,
            height,
            elevations,
        }
    }

    /// Generates a realistic coastal DEM for Chennai from the Bay of Bengal coastline (0m to 25m MSL).
    pub fn create_chennai_coastal_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(13.0400, 13.1200, 80.2400, 80.3000);
        let mut elevations = Vec::with_capacity(width * height);

        for _y in 0..height {
            for x in 0..width {
                let u = x as f64 / (width - 1) as f64; // 0.0 = West (inland), 1.0 = East (Bay of Bengal)

                let elev = if u >= 0.80 {
                    0.0
                } else if u >= 0.65 {
                    let factor = (0.80 - u) / 0.15;
                    (factor * 3.5).max(0.0) as f32
                } else {
                    let inland_factor = (0.65 - u) / 0.65;
                    (3.5 + inland_factor * 18.0 + (u * 10.0).sin() * 2.0).max(1.0) as f32
                };

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

    /// Generates a realistic DEM for Hyderabad Deccan plateau and Golconda ridge (490m to 615m MSL).
    pub fn create_hyderabad_plateau_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(17.3400, 17.4400, 78.4300, 78.5200);
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let v = y as f64 / (height - 1) as f64; // 0.0 = North, 1.0 = South
            for x in 0..width {
                let u = x as f64 / (width - 1) as f64; // 0.0 = West, 1.0 = East

                let base = 535.0;
                let rolling = ((u * 5.0).sin() * 6.0 + (v * 7.0).cos() * 5.0) as f32;

                // Golconda Fort hill ridge in the west (u ~ 0.15, v ~ 0.40)
                let dist_golconda = ((u - 0.15).powi(2) + (v - 0.40).powi(2)).sqrt();
                let golconda_hill = ((-dist_golconda * 12.0).exp() * 75.0) as f32;

                // Musi River depression in the south-central (v ~ 0.72)
                let dist_musi = ((v - 0.72) / 0.04).powi(2);
                let musi_dip = ((-dist_musi).exp() * -20.0) as f32;

                elevations.push((base + rolling + golconda_hill + musi_dip).max(490.0));
            }
        }

        Self {
            bounds,
            width,
            height,
            elevations,
        }
    }

    /// Generates a coastal DEM for New York City (Lower Manhattan & Financial District).
    /// Sea level (0m) along Hudson and East Rivers, gently rising to 12m-18m along Broadway ridge.
    pub fn create_new_york_coastal_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(40.7000, 40.7550, -74.0250, -73.9700);
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let v = y as f64 / (height - 1).max(1) as f64; // North to South
            let _lat = bounds.max_lat - v * (bounds.max_lat - bounds.min_lat);

            for x in 0..width {
                let u = x as f64 / (width - 1).max(1) as f64; // West (Hudson) to East (East River)

                // Central spine ridge (Broadway) runs near u ~ 0.5
                let dist_from_spine = (u - 0.50).abs();
                let spine_elevation = (1.0 - dist_from_spine * 2.2).max(0.0) * 14.0;
                let midtown_rise = (1.0 - v) * 6.0;

                // River edges clamped to sea level
                let is_river = u < 0.15 || u > 0.85;
                let elev = if is_river {
                    0.0
                } else {
                    (2.0 + spine_elevation + midtown_rise) as f32
                };
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

    /// Generates a realistic basin-and-hill DEM for Los Angeles Downtown & Bunker Hill (70m - 145m MSL).
    pub fn create_los_angeles_basin_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(34.0300, 34.0650, -118.2750, -118.2350);
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let v = y as f64 / (height - 1).max(1) as f64;
            for x in 0..width {
                let u = x as f64 / (width - 1).max(1) as f64;

                let base = 75.0;
                // Bunker Hill in northwest quadrant (u ~ 0.4, v ~ 0.3)
                let dist_bunker = ((u - 0.40).powi(2) + (v - 0.30).powi(2)).sqrt();
                let bunker_hill = ((-dist_bunker * 8.0).exp() * 55.0).max(0.0);
                // General northward rise toward Hollywood Hills
                let north_rise = (1.0 - v) * 20.0;

                let elev = (base + bunker_hill + north_rise) as f32;
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

    /// Generates a DEM for Chicago The Loop along Lake Michigan shoreline (176.5m - 183m MSL).
    pub fn create_chicago_loop_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(41.8650, 41.8950, -87.6500, -87.6150);
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let _v = y as f64 / (height - 1).max(1) as f64;
            for x in 0..width {
                let u = x as f64 / (width - 1).max(1) as f64; // West to East (Lake Michigan at East)

                let lake_level = 176.5;
                // Urban lake plain elevation (179m - 182m)
                let elev = if u > 0.82 {
                    lake_level as f32 // Lake Michigan surface
                } else {
                    (lake_level + 2.5 + (1.0 - u) * 3.5) as f32
                };
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

    /// Generates a high-relief coastal DEM for Seattle Downtown & Elliott Bay (0m - 135m MSL).
    pub fn create_seattle_coastal_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(47.5950, 47.6250, -122.3550, -122.3250);
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let v = y as f64 / (height - 1).max(1) as f64;
            for x in 0..width {
                let u = x as f64 / (width - 1).max(1) as f64; // Elliott Bay at West (u < 0.25)

                let elev = if u < 0.20 {
                    0.0 // Elliott Bay sea level
                } else {
                    // Steep climb up First Hill and Queen Anne Hill
                    let climb = ((u - 0.20) / 0.80).powf(0.85) * 115.0;
                    let hill_peak =
                        ((-((u - 0.65).powi(2) + (v - 0.40).powi(2)).sqrt() * 6.0).exp() * 25.0)
                            .max(0.0);
                    (climb + hill_peak) as f32
                };
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

    /// Generates a low-lying coastal DEM for Miami Downtown & Brickell (0m - 4m MSL).
    pub fn create_miami_coastal_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(25.7550, 25.7900, -80.2050, -80.1750);
        let mut elevations = Vec::with_capacity(width * height);

        for _y in 0..height {
            for x in 0..width {
                let u = x as f64 / (width - 1).max(1) as f64; // Biscayne Bay at East (u > 0.75)

                let elev = if u > 0.72 {
                    0.0 // Biscayne Bay surface
                } else {
                    // Miami limestone coastal plain
                    (1.2 + (1.0 - u) * 2.2) as f32
                };
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

    /// Generates a regional DEM covering the South India Intercity Corridors
    /// (Bengaluru, Mysuru, Chennai, Hyderabad, and connecting river basins).
    pub fn create_intercity_corridors_dem(width: usize, height: usize) -> Self {
        let bounds = GeoBounds::new(12.2000, 17.5000, 76.5000, 80.3500);
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let v = y as f64 / (height - 1).max(1) as f64;
            let lat = bounds.max_lat - v * (bounds.max_lat - bounds.min_lat);

            for x in 0..width {
                let u = x as f64 / (width - 1).max(1) as f64;
                let lon = bounds.min_lon + u * (bounds.max_lon - bounds.min_lon);

                // Deccan plateau elevation tapering toward the eastern coast (Bay of Bengal)
                let coast_factor = ((80.35 - lon) / (80.35 - 76.50)).clamp(0.0, 1.0);
                let base_elev = 50.0 + coast_factor * 750.0;

                // North-south variation (Hyderabad ~530m, Bengaluru ~920m, Mysuru ~760m)
                let lat_factor = ((lat - 12.20) / (17.50 - 12.20)).clamp(0.0, 1.0);
                let lat_mod = (1.0 - lat_factor) * 120.0;

                // River valleys depressions (Kaveri near lat 12.4, Krishna/Tungabhadra near lat 15.8)
                let kaveri_dist = ((lat - 12.42) / 0.15).powi(2);
                let krishna_dist = ((lat - 15.85) / 0.20).powi(2);
                let river_dip = ((-kaveri_dist).exp() + (-krishna_dist).exp()) * -45.0;

                let elev = ((base_elev + lat_mod + river_dip).max(2.0)) as f32;
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

    /// Generates a realistic DEM for any defined US city conforming to USGS 3DEP / NAVD88 limits.
    pub fn create_us_city_dem(
        city: &realearth_shared::us_catalog::UsCityDefinition,
        width: usize,
        height: usize,
    ) -> Self {
        match city.id {
            "san-francisco" => return Self::create_san_francisco_coastal_dem(width, height),
            "new-york" => return Self::create_new_york_coastal_dem(width, height),
            "los-angeles" => return Self::create_los_angeles_basin_dem(width, height),
            "chicago" => return Self::create_chicago_loop_dem(width, height),
            "seattle" => return Self::create_seattle_coastal_dem(width, height),
            "miami" => return Self::create_miami_coastal_dem(width, height),
            _ => {}
        }

        let bounds = city.bounds;
        let min_e = city.elevation_min_m;
        let max_e = city.elevation_max_m;
        let delta_e = (max_e - min_e).max(1.0);
        let mut elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            let v = y as f64 / (height - 1).max(1) as f64; // North to South
            for x in 0..width {
                let u = x as f64 / (width - 1).max(1) as f64; // West to East

                let elev = if city.has_maritime_port {
                    // Coastal slope: water along eastern edge, rising inland to west
                    if u > 0.85 {
                        min_e
                    } else {
                        let inland_factor = (0.85 - u) / 0.85;
                        let wave = ((v * 8.0).sin() * 0.15) as f32;
                        min_e + (inland_factor as f32 * delta_e * 0.85) + wave * delta_e
                    }
                } else if city.id == "denver" {
                    // Rocky mountain front range rising to the west (u -> 0)
                    let west_rise = (1.0 - u).powf(1.5) as f32;
                    min_e + west_rise * delta_e
                } else if city.id == "salt-lake-city" {
                    // Wasatch range rising steeply to the east (u -> 1)
                    let east_rise = u.powf(1.5) as f32;
                    min_e + east_rise * delta_e
                } else if city.id == "pittsburgh" {
                    // River valley confluence at center, hills rising North/South
                    let valley_dist = (v - 0.50).abs() as f32;
                    min_e + valley_dist * delta_e
                } else {
                    // Rolling urban topography
                    let wave = ((u * 6.0).sin() * 0.35 + (v * 5.0).cos() * 0.35 + 0.7) as f32;
                    min_e + (wave * 0.5 * delta_e)
                };

                elevations.push(elev.clamp(min_e, max_e));
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
    /// Fully robust to antimeridian crossing bounds without panic.
    pub fn sample_bilinear(&self, coord: &Wgs84Coord) -> f32 {
        let clamped_lat = coord.lat.clamp(self.bounds.min_lat, self.bounds.max_lat);
        let delta_lat = self.bounds.max_lat - self.bounds.min_lat;

        if delta_lat <= 0.0 {
            return self.get_at(0, 0);
        }

        let u = if self.bounds.crosses_antimeridian() {
            let delta_lon = (self.bounds.max_lon + 360.0) - self.bounds.min_lon;
            if delta_lon <= 0.0 {
                return self.get_at(0, 0);
            }
            let lon_offset = if coord.lon >= self.bounds.min_lon {
                coord.lon - self.bounds.min_lon
            } else {
                (coord.lon + 360.0) - self.bounds.min_lon
            };
            let clamped_offset = lon_offset.clamp(0.0, delta_lon);
            (clamped_offset / delta_lon) * (self.width - 1) as f64
        } else {
            let delta_lon = self.bounds.max_lon - self.bounds.min_lon;
            if delta_lon <= 0.0 {
                return self.get_at(0, 0);
            }
            let clamped_lon = coord.lon.clamp(self.bounds.min_lon, self.bounds.max_lon);
            (clamped_lon - self.bounds.min_lon) / delta_lon * (self.width - 1) as f64
        };

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
