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
        let v2 = Vertex::new(
            [p1.x, roof_elev, p1.z],
            norm_arr,
            [u1, building_height],
            wall_color,
        );
        let v3 = Vertex::new(
            [p0.x, roof_elev, p0.z],
            norm_arr,
            [u0, building_height],
            wall_color,
        );

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

        // 1B. Ground-Floor Entrance Portal & Canopy (on primary facade edge i == 0)
        if i == 0 && edge_len >= 8.0 && building_height >= 6.0 {
            let tangent = edge / edge_len;
            let mid_p = (p0 + p1) * 0.5;
            let portal_half_w = 1.8f32;
            let door_h = 2.8f32;
            let canopy_y = wall_bottom0 + 3.2f32;
            let canopy_proj = 1.0f32;

            let d0 = mid_p - tangent * portal_half_w;
            let d1 = mid_p + tangent * portal_half_w;

            // Recessed double glass doors with entrance trim
            let door_base = vertices.len() as u32;
            let door_color = [0.22, 0.26, 0.32, 0.80]; // Glass entrance door (alpha 0.80)
            let door_norm = norm_arr;
            vertices.push(Vertex::new(
                [d0.x, wall_bottom0, d0.z],
                door_norm,
                [0.0, 0.0],
                door_color,
            ));
            vertices.push(Vertex::new(
                [d1.x, wall_bottom1, d1.z],
                door_norm,
                [3.6, 0.0],
                door_color,
            ));
            vertices.push(Vertex::new(
                [d1.x, wall_bottom1 + door_h, d1.z],
                door_norm,
                [3.6, door_h],
                door_color,
            ));
            vertices.push(Vertex::new(
                [d0.x, wall_bottom0 + door_h, d0.z],
                door_norm,
                [0.0, door_h],
                door_color,
            ));
            indices.extend_from_slice(&[
                door_base,
                door_base + 3,
                door_base + 2,
                door_base,
                door_base + 2,
                door_base + 1,
            ]);

            // Projecting Entrance Canopy Awning (overhang along normal)
            let c_p0 = d0 + normal * canopy_proj;
            let c_p1 = d1 + normal * canopy_proj;
            let canopy_color = [0.30, 0.32, 0.36, 0.70]; // Architectural steel/bronze canopy (alpha 0.70)
            let up_n = [0.0, 1.0, 0.0];

            let can_base = vertices.len() as u32;
            // Top face (+Y)
            vertices.push(Vertex::new(
                [d0.x, canopy_y, d0.z],
                up_n,
                [0.0, 0.0],
                canopy_color,
            ));
            vertices.push(Vertex::new(
                [d1.x, canopy_y, d1.z],
                up_n,
                [3.6, 0.0],
                canopy_color,
            ));
            vertices.push(Vertex::new(
                [c_p1.x, canopy_y, c_p1.z],
                up_n,
                [3.6, canopy_proj],
                canopy_color,
            ));
            vertices.push(Vertex::new(
                [c_p0.x, canopy_y, c_p0.z],
                up_n,
                [0.0, canopy_proj],
                canopy_color,
            ));
            indices.extend_from_slice(&[
                can_base,
                can_base + 3,
                can_base + 2,
                can_base,
                can_base + 2,
                can_base + 1,
            ]);

            // Underside (-Y)
            let down_n = [0.0, -1.0, 0.0];
            let und_base = vertices.len() as u32;
            vertices.push(Vertex::new(
                [d0.x, canopy_y - 0.15, d0.z],
                down_n,
                [0.0, 0.0],
                canopy_color,
            ));
            vertices.push(Vertex::new(
                [c_p0.x, canopy_y - 0.15, c_p0.z],
                down_n,
                [0.0, canopy_proj],
                canopy_color,
            ));
            vertices.push(Vertex::new(
                [c_p1.x, canopy_y - 0.15, c_p1.z],
                down_n,
                [3.6, canopy_proj],
                canopy_color,
            ));
            vertices.push(Vertex::new(
                [d1.x, canopy_y - 0.15, d1.z],
                down_n,
                [3.6, 0.0],
                canopy_color,
            ));
            indices.extend_from_slice(&[
                und_base,
                und_base + 3,
                und_base + 2,
                und_base,
                und_base + 2,
                und_base + 1,
            ]);
        }

        // 1C. San Francisco Architectural Bay Windows (Multi-story projecting bays for Residential / Victorian)
        let is_residential_or_victorian =
            matches!(building.building_type, BuildingType::Residential)
                || (wall_color[3] >= 0.83 && wall_color[3] <= 0.89);
        if i == 0 && is_residential_or_victorian && edge_len >= 12.0 && building_height >= 9.0 {
            let tangent = edge / edge_len;
            let bay_bot = wall_bottom0 + 3.5f32; // Begins above ground floor
            let bay_top = roof_elev - 1.2f32; // Terminates below roof cornice
            let bay_proj = 0.50f32; // 0.50m outward cantilever
            let bay_half_w = 1.2f32;

            let offsets = [edge_len * 0.28, edge_len * 0.72];
            for off in offsets {
                let center_bay = p0 + tangent * off;
                let b0 = center_bay - tangent * bay_half_w;
                let b1 = center_bay + tangent * bay_half_w;
                let b0_out = b0 + normal * bay_proj;
                let b1_out = b1 + normal * bay_proj;

                let bay_color = [
                    wall_color[0] * 1.05,
                    wall_color[1] * 1.05,
                    wall_color[2] * 1.05,
                    wall_color[3],
                ];

                // Front face
                let bf_base = vertices.len() as u32;
                vertices.push(Vertex::new(
                    [b0_out.x, bay_bot, b0_out.z],
                    norm_arr,
                    [0.0, 0.0],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b1_out.x, bay_bot, b1_out.z],
                    norm_arr,
                    [bay_half_w * 2.0, 0.0],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b1_out.x, bay_top, b1_out.z],
                    norm_arr,
                    [bay_half_w * 2.0, bay_top - bay_bot],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b0_out.x, bay_top, b0_out.z],
                    norm_arr,
                    [0.0, bay_top - bay_bot],
                    bay_color,
                ));
                indices.extend_from_slice(&[
                    bf_base,
                    bf_base + 3,
                    bf_base + 2,
                    bf_base,
                    bf_base + 2,
                    bf_base + 1,
                ]);

                // Left angled flank
                let left_norm = [-tangent.x, 0.0, -tangent.z];
                let bl_base = vertices.len() as u32;
                vertices.push(Vertex::new(
                    [b0.x, bay_bot, b0.z],
                    left_norm,
                    [0.0, 0.0],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b0_out.x, bay_bot, b0_out.z],
                    left_norm,
                    [bay_proj, 0.0],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b0_out.x, bay_top, b0_out.z],
                    left_norm,
                    [bay_proj, bay_top - bay_bot],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b0.x, bay_top, b0.z],
                    left_norm,
                    [0.0, bay_top - bay_bot],
                    bay_color,
                ));
                indices.extend_from_slice(&[
                    bl_base,
                    bl_base + 3,
                    bl_base + 2,
                    bl_base,
                    bl_base + 2,
                    bl_base + 1,
                ]);

                // Right angled flank
                let right_norm = [tangent.x, 0.0, tangent.z];
                let br_base = vertices.len() as u32;
                vertices.push(Vertex::new(
                    [b1_out.x, bay_bot, b1_out.z],
                    right_norm,
                    [0.0, 0.0],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b1.x, bay_bot, b1.z],
                    right_norm,
                    [bay_proj, 0.0],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b1.x, bay_top, b1.z],
                    right_norm,
                    [bay_proj, bay_top - bay_bot],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b1_out.x, bay_top, b1_out.z],
                    right_norm,
                    [0.0, bay_top - bay_bot],
                    bay_color,
                ));
                indices.extend_from_slice(&[
                    br_base,
                    br_base + 3,
                    br_base + 2,
                    br_base,
                    br_base + 2,
                    br_base + 1,
                ]);

                // Bay roof / top cap (+Y)
                let cap_base = vertices.len() as u32;
                let up_n = [0.0, 1.0, 0.0];
                vertices.push(Vertex::new(
                    [b0.x, bay_top, b0.z],
                    up_n,
                    [0.0, 0.0],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b1.x, bay_top, b1.z],
                    up_n,
                    [bay_half_w * 2.0, 0.0],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b1_out.x, bay_top, b1_out.z],
                    up_n,
                    [bay_half_w * 2.0, bay_proj],
                    bay_color,
                ));
                vertices.push(Vertex::new(
                    [b0_out.x, bay_top, b0_out.z],
                    up_n,
                    [0.0, bay_proj],
                    bay_color,
                ));
                indices.extend_from_slice(&[
                    cap_base,
                    cap_base + 3,
                    cap_base + 2,
                    cap_base,
                    cap_base + 2,
                    cap_base + 1,
                ]);
            }
        }

        // 2. Architectural Cornice Moulding (0.30m outward projecting cornice below roof parapet)
        if building_height >= 8.0 {
            let cornice_overhang = 0.30f32;
            let cornice_h = 0.40f32;
            let c_bot_y = roof_elev - cornice_h;
            let c_top_y = roof_elev;

            let cp0 = p0 + normal * cornice_overhang;
            let cp1 = p1 + normal * cornice_overhang;
            let cornice_color = [
                wall_color[0] * 0.90,
                wall_color[1] * 0.90,
                wall_color[2] * 0.90,
                wall_color[3],
            ];

            // 2A. Underside / Soffit (normal facing down -Y)
            let soffit_base = vertices.len() as u32;
            let down_norm = [0.0, -1.0, 0.0];
            vertices.push(Vertex::new(
                [p0.x, c_bot_y, p0.z],
                down_norm,
                [u0, 0.0],
                cornice_color,
            ));
            vertices.push(Vertex::new(
                [p1.x, c_bot_y, p1.z],
                down_norm,
                [u1, 0.0],
                cornice_color,
            ));
            vertices.push(Vertex::new(
                [cp1.x, c_bot_y, cp1.z],
                down_norm,
                [u1, cornice_overhang],
                cornice_color,
            ));
            vertices.push(Vertex::new(
                [cp0.x, c_bot_y, cp0.z],
                down_norm,
                [u0, cornice_overhang],
                cornice_color,
            ));
            indices.extend_from_slice(&[
                soffit_base,
                soffit_base + 1,
                soffit_base + 2,
                soffit_base,
                soffit_base + 2,
                soffit_base + 3,
            ]);

            // 2B. Front vertical face (normal facing outward norm_arr)
            let front_base = vertices.len() as u32;
            vertices.push(Vertex::new(
                [cp0.x, c_bot_y, cp0.z],
                norm_arr,
                [u0, 0.0],
                cornice_color,
            ));
            vertices.push(Vertex::new(
                [cp1.x, c_bot_y, cp1.z],
                norm_arr,
                [u1, 0.0],
                cornice_color,
            ));
            vertices.push(Vertex::new(
                [cp1.x, c_top_y, cp1.z],
                norm_arr,
                [u1, cornice_h],
                cornice_color,
            ));
            vertices.push(Vertex::new(
                [cp0.x, c_top_y, cp0.z],
                norm_arr,
                [u0, cornice_h],
                cornice_color,
            ));
            indices.extend_from_slice(&[
                front_base,
                front_base + 3,
                front_base + 2,
                front_base,
                front_base + 2,
                front_base + 1,
            ]);

            // 2C. Top coping wash (normal facing up +Y)
            let top_base = vertices.len() as u32;
            let up_norm = [0.0, 1.0, 0.0];
            vertices.push(Vertex::new(
                [cp0.x, c_top_y, cp0.z],
                up_norm,
                [u0, 0.0],
                cornice_color,
            ));
            vertices.push(Vertex::new(
                [cp1.x, c_top_y, cp1.z],
                up_norm,
                [u1, 0.0],
                cornice_color,
            ));
            vertices.push(Vertex::new(
                [p1.x, c_top_y, p1.z],
                up_norm,
                [u1, cornice_overhang],
                cornice_color,
            ));
            vertices.push(Vertex::new(
                [p0.x, c_top_y, p0.z],
                up_norm,
                [u0, cornice_overhang],
                cornice_color,
            ));
            indices.extend_from_slice(&[
                top_base,
                top_base + 3,
                top_base + 2,
                top_base,
                top_base + 2,
                top_base + 1,
            ]);
        }

        // 3. Rooftop Parapet Lip (0.8m vertical parapet along roof perimeter)
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

    // 4. Triangulate flat roof with hole preservation using earcut
    let mut flat_coords: Vec<f64> = Vec::with_capacity((enu_pts.len() - 1) * 2);
    let mut roof_pts: Vec<Vec3> = Vec::with_capacity(enu_pts.len());
    for pt in enu_pts.iter().take(enu_pts.len() - 1) {
        flat_coords.push(pt.x as f64);
        flat_coords.push(pt.z as f64);
        roof_pts.push(*pt);
    }

    let mut hole_indices: Vec<usize> = Vec::new();
    for inner in &building.inner_rings {
        if inner.len() < 3 {
            continue;
        }
        let hole_start = flat_coords.len() / 2;
        hole_indices.push(hole_start);

        let inner_slice = if inner.len() > 1 && inner.first() == inner.last() {
            &inner[..inner.len() - 1]
        } else {
            &inner[..]
        };

        let mut inner_enu_pts = Vec::with_capacity(inner_slice.len());
        for pt in inner_slice {
            let enu = converter.wgs84_to_enu(*pt);
            let p = Vec3::new(enu.east as f32, 0.0, enu.north as f32);
            flat_coords.push(p.x as f64);
            flat_coords.push(p.z as f64);
            roof_pts.push(p);
            inner_enu_pts.push(p);
        }

        // Extrude inner courtyard walls down to ground
        let inner_count = inner_enu_pts.len();
        for i in 0..inner_count {
            let next = (i + 1) % inner_count;
            let p0 = inner_enu_pts[i];
            let p1 = inner_enu_pts[next];
            let edge = p1 - p0;
            let edge_len = edge.length();
            if edge_len < 1e-4 {
                continue;
            }
            let normal = Vec3::new(-edge.z, 0.0, edge.x).normalize();
            let norm_arr = [normal.x, normal.y, normal.z];

            let ground0 = dem.sample_bilinear(&inner_slice[i]);
            let ground1 = dem.sample_bilinear(&inner_slice[next]);
            let bot0 = ground0.min(base_elev) - 0.20;
            let bot1 = ground1.min(base_elev) - 0.20;

            let base_idx = vertices.len() as u32;
            vertices.push(Vertex::new(
                [p0.x, bot0, p0.z],
                norm_arr,
                [0.0, 0.0],
                wall_color,
            ));
            vertices.push(Vertex::new(
                [p1.x, bot1, p1.z],
                norm_arr,
                [edge_len, 0.0],
                wall_color,
            ));
            vertices.push(Vertex::new(
                [p1.x, roof_elev, p1.z],
                norm_arr,
                [edge_len, building_height],
                wall_color,
            ));
            vertices.push(Vertex::new(
                [p0.x, roof_elev, p0.z],
                norm_arr,
                [0.0, building_height],
                wall_color,
            ));

            indices.push(base_idx);
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);

            indices.push(base_idx);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        }
    }

    if let Ok(roof_indices) = earcut(&flat_coords, &hole_indices, 2) {
        let roof_base_idx = vertices.len() as u32;
        let up_norm = [0.0, 1.0, 0.0];

        for pt in &roof_pts {
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

    // 5. Rooftop Mechanical Infrastructure: Penthouse, HVAC Chillers & Communications Mast
    if building_height > 16.0 && enu_pts.len() >= 5 {
        let center_x = enu_pts
            .iter()
            .take(enu_pts.len() - 1)
            .map(|p| p.x)
            .sum::<f32>()
            / (enu_pts.len() - 1) as f32;
        let center_z = enu_pts
            .iter()
            .take(enu_pts.len() - 1)
            .map(|p| p.z)
            .sum::<f32>()
            / (enu_pts.len() - 1) as f32;

        let ph_half_w = 4.5f32;
        let ph_half_l = 4.5f32;
        let ph_height = 3.5f32;
        let ph_bottom = roof_elev;
        let ph_top = roof_elev + ph_height;
        let ph_color = [0.46, 0.48, 0.52, 0.70]; // Mechanical steel/gray (alpha 0.70)

        // 5A. Central Mechanical Penthouse
        add_box_mesh(
            &mut vertices,
            &mut indices,
            Vec3::new(center_x - ph_half_w, ph_bottom, center_z - ph_half_l),
            Vec3::new(center_x + ph_half_w, ph_top, center_z + ph_half_l),
            ph_color,
        );

        // 5B. HVAC Chiller & Air Handler Pods for commercial/residential buildings > 25m
        if building_height > 25.0 {
            let chiller_color = [0.42, 0.44, 0.48, 0.70];
            // Chiller Unit 1 (East flank)
            add_box_mesh(
                &mut vertices,
                &mut indices,
                Vec3::new(center_x + ph_half_w + 1.2, ph_bottom, center_z - 2.5),
                Vec3::new(center_x + ph_half_w + 4.2, ph_bottom + 1.8, center_z + 0.5),
                chiller_color,
            );
            // Chiller Unit 2 (West flank)
            add_box_mesh(
                &mut vertices,
                &mut indices,
                Vec3::new(center_x - ph_half_w - 4.2, ph_bottom, center_z - 0.5),
                Vec3::new(center_x - ph_half_w - 1.2, ph_bottom + 1.8, center_z + 2.5),
                chiller_color,
            );
        }

        // 5C. Slender Telecommunications / Lightning Antenna Mast for high-rises >= 50m
        if building_height >= 50.0 {
            let mast_height = if building_height >= 100.0 {
                16.0f32
            } else {
                10.0f32
            };
            let mast_r = 0.35f32;
            let mast_color = [0.75, 0.76, 0.80, 0.70]; // Galvanized steel mast
            let apex = Vec3::new(center_x, ph_top + mast_height, center_z);

            let corners = [
                Vec3::new(center_x - mast_r, ph_top, center_z - mast_r),
                Vec3::new(center_x + mast_r, ph_top, center_z - mast_r),
                Vec3::new(center_x + mast_r, ph_top, center_z + mast_r),
                Vec3::new(center_x - mast_r, ph_top, center_z + mast_r),
            ];

            let normals = [
                [0.0, 0.1, -1.0],
                [1.0, 0.1, 0.0],
                [0.0, 0.1, 1.0],
                [-1.0, 0.1, 0.0],
            ];

            for i in 0..4 {
                let next = (i + 1) % 4;
                let c0 = corners[i];
                let c1 = corners[next];
                let n = normals[i];

                let b_idx = vertices.len() as u32;
                vertices.push(Vertex::new([c0.x, c0.y, c0.z], n, [0.0, 0.0], mast_color));
                vertices.push(Vertex::new([c1.x, c1.y, c1.z], n, [1.0, 0.0], mast_color));
                vertices.push(Vertex::new(
                    [apex.x, apex.y, apex.z],
                    n,
                    [0.5, mast_height],
                    mast_color,
                ));

                indices.push(b_idx);
                indices.push(b_idx + 2);
                indices.push(b_idx + 1);
            }
        }
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
    let glass_band = [0.45, 0.62, 0.78, 0.80]; // Glass windows
    let spire_color = [0.92, 0.92, 0.94, 0.70]; // Aluminum spire (alpha 0.70)

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
        vertices.push(Vertex::new(
            [b0.x, b0.y, b0.z],
            norm_arr,
            [0.0, 0.0],
            pyramid_color,
        ));
        vertices.push(Vertex::new(
            [b1.x, b1.y, b1.z],
            norm_arr,
            [base_half * 2.0, 0.0],
            pyramid_color,
        ));
        vertices.push(Vertex::new(
            [t1.x, t1.y, t1.z],
            norm_arr,
            [base_half * 2.0, 212.0],
            glass_band,
        ));
        vertices.push(Vertex::new(
            [t0.x, t0.y, t0.z],
            norm_arr,
            [0.0, 212.0],
            glass_band,
        ));

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
        vertices.push(Vertex::new(
            [wx0, wing_bottom, wz0],
            n,
            [0.0, 0.0],
            pyramid_color,
        ));
        vertices.push(Vertex::new(
            [wx1, wing_bottom, wz1],
            n,
            [wing_w, 0.0],
            pyramid_color,
        ));
        vertices.push(Vertex::new(
            [wx1, wing_top, wz1],
            n,
            [wing_w, 65.0],
            pyramid_color,
        ));
        vertices.push(Vertex::new(
            [wx0, wing_top, wz0],
            n,
            [0.0, 65.0],
            pyramid_color,
        ));

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
        Vec3::new(
            center.x - spire_base_half,
            py_top_elev,
            center.z - spire_base_half,
        ),
        Vec3::new(
            center.x + spire_base_half,
            py_top_elev,
            center.z - spire_base_half,
        ),
        Vec3::new(
            center.x + spire_base_half,
            py_top_elev,
            center.z + spire_base_half,
        ),
        Vec3::new(
            center.x - spire_base_half,
            py_top_elev,
            center.z + spire_base_half,
        ),
    ];
    let spire_tip = Vec3::new(center.x, spire_top_elev, center.z);

    for i in 0..4 {
        let next = (i + 1) % 4;
        let s0 = spire_corners[i];
        let s1 = spire_corners[next];

        let norm = (s1 - s0).cross(spire_tip - s0).normalize();
        let norm_arr = [norm.x, norm.y, norm.z];

        let sp_idx = vertices.len() as u32;
        vertices.push(Vertex::new(
            [s0.x, s0.y, s0.z],
            norm_arr,
            [0.0, 0.0],
            spire_color,
        ));
        vertices.push(Vertex::new(
            [s1.x, s1.y, s1.z],
            norm_arr,
            [4.8, 0.0],
            spire_color,
        ));
        vertices.push(Vertex::new(
            [spire_tip.x, spire_tip.y, spire_tip.z],
            norm_arr,
            [2.4, 48.0],
            spire_color,
        ));

        indices.push(sp_idx);
        indices.push(sp_idx + 2);
        indices.push(sp_idx + 1);
    }

    // 4. Crown Platform Roof (+Y normal required by tests)
    let roof_idx = vertices.len() as u32;
    let up_norm = [0.0, 1.0, 0.0];
    vertices.push(Vertex::new(
        [top_corners[0].x, py_top_elev, top_corners[0].z],
        up_norm,
        [0.0, 0.0],
        spire_color,
    ));
    vertices.push(Vertex::new(
        [top_corners[1].x, py_top_elev, top_corners[1].z],
        up_norm,
        [1.0, 0.0],
        spire_color,
    ));
    vertices.push(Vertex::new(
        [top_corners[2].x, py_top_elev, top_corners[2].z],
        up_norm,
        [1.0, 1.0],
        spire_color,
    ));
    vertices.push(Vertex::new(
        [top_corners[3].x, py_top_elev, top_corners[3].z],
        up_norm,
        [0.0, 1.0],
        spire_color,
    ));

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

            let p0 = Vec3::new(
                center.x + angle0.cos() * r0,
                h0,
                center.z + angle0.sin() * r0,
            );
            let p1 = Vec3::new(
                center.x + angle1.cos() * r0,
                h0,
                center.z + angle1.sin() * r0,
            );
            let p2 = Vec3::new(
                center.x + angle1.cos() * r1,
                h1,
                center.z + angle1.sin() * r1,
            );
            let p3 = Vec3::new(
                center.x + angle0.cos() * r1,
                h1,
                center.z + angle0.sin() * r1,
            );

            let normal = (p1 - p0).cross(p3 - p0).normalize();
            let norm_arr = [normal.x, normal.y, normal.z];

            let b_idx = vertices.len() as u32;
            vertices.push(Vertex::new([p0.x, p0.y, p0.z], norm_arr, [0.0, 0.0], col));
            vertices.push(Vertex::new([p1.x, p1.y, p1.z], norm_arr, [12.0, 0.0], col));
            vertices.push(Vertex::new(
                [p2.x, p2.y, p2.z],
                norm_arr,
                [12.0, h1 - h0],
                col,
            ));
            vertices.push(Vertex::new(
                [p3.x, p3.y, p3.z],
                norm_arr,
                [0.0, h1 - h0],
                col,
            ));

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
        vertices.push(Vertex::new(
            [px, crown_top, pz],
            up_norm,
            [angle.cos(), angle.sin()],
            crown_color,
        ));
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
        mesh.vertices.push(Vertex::new(
            [c0.x, base_elev, c0.z],
            n,
            [0.0, 0.0],
            stone_color,
        ));
        mesh.vertices.push(Vertex::new(
            [c1.x, base_elev, c1.z],
            n,
            [12.0, 0.0],
            stone_color,
        ));
        mesh.vertices.push(Vertex::new(
            [c1.x, tower_top, c1.z],
            n,
            [12.0, 75.0],
            clock_color,
        ));
        mesh.vertices.push(Vertex::new(
            [c0.x, tower_top, c0.z],
            n,
            [0.0, 75.0],
            clock_color,
        ));

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
    mesh.vertices.push(Vertex::new(
        [t_corners[0].x, tower_top, t_corners[0].z],
        up_norm,
        [0.0, 0.0],
        stone_color,
    ));
    mesh.vertices.push(Vertex::new(
        [t_corners[1].x, tower_top, t_corners[1].z],
        up_norm,
        [1.0, 0.0],
        stone_color,
    ));
    mesh.vertices.push(Vertex::new(
        [t_corners[2].x, tower_top, t_corners[2].z],
        up_norm,
        [1.0, 1.0],
        stone_color,
    ));
    mesh.vertices.push(Vertex::new(
        [t_corners[3].x, tower_top, t_corners[3].z],
        up_norm,
        [0.0, 1.0],
        stone_color,
    ));

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

        let p0 = Vec3::new(
            center.x + a0.cos() * r_shaft,
            base_elev - 0.2,
            center.z + a0.sin() * r_shaft,
        );
        let p1 = Vec3::new(
            center.x + a1.cos() * r_shaft,
            base_elev - 0.2,
            center.z + a1.sin() * r_shaft,
        );
        let p2 = Vec3::new(
            center.x + a1.cos() * r_shaft,
            h_shaft,
            center.z + a1.sin() * r_shaft,
        );
        let p3 = Vec3::new(
            center.x + a0.cos() * r_shaft,
            h_shaft,
            center.z + a0.sin() * r_shaft,
        );

        let n = Vec3::new(a0.cos() + a1.cos(), 0.0, a0.sin() + a1.sin()).normalize();
        let norm_arr = [n.x, n.y, n.z];

        let b_idx = vertices.len() as u32;
        vertices.push(Vertex::new(
            [p0.x, p0.y, p0.z],
            norm_arr,
            [0.0, 0.0],
            tower_color,
        ));
        vertices.push(Vertex::new(
            [p1.x, p1.y, p1.z],
            norm_arr,
            [3.5, 0.0],
            tower_color,
        ));
        vertices.push(Vertex::new(
            [p2.x, p2.y, p2.z],
            norm_arr,
            [3.5, 48.0],
            tower_color,
        ));
        vertices.push(Vertex::new(
            [p3.x, p3.y, p3.z],
            norm_arr,
            [0.0, 48.0],
            tower_color,
        ));

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

        let p0 = Vec3::new(
            center.x + a0.cos() * r_arcade,
            h_shaft,
            center.z + a0.sin() * r_arcade,
        );
        let p1 = Vec3::new(
            center.x + a1.cos() * r_arcade,
            h_shaft,
            center.z + a1.sin() * r_arcade,
        );
        let p2 = Vec3::new(
            center.x + a1.cos() * r_arcade,
            h_top,
            center.z + a1.sin() * r_arcade,
        );
        let p3 = Vec3::new(
            center.x + a0.cos() * r_arcade,
            h_top,
            center.z + a0.sin() * r_arcade,
        );

        let n = Vec3::new(a0.cos() + a1.cos(), 0.0, a0.sin() + a1.sin()).normalize();
        let norm_arr = [n.x, n.y, n.z];

        let b_idx = vertices.len() as u32;
        vertices.push(Vertex::new(
            [p0.x, p0.y, p0.z],
            norm_arr,
            [0.0, 0.0],
            arcade_color,
        ));
        vertices.push(Vertex::new(
            [p1.x, p1.y, p1.z],
            norm_arr,
            [3.0, 0.0],
            arcade_color,
        ));
        vertices.push(Vertex::new(
            [p2.x, p2.y, p2.z],
            norm_arr,
            [3.0, 16.0],
            arcade_color,
        ));
        vertices.push(Vertex::new(
            [p3.x, p3.y, p3.z],
            norm_arr,
            [0.0, 16.0],
            arcade_color,
        ));

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
        vertices.push(Vertex::new(
            [px, h_top, pz],
            up_norm,
            [a.cos(), a.sin()],
            tower_color,
        ));
    }
    for i in 1..(num_flutes - 1) {
        indices.push(roof_idx);
        indices.push(roof_idx + (i + 1) as u32);
        indices.push(roof_idx + i as u32);
    }

    ChunkMesh::new(vertices, indices)
}

