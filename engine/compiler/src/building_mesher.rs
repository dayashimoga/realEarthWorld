//! 3D Building polygon extrusion and roof triangulation.

use crate::mesh::{ChunkMesh, Vertex};
use earcutr::earcut;
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::BuildingType;
use realearth_validation::polygon_validator::ValidatedBuilding;

/// Builds a 3D extruded mesh (walls + roof) for a validated building.
pub fn mesh_building(
    building: &ValidatedBuilding,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    if building.outer_ring.len() < 4 {
        return ChunkMesh::default();
    }

    // 1. Calculate centroid to sample base ground elevation (excluding closing duplicate point)
    let ring_slice = if building.outer_ring.len() > 1
        && building.outer_ring.first() == building.outer_ring.last()
    {
        &building.outer_ring[..building.outer_ring.len() - 1]
    } else {
        &building.outer_ring[..]
    };
    let mean_lat = ring_slice.iter().map(|p| p.lat).sum::<f64>() / ring_slice.len() as f64;
    let mean_lon = ring_slice.iter().map(|p| p.lon).sum::<f64>() / ring_slice.len() as f64;
    let centroid = realearth_shared::coordinates::Wgs84Coord::new(mean_lat, mean_lon, 0.0);
    let base_elev = dem.sample_bilinear(&centroid);
    let roof_elev = base_elev + building.height.height_meters as f32;

    // 2. Convert 2D outer ring to local ENU coordinates (X = east, Z = -north in graphics conventions, Y = up)
    let enu_pts: Vec<Vec3> = building
        .outer_ring
        .iter()
        .map(|pt| {
            let enu = converter.wgs84_to_enu(*pt);
            Vec3::new(enu.east as f32, 0.0, enu.north as f32)
        })
        .collect();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Material color based on building type
    let (wall_color, roof_color) = match building.building_type {
        BuildingType::Civic => ([0.85, 0.82, 0.75, 1.0], [0.65, 0.62, 0.58, 1.0]), // Granite / Classical
        BuildingType::Commercial => ([0.60, 0.75, 0.90, 1.0], [0.45, 0.50, 0.55, 1.0]), // Glass / Modern steel
        BuildingType::Stadium => ([0.80, 0.82, 0.85, 1.0], [0.25, 0.60, 0.35, 1.0]), // Field / Concrete
        BuildingType::Residential => ([0.82, 0.68, 0.58, 1.0], [0.55, 0.40, 0.35, 1.0]), // Terracotta / Warm
        BuildingType::Industrial => ([0.70, 0.70, 0.72, 1.0], [0.40, 0.40, 0.42, 1.0]),
        BuildingType::Other => ([0.75, 0.75, 0.75, 1.0], [0.50, 0.50, 0.50, 1.0]),
    };

    // 3. Extrude vertical walls with terrain-adaptive foundation skirts
    for i in 0..(enu_pts.len() - 1) {
        let p0 = enu_pts[i];
        let p1 = enu_pts[i + 1];

        let edge = Vec3::new(p1.x - p0.x, 0.0, p1.z - p0.z);
        if edge.length_squared() < 1e-4 {
            continue;
        }

        // Outward normal in X-Z plane: for CCW footprint in X-Z, right perpendicular is (edge.z, 0, -edge.x)
        let normal = Vec3::new(edge.z, 0.0, -edge.x).normalize();
        let norm_arr = [normal.x, normal.y, normal.z];

        // Sample terrain height at each vertex to prevent downhill floating or gaps on slopes
        let ground0 = dem.sample_bilinear(&building.outer_ring[i]);
        let ground1 = dem.sample_bilinear(&building.outer_ring[i + 1]);
        // Anchor wall bottom slightly below local ground level (-0.15m skirt)
        let wall_bottom0 = ground0.min(base_elev) - 0.15;
        let wall_bottom1 = ground1.min(base_elev) - 0.15;

        let base_idx = vertices.len() as u32;

        let v0 = Vertex::new([p0.x, wall_bottom0, p0.z], norm_arr, [0.0, 0.0], wall_color);
        let v1 = Vertex::new([p1.x, wall_bottom1, p1.z], norm_arr, [1.0, 0.0], wall_color);
        let v2 = Vertex::new([p1.x, roof_elev, p1.z], norm_arr, [1.0, 1.0], wall_color);
        let v3 = Vertex::new([p0.x, roof_elev, p0.z], norm_arr, [0.0, 1.0], wall_color);

        vertices.push(v0);
        vertices.push(v1);
        vertices.push(v2);
        vertices.push(v3);

        // Quad triangles (CCW facing outward from building)
        indices.push(base_idx);     // v0
        indices.push(base_idx + 3); // v3
        indices.push(base_idx + 2); // v2

        indices.push(base_idx);     // v0
        indices.push(base_idx + 2); // v2
        indices.push(base_idx + 1); // v1
    }

    // 4. Triangulate flat roof using earcutr
    // Prepare 2D coordinates for earcut: [x0, y0, x1, y1, ...]
    let mut flat_coords: Vec<f64> = Vec::with_capacity((enu_pts.len() - 1) * 2);
    for pt in enu_pts.iter().take(enu_pts.len() - 1) {
        flat_coords.push(pt.x as f64);
        flat_coords.push(pt.z as f64);
    }

    if let Ok(roof_indices) = earcut(&flat_coords, &[], 2) {
        let roof_base_idx = vertices.len() as u32;
        let up_norm = [0.0, 1.0, 0.0];

        for pt in enu_pts.iter().take(enu_pts.len() - 1) {
            vertices.push(Vertex::new(
                [pt.x, roof_elev, pt.z],
                up_norm,
                [pt.x * 0.05, pt.z * 0.05],
                roof_color,
            ));
        }

        // Invert winding of earcut triangles from CW to CCW facing upwards (+Y)
        for chunk in roof_indices.chunks(3) {
            if chunk.len() == 3 {
                indices.push(roof_base_idx + chunk[0] as u32);
                indices.push(roof_base_idx + chunk[2] as u32);
                indices.push(roof_base_idx + chunk[1] as u32);
            }
        }
    }

    ChunkMesh::new(vertices, indices)
}
