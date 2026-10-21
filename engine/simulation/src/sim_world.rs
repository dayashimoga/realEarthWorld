//! Headless Bevy ECS simulation world and execution loop.

use crate::components::{EntityTag, GeoPosition, InterpolatedTransform, LocalTransform, Velocity};
use crate::solar::{compute_solar_position, SolarPosition};
use crate::time_weather::{AtmosphericState, SimulationClock};
use bevy_ecs::prelude::*;
use realearth_shared::coordinates::Wgs84Coord;

/// Global simulation observer anchor resource.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ActiveObserver {
    pub coord: Wgs84Coord,
}

/// Fixed timestep accumulator resource for physics/presentation decoupling.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct FixedTimestepAccumulator {
    pub accumulator: f64,
    pub fixed_dt: f64,
    pub alpha: f32,
    pub tick_count: u64,
}

impl Default for FixedTimestepAccumulator {
    fn default() -> Self {
        Self {
            accumulator: 0.0,
            fixed_dt: 1.0 / 60.0,
            alpha: 0.0,
            tick_count: 0,
        }
    }
}

/// Headless decoupled simulation runtime engine.
pub struct RealEarthSimWorld {
    pub world: World,
    schedule: Schedule,
}

impl RealEarthSimWorld {
    /// Creates a new headless simulation world anchored at a geographic coordinate.
    pub fn new(anchor: Wgs84Coord) -> Self {
        let mut world = World::new();

        world.insert_resource(ActiveObserver { coord: anchor });
        world.insert_resource(SimulationClock::default());
        world.insert_resource(AtmosphericState::default());
        world.insert_resource(FixedTimestepAccumulator::default());

        let mut schedule = Schedule::default();
        schedule.add_systems((time_progression_system, kinematic_movement_system));

        Self { world, schedule }
    }

    /// Advances the simulation by a fixed time step in seconds (default 60Hz: 0.016667s).
    pub fn tick(&mut self, delta_seconds: f64) {
        // Run scheduled ECS systems
        self.schedule.run(&mut self.world);

        // Advance simulation clock
        if let Some(mut clock) = self.world.get_resource_mut::<SimulationClock>() {
            clock.advance(delta_seconds);
        }
    }

    /// Advances simulation with a fixed-timestep accumulator, updating InterpolatedTransform render states.
    pub fn tick_fixed_accumulator(&mut self, frame_dt: f64) -> usize {
        let fixed_dt = self.world.resource::<FixedTimestepAccumulator>().fixed_dt;
        let mut accum = self
            .world
            .resource::<FixedTimestepAccumulator>()
            .accumulator
            + frame_dt.min(0.25);
        let mut steps = 0;

        while accum >= fixed_dt {
            // 1. Store previous state
            for mut interp in self
                .world
                .query::<&mut InterpolatedTransform>()
                .iter_mut(&mut self.world)
            {
                interp.previous = interp.current;
            }

            // 2. Step fixed-rate schedule
            self.schedule.run(&mut self.world);
            if let Some(mut clock) = self.world.get_resource_mut::<SimulationClock>() {
                clock.advance(fixed_dt);
            }

            // 3. Update current state from LocalTransform
            let mut query = self
                .world
                .query::<(&LocalTransform, &mut InterpolatedTransform)>();
            for (local, mut interp) in query.iter_mut(&mut self.world) {
                interp.current = *local;
            }

            accum -= fixed_dt;
            steps += 1;
        }

        let alpha = (accum / fixed_dt) as f32;
        if let Some(mut ft) = self.world.get_resource_mut::<FixedTimestepAccumulator>() {
            ft.accumulator = accum;
            ft.alpha = alpha;
            ft.tick_count += steps as u64;
        }

        // 4. Update presentation interpolation
        for mut interp in self
            .world
            .query::<&mut InterpolatedTransform>()
            .iter_mut(&mut self.world)
        {
            interp.update_render(alpha);
        }

        steps
    }

    /// Computes the active solar position relative to the observer anchor.
    pub fn current_solar_position(&self) -> SolarPosition {
        let observer = self.world.resource::<ActiveObserver>().coord;
        let clock = self.world.resource::<SimulationClock>();
        compute_solar_position(&observer, clock.time_hours, clock.day_of_year)
    }

    /// Spawns a tracked dynamic simulation entity.
    pub fn spawn_entity(
        &mut self,
        name: &str,
        id: u64,
        geo: Wgs84Coord,
        transform: LocalTransform,
        velocity: Velocity,
    ) -> Entity {
        self.world
            .spawn((
                EntityTag {
                    id,
                    name: name.to_string(),
                },
                GeoPosition::new(geo),
                transform,
                InterpolatedTransform::new(transform),
                velocity,
            ))
            .id()
    }

    /// Registers or syncs a world chunk into the Bevy ECS hierarchy.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_chunk_entity(
        &mut self,
        cx: i32,
        cy: i32,
        lod: u32,
        center: glam::Vec3,
        radius: f32,
        has_terrain: bool,
        has_roads: bool,
        has_buildings: bool,
        has_water: bool,
        has_vegetation: bool,
    ) -> Entity {
        use crate::components::{
            BoundingSphere, BuildingMeshComponent, ChunkEntity, RoadMeshComponent,
            TerrainMeshComponent, VegetationMeshComponent, WaterMeshComponent,
        };

        let mut entity_builder = self.world.spawn((
            ChunkEntity {
                chunk_x: cx,
                chunk_y: cy,
                lod,
            },
            BoundingSphere { center, radius },
            LocalTransform {
                translation: center,
                ..Default::default()
            },
        ));

        if has_terrain {
            entity_builder.insert(TerrainMeshComponent);
        }
        if has_roads {
            entity_builder.insert(RoadMeshComponent);
        }
        if has_buildings {
            entity_builder.insert(BuildingMeshComponent);
        }
        if has_water {
            entity_builder.insert(WaterMeshComponent);
        }
        if has_vegetation {
            entity_builder.insert(VegetationMeshComponent);
        }

        entity_builder.id()
    }
}

/// ECS System: advances entity local transforms by their linear and angular velocities.
fn kinematic_movement_system(mut query: Query<(&mut LocalTransform, &Velocity)>) {
    let dt = 1.0 / 60.0_f32; // Fixed 60Hz step
    for (mut transform, vel) in query.iter_mut() {
        transform.translation += vel.linear * dt;
        transform.rotation_yaw += vel.angular_yaw * dt;
    }
}

/// ECS System: dummy/stub for future time progression systems.
fn time_progression_system(_clock: Option<Res<SimulationClock>>) {
    // Scheduled time updates
}
