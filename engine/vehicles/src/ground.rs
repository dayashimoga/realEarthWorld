//! Ground query interface for vehicle raycast suspensions.

use glam::Vec3;

/// Trait defining ground collision detection for wheel suspension rays.
pub trait GroundQuery {
    /// Casts a ray from `origin` along negative Y (downward) up to `max_dist`.
    /// Returns `Some((hit_distance, surface_normal))` if ground is hit.
    fn query_ground_contact(&self, origin: Vec3, max_dist: f32) -> Option<(f32, Vec3)>;
}

/// Simple flat ground plane at a fixed elevation `y`.
#[derive(Debug, Clone, Copy)]
pub struct FlatGround {
    pub elevation: f32,
}

impl FlatGround {
    pub const fn new(elevation: f32) -> Self {
        Self { elevation }
    }
}

impl Default for FlatGround {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl GroundQuery for FlatGround {
    fn query_ground_contact(&self, origin: Vec3, max_dist: f32) -> Option<(f32, Vec3)> {
        let dist = origin.y - self.elevation;
        if dist >= 0.0 && dist <= max_dist {
            Some((dist, Vec3::Y))
        } else {
            None
        }
    }
}

/// Dynamic ground query supporting sloped analytical terrains and elevation grids.
pub struct SlopedGround {
    pub base_elevation: f32,
    pub slope_x: f32,
    pub slope_z: f32,
}

impl SlopedGround {
    pub fn new(base_elevation: f32, slope_x: f32, slope_z: f32) -> Self {
        Self {
            base_elevation,
            slope_x,
            slope_z,
        }
    }
}

impl GroundQuery for SlopedGround {
    fn query_ground_contact(&self, origin: Vec3, max_dist: f32) -> Option<(f32, Vec3)> {
        let ground_y = self.base_elevation + self.slope_x * origin.x + self.slope_z * origin.z;
        let dist = origin.y - ground_y;
        if dist >= 0.0 && dist <= max_dist {
            // Normal perpendicular to surface: (-slope_x, 1.0, -slope_z) normalized
            let normal = Vec3::new(-self.slope_x, 1.0, -self.slope_z).normalize();
            Some((dist, normal))
        } else {
            None
        }
    }
}

/// Flexible ground query backed by an arbitrary elevation sampling closure or grid.
pub struct FunctionalGroundQuery<F>
where
    F: Fn(f32, f32) -> f32,
{
    pub elevation_fn: F,
}

impl<F> FunctionalGroundQuery<F>
where
    F: Fn(f32, f32) -> f32,
{
    pub fn new(elevation_fn: F) -> Self {
        Self { elevation_fn }
    }
}

impl<F> GroundQuery for FunctionalGroundQuery<F>
where
    F: Fn(f32, f32) -> f32,
{
    fn query_ground_contact(&self, origin: Vec3, max_dist: f32) -> Option<(f32, Vec3)> {
        let ground_y = (self.elevation_fn)(origin.x, origin.z);
        let dist = origin.y - ground_y;

        if dist >= 0.0 && dist <= max_dist {
            let eps = 0.5;
            let h_x_plus = (self.elevation_fn)(origin.x + eps, origin.z);
            let h_x_minus = (self.elevation_fn)(origin.x - eps, origin.z);
            let h_z_plus = (self.elevation_fn)(origin.x, origin.z + eps);
            let h_z_minus = (self.elevation_fn)(origin.x, origin.z - eps);

            let dh_dx = (h_x_plus - h_x_minus) / (2.0 * eps);
            let dh_dz = (h_z_plus - h_z_minus) / (2.0 * eps);

            let normal = Vec3::new(-dh_dx, 1.0, -dh_dz).normalize();
            Some((dist, normal))
        } else {
            None
        }
    }
}
