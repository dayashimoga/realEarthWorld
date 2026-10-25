//! 3D Railway corridor mesh generation with ballast beds, cross ties, continuous steel rails,
//! elevated viaduct piers, tunnel portals, and overhead catenary infrastructure.

use crate::mesh::{ChunkMesh, Vertex};
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::OsmRailwayFeature;

/// Detailed options for railway ribbon and infrastructure mesh generation.
#[derive(Debug, Clone, Default)]
pub struct RailwayMeshingOptions {
    /// Contiguous segment index range [start_seg, end_seg] to mesh (inclusive, 0 <= start_seg <= end_seg < n - 1).
    /// If None, all segments of the railway feature are meshed.
    pub segment_range: Option<(usize, usize)>,
}

/// Generates a comprehensive 3D railway corridor mesh with default whole-track options.
pub fn mesh_railway(
    railway: &OsmRailwayFeature,
    converter: &EnuConverter,
    dem: &DemGrid,
) -> ChunkMesh {
    mesh_railway_with_options(railway, converter, dem, &RailwayMeshingOptions::default())
}

/// Generates a comprehensive 3D railway corridor mesh (ballast bed, cross ties, dual steel rails,
/// elevated viaduct box girders, ground piers, and catenary masts) with options for chunk segment ranges.
pub fn mesh_railway_with_options(
    railway: &OsmRailwayFeature,
    converter: &EnuConverter,
    dem: &DemGrid,
    options: &RailwayMeshingOptions,
) -> ChunkMesh {
    if railway.geometry.len() < 2 {
        return ChunkMesh::default();
    }

    let gauge_m = railway
        .gauge_mm
        .map(|g| (g as f32) / 1000.0)
        .unwrap_or(1.435)
        .clamp(0.75, 2.14);
    let half_gauge = gauge_m * 0.5;

    let is_elevated = railway.layer > 0 || railway.is_bridge;
    let is_subterranean = railway.layer < 0 || railway.is_tunnel;

    // Convert centerline to 3D local ENU positions
    let mut centerlines: Vec<Vec3> = Vec::with_capacity(railway.geometry.len());
    let mut ground_elevs: Vec<f32> = Vec::with_capacity(railway.geometry.len());

    for pt in &railway.geometry {
        let enu = converter.wgs84_to_enu(*pt);
        let ground_elev = dem.sample_bilinear(pt);
        ground_elevs.push(ground_elev);

        let final_elev = if is_elevated {
            let layers = railway.layer.max(1) as f32;
            ground_elev + layers * 6.0
        } else if is_subterranean {
            let layers = railway.layer.abs().max(1) as f32;
            ground_elev - layers * 5.0
        } else {
            // Offset slightly above ground to prevent z-fighting with terrain
            ground_elev + 0.18
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

    // Material palette
    let ballast_color = [0.34, 0.33, 0.32, 0.94]; // Crushed granite ballast
    let sleeper_color = [0.22, 0.20, 0.18, 1.0]; // Creosote dark timber / concrete tie
    let rail_steel_color = [0.72, 0.74, 0.78, 1.0]; // Polished crown steel
    let rail_web_color = [0.45, 0.40, 0.35, 1.0]; // Weathered rail web
    let viaduct_color = [0.55, 0.56, 0.58, 0.88]; // Reinforced concrete viaduct
    let steel_mast_color = [0.38, 0.40, 0.44, 0.75]; // Catenary steel mast

    let up_norm = [0.0, 1.0, 0.0];

    // =========================================================================
    // 1. Crushed Rock Ballast Shoulder Bed (Trapezoidal prism)
    // =========================================================================
    let ballast_top_half_w = (gauge_m + 1.2).max(1.6);
    let ballast_bottom_half_w = ballast_top_half_w + 0.6;
    let ballast_depth = 0.32f32;

    let ballast_start_idx = vertices.len() as u32;
    for i in start_node..=end_node {
        let center = centerlines[i];
        let (miter, factor) = miter_perps[i];
        let top_left = center + miter * (ballast_top_half_w * factor);
        let top_right = center - miter * (ballast_top_half_w * factor);
        let bot_left = center + miter * (ballast_bottom_half_w * factor) - Vec3::Y * ballast_depth;
        let bot_right = center - miter * (ballast_bottom_half_w * factor) - Vec3::Y * ballast_depth;

        let cum_dist = node_cum_dists[i];

        // 4 vertices per profile slice: bot_left, top_left, top_right, bot_right
        vertices.push(Vertex::new(
            [bot_left.x, bot_left.y, bot_left.z],
            [miter.x, 0.5, miter.z],
            [0.0, cum_dist * 0.5],
            ballast_color,
        ));
        vertices.push(Vertex::new(
            [top_left.x, top_left.y, top_left.z],
            up_norm,
            [0.2, cum_dist * 0.5],
            ballast_color,
        ));
        vertices.push(Vertex::new(
            [top_right.x, top_right.y, top_right.z],
            up_norm,
            [0.8, cum_dist * 0.5],
            ballast_color,
        ));
        vertices.push(Vertex::new(
            [bot_right.x, bot_right.y, bot_right.z],
            [-miter.x, 0.5, -miter.z],
            [1.0, cum_dist * 0.5],
            ballast_color,
        ));
    }

    for seg in start_seg..=end_seg {
        let seg_local = (seg - start_node) as u32;
        let b0 = ballast_start_idx + seg_local * 4;
        let b1 = b0 + 4;

        // Left slope: (b0+0, b1+0, b1+1) and (b0+0, b1+1, b0+1)
        indices.push(b0);
        indices.push(b1);
        indices.push(b1 + 1);

        indices.push(b0);
        indices.push(b1 + 1);
        indices.push(b0 + 1);

        // Top surface: (b0+1, b1+1, b1+2) and (b0+1, b1+2, b0+2)
        indices.push(b0 + 1);
        indices.push(b1 + 1);
        indices.push(b1 + 2);

        indices.push(b0 + 1);
        indices.push(b1 + 2);
        indices.push(b0 + 2);

        // Right slope: (b0+2, b1+2, b1+3) and (b0+2, b1+3, b0+3)
        indices.push(b0 + 2);
        indices.push(b1 + 2);
        indices.push(b1 + 3);

        indices.push(b0 + 2);
        indices.push(b1 + 3);
        indices.push(b0 + 3);
    }

    // =========================================================================
    // 2. Concrete / Timber Cross Ties (Sleepers spaced every ~0.65m)
    // =========================================================================
    let tie_spacing = 0.65f32;
    let tie_width = (gauge_m + 1.0).max(2.4); // 2.44m standard US tie
    let tie_half_w = tie_width * 0.5;
    let tie_length = 0.24f32; // dimension along track
    let tie_half_l = tie_length * 0.5;
    let tie_height = 0.16f32;

    for seg in start_seg..=end_seg {
        let p0 = centerlines[seg];
        let p1 = centerlines[seg + 1];
        let seg_vec = p1 - p0;
        let seg_len = seg_vec.length();
        if seg_len < 0.1 {
            continue;
        }

        let forward = seg_vec / seg_len;
        let (miter0, _) = miter_perps[seg];
        let (miter1, _) = miter_perps[seg + 1];

        let num_ties = (seg_len / tie_spacing).floor() as usize;
        for t in 0..num_ties {
            let frac = ((t as f32) + 0.5) * tie_spacing / seg_len;
            if frac >= 1.0 {
                continue;
            }

            let tie_center = p0 + seg_vec * frac;
            let miter = miter0.lerp(miter1, frac).normalize();

            // Tie top corners
            let c0 = tie_center + miter * tie_half_w - forward * tie_half_l + Vec3::Y * tie_height;
            let c1 = tie_center - miter * tie_half_w - forward * tie_half_l + Vec3::Y * tie_height;
            let c2 = tie_center - miter * tie_half_w + forward * tie_half_l + Vec3::Y * tie_height;
            let c3 = tie_center + miter * tie_half_w + forward * tie_half_l + Vec3::Y * tie_height;

            let base_idx = vertices.len() as u32;
            vertices.push(Vertex::new(
                [c0.x, c0.y, c0.z],
                up_norm,
                [0.0, 0.0],
                sleeper_color,
            ));
            vertices.push(Vertex::new(
                [c1.x, c1.y, c1.z],
                up_norm,
                [1.0, 0.0],
                sleeper_color,
            ));
            vertices.push(Vertex::new(
                [c2.x, c2.y, c2.z],
                up_norm,
                [1.0, 1.0],
                sleeper_color,
            ));
            vertices.push(Vertex::new(
                [c3.x, c3.y, c3.z],
                up_norm,
                [0.0, 1.0],
                sleeper_color,
            ));

            indices.push(base_idx);
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);

            indices.push(base_idx);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        }
    }

    // =========================================================================
    // 3. Dual Continuous Steel Rails (Head, Web, and Base Flange)
    // =========================================================================
    let rail_crown_w = 0.072f32; // 72mm rail head crown
    let half_crown = rail_crown_w * 0.5;
    let rail_crown_h = tie_height + 0.17f32; // 170mm (136 RE rail) above tie
    let rail_base_h = tie_height;

    for side in &[-1.0f32, 1.0f32] {
        let rail_offset = half_gauge * (*side);
        let rail_start_idx = vertices.len() as u32;

        for i in start_node..=end_node {
            let center = centerlines[i];
            let (miter, factor) = miter_perps[i];
            let rail_center = center + miter * (rail_offset * factor);
            let cum_dist = node_cum_dists[i];

            let left_head = rail_center + miter * half_crown + Vec3::Y * rail_crown_h;
            let right_head = rail_center - miter * half_crown + Vec3::Y * rail_crown_h;
            let left_web = rail_center + miter * half_crown * 0.4 + Vec3::Y * rail_base_h;
            let right_web = rail_center - miter * half_crown * 0.4 + Vec3::Y * rail_base_h;

            // 4 vertices per slice
            vertices.push(Vertex::new(
                [left_head.x, left_head.y, left_head.z],
                up_norm,
                [0.0, cum_dist * 2.0],
                rail_steel_color,
            ));
            vertices.push(Vertex::new(
                [right_head.x, right_head.y, right_head.z],
                up_norm,
                [1.0, cum_dist * 2.0],
                rail_steel_color,
            ));
            vertices.push(Vertex::new(
                [left_web.x, left_web.y, left_web.z],
                [miter.x, 0.0, miter.z],
                [0.0, cum_dist * 2.0],
                rail_web_color,
            ));
            vertices.push(Vertex::new(
                [right_web.x, right_web.y, right_web.z],
                [-miter.x, 0.0, -miter.z],
                [1.0, cum_dist * 2.0],
                rail_web_color,
            ));
        }

        for seg in start_seg..=end_seg {
            let seg_local = (seg - start_node) as u32;
            let r0 = rail_start_idx + seg_local * 4;
            let r1 = r0 + 4;

            // Top crown surface
            indices.push(r0);
            indices.push(r0 + 1);
            indices.push(r1 + 1);

            indices.push(r0);
            indices.push(r1 + 1);
            indices.push(r1);

            // Left web face
            indices.push(r0);
            indices.push(r1);
            indices.push(r1 + 2);

            indices.push(r0);
            indices.push(r1 + 2);
            indices.push(r0 + 2);

            // Right web face
            indices.push(r0 + 1);
            indices.push(r1 + 3);
            indices.push(r1 + 1);

            indices.push(r0 + 1);
            indices.push(r0 + 3);
            indices.push(r1 + 3);
        }
    }

    // =========================================================================
    // 4. Elevated Viaduct Box Girders & Support Piers (When is_bridge / layer > 0)
    // =========================================================================
    if is_elevated {
        let deck_depth = 1.4f32;
        let pier_spacing = 35.0f32;
        let pier_radius = 0.85f32;

        let soffit_start_idx = vertices.len() as u32;
        let down_norm = [0.0, -1.0, 0.0];

        for i in start_node..=end_node {
            let center = centerlines[i];
            let (miter, factor) = miter_perps[i];
            let deck_half_w = ballast_bottom_half_w + 0.3;
            let l_bot =
                center + miter * (deck_half_w * factor) - Vec3::Y * (ballast_depth + deck_depth);
            let r_bot =
                center - miter * (deck_half_w * factor) - Vec3::Y * (ballast_depth + deck_depth);

            vertices.push(Vertex::new(
                [l_bot.x, l_bot.y, l_bot.z],
                down_norm,
                [0.0, 0.0],
                viaduct_color,
            ));
            vertices.push(Vertex::new(
                [r_bot.x, r_bot.y, r_bot.z],
                down_norm,
                [1.0, 0.0],
                viaduct_color,
            ));
        }

        for seg in start_seg..=end_seg {
            let seg_local = (seg - start_node) as u32;
            let s0 = soffit_start_idx + seg_local * 2;
            let s1 = s0 + 2;

            indices.push(s0);
            indices.push(s0 + 1);
            indices.push(s1 + 1);

            indices.push(s0);
            indices.push(s1 + 1);
            indices.push(s1);
        }

        // Heavy vertical piers to terrain
        let mut last_pier_dist = 0.0f32;
        for i in start_node..=end_node {
            let cum_dist = node_cum_dists[i];
            if i == start_node || cum_dist - last_pier_dist >= pier_spacing || i == end_node {
                last_pier_dist = cum_dist;
                let track_center = centerlines[i];
                let ground_y = ground_elevs[i];
                let deck_bot_y = track_center.y - (ballast_depth + deck_depth);

                if deck_bot_y > ground_y + 1.0 {
                    let pier_base = vertices.len() as u32;
                    let sides = 8;
                    for s in 0..sides {
                        let angle = (s as f32) * std::f32::consts::TAU / (sides as f32);
                        let dx = angle.cos() * pier_radius;
                        let dz = angle.sin() * pier_radius;
                        let norm = [angle.cos(), 0.0, angle.sin()];

                        vertices.push(Vertex::new(
                            [track_center.x + dx, ground_y, track_center.z + dz],
                            norm,
                            [s as f32 / sides as f32, 0.0],
                            viaduct_color,
                        ));
                        vertices.push(Vertex::new(
                            [track_center.x + dx, deck_bot_y, track_center.z + dz],
                            norm,
                            [s as f32 / sides as f32, 1.0],
                            viaduct_color,
                        ));
                    }

                    for s in 0..sides {
                        let next = (s + 1) % sides;
                        let v0 = pier_base + (s as u32) * 2;
                        let v1 = v0 + 1;
                        let v2 = pier_base + (next as u32) * 2;
                        let v3 = v2 + 1;

                        indices.push(v0);
                        indices.push(v1);
                        indices.push(v3);

                        indices.push(v0);
                        indices.push(v3);
                        indices.push(v2);
                    }
                }
            }
        }
    }

    // =========================================================================
    // 5. Overhead Catenary Electrification Masts (When is_electrified)
    // =========================================================================
    if railway.is_electrified {
        let mast_spacing = 45.0f32;
        let mast_offset = half_gauge + 1.8f32;
        let mast_height = 6.2f32;
        let mast_radius = 0.12f32;

        let mut last_mast_dist = 0.0f32;
        for i in start_node..=end_node {
            let cum_dist = node_cum_dists[i];
            if i == start_node || cum_dist - last_mast_dist >= mast_spacing {
                last_mast_dist = cum_dist;
                let center = centerlines[i];
                let (miter, factor) = miter_perps[i];
                let mast_pos = center + miter * (mast_offset * factor);

                let base_idx = vertices.len() as u32;

                // Vertical column
                let p_bot = mast_pos;
                let p_top = mast_pos + Vec3::Y * mast_height;
                let p_arm_end =
                    mast_pos + Vec3::Y * (mast_height - 0.4) - miter * (mast_offset * factor);

                // Add simplified mast column quad
                vertices.push(Vertex::new(
                    [p_bot.x - mast_radius, p_bot.y, p_bot.z],
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0],
                    steel_mast_color,
                ));
                vertices.push(Vertex::new(
                    [p_bot.x + mast_radius, p_bot.y, p_bot.z],
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0],
                    steel_mast_color,
                ));
                vertices.push(Vertex::new(
                    [p_top.x + mast_radius, p_top.y, p_top.z],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0],
                    steel_mast_color,
                ));
                vertices.push(Vertex::new(
                    [p_top.x - mast_radius, p_top.y, p_top.z],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0],
                    steel_mast_color,
                ));

                indices.push(base_idx);
                indices.push(base_idx + 1);
                indices.push(base_idx + 2);

                indices.push(base_idx);
                indices.push(base_idx + 2);
                indices.push(base_idx + 3);

                // Horizontal cantilever arm extending over the track
                let arm_idx = vertices.len() as u32;
                vertices.push(Vertex::new(
                    [p_top.x, p_top.y - 0.5, p_top.z],
                    up_norm,
                    [0.0, 0.0],
                    steel_mast_color,
                ));
                vertices.push(Vertex::new(
                    [p_arm_end.x, p_arm_end.y, p_arm_end.z],
                    up_norm,
                    [1.0, 0.0],
                    steel_mast_color,
                ));
                vertices.push(Vertex::new(
                    [p_arm_end.x, p_arm_end.y + 0.1, p_arm_end.z],
                    up_norm,
                    [1.0, 1.0],
                    steel_mast_color,
                ));
                vertices.push(Vertex::new(
                    [p_top.x, p_top.y - 0.4, p_top.z],
                    up_norm,
                    [0.0, 1.0],
                    steel_mast_color,
                ));

                indices.push(arm_idx);
                indices.push(arm_idx + 1);
                indices.push(arm_idx + 2);

                indices.push(arm_idx);
                indices.push(arm_idx + 2);
                indices.push(arm_idx + 3);
            }
        }
    }

    ChunkMesh::new(vertices, indices)
}
