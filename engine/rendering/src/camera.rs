//! Floating origin camera system delivering jitter-free planetary rendering.

use glam::{Mat4, Vec3};
use realearth_geospatial::transforms::wgs84_to_ecef;
use realearth_shared::coordinates::{EcefCoord, Wgs84Coord};

/// Camera operating relative to a floating ECEF origin anchor.
#[derive(Debug, Clone)]
pub struct FloatingOriginCamera {
    /// Absolute position of the camera in double precision ECEF meters.
    pub pos_ecef: EcefCoord,
    /// Current floating origin anchor in double precision ECEF meters.
    pub anchor_ecef: EcefCoord,
    /// Yaw orientation in radians.
    pub yaw: f32,
    /// Pitch orientation in radians.
    pub pitch: f32,
    /// Field of view in Y axis in radians.
    pub fov_y: f32,
    /// Aspect ratio (width / height).
    pub aspect: f32,
    /// Near clip plane distance.
    pub z_near: f32,
    /// Far clip plane distance.
    pub z_far: f32,
}

impl FloatingOriginCamera {
    /// Creates a camera anchored at a geographic location.
    pub fn new(coord: Wgs84Coord, aspect: f32) -> Self {
        let ecef = wgs84_to_ecef(coord);
        Self {
            pos_ecef: ecef,
            anchor_ecef: ecef,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.3,
            fov_y: 60.0_f32.to_radians(),
            aspect,
            z_near: 0.5,
            z_far: 10_000.0,
        }
    }

    /// Re-anchors floating origin to camera position if distance exceeds threshold.
    pub fn maybe_rebase_origin(&mut self, threshold_meters: f64) {
        let dx = self.pos_ecef.x - self.anchor_ecef.x;
        let dy = self.pos_ecef.y - self.anchor_ecef.y;
        let dz = self.pos_ecef.z - self.anchor_ecef.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist > threshold_meters {
            self.anchor_ecef = self.pos_ecef;
        }
    }

    /// Relative position of camera in single precision meters in local ENU frame relative to active anchor.
    pub fn relative_pos_f32(&self) -> Vec3 {
        let anchor_wgs84 = realearth_geospatial::transforms::ecef_to_wgs84(self.anchor_ecef);
        let conv = realearth_geospatial::transforms::EnuConverter::new(anchor_wgs84);
        let enu = conv.ecef_to_enu(self.pos_ecef);
        Vec3::new(enu.east as f32, enu.up as f32, enu.north as f32)
    }

    /// Computes the view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        let rel_pos = self.relative_pos_f32();
        let front = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        Mat4::look_to_rh(rel_pos, front, Vec3::Y)
    }

    /// Computes the perspective projection matrix.
    pub fn proj_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.z_near, self.z_far)
    }

    /// Computes composite View-Projection matrix.
    pub fn view_proj_matrix(&self) -> Mat4 {
        self.proj_matrix() * self.view_matrix()
    }

    /// Extracts active viewing frustum for geometric culling.
    pub fn frustum(&self) -> crate::culling::Frustum {
        crate::culling::Frustum::from_view_proj(&self.view_proj_matrix())
    }
}
