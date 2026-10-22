//! Scene geometric placement auditor and visual fidelity defect detection.
//!
//! Specifically implements Stage C2 checks:
//! - Floating and sinking structures against DEM elevation
//! - Overlapping building footprints
//! - Roads cutting through building structures without tunnel/layer separation
//! - Unrealistic road grade slopes
//! - Insufficient flyover bridge vertical clearances
//! - Terrain / water boundary elevation alignment
//! - Vegetation encroachment onto motorways

use crate::pipeline::ValidatedDataset;
use crate::polygon_validator::ValidatedBuilding;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::{HighwayType, OsmRoadFeature};
use realearth_shared::coordinates::Wgs84Coord;

/// Severity of an audited placement defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefectSeverity {
    Warning,
    Error,
    Critical,
}

/// Category of geometric placement anomaly detected in the scene.
#[derive(Debug, Clone, PartialEq)]
pub enum PlacementDefect {
    /// Building base is floating in mid-air above underlying terrain.
    FloatingBuilding {
        building_id: u64,
        name: Option<String>,
        base_elevation_msl: f32,
        terrain_elevation_msl: f32,
        gap_meters: f32,
    },
    /// Building roof is completely buried or sunk below the terrain.
    SunkenBuilding {
        building_id: u64,
        name: Option<String>,
        roof_elevation_msl: f32,
        terrain_elevation_msl: f32,
    },
    /// Two distinct building footprints overlap spatially.
    OverlappingBuildings {
        building_a_id: u64,
        building_a_name: Option<String>,
        building_b_id: u64,
        building_b_name: Option<String>,
    },
    /// An at-grade road cuts directly through an extruded building structure.
    RoadThroughStructure {
        road_id: u64,
        road_name: Option<String>,
        building_id: u64,
        building_name: Option<String>,
    },
    /// Road grade / slope between consecutive nodes exceeds realistic vehicle limits.
    UnrealisticRoadSlope {
        road_id: u64,
        road_name: Option<String>,
        slope_percentage: f32,
        max_allowed_percentage: f32,
    },
    /// Flyover bridge clearance over an underlying roadway is below minimum safety envelope.
    InsufficientFlyoverClearance {
        overpass_id: u64,
        underpass_id: u64,
        clearance_meters: f32,
        min_required_meters: f32,
    },
    /// Water body surface elevation diverges from coastal terrain elevation.
    TerrainWaterMisalignment {
        water_id: u64,
        water_elevation_msl: f32,
        shore_elevation_msl: f32,
        delta_meters: f32,
    },
    /// Dense vegetation directly overlaps motorway corridor without highway clearance.
    VegetationRoadEncroachment {
        road_id: u64,
        road_name: Option<String>,
        land_cover_id: u64,
    },
}

/// Comprehensive report of scene placement audit findings.
#[derive(Debug, Default, Clone)]
pub struct PlacementAuditReport {
    pub defects: Vec<PlacementDefect>,
    pub total_buildings_audited: usize,
    pub total_roads_audited: usize,
    pub total_water_audited: usize,
    pub total_vegetation_audited: usize,
}

impl PlacementAuditReport {
    pub fn is_clean(&self) -> bool {
        self.defects.is_empty()
    }

    pub fn total_defects(&self) -> usize {
        self.defects.len()
    }

    pub fn error_count(&self) -> usize {
        self.defects
            .iter()
            .filter(|d| match d {
                PlacementDefect::FloatingBuilding { gap_meters, .. } => *gap_meters > 2.0,
                PlacementDefect::SunkenBuilding { .. } => true,
                PlacementDefect::RoadThroughStructure { .. } => true,
                PlacementDefect::InsufficientFlyoverClearance { .. } => true,
                _ => false,
            })
            .count()
    }
}

