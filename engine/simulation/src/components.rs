//! ECS Components representing spatial simulation state.

use bevy_ecs::component::Component;
use glam::Vec3;
use realearth_shared::coordinates::{EcefCoord, Wgs84Coord};
use serde::{Deserialize, Serialize};

/// Global high-precision WGS84 geographic coordinate.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPosition {
    pub coord: Wgs84Coord,
}

impl GeoPosition {
    pub fn new(coord: Wgs84Coord) -> Self {
        Self { coord }
    }
}

/// Earth-Centered Earth-Fixed position in double precision.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EcefPosition {
    pub pos: EcefCoord,
}

/// Local 3D transform relative to active chunk anchor.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalTransform {
    pub translation: Vec3,
    pub rotation_yaw: f32,
    pub scale: Vec3,
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation_yaw: 0.0,
            scale: Vec3::ONE,
        }
    }
}

/// Kinematic 3D velocity in local coordinates (m/s).
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular_yaw: f32,
}

/// Unique entity tracking tag.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityTag {
    pub id: u64,
    pub name: String,
}
