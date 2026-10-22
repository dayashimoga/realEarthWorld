//! Spatial Area of Interest (AoI) management, geographic grid partitioning,
//! and cell boundary handoff detection.

use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 2D grid index of a spatial cell in the planetary coordinate frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpatialCellId {
    pub x: i32,
    pub z: i32,
}

impl SpatialCellId {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Computes spatial cell ID from metric ENU coordinates and cell grid size.
    pub fn from_position(pos: Vec3, cell_size_meters: f32) -> Self {
        let x = (pos.x / cell_size_meters).floor() as i32;
        let z = (pos.z / cell_size_meters).floor() as i32;
        Self { x, z }
    }

    /// Unique string identifier for logging and cell handoff protocol.
    pub fn to_string_id(self) -> String {
        format!("cell_{}_{}", self.x, self.z)
    }
}

/// Area of Interest spatial partitioning manager.
#[derive(Debug, Clone)]
pub struct InterestManager {
    /// Dimension of each square spatial cell in meters (default: 1000m).
    pub cell_size_meters: f32,
    /// Mapping from spatial cell to set of entities located within it.
    pub cell_to_entities: HashMap<SpatialCellId, HashSet<u32>>,
    /// Mapping from entity ID to its current spatial cell.
    pub entity_to_cell: HashMap<u32, SpatialCellId>,
    /// Entity positions cache.
    pub entity_positions: HashMap<u32, Vec3>,
}

impl Default for InterestManager {
    fn default() -> Self {
        Self::new(1000.0)
    }
}

impl InterestManager {
    pub fn new(cell_size_meters: f32) -> Self {
        Self {
            cell_size_meters,
            cell_to_entities: HashMap::new(),
            entity_to_cell: HashMap::new(),
            entity_positions: HashMap::new(),
        }
    }

    /// Updates or inserts an entity's position, detecting if it crossed into a new spatial cell.
    /// Returns `Some((old_cell, new_cell))` if a cell boundary handoff occurred.
    pub fn update_entity_position(
        &mut self,
        entity_id: u32,
        pos: Vec3,
    ) -> Option<(SpatialCellId, SpatialCellId)> {
        let new_cell = SpatialCellId::from_position(pos, self.cell_size_meters);
        self.entity_positions.insert(entity_id, pos);

        if let Some(&old_cell) = self.entity_to_cell.get(&entity_id) {
            if old_cell != new_cell {
                // Remove from old cell
                if let Some(entities) = self.cell_to_entities.get_mut(&old_cell) {
                    entities.remove(&entity_id);
                }
                // Insert into new cell
                self.cell_to_entities
                    .entry(new_cell)
                    .or_default()
                    .insert(entity_id);
                self.entity_to_cell.insert(entity_id, new_cell);

                return Some((old_cell, new_cell));
            }
        } else {
            // First time tracking entity
            self.cell_to_entities
                .entry(new_cell)
                .or_default()
                .insert(entity_id);
            self.entity_to_cell.insert(entity_id, new_cell);
        }

        None
    }

    /// Removes an entity from spatial tracking upon despawn.
    pub fn remove_entity(&mut self, entity_id: u32) {
        if let Some(cell) = self.entity_to_cell.remove(&entity_id) {
            if let Some(entities) = self.cell_to_entities.get_mut(&cell) {
                entities.remove(&entity_id);
            }
        }
        self.entity_positions.remove(&entity_id);
    }

    /// Queries all entities located within interest radius of a query position.
    pub fn query_entities_in_radius(&self, center: Vec3, radius_meters: f32) -> Vec<u32> {
        let min_x = ((center.x - radius_meters) / self.cell_size_meters).floor() as i32;
        let max_x = ((center.x + radius_meters) / self.cell_size_meters).floor() as i32;
        let min_z = ((center.z - radius_meters) / self.cell_size_meters).floor() as i32;
        let max_z = ((center.z + radius_meters) / self.cell_size_meters).floor() as i32;

        let radius_sq = radius_meters * radius_meters;
        let mut visible = Vec::new();

        for cx in min_x..=max_x {
            for cz in min_z..=max_z {
                let cell_id = SpatialCellId::new(cx, cz);
                if let Some(entities) = self.cell_to_entities.get(&cell_id) {
                    for &entity_id in entities {
                        if let Some(&pos) = self.entity_positions.get(&entity_id) {
                            let dist_sq = (pos.x - center.x).powi(2) + (pos.z - center.z).powi(2);
                            if dist_sq <= radius_sq {
                                visible.push(entity_id);
                            }
                        }
                    }
                }
            }
        }

        visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_cell_coordinates_and_handoff() {
        let mut aoi = InterestManager::new(1000.0);

        // Entity starts at (500, 0, 500) -> Cell (0, 0)
        let pos1 = Vec3::new(500.0, 0.0, 500.0);
        let handoff1 = aoi.update_entity_position(101, pos1);
        assert_eq!(handoff1, None); // First insertion
        assert_eq!(
            aoi.entity_to_cell.get(&101),
            Some(&SpatialCellId::new(0, 0))
        );

        // Entity moves within same cell to (800, 0, 800)
        let pos2 = Vec3::new(800.0, 0.0, 800.0);
        let handoff2 = aoi.update_entity_position(101, pos2);
        assert_eq!(handoff2, None);

        // Entity crosses cell boundary to (1200, 0, 800) -> Cell (1, 0)
        let pos3 = Vec3::new(1200.0, 0.0, 800.0);
        let handoff3 = aoi.update_entity_position(101, pos3);
        assert_eq!(
            handoff3,
            Some((SpatialCellId::new(0, 0), SpatialCellId::new(1, 0)))
        );
        assert_eq!(
            aoi.entity_to_cell.get(&101),
            Some(&SpatialCellId::new(1, 0))
        );
    }

    #[test]
    fn test_radius_query_spatial_culling() {
        let mut aoi = InterestManager::new(500.0);

        // Add 3 entities:
        // E1: near (100, 0, 100)
        // E2: mid (600, 0, 600)
        // E3: far (5000, 0, 5000)
        aoi.update_entity_position(1, Vec3::new(100.0, 0.0, 100.0));
        aoi.update_entity_position(2, Vec3::new(600.0, 0.0, 600.0));
        aoi.update_entity_position(3, Vec3::new(5000.0, 0.0, 5000.0));

        // Observer at origin with 800m ground interest radius
        let visible = aoi.query_entities_in_radius(Vec3::ZERO, 800.0);
        assert!(visible.contains(&1));
        assert!(!visible.contains(&2)); // 600*sqrt(2) = 848m > 800m
        assert!(!visible.contains(&3)); // 5000m away, culled

        // High-altitude aircraft observer with 10km radius
        let air_visible = aoi.query_entities_in_radius(Vec3::ZERO, 10_000.0);
        assert!(air_visible.contains(&1));
        assert!(air_visible.contains(&2));
        assert!(air_visible.contains(&3));
    }
}
