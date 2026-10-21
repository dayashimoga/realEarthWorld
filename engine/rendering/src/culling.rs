//! View frustum extraction and hierarchical spatial culling.

use glam::{Mat4, Vec3, Vec4};

/// Normalized 3D plane in Hessian normal form: Ax + By + Cz + D = 0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrustumPlane {
    pub normal: Vec3,
    pub d: f32,
}

impl FrustumPlane {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        let len = (x * x + y * y + z * z).sqrt();
        if len > 1e-6 {
            Self {
                normal: Vec3::new(x / len, y / len, z / len),
                d: w / len,
            }
        } else {
            Self {
                normal: Vec3::Y,
                d: 0.0,
            }
        }
    }

    /// Evaluates signed distance from the plane to a point.
    #[inline]
    pub fn distance_to(&self, pt: Vec3) -> f32 {
        self.normal.dot(pt) + self.d
    }
}

/// 6-plane viewing frustum for fast geometric culling.
#[derive(Debug, Clone, PartialEq)]
pub struct Frustum {
    pub planes: [FrustumPlane; 6],
}

impl Frustum {
    /// Extracts 6 normalized frustum planes from a composite View-Projection matrix.
    pub fn from_view_proj(vp: &Mat4) -> Self {
        // glam Mat4 cols are column vectors: [c0, c1, c2, c3]
        // Rows of the matrix:
        let r0 = Vec4::new(vp.x_axis.x, vp.y_axis.x, vp.z_axis.x, vp.w_axis.x);
        let r1 = Vec4::new(vp.x_axis.y, vp.y_axis.y, vp.z_axis.y, vp.w_axis.y);
        let r2 = Vec4::new(vp.x_axis.z, vp.y_axis.z, vp.z_axis.z, vp.w_axis.z);
        let r3 = Vec4::new(vp.x_axis.w, vp.y_axis.w, vp.z_axis.w, vp.w_axis.w);

        // Standard Gribb-Hartmann frustum plane extraction for wgpu [0, 1] depth
        let left = r3 + r0;
        let right = r3 - r0;
        let bottom = r3 + r1;
        let top = r3 - r1;
        let near = r2; // In wgpu / Vulkan / DirectX depth is [0, 1], near plane is r2
        let far = r3 - r2;

        Self {
            planes: [
                FrustumPlane::new(left.x, left.y, left.z, left.w),
                FrustumPlane::new(right.x, right.y, right.z, right.w),
                FrustumPlane::new(bottom.x, bottom.y, bottom.z, bottom.w),
                FrustumPlane::new(top.x, top.y, top.z, top.w),
                FrustumPlane::new(near.x, near.y, near.z, near.w),
                FrustumPlane::new(far.x, far.y, far.z, far.w),
            ],
        }
    }

    /// Tests if a bounding sphere intersects or is inside the frustum.
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to(center) < -radius {
                return false; // Completely outside this plane
            }
        }
        true
    }

    /// Tests if an axis-aligned bounding box intersects or is inside the frustum.
    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            // Find positive vertex along plane normal
            let p = Vec3::new(
                if plane.normal.x > 0.0 { max.x } else { min.x },
                if plane.normal.y > 0.0 { max.y } else { min.y },
                if plane.normal.z > 0.0 { max.z } else { min.z },
            );
            if plane.distance_to(p) < 0.0 {
                return false; // Box is behind plane
            }
        }
        true
    }
}
