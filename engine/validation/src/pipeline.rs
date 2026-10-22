//! End-to-end dataset validation and controlled repair pipeline.

use crate::aeroway_port_validator::{validate_aeroways, validate_ports};
use crate::building_deep_validator::validate_buildings_deep;
use crate::chunk_integrity_validator::validate_chunk_boundary_continuity;
use crate::diagnostic::ValidationTolerance;
use crate::polygon_validator::ValidatedBuilding;
use crate::railway_validator::validate_railway_network;
use crate::repair_engine::RepairEngine;
use crate::report::{QuarantineLog, RepairLog, RepairType, ValidationReport};
use crate::road_network_validator::validate_road_network;
use crate::terrain_water_validator::{
    validate_dem_continuity, validate_water_bodies, validate_water_building_conflicts,
};
use realearth_ingestion::osm_types::{
    IngestedDataset, OsmAerowayFeature, OsmLandCoverFeature, OsmPortFeature, OsmRailwayFeature,
    OsmRoadFeature, OsmWaterFeature,
};
use realearth_ingestion::DemGrid;

/// Fully validated and sanitized dataset ready for the world compiler.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedDataset {
    pub roads: Vec<OsmRoadFeature>,
    pub buildings: Vec<ValidatedBuilding>,
    pub water_bodies: Vec<OsmWaterFeature>,
    pub land_cover: Vec<OsmLandCoverFeature>,
    pub railways: Vec<OsmRailwayFeature>,
    pub aeroways: Vec<OsmAerowayFeature>,
    pub ports: Vec<OsmPortFeature>,
}

/// Runs the validation and repair pipeline with default surveyed tolerances.
pub fn validate_dataset(dataset: &IngestedDataset) -> (ValidatedDataset, ValidationReport) {
    validate_dataset_with_tolerances(dataset, &ValidationTolerance::default(), None)
}

/// Runs the complete deep validation, controlled repair, and quarantine pipeline.
pub fn validate_dataset_with_tolerances(
    dataset: &IngestedDataset,
    tolerance: &ValidationTolerance,
    dem_grid: Option<&DemGrid>,
) -> (ValidatedDataset, ValidationReport) {
    let mut repair_engine = RepairEngine::new();

    // 1. Roads
    let valid_roads = validate_road_network(&dataset.roads, &mut repair_engine, tolerance);

    // 2. Chunk integrity on roads
    validate_chunk_boundary_continuity(&valid_roads, 500.0, &mut repair_engine, tolerance);

    // 3. Buildings
    let valid_buildings =
        validate_buildings_deep(&dataset.buildings, &mut repair_engine, tolerance, dem_grid);

    // 4. Railways
    let valid_railways = validate_railway_network(
        &dataset.railways,
        &valid_roads,
        &mut repair_engine,
        tolerance,
    );

    // 5. Water bodies
    let valid_water = validate_water_bodies(&dataset.water_bodies, &mut repair_engine, tolerance);

    // 5b. Water / Building conflicts
    validate_water_building_conflicts(&valid_buildings, &valid_water, &mut repair_engine);

    // 6. Aeroways
    let valid_aeroways = validate_aeroways(&dataset.aeroways, &mut repair_engine, tolerance);

    // 7. Ports
    let valid_ports = validate_ports(&dataset.ports, &mut repair_engine, tolerance);

    // 8. DEM continuity (if DEM provided)
    if let Some(dem) = dem_grid {
        validate_dem_continuity(dem, &mut repair_engine, tolerance);
    }

    // Translate controlled repairs into backward-compatible RepairLogs
    let legacy_repairs: Vec<RepairLog> = repair_engine
        .applied_repairs
        .iter()
        .map(|cr| {
            let r_type = match cr.rule_id.as_str() {
                "RULE_CLOSE_OPEN_RING" => RepairType::UnclosedRingClosed,
                "RULE_REVERSE_CW_WINDING" => RepairType::InvertedWindingOrderReversed,
                "RULE_CULL_DUPLICATE_VERTICES" => RepairType::DuplicateVerticesCulled,
                "RULE_INFER_HEIGHT_FROM_LEVELS" => RepairType::HeightInferredFromLevels,
                "RULE_DEFAULT_CATEGORY_HEIGHT" => RepairType::HeightAssignedDefaultCategory,
                "RULE_DEFAULT_LANE_COUNT" => RepairType::LaneCountAssignedDefault,
                "RULE_SPEED_LIMIT_SANITY" => RepairType::SpeedLimitNormalized,
                "RULE_RUNWAY_DEFAULT_WIDTH" => RepairType::RunwayWidthAssignedDefault,
                _ => RepairType::DuplicateVerticesCulled,
            };
            RepairLog {
                feature_id: cr.feature_id,
                repair_type: r_type,
                description: cr.description.clone(),
                confidence_score: cr.confidence_score,
            }
        })
        .collect();

    // Translate quarantined features into backward-compatible QuarantineLogs
    let legacy_quarantine: Vec<QuarantineLog> = repair_engine
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == crate::diagnostic::DiagnosticSeverity::Error
                || d.severity == crate::diagnostic::DiagnosticSeverity::Blocker
        })
        .filter_map(|d| {
            d.feature_id.map(|id| QuarantineLog {
                feature_id: id,
                failure_reason: d.message.clone(),
            })
        })
        .collect();

    let report = ValidationReport {
        total_roads_evaluated: dataset.roads.len(),
        valid_roads_passed: valid_roads.len(),
        total_buildings_evaluated: dataset.buildings.len(),
        valid_buildings_passed: valid_buildings.len(),
        total_railways_evaluated: dataset.railways.len(),
        valid_railways_passed: valid_railways.len(),
        total_aeroways_evaluated: dataset.aeroways.len(),
        valid_aeroways_passed: valid_aeroways.len(),
        total_ports_evaluated: dataset.ports.len(),
        valid_ports_passed: valid_ports.len(),
        repairs: legacy_repairs,
        controlled_repairs: repair_engine.applied_repairs,
        quarantined: legacy_quarantine,
        diagnostics: repair_engine.diagnostics,
    };

    let validated = ValidatedDataset {
        roads: valid_roads,
        buildings: valid_buildings,
        water_bodies: valid_water,
        land_cover: dataset.land_cover.clone(),
        railways: valid_railways,
        aeroways: valid_aeroways,
        ports: valid_ports,
    };

    (validated, report)
}
