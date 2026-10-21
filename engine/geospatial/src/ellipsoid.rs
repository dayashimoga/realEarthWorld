//! WGS84 Geodetic Reference Ellipsoid standards and calculations.

/// WGS84 Reference Ellipsoid parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wgs84Ellipsoid;

impl Wgs84Ellipsoid {
    /// Semi-major axis (a) in meters.
    pub const A: f64 = 6_378_137.0;

    /// Reciprocal of flattening (1/f).
    pub const RECIPROCAL_F: f64 = 298.257_223_563;

    /// Flattening (f = 1 / 298.257223563).
    pub const F: f64 = 1.0 / Self::RECIPROCAL_F;

    /// Semi-minor axis (b = a * (1 - f)) in meters.
    pub const B: f64 = Self::A * (1.0 - Self::F);

    /// First eccentricity squared (e^2 = 2f - f^2 = (a^2 - b^2) / a^2).
    pub const E_SQ: f64 = Self::F * (2.0 - Self::F);

    /// Second eccentricity squared (e'^2 = (a^2 - b^2) / b^2).
    pub const E_PRIME_SQ: f64 = (Self::A * Self::A - Self::B * Self::B) / (Self::B * Self::B);

    /// Radius of curvature in the prime vertical:
    /// N(phi) = a / sqrt(1 - e^2 * sin^2(phi))
    #[inline]
    pub fn prime_vertical_radius(lat_rad: f64) -> f64 {
        let sin_lat = lat_rad.sin();
        Self::A / (1.0 - Self::E_SQ * sin_lat * sin_lat).sqrt()
    }

    /// Meridional radius of curvature:
    /// M(phi) = a * (1 - e^2) / (1 - e^2 * sin^2(phi))^(3/2)
    #[inline]
    pub fn meridional_radius(lat_rad: f64) -> f64 {
        let sin_lat = lat_rad.sin();
        let denom = 1.0 - Self::E_SQ * sin_lat * sin_lat;
        Self::A * (1.0 - Self::E_SQ) / (denom * denom.sqrt())
    }
}
