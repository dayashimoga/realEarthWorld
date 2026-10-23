//! A* Pathfinding, turn-by-turn guidance, and route tracking.

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
    pub edge_way_ids: Vec<u64>,
    pub road_names: Vec<String>,
}

/// Type of turn-by-turn driving maneuver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManeuverType {
    Depart,
    Continue,
    SlightLeft,
    SlightRight,
    TurnLeft,
    TurnRight,
    SharpLeft,
    SharpRight,
    Uturn,
    EnterRoundabout,
    ExitRoundabout,
    TakeRamp,
    Arrive,
}

/// A discrete step or maneuver in turn-by-turn route guidance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteInstruction {
    pub step_index: usize,
    pub maneuver: ManeuverType,
    pub instruction: String,
    pub distance_to_next_meters: f64,
    pub street_name: Option<String>,
    pub location: Wgs84Coord,
}

/// Full turn-by-turn guidance itinerary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurnByTurnGuidance {
    pub instructions: Vec<RouteInstruction>,
    pub total_distance_meters: f64,
    pub total_duration_seconds: f64,
}

/// Real-time navigation tracking status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationStatus {
    EnRoute,
    WrongWayWarning,
    OffRouteRerouteRequired,
    Arrived,
}

/// Real-time navigation update snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavigationUpdate {
    pub status: NavigationStatus,
    pub current_instruction: Option<RouteInstruction>,
    pub distance_to_next_maneuver_meters: f64,
    pub distance_remaining_meters: f64,
    pub time_remaining_seconds: f64,
    pub cross_track_error_meters: f64,
    pub is_wrong_way: bool,
    pub is_off_route: bool,
}

/// Active route tracker monitoring vehicle position, heading, maneuvers, and wrong-way status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteTracker {
    pub route: NavigationRoute,
    pub guidance: TurnByTurnGuidance,
    pub current_instruction_idx: usize,
    pub current_waypoint_idx: usize,
    pub destination_radius_meters: f64,
    pub off_route_threshold_meters: f64,
}

impl RouteTracker {
    pub fn new(route: NavigationRoute, guidance: TurnByTurnGuidance) -> Self {
        Self {
            route,
            guidance,
            current_instruction_idx: 0,
            current_waypoint_idx: 0,
            destination_radius_meters: 12.0,
            off_route_threshold_meters: 35.0,
        }
    }

    /// Evaluates current vehicle position and heading to yield real-time guidance and wrong-way warnings.
    pub fn update(&mut self, current_pos: Wgs84Coord, heading_deg: f64) -> NavigationUpdate {
        if self.route.waypoints.is_empty() {
            return NavigationUpdate {
                status: NavigationStatus::Arrived,
                current_instruction: None,
                distance_to_next_maneuver_meters: 0.0,
                distance_remaining_meters: 0.0,
                time_remaining_seconds: 0.0,
                cross_track_error_meters: 0.0,
                is_wrong_way: false,
                is_off_route: false,
            };
        }

        let destination = *self.route.waypoints.last().unwrap();
        let dist_to_dest = metric_distance_meters(current_pos, destination);

        // 1. Destination Arrival Check
        if dist_to_dest <= self.destination_radius_meters {
            return NavigationUpdate {
                status: NavigationStatus::Arrived,
                current_instruction: self.guidance.instructions.last().cloned(),
                distance_to_next_maneuver_meters: 0.0,
                distance_remaining_meters: 0.0,
                time_remaining_seconds: 0.0,
                cross_track_error_meters: 0.0,
                is_wrong_way: false,
                is_off_route: false,
            };
        }

        // 2. Advance active waypoint segment along the route
        let waypoints = &self.route.waypoints;
        while self.current_waypoint_idx < waypoints.len().saturating_sub(2) {
            let p_curr = waypoints[self.current_waypoint_idx];
            let p_next = waypoints[self.current_waypoint_idx + 1];
            let (t, _) = project_point_on_segment(current_pos, p_curr, p_next);

            // If past the endpoint of current segment, advance to next segment
            if t >= 0.95 || metric_distance_meters(current_pos, p_next) < 8.0 {
                self.current_waypoint_idx += 1;
            } else {
                break;
            }
        }

        let seg_start = waypoints[self.current_waypoint_idx];
        let seg_end = waypoints[(self.current_waypoint_idx + 1).min(waypoints.len() - 1)];
        let (_t_proj, cross_track_err) = project_point_on_segment(current_pos, seg_start, seg_end);

        // 3. Off-route Detection
        let is_off_route = cross_track_err > self.off_route_threshold_meters;

        // 4. Wrong-way Detection
        let seg_bearing = calculate_bearing_deg(seg_start, seg_end);
        let bearing_diff = angle_difference_deg(seg_bearing, heading_deg);
        let is_wrong_way = bearing_diff.abs() > 105.0;

        // 5. Compute Remaining Distance along Polyline
        let dist_to_seg_end = metric_distance_meters(current_pos, seg_end);
        let mut dist_remaining = dist_to_seg_end;
        for i in (self.current_waypoint_idx + 1)..(waypoints.len() - 1) {
            dist_remaining += metric_distance_meters(waypoints[i], waypoints[i + 1]);
        }

        let time_remaining = dist_remaining / (13.89); // ~50 km/h average speed

        // 6. Find current/next instruction
        while self.current_instruction_idx < self.guidance.instructions.len().saturating_sub(1) {
            let inst = &self.guidance.instructions[self.current_instruction_idx];
            let dist_to_inst = metric_distance_meters(current_pos, inst.location);
            if dist_to_inst < 10.0 {
                self.current_instruction_idx += 1;
            } else {
                break;
            }
        }

        let current_inst = self
            .guidance
            .instructions
            .get(self.current_instruction_idx)
            .cloned();

        let dist_to_maneuver = current_inst
            .as_ref()
            .map(|inst| metric_distance_meters(current_pos, inst.location))
            .unwrap_or(dist_remaining);

        let status = if is_wrong_way {
            NavigationStatus::WrongWayWarning
        } else if is_off_route {
            NavigationStatus::OffRouteRerouteRequired
        } else {
            NavigationStatus::EnRoute
        };

        NavigationUpdate {
            status,
            current_instruction: current_inst,
            distance_to_next_maneuver_meters: dist_to_maneuver,
            distance_remaining_meters: dist_remaining,
            time_remaining_seconds: time_remaining,
            cross_track_error_meters: cross_track_err,
            is_wrong_way,
            is_off_route,
        }
    }
}

