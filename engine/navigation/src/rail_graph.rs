//! Railway track topology, switches, block signaling, and level crossings.

use realearth_geospatial::EnuConverter;
use realearth_ingestion::osm_types::OsmRailwayFeature;
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Track gauge standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RailGauge {
    /// Indian Broad Gauge (1676 mm).
    Broad,
    /// Standard International Gauge (1435 mm).
    Standard,
    /// Metre Gauge (1000 mm).
    Metre,
}

/// Dynamic state of a railway track turnout/switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchState {
    /// Switch aligned to primary straight mainline track.
    Straight,
    /// Switch aligned to divergent siding or branch track.
    Diverging,
    /// Switch locked for maintenance or opposing conflicting route.
    Locked,
}

/// Automatic block signaling aspects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalAspect {
    /// Clear — proceed at maximum track line speed (Green).
    Clear,
    /// Approach — prepare to stop at next signal block (Yellow).
    Approach,
    /// Stop — danger, do not pass signal (Red).
    Stop,
}

/// Status of road/rail level crossings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossingState {
    /// Gates fully open, road traffic flowing.
    Open,
    /// Warning flashing/bell active, gates lowering.
    Closing,
    /// Gates fully lowered, train has right-of-way.
    Closed,
    /// Train cleared, gates raising.
    Opening,
}

/// A node within the railway network (junction, switch, station platform).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RailNode {
    pub id: String,
    pub coord: Wgs84Coord,
    pub is_station: bool,
    pub station_name: Option<String>,
}

/// A directed track segment connecting two rail nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RailTrackSegment {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    pub length_meters: f64,
    pub speed_limit_kph: f32,
    pub gauge: RailGauge,
    pub is_electrified: bool,
    pub switch_state: Option<SwitchState>,
    pub signal_aspect: SignalAspect,
    #[serde(default)]
    pub geometry: Vec<Wgs84Coord>,
}

/// 3D Position and orientation sampled along a railway track centerline.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RailTrackPose {
    pub position: glam::Vec3,
    pub orientation: glam::Quat,
    pub yaw: f32,
    pub pitch: f32,
    pub curvature_rad_per_m: f32,
}

/// A planned passenger or freight railway corridor between stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RailRoutePlan {
    pub line_name: String,
    pub departure_station_id: String,
    pub departure_station_name: String,
    pub destination_station_id: String,
    pub destination_station_name: String,
    pub segment_ids: Vec<String>,
    pub total_length_meters: f64,
}

impl RailTrackSegment {
    /// Samples local ENU 3D position (X=East, Y=Up, Z=North) and heading (radians from north)
    /// at arclength `s_meters` along this track segment relative to an anchor.
    pub fn sample_position_enu(&self, s_meters: f64, anchor: Wgs84Coord) -> (glam::Vec3, f32) {
        let pose = self.sample_pose_enu(s_meters, anchor);
        (pose.position, pose.yaw)
    }

