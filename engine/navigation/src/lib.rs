//! Multimodal navigation graphs and routing engine for Real Earth (Road, Rail, Waterway, Aviation).

pub mod air_navigation;
pub mod rail_graph;
pub mod road_graph;
pub mod router;
pub mod waterway;

pub use air_navigation::{
    AirCorridorSegment, AirNavigationDatabase, Airport, AirportRunway, IlsGuidance,
};
pub use rail_graph::{
    CrossingState, RailGauge, RailLevelCrossing, RailNetworkGraph, RailNode, RailTrackSegment,
    SignalAspect, SwitchState,
};
pub use realearth_ingestion::osm_types::HighwayType;
pub use road_graph::{
    InferredRoadAttributes, LaneDescriptor, RoadAccessRestriction, RoadEdge, RoadNetworkGraph,
    RoadNode, RuleOfTheRoad, TurnRestriction, TurnRestrictionType,
};
pub use router::{
    angle_difference_deg, calculate_bearing_deg, find_route, generate_turn_by_turn,
    metric_distance_meters, project_point_on_segment, ManeuverType, NavigationRoute,
    NavigationStatus, NavigationUpdate, RouteInstruction, RouteTracker, RoutingPreference,
    TurnByTurnGuidance,
};
pub use waterway::{
    MarinePort, NavigationAids, PortBerth, ShippingChannel, WaterwayCategory, WaterwayGraph,
    WaterwayNode,
};

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_ingestion::osm_types::{HighwayType, OsmRoadFeature};
    use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
    use realearth_shared::coordinates::Wgs84Coord;

    #[test]
    fn test_bengaluru_road_graph_creation() {
        let dataset = create_bengaluru_central_dataset();
        let network = RoadNetworkGraph::from_roads(&dataset.roads);

        assert!(network.graph.node_count() > 0, "Graph should have nodes");
        assert!(network.graph.edge_count() > 0, "Graph should have edges");
    }

    #[test]
    fn test_mg_road_a_star_pathfinding() {
        let dataset = create_bengaluru_central_dataset();
        let network = RoadNetworkGraph::from_roads(&dataset.roads);

        // MG Road start: (12.9740, 77.6000) -> MG Road end: (12.9760, 77.6150)
        let start = Wgs84Coord::new(12.9740, 77.6000, 0.0);
        let end = Wgs84Coord::new(12.9760, 77.6150, 0.0);

        let route = find_route(&network, start, end, RoutingPreference::ShortestDistance)
            .expect("Pathfinding should succeed on MG Road");

        assert!(!route.waypoints.is_empty());
        assert!(
            route.total_distance_meters > 1000.0,
            "MG Road length should be > 1km"
        );
        assert!(route.estimated_duration_seconds > 0.0);
        assert!(!route.road_names.is_empty());
    }

    #[test]
    fn test_lane_offset_calculation() {
        // 4-lane two-way road: 2 lanes each direction
        // Lane 0: inner forward lane
        let lht_lane0 = LaneDescriptor::new(0, 4, false, RuleOfTheRoad::DriveOnLeft);
        assert!(
            lht_lane0.lateral_offset_meters() < 0.0,
            "LHT traffic should be offset to the left (negative)"
        );
        assert!((lht_lane0.lateral_offset_meters() - -1.75).abs() < 1e-4);

        let rht_lane0 = LaneDescriptor::new(0, 4, false, RuleOfTheRoad::DriveOnRight);
        assert!(
            rht_lane0.lateral_offset_meters() > 0.0,
            "RHT traffic should be offset to the right (positive)"
        );
        assert!((rht_lane0.lateral_offset_meters() - 1.75).abs() < 1e-4);
    }

    #[test]
    fn test_vidhana_soudha_to_mg_road_connected_route() {
        let dataset = create_bengaluru_central_dataset();
        let network = RoadNetworkGraph::from_roads(&dataset.roads);

        // Vidhana Soudha -> MG Road Anil Kumble Circle
        let vidhana = Wgs84Coord::new(12.9791, 77.5901, 0.0);
        let mg_road_start = Wgs84Coord::new(12.9740, 77.6000, 0.0);

        let route = find_route(
            &network,
            vidhana,
            mg_road_start,
            RoutingPreference::ShortestDistance,
        )
        .expect("Connected pathfinding from Vidhana Soudha to MG Road must succeed");

        assert!(
            route.waypoints.len() >= 3,
            "Route should span multiple waypoints"
        );
        assert!(
            route.total_distance_meters > 1200.0,
            "Distance between Vidhana Soudha and MG Road should exceed 1.2km, got {}",
            route.total_distance_meters
        );
    }

    #[test]
    fn test_turn_by_turn_guidance_and_wrong_way_tracking() {
        // Build an L-shaped route: Eastbound then Northbound (Right turn)
        let w0 = Wgs84Coord::new(37.7900, -122.4000, 5.0);
        let w1 = Wgs84Coord::new(37.7900, -122.3950, 5.0); // ~440m East
        let w2 = Wgs84Coord::new(37.7950, -122.3950, 5.0); // ~555m North

        let route = NavigationRoute {
            waypoints: vec![w0, w1, w2],
            total_distance_meters: 995.0,
            estimated_duration_seconds: 70.0,
            edge_way_ids: vec![101, 102],
            road_names: vec!["Market Street".to_string(), "The Embarcadero".to_string()],
        };

        let dummy_network = RoadNetworkGraph::new();
        let guidance = generate_turn_by_turn(&route, &dummy_network);

        assert_eq!(guidance.instructions.len(), 3);
        assert_eq!(guidance.instructions[0].maneuver, ManeuverType::Depart);
        assert!(guidance.instructions[0]
            .instruction
            .contains("Market Street"));
        // Bearing from East (90 deg) to North (0 deg) is -90 deg (Turn Left)
        assert_eq!(guidance.instructions[1].maneuver, ManeuverType::TurnLeft);
        assert!(guidance.instructions[1]
            .instruction
            .contains("The Embarcadero"));
        assert_eq!(guidance.instructions[2].maneuver, ManeuverType::Arrive);

        let mut tracker = RouteTracker::new(route, guidance);

        // 1. Driving correctly along segment 0 heading East (~90 deg)
        let pos1 = Wgs84Coord::new(37.7900, -122.3980, 5.0);
        let update1 = tracker.update(pos1, 90.0);
        assert_eq!(update1.status, NavigationStatus::EnRoute);
        assert!(!update1.is_wrong_way);
        assert!(!update1.is_off_route);
        assert!(update1.cross_track_error_meters < 1.0);

        // 2. Driving Wrong-Way (heading West 270 deg on Eastbound route)
        let update_wrong = tracker.update(pos1, 270.0);
        assert_eq!(update_wrong.status, NavigationStatus::WrongWayWarning);
        assert!(update_wrong.is_wrong_way);

        // 3. Driving Off-Route (> 35m perpendicular offset)
        let pos_off = Wgs84Coord::new(37.7908, -122.3980, 5.0); // ~88 meters North of Market St
        let update_off = tracker.update(pos_off, 90.0);
        assert_eq!(update_off.status, NavigationStatus::OffRouteRerouteRequired);
        assert!(update_off.is_off_route);

        // 4. Arrived at destination
        let update_arr = tracker.update(w2, 0.0);
        assert_eq!(update_arr.status, NavigationStatus::Arrived);
    }

    #[test]
    fn test_grade_separated_layer_node_isolation() {
        let mut network = RoadNetworkGraph::new();

        // Ground road on layer 0 crossing from (37.7800, -122.4000) to (37.7800, -122.3900)
        let ground_road = OsmRoadFeature {
            way_id: 201,
            name: Some("Howard Street".to_string()),
            highway_type: HighwayType::Primary,
            geometry: vec![
                Wgs84Coord::new(37.7800, -122.4000, 0.0),
                Wgs84Coord::new(37.7800, -122.3950, 0.0),
                Wgs84Coord::new(37.7800, -122.3900, 0.0),
            ],
            lanes: 4,
            speed_limit_kph: 45,
            is_oneway: false,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
        };

        // Elevated freeway on layer 1 crossing from (37.7750, -122.3950) to (37.7850, -122.3950)
        // Passes through the same lat/lon (37.7800, -122.3950) as ground road!
        let elevated_flyover = OsmRoadFeature {
            way_id: 202,
            name: Some("I-80 Skyway".to_string()),
            highway_type: HighwayType::Motorway,
            geometry: vec![
                Wgs84Coord::new(37.7750, -122.3950, 20.0),
                Wgs84Coord::new(37.7800, -122.3950, 20.0),
                Wgs84Coord::new(37.7850, -122.3950, 20.0),
            ],
            lanes: 6,
            speed_limit_kph: 100,
            is_oneway: true,
            layer: 1,
            is_bridge: true,
            is_tunnel: false,
        };

        network.add_road(&ground_road);
        network.add_road(&elevated_flyover);

        // Nodes for ground road vs flyover at (37.7800, -122.3950) must be distinct!
        let node_ground = network
            .find_nearest_node_with_layer(Wgs84Coord::new(37.7800, -122.3950, 0.0), Some(0))
            .expect("Ground node must exist");
        let node_flyover = network
            .find_nearest_node_with_layer(Wgs84Coord::new(37.7800, -122.3950, 20.0), Some(1))
            .expect("Flyover node must exist");

        assert_ne!(
            node_ground, node_flyover,
            "Layer 0 ground street and Layer 1 flyover must have isolated nodes at crossing"
        );

        // No direct route should exist from ground street to elevated skyway
        let start_ground = Wgs84Coord::new(37.7800, -122.4000, 0.0);
        let end_skyway = Wgs84Coord::new(37.7850, -122.3950, 20.0);
        let route_result = find_route(
            &network,
            start_ground,
            end_skyway,
            RoutingPreference::ShortestDistance,
        );

        assert!(
            route_result.is_err(),
            "Vehicles cannot jump from ground road to grade-separated flyover without a ramp"
        );
    }
}
