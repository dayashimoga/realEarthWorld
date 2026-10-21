//! Velocity- and direction-aware predictive spatial prefetching.

use crate::config::StreamingConfig;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;

/// Priority classification for candidate chunks in the streaming pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkPriority {
    /// Highest priority: Immediate physics/collision zone directly underneath or within critical radius.
    CriticalCollision = 0,
    /// High priority: Immediately visible in camera frustum or within primary viewing circle.
    ImmediateFrustum = 1,
    /// Medium priority: Along forward velocity vector within lookahead time window.
    LookaheadCone = 2,
    /// Lower priority: Peripheral surrounding chunks maintaining rotational cache continuity.
    Peripheral = 3,
}

/// Computes an anisotropic bounding box covering both the omnidirectional view circle
/// and the forward velocity lookahead cone.
pub fn calculate_prefetch_bounds(
    camera_pos: Wgs84Coord,
    velocity_mps: glam::Vec3,
    config: &StreamingConfig,
) -> GeoBounds {
    // Baseline metric meters-to-degrees conversion
    let lat_rad = camera_pos.lat.to_radians();
    let m_per_deg_lat = 111_000.0;
    let m_per_deg_lon = 111_320.0 * lat_rad.cos().abs().max(0.1);

    // 1. Omnidirectional radius delta
    let r_lat = config.view_distance_meters / m_per_deg_lat;
    let r_lon = config.view_distance_meters / m_per_deg_lon;

    let mut min_lat = camera_pos.lat - r_lat;
    let mut max_lat = camera_pos.lat + r_lat;
    let mut min_lon = camera_pos.lon - r_lon;
    let mut max_lon = camera_pos.lon + r_lon;

    // 2. Velocity lookahead extension
    // velocity_mps: x = East (+lon), z = North (+lat)
    let vx = velocity_mps.x as f64;
    let vz = velocity_mps.z as f64;
    let speed = (vx * vx + vz * vz).sqrt();

    if speed > 0.5 {
        let lookahead_time = config.lookahead_seconds;
        let future_lat = camera_pos.lat + (vz * lookahead_time) / m_per_deg_lat;
        let future_lon = camera_pos.lon + (vx * lookahead_time) / m_per_deg_lon;

        // The future camera position needs its full viewing distance forward
        let cone_r_lat = config.view_distance_meters / m_per_deg_lat;
        let cone_r_lon = config.view_distance_meters / m_per_deg_lon;

        min_lat = min_lat.min(future_lat - cone_r_lat);
        max_lat = max_lat.max(future_lat + cone_r_lat);
        min_lon = min_lon.min(future_lon - cone_r_lon);
        max_lon = max_lon.max(future_lon + cone_r_lon);
    }

    GeoBounds::new(min_lat, max_lat, min_lon, max_lon)
}

/// Evaluates priority of a chunk based on camera position, motion vector, and distance.
pub fn evaluate_chunk_priority(
    chunk_bounds: &GeoBounds,
    camera_pos: Wgs84Coord,
    velocity_mps: glam::Vec3,
    config: &StreamingConfig,
) -> ChunkPriority {
    let center = chunk_bounds.center();
    let lat_rad = camera_pos.lat.to_radians();
    let m_per_deg_lat = 111_000.0;
    let m_per_deg_lon = 111_320.0 * lat_rad.cos().abs().max(0.1);

    let d_lat_m = (center.lat - camera_pos.lat) * m_per_deg_lat;
    let d_lon_m = (center.lon - camera_pos.lon) * m_per_deg_lon;
    let dist_m = (d_lat_m * d_lat_m + d_lon_m * d_lon_m).sqrt();

    // 1. Critical collision zone (underneath vehicle/camera)
    if dist_m <= config.collision_critical_radius_meters {
        return ChunkPriority::CriticalCollision;
    }

    // 2. Immediate viewing circle
    if dist_m <= config.lod1_distance_meters * 0.6 {
        return ChunkPriority::ImmediateFrustum;
    }

    // 3. Velocity lookahead cone check
    let vx = velocity_mps.x as f64;
    let vz = velocity_mps.z as f64;
    let speed = (vx * vx + vz * vz).sqrt();

    if speed > 1.0 {
        // Dot product between normalized velocity and direction to chunk
        let dir_x = d_lon_m / dist_m;
        let dir_z = d_lat_m / dist_m;
        let vel_dir_x = vx / speed;
        let vel_dir_z = vz / speed;
        let dot = dir_x * vel_dir_x + dir_z * vel_dir_z;

        // If chunk is in the forward half-plane (> 45 degree forward arc)
        if dot > 0.5 {
            return ChunkPriority::LookaheadCone;
        }
    }

    ChunkPriority::Peripheral
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_prefetch_bounds_expansion() {
        let config = StreamingConfig {
            view_distance_meters: 1000.0,
            lookahead_seconds: 2.0,
            ..Default::default()
        };

        let pos = Wgs84Coord::new(37.79, -122.40, 10.0);

        // 1. Stationary bounds
        let stationary_bounds = calculate_prefetch_bounds(pos, glam::Vec3::ZERO, &config);
        assert!(stationary_bounds.max_lat > pos.lat);
        assert!(stationary_bounds.min_lat < pos.lat);

        // 2. High-speed highway velocity heading North at 30 m/s (~108 km/h)
        let highway_vel = glam::Vec3::new(0.0, 0.0, 30.0);
        let moving_bounds = calculate_prefetch_bounds(pos, highway_vel, &config);

        // North bound should be significantly larger than stationary
        assert!(moving_bounds.max_lat > stationary_bounds.max_lat);
    }

    #[test]
    fn test_chunk_priority_classification() {
        let config = StreamingConfig {
            collision_critical_radius_meters: 250.0,
            lod1_distance_meters: 1500.0,
            ..Default::default()
        };

        let pos = Wgs84Coord::new(37.79, -122.40, 10.0);

        // Chunk right on top of camera
        let immediate_chunk = GeoBounds::new(37.789, 37.791, -122.401, -122.399);
        let prio_crit = evaluate_chunk_priority(&immediate_chunk, pos, glam::Vec3::ZERO, &config);
        assert_eq!(prio_crit, ChunkPriority::CriticalCollision);

        // Chunk 2km ahead North while moving North at 25 m/s
        let forward_chunk = GeoBounds::new(37.808, 37.810, -122.401, -122.399);
        let prio_forward = evaluate_chunk_priority(
            &forward_chunk,
            pos,
            glam::Vec3::new(0.0, 0.0, 25.0),
            &config,
        );
        assert_eq!(prio_forward, ChunkPriority::LookaheadCone);

        // Chunk 2km behind South while moving North
        let backward_chunk = GeoBounds::new(37.770, 37.772, -122.401, -122.399);
        let prio_behind = evaluate_chunk_priority(
            &backward_chunk,
            pos,
            glam::Vec3::new(0.0, 0.0, 25.0),
            &config,
        );
        assert_eq!(prio_behind, ChunkPriority::Peripheral);
    }
}
