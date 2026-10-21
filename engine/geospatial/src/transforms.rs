//! High-precision Geodetic, ECEF, and ENU coordinate transformations.

use crate::ellipsoid::Wgs84Ellipsoid;
use realearth_shared::coordinates::{EcefCoord, EnuCoord, Wgs84Coord};
use std::f64::consts::PI;

/// Converts WGS84 Geodetic coordinates (degrees, meters) to Earth-Centered Earth-Fixed (ECEF) coordinates in meters.
pub fn wgs84_to_ecef(coord: Wgs84Coord) -> EcefCoord {
    let lat_rad = coord.lat.to_radians();
    let lon_rad = coord.lon.to_radians();
    let alt = coord.alt;

    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();
    let sin_lon = lon_rad.sin();
    let cos_lon = lon_rad.cos();

    let n = Wgs84Ellipsoid::prime_vertical_radius(lat_rad);

    let x = (n + alt) * cos_lat * cos_lon;
    let y = (n + alt) * cos_lat * sin_lon;
    let z = (n * (1.0 - Wgs84Ellipsoid::E_SQ) + alt) * sin_lat;

    EcefCoord::new(x, y, z)
}

/// Converts Earth-Centered Earth-Fixed (ECEF) coordinates in meters to WGS84 Geodetic coordinates (degrees, meters).
/// Uses Bowring's method with iterative refinement to guarantee precision $< 10^{-6}$ meters globally.
pub fn ecef_to_wgs84(ecef: EcefCoord) -> Wgs84Coord {
    let x = ecef.x;
    let y = ecef.y;
    let z = ecef.z;

    let p = (x * x + y * y).sqrt();

    // Check singularity at the poles
    if p < 1e-10 {
        let lat = if z >= 0.0 { 90.0 } else { -90.0 };
        let lon = 0.0;
        let alt = z.abs() - Wgs84Ellipsoid::B;
        return Wgs84Coord::new(lat, lon, alt);
    }

    let lon_rad = y.atan2(x);

    // Initial Bowring closed-form approximation
    let theta = (z * Wgs84Ellipsoid::A).atan2(p * Wgs84Ellipsoid::B);
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();

    let mut lat_rad = (z + Wgs84Ellipsoid::E_PRIME_SQ * Wgs84Ellipsoid::B * sin_theta.powi(3))
        .atan2(p - Wgs84Ellipsoid::E_SQ * Wgs84Ellipsoid::A * cos_theta.powi(3));

    // Iterative refinement to achieve sub-nanometer convergence
    let mut alt;
    for _ in 0..5 {
        let sin_lat = lat_rad.sin();
        let cos_lat = lat_rad.cos();
        let n = Wgs84Ellipsoid::prime_vertical_radius(lat_rad);

        alt = p * cos_lat + z * sin_lat
            - Wgs84Ellipsoid::A * (1.0 - Wgs84Ellipsoid::E_SQ * sin_lat * sin_lat).sqrt();

        let new_lat_rad = z.atan2(p * (1.0 - Wgs84Ellipsoid::E_SQ * (n / (n + alt))));
        if (new_lat_rad - lat_rad).abs() < 1e-15 {
            lat_rad = new_lat_rad;
            break;
        }
        lat_rad = new_lat_rad;
    }

    // Precise altitude calculation
    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();
    let n = Wgs84Ellipsoid::prime_vertical_radius(lat_rad);
    if lat_rad.abs() > (PI / 4.0) {
        alt = z / sin_lat - n * (1.0 - Wgs84Ellipsoid::E_SQ);
    } else {
        alt = p / cos_lat - n;
    }

    Wgs84Coord::new(lat_rad.to_degrees(), lon_rad.to_degrees(), alt)
}

/// Local Tangent Plane (ENU) Coordinate Converter relative to a chosen WGS84 anchor origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnuConverter {
    pub anchor_wgs84: Wgs84Coord,
    pub anchor_ecef: EcefCoord,
    sin_lat: f64,
    cos_lat: f64,
    sin_lon: f64,
    cos_lon: f64,
}

impl EnuConverter {
    pub fn new(anchor_wgs84: Wgs84Coord) -> Self {
        let anchor_ecef = wgs84_to_ecef(anchor_wgs84);
        let lat_rad = anchor_wgs84.lat.to_radians();
        let lon_rad = anchor_wgs84.lon.to_radians();
        Self {
            anchor_wgs84,
            anchor_ecef,
            sin_lat: lat_rad.sin(),
            cos_lat: lat_rad.cos(),
            sin_lon: lon_rad.sin(),
            cos_lon: lon_rad.cos(),
        }
    }

    /// Converts an ECEF position to Local ENU coordinates relative to the anchor.
    pub fn ecef_to_enu(&self, ecef: EcefCoord) -> EnuCoord {
        let dx = ecef.x - self.anchor_ecef.x;
        let dy = ecef.y - self.anchor_ecef.y;
        let dz = ecef.z - self.anchor_ecef.z;

        let east = -self.sin_lon * dx + self.cos_lon * dy;
        let north = -self.sin_lat * self.cos_lon * dx - self.sin_lat * self.sin_lon * dy
            + self.cos_lat * dz;
        let up =
            self.cos_lat * self.cos_lon * dx + self.cos_lat * self.sin_lon * dy + self.sin_lat * dz;

        EnuCoord::new(east, north, up)
    }

    /// Converts Local ENU coordinates back to absolute ECEF coordinates.
    pub fn enu_to_ecef(&self, enu: EnuCoord) -> EcefCoord {
        let e = enu.east;
        let n = enu.north;
        let u = enu.up;

        let dx =
            -self.sin_lon * e - self.sin_lat * self.cos_lon * n + self.cos_lat * self.cos_lon * u;
        let dy =
            self.cos_lon * e - self.sin_lat * self.sin_lon * n + self.cos_lat * self.sin_lon * u;
        let dz = self.cos_lat * n + self.sin_lat * u;

        EcefCoord::new(
            self.anchor_ecef.x + dx,
            self.anchor_ecef.y + dy,
            self.anchor_ecef.z + dz,
        )
    }

    /// Direct conversion from WGS84 to local ENU.
    pub fn wgs84_to_enu(&self, coord: Wgs84Coord) -> EnuCoord {
        let ecef = wgs84_to_ecef(coord);
        self.ecef_to_enu(ecef)
    }

    /// Direct conversion from local ENU to WGS84.
    pub fn enu_to_wgs84(&self, enu: EnuCoord) -> Wgs84Coord {
        let ecef = self.enu_to_ecef(enu);
        ecef_to_wgs84(ecef)
    }
}