    /// Samples local ENU 3D pose (position, orientation, yaw, pitch, curvature)
    /// at arclength `s_meters` along this track segment relative to an anchor.
    pub fn sample_pose_enu(&self, s_meters: f64, anchor: Wgs84Coord) -> RailTrackPose {
        if self.geometry.is_empty() {
            return RailTrackPose {
                position: glam::Vec3::ZERO,
                orientation: glam::Quat::IDENTITY,
                yaw: 0.0,
                pitch: 0.0,
                curvature_rad_per_m: 0.0,
            };
        }
        let transforms = EnuConverter::new(anchor);
        let pts: Vec<glam::Vec3> = self
            .geometry
            .iter()
            .map(|c| transforms.wgs84_to_enu(*c).to_vec3())
            .collect();

        if pts.len() == 1 {
            return RailTrackPose {
                position: pts[0],
                orientation: glam::Quat::IDENTITY,
                yaw: 0.0,
                pitch: 0.0,
                curvature_rad_per_m: 0.0,
            };
        }

        let mut cum_dist = Vec::with_capacity(pts.len());
        cum_dist.push(0.0);
        let mut total = 0.0;
        for i in 0..pts.len() - 1 {
            let seg_len = pts[i].distance(pts[i + 1]) as f64;
            total += seg_len;
            cum_dist.push(total);
        }

        if total <= 1e-6 {
            return RailTrackPose {
                position: pts[0],
                orientation: glam::Quat::IDENTITY,
                yaw: 0.0,
                pitch: 0.0,
                curvature_rad_per_m: 0.0,
            };
        }

        let s = s_meters.clamp(0.0, total);
        for i in 0..pts.len() - 1 {
            let d0 = cum_dist[i];
            let d1 = cum_dist[i + 1];
            if s <= d1 || i == pts.len() - 2 {
                let seg_d = (d1 - d0).max(1e-6);
                let t = ((s - d0) / seg_d).clamp(0.0, 1.0) as f32;
                let pos = pts[i].lerp(pts[i + 1], t);
                let diff = pts[i + 1] - pts[i];
                let horiz_len = (diff.x * diff.x + diff.z * diff.z).sqrt().max(1e-4);
                let pitch = (-diff.y).atan2(horiz_len);
                let yaw = diff.x.atan2(diff.z);
                let orientation =
                    glam::Quat::from_rotation_y(yaw) * glam::Quat::from_rotation_x(pitch);

                let curvature = if i + 2 < pts.len() {
                    let diff2 = pts[i + 2] - pts[i + 1];
                    let yaw2 = diff2.x.atan2(diff2.z);
                    let mut dyaw = yaw2 - yaw;
                    while dyaw > std::f32::consts::PI {
                        dyaw -= 2.0 * std::f32::consts::PI;
                    }
                    while dyaw < -std::f32::consts::PI {
                        dyaw += 2.0 * std::f32::consts::PI;
                    }
                    (dyaw / seg_d as f32).abs()
                } else {
                    0.0
                };

                return RailTrackPose {
                    position: pos,
                    orientation,
                    yaw,
                    pitch,
                    curvature_rad_per_m: curvature,
                };
            }
        }

        let last_pt = pts[pts.len() - 1];
        let diff = last_pt - pts[pts.len() - 2];
        let yaw = diff.x.atan2(diff.z);
        RailTrackPose {
            position: last_pt,
            orientation: glam::Quat::from_rotation_y(yaw),
            yaw,
            pitch: 0.0,
            curvature_rad_per_m: 0.0,
        }
    }

    /// Computes minimum orthogonal distance from a point in ENU coordinates to this track segment.
    pub fn cross_track_distance(&self, point_enu: glam::Vec3, anchor: Wgs84Coord) -> f32 {
        if self.geometry.len() < 2 {
            return f32::MAX;
        }
        let transforms = EnuConverter::new(anchor);
        let pts: Vec<glam::Vec3> = self
            .geometry
            .iter()
            .map(|c| transforms.wgs84_to_enu(*c).to_vec3())
            .collect();

        let mut min_dist = f32::MAX;
        for i in 0..pts.len() - 1 {
            let a = pts[i];
            let b = pts[i + 1];
            let ab = b - a;
            let ap = point_enu - a;
            let ab_len_sq = ab.length_squared();
            if ab_len_sq < 1e-6 {
                min_dist = min_dist.min(ap.length());
                continue;
            }
            let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
            let proj = a + ab * t;
            let d = (point_enu - proj).length();
            min_dist = min_dist.min(d);
        }
        min_dist
    }
}

/// Road/rail intersection level crossing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RailLevelCrossing {
    pub id: String,
    pub road_name: String,
    pub coord: Wgs84Coord,
    pub state: CrossingState,
    pub timer: f32,
}

/// Railway network graph containing track topology, switches, and stations.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RailNetworkGraph {
    pub nodes: HashMap<String, RailNode>,
    pub segments: HashMap<String, RailTrackSegment>,
    pub crossings: HashMap<String, RailLevelCrossing>,
}

