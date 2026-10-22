//! Planetary Hierarchical Spatial Indexing (Discrete Global Grid System).
//! Divides the global WGS84 ellipsoid ([-180, 180] x [-90, 90]) into hierarchical
//! quadkey cells across zoom levels 0 (Planetary Root) to 18 (Sub-meter chunk).

use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::{RealEarthError, Result};
use serde::{Deserialize, Serialize};

/// Maximum supported hierarchical zoom level (Level 18 corresponds to ~0.15m at equator).
pub const MAX_PLANETARY_LEVEL: u8 = 18;

/// A discrete spatial cell key within the planetary hierarchical quadtree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GlobalCellKey {
    pub level: u8,
    pub x: u32,
    pub y: u32,
}

impl GlobalCellKey {
    /// Creates a new planetary grid cell key with validation.
    pub fn new(level: u8, x: u32, y: u32) -> Result<Self> {
        if level > MAX_PLANETARY_LEVEL {
            return Err(RealEarthError::Validation(format!(
                "Level {} exceeds max planetary level {}",
                level, MAX_PLANETARY_LEVEL
            )));
        }
        let max_dim = 1u32 << level;
        if x >= max_dim || y >= max_dim {
            return Err(RealEarthError::Validation(format!(
                "Cell coordinates ({}, {}) out of bounds for level {} (max {})",
                x,
                y,
                level,
                max_dim - 1
            )));
        }
        Ok(Self { level, x, y })
    }

    /// Root planetary cell representing the entire Earth at Level 0.
    pub fn planetary_root() -> Self {
        Self {
            level: 0,
            x: 0,
            y: 0,
        }
    }

    /// Converts a WGS84 coordinate into a planetary cell key at the specified zoom level.
    pub fn from_wgs84(coord: Wgs84Coord, level: u8) -> Result<Self> {
        if level > MAX_PLANETARY_LEVEL {
            return Err(RealEarthError::Validation(format!(
                "Level {} exceeds max planetary level {}",
                level, MAX_PLANETARY_LEVEL
            )));
        }

        // Clamp latitude to [-90.0, 90.0] and normalize longitude to [-180.0, 180.0]
        let lat = coord.lat.clamp(-90.0, 90.0);
        let mut lon = coord.lon;
        while lon < -180.0 {
            lon += 360.0;
        }
        while lon > 180.0 {
            lon -= 360.0;
        }

        let num_cells = (1u32 << level) as f64;

        // Normalized [0.0, 1.0) space
        let norm_x = ((lon + 180.0) / 360.0).clamp(0.0, 0.999999999);
        let norm_y = ((90.0 - lat) / 180.0).clamp(0.0, 0.999999999);

        let cell_x = (norm_x * num_cells).floor() as u32;
        let cell_y = (norm_y * num_cells).floor() as u32;

        let max_coord = (1u32 << level).saturating_sub(1);
        Ok(Self {
            level,
            x: cell_x.min(max_coord),
            y: cell_y.min(max_coord),
        })
    }

    /// Calculates the WGS84 geographic bounding envelope for this planetary cell.
    pub fn bounds(&self) -> GeoBounds {
        let num_cells = (1u32 << self.level) as f64;
        let step_lon = 360.0 / num_cells;
        let step_lat = 180.0 / num_cells;

        let min_lon = -180.0 + (self.x as f64) * step_lon;
        let max_lon = min_lon + step_lon;

        let max_lat = 90.0 - (self.y as f64) * step_lat;
        let min_lat = max_lat - step_lat;

        GeoBounds::new(min_lat, max_lat, min_lon, max_lon)
    }

    /// Returns the center WGS84 point of this cell.
    pub fn center(&self) -> Wgs84Coord {
        let b = self.bounds();
        Wgs84Coord::new(
            (b.min_lat + b.max_lat) * 0.5,
            (b.min_lon + b.max_lon) * 0.5,
            0.0,
        )
    }

    /// Checks if a WGS84 point falls within this cell.
    pub fn contains(&self, coord: Wgs84Coord) -> bool {
        self.bounds().contains(&coord)
    }

