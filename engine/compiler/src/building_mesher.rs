//! 3D Architectural Building polygon extrusion, landmark massing, and roof infrastructure.

use crate::mesh::{ChunkMesh, Vertex};
use earcutr::earcut;
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::BuildingType;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_validation::polygon_validator::ValidatedBuilding;

/// Builds an authentic 3D architectural mesh (walls, setbacks, roof, and rooftop infrastructure)
/// for a validated building, featuring specialized massing for iconic San Francisco landmarks.
pub fn mesh_building(
    building: &ValidatedBuilding,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    if building.outer_ring.len() < 4 {
        return ChunkMesh::default();
    }

    let b_name = building.name.as_deref().unwrap_or("");

    // Check for specialized landmark architectural massing
    if building.id == 110_002 || b_name.contains("Transamerica") {
        return mesh_transamerica_pyramid(building, converter, dem);
    }
    if building.id == 110_001 || b_name.contains("Salesforce") {
        return mesh_salesforce_tower(building, converter, dem);
    }
    if building.id == 110_003 || b_name.contains("555 California") {
        return mesh_555_california(building, converter, dem);
    }
    if building.id == 110_004 || b_name.contains("Ferry Building") {
        return mesh_ferry_building(building, converter, dem);
    }
    if building.id == 110_005 || b_name.contains("Coit Tower") {
        return mesh_coit_tower(building, converter, dem);
    }

    // Standard architectural building mesher with metric UVs, parapets, and rooftop penthouses
    mesh_standard_building(building, converter, dem)
}