/// Metric distance in meters between two WGS84 coordinates.
pub fn metric_distance_meters(p0: Wgs84Coord, p1: Wgs84Coord) -> f64 {
    let mean_lat = (p0.lat + p1.lat) * 0.5;
    let m_per_deg_lon = 111_320.0 * (mean_lat.to_radians()).cos().abs().max(0.1);
    let d_lat = (p1.lat - p0.lat) * 111_000.0;
    let d_lon = (p1.lon - p0.lon) * m_per_deg_lon;
    (d_lat * d_lat + d_lon * d_lon).sqrt()
}

/// Initial compass bearing in degrees [0, 360) from p0 to p1.
pub fn calculate_bearing_deg(p0: Wgs84Coord, p1: Wgs84Coord) -> f64 {
    let lat1 = p0.lat.to_radians();
    let lat2 = p1.lat.to_radians();
    let dlon = (p1.lon - p0.lon).to_radians();

    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    let initial_bearing = y.atan2(x).to_degrees();
    (initial_bearing + 360.0) % 360.0
}

/// Smallest signed angular difference in degrees between two bearings, bounded in [-180, 180].
pub fn angle_difference_deg(from_deg: f64, to_deg: f64) -> f64 {
    let mut diff = (to_deg - from_deg + 180.0) % 360.0 - 180.0;
    if diff < -180.0 {
        diff += 360.0;
    }
    diff
}

/// Projects point P onto line segment AB, returning parameter t clamped to [0, 1] and perpendicular distance in meters.
pub fn project_point_on_segment(p: Wgs84Coord, a: Wgs84Coord, b: Wgs84Coord) -> (f64, f64) {
    let mean_lat = (a.lat + b.lat) * 0.5;
    let m_per_deg_lon = 111_320.0 * (mean_lat.to_radians()).cos().abs().max(0.1);

    let ab_x = (b.lon - a.lon) * m_per_deg_lon;
    let ab_y = (b.lat - a.lat) * 111_000.0;

    let ap_x = (p.lon - a.lon) * m_per_deg_lon;
    let ap_y = (p.lat - a.lat) * 111_000.0;

    let ab_len_sq = ab_x * ab_x + ab_y * ab_y;
    if ab_len_sq < 1e-4 {
        return (0.0, (ap_x * ap_x + ap_y * ap_y).sqrt());
    }

    let t = ((ap_x * ab_x + ap_y * ab_y) / ab_len_sq).clamp(0.0, 1.0);
    let proj_x = t * ab_x;
    let proj_y = t * ab_y;

    let dx = ap_x - proj_x;
    let dy = ap_y - proj_y;
    let dist = (dx * dx + dy * dy).sqrt();

    (t, dist)
}

