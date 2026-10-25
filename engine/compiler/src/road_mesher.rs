//! 3D Road ribbon generation with sidewalks, curbs, elevated bridge piers, guardrails, and streetlights.

use crate::mesh::{ChunkMesh, Vertex};
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::{HighwayType, OsmRoadFeature};

/// Detailed options for road ribbon and infrastructure mesh generation.
#[derive(Debug, Clone, Default)]
pub struct RoadMeshingOptions<'a> {
    /// Contiguous segment index range [start_seg, end_seg] to mesh (inclusive, 0 <= start_seg <= end_seg < n - 1).
    /// If None, all segments of the road feature are meshed.
    pub segment_range: Option<(usize, usize)>,
    /// Set of shared junction node coordinates `(lat_1e7, lon_1e7, layer)` where connecting roads intersect.
    /// At junction nodes, the curb riser drops flush to the carriageway (`curb_h = 0.0`), preventing curbs from slicing across intersections.
    pub junction_coords: Option<&'a std::collections::HashSet<(i64, i64, i8)>>,
}

/// Generates a comprehensive 3D road infrastructure mesh with default whole-road options.
pub fn mesh_road(road: &OsmRoadFeature, converter: &EnuConverter, dem: &DemGrid) -> ChunkMesh {
    mesh_road_with_options(road, converter, dem, &RoadMeshingOptions::default())
}

