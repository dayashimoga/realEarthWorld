//! Directed lane-level road graph representation using Petgraph.

use petgraph::graph::{DiGraph, NodeIndex};
use realearth_ingestion::osm_types::{HighwayType, OsmRoadFeature};
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node in the road graph representing an intersection or geometric inflection point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RoadNode {
    pub id: u64,
    pub coord: Wgs84Coord,
}

/// Directed edge representing a drivable road segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadEdge {
    pub way_id: u64,
    pub highway_type: HighwayType,
    pub length_meters: f64,
    pub speed_limit_kph: u32,
    pub lanes: u32,
    pub layer: i8,
    pub is_bridge: bool,
    pub is_tunnel: bool,
}

impl RoadEdge {
    /// Travel time in seconds across this segment.
    pub fn traversal_time_seconds(&self) -> f64 {
        let speed_ms = (self.speed_limit_kph as f64) * (1000.0 / 3600.0);
        self.length_meters / speed_ms.max(1.0)
    }
}

/// Directed road network graph for spatial pathfinding.
#[derive(Debug, Clone, Default)]
pub struct RoadNetworkGraph {
    pub graph: DiGraph<RoadNode, RoadEdge>,
    coord_to_node: HashMap<(u64, u64), NodeIndex>,
}

impl RoadNetworkGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            coord_to_node: HashMap::new(),
        }
    }

    /// Builds a graph from a slice of validated road features.
    pub fn from_roads(roads: &[OsmRoadFeature]) -> Self {
        let mut network = Self::new();
        for road in roads {
            network.add_road(road);
        }
        network
    }

    /// Adds a single road feature into the directed graph.
    pub fn add_road(&mut self, road: &OsmRoadFeature) {
        if road.geometry.len() < 2 {
            return;
        }

        for i in 0..(road.geometry.len() - 1) {
            let p0 = road.geometry[i];
            let p1 = road.geometry[i + 1];

            let n0 = self.get_or_create_node(p0);
            let n1 = self.get_or_create_node(p1);

            // Approximate metric distance in meters
            let d_lat = (p1.lat - p0.lat) * 111_000.0;
            let d_lon = (p1.lon - p0.lon) * 108_000.0;
            let length_meters = (d_lat * d_lat + d_lon * d_lon).sqrt().max(0.1);

            let edge = RoadEdge {
                way_id: road.way_id,
                highway_type: road.highway_type,
                length_meters,
                speed_limit_kph: road.speed_limit_kph,
                lanes: road.lanes,
                layer: road.layer,
                is_bridge: road.is_bridge,
                is_tunnel: road.is_tunnel,
            };

            // Forward direction
            self.graph.add_edge(n0, n1, edge.clone());

            // Reverse direction if not a one-way street
            if !road.is_oneway {
                self.graph.add_edge(n1, n0, edge);
            }
        }
    }

    /// Gets existing node or inserts a new one for a coordinate.
    fn get_or_create_node(&mut self, coord: Wgs84Coord) -> NodeIndex {
        let key = ((coord.lat * 1e7) as u64, (coord.lon * 1e7) as u64);
        if let Some(&node_idx) = self.coord_to_node.get(&key) {
            node_idx
        } else {
            let id = self.graph.node_count() as u64;
            let node_idx = self.graph.add_node(RoadNode { id, coord });
            self.coord_to_node.insert(key, node_idx);
            node_idx
        }
    }

    /// Finds the nearest graph node to an arbitrary WGS84 coordinate.
    pub fn find_nearest_node(&self, coord: Wgs84Coord) -> Option<NodeIndex> {
        let mut best_node = None;
        let mut min_dist_sq = f64::MAX;

        for node_idx in self.graph.node_indices() {
            let node = &self.graph[node_idx];
            let d_lat = (node.coord.lat - coord.lat) * 111_000.0;
            let d_lon = (node.coord.lon - coord.lon) * 108_000.0;
            let dist_sq = d_lat * d_lat + d_lon * d_lon;

            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_node = Some(node_idx);
            }
        }

        best_node
    }
}