impl RailNetworkGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a rail node to the network.
    pub fn add_node(&mut self, node: RailNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Adds a track segment between two nodes.
    pub fn add_segment(&mut self, segment: RailTrackSegment) {
        self.segments.insert(segment.id.clone(), segment);
    }

    /// Adds a level crossing.
    pub fn add_crossing(&mut self, crossing: RailLevelCrossing) {
        self.crossings.insert(crossing.id.clone(), crossing);
    }

    /// Sets the turnout switch alignment on a track segment.
    pub fn set_switch_state(&mut self, segment_id: &str, state: SwitchState) -> bool {
        if let Some(seg) = self.segments.get_mut(segment_id) {
            seg.switch_state = Some(state);
            true
        } else {
            false
        }
    }

    /// Sets the block signal aspect on a track segment.
    pub fn set_signal_aspect(&mut self, segment_id: &str, aspect: SignalAspect) -> bool {
        if let Some(seg) = self.segments.get_mut(segment_id) {
            seg.signal_aspect = aspect;
            true
        } else {
            false
        }
    }

    /// Triggers level crossing warning and closure for approaching train.
    pub fn trigger_crossing_approach(&mut self, crossing_id: &str) {
        if let Some(crossing) = self.crossings.get_mut(crossing_id) {
            crossing.state = CrossingState::Closed;
        }
    }

    /// Releases level crossing after train passes.
    pub fn release_crossing(&mut self, crossing_id: &str) {
        if let Some(crossing) = self.crossings.get_mut(crossing_id) {
            crossing.state = CrossingState::Open;
        }
    }

    /// Finds connected outgoing segments from a given node, filtering out locked switches.
    pub fn outgoing_segments(&self, node_id: &str) -> Vec<&RailTrackSegment> {
        self.segments
            .values()
            .filter(|s| {
                s.from_node == node_id
                    && s.switch_state != Some(SwitchState::Locked)
                    && s.signal_aspect != SignalAspect::Stop
            })
            .collect()
    }
    /// Ingests OSM railway features into a railway network graph.
    pub fn from_railways(railways: &[OsmRailwayFeature]) -> Self {
        let mut graph = Self::new();
        for rwy in railways {
            if rwy.geometry.len() < 2 {
                continue;
            }
            let from_id = format!("NODE_WAY_{}_START", rwy.way_id);
            let to_id = format!("NODE_WAY_{}_END", rwy.way_id);

            let first_pt = rwy.geometry[0];
            let last_pt = rwy.geometry[rwy.geometry.len() - 1];

            graph.add_node(RailNode {
                id: from_id.clone(),
                coord: first_pt,
                is_station: false,
                station_name: None,
            });
            graph.add_node(RailNode {
                id: to_id.clone(),
                coord: last_pt,
                is_station: false,
                station_name: None,
            });

            let gauge = match rwy.gauge_mm {
                Some(1676) => RailGauge::Broad,
                Some(1000) => RailGauge::Metre,
                _ => RailGauge::Standard,
            };

            let mut length_m = 0.0;
            for i in 0..rwy.geometry.len() - 1 {
                let p1 = rwy.geometry[i];
                let p2 = rwy.geometry[i + 1];
                let d_lat = (p2.lat - p1.lat).to_radians();
                let d_lon = (p2.lon - p1.lon).to_radians();
                let lat_avg = ((p1.lat + p2.lat) * 0.5).to_radians();
                let dx = d_lon * 6371000.0 * lat_avg.cos();
                let dy = d_lat * 6371000.0;
                length_m += (dx * dx + dy * dy).sqrt();
            }

            graph.add_segment(RailTrackSegment {
                id: format!("SEG_WAY_{}", rwy.way_id),
                from_node: from_id,
                to_node: to_id,
                length_meters: length_m,
                speed_limit_kph: 120.0,
                gauge,
                is_electrified: rwy.is_electrified,
                switch_state: None,
                signal_aspect: SignalAspect::Clear,
                geometry: rwy.geometry.clone(),
            });
        }
        graph
    }

    /// Creates the authentic San Francisco Caltrain commuter rail corridor
    /// connecting San Francisco 4th & King Station through Mission Bay, 22nd St, Bayshore, and Millbrae Intermodal.
    pub fn create_san_francisco_caltrain_corridor() -> Self {
        let mut rail = Self::new();

        let n_4th_king = RailNode {
            id: "SF_4TH_KING".to_string(),
            coord: Wgs84Coord::new(37.7764, -122.3942, 5.0),
            is_station: true,
            station_name: Some("San Francisco 4th & King".to_string()),
        };
        let n_switch_mission = RailNode {
            id: "SF_MISSION_BAY_TURNOUT".to_string(),
            coord: Wgs84Coord::new(37.7680, -122.3920, 5.0),
            is_station: false,
            station_name: None,
        };
        let n_22nd_st = RailNode {
            id: "SF_22ND_STREET".to_string(),
            coord: Wgs84Coord::new(37.7570, -122.3925, 10.0),
            is_station: true,
            station_name: Some("22nd Street Station".to_string()),
        };
        let n_yard = RailNode {
            id: "SF_MISSION_BAY_YARD".to_string(),
            coord: Wgs84Coord::new(37.7670, -122.3890, 4.0),
            is_station: false,
            station_name: Some("Mission Bay Maintenance Facility".to_string()),
        };
        let n_bayshore = RailNode {
            id: "SF_BAYSHORE".to_string(),
            coord: Wgs84Coord::new(37.7075, -122.4015, 6.0),
            is_station: true,
            station_name: Some("Bayshore Station".to_string()),
        };
        let n_millbrae = RailNode {
            id: "SF_MILLBRAE_SFO".to_string(),
            coord: Wgs84Coord::new(37.5998, -122.3867, 3.0),
            is_station: true,
            station_name: Some("Millbrae SFO Intermodal".to_string()),
        };

        rail.add_node(n_4th_king);
        rail.add_node(n_switch_mission);
        rail.add_node(n_22nd_st);
        rail.add_node(n_yard);
        rail.add_node(n_bayshore);
        rail.add_node(n_millbrae);

        // Segment 1: 4th & King to Mission Bay Turnout
        rail.add_segment(RailTrackSegment {
            id: "TRK_SF_01".to_string(),
            from_node: "SF_4TH_KING".to_string(),
            to_node: "SF_MISSION_BAY_TURNOUT".to_string(),
            length_meters: 1050.0,
            speed_limit_kph: 45.0,
            gauge: RailGauge::Standard,
            is_electrified: true,
            switch_state: Some(SwitchState::Straight),
            signal_aspect: SignalAspect::Clear,
            geometry: vec![
                Wgs84Coord::new(37.7764, -122.3942, 5.0),
                Wgs84Coord::new(37.7720, -122.3930, 5.0),
                Wgs84Coord::new(37.7680, -122.3920, 5.0),
            ],
        });

        // Segment 2 (Mainline Straight): Turnout to 22nd Street
        rail.add_segment(RailTrackSegment {
            id: "TRK_SF_02_MAIN".to_string(),
            from_node: "SF_MISSION_BAY_TURNOUT".to_string(),
            to_node: "SF_22ND_STREET".to_string(),
            length_meters: 1250.0,
            speed_limit_kph: 80.0,
            gauge: RailGauge::Standard,
            is_electrified: true,
            switch_state: Some(SwitchState::Straight),
            signal_aspect: SignalAspect::Clear,
            geometry: vec![
                Wgs84Coord::new(37.7680, -122.3920, 5.0),
                Wgs84Coord::new(37.7625, -122.3922, 7.0),
                Wgs84Coord::new(37.7570, -122.3925, 10.0),
            ],
        });

        // Segment 2 (Diverging Siding): Turnout to Maintenance Yard
        rail.add_segment(RailTrackSegment {
            id: "TRK_SF_02_YARD".to_string(),
            from_node: "SF_MISSION_BAY_TURNOUT".to_string(),
            to_node: "SF_MISSION_BAY_YARD".to_string(),
            length_meters: 420.0,
            speed_limit_kph: 25.0,
            gauge: RailGauge::Standard,
            is_electrified: true,
            switch_state: Some(SwitchState::Diverging),
            signal_aspect: SignalAspect::Approach,
            geometry: vec![
                Wgs84Coord::new(37.7680, -122.3920, 5.0),
                Wgs84Coord::new(37.7675, -122.3905, 4.5),
                Wgs84Coord::new(37.7670, -122.3890, 4.0),
            ],
        });

        // Segment 3: 22nd Street to Bayshore
        rail.add_segment(RailTrackSegment {
            id: "TRK_SF_03".to_string(),
            from_node: "SF_22ND_STREET".to_string(),
            to_node: "SF_BAYSHORE".to_string(),
            length_meters: 5600.0,
            speed_limit_kph: 110.0,
            gauge: RailGauge::Standard,
            is_electrified: true,
            switch_state: None,
            signal_aspect: SignalAspect::Clear,
            geometry: vec![
                Wgs84Coord::new(37.7570, -122.3925, 10.0),
                Wgs84Coord::new(37.7320, -122.3980, 8.0),
                Wgs84Coord::new(37.7075, -122.4015, 6.0),
            ],
        });

        // Segment 4: Bayshore to Millbrae SFO
        rail.add_segment(RailTrackSegment {
            id: "TRK_SF_04".to_string(),
            from_node: "SF_BAYSHORE".to_string(),
            to_node: "SF_MILLBRAE_SFO".to_string(),
            length_meters: 12200.0,
            speed_limit_kph: 125.0,
            gauge: RailGauge::Standard,
            is_electrified: true,
            switch_state: None,
            signal_aspect: SignalAspect::Clear,
            geometry: vec![
                Wgs84Coord::new(37.7075, -122.4015, 6.0),
                Wgs84Coord::new(37.6560, -122.4060, 4.0),
                Wgs84Coord::new(37.5998, -122.3867, 3.0),
            ],
        });

        rail
    }

    /// Checks whether any track exists in this graph.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Number of verified track segments in this graph.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Number of station nodes in this graph.
    pub fn station_count(&self) -> usize {
        self.nodes.values().filter(|n| n.is_station).count()
    }

    /// Samples 3D pose along a multi-segment route at distance `distance_meters`.
    /// Returns: (Pose, Active Segment Index, Distance on Segment, Is at buffer stop).
    pub fn sample_route_pose(
        &self,
        route_segments: &[String],
        distance_meters: f64,
        anchor: Wgs84Coord,
    ) -> Option<(RailTrackPose, usize, f64, bool)> {
        if route_segments.is_empty() {
            return None;
        }

        let mut d = distance_meters;

        if d < 0.0 {
            if let Some(first_seg) = self.segments.get(&route_segments[0]) {
                let pose = first_seg.sample_pose_enu(0.0, anchor);
                return Some((pose, 0, 0.0, true));
            }
        }

        for (idx, seg_id) in route_segments.iter().enumerate() {
            if let Some(seg) = self.segments.get(seg_id) {
                let seg_len = seg.length_meters;
                if d <= seg_len || idx == route_segments.len() - 1 {
                    let clamped_s = d.clamp(0.0, seg_len);
                    let pose = seg.sample_pose_enu(clamped_s, anchor);
                    let at_buffer = idx == route_segments.len() - 1 && d >= seg_len;
                    return Some((pose, idx, clamped_s, at_buffer));
                }
                d -= seg_len;
            }
        }

        None
    }

    /// Total track length along a sequence of segment IDs.
    pub fn total_route_length(&self, route_segments: &[String]) -> f64 {
        route_segments
            .iter()
            .filter_map(|id| self.segments.get(id))
            .map(|s| s.length_meters)
            .sum()
    }

    /// Minimum cross-track distance from a point in ENU space to any track in this route.
    pub fn route_cross_track_distance(
        &self,
        route_segments: &[String],
        point_enu: glam::Vec3,
        anchor: Wgs84Coord,
    ) -> f32 {
        let mut min_d = f32::MAX;
        for seg_id in route_segments {
            if let Some(seg) = self.segments.get(seg_id) {
                let d = seg.cross_track_distance(point_enu, anchor);
                min_d = min_d.min(d);
            }
        }
        min_d
    }

    /// Finds a station node by name (case-insensitive substring match).
    pub fn find_station_by_name(&self, name_substr: &str) -> Option<&RailNode> {
        let needle = name_substr.to_lowercase();
        self.nodes.values().find(|n| {
            n.is_station
                && n.station_name
                    .as_ref()
                    .map(|s| s.to_lowercase().contains(&needle))
                    .unwrap_or(false)
        })
    }

    /// Lists all station nodes sorted alphabetically by station name.
    pub fn list_stations(&self) -> Vec<&RailNode> {
        let mut list: Vec<&RailNode> = self.nodes.values().filter(|n| n.is_station).collect();
        list.sort_by_key(|n| n.station_name.clone().unwrap_or_default());
        list
    }

    /// Builds the standard passenger route between two station nodes.
    pub fn build_caltrain_mainline_route() -> RailRoutePlan {
        RailRoutePlan {
            line_name: "Caltrain Peninsula Commuter Line".to_string(),
            departure_station_id: "SF_4TH_KING".to_string(),
            departure_station_name: "San Francisco 4th & King (Platform 1)".to_string(),
            destination_station_id: "SF_MILLBRAE_SFO".to_string(),
            destination_station_name: "Millbrae SFO Intermodal (Terminal)".to_string(),
            segment_ids: vec![
                "TRK_SF_01".to_string(),
                "TRK_SF_02_MAIN".to_string(),
                "TRK_SF_03".to_string(),
                "TRK_SF_04".to_string(),
            ],
            total_length_meters: 1050.0 + 1250.0 + 5600.0 + 12200.0,
        }
    }

    /// Builds a siding route into the maintenance yard.
    pub fn build_caltrain_yard_siding_route() -> RailRoutePlan {
        RailRoutePlan {
            line_name: "Mission Bay Depot & Maintenance Siding".to_string(),
            departure_station_id: "SF_4TH_KING".to_string(),
            departure_station_name: "San Francisco 4th & King (Yard Track)".to_string(),
            destination_station_id: "SF_MISSION_BAY_YARD".to_string(),
            destination_station_name: "Mission Bay Maintenance Facility".to_string(),
            segment_ids: vec!["TRK_SF_01".to_string(), "TRK_SF_02_YARD".to_string()],
            total_length_meters: 1050.0 + 420.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_railway_network_graph_and_switches() {
        let mut rail = RailNetworkGraph::new();

        let n1 = RailNode {
            id: "SBC_PLATFORM_1".to_string(),
            coord: Wgs84Coord::new(12.9780, 77.5680, 915.0),
            is_station: true,
            station_name: Some("KSR Bengaluru City".to_string()),
        };
        let n2 = RailNode {
            id: "SBC_TURNOUT_A".to_string(),
            coord: Wgs84Coord::new(12.9800, 77.5650, 914.0),
            is_station: false,
            station_name: None,
        };
        let n3 = RailNode {
            id: "MYS_MAINLINE".to_string(),
            coord: Wgs84Coord::new(12.9750, 77.5600, 912.0),
            is_station: false,
            station_name: None,
        };

        rail.add_node(n1);
        rail.add_node(n2);
        rail.add_node(n3);

        let seg1 = RailTrackSegment {
            id: "TRK_01".to_string(),
            from_node: "SBC_PLATFORM_1".to_string(),
            to_node: "SBC_TURNOUT_A".to_string(),
            length_meters: 450.0,
            speed_limit_kph: 50.0,
            gauge: RailGauge::Broad,
            is_electrified: true,
            switch_state: Some(SwitchState::Straight),
            signal_aspect: SignalAspect::Clear,
            geometry: vec![
                Wgs84Coord::new(12.9780, 77.5680, 915.0),
                Wgs84Coord::new(12.9800, 77.5650, 914.0),
            ],
        };
        rail.add_segment(seg1);

        assert_eq!(rail.outgoing_segments("SBC_PLATFORM_1").len(), 1);

        // Turn red signal -> outgoing empty
        rail.set_signal_aspect("TRK_01", SignalAspect::Stop);
        assert_eq!(rail.outgoing_segments("SBC_PLATFORM_1").len(), 0);
    }

    #[test]
    fn test_san_francisco_caltrain_corridor() {
        let rail = RailNetworkGraph::create_san_francisco_caltrain_corridor();
        assert_eq!(rail.nodes.len(), 6);
        assert_eq!(rail.segments.len(), 5);

        let seg1 = rail.segments.get("TRK_SF_01").unwrap();
        let anchor = Wgs84Coord::new(37.7764, -122.3942, 5.0);
        let (pos_start, _) = seg1.sample_position_enu(0.0, anchor);
        assert!(pos_start.length() < 0.1, "Start of track must match anchor");

        let (pos_end, heading) = seg1.sample_position_enu(1050.0, anchor);
        assert!(pos_end.length() > 500.0, "End of track must be non-zero");
        assert!(heading.is_finite());
    }
}
