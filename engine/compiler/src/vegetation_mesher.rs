//! 3D Vegetation, urban park ground covers, and procedural 3D trees with foliage canopies.

use crate::mesh::{ChunkMesh, Vertex};
use earcutr::earcut;
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::{LandCoverClass, OsmLandCoverFeature};
use realearth_shared::coordinates::EnuCoord;

/// Generates a triangulated vegetation surface mesh populated with 3D procedural trees:
/// - Lawn / ground turf layer adhering to local DEM elevation
/// - 3D tree trunks with organic bark coloring
/// - Multi-tiered foliage crowns with outward-pointing surface normals
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
            let elev = dem.sample_bilinear(pt) + 0.25; // Offset above terrain to prevent z-fighting
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
            [0.18, 0.46, 0.22, 1.0] // Deep forest green
        }
        LandCoverClass::GrasslandHerbaceous | LandCoverClass::PastureHay => {
            [0.32, 0.58, 0.26, 1.0] // Open meadow
        }
        _ => [0.26, 0.54, 0.28, 1.0], // Urban park green
    };

    let up_norm = [0.0, 1.0, 0.0];
    let mut vertices = Vec::with_capacity(count + 64);

    // 1. Ground turf polygon with metric UVs
    for pt in enu_pts.iter().take(count) {
        vertices.push(Vertex::new(
            [pt.x, pt.y, pt.z],
            up_norm,
            [pt.x * 0.2, pt.z * 0.2],
            veg_color,
        ));
    }

    let mut indices = Vec::with_capacity(veg_indices.len() + 128);
    for tri in veg_indices.as_chunks::<3>().0 {
        // Counter-clockwise winding facing upward (+Y)
        indices.push(tri[0] as u32);
        indices.push(tri[2] as u32);
        indices.push(tri[1] as u32);
    }

    // 2. Populate 3D Trees within the park boundary
    // Sample interior points using centroid and perimeter interpolations
    let center_x = enu_pts.iter().take(count).map(|p| p.x).sum::<f32>() / count as f32;
    let center_z = enu_pts.iter().take(count).map(|p| p.z).sum::<f32>() / count as f32;

    // Distribute a grid of authentic 3D trees
    let trunk_color = [0.38, 0.26, 0.18, 0.40]; // Wood bark (alpha 0.40)
    let foliage_color = [0.18, 0.52, 0.22, 0.50]; // Leaf canopy (alpha 0.50)

    let mut tree_offsets = vec![Vec3::new(center_x, 0.0, center_z)];
    for p in enu_pts.iter().take(count.min(4)) {
        let mid = Vec3::new(center_x * 0.4 + p.x * 0.6, 0.0, center_z * 0.4 + p.z * 0.6);
        tree_offsets.push(mid);
    }

    for tree_pos in tree_offsets {
        let pt_wgs =
            converter.enu_to_wgs84(EnuCoord::new(tree_pos.x as f64, tree_pos.z as f64, 0.0));
        let ground_y = dem.sample_bilinear(&pt_wgs);

        let trunk_h = 2.4f32;
        let trunk_r = 0.22f32;
        let canopy_bot = ground_y + trunk_h;
        let canopy_mid = canopy_bot + 2.5f32;
        let canopy_top = canopy_bot + 4.8f32;
        let canopy_r = 1.8f32;

        let tx = tree_pos.x;
        let tz = tree_pos.z;

        // A. 4-sided Tree Trunk
        let t_corners = [
            Vec3::new(tx - trunk_r, ground_y, tz - trunk_r),
            Vec3::new(tx + trunk_r, ground_y, tz - trunk_r),
            Vec3::new(tx + trunk_r, ground_y, tz + trunk_r),
            Vec3::new(tx - trunk_r, ground_y, tz + trunk_r),
        ];
        let t_normals = [
            [0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [-1.0, 0.0, 0.0],
        ];

        for f in 0..4 {
            let next = (f + 1) % 4;
            let c0 = t_corners[f];
            let c1 = t_corners[next];
            let n = t_normals[f];

            let b_idx = vertices.len() as u32;
            vertices.push(Vertex::new(
                [c0.x, ground_y, c0.z],
                n,
                [0.0, 0.0],
                trunk_color,
            ));
            vertices.push(Vertex::new(
                [c1.x, ground_y, c1.z],
                n,
                [1.0, 0.0],
                trunk_color,
            ));
            vertices.push(Vertex::new(
                [c1.x, canopy_bot, c1.z],
                n,
                [1.0, 1.0],
                trunk_color,
            ));
            vertices.push(Vertex::new(
                [c0.x, canopy_bot, c0.z],
                n,
                [0.0, 1.0],
                trunk_color,
            ));

            indices.push(b_idx);
            indices.push(b_idx + 3);
            indices.push(b_idx + 2);

            indices.push(b_idx);
            indices.push(b_idx + 2);
            indices.push(b_idx + 1);
        }

        // B. Multi-tier Octagonal Foliage Canopy
        let sides = 6;
        let mut canopy_pts_bot = Vec::with_capacity(sides);
        let mut canopy_pts_mid = Vec::with_capacity(sides);

        for s in 0..sides {
            let a = (s as f32) * (std::f32::consts::TAU / sides as f32);
            canopy_pts_bot.push(Vec3::new(
                tx + a.cos() * canopy_r * 0.7,
                canopy_bot,
                tz + a.sin() * canopy_r * 0.7,
            ));
            canopy_pts_mid.push(Vec3::new(
                tx + a.cos() * canopy_r,
                canopy_mid,
                tz + a.sin() * canopy_r,
            ));
        }
        let apex = Vec3::new(tx, canopy_top, tz);

        // Lower canopy skirt
        for s in 0..sides {
            let next = (s + 1) % sides;
            let p0 = canopy_pts_bot[s];
            let p1 = canopy_pts_bot[next];
            let p2 = canopy_pts_mid[next];
            let p3 = canopy_pts_mid[s];

            let norm = (p1 - p0).cross(p3 - p0).normalize();
            let norm_arr = [norm.x, norm.y, norm.z];

            let b_idx = vertices.len() as u32;
            vertices.push(Vertex::new(
                [p0.x, p0.y, p0.z],
                norm_arr,
                [0.0, 0.0],
                foliage_color,
            ));
            vertices.push(Vertex::new(
                [p1.x, p1.y, p1.z],
                norm_arr,
                [1.0, 0.0],
                foliage_color,
            ));
            vertices.push(Vertex::new(
                [p2.x, p2.y, p2.z],
                norm_arr,
                [1.0, 1.0],
                foliage_color,
            ));
            vertices.push(Vertex::new(
                [p3.x, p3.y, p3.z],
                norm_arr,
                [0.0, 1.0],
                foliage_color,
            ));

            indices.push(b_idx);
            indices.push(b_idx + 3);
            indices.push(b_idx + 2);

            indices.push(b_idx);
            indices.push(b_idx + 2);
            indices.push(b_idx + 1);
        }

        // Upper canopy pyramid
        for s in 0..sides {
            let next = (s + 1) % sides;
            let p0 = canopy_pts_mid[s];
            let p1 = canopy_pts_mid[next];

            let norm = (p1 - p0).cross(apex - p0).normalize();
            let norm_arr = [norm.x, norm.y, norm.z];

            let b_idx = vertices.len() as u32;
            vertices.push(Vertex::new(
                [p0.x, p0.y, p0.z],
                norm_arr,
                [0.0, 0.0],
                foliage_color,
            ));
            vertices.push(Vertex::new(
                [p1.x, p1.y, p1.z],
                norm_arr,
                [1.0, 0.0],
                foliage_color,
            ));
            vertices.push(Vertex::new(
                [apex.x, apex.y, apex.z],
                norm_arr,
                [0.5, 1.0],
                foliage_color,
            ));

            indices.push(b_idx);
            indices.push(b_idx + 2);
            indices.push(b_idx + 1);
        }
    }

    ChunkMesh::new(vertices, indices)
}
