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
    pub layer: i8,
}

/// Road access restriction classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RoadAccessRestriction {
    #[default]
    All,
    NoMotorVehicles,
    PedestrianOnly,
    EmergencyOnly,
    Private,
}

/// Inferred versus surveyed road attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct InferredRoadAttributes {
    pub is_lanes_inferred: bool,
    pub is_speed_limit_inferred: bool,
    pub is_name_missing: bool,
}

/// Turn restriction types between road ways at junctions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TurnRestrictionType {
    NoLeftTurn,
    NoRightTurn,
    NoUturn,
    OnlyStraightOn,
}

/// A turn restriction definition between two connected OSM ways.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnRestriction {
    pub from_way_id: u64,
    pub to_way_id: u64,
    pub restriction_type: TurnRestrictionType,
}

/// Directed edge representing a drivable road segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadEdge {
    pub way_id: u64,
    pub name: Option<String>,
    pub highway_type: HighwayType,
    pub length_meters: f64,
    pub speed_limit_kph: u32,
    pub lanes: u32,
    pub layer: i8,
    pub is_bridge: bool,
    pub is_tunnel: bool,
    pub is_oneway: bool,
    pub is_roundabout: bool,
    pub is_ramp: bool,
    pub access_restriction: RoadAccessRestriction,
    pub inferred_attributes: InferredRoadAttributes,
}

impl RoadEdge {
    /// Travel time in seconds across this segment.
    pub fn traversal_time_seconds(&self) -> f64 {
        let speed_ms = (self.speed_limit_kph as f64) * (1000.0 / 3600.0);
        self.length_meters / speed_ms.max(1.0)
    }

    /// Whether this road edge is drivable by standard passenger and commercial motor vehicles.
    pub fn is_drivable_motor_vehicle(&self) -> bool {
        matches!(
            self.access_restriction,
            RoadAccessRestriction::All | RoadAccessRestriction::EmergencyOnly
        ) && !matches!(self.highway_type, HighwayType::Footway | HighwayType::Path)
    }
}

/// Directed road network graph for spatial pathfinding.
#[derive(Debug, Clone, Default)]
pub struct RoadNetworkGraph {
    pub graph: DiGraph<RoadNode, RoadEdge>,
    coord_to_node: HashMap<(i64, i64, i8), NodeIndex>,
    pub turn_restrictions: Vec<TurnRestriction>,
}

