//! Astronomical solar position and directional sunlight vector calculations.

use glam::Vec3;
use realearth_shared::coordinates::Wgs84Coord;
use std::f64::consts::PI;

/// Computed solar parameters at a specific geographic location.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolarPosition {
    /// Solar elevation angle in degrees above horizon (-90° to +90°).
    pub elevation_deg: f64,
    /// Solar azimuth angle in degrees clockwise from true North (0° to 360°).
    pub azimuth_deg: f64,
    /// Direction vector pointing toward the sun in local ENU space.
    pub sun_direction_enu: Vec3,
    /// Sunlight intensity factor (0.0 at night to 1.0 at zenith noon).
    pub sun_intensity: f32,
    /// Is the sun currently above the horizon.
    pub is_day: bool,
}

/// Calculates solar position given observer WGS84 coordinate and simulated solar time of day.
/// `time_hours`: [0.0, 24.0) in local solar time (12.0 = solar noon).
/// `day_of_year`: 1 to 365 (e.g. 80 = vernal equinox).
pub fn compute_solar_position(
    observer: &Wgs84Coord,
    time_hours: f64,
    day_of_year: u32,
) -> SolarPosition {
    let lat_rad = observer.lat.to_radians();

    // Solar declination approximation (Spencer / Cooper formula)
    let b = 2.0 * PI * (day_of_year as f64 - 81.0) / 365.0;
    let declination_rad = (23.44_f64.to_radians()) * b.sin();

    // Hour angle H (15 degrees per hour from solar noon)
    let hour_angle_rad = (time_hours - 12.0) * 15.0_f64.to_radians();

    // Solar elevation angle alpha
    let sin_elevation = lat_rad.sin() * declination_rad.sin()
        + lat_rad.cos() * declination_rad.cos() * hour_angle_rad.cos();
    let elevation_rad = sin_elevation.clamp(-1.0, 1.0).asin();
    let elevation_deg = elevation_rad.to_degrees();

    // Solar azimuth angle gamma (measured from North towards East)
    let cos_elevation = elevation_rad.cos().max(1e-6);
    let cos_azimuth =
        (declination_rad.sin() - lat_rad.sin() * sin_elevation) / (lat_rad.cos() * cos_elevation);
    let cos_azimuth = cos_azimuth.clamp(-1.0, 1.0);

    let mut azimuth_rad = cos_azimuth.acos();
    if hour_angle_rad > 0.0 {
        azimuth_rad = 2.0 * PI - azimuth_rad;
    }
    let azimuth_deg = azimuth_rad.to_degrees();

    // Direction vector pointing to the sun in local ENU (East, North, Up)
    // East = sin(azimuth) * cos(elevation)
    // North = cos(azimuth) * cos(elevation)
    // Up = sin(elevation)
    let east = (azimuth_rad.sin() * cos_elevation) as f32;
    let north = (azimuth_rad.cos() * cos_elevation) as f32;
    let up = sin_elevation as f32;
    let sun_direction_enu = Vec3::new(east, up, north).normalize_or_zero();

    let is_day = elevation_deg > -0.5; // Include civil twilight
    let sun_intensity = if is_day {
        (sin_elevation as f32).max(0.0).powf(0.8).clamp(0.05, 1.0)
    } else {
        0.0
    };

    SolarPosition {
        elevation_deg,
        azimuth_deg,
        sun_direction_enu,
        sun_intensity,
        is_day,
    }
}
