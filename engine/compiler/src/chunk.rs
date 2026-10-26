//! World Chunk representation and metadata.

use crate::mesh::ChunkMesh;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_validation::provenance::HeightProvenance;
use serde::{Deserialize, Serialize};

/// Magic bytes header for Real Earth Chunk files.
pub const CHUNK_MAGIC: &[u8; 4] = b"RECH";
/// Current binary schema version (v3 for Stage 1.7-R World Geometry, Collision & Transport Infrastructure Overhaul).
pub const CHUNK_VERSION: u32 = 3;

/// Chunk file header with spatial bounds and index properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub lod_level: u32,
    pub bounds: GeoBounds,
    pub anchor: Wgs84Coord,
    pub min_elevation: f32,
    pub max_elevation: f32,
}

/// Metadata for an individual building residing within a chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkBuildingMeta {
    pub feature_id: u64,
    pub name: Option<String>,
    pub height_meters: f32,
    pub provenance: HeightProvenance,
    pub confidence: f32,
}

/// Metadata for an individual road feature residing within a chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkRoadMeta {
    pub way_id: u64,
    pub name: Option<String>,
    pub highway_type: String,
    pub lanes: u32,
    pub is_bridge: bool,
    pub is_tunnel: bool,
    pub layer: i8,
}

/// A complete, self-contained compiled 3D world chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldChunk {
    pub header: ChunkHeader,
    pub terrain: ChunkMesh,
    pub roads: ChunkMesh,
    pub buildings: ChunkMesh,
    pub water: ChunkMesh,
    pub vegetation: ChunkMesh,
    pub building_metadata: Vec<ChunkBuildingMeta>,
    #[serde(default)]
    pub road_metadata: Vec<ChunkRoadMeta>,
}

impl WorldChunk {
    pub fn total_vertices(&self) -> usize {
        self.terrain.vertex_count()
            + self.roads.vertex_count()
            + self.buildings.vertex_count()
            + self.water.vertex_count()
            + self.vegetation.vertex_count()
    }

    pub fn total_triangles(&self) -> usize {
        self.terrain.triangle_count()
            + self.roads.triangle_count()
            + self.buildings.triangle_count()
            + self.water.triangle_count()
            + self.vegetation.triangle_count()
    }

    /// Returns the approximate count of discrete geographic features represented in this chunk.
    pub fn feature_count(&self) -> usize {
        let b = self
            .building_metadata
            .len()
            .max(if !self.buildings.is_empty() { 1 } else { 0 });
        let r = self
            .road_metadata
            .len()
            .max(if !self.roads.is_empty() { 1 } else { 0 });
        let t = if !self.terrain.is_empty() { 1 } else { 0 };
        let w = if !self.water.is_empty() { 1 } else { 0 };
        let v = if !self.vegetation.is_empty() { 1 } else { 0 };
        b + r + t + w + v
    }

    /// Number of surveyed buildings with authoritative geometry (Lidar/Planning Dept).
    pub fn surveyed_building_count(&self) -> usize {
        self.building_metadata
            .iter()
            .filter(|b| b.provenance == HeightProvenance::Surveyed)
            .count()
    }

    /// Number of buildings with inferred visual appearance (e.g. from floor count or footprint).
    pub fn inferred_building_count(&self) -> usize {
        self.building_metadata
            .iter()
            .filter(|b| b.provenance != HeightProvenance::Surveyed)
            .count()
    }

    /// Number of distinct mapped road features in this chunk.
    pub fn road_count(&self) -> usize {
        self.road_metadata.len()
    }
}
