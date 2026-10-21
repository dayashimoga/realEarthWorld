//! Validation report, audit trails, and quarantine logging models.

use serde::{Deserialize, Serialize};

/// Type of traceable geometric repair performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairType {
    UnclosedRingClosed,
    InvertedWindingOrderReversed,
    DuplicateVerticesCulled,
    CollinearVerticesSimplified,
    HeightInferredFromLevels,
    HeightAssignedDefaultCategory,
}

/// Audit record for a successfully repaired feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairLog {
    pub feature_id: u64,
    pub repair_type: RepairType,
    pub description: String,
    /// Confidence score of the repair from 0.0 (uncertain) to 1.0 (exact).
    pub confidence_score: f32,
}

/// Record for a defective feature that cannot be safely repaired and is isolated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuarantineLog {
    pub feature_id: u64,
    pub failure_reason: String,
}

/// Comprehensive validation report for a dataset.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub total_roads_evaluated: usize,
    pub valid_roads_passed: usize,
    pub total_buildings_evaluated: usize,
    pub valid_buildings_passed: usize,
    pub repairs: Vec<RepairLog>,
    pub quarantined: Vec<QuarantineLog>,
}

impl ValidationReport {
    pub fn is_clean(&self) -> bool {
        self.quarantined.is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "Validation Summary: Roads: {}/{} valid. Buildings: {}/{} valid. Repairs: {}. Quarantined: {}.",
            self.valid_roads_passed,
            self.total_roads_evaluated,
            self.valid_buildings_passed,
            self.total_buildings_evaluated,
            self.repairs.len(),
            self.quarantined.len()
        )
    }
}
