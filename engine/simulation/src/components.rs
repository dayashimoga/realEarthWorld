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

/// Chunk entity identifier in Bevy ECS.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkEntity {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub lod: u32,
}

/// Marker component for terrain mesh entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TerrainMeshComponent;

/// Marker component for road mesh entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RoadMeshComponent;

/// Marker component for building mesh entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BuildingMeshComponent;

/// Marker component for water mesh entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WaterMeshComponent;

/// Marker component for vegetation mesh entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VegetationMeshComponent;

/// Bounding sphere for ECS frustum and visibility queries.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

/// Decoupled presentation transform interpolated between previous and current physics ticks.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InterpolatedTransform {
    pub previous: LocalTransform,
    pub current: LocalTransform,
    pub render: LocalTransform,
}

impl InterpolatedTransform {
    pub fn new(initial: LocalTransform) -> Self {
        Self {
            previous: initial,
            current: initial,
            render: initial,
        }
    }

    /// Interpolates rendering transform between `previous` and `current` using fraction `alpha` in [0, 1].
    pub fn update_render(&mut self, alpha: f32) {
        let alpha = alpha.clamp(0.0, 1.0);
        self.render.translation = self
            .previous
            .translation
            .lerp(self.current.translation, alpha);
        self.render.rotation_yaw =
            Self::lerp_yaw(self.previous.rotation_yaw, self.current.rotation_yaw, alpha);
        self.render.scale = self.previous.scale.lerp(self.current.scale, alpha);
    }

    fn lerp_yaw(a: f32, b: f32, t: f32) -> f32 {
        let mut diff = (b - a) % std::f32::consts::TAU;
        if diff < -std::f32::consts::PI {
            diff += std::f32::consts::TAU;
        } else if diff > std::f32::consts::PI {
            diff -= std::f32::consts::TAU;
        }
        a + diff * t
    }
}