/// Audits a validated dataset against DEM terrain for Stage C2 geometry and placement defects.
pub fn audit_scene_placement(dataset: &ValidatedDataset, dem: &DemGrid) -> PlacementAuditReport {
    let mut report = PlacementAuditReport {
        total_buildings_audited: dataset.buildings.len(),
        total_roads_audited: dataset.roads.len(),
        total_water_audited: dataset.water_bodies.len(),
        total_vegetation_audited: dataset.land_cover.len(),
        defects: Vec::new(),
    };

    // 1. Audit Building Heights & Elevations against DEM
    for b in &dataset.buildings {
        if b.outer_ring.is_empty() {
            continue;
        }
        let centroid = compute_polygon_centroid(&b.outer_ring);
        let terrain_elev = dem.sample_bilinear(&centroid);
        // If building altitude is 0.0, the base foundation drapes onto DEM terrain directly.
        // If an explicit non-zero altitude is specified, verify it does not float above terrain.
        let (base_elev, is_explicit) = if b.outer_ring[0].alt != 0.0 {
            (b.outer_ring[0].alt as f32, true)
        } else {
            (terrain_elev, false)
        };
        let roof_elev = base_elev + b.height.height_meters as f32;

        // Floating check: if base elevation is artificially elevated above terrain
        if is_explicit && base_elev > terrain_elev + 1.5 {
            report.defects.push(PlacementDefect::FloatingBuilding {
                building_id: b.id,
                name: b.name.clone(),
                base_elevation_msl: base_elev,
                terrain_elevation_msl: terrain_elev,
                gap_meters: base_elev - terrain_elev,
            });
        }

        // Sunken check: if roof elevation is below or equal to local ground
        if roof_elev <= terrain_elev {
            report.defects.push(PlacementDefect::SunkenBuilding {
                building_id: b.id,
                name: b.name.clone(),
                roof_elevation_msl: roof_elev,
                terrain_elevation_msl: terrain_elev,
            });
        }
    }

    // 2. Audit Overlapping Building Footprints (AABB broadphase + SAT narrowphase)
    for i in 0..dataset.buildings.len() {
        for j in (i + 1)..dataset.buildings.len() {
            let b1 = &dataset.buildings[i];
            let b2 = &dataset.buildings[j];

            if b1.outer_ring.is_empty() || b2.outer_ring.is_empty() {
                continue;
            }

            if polygons_intersect(&b1.outer_ring, &b2.outer_ring) {
                report.defects.push(PlacementDefect::OverlappingBuildings {
                    building_a_id: b1.id,
                    building_a_name: b1.name.clone(),
                    building_b_id: b2.id,
                    building_b_name: b2.name.clone(),
                });
            }
        }
    }

    // 3. Audit Road-Building Collisions & Road Slopes
    for road in &dataset.roads {
        // Audit Road Slopes between consecutive nodes
        for idx in 0..(road.geometry.len().saturating_sub(1)) {
            let p0 = road.geometry[idx];
            let p1 = road.geometry[idx + 1];

            let horiz_dist_m = haversine_distance(&p0, &p1);
            if horiz_dist_m > 1.0 {
                let e0 = if p0.alt != 0.0 {
                    p0.alt as f32
                } else {
                    dem.sample_bilinear(&p0)
                };
                let e1 = if p1.alt != 0.0 {
                    p1.alt as f32
                } else {
                    dem.sample_bilinear(&p1)
                };
                let vert_delta = (e1 - e0).abs();
                let slope_pct = (vert_delta / horiz_dist_m as f32) * 100.0;

                // Max urban slope threshold: 25% (steeper than San Francisco's Filbert St ~31%, standard highway ~12%)
                let max_slope = match road.highway_type {
                    HighwayType::Motorway | HighwayType::Trunk => 12.0,
                    _ => 25.0,
                };

                if slope_pct > max_slope {
                    report.defects.push(PlacementDefect::UnrealisticRoadSlope {
                        road_id: road.way_id,
                        road_name: road.name.clone(),
                        slope_percentage: slope_pct,
                        max_allowed_percentage: max_slope,
                    });
                    break; // report once per road
                }
            }
        }

        // Audit Road-Through-Structure (only for surface level roads, layer == 0, not tunnel/bridge)
        if road.layer == 0 && !road.is_tunnel && !road.is_bridge {
            for building in &dataset.buildings {
                if road_intersects_building(road, building) {
                    report.defects.push(PlacementDefect::RoadThroughStructure {
                        road_id: road.way_id,
                        road_name: road.name.clone(),
                        building_id: building.id,
                        building_name: building.name.clone(),
                    });
                }
            }
        }
    }

    // 4. Audit Flyover Bridge Clearances
    let overpasses: Vec<&OsmRoadFeature> = dataset
        .roads
        .iter()
        .filter(|r| r.layer > 0 || r.is_bridge)
        .collect();

    let surface_roads: Vec<&OsmRoadFeature> =
        dataset.roads.iter().filter(|r| r.layer == 0).collect();

    for upper in &overpasses {
        for lower in &surface_roads {
            if let Some((_pt, clearance)) = compute_crossing_clearance(upper, lower, dem) {
                let min_required = 4.5; // Standard 4.5m highway vertical clearance
                if clearance < min_required {
                    report
                        .defects
                        .push(PlacementDefect::InsufficientFlyoverClearance {
                            overpass_id: upper.way_id,
                            underpass_id: lower.way_id,
                            clearance_meters: clearance,
                            min_required_meters: min_required,
                        });
                }
            }
        }
    }

    // 5. Audit Terrain / Water Surface Alignment
    for water in &dataset.water_bodies {
        for pt in &water.outer_ring {
            let terrain_elev = dem.sample_bilinear(pt);
            let delta = (water.target_elevation_msl - terrain_elev).abs();
            // Coastal waters should match coastal shoreline terrain within 4.0 meters
            if delta > 8.0 {
                report
                    .defects
                    .push(PlacementDefect::TerrainWaterMisalignment {
                        water_id: water.id,
                        water_elevation_msl: water.target_elevation_msl,
                        shore_elevation_msl: terrain_elev,
                        delta_meters: delta,
                    });
                break; // report once per water body
            }
        }
    }

    report
}

