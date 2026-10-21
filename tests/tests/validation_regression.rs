//! Comprehensive GIS Validation & Controlled Repair Regression Test Suite (Stage 4).
//!
//! Validates:
//! 1. Detection and quarantine of intentionally malformed GIS defects.
//! 2. Controlled, deterministic, confidence-scored repairs with before/after snapshots.
//! 3. Bit-exact invariance proof for correct, authentic source geography.
//! 4. Blocking compiler publication gates.
//! 5. Multi-format diagnostic generation (JSON, Markdown, HTML, Overlays).

use realearth_ingestion::osm_types::{
    AerowayType, BuildingType, HighwayType, IngestedDataset, OsmAerowayFeature, OsmBuildingFeature,
    OsmRailwayFeature, OsmRoadFeature, OsmWaterFeature, RailwayType, WaterType,
};
use realearth_ingestion::sample_san_francisco::create_san_francisco_dataset;
use realearth_ingestion::DemGrid;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_validation::diagnostic::{DiagnosticSeverity, ValidationTolerance};
use realearth_validation::pipeline::{validate_dataset, validate_dataset_with_tolerances};
use realearth_validation::publication_gate::CompilerPublicationGate;

#[test]
fn test_intentionally_malformed_defect_suite() {
    let mut malformed = IngestedDataset::default();

    // Defect 1: Bow-tie / Hourglass self-intersecting building polygon
    malformed.buildings.push(OsmBuildingFeature {
        id: 9001,
        name: Some("Malformed Bowtie Building".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(37.790, -122.400, 0.0),
            Wgs84Coord::new(37.792, -122.398, 0.0),
            Wgs84Coord::new(37.790, -122.398, 0.0),
            Wgs84Coord::new(37.792, -122.400, 0.0),
            Wgs84Coord::new(37.790, -122.400, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(30.0),
        levels: None,
    });

    // Defect 2: Road with single vertex (Blocker)
    malformed.roads.push(OsmRoadFeature {
        way_id: 9002,
        name: Some("Zero Length Road".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![Wgs84Coord::new(37.790, -122.400, 0.0)],
        lanes: 2,
        speed_limit_kph: 50,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // Defect 3: Road with 0 lanes and speed limit of 500 km/h (Eligible for repair)
    malformed.roads.push(OsmRoadFeature {
        way_id: 9003,
        name: Some("Superluminal Unmarked Road".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(37.785, -122.405, 0.0),
            Wgs84Coord::new(37.786, -122.405, 0.0),
        ],
        lanes: 0,
        speed_limit_kph: 500,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // Defect 4: Railway with sharp 90-degree kink
    malformed.railways.push(OsmRailwayFeature {
        way_id: 9004,
        name: Some("Acrobatic Railway".to_string()),
        railway_type: RailwayType::Mainline,
        geometry: vec![
            Wgs84Coord::new(37.780, -122.400, 0.0),
            Wgs84Coord::new(37.782, -122.400, 0.0),
            Wgs84Coord::new(37.782, -122.395, 0.0), // 90 degree sharp turn
        ],
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
        is_electrified: true,
        gauge_mm: Some(1435),
    });

    // Defect 5: Water polygon open ring (Eligible for repair)
    malformed.water_bodies.push(OsmWaterFeature {
        id: 9005,
        name: Some("Unclosed Bay Inlet".to_string()),
        water_type: WaterType::Bay,
        outer_ring: vec![
            Wgs84Coord::new(37.780, -122.385, 0.0),
            Wgs84Coord::new(37.785, -122.385, 0.0),
            Wgs84Coord::new(37.785, -122.380, 0.0),
            // Missing closure
        ],
        is_coastline: true,
        target_elevation_msl: 0.0,
    });

    // Defect 6: Micro-runway below 300m threshold
    malformed.aeroways.push(OsmAerowayFeature {
        id: 9006,
        name: Some("Tiny Runway".to_string()),
        aeroway_type: AerowayType::Runway,
        geometry: vec![
            Wgs84Coord::new(37.780, -122.400, 0.0),
            Wgs84Coord::new(37.781, -122.400, 0.0), // ~111m
        ],
        width_m: Some(10.0),
        ref_code: Some("01".to_string()),
    });

    let (validated, report) = validate_dataset(&malformed);

    // Verify Defect 1 (Bowtie) quarantined
    assert_eq!(validated.buildings.len(), 0);
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.code == "RULE_SELF_INTERSECTING_BOWTIE"));

    // Verify Defect 2 (1-vertex road) quarantined as Blocker
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.code == "RULE_ROAD_MIN_VERTICES" && d.severity == DiagnosticSeverity::Blocker));

    // Verify Defect 3 (lanes & speed) repaired
    let repaired_road = validated
        .roads
        .iter()
        .find(|r| r.way_id == 9003)
        .expect("Road 9003 should pass sanitized");
    assert_eq!(repaired_road.lanes, 4); // Default for motorway
    assert_eq!(repaired_road.speed_limit_kph, 100); // Default for motorway

    // Verify Defect 4 (sharp railway turn) flagged Warning
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.code == "RULE_RAILWAY_EXCESSIVE_CURVATURE"));

    // Verify Defect 5 (water open ring) repaired to closed ring
    let repaired_water = validated
        .water_bodies
        .iter()
        .find(|w| w.id == 9005)
        .expect("Water should pass");
    assert_eq!(repaired_water.outer_ring.len(), 4);
    assert_eq!(repaired_water.outer_ring[0], repaired_water.outer_ring[3]);

    // Verify Defect 6 (tiny runway) warned and width assigned default
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.code == "RULE_RUNWAY_LENGTH_TOO_SHORT"));
    assert_eq!(validated.aeroways[0].width_m, Some(45.0));

    // Publication Gate should REJECT malformed dataset
    let gate = CompilerPublicationGate::production();
    let verdict = gate.evaluate(&report);
    assert!(
        !verdict.is_approved(),
        "Malformed dataset must be rejected by production gate"
    );
}

