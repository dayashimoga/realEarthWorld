//! Web Mercator tile index conversions and bounding calculations.

use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use std::f64::consts::PI;

/// Slippy map / Web Mercator tile coordinate at zoom level `z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

impl TileCoord {
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        Self { x, y, z }
    }

    /// Converts WGS84 lat/lon to a Tile coordinate at the specified zoom level.
    pub fn from_wgs84(coord: Wgs84Coord, z: u8) -> Self {
        let n = 2.0_f64.powi(z as i32);
        let x = ((coord.lon + 180.0) / 360.0 * n)
            .floor()
            .clamp(0.0, n - 1.0) as u32;

        let lat_rad = coord.lat.to_radians();
        let y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0 * n)
            .floor()
            .clamp(0.0, n - 1.0) as u32;

        Self { x, y, z }
    }

    /// Computes the geographic bounding box of this tile in WGS84.
    pub fn bounds(&self) -> GeoBounds {
        let n = 2.0_f64.powi(self.z as i32);

        let min_lon = self.x as f64 / n * 360.0 - 180.0;
        let max_lon = (self.x + 1) as f64 / n * 360.0 - 180.0;

        let max_lat_rad = (PI * (1.0 - 2.0 * self.y as f64 / n)).sinh().atan();
        let min_lat_rad = (PI * (1.0 - 2.0 * (self.y + 1) as f64 / n)).sinh().atan();

        GeoBounds::new(
            min_lat_rad.to_degrees(),
            max_lat_rad.to_degrees(),
            min_lon,
            max_lon,
        )
    }
}
