//! Standard 3D geometry mesh and vertex representations for Real Earth.

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// 3D Vertex representation compatible with wgpu buffer layout.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Vertex {
    /// Local East-North-Up coordinate in meters relative to chunk origin.
    pub position: [f32; 3],
    /// Surface normal vector.
    pub normal: [f32; 3],
    /// Texture / UV coordinate.
    pub uv: [f32; 2],
    /// Vertex color (RGBA).
    pub color: [f32; 4],
}

impl Vertex {
    pub const fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            position,
            normal,
            uv,
            color,
        }
    }
}

/// A contiguous 3D mesh with vertex and index buffers.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }

    pub fn append(&mut self, other: &ChunkMesh) {
        let index_offset = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        for idx in &other.indices {
            self.indices.push(idx + index_offset);
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}
