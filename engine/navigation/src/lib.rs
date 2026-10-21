//! Lane-level road network graph and A* routing engine for Real Earth.

pub mod road_graph;
pub mod router;

pub use road_graph::{LaneDescriptor, RoadEdge, RoadNetworkGraph, RoadNode, RuleOfTheRoad};
pub use router::{find_route, NavigationRoute, RoutingPreference};

#[cfg(test)]
mod tests {
    use super::*;
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
}
