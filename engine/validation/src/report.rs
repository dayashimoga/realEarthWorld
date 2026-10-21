//! Validation report, multi-format diagnostics export, and audit trails.

use crate::diagnostic::{DiagnosticEntry, DiagnosticOverlay, DiagnosticSeverity};
use crate::repair_engine::ControlledRepair;
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
    LaneCountAssignedDefault,
    SpeedLimitNormalized,
    RunwayWidthAssignedDefault,
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

/// Comprehensive validation report for a dataset with multi-format export.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub total_roads_evaluated: usize,
    pub valid_roads_passed: usize,
    pub total_buildings_evaluated: usize,
    pub valid_buildings_passed: usize,
    pub total_railways_evaluated: usize,
    pub valid_railways_passed: usize,
    pub total_aeroways_evaluated: usize,
    pub valid_aeroways_passed: usize,
    pub total_ports_evaluated: usize,
    pub valid_ports_passed: usize,
    pub repairs: Vec<RepairLog>,
    pub controlled_repairs: Vec<ControlledRepair>,
    pub quarantined: Vec<QuarantineLog>,
    pub diagnostics: Vec<DiagnosticEntry>,
}

impl ValidationReport {
    pub fn is_clean(&self) -> bool {
        self.quarantined.is_empty()
            && !self.diagnostics.iter().any(|d| {
                d.severity == DiagnosticSeverity::Blocker || d.severity == DiagnosticSeverity::Error
            })
    }

    pub fn summary(&self) -> String {
        format!(
            "Validation Summary: Roads: {}/{} valid. Buildings: {}/{} valid. Railways: {}/{} valid. Aeroways: {}/{} valid. Ports: {}/{} valid. Repairs: {}. Quarantined: {}. Total Diagnostics: {}.",
            self.valid_roads_passed,
            self.total_roads_evaluated,
            self.valid_buildings_passed,
            self.total_buildings_evaluated,
            self.valid_railways_passed,
            self.total_railways_evaluated,
            self.valid_aeroways_passed,
            self.total_aeroways_evaluated,
            self.valid_ports_passed,
            self.total_ports_evaluated,
            self.repairs.len(),
            self.quarantined.len(),
            self.diagnostics.len()
        )
    }

    /// Serializes report to a formatted JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generates a formatted GitHub-Flavored Markdown summary.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# Real Earth — Comprehensive GIS Validation & Audit Report\n\n");
        md.push_str(&format!(
            "**Status**: {}\n\n",
            if self.is_clean() {
                "PASS ✅"
            } else {
                "DEFECTS DETECTED ⚠️"
            }
        ));

        md.push_str("## Feature Evaluation Summary\n\n");
        md.push_str("| Layer | Evaluated | Passed | Quarantined |\n");
        md.push_str("| :--- | :--- | :--- | :--- |\n");
        md.push_str(&format!(
            "| Roads | {} | {} | {} |\n",
            self.total_roads_evaluated,
            self.valid_roads_passed,
            self.total_roads_evaluated
                .saturating_sub(self.valid_roads_passed)
        ));
        md.push_str(&format!(
            "| Buildings | {} | {} | {} |\n",
            self.total_buildings_evaluated,
            self.valid_buildings_passed,
            self.total_buildings_evaluated
                .saturating_sub(self.valid_buildings_passed)
        ));
        md.push_str(&format!(
            "| Railways | {} | {} | {} |\n",
            self.total_railways_evaluated,
            self.valid_railways_passed,
            self.total_railways_evaluated
                .saturating_sub(self.valid_railways_passed)
        ));
        md.push_str(&format!(
            "| Aeroways | {} | {} | {} |\n",
            self.total_aeroways_evaluated,
            self.valid_aeroways_passed,
            self.total_aeroways_evaluated
                .saturating_sub(self.valid_aeroways_passed)
        ));
        md.push_str(&format!(
            "| Ports | {} | {} | {} |\n\n",
            self.total_ports_evaluated,
            self.valid_ports_passed,
            self.total_ports_evaluated
                .saturating_sub(self.valid_ports_passed)
        ));

