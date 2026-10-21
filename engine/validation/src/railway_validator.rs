//! Railway topological validation, curve radius, and highway level-crossing analysis.

use crate::diagnostic::{DiagnosticSeverity, ValidationTolerance};
use crate::repair_engine::{GeometrySnapshot, RepairEngine};
use realearth_ingestion::osm_types::{OsmRailwayFeature, OsmRoadFeature};
use realearth_shared::coordinates::Wgs84Coord;

/// Validates railway track segments and detects level crossings with road networks.
pub fn validate_railway_network(
    railways: &[OsmRailwayFeature],
    roads: &[OsmRoadFeature],
    repair_engine: &mut RepairEngine,
    tolerance: &ValidationTolerance,
) -> Vec<OsmRailwayFeature> {
    let mut sanitized: Vec<OsmRailwayFeature> = Vec::with_capacity(railways.len());

    for rail in railways {
        if rail.geometry.len() < 2 {
            repair_engine.record_quarantine(
                rail.way_id,
                "Railway",
                "RULE_RAILWAY_MIN_VERTICES",
                &format!(
                    "Railway track has fewer than 2 vertices ({})",
                    rail.geometry.len()
                ),
                DiagnosticSeverity::Blocker,
                rail.geometry.first().copied(),
            );
            continue;
        }

        // Cull duplicates
        let mut clean_geom: Vec<Wgs84Coord> = Vec::with_capacity(rail.geometry.len());
        let mut duplicates = 0;
        let deg_tolerance = tolerance.linear_snap_m / 111_000.0;

        for pt in &rail.geometry {
            if let Some(prev) = clean_geom.last() {
                if (pt.lat - prev.lat).abs() < deg_tolerance
                    && (pt.lon - prev.lon).abs() < deg_tolerance
                {
                    duplicates += 1;
                    continue;
                }
            }
            clean_geom.push(*pt);
        }

        if duplicates > 0 {
            let before = GeometrySnapshot::Points(rail.geometry.clone());
            let after = GeometrySnapshot::Points(clean_geom.clone());
            repair_engine.propose_repair(
                rail.way_id,
                "Railway",
                "RULE_CULL_DUPLICATE_VERTICES",
                &format!("Culled {} duplicate railway vertices", duplicates),
                1.0,
                before,
                after,
                clean_geom.first().copied(),
                tolerance,
            );
        }

        if clean_geom.len() < 2 {
            repair_engine.record_quarantine(
                rail.way_id,
                "Railway",
                "RULE_RAILWAY_COLLAPSED",
                "Railway collapsed to single vertex after culling duplicates",
                DiagnosticSeverity::Error,
                clean_geom.first().copied(),
            );
            continue;
        }

        let mut validated = rail.clone();
        validated.geometry = clean_geom;

        // Verify gauge sanity
        if let Some(gauge) = validated.gauge_mm {
            if !(600..=2140).contains(&gauge) {
                repair_engine.propose_repair(
                    validated.way_id,
                    "Railway",
                    "RULE_RAILWAY_GAUGE_SANITY",
                    &format!("Corrected unphysical gauge {}mm to standard 1435mm", gauge),
                    0.85,
                    GeometrySnapshot::Scalar(gauge as f64),
                    GeometrySnapshot::Scalar(1435.0),
                    validated.geometry.first().copied(),
                    tolerance,
                );
                validated.gauge_mm = Some(1435);
            }
        }

        // Check curvature limits (trains cannot make acute turns < 25 degrees)
        for i in 1..validated.geometry.len().saturating_sub(1) {
            let p0 = &validated.geometry[i - 1];
            let p1 = &validated.geometry[i];
            let p2 = &validated.geometry[i + 1];

            let v1 = (p1.lon - p0.lon, p1.lat - p0.lat);
            let v2 = (p2.lon - p1.lon, p2.lat - p1.lat);
            let dot = v1.0 * v2.0 + v1.1 * v2.1;
            let mag1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
            let mag2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();

            if mag1 > 1e-9 && mag2 > 1e-9 {
                let cos_t = (dot / (mag1 * mag2)).clamp(-1.0, 1.0);
                let turn_angle = cos_t.acos().to_degrees();
                if turn_angle > 75.0 {
                    repair_engine.record_quarantine(
                        validated.way_id,
                        "Railway",
                        "RULE_RAILWAY_EXCESSIVE_CURVATURE",
                        &format!(
                            "Unphysical sharp railway turn ({:.1} deg) at vertex {}",
                            turn_angle, i
                        ),
                        DiagnosticSeverity::Warning,
                        Some(*p1),
                    );
                }
            }
        }

        // Level-crossing verification with ground roads
        for road in roads {
            if road.layer == 0
                && validated.layer == 0
                && !validated.is_bridge
                && !validated.is_tunnel
            {
                for pt_rail in &validated.geometry {
                    for pt_road in &road.geometry {
                        let d_lat = (pt_rail.lat - pt_road.lat).abs();
                        let d_lon = (pt_rail.lon - pt_road.lon).abs();
                        if d_lat < 1e-7 && d_lon < 1e-7 {
                            tracing::info!(
                                "Identified at-grade rail-highway level crossing between rail {} and road {} at ({:.5}, {:.5})",
                                validated.way_id, road.way_id, pt_rail.lat, pt_rail.lon
                            );
                        }
                    }
                }
            }
        }

        sanitized.push(validated);
    }

    sanitized
}
