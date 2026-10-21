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

    // Compute left and right ribbon edges
    for i in 0..(centerlines.len() - 1) {
        let p0 = centerlines[i];
        let p1 = centerlines[i + 1];

        let forward = Vec3::new(p1.x - p0.x, 0.0, p1.z - p0.z);
        let len = forward.length();
        if len < 1e-4 {
            continue;
        }

        // Perpendicular vector in X-Z plane: (-forward.z, 0, forward.x) / len
        let perp = Vec3::new(-forward.z, 0.0, forward.x) / len;

        let left0 = p0 + perp * half_width;
        let right0 = p0 - perp * half_width;
        let left1 = p1 + perp * half_width;
        let right1 = p1 - perp * half_width;

        let base_idx = vertices.len() as u32;

        let v0 = Vertex::new([left0.x, left0.y, left0.z], up_norm, [0.0, 0.0], road_color);
        let v1 = Vertex::new(
            [right0.x, right0.y, right0.z],
            up_norm,
            [1.0, 0.0],
            road_color,
        );
        let v2 = Vertex::new(
            [right1.x, right1.y, right1.z],
            up_norm,
            [1.0, 1.0],
            road_color,
        );
        let v3 = Vertex::new([left1.x, left1.y, left1.z], up_norm, [0.0, 1.0], road_color);

        vertices.push(v0);
        vertices.push(v1);
        vertices.push(v2);
        vertices.push(v3);

        // Quad triangles
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);

        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    ChunkMesh::new(vertices, indices)
}
