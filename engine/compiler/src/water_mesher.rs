//! 3D Water body surface polygon meshing.

use crate::mesh::{ChunkMesh, Vertex};
use earcutr::earcut;
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::OsmWaterFeature;

/// Generates a triangulated surface water mesh for an OSM water feature.
pub fn mesh_water(water: &OsmWaterFeature, converter: &EnuConverter, _dem: &DemGrid) -> ChunkMesh {
    if water.outer_ring.len() < 3 {
        return ChunkMesh::default();
    }

    let enu_pts: Vec<Vec3> = water
        .outer_ring
        .iter()
        .map(|pt| {
            let enu = converter.wgs84_to_enu(*pt);
            Vec3::new(
                enu.east as f32,
                water.target_elevation_msl,
                enu.north as f32,
            )
        })
        .collect();

    // Prepare 2D coordinates for earcut triangulation in X-Z plane
    let count = if enu_pts.first() == enu_pts.last() && enu_pts.len() > 3 {
        enu_pts.len() - 1
    } else {
        enu_pts.len()
    };

    let mut flat_coords: Vec<f64> = Vec::with_capacity(count * 2);
    for pt in enu_pts.iter().take(count) {
        flat_coords.push(pt.x as f64);
        flat_coords.push(pt.z as f64);
    }

    let water_indices = match earcut(&flat_coords, &[], 2) {
        Ok(idx) => idx,
        Err(_) => return ChunkMesh::default(),
    };

    let mut vertices = Vec::with_capacity(count);
    let up_norm = [0.0, 1.0, 0.0];
    let water_color = [0.12, 0.42, 0.74, 0.88]; // Vibrant aquatic ocean/bay blue

    for pt in enu_pts.iter().take(count) {
        vertices.push(Vertex::new(
            [pt.x, pt.y, pt.z],
            up_norm,
            [pt.x * 0.02, pt.z * 0.02],
            water_color,
        ));
    }

    let mut indices = Vec::with_capacity(water_indices.len());
    for tri in water_indices.chunks_exact(3) {
        indices.push(tri[0] as u32);
        indices.push(tri[2] as u32);
        indices.push(tri[1] as u32);
    }

    ChunkMesh::new(vertices, indices)
}
