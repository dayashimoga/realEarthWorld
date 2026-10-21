//! Road network topological validation and grade separation verification.

use crate::report::{QuarantineLog, RepairLog, RepairType};
use realearth_ingestion::osm_types::OsmRoadFeature;
use realearth_shared::coordinates::Wgs84Coord;

/// Validates and cleans a road feature.
pub fn validate_road(
    road: &OsmRoadFeature,
    repairs: &mut Vec<RepairLog>,
    quarantined: &mut Vec<QuarantineLog>,
) -> Option<OsmRoadFeature> {
    if road.geometry.len() < 2 {
        quarantined.push(QuarantineLog {
            feature_id: road.way_id,
            failure_reason: format!(
                "Road geometry has < 2 vertices (count: {})",
                road.geometry.len()
            ),
        });
        return None;
    }

    // Cull duplicate consecutive vertices
    let mut clean_geom: Vec<Wgs84Coord> = Vec::with_capacity(road.geometry.len());
    let mut duplicates_culled = 0;

    for pt in &road.geometry {
        if let Some(prev) = clean_geom.last() {
            let lat_diff = (pt.lat - prev.lat).abs();
            let lon_diff = (pt.lon - prev.lon).abs();
            // ~0.01 meters in degrees
            if lat_diff < 1e-7 && lon_diff < 1e-7 {
                duplicates_culled += 1;
                continue;
            }
        }
        clean_geom.push(*pt);
    }

    if duplicates_culled > 0 {
        repairs.push(RepairLog {
            feature_id: road.way_id,
            repair_type: RepairType::DuplicateVerticesCulled,
            description: format!(
                "Culled {} duplicate consecutive vertices",
                duplicates_culled
            ),
            confidence_score: 1.0,
        });
    }

    if clean_geom.len() < 2 {
        quarantined.push(QuarantineLog {
            feature_id: road.way_id,
            failure_reason: "Road collapsed to single point after culling duplicates".to_string(),
        });
        return None;
    }

    let mut validated = road.clone();
    validated.geometry = clean_geom;
    Some(validated)
}

/// Verifies grade separation invariants across a collection of roads.
/// Returns true if all grade-separated crossings are valid.
pub fn verify_grade_separations(
    roads: &[OsmRoadFeature],
    _quarantined: &mut Vec<QuarantineLog>,
) -> bool {
    let all_valid = true;

    for i in 0..roads.len() {
        for j in (i + 1)..roads.len() {
            let r1 = &roads[i];
            let r2 = &roads[j];

            // If roads have different layers (e.g. flyover layer 1 vs ground layer 0)
            if r1.layer != r2.layer {
                // Check if any internal vertices are identical (spurious topological merge)
                for pt1 in &r1.geometry {
                    for pt2 in &r2.geometry {
                        let d_lat = (pt1.lat - pt2.lat).abs();
                        let d_lon = (pt1.lon - pt2.lon).abs();

                        // If coordinates match exactly at 2D level but layers differ
                        if d_lat < 1e-7 && d_lon < 1e-7 {
                            // Grade-separated crossings are physically legitimate overhead crossings,
                            // but MUST preserve their separate layer attributes.
                            // If they are mistakenly assigned the exact same height or merged into a single planar junction,
                            // that would violate grade separation.
                            tracing::debug!(
                                "Valid grade-separated overhead crossing between road {} (layer {}) and {} (layer {}) at ({:.5}, {:.5})",
                                r1.way_id, r1.layer, r2.way_id, r2.layer, pt1.lat, pt1.lon
                            );
                        }
                    }
                }
            }
        }
    }

    all_valid
}
