//! Navigable waterways, dredged shipping channels, marine ports, and fairway navigation graphs.
//!
//! Maintains independent water semantics strictly decoupled from road and rail networks.

use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of navigable marine waterway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaterwayCategory {
    /// Deep sea / open ocean navigation route.
    OpenSea,
    /// Dredged deep-water coastal approach channel.
    DredgedChannel,
    /// Inland navigable river or canal.
    InlandCanal,
    /// Protected harbor basin / turning basin.
    HarborBasin,
}

/// Lateral and cardinal marine navigation aid markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationAids {
    /// Port lateral buoy (Red in IALA Region A, Green in Region B).
    PortLateral,
    /// Starboard lateral buoy (Green in IALA Region A, Red in Region B).
    StarboardLateral,
    /// Safe water fairway center buoy.
    SafeWater,
    /// Cardinal mark indicating navigable deep water relative to hazard.
    CardinalNorth,
    /// Lighthouses or sector lights.
    SectorLight,
}

/// A node along a navigable marine waterway network.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterwayNode {
    pub id: String,
    pub coord: Wgs84Coord,
    pub water_depth_meters: f32,
    pub aid_marker: Option<NavigationAids>,
}

/// Navigable edge between two marine waterway nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShippingChannel {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    pub channel_width_meters: f32,
    pub minimum_depth_meters: f32,
    pub speed_limit_knots: f32,
    pub category: WaterwayCategory,
    pub length_meters: f64,
}

/// Marine port facility with berths, docks, and cargo terminals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarinePort {
    pub id: String,
    pub name: String,
    pub basin_coord: Wgs84Coord,
    pub max_draft_meters: f32,
    pub berth_count: u32,
    pub berths: Vec<PortBerth>,
}

/// Dedicated dock or container berth within a port.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortBerth {
    pub berth_id: String,
    pub length_meters: f32,
    pub depth_meters: f32,
    pub is_occupied: bool,
}

/// Navigable marine network graph containing channels, harbors, and ports.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WaterwayGraph {
    pub nodes: HashMap<String, WaterwayNode>,
    pub channels: HashMap<String, ShippingChannel>,
    pub ports: HashMap<String, MarinePort>,
}

impl WaterwayGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a marine navigation node.
    pub fn add_node(&mut self, node: WaterwayNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Registers a shipping fairway channel.
    pub fn add_channel(&mut self, channel: ShippingChannel) {
        self.channels.insert(channel.id.clone(), channel);
    }

    /// Registers a marine port facility.
    pub fn add_port(&mut self, port: MarinePort) {
        self.ports.insert(port.id.clone(), port);
    }

    /// Verifies if a vessel with given draft (depth under waterline) can safely navigate channel.
    pub fn is_channel_navigable(&self, channel_id: &str, vessel_draft_meters: f32) -> bool {
        if let Some(channel) = self.channels.get(channel_id) {
            // Require safety clearance margin of 1.0m under keel
            channel.minimum_depth_meters >= (vessel_draft_meters + 1.0)
        } else {
            false
        }
    }

    /// Finds available berth for docking a vessel of specified length and draft.
    pub fn find_available_berth(
        &mut self,
        port_id: &str,
        vessel_length: f32,
        vessel_draft: f32,
    ) -> Option<String> {
        if let Some(port) = self.ports.get_mut(port_id) {
            for berth in &mut port.berths {
                if !berth.is_occupied
                    && berth.length_meters >= vessel_length
                    && berth.depth_meters >= (vessel_draft + 0.8)
                {
                    berth.is_occupied = true;
                    return Some(berth.berth_id.clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chennai_port_waterway_graph() {
        let mut waterway = WaterwayGraph::new();

        let n1 = WaterwayNode {
            id: "N_BAY_ENTRY".to_string(),
            coord: Wgs84Coord::new(13.0850, 80.3200, 0.0),
            water_depth_meters: 22.0,
            aid_marker: Some(NavigationAids::SafeWater),
        };
        let n2 = WaterwayNode {
            id: "N_OUTER_FAIRWAY".to_string(),
            coord: Wgs84Coord::new(13.0845, 80.3000, 0.0),
            water_depth_meters: 18.5,
            aid_marker: Some(NavigationAids::PortLateral),
        };
        let n3 = WaterwayNode {
            id: "N_HARBOR_BASIN".to_string(),
            coord: Wgs84Coord::new(13.0840, 80.2940, 0.0),
            water_depth_meters: 16.0,
            aid_marker: None,
        };

        waterway.add_node(n1);
        waterway.add_node(n2);
        waterway.add_node(n3);

        let ch1 = ShippingChannel {
            id: "CH_APPROACH".to_string(),
            from_node: "N_BAY_ENTRY".to_string(),
            to_node: "N_OUTER_FAIRWAY".to_string(),
            channel_width_meters: 300.0,
            minimum_depth_meters: 18.5,
            speed_limit_knots: 12.0,
            category: WaterwayCategory::DredgedChannel,
            length_meters: 2100.0,
        };

        let ch2 = ShippingChannel {
            id: "CH_HARBOR_ENTRANCE".to_string(),
            from_node: "N_OUTER_FAIRWAY".to_string(),
            to_node: "N_HARBOR_BASIN".to_string(),
            channel_width_meters: 180.0,
            minimum_depth_meters: 15.5,
            speed_limit_knots: 6.0,
            category: WaterwayCategory::HarborBasin,
            length_meters: 850.0,
        };

        waterway.add_channel(ch1);
        waterway.add_channel(ch2);

        // Port berths
        let port = MarinePort {
            id: "PORT_CHENNAI".to_string(),
            name: "Chennai Container Terminal".to_string(),
            basin_coord: Wgs84Coord::new(13.0840, 80.2940, 0.0),
            max_draft_meters: 15.0,
            berth_count: 2,
            berths: vec![
                PortBerth {
                    berth_id: "BERTH_CCT_1".to_string(),
                    length_meters: 350.0,
                    depth_meters: 15.5,
                    is_occupied: false,
                },
                PortBerth {
                    berth_id: "BERTH_CCT_2".to_string(),
                    length_meters: 280.0,
                    depth_meters: 14.0,
                    is_occupied: true,
                },
            ],
        };
        waterway.add_port(port);

        // Vessel draft validation
        let container_ship_draft = 12.5; // Panamax
        assert!(waterway.is_channel_navigable("CH_APPROACH", container_ship_draft));
        assert!(waterway.is_channel_navigable("CH_HARBOR_ENTRANCE", container_ship_draft));

        let ultra_large_tanker_draft = 20.0; // ULCC
        assert!(!waterway.is_channel_navigable("CH_APPROACH", ultra_large_tanker_draft));

        // Berth assignment
        let berth = waterway.find_available_berth("PORT_CHENNAI", 300.0, 12.5);
        assert_eq!(berth, Some("BERTH_CCT_1".to_string()));

        // Second query should fail because BERTH_CCT_1 is now occupied
        let berth_again = waterway.find_available_berth("PORT_CHENNAI", 300.0, 12.5);
        assert_eq!(berth_again, None);
    }
}