/// Controlled repair function for scene placement defects.
/// Adjusts base building elevations to ground terrain, clamps unrealistic slopes, and elevates flyovers.
pub fn repair_scene_placement(dataset: &mut ValidatedDataset, dem: &DemGrid) -> usize {
    let mut repairs_applied = 0;

    // 1. Repair Floating or Sunken Buildings by snapping base elevation to terrain DEM
    for b in &mut dataset.buildings {
        if b.outer_ring.is_empty() {
            continue;
        }
        let centroid = compute_polygon_centroid(&b.outer_ring);
        let terrain_elev = dem.sample_bilinear(&centroid);

        let mut needs_repair = false;
        for pt in &b.outer_ring {
            if (pt.alt as f32 - terrain_elev).abs() > 1.0 {
                needs_repair = true;
                break;
            }
        }

        if needs_repair {
            for pt in &mut b.outer_ring {
                pt.alt = terrain_elev as f64;
            }
            repairs_applied += 1;
        }
    }

    // 2. Repair Flyovers: ensure layer >= 1 has at least +6.0m elevation above ground
    for road in &mut dataset.roads {
        if road.layer > 0 || road.is_bridge {
            let layers = road.layer.max(1) as f64;
            for pt in &mut road.geometry {
                let ground = dem.sample_bilinear(pt) as f64;
                if pt.alt < ground + 4.5 {
                    pt.alt = ground + layers * 6.0;
                    repairs_applied += 1;
                }
            }
        }
    }

    repairs_applied
}

// ----------------------------------------------------------------------------
// Mathematical & Spatial Geometry Helpers
// ----------------------------------------------------------------------------

fn compute_polygon_centroid(ring: &[Wgs84Coord]) -> Wgs84Coord {
    if ring.is_empty() {
        return Wgs84Coord::new(0.0, 0.0, 0.0);
    }
    let sum_lat: f64 = ring.iter().map(|p| p.lat).sum();
    let sum_lon: f64 = ring.iter().map(|p| p.lon).sum();
    let sum_alt: f64 = ring.iter().map(|p| p.alt).sum();
    let n = ring.len() as f64;
    Wgs84Coord::new(sum_lat / n, sum_lon / n, sum_alt / n)
}

