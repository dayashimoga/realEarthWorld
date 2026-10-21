//! Rapier3D physics world and vehicle rigid-body collision simulation.

use crate::ground::GroundQuery;
use glam::Vec3;
use rapier3d::prelude::*;

/// Spatial collision filtering groups.
pub const GROUP_VEHICLE: Group = Group::GROUP_1;
pub const GROUP_TERRAIN: Group = Group::GROUP_2;
pub const GROUP_ROAD: Group = Group::GROUP_3;
pub const GROUP_BUILDING: Group = Group::GROUP_4;
pub const GROUP_OBSTACLE: Group = Group::GROUP_5;

/// Encapsulated Rapier3D physics simulation world.
pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub gravity: Vector<Real>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }
    }

    /// Advances physics simulation by `dt` seconds and updates spatial query pipeline.
    pub fn step(&mut self, dt: f32) {
        self.integration_parameters.dt = dt;
        let physics_hooks = ();
        let event_handler = ();

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &physics_hooks,
            &event_handler,
        );

        self.query_pipeline.update(&self.collider_set);
    }

    /// Spawns a static flat ground collider at elevation `y` with terrain collision filtering.
    pub fn spawn_ground(&mut self, y: f32) -> ColliderHandle {
        let interaction = InteractionGroups::new(
            GROUP_TERRAIN,
            GROUP_VEHICLE | GROUP_TERRAIN | GROUP_OBSTACLE,
        );

        let collider = ColliderBuilder::cuboid(10_000.0, 0.1, 10_000.0)
            .translation(vector![0.0, y - 0.1, 0.0])
            .collision_groups(interaction)
            .friction(0.8)
            .build();
        let handle = self.collider_set.insert(collider);
        self.query_pipeline.update(&self.collider_set);
        handle
    }

    /// Spawns a static triangular mesh collider (e.g. for road or terrain chunks).
    pub fn spawn_trimesh_collider(
        &mut self,
        vertices: Vec<Point<Real>>,
        indices: Vec<[u32; 3]>,
        group: Group,
    ) -> ColliderHandle {
        let interaction = InteractionGroups::new(group, GROUP_VEHICLE | GROUP_OBSTACLE);
        let collider = ColliderBuilder::trimesh(vertices, indices)
            .collision_groups(interaction)
            .friction(0.85)
            .build();
        let handle = self.collider_set.insert(collider);
        self.query_pipeline.update(&self.collider_set);
        handle
    }

    /// Spawns a static building collider with building collision filtering.
    pub fn spawn_building_collider(&mut self, center: Vec3, half_extents: Vec3) -> ColliderHandle {
        let interaction = InteractionGroups::new(GROUP_BUILDING, GROUP_VEHICLE);
        let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            .translation(vector![center.x, center.y, center.z])
            .collision_groups(interaction)
            .friction(0.9)
            .build();
        let handle = self.collider_set.insert(collider);
        self.query_pipeline.update(&self.collider_set);
        handle
    }

    /// Spawns a dynamic vehicle body with a cuboid collider and spatial filtering.
    pub fn spawn_vehicle_body(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        half_extents: [f32; 3],
        mass_kg: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, y, z])
            .additional_mass(mass_kg)
            .build();
        let body_handle = self.rigid_body_set.insert(rigid_body);

        let vehicle_interactions = InteractionGroups::new(
            GROUP_VEHICLE,
            GROUP_TERRAIN | GROUP_ROAD | GROUP_BUILDING | GROUP_OBSTACLE | GROUP_VEHICLE,
        );

        let collider = ColliderBuilder::cuboid(half_extents[0], half_extents[1], half_extents[2])
            .collision_groups(vehicle_interactions)
            .friction(0.7)
            .restitution(0.1)
            .build();
        let collider_handle =
            self.collider_set
                .insert_with_parent(collider, body_handle, &mut self.rigid_body_set);

        self.query_pipeline.update(&self.collider_set);

        (body_handle, collider_handle)
    }

    /// Safely spawns a vehicle at (x, z) by probing ground elevation, applying clearance,
    /// and guaranteeing the vehicle spawns upright on the surface without falling through.
    pub fn spawn_safe_vehicle(
        &mut self,
        x: f32,
        z: f32,
        clearance_m: f32,
        half_extents: [f32; 3],
        mass_kg: f32,
    ) -> (RigidBodyHandle, ColliderHandle, f32) {
        // Cast ray downwards from high elevation (1000m) to find ground surface
        let ray_origin = Vec3::new(x, 1000.0, z);
        let ground_y = if let Some((dist, _)) = self.query_ground_contact(ray_origin, 2000.0) {
            1000.0 - dist
        } else {
            0.0 // Default to sea level if no collider registered yet
        };

        let spawn_y = ground_y + half_extents[1] + clearance_m;
        let (body, collider) = self.spawn_vehicle_body(x, spawn_y, z, half_extents, mass_kg);
        (body, collider, spawn_y)
    }
}

