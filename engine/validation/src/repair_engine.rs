//! Controlled, deterministic, and reversible geometric repair engine.

use crate::diagnostic::{DiagnosticEntry, DiagnosticSeverity, ValidationTolerance};
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Snapshot of feature geometry before or after a controlled repair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeometrySnapshot {
    Points(Vec<Wgs84Coord>),
    Rings {
        outer: Vec<Wgs84Coord>,
        inners: Vec<Vec<Wgs84Coord>>,
    },
    Scalar(f64),
    Attribute(String),
}

/// A structured, auditable, and reversible repair record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlledRepair {
    pub feature_id: u64,
    pub feature_kind: String,
    pub rule_id: String,
    pub description: String,
    pub confidence_score: f32,
    pub before_geometry: GeometrySnapshot,
    pub after_geometry: GeometrySnapshot,
    pub is_reversible: bool,
    pub provenance: String,
}

/// Transactional repair engine enforcing mathematical justification and quarantine.
#[derive(Debug, Default)]
pub struct RepairEngine {
    pub applied_repairs: Vec<ControlledRepair>,
    pub diagnostics: Vec<DiagnosticEntry>,
    pub quarantined_feature_ids: Vec<u64>,
}

impl RepairEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates whether a proposed repair satisfies the tolerance confidence threshold.
    /// If accepted, records the repair audit trail.
    /// If rejected, emits an Error diagnostic and flags the feature for quarantine.
    #[allow(clippy::too_many_arguments)]
    pub fn propose_repair(
        &mut self,
        feature_id: u64,
        feature_kind: &str,
        rule_id: &str,
        description: &str,
        confidence_score: f32,
        before: GeometrySnapshot,
        after: GeometrySnapshot,
        location: Option<Wgs84Coord>,
        tolerance: &ValidationTolerance,
    ) -> bool {
        if confidence_score >= tolerance.min_repair_confidence {
            self.applied_repairs.push(ControlledRepair {
                feature_id,
                feature_kind: feature_kind.to_string(),
                rule_id: rule_id.to_string(),
                description: description.to_string(),
                confidence_score,
                before_geometry: before,
                after_geometry: after,
                is_reversible: true,
                provenance: "Automatic Controlled Repair".to_string(),
            });

            self.diagnostics.push(DiagnosticEntry {
                code: format!("{}_REPAIRED", rule_id),
                severity: DiagnosticSeverity::Warning,
                feature_id: Some(feature_id),
                feature_type: Some(feature_kind.to_string()),
                location,
                message: format!(
                    "{} (Rule: {}, Confidence: {:.2})",
                    description, rule_id, confidence_score
                ),
                rule_id: Some(rule_id.to_string()),
                source_dataset: None,
            });

            true
        } else {
            self.quarantined_feature_ids.push(feature_id);
            self.diagnostics.push(DiagnosticEntry {
                code: format!("{}_REJECTED_LOW_CONFIDENCE", rule_id),
                severity: DiagnosticSeverity::Error,
                feature_id: Some(feature_id),
                feature_type: Some(feature_kind.to_string()),
                location,
                message: format!(
                    "Defect cannot be safely repaired (Confidence {:.2} < threshold {:.2}): {}",
                    confidence_score, tolerance.min_repair_confidence, description
                ),
                rule_id: Some(rule_id.to_string()),
                source_dataset: None,
            });

            false
        }
    }

    /// Records a non-repairable critical or blocking defect.
    pub fn record_quarantine(
        &mut self,
        feature_id: u64,
        feature_kind: &str,
        rule_id: &str,
        reason: &str,
        severity: DiagnosticSeverity,
        location: Option<Wgs84Coord>,
    ) {
        self.quarantined_feature_ids.push(feature_id);
        self.diagnostics.push(DiagnosticEntry {
            code: rule_id.to_string(),
            severity,
            feature_id: Some(feature_id),
            feature_type: Some(feature_kind.to_string()),
            location,
            message: reason.to_string(),
            rule_id: Some(rule_id.to_string()),
            source_dataset: None,
        });
    }

    /// Rolls back an applied repair if subsequent validation steps discover conflicts.
    pub fn rollback_last(&mut self) -> Option<ControlledRepair> {
        let rolled_back = self.applied_repairs.pop()?;
        self.diagnostics.retain(|d| {
            !(d.feature_id == Some(rolled_back.feature_id)
                && d.rule_id.as_deref() == Some(&rolled_back.rule_id))
        });
        Some(rolled_back)
    }
}
