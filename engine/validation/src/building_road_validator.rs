//! Building-to-road corridor spatial conflict detection and quarantine.

use crate::diagnostic::DiagnosticSeverity;
use crate::polygon_validator::ValidatedBuilding;
use crate::repair_engine::RepairEngine;
use realearth_ingestion::osm_types::{OsmRailwayFeature, OsmRoadFeature};
use realearth_shared::coordinates::Wgs84Coord;

fn point_in_polygon(pt: &Wgs84Coord, ring: &[Wgs84Coord]) -> bool {
    let mut inside = false;
    let n = ring.len();
    if n < 3 {
        return false;
    }
    let mut j = n - 1;
    for i in 0..n {
        let pi = ring[i];
        let pj = ring[j];
        if ((pi.lat > pt.lat) != (pj.lat > pt.lat))
            && (pt.lon < (pj.lon - pi.lon) * (pt.lat - pi.lat) / (pj.lat - pi.lat) + pi.lon)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn segments_intersect(a1: Wgs84Coord, a2: Wgs84Coord, b1: Wgs84Coord, b2: Wgs84Coord) -> bool {
    let ccw = |p1: Wgs84Coord, p2: Wgs84Coord, p3: Wgs84Coord| -> f64 {
        (p2.lon - p1.lon) * (p3.lat - p1.lat) - (p2.lat - p1.lat) * (p3.lon - p1.lon)
    };
    let d1 = ccw(a1, a2, b1);
    let d2 = ccw(a1, a2, b2);
    let d3 = ccw(b1, b2, a1);
    let d4 = ccw(b1, b2, a2);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    false
}

/// Detects and logs building footprints that penetrate surface road carriageways or sidewalks.
///
/// Grade-separated infrastructure (bridges and tunnels) is explicitly respected and excluded.
pub fn validate_building_road_conflicts(
    buildings: &[ValidatedBuilding],
    roads: &[OsmRoadFeature],
    repair_engine: &mut RepairEngine,
) {
    let lat_scale = 111_139.0;
    let lon_scale = 111_139.0 * 37.8f64.to_radians().cos();

    for road in roads {
        if road.layer != 0 || road.is_bridge || road.is_tunnel {
            continue;
        }

        let road_half_w_m = (road.lanes.max(1) as f64) * 3.5 * 0.5 + 2.0; // road + sidewalk in meters

        for bldg in buildings {
            if bldg.outer_ring.len() < 3 {
                continue;
            }

            let mut collides = false;
            let mut reason = "";
            let mut conflict_coord = bldg.outer_ring.first().copied();

            // Check 1: Any road segment intersects any building polygon edge
            for r_idx in 0..(road.geometry.len().saturating_sub(1)) {
                let r1 = road.geometry[r_idx];
                let r2 = road.geometry[r_idx + 1];

                for b_idx in 0..(bldg.outer_ring.len().saturating_sub(1)) {
                    let b1 = bldg.outer_ring[b_idx];
                    let b2 = bldg.outer_ring[b_idx + 1];
                    if segments_intersect(r1, r2, b1, b2) {
                        collides = true;
                        reason = "road centerline penetrates building facade";
                        conflict_coord = Some(Wgs84Coord::new(
                            (r1.lat + r2.lat) * 0.5,
                            (r1.lon + r2.lon) * 0.5,
                            0.0,
                        ));
                        break;
                    }
                }
                if collides {
                    break;
                }
            }

            // Check 2: Any road node is inside building
            if !collides {
                for r_pt in &road.geometry {
                    if point_in_polygon(r_pt, &bldg.outer_ring) {
                        collides = true;
                        reason = "road centerline node enclosed within building footprint";
                        conflict_coord = Some(*r_pt);
                        break;
                    }
                }
            }

            // Check 3: Any building vertex penetrates road corridor buffer
            if !collides {
                for b_pt in &bldg.outer_ring {
                    for r_idx in 0..(road.geometry.len().saturating_sub(1)) {
                        let r1 = road.geometry[r_idx];
                        let r2 = road.geometry[r_idx + 1];
                        let dx = (r2.lon - r1.lon) * lon_scale;
                        let dy = (r2.lat - r1.lat) * lat_scale;
                        let len_sq = dx * dx + dy * dy;
                        if len_sq > 1e-6 {
                            let px = (b_pt.lon - r1.lon) * lon_scale;
                            let py = (b_pt.lat - r1.lat) * lat_scale;
                            let t = ((px * dx + py * dy) / len_sq).clamp(0.0, 1.0);
                            let proj_x = r1.lon * lon_scale + t * dx;
                            let proj_y = r1.lat * lat_scale + t * dy;
                            let dist_m = ((b_pt.lon * lon_scale - proj_x).powi(2)
                                + (b_pt.lat * lat_scale - proj_y).powi(2))
                            .sqrt();
                            if dist_m < road_half_w_m {
                                collides = true;
                                reason =
                                    "building perimeter vertex violates road sidewalk clearance";
                                conflict_coord = Some(*b_pt);
                                break;
                            }
                        }
                    }
                    if collides {
                        break;
                    }
                }
            }

            if collides {
                repair_engine.record_quarantine(
                    bldg.id,
                    "BuildingRoadCorridor",
                    "RULE_BUILDING_ROAD_CORRIDOR_CONFLICT",
                    &format!(
                        "Building [{}] '{}' intersects Road [{}] '{}': {}",
                        bldg.id,
                        bldg.name.as_deref().unwrap_or("Unnamed"),
                        road.way_id,
                        road.name.as_deref().unwrap_or("Unnamed"),
                        reason
                    ),
                    DiagnosticSeverity::Warning,
                    conflict_coord,
                );
            }
        }
    }
}

/// Detects and logs building footprints that penetrate surface railway corridors.
///
/// Elevation-aware: Grade-separated railway corridors (bridges, viaducts, tunnels) are
/// recognized as legitimate transit layers and not penalized.
pub fn validate_building_railway_conflicts(
    buildings: &[ValidatedBuilding],
    railways: &[OsmRailwayFeature],
    repair_engine: &mut RepairEngine,
) {
    let lat_scale = 111_139.0;
    let lon_scale = 111_139.0 * 37.8f64.to_radians().cos();

    for rail in railways {
        // Grade-separated railway corridors (viaducts and tunnels) are legitimate
        if rail.layer != 0 || rail.is_bridge || rail.is_tunnel {
            continue;
        }

        let rail_half_w_m = 2.5; // Railway track clearance envelope half-width

        for bldg in buildings {
            if bldg.outer_ring.len() < 3 {
                continue;
            }

            let mut collides = false;
            let mut reason = "";
            let mut conflict_coord = bldg.outer_ring.first().copied();

            // Check 1: Any rail segment intersects any building polygon edge
            for r_idx in 0..(rail.geometry.len().saturating_sub(1)) {
                let r1 = rail.geometry[r_idx];
                let r2 = rail.geometry[r_idx + 1];

                for b_idx in 0..(bldg.outer_ring.len().saturating_sub(1)) {
                    let b1 = bldg.outer_ring[b_idx];
                    let b2 = bldg.outer_ring[b_idx + 1];
                    if segments_intersect(r1, r2, b1, b2) {
                        collides = true;
                        reason = "railway track centerline penetrates building facade at grade";
                        conflict_coord = Some(Wgs84Coord::new(
                            (r1.lat + r2.lat) * 0.5,
                            (r1.lon + r2.lon) * 0.5,
                            0.0,
                        ));
                        break;
                    }
                }
                if collides {
                    break;
                }
            }

            // Check 2: Any rail node is inside building
            if !collides {
                for r_pt in &rail.geometry {
                    if point_in_polygon(r_pt, &bldg.outer_ring) {
                        collides = true;
                        reason = "railway track node enclosed within surface building footprint";
                        conflict_coord = Some(*r_pt);
                        break;
                    }
                }
            }

            // Check 3: Any building vertex violates railway clearance buffer
            if !collides {
                for b_pt in &bldg.outer_ring {
                    for r_idx in 0..(rail.geometry.len().saturating_sub(1)) {
                        let r1 = rail.geometry[r_idx];
                        let r2 = rail.geometry[r_idx + 1];
                        let dx = (r2.lon - r1.lon) * lon_scale;
                        let dy = (r2.lat - r1.lat) * lat_scale;
                        let len_sq = dx * dx + dy * dy;
                        if len_sq > 1e-6 {
                            let px = (b_pt.lon - r1.lon) * lon_scale;
                            let py = (b_pt.lat - r1.lat) * lat_scale;
                            let t = ((px * dx + py * dy) / len_sq).clamp(0.0, 1.0);
                            let proj_x = r1.lon * lon_scale + t * dx;
                            let proj_y = r1.lat * lat_scale + t * dy;
                            let dist_m = ((b_pt.lon * lon_scale - proj_x).powi(2)
                                + (b_pt.lat * lat_scale - proj_y).powi(2))
                            .sqrt();
                            if dist_m < rail_half_w_m {
                                collides = true;
                                reason =
                                    "building perimeter vertex violates railway safety clearance";
                                conflict_coord = Some(*b_pt);
                                break;
                            }
                        }
                    }
                    if collides {
                        break;
                    }
                }
            }

            if collides {
                repair_engine.record_quarantine(
                    bldg.id,
                    "BuildingRailwayCorridor",
                    "RULE_BUILDING_RAILWAY_CORRIDOR_CONFLICT",
                    &format!(
                        "Building [{}] '{}' intersects Railway [{}] '{}': {}",
                        bldg.id,
                        bldg.name.as_deref().unwrap_or("Unnamed"),
                        rail.way_id,
                        rail.name.as_deref().unwrap_or("Unnamed"),
                        reason
                    ),
                    DiagnosticSeverity::Warning,
                    conflict_coord,
                );
            }
        }
    }
}

/// Unified elevation-aware building conflict validator checking both road and railway networks.
pub fn validate_building_infrastructure_conflicts(
    buildings: &[ValidatedBuilding],
    roads: &[OsmRoadFeature],
    railways: &[OsmRailwayFeature],
    repair_engine: &mut RepairEngine,
) {
    validate_building_road_conflicts(buildings, roads, repair_engine);
    validate_building_railway_conflicts(buildings, railways, repair_engine);
}
