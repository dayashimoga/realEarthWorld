//! World Chunk representation and metadata.

use crate::mesh::ChunkMesh;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_validation::provenance::HeightProvenance;
use serde::{Deserialize, Serialize};

/// Magic bytes header for Real Earth Chunk files.
pub const CHUNK_MAGIC: &[u8; 4] = b"RECH";
/// Current binary schema version.
pub const CHUNK_VERSION: u32 = 1;

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
        let r = if !self.roads.is_empty() { 1 } else { 0 };
        let t = if !self.terrain.is_empty() { 1 } else { 0 };
        let w = if !self.water.is_empty() { 1 } else { 0 };
        let v = if !self.vegetation.is_empty() { 1 } else { 0 };
        b + r + t + w + v
    }
}
