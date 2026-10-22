//! Terrain heightfield grid mesh generation from DEM raster samples.

use crate::mesh::{ChunkMesh, Vertex};
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;

/// Generates a triangulated regular terrain heightfield grid for a bounding box.
pub fn mesh_terrain(
    bounds: &GeoBounds,
    converter: &EnuConverter,
    dem: &DemGrid,
    grid_res: usize,
) -> ChunkMesh {
    let res = grid_res.max(2);
    let mut vertices = Vec::with_capacity(res * res);
    let mut indices = Vec::with_capacity((res - 1) * (res - 1) * 6);

    let delta_lat = (bounds.max_lat - bounds.min_lat) / (res - 1) as f64;
    let delta_lon = (bounds.max_lon - bounds.min_lon) / (res - 1) as f64;

    // Terrain color (natural Deccan plateau green/earthy tone)
    let terrain_color = [0.35, 0.44, 0.30, 1.0];

    for y in 0..res {
        let lat = bounds.max_lat - (y as f64) * delta_lat;
        let v = y as f32 / (res - 1) as f32;

        for x in 0..res {
            let lon = bounds.min_lon + (x as f64) * delta_lon;
            let u = x as f32 / (res - 1) as f32;

            let coord = Wgs84Coord::new(lat, lon, 0.0);
            let elev = dem.sample_bilinear(&coord);

            let enu = converter.wgs84_to_enu(coord);

            // Compute physically accurate normal vector via partial derivatives with latitude cosine scaling
            let eps_deg = 1e-5;
            let e_r = dem.sample_bilinear(&Wgs84Coord::new(lat, lon + eps_deg, 0.0));
            let e_l = dem.sample_bilinear(&Wgs84Coord::new(lat, lon - eps_deg, 0.0));
            let e_u = dem.sample_bilinear(&Wgs84Coord::new(lat + eps_deg, lon, 0.0));
            let e_d = dem.sample_bilinear(&Wgs84Coord::new(lat - eps_deg, lon, 0.0));

            let cos_lat = (lat.to_radians()).cos().abs().max(0.1);
            let delta_x_m = 2.0 * eps_deg * 111_320.0 * cos_lat;
            let delta_z_m = 2.0 * eps_deg * 111_000.0;

            let slope_x = (e_r - e_l) / (delta_x_m as f32);
            let slope_z = (e_u - e_d) / (delta_z_m as f32);
            let normal = Vec3::new(-slope_x, 1.0, -slope_z).normalize();

            vertices.push(Vertex::new(
                [enu.east as f32, elev, enu.north as f32],
                [normal.x, normal.y, normal.z],
                [u * 4.0, v * 4.0], // repeat texture
                terrain_color,
            ));
        }
    }

    // Build triangle index grid
    for y in 0..(res - 1) {
        for x in 0..(res - 1) {
            let top_left = (y * res + x) as u32;
            let top_right = top_left + 1;
            let bottom_left = ((y + 1) * res + x) as u32;
            let bottom_right = bottom_left + 1;

            // First triangle
            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            // Second triangle
            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    ChunkMesh::new(vertices, indices)
}
