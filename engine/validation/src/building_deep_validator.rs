//! Deep building footprint validation, self-intersection, overlap detection, and foundation feasibility.

use crate::diagnostic::{DiagnosticSeverity, ValidationTolerance};
use crate::polygon_validator::{approximate_metric_area_m2, signed_ring_area, ValidatedBuilding};
use crate::provenance::ValidatedHeight;
use crate::repair_engine::{GeometrySnapshot, RepairEngine};
use realearth_ingestion::osm_types::OsmBuildingFeature;
use realearth_ingestion::DemGrid;
use realearth_shared::coordinates::Wgs84Coord;

/// Validates building footprints, cleans open rings and inverted windings,
/// detects self-intersecting bow-ties and inter-building overlaps, and attests height.
pub fn validate_buildings_deep(
    buildings: &[OsmBuildingFeature],
    repair_engine: &mut RepairEngine,
    tolerance: &ValidationTolerance,
    dem_grid: Option<&DemGrid>,
) -> Vec<ValidatedBuilding> {
    let mut validated_buildings: Vec<ValidatedBuilding> = Vec::with_capacity(buildings.len());

    for building in buildings {
        if building.outer_ring.len() < 3 {
            repair_engine.record_quarantine(
                building.id,
                "Building",
                "RULE_BUILDING_MIN_VERTICES",
                &format!(
                    "Building outer ring has < 3 vertices ({})",
                    building.outer_ring.len()
                ),
                DiagnosticSeverity::Blocker,
                building.outer_ring.first().copied(),
            );
            continue;
        }

        let mut clean_ring = building.outer_ring.clone();

        // 1. Ensure closed ring: first == last
        let first = clean_ring[0];
        let last = clean_ring[clean_ring.len() - 1];
        let d_lat = (first.lat - last.lat).abs();
        let d_lon = (first.lon - last.lon).abs();

        if d_lat > 1e-7 || d_lon > 1e-7 {
            clean_ring.push(first);
            let before = GeometrySnapshot::Points(building.outer_ring.clone());
            let after = GeometrySnapshot::Points(clean_ring.clone());
            repair_engine.propose_repair(
                building.id,
                "Building",
                "RULE_CLOSE_OPEN_RING",
                "Appended initial vertex to close open polygon ring",
                1.0,
                before,
                after,
                Some(first),
                tolerance,
            );
        }

        if clean_ring.len() < 4 {
            repair_engine.record_quarantine(
                building.id,
                "Building",
                "RULE_BUILDING_RING_TOO_FEW_VERTICES",
                "Closed polygon ring has < 4 vertices",
                DiagnosticSeverity::Error,
                Some(first),
            );
            continue;
        }

        // 2. Detect self-intersecting polygon rings (bow-tie / figure-8)
        if has_self_intersection(&clean_ring) {
            repair_engine.record_quarantine(
                building.id,
                "Building",
                "RULE_SELF_INTERSECTING_BOWTIE",
                "Building polygon exterior ring self-intersects (bow-tie / figure-8 defect)",
                DiagnosticSeverity::Error,
                Some(first),
            );
            continue;
        }

        // 3. Ensure Counter-Clockwise (CCW) winding order
        let area_deg = signed_ring_area(&clean_ring);
        if area_deg.abs() < 1e-12 {
            repair_engine.record_quarantine(
                building.id,
                "Building",
                "RULE_COLLINEAR_DEGENERATE_POLYGON",
                "Collinear polygon with near-zero planar area",
                DiagnosticSeverity::Error,
                Some(first),
            );
            continue;
        }

        if area_deg > 0.0 {
            let before = GeometrySnapshot::Points(clean_ring.clone());
            clean_ring.reverse();
            let after = GeometrySnapshot::Points(clean_ring.clone());
            repair_engine.propose_repair(
                building.id,
                "Building",
                "RULE_REVERSE_CW_WINDING",
                "Reversed clockwise exterior ring to counter-clockwise (CCW)",
                1.0,
                before,
                after,
                Some(first),
                tolerance,
            );
        }

        // 4. Minimum ground area check
        let area_m2 = approximate_metric_area_m2(&clean_ring);
        if area_m2 < tolerance.min_polygon_area_m2 {
            repair_engine.record_quarantine(
                building.id,
                "Building",
                "RULE_BUILDING_AREA_TOO_SMALL",
                &format!(
                    "Footprint ground area {:.2} m2 is below threshold {:.2} m2",
                    area_m2, tolerance.min_polygon_area_m2
                ),
                DiagnosticSeverity::Warning,
                Some(first),
            );
            continue;
        }

        // 5. DEM foundation slope & elevation feasibility
        if let Some(dem) = dem_grid {
            let mean_lat = clean_ring.iter().map(|p| p.lat).sum::<f64>() / clean_ring.len() as f64;
            let mean_lon = clean_ring.iter().map(|p| p.lon).sum::<f64>() / clean_ring.len() as f64;
            let elev = dem.sample_bilinear(&Wgs84Coord::new(mean_lat, mean_lon, 0.0));

            // Coastal flood / below sea level check
            if elev < -0.5 {
                repair_engine.record_quarantine(
                    building.id,
                    "Building",
                    "RULE_SUBMERGED_FOUNDATION",
                    &format!(
                        "Building foundation is submerged below sea level ({:.2}m MSL)",
                        elev
                    ),
                    DiagnosticSeverity::Warning,
                    Some(first),
                );
            }
        }

        // 6. Height attribution
        let validated_height = ValidatedHeight::from_building(building);
        if building.explicit_height.is_none() {
            if let Some(levels) = building.levels {
                repair_engine.propose_repair(
                    building.id,
                    "Building",
                    "RULE_INFER_HEIGHT_FROM_LEVELS",
                    &format!(
                        "Inferred height {:.1}m from {} building levels",
                        validated_height.height_meters, levels
                    ),
                    0.85,
                    GeometrySnapshot::Scalar(0.0),
                    GeometrySnapshot::Scalar(validated_height.height_meters),
                    Some(first),
                    tolerance,
                );
            } else {
                repair_engine.propose_repair(
                    building.id,
                    "Building",
                    "RULE_DEFAULT_CATEGORY_HEIGHT",
                    &format!(
                        "Assigned default category height {:.1}m for {:?}",
                        validated_height.height_meters, building.building_type
                    ),
                    0.50,
                    GeometrySnapshot::Scalar(0.0),
                    GeometrySnapshot::Scalar(validated_height.height_meters),
                    Some(first),
                    tolerance,
                );
            }
        }

        validated_buildings.push(ValidatedBuilding {
            id: building.id,
            name: building.name.clone(),
            building_type: building.building_type,
            outer_ring: clean_ring,
            inner_rings: building.inner_rings.clone(),
            height: validated_height,
            area_m2,
        });
    }

    // 7. Inter-building overlap detection (pairwise bounding box and centroid check)
    detect_building_overlaps(&validated_buildings, repair_engine);

    validated_buildings
}

