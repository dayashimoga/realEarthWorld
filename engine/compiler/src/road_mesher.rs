//! 3D Road ribbon generation with sidewalks, curbs, elevated bridge piers, guardrails, and streetlights.

use crate::mesh::{ChunkMesh, Vertex};
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::{HighwayType, OsmRoadFeature};

/// Generates a comprehensive 3D road infrastructure mesh:
/// - Asphalt carriageway with metric UV mapping for lane markings
/// - Concrete curbs and raised sidewalks along urban surface streets
/// - Heavy bridge box girders, safety guardrails, and vertical ground piers for elevated structures
/// - Tunnel portal structures for subterranean routes
/// - 3D Streetlight poles along urban arterials
pub fn mesh_road(road: &OsmRoadFeature, converter: &EnuConverter, dem: &DemGrid) -> ChunkMesh {
    if road.geometry.len() < 2 {
        return ChunkMesh::default();
    }

    let lane_width = road.highway_type.default_lane_width();
    let total_width = (road.lanes.max(1) as f32) * lane_width;
    let half_width = total_width * 0.5;

    let is_elevated = road.layer > 0 || road.is_bridge;
    let is_subterranean = road.layer < 0 || road.is_tunnel;

    // Convert centerline to 3D local ENU positions
    let mut centerlines: Vec<Vec3> = Vec::with_capacity(road.geometry.len());
    let mut ground_elevs: Vec<f32> = Vec::with_capacity(road.geometry.len());

    for pt in &road.geometry {
        let enu = converter.wgs84_to_enu(*pt);
        let ground_elev = dem.sample_bilinear(pt);
        ground_elevs.push(ground_elev);

        let final_elev = if is_elevated {
            let layers = road.layer.max(1) as f32;
            ground_elev + layers * 6.0
        } else if is_subterranean {
            let layers = road.layer.abs().max(1) as f32;
            ground_elev - layers * 5.0
        } else {
            // Offset above ground to prevent z-fighting with terrain
            ground_elev + 0.20
        };

        centerlines.push(Vec3::new(enu.east as f32, final_elev, enu.north as f32));
    }

    let n = centerlines.len();
    let mut seg_perps = Vec::with_capacity(n - 1);
    for i in 0..(n - 1) {
        let p0 = centerlines[i];
        let p1 = centerlines[i + 1];
        let forward = Vec3::new(p1.x - p0.x, 0.0, p1.z - p0.z);
        let len = forward.length();
        let perp = if len > 1e-4 {
            Vec3::new(-forward.z, 0.0, forward.x) / len
        } else {
            Vec3::X
        };
        seg_perps.push(perp);
    }

    if seg_perps.is_empty() {
        return ChunkMesh::default();
    }

    let mut miter_perps = Vec::with_capacity(n);
    for i in 0..n {
        if i == 0 {
            miter_perps.push((seg_perps[0], 1.0f32));
        } else if i == n - 1 {
            miter_perps.push((seg_perps[n - 2], 1.0f32));
        } else {
            let p_prev = seg_perps[i - 1];
            let p_curr = seg_perps[i];
            let miter_sum = p_prev + p_curr;
            if miter_sum.length_squared() < 1e-4 {
                miter_perps.push((p_prev, 1.0f32));
            } else {
                let miter = miter_sum.normalize();
                let cos_half = ((1.0 + p_prev.dot(p_curr)) * 0.5).max(0.0).sqrt();
                let miter_factor = if cos_half > 0.3 {
                    (1.0 / cos_half).min(2.0)
                } else {
                    2.0
                };
                miter_perps.push((miter, miter_factor));
            }
        }
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Road surface color: asphalt dark grey (alpha 0.90 triggers road marking shader)
    let road_color = if is_elevated {
        [0.30, 0.30, 0.34, 0.90] // Bridge deck concrete-asphalt
    } else {
        [0.22, 0.22, 0.24, 0.90] // Fresh asphalt
    };

    let up_norm = [0.0, 1.0, 0.0];

    // =========================================================================
    // 1. Asphalt Carriageway Ribbon with Metric UVs
    // =========================================================================
    let mut cumulative_dist = 0.0f32;
    let mut left_edge_pts = Vec::with_capacity(n);
    let mut right_edge_pts = Vec::with_capacity(n);

    for i in 0..n {
        if i > 0 {
            cumulative_dist += (centerlines[i] - centerlines[i - 1]).length();
        }

        let pt = centerlines[i];
        let (miter, factor) = miter_perps[i];
        let offset = miter * (half_width * factor);

        let left = pt + offset;
        let right = pt - offset;

        left_edge_pts.push(left);
        right_edge_pts.push(right);

        let v = cumulative_dist;
        vertices.push(Vertex::new(
            [left.x, left.y, left.z],
            up_norm,
            [0.0, v],
            road_color,
        ));
        vertices.push(Vertex::new(
            [right.x, right.y, right.z],
            up_norm,
            [1.0, v],
            road_color,
        ));
    }

    // CCW triangle winding facing upwards (+Y)
    for i in 0..(n - 1) {
        let base = (i * 2) as u32;
        indices.push(base);
        indices.push(base + 3);
        indices.push(base + 1);

        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    ChunkMesh::new(vertices, indices)
}

/// Generates complementary 3D urban road infrastructure:
/// - Raised concrete curbs and sidewalks along surface streets
/// - Bridge box girders, guardrail barriers, and vertical ground piers for elevated structures
/// - 3D Streetlight poles along major urban arterials
pub fn mesh_road_infrastructure(
    road: &OsmRoadFeature,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    if road.geometry.len() < 2 {
        return ChunkMesh::default();
    }

    let lane_width = road.highway_type.default_lane_width();
    let total_width = (road.lanes.max(1) as f32) * lane_width;
    let half_width = total_width * 0.5;

    let is_elevated = road.layer > 0 || road.is_bridge;
    let is_subterranean = road.layer < 0 || road.is_tunnel;
    let is_surface = !is_elevated && !is_subterranean;

    let mut centerlines: Vec<Vec3> = Vec::with_capacity(road.geometry.len());
    let mut ground_elevs: Vec<f32> = Vec::with_capacity(road.geometry.len());

    for pt in &road.geometry {
        let enu = converter.wgs84_to_enu(*pt);
        let ground_elev = dem.sample_bilinear(pt);
        ground_elevs.push(ground_elev);

        let final_elev = if is_elevated {
            let layers = road.layer.max(1) as f32;
            ground_elev + layers * 6.0
        } else if is_subterranean {
            let layers = road.layer.abs().max(1) as f32;
            ground_elev - layers * 5.0
        } else {
            ground_elev + 0.20
        };

        centerlines.push(Vec3::new(enu.east as f32, final_elev, enu.north as f32));
    }

    let n = centerlines.len();
    let mut seg_perps = Vec::with_capacity(n - 1);
    for i in 0..(n - 1) {
        let p0 = centerlines[i];
        let p1 = centerlines[i + 1];
        let forward = Vec3::new(p1.x - p0.x, 0.0, p1.z - p0.z);
        let len = forward.length();
        let perp = if len > 1e-4 {
            Vec3::new(-forward.z, 0.0, forward.x) / len
        } else {
            Vec3::X
        };
        seg_perps.push(perp);
    }

    if seg_perps.is_empty() {
        return ChunkMesh::default();
    }

    let mut miter_perps = Vec::with_capacity(n);
    for i in 0..n {
        if i == 0 {
            miter_perps.push((seg_perps[0], 1.0f32));
        } else if i == n - 1 {
            miter_perps.push((seg_perps[n - 2], 1.0f32));
        } else {
            let p_prev = seg_perps[i - 1];
            let p_curr = seg_perps[i];
            let miter_sum = p_prev + p_curr;
            if miter_sum.length_squared() < 1e-4 {
                miter_perps.push((p_prev, 1.0f32));
            } else {
                let miter = miter_sum.normalize();
                let cos_half = ((1.0 + p_prev.dot(p_curr)) * 0.5).max(0.0).sqrt();
                let miter_factor = if cos_half > 0.3 {
                    (1.0 / cos_half).min(2.0)
                } else {
                    2.0
                };
                miter_perps.push((miter, miter_factor));
            }
        }
    }

    let mut left_edge_pts = Vec::with_capacity(n);
    let mut right_edge_pts = Vec::with_capacity(n);
    for i in 0..n {
        let pt = centerlines[i];
        let (miter, factor) = miter_perps[i];
        let offset = miter * (half_width * factor);
        left_edge_pts.push(pt + offset);
        right_edge_pts.push(pt - offset);
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let sidewalk_color = [0.72, 0.71, 0.69, 0.92]; // Concrete sidewalk / curb
    let barrier_color = [0.65, 0.65, 0.68, 0.86]; // Concrete guardrail / bridge girder
    let steel_pole_color = [0.35, 0.38, 0.42, 0.70]; // Streetlight pole
    let up_norm = [0.0, 1.0, 0.0];

    // =========================================================================
    // 1. Concrete Curbs & Raised Sidewalks along Surface Streets
    // =========================================================================
    if is_surface && road.highway_type != HighwayType::Motorway {
        let sidewalk_w = 2.0f32;
        let curb_h = 0.16f32;

        for side in &[-1.0f32, 1.0f32] {
            let mut cum_dist = 0.0f32;

            // 1A. Vertical Concrete Curb Riser (connecting road asphalt edge to top of curb)
            let riser_base_idx = vertices.len() as u32;
            for i in 0..n {
                let road_edge = if *side > 0.0 {
                    left_edge_pts[i]
                } else {
                    right_edge_pts[i]
                };
                let (miter, factor) = miter_perps[i];
                let out_dir = miter * (*side * factor);
                let curb_top = road_edge + Vec3::new(0.0, curb_h, 0.0);

                let v = if i > 0 {
                    cum_dist += (centerlines[i] - centerlines[i - 1]).length();
                    cum_dist * 0.5
                } else {
                    0.0
                };

                let street_norm = if out_dir.length_squared() > 1e-4 {
                    -out_dir.normalize()
                } else {
                    -Vec3::X
                };
                let street_norm_arr = [street_norm.x, 0.0, street_norm.z];

                vertices.push(Vertex::new(
                    [road_edge.x, road_edge.y, road_edge.z],
                    street_norm_arr,
                    [0.0, v],
                    sidewalk_color,
                ));
                vertices.push(Vertex::new(
                    [curb_top.x, curb_top.y, curb_top.z],
                    street_norm_arr,
                    [curb_h, v],
                    sidewalk_color,
                ));
            }

            for i in 0..(n - 1) {
                let base = riser_base_idx + (i * 2) as u32;
                if *side > 0.0 {
                    indices.push(base);
                    indices.push(base + 1);
                    indices.push(base + 2);

                    indices.push(base + 1);
                    indices.push(base + 3);
                    indices.push(base + 2);
                } else {
                    indices.push(base);
                    indices.push(base + 2);
                    indices.push(base + 1);

                    indices.push(base + 1);
                    indices.push(base + 2);
                    indices.push(base + 3);
                }
            }

            // 1B. Sidewalk Top Deck (horizontal walking surface with scored expansion joints)
            let deck_base_idx = vertices.len() as u32;
            cum_dist = 0.0;
            for i in 0..n {
                let road_edge = if *side > 0.0 {
                    left_edge_pts[i]
                } else {
                    right_edge_pts[i]
                };
                let (miter, factor) = miter_perps[i];
                let out_dir = miter * (*side * factor);

                let curb_top = road_edge + Vec3::new(0.0, curb_h, 0.0);
                let sidewalk_outer = curb_top + out_dir * sidewalk_w;

                let v = if i > 0 {
                    cum_dist += (centerlines[i] - centerlines[i - 1]).length();
                    cum_dist * 0.5
                } else {
                    0.0
                };

                vertices.push(Vertex::new(
                    [curb_top.x, curb_top.y, curb_top.z],
                    up_norm,
                    [0.0, v],
                    sidewalk_color,
                ));
                vertices.push(Vertex::new(
                    [sidewalk_outer.x, sidewalk_outer.y, sidewalk_outer.z],
                    up_norm,
                    [1.0, v],
                    sidewalk_color,
                ));
            }

            for i in 0..(n - 1) {
                let base = deck_base_idx + (i * 2) as u32;
                if *side > 0.0 {
                    indices.push(base);
                    indices.push(base + 3);
                    indices.push(base + 1);

                    indices.push(base);
                    indices.push(base + 2);
                    indices.push(base + 3);
                } else {
                    indices.push(base);
                    indices.push(base + 1);
                    indices.push(base + 3);

                    indices.push(base);
                    indices.push(base + 3);
                    indices.push(base + 2);
                }
            }

            // 1C. Outer Sidewalk Drop (vertical drop face sealing to terrain grade)
            let drop_base_idx = vertices.len() as u32;
            cum_dist = 0.0;
            for i in 0..n {
                let road_edge = if *side > 0.0 {
                    left_edge_pts[i]
                } else {
                    right_edge_pts[i]
                };
                let (miter, factor) = miter_perps[i];
                let out_dir = miter * (*side * factor);

                let curb_top = road_edge + Vec3::new(0.0, curb_h, 0.0);
                let sidewalk_outer = curb_top + out_dir * sidewalk_w;
                let sidewalk_outer_bot = sidewalk_outer - Vec3::new(0.0, curb_h + 0.10, 0.0);

                let v = if i > 0 {
                    cum_dist += (centerlines[i] - centerlines[i - 1]).length();
                    cum_dist * 0.5
                } else {
                    0.0
                };

                let out_norm = if out_dir.length_squared() > 1e-4 {
                    out_dir.normalize()
                } else {
                    Vec3::X
                };
                let out_norm_arr = [out_norm.x, 0.0, out_norm.z];

                vertices.push(Vertex::new(
                    [sidewalk_outer.x, sidewalk_outer.y, sidewalk_outer.z],
                    out_norm_arr,
                    [0.0, v],
                    sidewalk_color,
                ));
                vertices.push(Vertex::new(
                    [
                        sidewalk_outer_bot.x,
                        sidewalk_outer_bot.y,
                        sidewalk_outer_bot.z,
                    ],
                    out_norm_arr,
                    [curb_h + 0.10, v],
                    sidewalk_color,
                ));
            }

            for i in 0..(n - 1) {
                let base = drop_base_idx + (i * 2) as u32;
                if *side > 0.0 {
                    indices.push(base);
                    indices.push(base + 1);
                    indices.push(base + 2);

                    indices.push(base + 1);
                    indices.push(base + 3);
                    indices.push(base + 2);
                } else {
                    indices.push(base);
                    indices.push(base + 2);
                    indices.push(base + 1);

                    indices.push(base + 1);
                    indices.push(base + 2);
                    indices.push(base + 3);
                }
            }
        }
    }

    // =========================================================================
    // 3. Elevated Bridge Box Girders, Concrete Barriers & Vertical Ground Piers
    // =========================================================================
    if is_elevated {
        let girder_depth = 1.6f32;
        let barrier_h = 0.95f32;

        // A. Guardrail Barriers along Left & Right bridge shoulders
        for side in &[-1.0f32, 1.0f32] {
            let start_b_idx = vertices.len() as u32;
            let mut cum_dist = 0.0f32;

            for i in 0..n {
                if i > 0 {
                    cum_dist += (centerlines[i] - centerlines[i - 1]).length();
                }
                let pt = if *side > 0.0 {
                    left_edge_pts[i]
                } else {
                    right_edge_pts[i]
                };
                let pt_top = pt + Vec3::new(0.0, barrier_h, 0.0);
                let (miter, _) = miter_perps[i];
                let norm = miter * (*side);
                let norm_arr = [norm.x, norm.y, norm.z];

                vertices.push(Vertex::new(
                    [pt.x, pt.y, pt.z],
                    norm_arr,
                    [0.0, cum_dist],
                    barrier_color,
                ));
                vertices.push(Vertex::new(
                    [pt_top.x, pt_top.y, pt_top.z],
                    norm_arr,
                    [1.0, cum_dist],
                    barrier_color,
                ));
            }

            for i in 0..(n - 1) {
                let base = start_b_idx + (i * 2) as u32;
                indices.push(base);
                indices.push(base + 3);
                indices.push(base + 1);

                indices.push(base);
                indices.push(base + 2);
                indices.push(base + 3);
            }
        }

        // B. Solid Bridge Deck Soffit underneath the carriageway
        let soffit_idx = vertices.len() as u32;
        let down_norm = [0.0, -1.0, 0.0];
        for i in 0..n {
            let left_bot = left_edge_pts[i] - Vec3::new(0.0, girder_depth, 0.0);
            let right_bot = right_edge_pts[i] - Vec3::new(0.0, girder_depth, 0.0);
            vertices.push(Vertex::new(
                [left_bot.x, left_bot.y, left_bot.z],
                down_norm,
                [0.0, 0.0],
                barrier_color,
            ));
            vertices.push(Vertex::new(
                [right_bot.x, right_bot.y, right_bot.z],
                down_norm,
                [1.0, 0.0],
                barrier_color,
            ));
        }
        for i in 0..(n - 1) {
            let base = soffit_idx + (i * 2) as u32;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 3);

            indices.push(base);
            indices.push(base + 3);
            indices.push(base + 2);
        }

        // C. Heavy Vertical Ground Support Piers spaced every ~35m
        let mut dist_since_pier = 20.0f32; // Start with first pier soon
        for i in 1..n {
            let seg_len = (centerlines[i] - centerlines[i - 1]).length();
            dist_since_pier += seg_len;

            if dist_since_pier >= 35.0 {
                dist_since_pier = 0.0;

                let pt = centerlines[i];
                let deck_bot = pt.y - girder_depth;
                let ground_y = ground_elevs[i];

                if deck_bot > ground_y + 1.0 {
                    let pier_w = 1.8f32; // Column cross-section
                    let p_x = pt.x;
                    let p_z = pt.z;

                    let c0 = Vec3::new(p_x - pier_w, ground_y, p_z - pier_w);
                    let c1 = Vec3::new(p_x + pier_w, ground_y, p_z - pier_w);
                    let c2 = Vec3::new(p_x + pier_w, ground_y, p_z + pier_w);
                    let c3 = Vec3::new(p_x - pier_w, ground_y, p_z + pier_w);

                    let corners = [c0, c1, c2, c3];
                    let p_normals = [
                        [0.0, 0.0, -1.0],
                        [1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0],
                        [-1.0, 0.0, 0.0],
                    ];

                    for f in 0..4 {
                        let next = (f + 1) % 4;
                        let corner0 = corners[f];
                        let corner1 = corners[next];
                        let n_arr = p_normals[f];

                        let b_idx = vertices.len() as u32;
                        vertices.push(Vertex::new(
                            [corner0.x, ground_y, corner0.z],
                            n_arr,
                            [0.0, 0.0],
                            barrier_color,
                        ));
                        vertices.push(Vertex::new(
                            [corner1.x, ground_y, corner1.z],
                            n_arr,
                            [1.0, 0.0],
                            barrier_color,
                        ));
                        vertices.push(Vertex::new(
                            [corner1.x, deck_bot, corner1.z],
                            n_arr,
                            [1.0, 1.0],
                            barrier_color,
                        ));
                        vertices.push(Vertex::new(
                            [corner0.x, deck_bot, corner0.z],
                            n_arr,
                            [0.0, 1.0],
                            barrier_color,
                        ));

                        indices.push(b_idx);
                        indices.push(b_idx + 3);
                        indices.push(b_idx + 2);

                        indices.push(b_idx);
                        indices.push(b_idx + 2);
                        indices.push(b_idx + 1);
                    }
                }
            }
        }
    }

    // =========================================================================
    // 4. Urban Street Furniture: 3D Streetlight Poles along Major Arterials
    // =========================================================================
    let is_arterial = matches!(
        road.highway_type,
        HighwayType::Primary | HighwayType::Secondary | HighwayType::Tertiary
    );

    if is_surface && is_arterial && n >= 2 {
        let mut dist_since_light = 15.0f32;
        let pole_h = 6.2f32;
        let arm_len = 2.4f32;

        for i in 1..n {
            let seg_len = (centerlines[i] - centerlines[i - 1]).length();
            dist_since_light += seg_len;

            if dist_since_light >= 38.0 {
                dist_since_light = 0.0;

                // Place streetlight pole along right sidewalk
                let base_pt = right_edge_pts[i] + (miter_perps[i].0 * -1.8f32);
                let pole_top = base_pt + Vec3::new(0.0, pole_h, 0.0);
                let arm_tip = pole_top + (miter_perps[i].0 * arm_len); // Reach over roadway

                // Thin vertical pole quad
                let w = 0.12f32;
                let pole_base_idx = vertices.len() as u32;
                let norm = [miter_perps[i].0.x, 0.0, miter_perps[i].0.z];

                vertices.push(Vertex::new(
                    [base_pt.x - w, base_pt.y, base_pt.z],
                    norm,
                    [0.0, 0.0],
                    steel_pole_color,
                ));
                vertices.push(Vertex::new(
                    [base_pt.x + w, base_pt.y, base_pt.z],
                    norm,
                    [1.0, 0.0],
                    steel_pole_color,
                ));
                vertices.push(Vertex::new(
                    [pole_top.x + w, pole_top.y, pole_top.z],
                    norm,
                    [1.0, 1.0],
                    steel_pole_color,
                ));
                vertices.push(Vertex::new(
                    [pole_top.x - w, pole_top.y, pole_top.z],
                    norm,
                    [0.0, 1.0],
                    steel_pole_color,
                ));

                indices.push(pole_base_idx);
                indices.push(pole_base_idx + 3);
                indices.push(pole_base_idx + 2);

                indices.push(pole_base_idx);
                indices.push(pole_base_idx + 2);
                indices.push(pole_base_idx + 1);

                // Cantilever luminaire arm extending across lane
                let arm_base_idx = vertices.len() as u32;
                let down_n = [0.0, -1.0, 0.0];
                let light_color = [0.95, 0.90, 0.70, 0.70]; // Warm luminaire head

                vertices.push(Vertex::new(
                    [pole_top.x, pole_top.y, pole_top.z],
                    down_n,
                    [0.0, 0.0],
                    steel_pole_color,
                ));
                vertices.push(Vertex::new(
                    [arm_tip.x, arm_tip.y, arm_tip.z],
                    down_n,
                    [1.0, 0.0],
                    light_color,
                ));
                vertices.push(Vertex::new(
                    [arm_tip.x, arm_tip.y - 0.2, arm_tip.z],
                    down_n,
                    [1.0, 1.0],
                    light_color,
                ));

                indices.push(arm_base_idx);
                indices.push(arm_base_idx + 2);
                indices.push(arm_base_idx + 1);
            }
        }
    }

    ChunkMesh::new(vertices, indices)
}