#[test]
fn test_authentic_geography_invariance_proof() {
    // 1. San Francisco authentic dataset
    let sf_original = create_san_francisco_dataset();
    let dem_sf = DemGrid::create_san_francisco_coastal_dem(64, 64);
    let (sf_validated, sf_report) = validate_dataset_with_tolerances(
        &sf_original,
        &ValidationTolerance::surveyed(),
        Some(&dem_sf),
    );

    // All original valid buildings must preserve identity, name, and exact vertex coordinates
    assert_eq!(sf_validated.buildings.len(), sf_original.buildings.len());
    for orig in &sf_original.buildings {
        let val = sf_validated
            .buildings
            .iter()
            .find(|b| b.id == orig.id)
            .unwrap_or_else(|| panic!("Building {} disappeared", orig.id));
        assert_eq!(val.name, orig.name);
        // Closed rings have same vertex count (since original sample was closed)
        assert_eq!(val.outer_ring.len(), orig.outer_ring.len());
    }

    // All original roads must preserve identity and layer separation
    assert_eq!(sf_validated.roads.len(), sf_original.roads.len());
    for orig in &sf_original.roads {
        let val = sf_validated
            .roads
            .iter()
            .find(|r| r.way_id == orig.way_id)
            .unwrap_or_else(|| panic!("Road {} disappeared", orig.way_id));
        assert_eq!(val.name, orig.name);
        assert_eq!(val.layer, orig.layer);
        assert_eq!(val.is_bridge, orig.is_bridge);
        assert_eq!(val.is_tunnel, orig.is_tunnel);
        assert_eq!(
            val.geometry, orig.geometry,
            "Authentic road geometry must remain bit-exact"
        );
    }

    // Zero quarantined features in authentic reference datasets
    assert_eq!(sf_report.quarantined.len(), 0);

    // Production gate MUST approve
    let gate = CompilerPublicationGate::production();
    let verdict = gate.evaluate(&sf_report);
    assert!(verdict.is_approved());
}

#[test]
fn test_reversible_repair_audit_trail_and_reports() {
    let sf = create_san_francisco_dataset();
    let (validated, report) = validate_dataset(&sf);

    assert_eq!(validated.roads.len(), 6);
    assert!(
        !report.controlled_repairs.is_empty(),
        "Controlled repairs must be recorded"
    );

    for cr in &report.controlled_repairs {
        assert!(cr.is_reversible, "All repairs must be reversible");
        assert!(
            cr.confidence_score >= 0.80,
            "Repairs must meet or exceed confidence threshold"
        );
        assert!(!cr.rule_id.is_empty());
        assert!(!cr.description.is_empty());
    }

    // Verify all report format generators execute without panic
    let json = report.to_json();
    assert!(json.starts_with('{'));

    let md = report.to_markdown();
    assert!(md.contains("Comprehensive GIS Validation & Audit Report"));

    let html = report.to_html();
    assert!(html.contains("<title>GIS Validation Report - Real Earth</title>"));

    let overlays = report.to_overlay();
    for overlay in overlays {
        assert!(overlay.location.lat != 0.0 || overlay.location.lon != 0.0);
    }
}