/// Generates a comprehensive 3D road asphalt carriageway ribbon with options for chunk segment range and intersection curb cuts.
pub fn mesh_road_with_options(
    road: &OsmRoadFeature,
    converter: &EnuConverter,
    dem: &DemGrid,
    options: &RoadMeshingOptions,
) -> ChunkMesh {
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
    if n < 2 {
        return ChunkMesh::default();
    }

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
                    (1.0 / cos_half).min(1.8) // Clamp miter factor to 1.8 for smooth corner geometry
                } else {
                    1.8
                };
                miter_perps.push((miter, miter_factor));
            }
        }
    }

    // Determine active segment range
    let (start_seg, end_seg) = match options.segment_range {
        Some((s, e)) => {
            let s_clamped = s.min(n.saturating_sub(2));
            let e_clamped = e.min(n.saturating_sub(2)).max(s_clamped);
            (s_clamped, e_clamped)
        }
        None => (0, n.saturating_sub(2)),
    };

    if start_seg > end_seg || n < 2 {
        return ChunkMesh::default();
    }

    let start_node = start_seg;
    let end_node = end_seg + 1;

    let mut cumulative_dist = 0.0f32;
    let mut node_cum_dists = Vec::with_capacity(n);
    for i in 0..n {
        if i > 0 {
            cumulative_dist += (centerlines[i] - centerlines[i - 1]).length();
        }
        node_cum_dists.push(cumulative_dist);
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
    for i in start_node..=end_node {
        let pt = centerlines[i];
        let (miter, factor) = miter_perps[i];
        let offset = miter * (half_width * factor);

        let left = pt + offset;
        let right = pt - offset;

        let v = node_cum_dists[i];
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
    for seg in start_seg..=end_seg {
        let seg_local = (seg - start_node) as u32;
        let base = seg_local * 2;
        indices.push(base);
        indices.push(base + 3);
        indices.push(base + 1);

        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    ChunkMesh::new(vertices, indices)
}

/// Generates complementary 3D urban road infrastructure with default whole-road options:
/// - Raised concrete curbs and sidewalks along surface streets
/// - Bridge box girders, guardrail barriers, and vertical ground piers for elevated structures
/// - 3D Streetlight poles along major urban arterials
pub fn mesh_road_infrastructure(
    road: &OsmRoadFeature,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    mesh_road_infrastructure_with_options(road, converter, dem, &RoadMeshingOptions::default())
}

/// Generates complementary 3D urban road infrastructure with options for chunk segment range and intersection curb cuts.
pub fn mesh_road_infrastructure_with_options(
    road: &OsmRoadFeature,
    converter: &EnuConverter,
    dem: &DemGrid,
    options: &RoadMeshingOptions,
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
    if n < 2 {
        return ChunkMesh::default();
    }

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
                    (1.0 / cos_half).min(1.8)
                } else {
                    1.8
                };
                miter_perps.push((miter, miter_factor));
            }
        }
    }

    // Determine active segment range
    let (start_seg, end_seg) = match options.segment_range {
        Some((s, e)) => {
            let s_clamped = s.min(n.saturating_sub(2));
            let e_clamped = e.min(n.saturating_sub(2)).max(s_clamped);
            (s_clamped, e_clamped)
        }
        None => (0, n.saturating_sub(2)),
    };

    if start_seg > end_seg || n < 2 {
        return ChunkMesh::default();
    }

    let start_node = start_seg;
    let end_node = end_seg + 1;

    let mut left_edge_pts = Vec::with_capacity(n);
    let mut right_edge_pts = Vec::with_capacity(n);
    for i in 0..n {
        let pt = centerlines[i];
        let (miter, factor) = miter_perps[i];
        let offset = miter * (half_width * factor);
        left_edge_pts.push(pt + offset);
        right_edge_pts.push(pt - offset);
    }

    let mut cumulative_dist = 0.0f32;
    let mut node_cum_dists = Vec::with_capacity(n);
    for i in 0..n {
        if i > 0 {
            cumulative_dist += (centerlines[i] - centerlines[i - 1]).length();
        }
        node_cum_dists.push(cumulative_dist);
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

        for side in &[-1.0f32, 1.0f32] {
            // 1A. Vertical Concrete Curb Riser (connecting road asphalt edge to top of curb)
            // At intersections, curb height drops smoothly to 0.0 to form ADA curb ramp / crossing
            let riser_base_idx = vertices.len() as u32;
            for i in start_node..=end_node {
                let road_edge = if *side > 0.0 {
                    left_edge_pts[i]
                } else {
                    right_edge_pts[i]
                };
                let (miter, factor) = miter_perps[i];
                let out_dir = miter * (*side * factor);

                let coord = road.geometry[i];
                let key = (
                    (coord.lat * 1e7).round() as i64,
                    (coord.lon * 1e7).round() as i64,
                    road.layer,
                );
                let is_junction = options
                    .junction_coords
                    .map(|j| j.contains(&key))
                    .unwrap_or(false);
                let curb_h = if is_junction { 0.0f32 } else { 0.16f32 };
                let curb_top = road_edge + Vec3::new(0.0, curb_h, 0.0);

                let v = node_cum_dists[i] * 0.5;

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

            for seg in start_seg..=end_seg {
                let seg_local = (seg - start_node) as u32;
                let base = riser_base_idx + seg_local * 2;
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
            for i in start_node..=end_node {
                let road_edge = if *side > 0.0 {
                    left_edge_pts[i]
                } else {
                    right_edge_pts[i]
                };
                let (miter, factor) = miter_perps[i];
                let out_dir = miter * (*side * factor);

                let coord = road.geometry[i];
                let key = (
                    (coord.lat * 1e7).round() as i64,
                    (coord.lon * 1e7).round() as i64,
                    road.layer,
                );
                let is_junction = options
                    .junction_coords
                    .map(|j| j.contains(&key))
                    .unwrap_or(false);
                let curb_h = if is_junction { 0.0f32 } else { 0.16f32 };

                let curb_top = road_edge + Vec3::new(0.0, curb_h, 0.0);
                let sidewalk_outer = curb_top + out_dir * sidewalk_w;

                let v = node_cum_dists[i] * 0.5;

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

            for seg in start_seg..=end_seg {
                let seg_local = (seg - start_node) as u32;
                let base = deck_base_idx + seg_local * 2;
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

            // 1C. Outer Sidewalk Drop (vertical drop face sealing into terrain grade)
            // Evaluates DEM terrain elevation at the outer edge coordinate and extends
            // 0.20m below the DEM surface to guarantee complete elimination of floating gaps.
            let drop_base_idx = vertices.len() as u32;
            for i in start_node..=end_node {
                let road_edge = if *side > 0.0 {
                    left_edge_pts[i]
                } else {
                    right_edge_pts[i]
                };
                let (miter, factor) = miter_perps[i];
                let out_dir = miter * (*side * factor);

                let coord = road.geometry[i];
                let key = (
                    (coord.lat * 1e7).round() as i64,
                    (coord.lon * 1e7).round() as i64,
                    road.layer,
                );
                let is_junction = options
                    .junction_coords
                    .map(|j| j.contains(&key))
                    .unwrap_or(false);
                let curb_h = if is_junction { 0.0f32 } else { 0.16f32 };

                let curb_top = road_edge + Vec3::new(0.0, curb_h, 0.0);
                let sidewalk_outer = curb_top + out_dir * sidewalk_w;

                // Sample DEM at outer sidewalk point and drop 0.20m below DEM to seal completely
                let outer_enu = realearth_shared::coordinates::EnuCoord::new(
                    sidewalk_outer.x as f64,
                    sidewalk_outer.z as f64,
                    0.0,
                );
                let outer_wgs = converter.enu_to_wgs84(outer_enu);
                let outer_dem_y = dem.sample_bilinear(&outer_wgs);
                let sidewalk_outer_bot =
                    Vec3::new(sidewalk_outer.x, outer_dem_y - 0.20, sidewalk_outer.z);

                let v = node_cum_dists[i] * 0.5;

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
                    [curb_h + 0.25, v],
                    sidewalk_color,
                ));
            }

            for seg in start_seg..=end_seg {
                let seg_local = (seg - start_node) as u32;
                let base = drop_base_idx + seg_local * 2;
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
    // 2. Elevated Bridge Box Girders, Concrete Barriers & Vertical Ground Piers
    // =========================================================================
    if is_elevated {
        let girder_depth = 1.6f32;
        let barrier_h = 0.95f32;

        // A. Guardrail Barriers along Left & Right bridge shoulders
        for side in &[-1.0f32, 1.0f32] {
            let start_b_idx = vertices.len() as u32;

            for i in start_node..=end_node {
                let pt = if *side > 0.0 {
                    left_edge_pts[i]
                } else {
                    right_edge_pts[i]
                };
                let pt_top = pt + Vec3::new(0.0, barrier_h, 0.0);
                let (miter, _) = miter_perps[i];
                let norm = miter * (*side);
                let norm_arr = [norm.x, norm.y, norm.z];
                let cum_dist = node_cum_dists[i];

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

            for seg in start_seg..=end_seg {
                let seg_local = (seg - start_node) as u32;
                let base = start_b_idx + seg_local * 2;
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
        for i in start_node..=end_node {
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
        for seg in start_seg..=end_seg {
            let seg_local = (seg - start_node) as u32;
            let base = soffit_idx + seg_local * 2;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 3);

            indices.push(base);
            indices.push(base + 3);
            indices.push(base + 2);
        }

        // C. Heavy Vertical Ground Support Piers spaced every ~35m
        let mut dist_since_pier = 20.0f32;
        for i in 1..n {
            let seg_len = (centerlines[i] - centerlines[i - 1]).length();
            dist_since_pier += seg_len;

            if dist_since_pier >= 35.0 {
                dist_since_pier = 0.0;

                if i >= start_node && i <= end_node {
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
    }

    // =========================================================================
    // 3. Urban Street Furniture: 3D Streetlight Poles along Major Arterials
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

                if i >= start_node && i <= end_node {
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

                // Urban Street Trees: Placed between streetlights along sidewalks
                if (18.0..=22.0).contains(&dist_since_light) && (start_node..=end_node).contains(&i)
                {
                    let tree_base_pt = right_edge_pts[i] + (miter_perps[i].0 * -1.8f32);
                    let trunk_h = 2.6f32;
                    let trunk_r = 0.15f32;
                    let canopy_bot = tree_base_pt.y + trunk_h;
                    let canopy_top = canopy_bot + 3.2f32;
                    let canopy_r = 1.4f32;

                    let trunk_color = [0.38, 0.26, 0.18, 0.40]; // Wood bark (alpha 0.40)
                    let foliage_color = [0.20, 0.52, 0.24, 0.50]; // Leaf canopy (alpha 0.50)

                    // 4-sided Tree Trunk
                    let t_corners = [
                        tree_base_pt + Vec3::new(-trunk_r, 0.0, -trunk_r),
                        tree_base_pt + Vec3::new(trunk_r, 0.0, -trunk_r),
                        tree_base_pt + Vec3::new(trunk_r, 0.0, trunk_r),
                        tree_base_pt + Vec3::new(-trunk_r, 0.0, trunk_r),
                    ];
                    let t_norms = [
                        [0.0, 0.0, -1.0],
                        [1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0],
                        [-1.0, 0.0, 0.0],
                    ];
                    for f in 0..4 {
                        let next = (f + 1) % 4;
                        let c0 = t_corners[f];
                        let c1 = t_corners[next];
                        let n = t_norms[f];

                        let b_idx = vertices.len() as u32;
                        vertices.push(Vertex::new([c0.x, c0.y, c0.z], n, [0.0, 0.0], trunk_color));
                        vertices.push(Vertex::new([c1.x, c1.y, c1.z], n, [1.0, 0.0], trunk_color));
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

                    // 4-sided Foliage Canopy Pyramid
                    let f_corners = [
                        tree_base_pt + Vec3::new(-canopy_r, 0.0, -canopy_r),
                        tree_base_pt + Vec3::new(canopy_r, 0.0, -canopy_r),
                        tree_base_pt + Vec3::new(canopy_r, 0.0, canopy_r),
                        tree_base_pt + Vec3::new(-canopy_r, 0.0, canopy_r),
                    ];
                    let apex = Vec3::new(tree_base_pt.x, canopy_top, tree_base_pt.z);
                    for f in 0..4 {
                        let next = (f + 1) % 4;
                        let c0 = f_corners[f];
                        let c1 = f_corners[next];
                        let n = t_norms[f];

                        let b_idx = vertices.len() as u32;
                        vertices.push(Vertex::new(
                            [c0.x, canopy_bot, c0.z],
                            n,
                            [0.0, 0.0],
                            foliage_color,
                        ));
                        vertices.push(Vertex::new(
                            [c1.x, canopy_bot, c1.z],
                            n,
                            [1.0, 0.0],
                            foliage_color,
                        ));
                        vertices.push(Vertex::new(
                            [apex.x, apex.y, apex.z],
                            n,
                            [0.5, 1.0],
                            foliage_color,
                        ));

                        indices.push(b_idx);
                        indices.push(b_idx + 2);
                        indices.push(b_idx + 1);
                    }
                }
            }
        }

        // 4. Urban Traffic Signals & Street Signage at Authenticated Intersections
        if let Some(j_coords) = options.junction_coords {
            for i in start_node..=end_node {
                let key = (
                    (road.geometry[i].lat * 1e7).round() as i64,
                    (road.geometry[i].lon * 1e7).round() as i64,
                    road.layer,
                );
                if j_coords.contains(&key) && is_arterial {
                    // Place 3D Traffic Signal Cantilever Mast on right sidewalk corner
                    let sig_base = right_edge_pts[i] + (miter_perps[i].0 * -1.8f32);
                    let mast_h = 5.8f32;
                    let mast_top = sig_base + Vec3::new(0.0, mast_h, 0.0);
                    let arm_len = 4.2f32;
                    let arm_end = mast_top + (miter_perps[i].0 * arm_len); // Reach over carriageway

                    let sig_base_idx = vertices.len() as u32;
                    let norm = [miter_perps[i].0.x, 0.0, miter_perps[i].0.z];
                    let w = 0.14f32;

                    // Vertical Signal Mast
                    vertices.push(Vertex::new(
                        [sig_base.x - w, sig_base.y, sig_base.z],
                        norm,
                        [0.0, 0.0],
                        steel_pole_color,
                    ));
                    vertices.push(Vertex::new(
                        [sig_base.x + w, sig_base.y, sig_base.z],
                        norm,
                        [1.0, 0.0],
                        steel_pole_color,
                    ));
                    vertices.push(Vertex::new(
                        [mast_top.x + w, mast_top.y, mast_top.z],
                        norm,
                        [1.0, 1.0],
                        steel_pole_color,
                    ));
                    vertices.push(Vertex::new(
                        [mast_top.x - w, mast_top.y, mast_top.z],
                        norm,
                        [0.0, 1.0],
                        steel_pole_color,
                    ));
                    indices.extend_from_slice(&[
                        sig_base_idx,
                        sig_base_idx + 3,
                        sig_base_idx + 2,
                        sig_base_idx,
                        sig_base_idx + 2,
                        sig_base_idx + 1,
                    ]);

                    // Horizontal Cantilever Arm
                    let arm_base_idx = vertices.len() as u32;
                    let down_n = [0.0, -1.0, 0.0];
                    vertices.push(Vertex::new(
                        [mast_top.x, mast_top.y, mast_top.z],
                        down_n,
                        [0.0, 0.0],
                        steel_pole_color,
                    ));
                    vertices.push(Vertex::new(
                        [arm_end.x, arm_end.y, arm_end.z],
                        down_n,
                        [1.0, 0.0],
                        steel_pole_color,
                    ));
                    vertices.push(Vertex::new(
                        [arm_end.x, arm_end.y - 0.18, arm_end.z],
                        down_n,
                        [1.0, 1.0],
                        steel_pole_color,
                    ));
                    vertices.push(Vertex::new(
                        [mast_top.x, mast_top.y - 0.18, mast_top.z],
                        down_n,
                        [0.0, 1.0],
                        steel_pole_color,
                    ));
                    indices.extend_from_slice(&[
                        arm_base_idx,
                        arm_base_idx + 3,
                        arm_base_idx + 2,
                        arm_base_idx,
                        arm_base_idx + 2,
                        arm_base_idx + 1,
                    ]);

                    // 3-Aspect Traffic Signal Head (Red, Yellow, Green lamps)
                    let head_pos = arm_end - (miter_perps[i].0 * 0.8);
                    let head_w = 0.35f32;
                    let red_light = [0.92, 0.15, 0.15, 0.70];
                    let yellow_light = [0.92, 0.78, 0.15, 0.70];
                    let green_light = [0.15, 0.85, 0.25, 0.70];

                    let lamp_base = vertices.len() as u32;
                    // Red Aspect
                    vertices.push(Vertex::new(
                        [head_pos.x - head_w, head_pos.y, head_pos.z],
                        norm,
                        [0.0, 0.0],
                        red_light,
                    ));
                    vertices.push(Vertex::new(
                        [head_pos.x + head_w, head_pos.y, head_pos.z],
                        norm,
                        [1.0, 0.0],
                        red_light,
                    ));
                    vertices.push(Vertex::new(
                        [head_pos.x + head_w, head_pos.y - 0.28, head_pos.z],
                        norm,
                        [1.0, 1.0],
                        red_light,
                    ));
                    vertices.push(Vertex::new(
                        [head_pos.x - head_w, head_pos.y - 0.28, head_pos.z],
                        norm,
                        [0.0, 1.0],
                        red_light,
                    ));
                    // Yellow Aspect
                    vertices.push(Vertex::new(
                        [head_pos.x - head_w, head_pos.y - 0.32, head_pos.z],
                        norm,
                        [0.0, 0.0],
                        yellow_light,
                    ));
                    vertices.push(Vertex::new(
                        [head_pos.x + head_w, head_pos.y - 0.32, head_pos.z],
                        norm,
                        [1.0, 0.0],
                        yellow_light,
                    ));
                    vertices.push(Vertex::new(
                        [head_pos.x + head_w, head_pos.y - 0.60, head_pos.z],
                        norm,
                        [1.0, 1.0],
                        yellow_light,
                    ));
                    vertices.push(Vertex::new(
                        [head_pos.x - head_w, head_pos.y - 0.60, head_pos.z],
                        norm,
                        [0.0, 1.0],
                        yellow_light,
                    ));
                    // Green Aspect
                    vertices.push(Vertex::new(
                        [head_pos.x - head_w, head_pos.y - 0.64, head_pos.z],
                        norm,
                        [0.0, 0.0],
                        green_light,
                    ));
                    vertices.push(Vertex::new(
                        [head_pos.x + head_w, head_pos.y - 0.64, head_pos.z],
                        norm,
                        [1.0, 0.0],
                        green_light,
                    ));
                    vertices.push(Vertex::new(
                        [head_pos.x + head_w, head_pos.y - 0.92, head_pos.z],
                        norm,
                        [1.0, 1.0],
                        green_light,
                    ));
                    vertices.push(Vertex::new(
                        [head_pos.x - head_w, head_pos.y - 0.92, head_pos.z],
                        norm,
                        [0.0, 1.0],
                        green_light,
                    ));

                    indices.extend_from_slice(&[
                        lamp_base,
                        lamp_base + 3,
                        lamp_base + 2,
                        lamp_base,
                        lamp_base + 2,
                        lamp_base + 1,
                        lamp_base + 4,
                        lamp_base + 7,
                        lamp_base + 6,
                        lamp_base + 4,
                        lamp_base + 6,
                        lamp_base + 5,
                        lamp_base + 8,
                        lamp_base + 11,
                        lamp_base + 10,
                        lamp_base + 8,
                        lamp_base + 10,
                        lamp_base + 9,
                    ]);

                    // 5. Authentic San Francisco Low-Pressure Fire Hydrant near curb
                    let hyd_pos = right_edge_pts[i] + (miter_perps[i].0 * -0.65f32);
                    let hyd_h = 0.75f32;
                    let hyd_r = 0.16f32;
                    let hyd_color = [0.95, 0.78, 0.12, 0.70]; // Safety yellow barrel
                    let cap_color = [0.92, 0.92, 0.94, 0.70]; // White top bonnet

                    let _hyd_base_idx = vertices.len() as u32;
                    let h_corners = [
                        hyd_pos + Vec3::new(-hyd_r, 0.0, -hyd_r),
                        hyd_pos + Vec3::new(hyd_r, 0.0, -hyd_r),
                        hyd_pos + Vec3::new(hyd_r, 0.0, hyd_r),
                        hyd_pos + Vec3::new(-hyd_r, 0.0, hyd_r),
                    ];
                    for f in 0..4 {
                        let next = (f + 1) % 4;
                        let c0 = h_corners[f];
                        let c1 = h_corners[next];
                        let b_idx = vertices.len() as u32;
                        vertices.push(Vertex::new([c0.x, c0.y, c0.z], norm, [0.0, 0.0], hyd_color));
                        vertices.push(Vertex::new([c1.x, c1.y, c1.z], norm, [1.0, 0.0], hyd_color));
                        vertices.push(Vertex::new(
                            [c1.x, c1.y + hyd_h, c1.z],
                            norm,
                            [1.0, 1.0],
                            hyd_color,
                        ));
                        vertices.push(Vertex::new(
                            [c0.x, c0.y + hyd_h, c0.z],
                            norm,
                            [0.0, 1.0],
                            hyd_color,
                        ));
                        indices.extend_from_slice(&[
                            b_idx,
                            b_idx + 3,
                            b_idx + 2,
                            b_idx,
                            b_idx + 2,
                            b_idx + 1,
                        ]);
                    }
                    // Cap (+Y)
                    let cap_idx = vertices.len() as u32;
                    vertices.push(Vertex::new(
                        [h_corners[0].x, hyd_pos.y + hyd_h, h_corners[0].z],
                        [0.0, 1.0, 0.0],
                        [0.0, 0.0],
                        cap_color,
                    ));
                    vertices.push(Vertex::new(
                        [h_corners[1].x, hyd_pos.y + hyd_h, h_corners[1].z],
                        [0.0, 1.0, 0.0],
                        [1.0, 0.0],
                        cap_color,
                    ));
                    vertices.push(Vertex::new(
                        [h_corners[2].x, hyd_pos.y + hyd_h, h_corners[2].z],
                        [0.0, 1.0, 0.0],
                        [1.0, 1.0],
                        cap_color,
                    ));
                    vertices.push(Vertex::new(
                        [h_corners[3].x, hyd_pos.y + hyd_h, h_corners[3].z],
                        [0.0, 1.0, 0.0],
                        [0.0, 1.0],
                        cap_color,
                    ));
                    indices.extend_from_slice(&[
                        cap_idx,
                        cap_idx + 3,
                        cap_idx + 2,
                        cap_idx,
                        cap_idx + 2,
                        cap_idx + 1,
                    ]);

                    // 6. 3D Street Sign / Stop Sign on opposite sidewalk corner
                    let sign_pos = left_edge_pts[i] + (miter_perps[i].0 * 1.6f32);
                    let sign_post_h = 2.8f32;
                    let sign_base_idx = vertices.len() as u32;
                    let sign_color = [0.85, 0.15, 0.15, 0.70]; // Octagonal red stop sign
                    let post_color = [0.65, 0.65, 0.68, 0.70];

                    // Sign post
                    let sw = 0.06f32;
                    vertices.push(Vertex::new(
                        [sign_pos.x - sw, sign_pos.y, sign_pos.z],
                        norm,
                        [0.0, 0.0],
                        post_color,
                    ));
                    vertices.push(Vertex::new(
                        [sign_pos.x + sw, sign_pos.y, sign_pos.z],
                        norm,
                        [1.0, 0.0],
                        post_color,
                    ));
                    vertices.push(Vertex::new(
                        [sign_pos.x + sw, sign_pos.y + sign_post_h, sign_pos.z],
                        norm,
                        [1.0, 1.0],
                        post_color,
                    ));
                    vertices.push(Vertex::new(
                        [sign_pos.x - sw, sign_pos.y + sign_post_h, sign_pos.z],
                        norm,
                        [0.0, 1.0],
                        post_color,
                    ));
                    indices.extend_from_slice(&[
                        sign_base_idx,
                        sign_base_idx + 3,
                        sign_base_idx + 2,
                        sign_base_idx,
                        sign_base_idx + 2,
                        sign_base_idx + 1,
                    ]);

                    // Sign face
                    let sf_idx = vertices.len() as u32;
                    let sf_w = 0.38f32;
                    let sf_y = sign_pos.y + sign_post_h - 0.2;
                    vertices.push(Vertex::new(
                        [sign_pos.x - sf_w, sf_y - sf_w, sign_pos.z],
                        norm,
                        [0.0, 0.0],
                        sign_color,
                    ));
                    vertices.push(Vertex::new(
                        [sign_pos.x + sf_w, sf_y - sf_w, sign_pos.z],
                        norm,
                        [1.0, 0.0],
                        sign_color,
                    ));
                    vertices.push(Vertex::new(
                        [sign_pos.x + sf_w, sf_y + sf_w, sign_pos.z],
                        norm,
                        [1.0, 1.0],
                        sign_color,
                    ));
                    vertices.push(Vertex::new(
                        [sign_pos.x - sf_w, sf_y + sf_w, sign_pos.z],
                        norm,
                        [0.0, 1.0],
                        sign_color,
                    ));
                    indices.extend_from_slice(&[
                        sf_idx,
                        sf_idx + 3,
                        sf_idx + 2,
                        sf_idx,
                        sf_idx + 2,
                        sf_idx + 1,
                    ]);
                }
            }
        }
    }

    ChunkMesh::new(vertices, indices)
}