    /// Returns the immediate parent cell at level - 1, or None if this is the root.
    pub fn parent(&self) -> Option<Self> {
        if self.level == 0 {
            None
        } else {
            Some(Self {
                level: self.level - 1,
                x: self.x >> 1,
                y: self.y >> 1,
            })
        }
    }

    /// Returns the four immediate children cells at level + 1, or None if at max level.
    pub fn children(&self) -> Option<[Self; 4]> {
        if self.level >= MAX_PLANETARY_LEVEL {
            None
        } else {
            let next_level = self.level + 1;
            let cx = self.x << 1;
            let cy = self.y << 1;
            Some([
                Self {
                    level: next_level,
                    x: cx,
                    y: cy,
                }, // Top-Left (North-West)
                Self {
                    level: next_level,
                    x: cx + 1,
                    y: cy,
                }, // Top-Right (North-East)
                Self {
                    level: next_level,
                    x: cx,
                    y: cy + 1,
                }, // Bottom-Left (South-West)
                Self {
                    level: next_level,
                    x: cx + 1,
                    y: cy + 1,
                }, // Bottom-Right (South-East)
            ])
        }
    }

    /// Encodes this planetary cell key into a hierarchical base-4 quadkey string.
    pub fn to_quadkey(&self) -> String {
        let mut quadkey = String::with_capacity(self.level as usize);
        for i in (0..self.level).rev() {
            let mask = 1u32 << i;
            let mut digit = 0u8;
            if (self.x & mask) != 0 {
                digit += 1;
            }
            if (self.y & mask) != 0 {
                digit += 2;
            }
            quadkey.push((b'0' + digit) as char);
        }
        quadkey
    }

    /// Decodes a base-4 quadkey string back into a planetary cell key.
    pub fn from_quadkey(quadkey: &str) -> Result<Self> {
        let level = quadkey.len() as u8;
        if level > MAX_PLANETARY_LEVEL {
            return Err(RealEarthError::Validation(format!(
                "Quadkey length {} exceeds max level {}",
                level, MAX_PLANETARY_LEVEL
            )));
        }

        let mut x = 0u32;
        let mut y = 0u32;

        for (i, ch) in quadkey.chars().enumerate() {
            let bit_pos = (level as usize) - 1 - i;
            let mask = 1u32 << bit_pos;
            match ch {
                '0' => {}
                '1' => x |= mask,
                '2' => y |= mask,
                '3' => {
                    x |= mask;
                    y |= mask;
                }
                other => {
                    return Err(RealEarthError::Validation(format!(
                        "Invalid quadkey character '{}' (must be '0'..'3')",
                        other
                    )))
                }
            }
        }

        Ok(Self { level, x, y })
    }
}

/// Planetary Spatial Index orchestrating multi-resolution spatial lookups,
/// frustum culling, and level-of-detail bounding queries across Earth scale.
#[derive(Debug, Clone, Default)]
pub struct PlanetarySpatialIndex;

