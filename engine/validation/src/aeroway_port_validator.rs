//! Aeroway runway/taxiway geometry and maritime port/pier validation.

use crate::diagnostic::{DiagnosticSeverity, ValidationTolerance};
use crate::repair_engine::{GeometrySnapshot, RepairEngine};
use realearth_ingestion::osm_types::{AerowayType, OsmAerowayFeature, OsmPortFeature};

/// Validates airport runway/taxiway geometry and maritime ports.
pub fn validate_aeroways(
    aeroways: &[OsmAerowayFeature],
    repair_engine: &mut RepairEngine,
    tolerance: &ValidationTolerance,
) -> Vec<OsmAerowayFeature> {
    let mut validated: Vec<OsmAerowayFeature> = Vec::with_capacity(aeroways.len());

    for aero in aeroways {
        if aero.geometry.len() < 2 {
            repair_engine.record_quarantine(
                aero.id,
                "Aeroway",
                "RULE_AEROWAY_MIN_VERTICES",
                &format!("Aeroway feature has < 2 vertices ({})", aero.geometry.len()),
                DiagnosticSeverity::Blocker,
                aero.geometry.first().copied(),
            );
            continue;
        }

        let mut v_aero = aero.clone();

        // Check runway length
        if aero.aeroway_type == AerowayType::Runway {
            let mut total_len = 0.0;
            for i in 0..aero.geometry.len().saturating_sub(1) {
                let p1 = &aero.geometry[i];
                let p2 = &aero.geometry[i + 1];
                let dy = (p2.lat - p1.lat) * 111_000.0;
                let dx = (p2.lon - p1.lon) * 88_000.0;
                total_len += (dx * dx + dy * dy).sqrt();
            }

            if total_len < tolerance.min_runway_length_m {
                repair_engine.record_quarantine(
                    aero.id,
                    "Aeroway",
                    "RULE_RUNWAY_LENGTH_TOO_SHORT",
                    &format!(
                        "Runway length ({:.1}m) is below minimum required threshold ({:.1}m)",
                        total_len, tolerance.min_runway_length_m
                    ),
                    DiagnosticSeverity::Warning,
                    aero.geometry.first().copied(),
                );
            }

            // Runway width sanity check
            if aero.width_m.unwrap_or(0.0) < 15.0 {
                repair_engine.propose_repair(
                    aero.id,
                    "Aeroway",
                    "RULE_RUNWAY_DEFAULT_WIDTH",
                    "Assigned standard 45m runway width",
                    0.90,
                    GeometrySnapshot::Scalar(aero.width_m.unwrap_or(0.0) as f64),
                    GeometrySnapshot::Scalar(45.0),
                    aero.geometry.first().copied(),
                    tolerance,
                );
                v_aero.width_m = Some(45.0);
            }
        }

        validated.push(v_aero);
    }

    validated
}

/// Validates maritime port, pier, and ferry dock features.
pub fn validate_ports(
    ports: &[OsmPortFeature],
    repair_engine: &mut RepairEngine,
    tolerance: &ValidationTolerance,
) -> Vec<OsmPortFeature> {
    let mut validated: Vec<OsmPortFeature> = Vec::with_capacity(ports.len());

    for port in ports {
        if port.polygon.len() < 3 {
            repair_engine.record_quarantine(
                port.id,
                "Port",
                "RULE_PORT_MIN_VERTICES",
                &format!("Port polygon has < 3 vertices ({})", port.polygon.len()),
                DiagnosticSeverity::Blocker,
                port.polygon.first().copied(),
            );
            continue;
        }

        let mut clean = port.polygon.clone();
        let first = clean[0];
        let last = clean[clean.len() - 1];

        // Ensure closed ring
        if (first.lat - last.lat).abs() > 1e-7 || (first.lon - last.lon).abs() > 1e-7 {
            clean.push(first);
            repair_engine.propose_repair(
                port.id,
                "Port",
                "RULE_CLOSE_OPEN_RING",
                "Closed open port polygon ring",
                1.0,
                GeometrySnapshot::Points(port.polygon.clone()),
                GeometrySnapshot::Points(clean.clone()),
                Some(first),
                tolerance,
            );
        }

        let mut v_port = port.clone();
        v_port.polygon = clean;
        validated.push(v_port);
    }

    validated
}