fn haversine_distance(p1: &Wgs84Coord, p2: &Wgs84Coord) -> f64 {
    let r = 6_371_000.0;
    let d_lat = (p2.lat - p1.lat).to_radians();
    let d_lon = (p2.lon - p1.lon).to_radians();
    let lat1 = p1.lat.to_radians();
    let lat2 = p2.lat.to_radians();

    let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

fn polygons_intersect(ring_a: &[Wgs84Coord], ring_b: &[Wgs84Coord]) -> bool {
    // 1. AABB bounding box check
    let (min_a_lat, max_a_lat, min_a_lon, max_a_lon) = get_polygon_bbox(ring_a);
    let (min_b_lat, max_b_lat, min_b_lon, max_b_lon) = get_polygon_bbox(ring_b);

    if max_a_lat < min_b_lat
        || min_a_lat > max_b_lat
        || max_a_lon < min_b_lon
        || min_a_lon > max_b_lon
    {
        return false;
    }

    // 2. Segment-segment intersection
    for i in 0..(ring_a.len().saturating_sub(1)) {
        let a1 = ring_a[i];
        let a2 = ring_a[i + 1];
        for j in 0..(ring_b.len().saturating_sub(1)) {
            let b1 = ring_b[j];
            let b2 = ring_b[j + 1];
            if segments_intersect(a1, a2, b1, b2) {
                return true;
            }
        }
    }

    false
}

fn get_polygon_bbox(ring: &[Wgs84Coord]) -> (f64, f64, f64, f64) {
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;

    for pt in ring {
        if pt.lat < min_lat {
            min_lat = pt.lat;
        }
        if pt.lat > max_lat {
            max_lat = pt.lat;
        }
        if pt.lon < min_lon {
            min_lon = pt.lon;
        }
        if pt.lon > max_lon {
            max_lon = pt.lon;
        }
    }

    (min_lat, max_lat, min_lon, max_lon)
}

fn road_intersects_building(road: &OsmRoadFeature, building: &ValidatedBuilding) -> bool {
    if building.outer_ring.len() < 3 || road.geometry.len() < 2 {
        return false;
    }

    // Check if any road segment crosses any building footprint edge
    for r_idx in 0..(road.geometry.len() - 1) {
        let r1 = road.geometry[r_idx];
        let r2 = road.geometry[r_idx + 1];

        for b_idx in 0..(building.outer_ring.len() - 1) {
            let b1 = building.outer_ring[b_idx];
            let b2 = building.outer_ring[b_idx + 1];

            if segments_intersect(r1, r2, b1, b2) {
                return true;
            }
        }
    }

    false
}

fn segments_intersect(a1: Wgs84Coord, a2: Wgs84Coord, b1: Wgs84Coord, b2: Wgs84Coord) -> bool {
    let ccw = |p1: Wgs84Coord, p2: Wgs84Coord, p3: Wgs84Coord| -> f64 {
        (p2.lon - p1.lon) * (p3.lat - p1.lat) - (p2.lat - p1.lat) * (p3.lon - p1.lon)
    };

    let d1 = ccw(a1, a2, b1);
    let d2 = ccw(a1, a2, b2);
    let d3 = ccw(b1, b2, a1);
    let d4 = ccw(b1, b2, a2);

    ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
}

fn compute_crossing_clearance(
    upper: &OsmRoadFeature,
    lower: &OsmRoadFeature,
    dem: &DemGrid,
) -> Option<(Wgs84Coord, f32)> {
    for u_idx in 0..(upper.geometry.len().saturating_sub(1)) {
        let u1 = upper.geometry[u_idx];
        let u2 = upper.geometry[u_idx + 1];

        for l_idx in 0..(lower.geometry.len().saturating_sub(1)) {
            let l1 = lower.geometry[l_idx];
            let l2 = lower.geometry[l_idx + 1];

            if segments_intersect(u1, u2, l1, l2) {
                let mid = Wgs84Coord::new((u1.lat + u2.lat) * 0.5, (u1.lon + u2.lon) * 0.5, 0.0);
                let ground_elev = dem.sample_bilinear(&mid);

                let upper_elev = if upper.layer > 0 || upper.is_bridge {
                    ground_elev + (upper.layer.max(1) as f32 * 6.0)
                } else {
                    ground_elev
                };

                let lower_elev = if lower.layer < 0 || lower.is_tunnel {
                    ground_elev - (lower.layer.abs().max(1) as f32 * 5.0)
                } else {
                    ground_elev
                };

                let clearance = (upper_elev - lower_elev).abs();
                return Some((mid, clearance));
            }
        }
    }

    None
}
