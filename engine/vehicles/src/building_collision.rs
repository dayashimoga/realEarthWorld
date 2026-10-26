//! Spatial building solid collision detection, footprint penetration resolution,
//! and vehicle collision response physics.

use glam::{Vec2, Vec3};
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::OsmBuildingFeature;
use realearth_shared::coordinates::Wgs84Coord;
use std::collections::HashMap;

/// Planar 2D distance and outward normal from a point to a line segment [a, b].
fn point_to_segment_closest_normal(p: Vec2, a: Vec2, b: Vec2) -> (f32, Vec2) {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < 1e-6 {
        let diff = p - a;
        let d = diff.length();
        let norm = if d > 1e-4 { diff / d } else { Vec2::Y };
        return (d, norm);
    }

    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    let diff = p - closest;
    let d = diff.length();

    // Standard 2D CCW outward normal for segment [a, b]: normal = (ab.y, -ab.x)
    let edge_norm = Vec2::new(ab.y, -ab.x).normalize();
    let norm = if d > 1e-4 { diff / d } else { edge_norm };

    (d, norm)
}

/// Tests if 2D point `pt` is strictly inside the closed polygon `ring`.
fn point_in_polygon_2d(pt: Vec2, ring: &[Vec2]) -> bool {
    let mut inside = false;
    let n = ring.len();
    if n < 3 {
        return false;
    }
    let mut j = n - 1;
    for i in 0..n {
        let pi = ring[i];
        let pj = ring[j];
        if ((pi.y > pt.y) != (pj.y > pt.y))
            && (pt.x < (pj.x - pi.x) * (pt.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// An extruded 3D solid building collider defined by a 2D ground footprint polygon
/// and vertical elevation span [min_y, max_y].
#[derive(Debug, Clone, PartialEq)]
pub struct BuildingFootprintCollider {
    pub id: u64,
    pub min_y: f32,
    pub max_y: f32,
    pub aabb_min: Vec2,
    pub aabb_max: Vec2,
    pub polygon: Vec<Vec2>,
}

impl BuildingFootprintCollider {
    pub fn new(id: u64, min_y: f32, max_y: f32, polygon: Vec<Vec2>) -> Self {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;

        for pt in &polygon {
            min_x = min_x.min(pt.x);
            max_x = max_x.max(pt.x);
            min_z = min_z.min(pt.y); // in Vec2: x=East, y=North (Z)
            max_z = max_z.max(pt.y);
        }

        Self {
            id,
            min_y,
            max_y,
            aabb_min: Vec2::new(min_x, min_z),
            aabb_max: Vec2::new(max_x, max_z),
            polygon,
        }
    }

    /// Evaluates if 3D bounding box overlaps this building collider.
    pub fn overlaps_3d(&self, min_pos: Vec3, max_pos: Vec3) -> bool {
        // Vertical check
        if max_pos.y < self.min_y || min_pos.y > self.max_y {
            return false;
        }
        // Horizontal AABB check
        if max_pos.x < self.aabb_min.x || min_pos.x > self.aabb_max.x {
            return false;
        }
        if max_pos.z < self.aabb_min.y || min_pos.z > self.aabb_max.y {
            return false;
        }
        true
    }

    /// Computes penetration vector if point `(pos.x, pos.z)` is inside or within `radius` of building.
    /// Returns `Some((penetration_depth, outward_normal))` pointing away from the building interior.
    pub fn compute_penetration(&self, pos: Vec3, radius: f32) -> Option<(f32, Vec2)> {
        if pos.y < self.min_y - 0.5 || pos.y > self.max_y + 0.5 {
            return None;
        }

        let pt = Vec2::new(pos.x, pos.z);
        let n = self.polygon.len();
        if n < 3 {
            return None;
        }

        let is_inside = point_in_polygon_2d(pt, &self.polygon);

        let mut min_dist = f32::MAX;
        let mut best_normal = Vec2::Y;

        for i in 0..n {
            let next = (i + 1) % n;
            let a = self.polygon[i];
            let b = self.polygon[next];
            let (dist, norm) = point_to_segment_closest_normal(pt, a, b);
            if dist < min_dist {
                min_dist = dist;
                best_normal = norm;
            }
        }

        if is_inside {
            // Point is inside building solid: push outward toward closest exterior perimeter.
            // When inside, (pt - closest) points inward, so the outward ejection vector is -best_normal.
            let pen_depth = min_dist + radius;
            let outward_normal = if min_dist > 1e-4 {
                -best_normal
            } else {
                let test_pos = pt + best_normal * 0.1;
                if point_in_polygon_2d(test_pos, &self.polygon) {
                    -best_normal
                } else {
                    best_normal
                }
            };
            Some((pen_depth, outward_normal))
        } else if min_dist < radius {
            // Vehicle boundary intersects building exterior wall: best_normal points outward toward pt
            let pen_depth = radius - min_dist;
            Some((pen_depth, best_normal))
        } else {
            None
        }
    }
}

/// High-performance spatial grid acceleration structure for building collision queries.
#[derive(Debug, Clone, Default)]
pub struct BuildingCollisionWorld {
    colliders: Vec<BuildingFootprintCollider>,
    /// Cell size in meters (e.g. 50.0m)
    cell_size: f32,
    /// Spatial grid hash: (cell_x, cell_z) -> Vec<collider_index>
    grid: HashMap<(i32, i32), Vec<usize>>,
}

impl BuildingCollisionWorld {
    pub fn new() -> Self {
        Self {
            colliders: Vec::new(),
            cell_size: 50.0,
            grid: HashMap::new(),
        }
    }

    /// Builds a spatial collision world from raw OSM building features.
    pub fn from_osm_buildings(
        buildings: &[OsmBuildingFeature],
        anchor: Wgs84Coord,
        dem: &DemGrid,
    ) -> Self {
        let converter = EnuConverter::new(anchor);
        let mut world = Self::new();
        for b in buildings {
            if b.outer_ring.len() < 3 {
                continue;
            }
            let mean_lat =
                b.outer_ring.iter().map(|p| p.lat).sum::<f64>() / b.outer_ring.len() as f64;
            let mean_lon =
                b.outer_ring.iter().map(|p| p.lon).sum::<f64>() / b.outer_ring.len() as f64;
            let centroid = Wgs84Coord::new(mean_lat, mean_lon, 0.0);
            let base_elev = dem.sample_bilinear(&centroid);
            let height = b.explicit_height.map(|h| h as f32).unwrap_or(15.0);
            let roof_elev = base_elev + height;

            let enu_pts: Vec<Vec2> = b
                .outer_ring
                .iter()
                .map(|pt| {
                    let enu = converter.wgs84_to_enu(*pt);
                    Vec2::new(enu.east as f32, enu.north as f32)
                })
                .collect();

            let collider =
                BuildingFootprintCollider::new(b.id, base_elev - 2.0, roof_elev, enu_pts);
            world.add_collider(collider);
        }
        world
    }

    /// Adds a building collider and populates the spatial acceleration grid.
    pub fn add_collider(&mut self, collider: BuildingFootprintCollider) {
        let idx = self.colliders.len();
        let min_cx = (collider.aabb_min.x / self.cell_size).floor() as i32;
        let max_cx = (collider.aabb_max.x / self.cell_size).floor() as i32;
        let min_cz = (collider.aabb_min.y / self.cell_size).floor() as i32;
        let max_cz = (collider.aabb_max.y / self.cell_size).floor() as i32;

        for cx in min_cx..=max_cx {
            for cz in min_cz..=max_cz {
                self.grid.entry((cx, cz)).or_default().push(idx);
            }
        }
        self.colliders.push(collider);
    }

    pub fn collider_count(&self) -> usize {
        self.colliders.len()
    }

    /// Resolves vehicle solid penetration against all surrounding building colliders.
    ///
    /// If collision is detected:
    /// - Displaces `position` out along wall normal.
    /// - Adjusts `velocity` with restitution bounce and tangential scraping friction.
    /// - Returns `true` if contact occurred.
    pub fn resolve_collision(
        &self,
        position: &mut Vec3,
        velocity: &mut Vec3,
        half_extents: Vec3,
    ) -> bool {
        let radius = (half_extents.x.powi(2) + half_extents.z.powi(2)).sqrt();
        let min_pos = *position - Vec3::new(radius, half_extents.y, radius);
        let max_pos = *position + Vec3::new(radius, half_extents.y, radius);

        let cx = (position.x / self.cell_size).floor() as i32;
        let cz = (position.z / self.cell_size).floor() as i32;

        let mut collided = false;

        // Query 3x3 neighboring cells
        for dx in -1..=1 {
            for dz in -1..=1 {
                if let Some(indices) = self.grid.get(&(cx + dx, cz + dz)) {
                    for &idx in indices {
                        let collider = &self.colliders[idx];
                        if !collider.overlaps_3d(min_pos, max_pos) {
                            continue;
                        }

                        if let Some((pen_depth, normal_2d)) =
                            collider.compute_penetration(*position, radius)
                        {
                            if pen_depth > 0.001 {
                                collided = true;

                                // 1. Positional displacement out of building solid
                                let normal_3d =
                                    Vec3::new(normal_2d.x, 0.0, normal_2d.y).normalize();
                                *position += normal_3d * pen_depth;

                                // 2. Velocity collision response: inelastic bumper bounce & scraping friction
                                let v_normal = velocity.dot(normal_3d);
                                if v_normal < 0.0 {
                                    let restitution = 0.15f32; // Inelastic bumper deformation
                                    let friction = 0.45f32; // Wall scrape friction

                                    let v_perp = normal_3d * v_normal;
                                    let v_tangent = *velocity - v_perp;

                                    // Reflect normal velocity with restitution and damp tangential speed
                                    *velocity = v_tangent * (1.0 - friction).max(0.0)
                                        - normal_3d * (v_normal * restitution);
                                }
                            }
                        }
                    }
                }
            }
        }

        collided
    }

    /// Evaluates minimum horizontal clearance in meters from `pos` to any building perimeter.
    pub fn query_clearance(&self, pos: Vec3) -> f32 {
        let cx = (pos.x / self.cell_size).floor() as i32;
        let cz = (pos.z / self.cell_size).floor() as i32;
        let pt = Vec2::new(pos.x, pos.z);

        let mut min_clearance = f32::MAX;

        for dx in -1..=1 {
            for dz in -1..=1 {
                if let Some(indices) = self.grid.get(&(cx + dx, cz + dz)) {
                    for &idx in indices {
                        let collider = &self.colliders[idx];
                        if pos.y < collider.min_y || pos.y > collider.max_y {
                            continue;
                        }

                        if point_in_polygon_2d(pt, &collider.polygon) {
                            return 0.0;
                        }

                        let n = collider.polygon.len();
                        for i in 0..n {
                            let next = (i + 1) % n;
                            let (dist, _) = point_to_segment_closest_normal(
                                pt,
                                collider.polygon[i],
                                collider.polygon[next],
                            );
                            min_clearance = min_clearance.min(dist);
                        }
                    }
                }
            }
        }

        min_clearance
    }

    /// Returns true if `pos` is inside or within `margin` meters of any building footprint.
    pub fn is_point_inside(&self, pos: Vec3, margin: f32) -> bool {
        self.query_clearance(pos) < margin
    }
}