impl PlanetarySpatialIndex {
    /// Queries all planetary cells at a specified zoom level that intersect a geographic bounding envelope.
    /// Correctly supports antimeridian crossing bounds where min_lon > max_lon.
    pub fn query_cells_in_bounds(bounds: &GeoBounds, level: u8) -> Vec<GlobalCellKey> {
        if level > MAX_PLANETARY_LEVEL {
            return Vec::new();
        }

        let num_cells = (1u32 << level) as f64;
        let step_lon = 360.0 / num_cells;
        let step_lat = 180.0 / num_cells;

        // Latitude range [min_lat, max_lat] -> y range [min_y, max_y]
        let max_lat_clamped = bounds.max_lat.clamp(-90.0, 90.0);
        let min_lat_clamped = bounds.min_lat.clamp(-90.0, 90.0);

        let min_y = (((90.0 - max_lat_clamped) / step_lat).floor() as u32).min((1 << level) - 1);
        let max_y = (((90.0 - min_lat_clamped) / step_lat).floor() as u32).min((1 << level) - 1);

        let mut cells = Vec::new();

        // Check if bounds cross the antimeridian
        if bounds.min_lon > bounds.max_lon {
            // Segment 1: min_lon to 180.0
            let start_x1 =
                (((bounds.min_lon + 180.0) / step_lon).floor() as u32).min((1 << level) - 1);
            let end_x1 = (1u32 << level) - 1;

            // Segment 2: -180.0 to max_lon
            let start_x2 = 0u32;
            let end_x2 =
                (((bounds.max_lon + 180.0) / step_lon).floor() as u32).min((1 << level) - 1);

            for y in min_y..=max_y {
                for x in start_x1..=end_x1 {
                    cells.push(GlobalCellKey { level, x, y });
                }
                for x in start_x2..=end_x2 {
                    cells.push(GlobalCellKey { level, x, y });
                }
            }
        } else {
            let start_x =
                (((bounds.min_lon + 180.0) / step_lon).floor() as u32).min((1 << level) - 1);
            let end_x =
                (((bounds.max_lon + 180.0) / step_lon).floor() as u32).min((1 << level) - 1);

            for y in min_y..=max_y {
                for x in start_x..=end_x {
                    cells.push(GlobalCellKey { level, x, y });
                }
            }
        }

        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planetary_root_and_level_one_quadkeys() {
        let root = GlobalCellKey::planetary_root();
        assert_eq!(root.level, 0);
        assert_eq!(root.to_quadkey(), "");

        let bounds = root.bounds();
        assert_eq!(bounds.min_lat, -90.0);
        assert_eq!(bounds.max_lat, 90.0);
        assert_eq!(bounds.min_lon, -180.0);
        assert_eq!(bounds.max_lon, 180.0);

        let children = root.children().unwrap();
        assert_eq!(children.len(), 4);
        assert_eq!(children[0].to_quadkey(), "0"); // NW
        assert_eq!(children[1].to_quadkey(), "1"); // NE
        assert_eq!(children[2].to_quadkey(), "2"); // SW
        assert_eq!(children[3].to_quadkey(), "3"); // SE

        for child in children {
            assert_eq!(child.parent().unwrap(), root);
        }
    }

    #[test]
    fn test_bengaluru_and_san_francisco_quadkey_roundtrip() {
        // Bengaluru Vidhana Soudha: 12.9797°N, 77.5907°E
        let bengaluru = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let cell_blr = GlobalCellKey::from_wgs84(bengaluru, 12).unwrap();
        assert_eq!(cell_blr.level, 12);
        assert!(cell_blr.contains(bengaluru));

        let qk = cell_blr.to_quadkey();
        assert_eq!(qk.len(), 12);
        let decoded = GlobalCellKey::from_quadkey(&qk).unwrap();
        assert_eq!(cell_blr, decoded);

        // San Francisco Transamerica: 37.7952°N, -122.4028°W
        let sf = Wgs84Coord::new(37.7952, -122.4028, 10.0);
        let cell_sf = GlobalCellKey::from_wgs84(sf, 14).unwrap();
        assert_eq!(cell_sf.level, 14);
        assert!(cell_sf.contains(sf));
        let decoded_sf = GlobalCellKey::from_quadkey(&cell_sf.to_quadkey()).unwrap();
        assert_eq!(cell_sf, decoded_sf);
    }

    #[test]
    fn test_antimeridian_crossing_query() {
        // Envelope crossing antimeridian: 175°E to 175°W (-175°) around Fiji / Tonga
        let antimeridian_bounds = GeoBounds::new_antimeridian(-20.0, -10.0, 175.0, -175.0);
        let cells = PlanetarySpatialIndex::query_cells_in_bounds(&antimeridian_bounds, 4);

        assert!(!cells.is_empty());
        // Must contain cells near x = 0 (West of -180°) and near x = 15 (East of +180°)
        let has_east_end = cells.iter().any(|c| c.x == 15);
        let has_west_start = cells.iter().any(|c| c.x == 0);
        assert!(has_east_end, "Must contain cells east of antimeridian");
        assert!(has_west_start, "Must contain cells west of antimeridian");
    }
}