        md.push_str(&format!(
            "**Controlled Repairs Applied**: {}\n",
            self.repairs.len()
        ));
        md.push_str(&format!(
            "**Quarantined Ambiguous Features**: {}\n\n",
            self.quarantined.len()
        ));

        if !self.diagnostics.is_empty() {
            md.push_str("## Diagnostic Findings\n\n");
            md.push_str("| Severity | Rule / Code | Feature ID | Message |\n");
            md.push_str("| :--- | :--- | :--- | :--- |\n");
            for d in &self.diagnostics {
                md.push_str(&format!(
                    "| **{}** | `{}` | {} | {} |\n",
                    d.severity,
                    d.code,
                    d.feature_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    d.message
                ));
            }
            md.push('\n');
        }

        md
    }

    /// Generates a styled, standalone HTML dashboard.
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str(
            "<meta charset=\"UTF-8\">\n<title>GIS Validation Report - Real Earth</title>\n",
        );
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0d1117; color: #c9d1d9; padding: 24px; }\n");
        html.push_str("h1, h2 { color: #58a6ff; }\n");
        html.push_str(".card { background: #161b22; border: 1px solid #30363d; border-radius: 6px; padding: 16px; margin-bottom: 20px; }\n");
        html.push_str("table { width: 100%; border-collapse: collapse; margin-top: 10px; }\n");
        html.push_str(
            "th, td { border: 1px solid #30363d; padding: 8px 12px; text-align: left; }\n",
        );
        html.push_str("th { background: #21262d; color: #f0f6fc; }\n");
        html.push_str(".badge-blocker { background: #da3633; color: white; padding: 2px 6px; border-radius: 4px; font-weight: bold; }\n");
        html.push_str(".badge-error { background: #f85149; color: white; padding: 2px 6px; border-radius: 4px; }\n");
        html.push_str(".badge-warning { background: #d29922; color: black; padding: 2px 6px; border-radius: 4px; font-weight: bold; }\n");
        html.push_str(".badge-info { background: #1f6feb; color: white; padding: 2px 6px; border-radius: 4px; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str("<h1>Real Earth — GIS Validation Dashboard</h1>\n");
        html.push_str("<div class=\"card\">\n");
        html.push_str(&format!(
            "<h2>Overall Status: {}</h2>\n",
            if self.is_clean() {
                "<span style='color: #3fb950;'>ALL GATES PASSED</span>"
            } else {
                "<span style='color: #f85149;'>DEFECTS DETECTED</span>"
            }
        ));
        html.push_str(&format!("<p>{}</p>\n", self.summary()));
        html.push_str("</div>\n");

        html.push_str("<div class=\"card\">\n<h2>Diagnostics Table</h2>\n<table>\n");
        html.push_str(
            "<tr><th>Severity</th><th>Code</th><th>Feature ID</th><th>Description</th></tr>\n",
        );
        for d in &self.diagnostics {
            let badge_cls = match d.severity {
                DiagnosticSeverity::Blocker => "badge-blocker",
                DiagnosticSeverity::Error => "badge-error",
                DiagnosticSeverity::Warning => "badge-warning",
                DiagnosticSeverity::Info => "badge-info",
            };
            html.push_str(&format!(
                "<tr><td><span class=\"{}\">{}</span></td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                badge_cls,
                d.severity,
                d.code,
                d.feature_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                d.message
            ));
        }
        html.push_str("</table>\n</div>\n</body>\n</html>\n");

        html
    }

    /// Converts geolocated diagnostics into visual error overlays for the 3D viewer.
    pub fn to_overlay(&self) -> Vec<DiagnosticOverlay> {
        self.diagnostics
            .iter()
            .enumerate()
            .filter_map(|(idx, d)| DiagnosticOverlay::from_diagnostic(idx as u64, d))
            .collect()
    }
}