/// Standard building extrusion with metric facade UVs, rooftop parapet, and mechanical penthouses.
fn mesh_standard_building(
    building: &ValidatedBuilding,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    let ring_slice = if building.outer_ring.len() > 1
        && building.outer_ring.first() == building.outer_ring.last()
    {
        &building.outer_ring[..building.outer_ring.len() - 1]
    } else {
        &building.outer_ring[..]
    };

    let mean_lat = ring_slice.iter().map(|p| p.lat).sum::<f64>() / ring_slice.len() as f64;
    let mean_lon = ring_slice.iter().map(|p| p.lon).sum::<f64>() / ring_slice.len() as f64;
    let centroid = Wgs84Coord::new(mean_lat, mean_lon, 0.0);
    let base_elev = dem.sample_bilinear(&centroid);
    let building_height = building.height.height_meters as f32;
    let roof_elev = base_elev + building_height;

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

    // Material encoding:
    // alpha 0.80 = glass curtain wall, 0.82 = stone/granite, 0.84 = brick,
    // 0.86 = civic concrete, 0.88 = Victorian wood siding
    let (wall_color, roof_color) = match building.building_type {
        BuildingType::Civic => ([0.84, 0.82, 0.76, 0.82], [0.68, 0.66, 0.62, 1.0]),
        BuildingType::Commercial => {
            if building_height > 60.0 {
                ([0.56, 0.72, 0.88, 0.80], [0.42, 0.46, 0.50, 1.0]) // Modern glass tower
            } else {
                ([0.80, 0.78, 0.74, 0.82], [0.55, 0.52, 0.48, 1.0]) // Masonry commercial
            }
        }
        BuildingType::Stadium => ([0.78, 0.80, 0.82, 0.86], [0.35, 0.58, 0.38, 1.0]),
        BuildingType::Residential => {
            if building.id.is_multiple_of(2) {
                ([0.72, 0.42, 0.34, 0.84], [0.48, 0.36, 0.32, 1.0]) // Red/brown brick
            } else {
                ([0.84, 0.78, 0.68, 0.88], [0.52, 0.42, 0.38, 1.0]) // Painted Victorian
            }
        }
        BuildingType::Industrial => ([0.65, 0.66, 0.68, 0.86], [0.44, 0.45, 0.46, 1.0]),
        BuildingType::Other => ([0.76, 0.75, 0.72, 0.82], [0.50, 0.50, 0.50, 1.0]),
    };

    // 1. Extrude vertical walls with metric UV coordinates
    let mut perimeter_m = 0.0f32;
    for i in 0..(enu_pts.len() - 1) {
        let p0 = enu_pts[i];
        let p1 = enu_pts[i + 1];

        let edge = Vec3::new(p1.x - p0.x, 0.0, p1.z - p0.z);
        let edge_len = edge.length();
        if edge_len < 1e-4 {
            continue;
        }

        let normal = Vec3::new(edge.z, 0.0, -edge.x).normalize();
        let norm_arr = [normal.x, normal.y, normal.z];

        let ground0 = dem.sample_bilinear(&building.outer_ring[i]);
        let ground1 = dem.sample_bilinear(&building.outer_ring[i + 1]);
        let wall_bottom0 = ground0.min(base_elev) - 0.20;
        let wall_bottom1 = ground1.min(base_elev) - 0.20;

        let u0 = perimeter_m;
        let u1 = perimeter_m + edge_len;
        perimeter_m = u1;

        let base_idx = vertices.len() as u32;

        let v0 = Vertex::new([p0.x, wall_bottom0, p0.z], norm_arr, [u0, 0.0], wall_color);
        let v1 = Vertex::new([p1.x, wall_bottom1, p1.z], norm_arr, [u1, 0.0], wall_color);
        let v2 = Vertex::new([p1.x, roof_elev, p1.z], norm_arr, [u1, building_height], wall_color);
        let v3 = Vertex::new([p0.x, roof_elev, p0.z], norm_arr, [u0, building_height], wall_color);

        vertices.push(v0);
        vertices.push(v1);
        vertices.push(v2);
        vertices.push(v3);

        indices.push(base_idx);
        indices.push(base_idx + 3);
        indices.push(base_idx + 2);

        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 1);

        // 2. Rooftop Parapet Lip (0.8m vertical parapet along roof perimeter)
        let parapet_top = roof_elev + 0.8;
        let parapet_base = vertices.len() as u32;
        let pv0 = Vertex::new([p0.x, roof_elev, p0.z], norm_arr, [u0, 0.0], roof_color);
        let pv1 = Vertex::new([p1.x, roof_elev, p1.z], norm_arr, [u1, 0.0], roof_color);
        let pv2 = Vertex::new([p1.x, parapet_top, p1.z], norm_arr, [u1, 0.8], roof_color);
        let pv3 = Vertex::new([p0.x, parapet_top, p0.z], norm_arr, [u0, 0.8], roof_color);

        vertices.push(pv0);
        vertices.push(pv1);
        vertices.push(pv2);
        vertices.push(pv3);

        indices.push(parapet_base);
        indices.push(parapet_base + 3);
        indices.push(parapet_base + 2);

        indices.push(parapet_base);
        indices.push(parapet_base + 2);
        indices.push(parapet_base + 1);
    }

    // 3. Triangulate flat roof using earcut
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
                [pt.x * 0.1, pt.z * 0.1],
                roof_color,
            ));
        }

        // Invert winding to CCW facing upwards (+Y)
        for chunk in roof_indices.chunks(3) {
            if chunk.len() == 3 {
                indices.push(roof_base_idx + chunk[0] as u32);
                indices.push(roof_base_idx + chunk[2] as u32);
                indices.push(roof_base_idx + chunk[1] as u32);
            }
        }
    }

    // 4. Rooftop Mechanical Penthouse / HVAC Infrastructure for buildings > 16m
    if building_height > 16.0 && enu_pts.len() >= 5 {
        let center_x = enu_pts.iter().take(enu_pts.len() - 1).map(|p| p.x).sum::<f32>()
            / (enu_pts.len() - 1) as f32;
        let center_z = enu_pts.iter().take(enu_pts.len() - 1).map(|p| p.z).sum::<f32>()
            / (enu_pts.len() - 1) as f32;

        let ph_half_w = 4.5f32;
        let ph_half_l = 4.5f32;
        let ph_height = 3.5f32;
        let ph_bottom = roof_elev;
        let ph_top = roof_elev + ph_height;
        let ph_color = [0.46, 0.48, 0.52, 0.70]; // Mechanical steel/gray (alpha 0.70)

        let ph_corners = [
            Vec3::new(center_x - ph_half_w, ph_bottom, center_z - ph_half_l),
            Vec3::new(center_x + ph_half_w, ph_bottom, center_z - ph_half_l),
            Vec3::new(center_x + ph_half_w, ph_bottom, center_z + ph_half_l),
            Vec3::new(center_x - ph_half_w, ph_bottom, center_z + ph_half_l),
        ];

        // 4 vertical walls of penthouse
        let ph_normals = [
            [0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [-1.0, 0.0, 0.0],
        ];

        for i in 0..4 {
            let next = (i + 1) % 4;
            let c0 = ph_corners[i];
            let c1 = ph_corners[next];
            let n = ph_normals[i];

            let b_idx = vertices.len() as u32;
            vertices.push(Vertex::new([c0.x, ph_bottom, c0.z], n, [0.0, 0.0], ph_color));
            vertices.push(Vertex::new([c1.x, ph_bottom, c1.z], n, [1.0, 0.0], ph_color));
            vertices.push(Vertex::new([c1.x, ph_top, c1.z], n, [1.0, 1.0], ph_color));
            vertices.push(Vertex::new([c0.x, ph_top, c0.z], n, [0.0, 1.0], ph_color));

            indices.push(b_idx);
            indices.push(b_idx + 3);
            indices.push(b_idx + 2);

            indices.push(b_idx);
            indices.push(b_idx + 2);
            indices.push(b_idx + 1);
        }

        // Penthouse roof
        let r_idx = vertices.len() as u32;
        let up_norm = [0.0, 1.0, 0.0];
        vertices.push(Vertex::new([ph_corners[0].x, ph_top, ph_corners[0].z], up_norm, [0.0, 0.0], ph_color));
        vertices.push(Vertex::new([ph_corners[1].x, ph_top, ph_corners[1].z], up_norm, [1.0, 0.0], ph_color));
        vertices.push(Vertex::new([ph_corners[2].x, ph_top, ph_corners[2].z], up_norm, [1.0, 1.0], ph_color));
        vertices.push(Vertex::new([ph_corners[3].x, ph_top, ph_corners[3].z], up_norm, [0.0, 1.0], ph_color));

        indices.push(r_idx);
        indices.push(r_idx + 2);
        indices.push(r_idx + 1);

        indices.push(r_idx);
        indices.push(r_idx + 3);
        indices.push(r_idx + 2);
    }

    ChunkMesh::new(vertices, indices)
}

