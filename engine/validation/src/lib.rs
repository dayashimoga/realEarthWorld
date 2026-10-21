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
    fn test_san_francisco_dataset_validation() {
        use realearth_ingestion::sample_san_francisco::create_san_francisco_dataset;
        let dataset = create_san_francisco_dataset();
        let (validated, report) = validate_dataset(&dataset);

        assert_eq!(validated.roads.len(), 6);
        assert_eq!(validated.buildings.len(), 5);
        assert_eq!(validated.water_bodies.len(), 1);
        assert_eq!(validated.land_cover.len(), 2);
        assert_eq!(
            report.quarantined.len(),
            0,
            "Quarantined features found in SF dataset"
        );

        // Verify Salesforce Tower height provenance
        let salesforce = validated
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Salesforce Tower"))
            .expect("Salesforce Tower missing");
        assert_eq!(salesforce.height.height_meters, 326.0);
        assert_eq!(salesforce.height.provenance, HeightProvenance::Surveyed);
        assert_eq!(salesforce.height.confidence, 1.0);

        // Verify Transamerica Pyramid height provenance
        let transamerica = validated
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Transamerica Pyramid"))
            .expect("Transamerica missing");
        assert_eq!(transamerica.height.height_meters, 260.0);
        assert_eq!(transamerica.height.provenance, HeightProvenance::Surveyed);

        // Verify Bay Bridge I-80 Approach (elevated bridge)
        let bay_bridge = validated
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Interstate 80 Bay Bridge Approach"))
            .expect("Bay Bridge approach missing");
        assert_eq!(bay_bridge.layer, 1);
        assert!(bay_bridge.is_bridge);
        assert_eq!(bay_bridge.lanes, 5);

        // Verify Broadway Tunnel
        let tunnel = validated
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Broadway Tunnel"))
            .expect("Broadway Tunnel missing");
        assert_eq!(tunnel.layer, -1);
        assert!(tunnel.is_tunnel);
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