impl RoadNetworkGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            coord_to_node: HashMap::new(),
            turn_restrictions: Vec::new(),
        }
    }

    /// Adds a turn restriction to the road graph.
    pub fn add_turn_restriction(&mut self, restriction: TurnRestriction) {
        self.turn_restrictions.push(restriction);
    }

    /// Checks if a turn between two way IDs is restricted.
    pub fn get_turn_restriction(
        &self,
        from_way_id: u64,
        to_way_id: u64,
    ) -> Option<TurnRestrictionType> {
        self.turn_restrictions
            .iter()
            .find(|r| r.from_way_id == from_way_id && r.to_way_id == to_way_id)
            .map(|r| r.restriction_type)
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

        let is_ramp = road
            .name
            .as_deref()
            .map(|n| {
                let lower = n.to_lowercase();
                lower.contains("ramp") || lower.contains("exit") || lower.contains("connector")
            })
            .unwrap_or(false);

        let is_roundabout = road
            .name
            .as_deref()
            .map(|n| {
                let lower = n.to_lowercase();
                lower.contains("roundabout") || lower.contains("rotary") || lower.contains("circus")
            })
            .unwrap_or(false)
            || (road.geometry.len() >= 4 && road.geometry.first() == road.geometry.last());

        let access_restriction = match road.highway_type {
            HighwayType::Footway | HighwayType::Path => RoadAccessRestriction::PedestrianOnly,
            HighwayType::Service => RoadAccessRestriction::All,
            _ => RoadAccessRestriction::All,
        };

        let inferred_attributes = InferredRoadAttributes {
            is_lanes_inferred: road.lanes <= 2,
            is_speed_limit_inferred: road.speed_limit_kph
                == road.highway_type.default_speed_limit(),
            is_name_missing: road.name.is_none(),
        };

        let geom_len = road.geometry.len();
        for i in 0..(geom_len - 1) {
            let p0 = road.geometry[i];
            let p1 = road.geometry[i + 1];

            let layer0 = self.resolve_point_layer(road, i, geom_len);
            let layer1 = self.resolve_point_layer(road, i + 1, geom_len);

            let n0 = self.get_or_create_node(p0, layer0);
            let n1 = self.get_or_create_node(p1, layer1);

            // Accurate metric distance in meters with latitude-dependent cosine scaling
            let d_lat = (p1.lat - p0.lat) * 111_000.0;
            let m_per_deg_lon = 111_320.0 * (p0.lat.to_radians()).cos().abs().max(0.1);
            let d_lon = (p1.lon - p0.lon) * m_per_deg_lon;
            let length_meters = (d_lat * d_lat + d_lon * d_lon).sqrt().max(0.1);

            let edge = RoadEdge {
                way_id: road.way_id,
                name: road.name.clone(),
                highway_type: road.highway_type,
                length_meters,
                speed_limit_kph: road.speed_limit_kph,
                lanes: road.lanes,
                layer: road.layer,
                is_bridge: road.is_bridge,
                is_tunnel: road.is_tunnel,
                is_oneway: road.is_oneway,
                is_roundabout,
                is_ramp,
                access_restriction,
                inferred_attributes,
            };

            // Forward direction
            self.graph.add_edge(n0, n1, edge.clone());

            // Reverse direction if not a one-way street
            if !road.is_oneway {
                self.graph.add_edge(n1, n0, edge);
            }
        }
    }

    /// Resolves the vertical layer for a specific point on a road feature.
    /// Bridge abutments and tunnel portals at the way terminals transition to ground (layer 0)
    /// to connect with approach roads, while elevated spans remain on their designated layer.
    fn resolve_point_layer(&self, road: &OsmRoadFeature, idx: usize, geom_len: usize) -> i8 {
        if road.is_bridge || road.is_tunnel {
            // Only terminal endpoints of a bridge/tunnel can be abutments/portals touching ground level (layer 0).
            let is_terminal = idx == 0 || idx == geom_len - 1;
            if is_terminal {
                let key_ground = (
                    (road.geometry[idx].lat * 1e7).round() as i64,
                    (road.geometry[idx].lon * 1e7).round() as i64,
                    0,
                );
                if self.coord_to_node.contains_key(&key_ground) {
                    return 0;
                }
                if idx == 0 {
                    return 0;
                }
                if idx == geom_len - 1 && geom_len >= 3 {
                    return 0;
                }
            }
            road.layer
        } else {
            road.layer
        }
    }

    /// Adds an explicit inter-layer transition ramp edge connecting different elevation layers.
    #[allow(clippy::too_many_arguments)]
    pub fn add_ramp_connection(
        &mut self,
        p0: Wgs84Coord,
        layer0: i8,
        p1: Wgs84Coord,
        layer1: i8,
        way_id: u64,
        name: Option<String>,
        speed_limit_kph: u32,
    ) {
        let n0 = self.get_or_create_node(p0, layer0);
        let n1 = self.get_or_create_node(p1, layer1);

        let d_lat = (p1.lat - p0.lat) * 111_000.0;
        let m_per_deg_lon = 111_320.0 * (p0.lat.to_radians()).cos().abs().max(0.1);
        let d_lon = (p1.lon - p0.lon) * m_per_deg_lon;
        let length_meters = (d_lat * d_lat + d_lon * d_lon).sqrt().max(0.1);

        let edge = RoadEdge {
            way_id,
            name,
            highway_type: HighwayType::Motorway,
            length_meters,
            speed_limit_kph,
            lanes: 1,
            layer: layer1,
            is_bridge: layer1 > 0,
            is_tunnel: layer1 < 0,
            is_oneway: true,
            is_roundabout: false,
            is_ramp: true,
            access_restriction: RoadAccessRestriction::All,
            inferred_attributes: InferredRoadAttributes::default(),
        };

        self.graph.add_edge(n0, n1, edge);
    }

    /// Gets existing node or inserts a new one for a coordinate and vertical layer.
    pub fn get_or_create_node(&mut self, coord: Wgs84Coord, layer: i8) -> NodeIndex {
        let key = (
            (coord.lat * 1e7).round() as i64,
            (coord.lon * 1e7).round() as i64,
            layer,
        );
        if let Some(&node_idx) = self.coord_to_node.get(&key) {
            node_idx
        } else {
            let id = self.graph.node_count() as u64;
            let node_idx = self.graph.add_node(RoadNode { id, coord, layer });
            self.coord_to_node.insert(key, node_idx);
            node_idx
        }
    }

    /// Finds the nearest graph node to an arbitrary WGS84 coordinate, optionally constrained to a layer.
    pub fn find_nearest_node_with_layer(
        &self,
        coord: Wgs84Coord,
        layer: Option<i8>,
    ) -> Option<NodeIndex> {
        let mut best_node = None;
        let mut min_dist_sq = f64::MAX;
        let m_per_deg_lon = 111_320.0 * (coord.lat.to_radians()).cos().abs().max(0.1);

        for node_idx in self.graph.node_indices() {
            let node = &self.graph[node_idx];
            if let Some(target_layer) = layer {
                if node.layer != target_layer {
                    continue;
                }
            }

            let d_lat = (node.coord.lat - coord.lat) * 111_000.0;
            let d_lon = (node.coord.lon - coord.lon) * m_per_deg_lon;
            let dist_sq = d_lat * d_lat + d_lon * d_lon;

            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_node = Some(node_idx);
            }
        }

        best_node
    }

    /// Finds the nearest graph node to an arbitrary WGS84 coordinate across all layers.
    pub fn find_nearest_node(&self, coord: Wgs84Coord) -> Option<NodeIndex> {
        self.find_nearest_node_with_layer(coord, None)
    }

    /// Computes parallel lane centerline coordinates offset according to traffic rules.
    pub fn compute_lane_coordinates(
        &self,
        p0: Wgs84Coord,
        p1: Wgs84Coord,
        lane: &LaneDescriptor,
    ) -> (Wgs84Coord, Wgs84Coord) {
        let d_lat = (p1.lat - p0.lat) * 111_000.0;
        let m_per_deg_lon = 111_320.0 * (p0.lat.to_radians()).cos().abs().max(0.1);
        let d_lon = (p1.lon - p0.lon) * m_per_deg_lon;
        let len = (d_lat * d_lat + d_lon * d_lon).sqrt().max(0.1);

        // Unit forward vector in meters: (u_lon, u_lat)
        let u_lon = d_lon / len;
        let u_lat = d_lat / len;

        // Perpendicular vector pointing to the right of forward heading: (u_lat, -u_lon)
        let perp_lon = u_lat;
        let perp_lat = -u_lon;

        let offset = lane.lateral_offset_meters() as f64;
        let off_lon_deg = (perp_lon * offset) / m_per_deg_lon;
        let off_lat_deg = (perp_lat * offset) / 111_000.0;

        (
            Wgs84Coord::new(p0.lat + off_lat_deg, p0.lon + off_lon_deg, p0.alt),
            Wgs84Coord::new(p1.lat + off_lat_deg, p1.lon + off_lon_deg, p1.alt),
        )
    }
}