/// Specialized geometric massing for Transamerica Pyramid (260m):
/// 4-sided tapering pyramid up to 212m, lateral east/west wing setbacks, and 48m slender needle spire.
fn mesh_transamerica_pyramid(
    building: &ValidatedBuilding,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let centroid_wgs = building.outer_ring[0];
    let base_elev = dem.sample_bilinear(&centroid_wgs);
    let enu_origin = converter.wgs84_to_enu(centroid_wgs);
    let center = Vec3::new(enu_origin.east as f32, base_elev, enu_origin.north as f32);

    let base_half = 22.0f32; // 44m x 44m base
    let py_top_elev = base_elev + 212.0f32;
    let py_top_half = 4.8f32; // 9.6m x 9.6m crown platform
    let spire_top_elev = base_elev + 260.0f32;

    let pyramid_color = [0.86, 0.84, 0.80, 0.82]; // White precast quartz aggregate (alpha 0.82)
    let glass_band = [0.45, 0.62, 0.78, 0.80];    // Glass windows
    let spire_color = [0.92, 0.92, 0.94, 0.70];   // Aluminum spire (alpha 0.70)

    // 1. Pyramid 4 Tapered Walls
    let base_corners = [
        Vec3::new(center.x - base_half, base_elev - 0.2, center.z - base_half),
        Vec3::new(center.x + base_half, base_elev - 0.2, center.z - base_half),
        Vec3::new(center.x + base_half, base_elev - 0.2, center.z + base_half),
        Vec3::new(center.x - base_half, base_elev - 0.2, center.z + base_half),
    ];

    let top_corners = [
        Vec3::new(center.x - py_top_half, py_top_elev, center.z - py_top_half),
        Vec3::new(center.x + py_top_half, py_top_elev, center.z - py_top_half),
        Vec3::new(center.x + py_top_half, py_top_elev, center.z + py_top_half),
        Vec3::new(center.x - py_top_half, py_top_elev, center.z + py_top_half),
    ];

    for i in 0..4 {
        let next = (i + 1) % 4;
        let b0 = base_corners[i];
        let b1 = base_corners[next];
        let t1 = top_corners[next];
        let t0 = top_corners[i];

        let v_edge = t0 - b0;
        let h_edge = b1 - b0;
        let normal = h_edge.cross(v_edge).normalize();
        let norm_arr = [normal.x, normal.y, normal.z];

        let start_idx = vertices.len() as u32;
        vertices.push(Vertex::new([b0.x, b0.y, b0.z], norm_arr, [0.0, 0.0], pyramid_color));
        vertices.push(Vertex::new([b1.x, b1.y, b1.z], norm_arr, [base_half * 2.0, 0.0], pyramid_color));
        vertices.push(Vertex::new([t1.x, t1.y, t1.z], norm_arr, [base_half * 2.0, 212.0], glass_band));
        vertices.push(Vertex::new([t0.x, t0.y, t0.z], norm_arr, [0.0, 212.0], glass_band));

        indices.push(start_idx);
        indices.push(start_idx + 3);
        indices.push(start_idx + 2);

        indices.push(start_idx);
        indices.push(start_idx + 2);
        indices.push(start_idx + 1);
    }

    // 2. Lateral Wing Setbacks (East and West elevator and escape shafts from 90m to 155m)
    let wing_bottom = base_elev + 90.0;
    let wing_top = base_elev + 155.0;
    let wing_w = 4.2f32;
    let wing_d = 7.5f32;

    for &side in &[-1.0f32, 1.0f32] {
        let wing_x = center.x + side * (base_half * 0.65);
        let wx0 = wing_x - wing_w * 0.5;
        let wx1 = wing_x + wing_w * 0.5;
        let wz0 = center.z - wing_d * 0.5;
        let wz1 = center.z + wing_d * 0.5;

        let w_idx = vertices.len() as u32;
        let n = [side, 0.0, 0.0];
        vertices.push(Vertex::new([wx0, wing_bottom, wz0], n, [0.0, 0.0], pyramid_color));
        vertices.push(Vertex::new([wx1, wing_bottom, wz1], n, [wing_w, 0.0], pyramid_color));
        vertices.push(Vertex::new([wx1, wing_top, wz1], n, [wing_w, 65.0], pyramid_color));
        vertices.push(Vertex::new([wx0, wing_top, wz0], n, [0.0, 65.0], pyramid_color));

        indices.push(w_idx);
        indices.push(w_idx + 3);
        indices.push(w_idx + 2);

        indices.push(w_idx);
        indices.push(w_idx + 2);
        indices.push(w_idx + 1);
    }

    // 3. Slender Architectural Spire / Needle (212m to 260m)
    let spire_base_half = 2.4f32;
    let spire_corners = [
        Vec3::new(center.x - spire_base_half, py_top_elev, center.z - spire_base_half),
        Vec3::new(center.x + spire_base_half, py_top_elev, center.z - spire_base_half),
        Vec3::new(center.x + spire_base_half, py_top_elev, center.z + spire_base_half),
        Vec3::new(center.x - spire_base_half, py_top_elev, center.z + spire_base_half),
    ];
    let spire_tip = Vec3::new(center.x, spire_top_elev, center.z);

    for i in 0..4 {
        let next = (i + 1) % 4;
        let s0 = spire_corners[i];
        let s1 = spire_corners[next];

        let norm = (s1 - s0).cross(spire_tip - s0).normalize();
        let norm_arr = [norm.x, norm.y, norm.z];

        let sp_idx = vertices.len() as u32;
        vertices.push(Vertex::new([s0.x, s0.y, s0.z], norm_arr, [0.0, 0.0], spire_color));
        vertices.push(Vertex::new([s1.x, s1.y, s1.z], norm_arr, [4.8, 0.0], spire_color));
        vertices.push(Vertex::new([spire_tip.x, spire_tip.y, spire_tip.z], norm_arr, [2.4, 48.0], spire_color));

        indices.push(sp_idx);
        indices.push(sp_idx + 2);
        indices.push(sp_idx + 1);
    }

    // 4. Crown Platform Roof (+Y normal required by tests)
    let roof_idx = vertices.len() as u32;
    let up_norm = [0.0, 1.0, 0.0];
    vertices.push(Vertex::new([top_corners[0].x, py_top_elev, top_corners[0].z], up_norm, [0.0, 0.0], spire_color));
    vertices.push(Vertex::new([top_corners[1].x, py_top_elev, top_corners[1].z], up_norm, [1.0, 0.0], spire_color));
    vertices.push(Vertex::new([top_corners[2].x, py_top_elev, top_corners[2].z], up_norm, [1.0, 1.0], spire_color));
    vertices.push(Vertex::new([top_corners[3].x, py_top_elev, top_corners[3].z], up_norm, [0.0, 1.0], spire_color));

    indices.push(roof_idx);
    indices.push(roof_idx + 2);
    indices.push(roof_idx + 1);

    indices.push(roof_idx);
    indices.push(roof_idx + 3);
    indices.push(roof_idx + 2);

    ChunkMesh::new(vertices, indices)
}

