//! Headless Bevy ECS simulation engine, astronomical solar positioning, and time progression.

pub mod components;
pub mod sim_world;
pub mod solar;
pub mod time_weather;

pub use components::{
    BoundingSphere, BuildingMeshComponent, ChunkEntity, EcefPosition, EntityTag, GeoPosition,
    InterpolatedTransform, LocalTransform, RoadMeshComponent, TerrainMeshComponent,
    VegetationMeshComponent, Velocity, WaterMeshComponent,
};
pub use sim_world::{ActiveObserver, FixedTimestepAccumulator, RealEarthSimWorld};
pub use solar::{compute_solar_position, SolarPosition};
pub use time_weather::{AtmosphericState, SimulationClock, WeatherType};

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use realearth_shared::coordinates::Wgs84Coord;

    #[test]
    fn test_simulation_world_ticks_and_movement() {
        let vidhana = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let mut sim = RealEarthSimWorld::new(vidhana);

        let entity = sim.spawn_entity(
            "Test Car",
            101,
            vidhana,
            LocalTransform::default(),
            Velocity {
                linear: Vec3::new(10.0, 0.0, 0.0), // 10 m/s East
                angular_yaw: 0.0,
            },
        );

        // Tick 60 times at 1/60s = exactly 1 second
        for _ in 0..60 {
            sim.tick(1.0 / 60.0);
        }

        let transform = sim.world.get::<LocalTransform>(entity).unwrap();
        // After 1 second at 10 m/s, translation.x should equal 10.0m
        assert!((transform.translation.x - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_solar_position_bengaluru_noon_and_midnight() {
        let vidhana = Wgs84Coord::new(12.9797, 77.5907, 920.0);

        // 1. High Noon (12:00) at Vernal Equinox (Day 80)
        let noon = compute_solar_position(&vidhana, 12.0, 80);
        assert!(noon.is_day);
        assert!(
            noon.elevation_deg > 70.0,
            "Expected elevation > 70 deg near equator at noon"
        );
        assert!(noon.sun_intensity > 0.8);
        assert!(noon.sun_direction_enu.y > 0.9, "Sun should be overhead");

        // 2. Midnight (0:00)
        let midnight = compute_solar_position(&vidhana, 0.0, 80);
        assert!(!midnight.is_day);
        assert!(midnight.elevation_deg < -50.0);
        assert_eq!(midnight.sun_intensity, 0.0);
    }

    #[test]
    fn test_simulation_clock_time_progression() {
        let mut clock = SimulationClock {
            time_hours: 12.0,
            time_scale: 60.0, // 60x speed: 1 minute per real second
            ..Default::default()
        };

        // Advance 60 seconds of real time = 60 minutes = 1 hour
        clock.advance(60.0);
        assert!((clock.time_hours - 13.0).abs() < 1e-6);
    }

    #[test]
    fn test_simulation_determinism() {
        let vidhana = Wgs84Coord::new(12.9797, 77.5907, 920.0);

        let mut sim1 = RealEarthSimWorld::new(vidhana);
        let mut sim2 = RealEarthSimWorld::new(vidhana);

        let e1 = sim1.spawn_entity(
            "E1",
            1,
            vidhana,
            LocalTransform::default(),
            Velocity {
                linear: Vec3::new(5.0, 0.0, 2.0),
                angular_yaw: 0.1,
            },
        );
        let e2 = sim2.spawn_entity(
            "E1",
            1,
            vidhana,
            LocalTransform::default(),
            Velocity {
                linear: Vec3::new(5.0, 0.0, 2.0),
                angular_yaw: 0.1,
            },
        );

        // Run both for 500 ticks
        for _ in 0..500 {
            sim1.tick(1.0 / 60.0);
            sim2.tick(1.0 / 60.0);
        }

        let t1 = sim1.world.get::<LocalTransform>(e1).unwrap();
        let t2 = sim2.world.get::<LocalTransform>(e2).unwrap();

        assert_eq!(t1.translation, t2.translation);
        assert_eq!(t1.rotation_yaw, t2.rotation_yaw);
    }

    #[test]
    fn test_fixed_timestep_accumulator_and_interpolation() {
        let vidhana = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let mut sim = RealEarthSimWorld::new(vidhana);

        let entity = sim.spawn_entity(
            "Car",
            42,
            vidhana,
            LocalTransform::default(),
            Velocity {
                linear: Vec3::new(60.0, 0.0, 0.0), // 60 m/s
                angular_yaw: 0.0,
            },
        );

        // Advance 0.025s (which is 1.5 fixed ticks of 1/60s = 0.016667s)
        let steps = sim.tick_fixed_accumulator(0.025);
        assert_eq!(steps, 1, "Exactly 1 fixed step should have executed");

        let interp = sim.world.get::<InterpolatedTransform>(entity).unwrap();
        // After 1 step (1/60s) at 60m/s: previous was 0.0m, current is 1.0m.
        // Remainder accumulator is 0.025 - 0.016667 = 0.008333s => alpha = 0.5.
        // Render translation.x should be interpolated to 0.5m!
        assert!((interp.previous.translation.x - 0.0).abs() < 1e-4);
        assert!((interp.current.translation.x - 1.0).abs() < 1e-4);
        assert!(
            (interp.render.translation.x - 0.5).abs() < 0.05,
            "Render transform should be interpolated between previous and current with alpha ~0.5, got {}",
            interp.render.translation.x
        );
    }
}
