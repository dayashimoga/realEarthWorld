//! 3D Vegetation and urban park surface polygon meshing.

use crate::mesh::{ChunkMesh, Vertex};
use earcutr::earcut;
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::{LandCoverClass, OsmLandCoverFeature};

/// Generates a triangulated vegetation / park surface mesh.
pub fn mesh_vegetation(
    land_cover: &OsmLandCoverFeature,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    if land_cover.polygon.len() < 3 {
        return ChunkMesh::default();
    }

    let enu_pts: Vec<Vec3> = land_cover
        .polygon
        .iter()
        .map(|pt| {
            let enu = converter.wgs84_to_enu(*pt);
            let elev = dem.sample_bilinear(pt) + 0.35; // Slight offset above terrain mesh to prevent Z-fighting
            Vec3::new(enu.east as f32, elev, enu.north as f32)
        })
        .collect();

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

    let veg_indices = match earcut(&flat_coords, &[], 2) {
        Ok(idx) => idx,
        Err(_) => return ChunkMesh::default(),
    };

    let veg_color = match land_cover.class {
        LandCoverClass::DeciduousForest
        | LandCoverClass::EvergreenForest
        | LandCoverClass::MixedForest => {
            [0.16, 0.44, 0.20, 1.0] // Deep forest green
        }
        LandCoverClass::GrasslandHerbaceous | LandCoverClass::PastureHay => {
            [0.30, 0.62, 0.25, 1.0] // Open grassland
        }
        _ => [0.24, 0.55, 0.28, 1.0], // Urban park green
    };

    let up_norm = [0.0, 1.0, 0.0];
    let mut vertices = Vec::with_capacity(count);

    for pt in enu_pts.iter().take(count) {
        vertices.push(Vertex::new(
            [pt.x, pt.y, pt.z],
            up_norm,
            [pt.x * 0.05, pt.z * 0.05],
            veg_color,
        ));
    }

    let indices = veg_indices.into_iter().map(|i| i as u32).collect();

    ChunkMesh::new(vertices, indices)
}
