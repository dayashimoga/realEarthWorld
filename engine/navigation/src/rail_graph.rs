//! Railway track topology, switches, block signaling, and level crossings.

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
        };
        rail.add_segment(seg1);

        assert_eq!(rail.outgoing_segments("SBC_PLATFORM_1").len(), 1);

        // Turn red signal -> outgoing empty
        rail.set_signal_aspect("TRK_01", SignalAspect::Stop);
        assert_eq!(rail.outgoing_segments("SBC_PLATFORM_1").len(), 0);
    }
}
