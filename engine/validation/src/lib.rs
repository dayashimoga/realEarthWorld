//! Topological validation, geometric sanitization, and defect quarantine for Real Earth.

pub mod aeroway_port_validator;
pub mod building_deep_validator;
pub mod building_road_validator;
pub mod chunk_integrity_validator;
pub mod city_catalog_validator;
pub mod diagnostic;
pub mod pipeline;
pub mod polygon_validator;
pub mod provenance;
pub mod publication_gate;
pub mod railway_validator;
pub mod repair_engine;
pub mod report;
pub mod road_network_validator;
pub mod road_validator;
pub mod scene_placement_auditor;
pub mod terrain_water_validator;

pub use aeroway_port_validator::{validate_aeroways, validate_ports};
pub use building_deep_validator::validate_buildings_deep;
pub use building_road_validator::{
    validate_building_infrastructure_conflicts, validate_building_railway_conflicts,
    validate_building_road_conflicts,
};
pub use chunk_integrity_validator::validate_chunk_boundary_continuity;
pub use city_catalog_validator::{
    audit_all_us_cities, validate_and_certify_us_city, CatalogAuditSummary,
};
pub use diagnostic::{DiagnosticEntry, DiagnosticOverlay, DiagnosticSeverity, ValidationTolerance};
pub use pipeline::{validate_dataset, validate_dataset_with_tolerances, ValidatedDataset};
pub use polygon_validator::{approximate_metric_area_m2, signed_ring_area, ValidatedBuilding};
pub use provenance::{HeightProvenance, ValidatedHeight};
pub use publication_gate::{CompilerPublicationGate, PublicationVerdict};
pub use railway_validator::validate_railway_network;
pub use repair_engine::{ControlledRepair, GeometrySnapshot, RepairEngine};
pub use report::{QuarantineLog, RepairLog, RepairType, ValidationReport};
pub use road_network_validator::{validate_road_network, verify_grade_separations};
pub use scene_placement_auditor::{
    audit_scene_placement, repair_scene_placement, DefectSeverity, PlacementAuditReport,
    PlacementDefect,
};
pub use terrain_water_validator::{validate_dem_continuity, validate_water_bodies};

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_ingestion::osm_types::{
        AerowayType, BuildingType, HighwayType, IngestedDataset, OsmAerowayFeature,
        OsmBuildingFeature, OsmPortFeature, OsmRoadFeature, PortType,
    };
    use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
    use realearth_ingestion::sample_san_francisco::create_san_francisco_dataset;
    use realearth_shared::coordinates::Wgs84Coord;

    #[test]
    fn test_bengaluru_central_validation() {
        let dataset = create_bengaluru_central_dataset();
        let (validated, report) = validate_dataset(&dataset);

        assert_eq!(validated.roads.len(), 7);
        assert_eq!(validated.buildings.len(), 4);
        assert!(
            report.is_clean(),
            "Expected clean validation with 0 quarantined features: {:?}",
            report.quarantined
        );

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

        let salesforce = validated
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Salesforce Tower"))
            .expect("Salesforce Tower missing");
        assert_eq!(salesforce.height.height_meters, 326.0);
        assert_eq!(salesforce.height.provenance, HeightProvenance::Surveyed);
        assert_eq!(salesforce.height.confidence, 1.0);

        let transamerica = validated
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Transamerica Pyramid"))
            .expect("Transamerica missing");
        assert_eq!(transamerica.height.height_meters, 260.0);
        assert_eq!(transamerica.height.provenance, HeightProvenance::Surveyed);

        let bay_bridge = validated
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Interstate 80 Bay Bridge Approach"))
            .expect("Bay Bridge approach missing");
        assert_eq!(bay_bridge.layer, 1);
        assert!(bay_bridge.is_bridge);
        assert_eq!(bay_bridge.lanes, 5);

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
        dataset.buildings.push(OsmBuildingFeature {
            id: 999,
            name: Some("Unclosed Building".to_string()),
            building_type: BuildingType::Residential,
            outer_ring: vec![
                Wgs84Coord::new(12.970, 77.600, 0.0),
                Wgs84Coord::new(12.971, 77.600, 0.0),
                Wgs84Coord::new(12.971, 77.601, 0.0),
                Wgs84Coord::new(12.970, 77.601, 0.0),
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

        let repair = report
            .repairs
            .iter()
            .find(|r| r.feature_id == 999 && r.repair_type == RepairType::UnclosedRingClosed);
        assert!(repair.is_some(), "Expected unclosed ring repair log");
    }

    #[test]
    fn test_polygon_degenerate_quarantine() {
        let mut dataset = IngestedDataset::default();
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
        assert!(!report.quarantined.is_empty());
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
                Wgs84Coord::new(12.970, 77.600, 0.0),
                Wgs84Coord::new(12.971, 77.600, 0.0),
                Wgs84Coord::new(12.971, 77.600, 0.0),
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

    #[test]
    fn test_self_intersecting_bowtie_polygon_quarantine() {
        let mut dataset = IngestedDataset::default();
        // Hourglass / bow-tie self-intersecting polygon
        dataset.buildings.push(OsmBuildingFeature {
            id: 666,
            name: Some("Bowtie Building".to_string()),
            building_type: BuildingType::Commercial,
            outer_ring: vec![
                Wgs84Coord::new(12.970, 77.600, 0.0),
                Wgs84Coord::new(12.971, 77.601, 0.0),
                Wgs84Coord::new(12.970, 77.601, 0.0),
                Wgs84Coord::new(12.971, 77.600, 0.0),
                Wgs84Coord::new(12.970, 77.600, 0.0),
            ],
            inner_rings: Vec::new(),
            explicit_height: Some(20.0),
            levels: None,
        });

        let (validated, report) = validate_dataset(&dataset);
        assert_eq!(
            validated.buildings.len(),
            0,
            "Self-intersecting polygon must be quarantined"
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "RULE_SELF_INTERSECTING_BOWTIE"));
    }

    #[test]
    fn test_compiler_publication_gate_and_reports() {
        let dataset = create_bengaluru_central_dataset();
        let (_validated, report) = validate_dataset(&dataset);

        let gate = CompilerPublicationGate::production();
        let verdict = gate.evaluate(&report);
        assert!(
            verdict.is_approved(),
            "Clean Bengaluru dataset must pass production gate"
        );

        // Verify JSON, Markdown, HTML, and Overlay report outputs
        let json = report.to_json();
        assert!(json.contains("valid_roads_passed"));

        let md = report.to_markdown();
        assert!(md.contains("Comprehensive GIS Validation & Audit Report"));

        let html = report.to_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("GIS Validation Dashboard"));

        let overlays = report.to_overlay();
        assert_eq!(
            overlays.len(),
            report
                .diagnostics
                .iter()
                .filter(|d| d.location.is_some())
                .count()
        );
    }

    #[test]
    fn test_aeroway_and_port_validation() {
        let mut dataset = IngestedDataset::default();
        dataset.aeroways.push(OsmAerowayFeature {
            id: 101,
            name: Some("Short Runway".to_string()),
            aeroway_type: AerowayType::Runway,
            geometry: vec![
                Wgs84Coord::new(37.618, -122.375, 0.0),
                Wgs84Coord::new(37.619, -122.375, 0.0), // ~111m < 300m threshold
            ],
            width_m: Some(10.0), // Below 15m threshold
            ref_code: Some("28L".to_string()),
        });

        dataset.ports.push(OsmPortFeature {
            id: 202,
            name: Some("Ferry Pier".to_string()),
            port_type: PortType::Pier,
            polygon: vec![
                Wgs84Coord::new(37.795, -122.393, 0.0),
                Wgs84Coord::new(37.796, -122.393, 0.0),
                Wgs84Coord::new(37.796, -122.392, 0.0),
                // Open polygon
            ],
            min_depth_m: Some(8.0),
        });

        let (validated, report) = validate_dataset(&dataset);
        assert_eq!(validated.aeroways.len(), 1);
        assert_eq!(validated.aeroways[0].width_m, Some(45.0)); // Default repaired width
        assert_eq!(validated.ports.len(), 1);
        assert_eq!(validated.ports[0].polygon.len(), 4); // Closed ring repair

        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "RULE_RUNWAY_LENGTH_TOO_SHORT"));
    }

    #[test]
    fn test_road_validator_direct_cases() {
        let mut repairs = Vec::new();
        let mut quarantined = Vec::new();

        // 1. Quarantined for < 2 points
        let bad_road = OsmRoadFeature {
            way_id: 1,
            name: None,
            highway_type: HighwayType::Residential,
            geometry: vec![Wgs84Coord::new(10.0, 20.0, 0.0)],
            lanes: 1,
            speed_limit_kph: 30,
            is_oneway: false,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
        };
        assert!(
            crate::road_validator::validate_road(&bad_road, &mut repairs, &mut quarantined)
                .is_none()
        );
        assert_eq!(quarantined.len(), 1);

        // 2. Duplicate points culled
        let jittery_road = OsmRoadFeature {
            way_id: 2,
            name: None,
            highway_type: HighwayType::Residential,
            geometry: vec![
                Wgs84Coord::new(10.0, 20.0, 0.0),
                Wgs84Coord::new(10.0, 20.0, 0.0),
                Wgs84Coord::new(10.001, 20.0, 0.0),
            ],
            lanes: 1,
            speed_limit_kph: 30,
            is_oneway: false,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
        };
        let valid_res =
            crate::road_validator::validate_road(&jittery_road, &mut repairs, &mut quarantined);
        assert!(valid_res.is_some());
        assert_eq!(valid_res.unwrap().geometry.len(), 2);

        // 3. Grade separation check
        let r1 = OsmRoadFeature {
            way_id: 10,
            name: None,
            highway_type: HighwayType::Motorway,
            geometry: vec![
                Wgs84Coord::new(10.0, 20.0, 10.0),
                Wgs84Coord::new(10.002, 20.0, 10.0),
            ],
            lanes: 2,
            speed_limit_kph: 100,
            is_oneway: false,
            layer: 1,
            is_bridge: true,
            is_tunnel: false,
        };
        let r2 = OsmRoadFeature {
            way_id: 11,
            name: None,
            highway_type: HighwayType::Secondary,
            geometry: vec![
                Wgs84Coord::new(10.001, 19.999, 0.0),
                Wgs84Coord::new(10.001, 20.001, 0.0),
            ],
            lanes: 2,
            speed_limit_kph: 50,
            is_oneway: false,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
        };
        let mut grade_quarantined = Vec::new();
        assert!(crate::road_validator::verify_grade_separations(
            &[r1, r2],
            &mut grade_quarantined
        ));
    }

    #[test]
    fn test_polygon_validator_direct_cases() {
        let mut repairs = Vec::new();
        let mut quarantined = Vec::new();

        // 1. Area calculation
        let square = vec![
            Wgs84Coord::new(37.77, -122.42, 0.0),
            Wgs84Coord::new(37.771, -122.42, 0.0),
            Wgs84Coord::new(37.771, -122.419, 0.0),
            Wgs84Coord::new(37.77, -122.419, 0.0),
            Wgs84Coord::new(37.77, -122.42, 0.0),
        ];
        let area = crate::polygon_validator::approximate_metric_area_m2(&square);
        assert!(area > 1000.0);

        // 2. Validate building with winding order repair and levels attribution
        let building = OsmBuildingFeature {
            id: 501,
            name: Some("Test Building".to_string()),
            building_type: BuildingType::Residential,
            outer_ring: square,
            inner_rings: Vec::new(),
            explicit_height: None,
            levels: Some(4),
        };
        let validated =
            crate::polygon_validator::validate_building(&building, &mut repairs, &mut quarantined);
        assert!(validated.is_some());
        let val_b = validated.unwrap();
        assert_eq!(val_b.height.height_meters, 15.0); // 4 levels * 3.5m + 1.0m ground floor
    }
}
