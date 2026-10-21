//! Rapier3D physics world and vehicle rigid-body collision simulation.

use rapier3d::prelude::*;

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
        }
    }

    /// Advances physics simulation by `dt` seconds.
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
    }

    /// Spawns a static flat ground collider at elevation `y`.
    pub fn spawn_ground(&mut self, y: f32) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(10_000.0, 0.1, 10_000.0)
            .translation(vector![0.0, y - 0.1, 0.0])
            .friction(0.8)
            .build();
        self.collider_set.insert(collider)
    }

    /// Spawns a dynamic vehicle body with a cuboid collider.
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

        let collider = ColliderBuilder::cuboid(half_extents[0], half_extents[1], half_extents[2])
            .friction(0.7)
            .restitution(0.1)
            .build();
        let collider_handle =
            self.collider_set
                .insert_with_parent(collider, body_handle, &mut self.rigid_body_set);

        (body_handle, collider_handle)
    }
}
