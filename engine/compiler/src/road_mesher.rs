//! 3D Road ribbon generation with grade-separated bridge elevation profiles.

use crate::mesh::{ChunkMesh, Vertex};
use glam::Vec3;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::OsmRoadFeature;

/// Generates a 3D road ribbon mesh conforming to terrain or elevated bridges.
pub fn mesh_road(road: &OsmRoadFeature, converter: &EnuConverter, dem: &DemGrid) -> ChunkMesh {
    if road.geometry.len() < 2 {
        return ChunkMesh::default();
    }

    let lane_width = road.highway_type.default_lane_width();
    let total_width = (road.lanes.max(1) as f32) * lane_width;
    let half_width = total_width * 0.5;

    // Convert centerline to 3D local ENU positions
    let mut centerlines: Vec<Vec3> = Vec::with_capacity(road.geometry.len());
    for pt in &road.geometry {
        let enu = converter.wgs84_to_enu(*pt);
        let ground_elev = dem.sample_bilinear(pt);

        // Grade separation clearance:
        // Overpasses (layer >= 1 or is_bridge) elevate 6.0m per layer
        // Underpasses (layer <= -1 or is_tunnel) depress 5.0m per layer
        let final_elev = if road.layer > 0 || road.is_bridge {
            let layers = road.layer.max(1) as f32;
            ground_elev + layers * 6.0
        } else if road.layer < 0 || road.is_tunnel {
            let layers = road.layer.abs().max(1) as f32;
            ground_elev - layers * 5.0
        } else {
            // Slight offset above ground to avoid z-fighting with terrain
            ground_elev + 0.15
        };

        centerlines.push(Vec3::new(enu.east as f32, final_elev, enu.north as f32));
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Road surface color (asphalt dark grey, bridges slightly lighter concrete)
    let road_color = if road.is_bridge || road.layer > 0 {
        [0.32, 0.32, 0.35, 1.0]
    } else {
        [0.22, 0.22, 0.24, 1.0]
    };

    let up_norm = [0.0, 1.0, 0.0];

    // Compute segment unit forward and perpendicular vectors
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

    // Compute continuous miter perpendiculars at each centerline vertex
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

    // Generate shared left and right vertices along continuous centerline
    let mut cumulative_dist = 0.0f32;
    for i in 0..n {
        if i > 0 {
            let p_prev = centerlines[i - 1];
            let p_curr = centerlines[i];
            cumulative_dist += (p_curr - p_prev).length();
        }

        let pt = centerlines[i];
        let (miter, factor) = miter_perps[i];
        let offset = miter * (half_width * factor);

        let left = pt + offset;
        let right = pt - offset;

        let v = cumulative_dist * 0.1; // texture coordinate v
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

    // Triangulate continuous ribbon quads
    for i in 0..(n - 1) {
        let base = (i * 2) as u32;
        // CCW triangles: (left0, right0, right1) and (left0, right1, left1)
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 3);

        indices.push(base);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    ChunkMesh::new(vertices, indices)
}