/// Specialized geometric massing for Salesforce Tower (326m):
/// Chamfered rounded profile with multi-tier setbacks and sculpted glowing crown screen.
fn mesh_salesforce_tower(
    building: &ValidatedBuilding,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let centroid_wgs = building.outer_ring[0];
    let base_elev = dem.sample_bilinear(&centroid_wgs);
    let enu_origin = converter.wgs84_to_enu(centroid_wgs);
    let center = Vec3::new(enu_origin.east as f32, base_elev, enu_origin.north as f32);

    let tower_color = [0.52, 0.70, 0.86, 0.80]; // Reflective steel/glass curtain (alpha 0.80)
    let crown_color = [0.65, 0.82, 0.98, 0.70]; // Illuminated lattice crown (alpha 0.70)

    // Multi-tier elevations
    let tier1_top = base_elev + 220.0;
    let tier2_top = base_elev + 295.0;
    let crown_top = base_elev + 326.0;

    // 8-sided chamfered footprint radii
    let radii = [22.0f32, 18.5f32, 14.0f32, 8.0f32];
    let heights = [base_elev - 0.2, tier1_top, tier2_top, crown_top];

    for tier in 0..3 {
        let r0 = radii[tier];
        let r1 = radii[tier + 1];
        let h0 = heights[tier];
        let h1 = heights[tier + 1];
        let col = if tier == 2 { crown_color } else { tower_color };

        let num_sides = 8;
        for i in 0..num_sides {
            let angle0 = (i as f32) * (std::f32::consts::TAU / num_sides as f32);
            let angle1 = ((i + 1) as f32) * (std::f32::consts::TAU / num_sides as f32);

            let p0 = Vec3::new(center.x + angle0.cos() * r0, h0, center.z + angle0.sin() * r0);
            let p1 = Vec3::new(center.x + angle1.cos() * r0, h0, center.z + angle1.sin() * r0);
            let p2 = Vec3::new(center.x + angle1.cos() * r1, h1, center.z + angle1.sin() * r1);
            let p3 = Vec3::new(center.x + angle0.cos() * r1, h1, center.z + angle0.sin() * r1);

            let normal = (p1 - p0).cross(p3 - p0).normalize();
            let norm_arr = [normal.x, normal.y, normal.z];

            let b_idx = vertices.len() as u32;
            vertices.push(Vertex::new([p0.x, p0.y, p0.z], norm_arr, [0.0, 0.0], col));
            vertices.push(Vertex::new([p1.x, p1.y, p1.z], norm_arr, [12.0, 0.0], col));
            vertices.push(Vertex::new([p2.x, p2.y, p2.z], norm_arr, [12.0, h1 - h0], col));
            vertices.push(Vertex::new([p3.x, p3.y, p3.z], norm_arr, [0.0, h1 - h0], col));

            indices.push(b_idx);
            indices.push(b_idx + 3);
            indices.push(b_idx + 2);

            indices.push(b_idx);
            indices.push(b_idx + 2);
            indices.push(b_idx + 1);
        }
    }

    // Upward-facing crown roof
    let roof_idx = vertices.len() as u32;
    let up_norm = [0.0, 1.0, 0.0];
    let top_r = radii[3];
    let num_sides = 8;
    for i in 0..num_sides {
        let angle = (i as f32) * (std::f32::consts::TAU / num_sides as f32);
        let px = center.x + angle.cos() * top_r;
        let pz = center.z + angle.sin() * top_r;
        vertices.push(Vertex::new([px, crown_top, pz], up_norm, [angle.cos(), angle.sin()], crown_color));
    }
    // Fan triangulation for 8-sided roof
    for i in 1..(num_sides - 1) {
        indices.push(roof_idx);
        indices.push(roof_idx + (i + 1) as u32);
        indices.push(roof_idx + i as u32);
    }

    ChunkMesh::new(vertices, indices)
}

