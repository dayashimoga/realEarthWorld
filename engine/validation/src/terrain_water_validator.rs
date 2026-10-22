//! Terrain slope continuity, DEM seam crack detection, and water boundary validation.

use crate::diagnostic::{DiagnosticSeverity, ValidationTolerance};
use crate::polygon_validator::signed_ring_area;
use crate::repair_engine::{GeometrySnapshot, RepairEngine};
use realearth_ingestion::osm_types::OsmWaterFeature;
use realearth_ingestion::DemGrid;
use realearth_shared::coordinates::Wgs84Coord;

/// Validates DEM grid raster slope continuity and seam step discontinuities.
pub fn validate_dem_continuity(
    dem: &DemGrid,
    repair_engine: &mut RepairEngine,
    tolerance: &ValidationTolerance,
) {
    let width = dem.width;
    let height = dem.height;

    for y in 0..height {
        for x in 0..width {
            let elev = dem.get_at(x, y);

            // Check horizontal neighbor
            if x + 1 < width {
                let elev_right = dem.get_at(x + 1, y);
                let diff = (elev - elev_right).abs();
                if diff > tolerance.max_dem_step_m {
                    let u = x as f64 / width.max(1) as f64;
                    let v = y as f64 / height.max(1) as f64;
                    let lat = dem.bounds.max_lat - v * (dem.bounds.max_lat - dem.bounds.min_lat);
                    let lon = dem.bounds.min_lon + u * (dem.bounds.max_lon - dem.bounds.min_lon);
                    repair_engine.record_quarantine(
                        (y * width + x) as u64,
                        "TerrainDEM",
                        "RULE_DEM_STEP_DISCONTINUITY",
                        &format!(
                            "DEM elevation step discontinuity ({:.1}m > {:.1}m threshold) at x {}, y {}",
                            diff, tolerance.max_dem_step_m, x, y
                        ),
                        DiagnosticSeverity::Warning,
                        Some(Wgs84Coord::new(lat, lon, elev as f64)),
                    );
                }
            }

            // Check vertical neighbor
            if y + 1 < height {
                let elev_down = dem.get_at(x, y + 1);
                let diff = (elev - elev_down).abs();
                if diff > tolerance.max_dem_step_m {
                    let u = x as f64 / width.max(1) as f64;
                    let v = y as f64 / height.max(1) as f64;
                    let lat = dem.bounds.max_lat - v * (dem.bounds.max_lat - dem.bounds.min_lat);
                    let lon = dem.bounds.min_lon + u * (dem.bounds.max_lon - dem.bounds.min_lon);
                    repair_engine.record_quarantine(
                        (y * width + x) as u64,
                        "TerrainDEM",
                        "RULE_DEM_STEP_DISCONTINUITY",
                        &format!(
                            "DEM elevation step discontinuity ({:.1}m > {:.1}m threshold) at x {}, y {}",
                            diff, tolerance.max_dem_step_m, x, y
                        ),
                        DiagnosticSeverity::Warning,
                        Some(Wgs84Coord::new(lat, lon, elev as f64)),
                    );
                }
            }
        }
    }
}

/// Validates coastal and inland water boundary polygons.
pub fn validate_water_bodies(
    water_bodies: &[OsmWaterFeature],
    repair_engine: &mut RepairEngine,
    tolerance: &ValidationTolerance,
) -> Vec<OsmWaterFeature> {
    let mut validated: Vec<OsmWaterFeature> = Vec::with_capacity(water_bodies.len());

    for water in water_bodies {
        if water.outer_ring.len() < 3 {
            repair_engine.record_quarantine(
                water.id,
                "Water",
                "RULE_WATER_MIN_VERTICES",
                &format!(
                    "Water boundary has < 3 vertices ({})",
                    water.outer_ring.len()
                ),
                DiagnosticSeverity::Blocker,
                water.outer_ring.first().copied(),
            );
            continue;
        }

        let mut clean_ring = water.outer_ring.clone();

        // 1. Ensure closed ring
        let first = clean_ring[0];
        let last = clean_ring[clean_ring.len() - 1];
        if (first.lat - last.lat).abs() > 1e-7 || (first.lon - last.lon).abs() > 1e-7 {
            clean_ring.push(first);
            let before = GeometrySnapshot::Points(water.outer_ring.clone());
            let after = GeometrySnapshot::Points(clean_ring.clone());
            repair_engine.propose_repair(
                water.id,
                "Water",
                "RULE_CLOSE_OPEN_RING",
                "Closed open water boundary ring",
                1.0,
                before,
                after,
                Some(first),
                tolerance,
            );
        }

        // 2. Winding order check (CCW)
        let area_deg = signed_ring_area(&clean_ring);
        if area_deg > 0.0 {
            let before = GeometrySnapshot::Points(clean_ring.clone());
            clean_ring.reverse();
            let after = GeometrySnapshot::Points(clean_ring.clone());
            repair_engine.propose_repair(
                water.id,
                "Water",
                "RULE_REVERSE_CW_WINDING",
                "Reversed clockwise water ring to CCW",
                1.0,
                before,
                after,
                Some(first),
                tolerance,
            );
        }

        let mut v_water = water.clone();
        v_water.outer_ring = clean_ring;
        validated.push(v_water);
    }

    validated
}

/// Helper: Checks if a point is inside a closed 2D polygon ring using standard ray-casting.
pub fn point_in_polygon(pt: Wgs84Coord, ring: &[Wgs84Coord]) -> bool {
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

/// Validates potential spatial conflicts between buildings and water bodies.
pub fn validate_water_building_conflicts(
    buildings: &[crate::polygon_validator::ValidatedBuilding],
    water_bodies: &[OsmWaterFeature],
    repair_engine: &mut RepairEngine,
) {
    for b in buildings {
        if b.outer_ring.is_empty() {
            continue;
        }
        let mean_lat = b.outer_ring.iter().map(|p| p.lat).sum::<f64>() / b.outer_ring.len() as f64;
        let mean_lon = b.outer_ring.iter().map(|p| p.lon).sum::<f64>() / b.outer_ring.len() as f64;
        let centroid = Wgs84Coord::new(mean_lat, mean_lon, 0.0);

        for w in water_bodies {
            if point_in_polygon(centroid, &w.outer_ring) {
                repair_engine.record_quarantine(
                    b.id,
                    "Building",
                    "RULE_WATER_BUILDING_CONFLICT",
                    &format!(
                        "Building {} footprint centroid is submerged in water body {}",
                        b.id, w.id
                    ),
                    DiagnosticSeverity::Warning,
                    Some(centroid),
                );
            }
        }
    }
}