impl GroundQuery for PhysicsWorld {
    fn query_ground_contact(&self, origin: Vec3, max_dist: f32) -> Option<(f32, Vec3)> {
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![0.0, -1.0, 0.0],
        );

        // Exclude dynamic vehicles so wheels don't detect the car's own body as ground
        let filter = QueryFilter::default().groups(InteractionGroups::new(
            GROUP_VEHICLE,
            GROUP_TERRAIN | GROUP_ROAD | GROUP_OBSTACLE,
        ));

        if let Some((_handle, hit)) = self.query_pipeline.cast_ray_and_get_normal(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_dist,
            true,
            filter,
        ) {
            let normal = Vec3::new(hit.normal.x, hit.normal.y, hit.normal.z);
            Some((hit.time_of_impact, normal))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_ground_query_and_safe_spawning() {
        let mut physics = PhysicsWorld::new();
        physics.spawn_ground(15.0); // Ground plane at y = 15.0m

        // Query ground from y = 50.0m
        let origin = Vec3::new(100.0, 50.0, 100.0);
        let contact = physics.query_ground_contact(origin, 100.0);
        assert!(contact.is_some(), "Ground should be hit");
        let (dist, normal) = contact.unwrap();
        assert!(
            (dist - 35.0).abs() < 1e-3,
            "Distance should be 50 - 15 = 35m"
        );
        assert!(
            (normal.y - 1.0).abs() < 1e-4,
            "Normal should point straight up"
        );

        // Safe spawn vehicle at (x=20, z=30)
        let (body, _collider, spawn_y) =
            physics.spawn_safe_vehicle(20.0, 30.0, 0.2, [1.0, 0.5, 2.0], 1500.0);
        // spawn_y should be ground (15.0) + half_extent_y (0.5) + clearance (0.2) = 15.7m
        assert!((spawn_y - 15.7).abs() < 1e-3);

        let initial_pos = physics.rigid_body_set[body].translation();
        assert!((initial_pos.y - 15.7).abs() < 1e-3);

        // Step physics for 10 ticks: vehicle should remain stable without falling through
        for _ in 0..10 {
            physics.step(1.0 / 60.0);
        }

        let settled_pos = physics.rigid_body_set[body].translation();
        assert!(
            settled_pos.y >= 15.4 && settled_pos.y <= 15.7,
            "Vehicle should settle gently on ground, got y={}",
            settled_pos.y
        );
    }

    #[test]
    fn test_physics_building_collision_filtering() {
        let mut physics = PhysicsWorld::new();
        physics.spawn_ground(0.0);

        // Spawn a building obstacle at (0, 0, 10) with half extents (2, 5, 2)
        physics.spawn_building_collider(Vec3::new(0.0, 5.0, 10.0), Vec3::new(2.0, 5.0, 2.0));

        // Spawn vehicle moving towards the building
        let (body, _) = physics.spawn_vehicle_body(0.0, 1.0, 0.0, [1.0, 0.5, 2.0], 1500.0);
        physics.rigid_body_set[body].set_linvel(vector![0.0, 0.0, 15.0], true); // 15 m/s towards building

        // Step physics 60 times (1 second)
        for _ in 0..60 {
            physics.step(1.0 / 60.0);
        }

        let final_pos = physics.rigid_body_set[body].translation();
        // Vehicle front starts at z = 2.0m, building front face is at z = 10.0 - 2.0 = 8.0m.
        // Vehicle should collide and be stopped before z reaches 8.0m!
        assert!(
            final_pos.z < 8.0,
            "Vehicle should not penetrate building (expected z < 8.0, got z = {})",
            final_pos.z
        );
    }
}