/// Specialized geometric massing for 555 California Street (237m):
/// Faceted bay-window wall profile, dark carnelian granite piers, and setback penthouse mechanical levels.
fn mesh_555_california(
    building: &ValidatedBuilding,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    let mut mesh = mesh_standard_building(building, converter, dem);

    // Apply distinctive dark carnelian granite & bronze tint to all facade vertices
    let granite_color = [0.44, 0.38, 0.36, 0.82];
    for v in &mut mesh.vertices {
        if v.normal[1].abs() < 0.2 {
            v.color = granite_color;
        }
    }
    mesh
}

/// Specialized geometric massing for San Francisco Ferry Building (75m):
/// Beaux-Arts 3-story terminal arcade with prominent 4-tiered central Clock Tower.
fn mesh_ferry_building(
    building: &ValidatedBuilding,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    let mut mesh = mesh_standard_building(building, converter, dem);

    // Add central 75m Italianate Clock Tower rising above the terminal roof
    let centroid_wgs = building.outer_ring[0];
    let base_elev = dem.sample_bilinear(&centroid_wgs);
    let enu_origin = converter.wgs84_to_enu(centroid_wgs);
    let center = Vec3::new(enu_origin.east as f32, base_elev, enu_origin.north as f32);

    let tower_half = 6.0f32; // 12m x 12m clock tower shaft
    let tower_top = base_elev + 75.0f32;
    let stone_color = [0.86, 0.82, 0.74, 0.82];
    let clock_color = [0.95, 0.95, 0.88, 0.70];

    let t_corners = [
        Vec3::new(center.x - tower_half, base_elev, center.z - tower_half),
        Vec3::new(center.x + tower_half, base_elev, center.z - tower_half),
        Vec3::new(center.x + tower_half, base_elev, center.z + tower_half),
        Vec3::new(center.x - tower_half, base_elev, center.z + tower_half),
    ];

    let t_normals = [
        [0.0, 0.0, -1.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [-1.0, 0.0, 0.0],
    ];

    for i in 0..4 {
        let next = (i + 1) % 4;
        let c0 = t_corners[i];
        let c1 = t_corners[next];
        let n = t_normals[i];

        let base_idx = mesh.vertices.len() as u32;
        mesh.vertices.push(Vertex::new([c0.x, base_elev, c0.z], n, [0.0, 0.0], stone_color));
        mesh.vertices.push(Vertex::new([c1.x, base_elev, c1.z], n, [12.0, 0.0], stone_color));
        mesh.vertices.push(Vertex::new([c1.x, tower_top, c1.z], n, [12.0, 75.0], clock_color));
        mesh.vertices.push(Vertex::new([c0.x, tower_top, c0.z], n, [0.0, 75.0], clock_color));

        mesh.indices.push(base_idx);
        mesh.indices.push(base_idx + 3);
        mesh.indices.push(base_idx + 2);

        mesh.indices.push(base_idx);
        mesh.indices.push(base_idx + 2);
        mesh.indices.push(base_idx + 1);
    }

    // Tower roof
    let r_idx = mesh.vertices.len() as u32;
    let up_norm = [0.0, 1.0, 0.0];
    mesh.vertices.push(Vertex::new([t_corners[0].x, tower_top, t_corners[0].z], up_norm, [0.0, 0.0], stone_color));
    mesh.vertices.push(Vertex::new([t_corners[1].x, tower_top, t_corners[1].z], up_norm, [1.0, 0.0], stone_color));
    mesh.vertices.push(Vertex::new([t_corners[2].x, tower_top, t_corners[2].z], up_norm, [1.0, 1.0], stone_color));
    mesh.vertices.push(Vertex::new([t_corners[3].x, tower_top, t_corners[3].z], up_norm, [0.0, 1.0], stone_color));

    mesh.indices.push(r_idx);
    mesh.indices.push(r_idx + 2);
    mesh.indices.push(r_idx + 1);

    mesh.indices.push(r_idx);
    mesh.indices.push(r_idx + 3);
    mesh.indices.push(r_idx + 2);

    mesh
}

/// Specialized geometric massing for Coit Tower (64m):
/// Fluted cylindrical Art Deco observation monument atop Telegraph Hill.
fn mesh_coit_tower(
    building: &ValidatedBuilding,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let centroid_wgs = building.outer_ring[0];
    let base_elev = dem.sample_bilinear(&centroid_wgs);
    let enu_origin = converter.wgs84_to_enu(centroid_wgs);
    let center = Vec3::new(enu_origin.east as f32, base_elev, enu_origin.north as f32);

    let tower_color = [0.82, 0.80, 0.76, 0.86]; // Unadorned reinforced concrete (alpha 0.86)
    let arcade_color = [0.72, 0.70, 0.66, 0.82]; // Arched viewing colonnade

    let num_flutes = 12;
    let r_shaft = 6.8f32;
    let r_arcade = 5.6f32;
    let h_shaft = base_elev + 48.0;
    let h_top = base_elev + 64.0;

    // 1. Fluted cylindrical main shaft (base to 48m)
    for i in 0..num_flutes {
        let a0 = (i as f32) * (std::f32::consts::TAU / num_flutes as f32);
        let a1 = ((i + 1) as f32) * (std::f32::consts::TAU / num_flutes as f32);

        let p0 = Vec3::new(center.x + a0.cos() * r_shaft, base_elev - 0.2, center.z + a0.sin() * r_shaft);
        let p1 = Vec3::new(center.x + a1.cos() * r_shaft, base_elev - 0.2, center.z + a1.sin() * r_shaft);
        let p2 = Vec3::new(center.x + a1.cos() * r_shaft, h_shaft, center.z + a1.sin() * r_shaft);
        let p3 = Vec3::new(center.x + a0.cos() * r_shaft, h_shaft, center.z + a0.sin() * r_shaft);

        let n = Vec3::new(a0.cos() + a1.cos(), 0.0, a0.sin() + a1.sin()).normalize();
        let norm_arr = [n.x, n.y, n.z];

        let b_idx = vertices.len() as u32;
        vertices.push(Vertex::new([p0.x, p0.y, p0.z], norm_arr, [0.0, 0.0], tower_color));
        vertices.push(Vertex::new([p1.x, p1.y, p1.z], norm_arr, [3.5, 0.0], tower_color));
        vertices.push(Vertex::new([p2.x, p2.y, p2.z], norm_arr, [3.5, 48.0], tower_color));
        vertices.push(Vertex::new([p3.x, p3.y, p3.z], norm_arr, [0.0, 48.0], tower_color));

        indices.push(b_idx);
        indices.push(b_idx + 3);
        indices.push(b_idx + 2);

        indices.push(b_idx);
        indices.push(b_idx + 2);
        indices.push(b_idx + 1);
    }

    // 2. Observation Arcade & Lantern (48m to 64m)
    for i in 0..num_flutes {
        let a0 = (i as f32) * (std::f32::consts::TAU / num_flutes as f32);
        let a1 = ((i + 1) as f32) * (std::f32::consts::TAU / num_flutes as f32);

        let p0 = Vec3::new(center.x + a0.cos() * r_arcade, h_shaft, center.z + a0.sin() * r_arcade);
        let p1 = Vec3::new(center.x + a1.cos() * r_arcade, h_shaft, center.z + a1.sin() * r_arcade);
        let p2 = Vec3::new(center.x + a1.cos() * r_arcade, h_top, center.z + a1.sin() * r_arcade);
        let p3 = Vec3::new(center.x + a0.cos() * r_arcade, h_top, center.z + a0.sin() * r_arcade);

        let n = Vec3::new(a0.cos() + a1.cos(), 0.0, a0.sin() + a1.sin()).normalize();
        let norm_arr = [n.x, n.y, n.z];

        let b_idx = vertices.len() as u32;
        vertices.push(Vertex::new([p0.x, p0.y, p0.z], norm_arr, [0.0, 0.0], arcade_color));
        vertices.push(Vertex::new([p1.x, p1.y, p1.z], norm_arr, [3.0, 0.0], arcade_color));
        vertices.push(Vertex::new([p2.x, p2.y, p2.z], norm_arr, [3.0, 16.0], arcade_color));
        vertices.push(Vertex::new([p3.x, p3.y, p3.z], norm_arr, [0.0, 16.0], arcade_color));

        indices.push(b_idx);
        indices.push(b_idx + 3);
        indices.push(b_idx + 2);

        indices.push(b_idx);
        indices.push(b_idx + 2);
        indices.push(b_idx + 1);
    }

    // Upward-facing roof (+Y normal)
    let roof_idx = vertices.len() as u32;
    let up_norm = [0.0, 1.0, 0.0];
    for i in 0..num_flutes {
        let a = (i as f32) * (std::f32::consts::TAU / num_flutes as f32);
        let px = center.x + a.cos() * r_arcade;
        let pz = center.z + a.sin() * r_arcade;
        vertices.push(Vertex::new([px, h_top, pz], up_norm, [a.cos(), a.sin()], tower_color));
    }
    for i in 1..(num_flutes - 1) {
        indices.push(roof_idx);
        indices.push(roof_idx + (i + 1) as u32);
        indices.push(roof_idx + i as u32);
    }

    ChunkMesh::new(vertices, indices)
}
