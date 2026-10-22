//! A* Pathfinding and route calculation.

use crate::road_graph::RoadNetworkGraph;
use petgraph::algo::astar;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::{RealEarthError, Result};
use serde::{Deserialize, Serialize};

/// Navigation route metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingPreference {
    ShortestDistance,
    FastestTime,
}

/// A computed driving route with waypoints and metric estimates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavigationRoute {
    pub waypoints: Vec<Wgs84Coord>,
    pub total_distance_meters: f64,
    pub estimated_duration_seconds: f64,
}

/// Computes an optimal navigation path between two coordinates using A* search.
pub fn find_route(
    network: &RoadNetworkGraph,
    start_coord: Wgs84Coord,
    target_coord: Wgs84Coord,
    preference: RoutingPreference,
) -> Result<NavigationRoute> {
    let start_node = network.find_nearest_node(start_coord).ok_or_else(|| {
        RealEarthError::Geospatial("No nearby road node found for start coordinate".to_string())
    })?;

    let target_node = network.find_nearest_node(target_coord).ok_or_else(|| {
        RealEarthError::Geospatial("No nearby road node found for target coordinate".to_string())
    })?;

    if start_node == target_node {
        return Ok(NavigationRoute {
            waypoints: vec![start_coord, target_coord],
            total_distance_meters: 0.0,
            estimated_duration_seconds: 0.0,
        });
    }

    let target_pos = network.graph[target_node].coord;

    // A* search using petgraph
    let path = astar(
        &network.graph,
        start_node,
        |finish| finish == target_node,
        |e| match preference {
            RoutingPreference::ShortestDistance => e.weight().length_meters,
            RoutingPreference::FastestTime => e.weight().traversal_time_seconds(),
        },
        |node_idx| {
            // Strictly admissible heuristic: Euclidean metric distance in meters
            let node = &network.graph[node_idx];
            let mean_lat = (node.coord.lat + target_pos.lat) * 0.5;
            let m_per_deg_lon = 111_320.0 * (mean_lat.to_radians()).cos().abs().max(0.1);
            let d_lat = (node.coord.lat - target_pos.lat) * 111_000.0;
            let d_lon = (node.coord.lon - target_pos.lon) * m_per_deg_lon;
            let dist = (d_lat * d_lat + d_lon * d_lon).sqrt();

            match preference {
                RoutingPreference::ShortestDistance => dist,
                // Upper bound highway velocity (140 km/h = 38.89 m/s) guarantees h(n) <= h*(n) globally
                RoutingPreference::FastestTime => dist / (140.0 * 1000.0 / 3600.0),
            }
        },
    );

    match path {
        Some((_, node_indices)) => {
            let mut waypoints = Vec::with_capacity(node_indices.len());
            let mut total_distance = 0.0;
            let mut total_duration = 0.0;

            for i in 0..node_indices.len() {
                let idx = node_indices[i];
                waypoints.push(network.graph[idx].coord);

                if i > 0 {
                    let prev_idx = node_indices[i - 1];
                    if let Some(edge) = network.graph.find_edge(prev_idx, idx) {
                        let w = &network.graph[edge];
                        total_distance += w.length_meters;
                        total_duration += w.traversal_time_seconds();
                    }
                }
            }

            Ok(NavigationRoute {
                waypoints,
                total_distance_meters: total_distance,
                estimated_duration_seconds: total_duration,
            })
        }
        None => Err(RealEarthError::Geospatial(
            "No navigable road route found between coordinates".to_string(),
        )),
    }
}