/// Rule of the road governing traffic flow side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RuleOfTheRoad {
    #[default]
    DriveOnLeft, // India, UK, Japan, Australia
    DriveOnRight, // USA, Canada, Continental Europe
}

/// Lane descriptor with lateral offset calculation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LaneDescriptor {
    pub lane_index: u32,
    pub total_lanes: u32,
    pub is_oneway: bool,
    pub rule: RuleOfTheRoad,
    pub lane_width_meters: f32,
}

impl LaneDescriptor {
    pub fn new(lane_index: u32, total_lanes: u32, is_oneway: bool, rule: RuleOfTheRoad) -> Self {
        Self {
            lane_index,
            total_lanes: total_lanes.max(1),
            is_oneway,
            rule,
            lane_width_meters: 3.5,
        }
    }

    /// Computes signed lateral offset from road centerline in meters.
    /// Positive offset is to the right of travel direction; negative is to the left.
    pub fn lateral_offset_meters(&self) -> f32 {
        if self.is_oneway {
            let center = (self.total_lanes as f32 - 1.0) * 0.5;
            (self.lane_index as f32 - center) * self.lane_width_meters
        } else {
            let forward_lanes = (self.total_lanes / 2).max(1);
            let lane_in_dir = (self.lane_index % forward_lanes) as f32;
            let offset_mag = (lane_in_dir + 0.5) * self.lane_width_meters;

            match self.rule {
                RuleOfTheRoad::DriveOnLeft => -offset_mag, // Left-Hand Traffic: drive on left side of center
                RuleOfTheRoad::DriveOnRight => offset_mag, // Right-Hand Traffic: drive on right side of center
            }
        }
    }
}
