//! Headless Bevy ECS simulation world and execution loop.

use crate::components::{EntityTag, GeoPosition, LocalTransform, Velocity};
use crate::solar::{compute_solar_position, SolarPosition};
use crate::time_weather::{AtmosphericState, SimulationClock};
use bevy_ecs::prelude::*;
use realearth_shared::coordinates::Wgs84Coord;

/// Global simulation observer anchor resource.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ActiveObserver {
    pub coord: Wgs84Coord,
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
                velocity,
            ))
            .id()
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
