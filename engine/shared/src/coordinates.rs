use glam::{DVec3, Vec3};
use serde::{Deserialize, Serialize};

/// WGS84 Geodetic Coordinate: Latitude (degrees), Longitude (degrees), Altitude/Height above ellipsoid (meters).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Wgs84Coord {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
}

impl Wgs84Coord {
    pub const fn new(lat: f64, lon: f64, alt: f64) -> Self {
        Self { lat, lon, alt }
    }
}

/// Earth-Centered, Earth-Fixed (ECEF) Cartesian coordinate in meters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EcefCoord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl EcefCoord {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn to_dvec3(self) -> DVec3 {
        DVec3::new(self.x, self.y, self.z)
    }

    pub fn from_dvec3(v: DVec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

/// East-North-Up (ENU) Local Tangent Plane coordinate in meters relative to an anchor.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnuCoord {
    pub east: f64,
    pub north: f64,
    pub up: f64,
}

impl EnuCoord {
    pub const fn new(east: f64, north: f64, up: f64) -> Self {
        Self { east, north, up }
    }

    /// Converts ENU to standard 3D simulation coordinate (X = east, Y = up, Z = north),
    /// matching the coordinate layout of the compiler terrain/road meshers.
    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.east as f32, self.up as f32, self.north as f32)
    }

    /// Converts ENU to standard right-handed graphics camera coordinates (X = east, Y = up, Z = -north).
    pub fn to_graphics_vec3(self) -> Vec3 {
        Vec3::new(self.east as f32, self.up as f32, -self.north as f32)
    }
}

/// Floating origin for eliminating single-precision floating point jitter at planetary scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FloatingOrigin {
    pub anchor_ecef: EcefCoord,
}

impl FloatingOrigin {
    pub fn new(anchor_ecef: EcefCoord) -> Self {
        Self { anchor_ecef }
    }

    /// Converts an absolute ECEF coordinate to relative local offset (meters).
    pub fn to_relative_f32(&self, pos: EcefCoord) -> Vec3 {
        let diff = pos.to_dvec3() - self.anchor_ecef.to_dvec3();
        Vec3::new(diff.x as f32, diff.y as f32, diff.z as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinates_and_conversions() {
        let wgs = Wgs84Coord::new(37.7749, -122.4194, 10.0);
        assert_eq!(wgs.lat, 37.7749);
        assert_eq!(wgs.lon, -122.4194);
        assert_eq!(wgs.alt, 10.0);

        let ecef = EcefCoord::new(100.0, 200.0, 300.0);
        let dvec = ecef.to_dvec3();
        assert_eq!(dvec.x, 100.0);
        let roundtrip = EcefCoord::from_dvec3(dvec);
        assert_eq!(roundtrip, ecef);

        let enu = EnuCoord::new(10.0, 20.0, 5.0);
        let sim_vec = enu.to_vec3();
        assert_eq!(sim_vec, Vec3::new(10.0, 5.0, 20.0));
        let gfx_vec = enu.to_graphics_vec3();
        assert_eq!(gfx_vec, Vec3::new(10.0, 5.0, -20.0));

        let origin = FloatingOrigin::new(EcefCoord::new(1000.0, 2000.0, 3000.0));
        let rel = origin.to_relative_f32(EcefCoord::new(1010.0, 2020.0, 3005.0));
        assert_eq!(rel, Vec3::new(10.0, 20.0, 5.0));
    }
}
