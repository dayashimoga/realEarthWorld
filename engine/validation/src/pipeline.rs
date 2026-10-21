//! End-to-end dataset validation pipeline.

use crate::polygon_validator::{validate_building, ValidatedBuilding};
use crate::report::ValidationReport;
use crate::road_validator::{validate_road, verify_grade_separations};
use realearth_ingestion::osm_types::{
    IngestedDataset, OsmLandCoverFeature, OsmRoadFeature, OsmWaterFeature,
};

/// Fully validated and sanitized dataset ready for the world compiler.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedDataset {
    pub roads: Vec<OsmRoadFeature>,
    pub buildings: Vec<ValidatedBuilding>,
    pub water_bodies: Vec<OsmWaterFeature>,
    pub land_cover: Vec<OsmLandCoverFeature>,
}

/// Runs the complete validation, repair, and quarantine pipeline on an ingested dataset.
pub fn validate_dataset(dataset: &IngestedDataset) -> (ValidatedDataset, ValidationReport) {
    let mut report = ValidationReport {
        total_roads_evaluated: dataset.roads.len(),
        total_buildings_evaluated: dataset.buildings.len(),
        ..Default::default()
    };

    let mut valid_roads = Vec::with_capacity(dataset.roads.len());
    for r in &dataset.roads {
        if let Some(clean_r) = validate_road(r, &mut report.repairs, &mut report.quarantined) {
            valid_roads.push(clean_r);
        }
    }
    report.valid_roads_passed = valid_roads.len();

    // Verify grade-separation invariants
    verify_grade_separations(&valid_roads, &mut report.quarantined);

    let mut valid_buildings = Vec::with_capacity(dataset.buildings.len());
    for b in &dataset.buildings {
        if let Some(clean_b) = validate_building(b, &mut report.repairs, &mut report.quarantined) {
            valid_buildings.push(clean_b);
        }
    }
    report.valid_buildings_passed = valid_buildings.len();

    let validated = ValidatedDataset {
        roads: valid_roads,
        buildings: valid_buildings,
        water_bodies: dataset.water_bodies.clone(),
        land_cover: dataset.land_cover.clone(),
    };

    (validated, report)
}
