//! Geospatial data ingestion for OpenStreetMap vector and Digital Elevation Model rasters.

pub mod dem_grid;
pub mod osm_parser;
pub mod osm_types;
pub mod sample_bengaluru;

pub use dem_grid::DemGrid;
pub use osm_parser::parse_osm_xml;
pub use osm_types::{
    BuildingType, HighwayType, IngestedDataset, OsmBuildingFeature, OsmRoadFeature, OsmWaterFeature,
};
pub use sample_bengaluru::{create_bengaluru_central_dataset, generate_bengaluru_osm_xml};

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_shared::coordinates::Wgs84Coord;

    #[test]
    fn test_bengaluru_osm_xml_parsing() {
        let xml = generate_bengaluru_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse generated Bengaluru OSM XML");

        assert_eq!(dataset.buildings.len(), 4, "Expected 4 landmark buildings");
        assert_eq!(dataset.roads.len(), 4, "Expected 4 road segments");

        // Verify Vidhana Soudha
        let vidhana = dataset
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Vidhana Soudha"))
            .expect("Vidhana Soudha not found");
        assert_eq!(vidhana.explicit_height, Some(46.0));
        assert_eq!(vidhana.levels, Some(4));
        assert_eq!(vidhana.outer_ring.len(), 5);

        // Verify Flyover Grade Separation
        let flyover = dataset
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Cubbon Park Elevated Flyover"))
            .expect("Flyover not found");
        assert_eq!(flyover.layer, 1);
        assert!(flyover.is_bridge);

        let ground_road = dataset
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Kasturba Road"))
            .expect("Kasturba Road not found");
        assert_eq!(ground_road.layer, 0);
        assert!(!ground_road.is_bridge);
    }

    #[test]
    fn test_dem_grid_elevation_sampling() {
        let dem = DemGrid::create_bengaluru_plateau_dem(64, 64);
        assert_eq!(dem.width, 64);
        assert_eq!(dem.height, 64);

        // Test center coordinate of Bengaluru Central
        let center = dem.bounds.center();
        let elev = dem.sample_bilinear(&center);

        // Bengaluru plateau elevation is in range 900m - 940m
        assert!(
            (900.0..=940.0).contains(&elev),
            "Elevation at Bengaluru center was unexpected: {} m",
            elev
        );
    }

    #[test]
    fn test_dem_bounds_clamping() {
        let dem = DemGrid::create_bengaluru_plateau_dem(32, 32);
        // Coordinate slightly outside bounds should clamp smoothly without panicking
        let outside = Wgs84Coord::new(12.9500, 77.5700, 0.0);
        let elev = dem.sample_bilinear(&outside);
        assert!(elev > 0.0);
    }
}
