//! Topological validation, geometric sanitization, and defect quarantine for Real Earth.

pub mod pipeline;
pub mod polygon_validator;
pub mod provenance;
pub mod report;
pub mod road_validator;

pub use pipeline::{validate_dataset, ValidatedDataset};
pub use polygon_validator::{approximate_metric_area_m2, signed_ring_area, ValidatedBuilding};
pub use provenance::{HeightProvenance, ValidatedHeight};
pub use report::{QuarantineLog, RepairLog, RepairType, ValidationReport};
pub use road_validator::{validate_road, verify_grade_separations};

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_ingestion::osm_types::{
        BuildingType, HighwayType, IngestedDataset, OsmBuildingFeature, OsmRoadFeature,
    };
    use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
    use realearth_shared::coordinates::Wgs84Coord;

    #[test]
    fn test_bengaluru_central_validation() {
        let dataset = create_bengaluru_central_dataset();
        let (validated, report) = validate_dataset(&dataset);

        assert_eq!(validated.roads.len(), 4);
        assert_eq!(validated.buildings.len(), 4);
        assert!(
            report.is_clean(),
            "Expected clean validation with 0 quarantined features: {:?}",
            report.quarantined
        );

        // Check Vidhana Soudha surveyed height
        let vidhana = validated
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Vidhana Soudha"))
            .expect("Vidhana Soudha missing");
        assert_eq!(vidhana.height.provenance, HeightProvenance::Surveyed);
        assert_eq!(vidhana.height.height_meters, 46.0);
        assert_eq!(vidhana.height.confidence, 1.0);
        assert!(
            vidhana.area_m2 > 1000.0,
            "Vidhana Soudha footprint should be > 1000 m2"
        );

        // Verify grade separated flyover
        let flyover = validated
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Cubbon Park Elevated Flyover"))
            .expect("Flyover missing");
        assert_eq!(flyover.layer, 1);
        assert!(flyover.is_bridge);
    }

    #[test]
    fn test_polygon_unclosed_ring_repair() {
        let mut dataset = IngestedDataset::default();
        // Open ring (first != last)
        dataset.buildings.push(OsmBuildingFeature {
            id: 999,
            name: Some("Unclosed Building".to_string()),
            building_type: BuildingType::Residential,
            outer_ring: vec![
                Wgs84Coord::new(12.970, 77.600, 0.0),
                Wgs84Coord::new(12.971, 77.600, 0.0),
                Wgs84Coord::new(12.971, 77.601, 0.0),
                Wgs84Coord::new(12.970, 77.601, 0.0),
                // Notice missing closing point (12.970, 77.600)
            ],
            inner_rings: Vec::new(),
            explicit_height: None,
            levels: Some(3),
        });

        let (validated, report) = validate_dataset(&dataset);
        assert_eq!(validated.buildings.len(), 1);
        assert_eq!(validated.buildings[0].outer_ring.len(), 5);
        assert_eq!(
            validated.buildings[0].outer_ring[0],
            validated.buildings[0].outer_ring[4]
        );

        // Check repair log
        let repair = report
            .repairs
            .iter()
            .find(|r| r.feature_id == 999 && r.repair_type == RepairType::UnclosedRingClosed);
        assert!(repair.is_some(), "Expected unclosed ring repair log");
    }

    #[test]
    fn test_polygon_degenerate_quarantine() {
        let mut dataset = IngestedDataset::default();
        // 2 points only - impossible polygon
        dataset.buildings.push(OsmBuildingFeature {
            id: 888,
            name: Some("Degenerate Line Building".to_string()),
            building_type: BuildingType::Commercial,
            outer_ring: vec![
                Wgs84Coord::new(12.970, 77.600, 0.0),
                Wgs84Coord::new(12.971, 77.600, 0.0),
            ],
            inner_rings: Vec::new(),
            explicit_height: None,
            levels: None,
        });

        let (validated, report) = validate_dataset(&dataset);
        assert_eq!(validated.buildings.len(), 0);
        assert_eq!(report.quarantined.len(), 1);
        assert_eq!(report.quarantined[0].feature_id, 888);
    }

    #[test]
    fn test_road_duplicate_vertex_culling() {
        let mut dataset = IngestedDataset::default();
        dataset.roads.push(OsmRoadFeature {
            way_id: 777,
            name: Some("Jittery Road".to_string()),
            highway_type: HighwayType::Secondary,
            geometry: vec![
                Wgs84Coord::new(12.970, 77.600, 0.0),
                Wgs84Coord::new(12.970, 77.600, 0.0), // Duplicate point
                Wgs84Coord::new(12.971, 77.600, 0.0),
                Wgs84Coord::new(12.971, 77.600, 0.0), // Duplicate point
                Wgs84Coord::new(12.972, 77.600, 0.0),
            ],
            lanes: 2,
            speed_limit_kph: 40,
            is_oneway: false,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
        });

        let (validated, report) = validate_dataset(&dataset);
        assert_eq!(validated.roads.len(), 1);
        assert_eq!(validated.roads[0].geometry.len(), 3);

        let repair = report
            .repairs
            .iter()
            .find(|r| r.feature_id == 777 && r.repair_type == RepairType::DuplicateVerticesCulled);
        assert!(repair.is_some());
    }
}