/// Generates turn-by-turn guidance from a calculated navigation route.
pub fn generate_turn_by_turn(
    route: &NavigationRoute,
    _network: &RoadNetworkGraph,
) -> TurnByTurnGuidance {
    if route.waypoints.len() < 2 {
        return TurnByTurnGuidance {
            instructions: vec![RouteInstruction {
                step_index: 0,
                maneuver: ManeuverType::Arrive,
                instruction: "You have arrived at your destination".to_string(),
                distance_to_next_meters: 0.0,
                street_name: route.road_names.first().cloned(),
                location: route
                    .waypoints
                    .first()
                    .copied()
                    .unwrap_or(Wgs84Coord::new(0.0, 0.0, 0.0)),
            }],
            total_distance_meters: route.total_distance_meters,
            total_duration_seconds: route.estimated_duration_seconds,
        };
    }

    let mut instructions = Vec::new();

    // 1. Initial departure instruction
    let first_street = route.road_names.first().cloned();
    let depart_text = match &first_street {
        Some(name) => format!("Depart heading along {}", name),
        None => "Depart along route".to_string(),
    };
    instructions.push(RouteInstruction {
        step_index: 0,
        maneuver: ManeuverType::Depart,
        instruction: depart_text,
        distance_to_next_meters: metric_distance_meters(route.waypoints[0], route.waypoints[1]),
        street_name: first_street,
        location: route.waypoints[0],
    });

    // 2. Intermediate maneuvers
    for i in 1..(route.waypoints.len() - 1) {
        let p_prev = route.waypoints[i - 1];
        let p_curr = route.waypoints[i];
        let p_next = route.waypoints[i + 1];

        let b1 = calculate_bearing_deg(p_prev, p_curr);
        let b2 = calculate_bearing_deg(p_curr, p_next);
        let angle_diff = angle_difference_deg(b1, b2);

        let street_name = route.road_names.get(i).cloned();
        let prev_street = route.road_names.get(i - 1).cloned();
        let name_changed = street_name != prev_street && street_name.is_some();

        let maneuver = if angle_diff > 135.0 {
            ManeuverType::SharpRight
        } else if angle_diff > 45.0 {
            ManeuverType::TurnRight
        } else if angle_diff > 20.0 {
            ManeuverType::SlightRight
        } else if angle_diff < -135.0 {
            ManeuverType::SharpLeft
        } else if angle_diff < -45.0 {
            ManeuverType::TurnLeft
        } else if angle_diff < -20.0 {
            ManeuverType::SlightLeft
        } else if name_changed {
            ManeuverType::Continue
        } else {
            continue;
        };

        let street_label = street_name.as_deref().unwrap_or("road");
        let instruction_text = match maneuver {
            ManeuverType::TurnRight => format!("Turn right onto {}", street_label),
            ManeuverType::SlightRight => format!("Bear right onto {}", street_label),
            ManeuverType::SharpRight => format!("Take sharp right onto {}", street_label),
            ManeuverType::TurnLeft => format!("Turn left onto {}", street_label),
            ManeuverType::SlightLeft => format!("Bear left onto {}", street_label),
            ManeuverType::SharpLeft => format!("Take sharp left onto {}", street_label),
            ManeuverType::Continue => format!("Continue onto {}", street_label),
            _ => format!("Proceed along {}", street_label),
        };

        let dist_to_next = metric_distance_meters(p_curr, p_next);
        instructions.push(RouteInstruction {
            step_index: instructions.len(),
            maneuver,
            instruction: instruction_text,
            distance_to_next_meters: dist_to_next,
            street_name,
            location: p_curr,
        });
    }

    // 3. Final arrival instruction
    let last_pos = *route.waypoints.last().unwrap();
    instructions.push(RouteInstruction {
        step_index: instructions.len(),
        maneuver: ManeuverType::Arrive,
        instruction: "You have arrived at your destination".to_string(),
        distance_to_next_meters: 0.0,
        street_name: route.road_names.last().cloned(),
        location: last_pos,
    });

    TurnByTurnGuidance {
        instructions,
        total_distance_meters: route.total_distance_meters,
        total_duration_seconds: route.estimated_duration_seconds,
    }
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
            edge_way_ids: Vec::new(),
            road_names: Vec::new(),
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
            let mut edge_way_ids = Vec::new();
            let mut road_names = Vec::new();
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
                        edge_way_ids.push(w.way_id);
                        if let Some(ref name) = w.name {
                            road_names.push(name.clone());
                        } else {
                            road_names.push("Unnamed Road".to_string());
                        }
                    }
                }
            }

            Ok(NavigationRoute {
                waypoints,
                total_distance_meters: total_distance,
                estimated_duration_seconds: total_duration,
                edge_way_ids,
                road_names,
            })
        }
        None => Err(RealEarthError::Geospatial(
            "No navigable road route found between coordinates".to_string(),
        )),
    }
}