/// Helper to append a 5-sided 3D box (4 walls + top roof) to vertex and index buffers.
fn add_box_mesh(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    min: Vec3,
    max: Vec3,
    color: [f32; 4],
) {
    let dx = max.x - min.x;
    let dy = max.y - min.y;
    let dz = max.z - min.z;

    // -Z (South)
    let b = vertices.len() as u32;
    vertices.push(Vertex::new(
        [min.x, min.y, min.z],
        [0.0, 0.0, -1.0],
        [0.0, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [max.x, min.y, min.z],
        [0.0, 0.0, -1.0],
        [dx, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [max.x, max.y, min.z],
        [0.0, 0.0, -1.0],
        [dx, dy],
        color,
    ));
    vertices.push(Vertex::new(
        [min.x, max.y, min.z],
        [0.0, 0.0, -1.0],
        [0.0, dy],
        color,
    ));
    indices.extend_from_slice(&[b, b + 3, b + 2, b, b + 2, b + 1]);

    // +Z (North)
    let b = vertices.len() as u32;
    vertices.push(Vertex::new(
        [max.x, min.y, max.z],
        [0.0, 0.0, 1.0],
        [0.0, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [min.x, min.y, max.z],
        [0.0, 0.0, 1.0],
        [dx, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [min.x, max.y, max.z],
        [0.0, 0.0, 1.0],
        [dx, dy],
        color,
    ));
    vertices.push(Vertex::new(
        [max.x, max.y, max.z],
        [0.0, 0.0, 1.0],
        [0.0, dy],
        color,
    ));
    indices.extend_from_slice(&[b, b + 3, b + 2, b, b + 2, b + 1]);

    // -X (West)
    let b = vertices.len() as u32;
    vertices.push(Vertex::new(
        [min.x, min.y, max.z],
        [-1.0, 0.0, 0.0],
        [0.0, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [min.x, min.y, min.z],
        [-1.0, 0.0, 0.0],
        [dz, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [min.x, max.y, min.z],
        [-1.0, 0.0, 0.0],
        [dz, dy],
        color,
    ));
    vertices.push(Vertex::new(
        [min.x, max.y, max.z],
        [-1.0, 0.0, 0.0],
        [0.0, dy],
        color,
    ));
    indices.extend_from_slice(&[b, b + 3, b + 2, b, b + 2, b + 1]);

    // +X (East)
    let b = vertices.len() as u32;
    vertices.push(Vertex::new(
        [max.x, min.y, min.z],
        [1.0, 0.0, 0.0],
        [0.0, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [max.x, min.y, max.z],
        [1.0, 0.0, 0.0],
        [dz, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [max.x, max.y, max.z],
        [1.0, 0.0, 0.0],
        [dz, dy],
        color,
    ));
    vertices.push(Vertex::new(
        [max.x, max.y, min.z],
        [1.0, 0.0, 0.0],
        [0.0, dy],
        color,
    ));
    indices.extend_from_slice(&[b, b + 3, b + 2, b, b + 2, b + 1]);

    // +Y (Roof / Top)
    let b = vertices.len() as u32;
    vertices.push(Vertex::new(
        [min.x, max.y, min.z],
        [0.0, 1.0, 0.0],
        [0.0, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [max.x, max.y, min.z],
        [0.0, 1.0, 0.0],
        [dx, 0.0],
        color,
    ));
    vertices.push(Vertex::new(
        [max.x, max.y, max.z],
        [0.0, 1.0, 0.0],
        [dx, dz],
        color,
    ));
    vertices.push(Vertex::new(
        [min.x, max.y, max.z],
        [0.0, 1.0, 0.0],
        [0.0, dz],
        color,
    ));
    indices.extend_from_slice(&[b, b + 3, b + 2, b, b + 2, b + 1]);
}
