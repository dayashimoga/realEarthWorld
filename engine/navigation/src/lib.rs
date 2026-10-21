//! Lane-level road network graph and A* routing engine for Real Earth.

pub mod road_graph;
pub mod router;

pub use road_graph::{RoadEdge, RoadNetworkGraph, RoadNode};
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
}
