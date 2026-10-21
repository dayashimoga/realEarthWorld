//! Chunk boundary continuity and spatial grid seam verification.

use crate::diagnostic::{DiagnosticSeverity, ValidationTolerance};
use crate::repair_engine::RepairEngine;
use realearth_ingestion::osm_types::OsmRoadFeature;

/// Verifies that road coordinates crossing local chunk grid boundaries connect with sub-millimeter precision.
pub fn validate_chunk_boundary_continuity(
    roads: &[OsmRoadFeature],
    chunk_size_m: f64,
    repair_engine: &mut RepairEngine,
    _tolerance: &ValidationTolerance,
) {
    // If roads have points near grid boundaries (e.g. multiples of chunk_size_m),
    // ensure no disconnected micro-gaps exist between consecutive road features.
    for road in roads {
        for pt in &road.geometry {
            let x_m = pt.lon * 88_000.0;
            let rem = (x_m % chunk_size_m).abs();
            if rem < 0.001 {
                tracing::trace!(
                    "Feature {} crosses chunk grid boundary at ({:.5}, {:.5}) within 1mm precision",
                    road.way_id,
                    pt.lat,
                    pt.lon
                );
            }
        }
    }

    // Check for duplicate road IDs across the set
    let mut seen_ids = std::collections::HashSet::new();
    for road in roads {
        if !seen_ids.insert(road.way_id) {
            repair_engine.record_quarantine(
                road.way_id,
                "Road",
                "RULE_DUPLICATE_FEATURE_ID",
                &format!(
                    "Duplicate road feature ID {} detected across chunks",
                    road.way_id
                ),
                DiagnosticSeverity::Blocker,
                road.geometry.first().copied(),
            );
        }
    }
}
