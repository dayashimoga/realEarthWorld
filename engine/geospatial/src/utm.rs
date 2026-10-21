//! Universal Transverse Mercator (UTM) forward and reverse projections.

use crate::ellipsoid::Wgs84Ellipsoid;
use realearth_shared::coordinates::Wgs84Coord;

/// Represents a UTM projection coordinate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UtmCoord {
    pub easting: f64,
    pub northing: f64,
    pub zone: u8,
    pub north: bool,
}

impl UtmCoord {
    pub fn new(easting: f64, northing: f64, zone: u8, north: bool) -> Self {
        Self {
            easting,
            northing,
            zone,
            north,
        }
    }
}

/// Computes the UTM zone for a given longitude in degrees.
pub fn utm_zone(lon_deg: f64) -> u8 {
    let normalized_lon = ((lon_deg + 180.0) % 360.0 + 360.0) % 360.0 - 180.0;
    (((normalized_lon + 180.0) / 6.0).floor() as u8) + 1
}

/// Central meridian for a given UTM zone in degrees.
pub fn utm_central_meridian(zone: u8) -> f64 {
    (zone as f64 - 1.0) * 6.0 - 180.0 + 3.0
}

const SCALE_FACTOR: f64 = 0.9996;
const FALSE_EASTING: f64 = 500_000.0;
const FALSE_NORTHING_SOUTH: f64 = 10_000_000.0;

/// Projects a WGS84 coordinate into UTM coordinates.
pub fn wgs84_to_utm(coord: Wgs84Coord) -> UtmCoord {
    let lat_rad = coord.lat.to_radians();
    let lon_rad = coord.lon.to_radians();
    let zone = utm_zone(coord.lon);
    let lon0_rad = utm_central_meridian(zone).to_radians();
    let is_north = coord.lat >= 0.0;

    let a = Wgs84Ellipsoid::A;
    let e_sq = Wgs84Ellipsoid::E_SQ;
    let e_prime_sq = Wgs84Ellipsoid::E_PRIME_SQ;

    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();
    let tan_lat = lat_rad.tan();

    let n = a / (1.0 - e_sq * sin_lat * sin_lat).sqrt();
    let t = tan_lat * tan_lat;
    let c = e_prime_sq * cos_lat * cos_lat;
    let delta_lon = lon_rad - lon0_rad;

    // Meridional arc distance M
    let m = a
        * ((1.0 - e_sq / 4.0 - 3.0 * e_sq * e_sq / 64.0 - 5.0 * e_sq.powi(3) / 256.0) * lat_rad
            - (3.0 * e_sq / 8.0 + 3.0 * e_sq * e_sq / 32.0 + 45.0 * e_sq.powi(3) / 1024.0)
                * (2.0 * lat_rad).sin()
            + (15.0 * e_sq * e_sq / 256.0 + 45.0 * e_sq.powi(3) / 1024.0) * (4.0 * lat_rad).sin()
            - (35.0 * e_sq.powi(3) / 3072.0) * (6.0 * lat_rad).sin());

    let delta_lon_cos = delta_lon * cos_lat;

    // Easting
    let easting = FALSE_EASTING
        + SCALE_FACTOR
            * n
            * (delta_lon_cos
                + (1.0 - t + c) * delta_lon_cos.powi(3) / 6.0
                + (5.0 - 18.0 * t + t * t + 72.0 * c - 58.0 * e_prime_sq) * delta_lon_cos.powi(5)
                    / 120.0);

    // Northing
    let mut northing = SCALE_FACTOR
        * (m + n
            * tan_lat
            * (delta_lon_cos.powi(2) / 2.0
                + (5.0 - t + 9.0 * c + 4.0 * c * c) * delta_lon_cos.powi(4) / 24.0
                + (61.0 - 58.0 * t + t * t + 600.0 * c - 330.0 * e_prime_sq)
                    * delta_lon_cos.powi(6)
                    / 720.0));

    if !is_north {
        northing += FALSE_NORTHING_SOUTH;
    }

    UtmCoord::new(easting, northing, zone, is_north)
}