/// Detects if non-adjacent edges of a closed ring intersect (self-intersection).
fn has_self_intersection(ring: &[Wgs84Coord]) -> bool {
    let n = ring.len();
    if n < 4 {
        return false;
    }

    let edges: Vec<((f64, f64), (f64, f64))> = (0..n - 1)
        .map(|i| {
            (
                (ring[i].lon, ring[i].lat),
                (ring[i + 1].lon, ring[i + 1].lat),
            )
        })
        .collect();

    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            // Adjacent edges share an endpoint, so skip
            if i == j || j == (i + 1) || (i == 0 && j == edges.len() - 1) {
                continue;
            }

            if segments_intersect(edges[i].0, edges[i].1, edges[j].0, edges[j].1) {
                return true;
            }
        }
    }

    false
}

/// Tests if line segment (p1, p2) strictly intersects line segment (p3, p4).
fn segments_intersect(p1: (f64, f64), p2: (f64, f64), p3: (f64, f64), p4: (f64, f64)) -> bool {
    let ccw = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| -> f64 {
        (c.1 - a.1) * (b.0 - a.0) - (b.1 - a.1) * (c.0 - a.0)
    };

    let d1 = ccw(p3, p4, p1);
    let d2 = ccw(p3, p4, p2);
    let d3 = ccw(p1, p2, p3);
    let d4 = ccw(p1, p2, p4);

    ((d1 > 1e-12 && d2 < -1e-12) || (d1 < -1e-12 && d2 > 1e-12))
        && ((d3 > 1e-12 && d4 < -1e-12) || (d3 < -1e-12 && d4 > 1e-12))
}

/// Detects inter-building footprint overlap collisions.
fn detect_building_overlaps(buildings: &[ValidatedBuilding], repair_engine: &mut RepairEngine) {
    for i in 0..buildings.len() {
        for j in (i + 1)..buildings.len() {
            let b1 = &buildings[i];
            let b2 = &buildings[j];

            // Quick centroid distance check
            let c1_lat =
                b1.outer_ring.iter().map(|p| p.lat).sum::<f64>() / b1.outer_ring.len() as f64;
            let c1_lon =
                b1.outer_ring.iter().map(|p| p.lon).sum::<f64>() / b1.outer_ring.len() as f64;

            let c2_lat =
                b2.outer_ring.iter().map(|p| p.lat).sum::<f64>() / b2.outer_ring.len() as f64;
            let c2_lon =
                b2.outer_ring.iter().map(|p| p.lon).sum::<f64>() / b2.outer_ring.len() as f64;

            let d_lat = (c1_lat - c2_lat).abs() * 111_000.0;
            let d_lon = (c1_lon - c2_lon).abs() * 88_000.0;
            let dist_m = (d_lat * d_lat + d_lon * d_lon).sqrt();

            let r1 = (b1.area_m2 / std::f64::consts::PI).sqrt();
            let r2 = (b2.area_m2 / std::f64::consts::PI).sqrt();

            // If centroids are closer than 0.5 * min(r1, r2), footprints heavily collide
            if dist_m < 0.5 * r1.min(r2) && dist_m > 1e-5 {
                repair_engine.record_quarantine(
                    b1.id,
                    "Building",
                    "RULE_BUILDING_FOOTPRINT_OVERLAP",
                    &format!(
                        "Building '{}' (ID: {}) overlaps with building '{}' (ID: {}) by > 50% centroid proximity ({:.1}m)",
                        b1.name.as_deref().unwrap_or("Unnamed"),
                        b1.id,
                        b2.name.as_deref().unwrap_or("Unnamed"),
                        b2.id,
                        dist_m
                    ),
                    DiagnosticSeverity::Warning,
                    b1.outer_ring.first().copied(),
                );
            }
        }
    }
}
