//! Blocking compiler publication gates.

use crate::diagnostic::DiagnosticSeverity;
use crate::report::ValidationReport;
use serde::{Deserialize, Serialize};

/// Publication verdict returned by the compiler publication gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicationVerdict {
    Approved {
        total_features: usize,
        repairs_applied: usize,
    },
    Rejected {
        blocker_count: usize,
        error_count: usize,
        reasons: Vec<String>,
    },
}

impl PublicationVerdict {
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved { .. })
    }
}

/// Gate enforcing zero Blocker defects and controlled Error quarantine budgets.
#[derive(Debug, Clone, Copy)]
pub struct CompilerPublicationGate {
    pub max_allowed_errors: usize,
    pub allow_warnings: bool,
}

impl Default for CompilerPublicationGate {
    fn default() -> Self {
        Self::production()
    }
}

impl CompilerPublicationGate {
    /// Strict production gate: 0 blockers, 0 unquarantined errors.
    pub fn production() -> Self {
        Self {
            max_allowed_errors: 0,
            allow_warnings: true,
        }
    }

    /// Permissive development gate: allows up to N errors for testing.
    pub fn development(max_errors: usize) -> Self {
        Self {
            max_allowed_errors: max_errors,
            allow_warnings: true,
        }
    }

    /// Evaluates a validation report against publication gate criteria.
    pub fn evaluate(&self, report: &ValidationReport) -> PublicationVerdict {
        let blocker_count = report
            .diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Blocker)
            .count();

        let error_count = report
            .diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count();

        let mut reasons = Vec::new();

        if blocker_count > 0 {
            reasons.push(format!(
                "Compiler Gate REJECTED: {} fatal BLOCKER defects present in dataset",
                blocker_count
            ));
        }

        if error_count > self.max_allowed_errors {
            reasons.push(format!(
                "Compiler Gate REJECTED: {} ERROR defects exceed allowable threshold ({})",
                error_count, self.max_allowed_errors
            ));
        }

        if reasons.is_empty() {
            let total_features = report.valid_roads_passed
                + report.valid_buildings_passed
                + report.valid_railways_passed
                + report.valid_aeroways_passed
                + report.valid_ports_passed;

            PublicationVerdict::Approved {
                total_features,
                repairs_applied: report.repairs.len(),
            }
        } else {
            PublicationVerdict::Rejected {
                blocker_count,
                error_count,
                reasons,
            }
        }
    }
}
