//! Road network topological validation, connectivity graph analysis, and grade-separation invariance.

use crate::diagnostic::{DiagnosticSeverity, ValidationTolerance};
use crate::repair_engine::{GeometrySnapshot, RepairEngine};
use realearth_ingestion::osm_types::{HighwayType, OsmRoadFeature};
use realearth_shared::coordinates::Wgs84Coord;
use std::collections::HashMap;

/// Validates individual roads and the interconnected road network graph.
pub fn validate_road_network(
    roads: &[OsmRoadFeature],
    repair_engine: &mut RepairEngine,
    tolerance: &ValidationTolerance,
) -> Vec<OsmRoadFeature> {
    let mut sanitized_roads: Vec<OsmRoadFeature> = Vec::with_capacity(roads.len());

    // 1. Feature-level geometric hygiene and attribute semantics
    for road in roads {
        if road.geometry.len() < 2 {
            repair_engine.record_quarantine(
                road.way_id,
                "Road",
                "RULE_ROAD_MIN_VERTICES",
                &format!(
                    "Road geometry has fewer than 2 vertices ({})",
                    road.geometry.len()
                ),
                DiagnosticSeverity::Blocker,
                road.geometry.first().copied(),
            );
            continue;
        }

        // Cull duplicate consecutive vertices
        let mut clean_geom: Vec<Wgs84Coord> = Vec::with_capacity(road.geometry.len());
        let mut duplicates_culled = 0;

        for pt in &road.geometry {
            if let Some(prev) = clean_geom.last() {
                let d_lat = (pt.lat - prev.lat).abs();
                let d_lon = (pt.lon - prev.lon).abs();
                // Check against linear snap tolerance (~0.05m in degrees)
                let deg_tolerance = tolerance.linear_snap_m / 111_000.0;
                if d_lat < deg_tolerance && d_lon < deg_tolerance {
                    duplicates_culled += 1;
                    continue;
                }
            }
            clean_geom.push(*pt);
        }

        if duplicates_culled > 0 {
            let before = GeometrySnapshot::Points(road.geometry.clone());
            let after = GeometrySnapshot::Points(clean_geom.clone());
            let accepted = repair_engine.propose_repair(
                road.way_id,
                "Road",
                "RULE_CULL_DUPLICATE_VERTICES",
                &format!(
                    "Culled {} duplicate consecutive vertices",
                    duplicates_culled
                ),
                1.0,
                before,
                after,
                clean_geom.first().copied(),
                tolerance,
            );
            if !accepted {
                continue;
            }
        }

        if clean_geom.len() < 2 {
            repair_engine.record_quarantine(
                road.way_id,
                "Road",
                "RULE_ROAD_COLLAPSED",
                "Road collapsed to a single point after culling duplicate vertices",
                DiagnosticSeverity::Error,
                clean_geom.first().copied(),
            );
            continue;
        }

        let mut validated = road.clone();
        validated.geometry = clean_geom;

        // Lane count sanity check
        if validated.lanes == 0 {
            let default_lanes = match validated.highway_type {
                HighwayType::Motorway | HighwayType::Trunk => 4,
                HighwayType::Primary | HighwayType::Secondary => 2,
                _ => 1,
            };
            repair_engine.propose_repair(
                validated.way_id,
                "Road",
                "RULE_DEFAULT_LANE_COUNT",
                &format!("Corrected invalid 0 lanes to default {}", default_lanes),
                0.90,
                GeometrySnapshot::Scalar(0.0),
                GeometrySnapshot::Scalar(default_lanes as f64),
                validated.geometry.first().copied(),
                tolerance,
            );
            validated.lanes = default_lanes;
        }

        // Speed limit sanity check (5 to 160 km/h)
        if validated.speed_limit_kph < 5 || validated.speed_limit_kph > 160 {
            let default_speed = validated.highway_type.default_speed_limit();
            repair_engine.propose_repair(
                validated.way_id,
                "Road",
                "RULE_SPEED_LIMIT_SANITY",
                &format!(
                    "Corrected out-of-range speed limit {} km/h to default {} km/h",
                    validated.speed_limit_kph, default_speed
                ),
                0.85,
                GeometrySnapshot::Scalar(validated.speed_limit_kph as f64),
                GeometrySnapshot::Scalar(default_speed as f64),
                validated.geometry.first().copied(),
                tolerance,
            );
            validated.speed_limit_kph = default_speed;
        }

        sanitized_roads.push(validated);
    }

    // 2. Network-level graph connectivity and dead-end stub analysis
    if sanitized_roads.len() > 1 {
        analyze_network_connectivity(&sanitized_roads, repair_engine);
    }

    // 3. Grade separation invariance check
    verify_grade_separations(&sanitized_roads, repair_engine);

    sanitized_roads
}

/// Identifies isolated dead-end stubs that do not connect to the broader network.
fn analyze_network_connectivity(roads: &[OsmRoadFeature], repair_engine: &mut RepairEngine) {
    // Quantize coordinates to ~1 meter grid to find shared junctions
    let quantize = |pt: &Wgs84Coord| -> (i64, i64) {
        (
            (pt.lat * 100_000.0).round() as i64,
            (pt.lon * 100_000.0).round() as i64,
        )
    };

    let mut node_degree: HashMap<(i64, i64), usize> = HashMap::new();
    for road in roads {
        for pt in &road.geometry {
            *node_degree.entry(quantize(pt)).or_insert(0) += 1;
        }
    }

    for road in roads {
        let has_connection = road
            .geometry
            .iter()
            .any(|pt| node_degree.get(&quantize(pt)).copied().unwrap_or(0) > 1);

        if !has_connection && roads.len() > 3 {
            repair_engine.record_quarantine(
                road.way_id,
                "Road",
                "RULE_DISCONNECTED_ISLAND_ROAD",
                &format!(
                    "Road '{}' (ID: {}) has no shared junctions with the road network",
                    road.name.as_deref().unwrap_or("Unnamed"),
                    road.way_id
                ),
                DiagnosticSeverity::Warning,
                road.geometry.first().copied(),
            );
        }
    }
}

/// Verifies grade-separated crossings (bridges, tunnels, flyovers) against ground roads.
pub fn verify_grade_separations(roads: &[OsmRoadFeature], repair_engine: &mut RepairEngine) {
    for i in 0..roads.len() {
        for j in (i + 1)..roads.len() {
            let r1 = &roads[i];
            let r2 = &roads[j];

            if r1.layer != r2.layer {
                for pt1 in &r1.geometry {
                    for pt2 in &r2.geometry {
                        let d_lat = (pt1.lat - pt2.lat).abs();
                        let d_lon = (pt1.lon - pt2.lon).abs();

                        if d_lat < 1e-7 && d_lon < 1e-7 {
                            // Grade-separated crossing detected
                            tracing::debug!(
                                "Grade-separated overhead crossing between road {} (layer {}) and {} (layer {}) at ({:.5}, {:.5})",
                                r1.way_id, r1.layer, r2.way_id, r2.layer, pt1.lat, pt1.lon
                            );
                        }
                    }
                }
            } else if r1.layer == 0 && r2.layer == 0 {
                // If two ground roads cross, check for acute hairpin merges (< 15 degrees)
                check_intersection_angle(r1, r2, repair_engine);
            }
        }
    }
}

/// Checks intersection geometry for acute hairpin angles (< 15 degrees) on connecting roads.
fn check_intersection_angle(
    r1: &OsmRoadFeature,
    r2: &OsmRoadFeature,
    repair_engine: &mut RepairEngine,
) {
    let endpoints = [
        (r1.geometry.first(), r2.geometry.first()),
        (r1.geometry.first(), r2.geometry.last()),
        (r1.geometry.last(), r2.geometry.first()),
        (r1.geometry.last(), r2.geometry.last()),
    ];

    for (p1_opt, p2_opt) in endpoints {
        if let (Some(p1), Some(p2)) = (p1_opt, p2_opt) {
            let d_lat = (p1.lat - p2.lat).abs();
            let d_lon = (p1.lon - p2.lon).abs();
            if d_lat < 1e-7 && d_lon < 1e-7 {
                // Connected junction found. Check segment vectors:
                let v1 = if r1.geometry.len() >= 2 {
                    let next = &r1.geometry[1];
                    (next.lon - p1.lon, next.lat - p1.lat)
                } else {
                    continue;
                };
                let v2 = if r2.geometry.len() >= 2 {
                    let next = &r2.geometry[1];
                    (next.lon - p2.lon, next.lat - p2.lat)
                } else {
                    continue;
                };

                let dot = v1.0 * v2.0 + v1.1 * v2.1;
                let mag1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
                let mag2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();

                if mag1 > 1e-9 && mag2 > 1e-9 {
                    let cos_theta = (dot / (mag1 * mag2)).clamp(-1.0, 1.0);
                    let angle_deg = cos_theta.acos().to_degrees();
                    if angle_deg < 15.0 && angle_deg > 0.1 {
                        repair_engine.record_quarantine(
                            r1.way_id,
                            "Road",
                            "RULE_ACUTE_HAIRPIN_JUNCTION",
                            &format!(
                                "Acute hairpin junction ({:.1} deg) between way {} and way {}",
                                angle_deg, r1.way_id, r2.way_id
                            ),
                            DiagnosticSeverity::Warning,
                            Some(*p1),
                        );
                    }
                }
            }
        }
    }
}
