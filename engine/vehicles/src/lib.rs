//! Vehicle kinematics, automotive dynamics, camera controllers, and Rapier3D physics.

pub mod building_collision;
pub mod camera;
pub mod dynamics;
pub mod ground;
pub mod kinematics;
pub mod multimodal;
pub mod physics;
pub mod traffic;
pub mod vehicle_agent;

pub use building_collision::{BuildingCollisionWorld, BuildingFootprintCollider};
pub use camera::{VehicleCameraController, VehicleCameraPerspective};
pub use dynamics::{
    DynamicVehicle, TransmissionMode, VehicleConfig, VehicleInputs, VehicleStateSnapshot,
    WheelIndex, WheelState,
};
pub use ground::{FlatGround, FunctionalGroundQuery, GroundQuery, SlopedGround};
pub use kinematics::AckermannVehicle;
pub use multimodal::{
    ActiveTransportMode, AircraftFlight, AuthoritativeVehicleState, AviationCategory,
    DavisCoefficients, FlightInputs, HeavyTruckModel, LocomotiveType, MarineInputs, MarineVessel,
    MarineVesselType, MotorcycleModel, MultimodalAudioEvent, RoadAccessRestriction,
    RoadVehicleCategory, TrainBrakeState, TrainCar, TrainConsist, TransitBusModel,
};
pub use physics::{
    PhysicsWorld, GROUP_BUILDING, GROUP_OBSTACLE, GROUP_ROAD, GROUP_TERRAIN, GROUP_VEHICLE,
};
pub use rapier3d;
pub use traffic::{
    IdmConfig, IntersectionStopLine, PedestrianAgent, TrafficAgent, TrafficAudioEvent,
    TrafficLight, TrafficLightState, TrafficSimulation, TrafficSpatialGrid,
};
pub use vehicle_agent::PurePursuitController;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_ackermann_kinematic_straight_motion() {
        let mut car = AckermannVehicle::new(2.7, 0.6);
        car.set_velocity(20.0); // 20 m/s (~72 km/h)
        car.set_steer(0.0);

        // Step 50 ticks of 0.02s = 1.0s
        for _ in 0..50 {
            car.step(0.02);
        }

        // Distance = 20m in X direction
        assert!((car.position.x - 20.0).abs() < 1e-3);
        assert!(car.position.y.abs() < 1e-4);
        assert!(car.yaw.abs() < 1e-4);
    }

    #[test]
    fn test_ackermann_kinematic_turning() {
        let mut car = AckermannVehicle::new(2.7, 0.6);
        car.set_velocity(10.0);
        car.set_steer(0.3); // Steer left

        for _ in 0..50 {
            car.step(0.02);
        }

        assert!(car.yaw > 0.0, "Car should have yawed left");
        assert!(car.position.y > 0.0, "Car should have turned towards North");
    }

    #[test]
    fn test_rapier_physics_ground_collision() {
        let mut physics = PhysicsWorld::new();
        physics.spawn_ground(0.0);

        // Vehicle starts at y = 5.0m
        let (body, _) = physics.spawn_vehicle_body(0.0, 5.0, 0.0, [1.0, 0.5, 2.0], 1500.0);

        // Step physics 120 steps (2 seconds)
        for _ in 0..120 {
            physics.step(1.0 / 60.0);
        }

        let pos = physics.rigid_body_set[body].translation();
        // Resting on ground: ground is at 0.0, half-height is 0.5, so y ~ 0.5
        assert!(
            (pos.y - 0.5).abs() < 0.1,
            "Vehicle should rest on ground, was {}",
            pos.y
        );
        assert!(pos.y > 0.0, "Vehicle should not penetrate beneath ground");
    }

    #[test]
    fn test_pure_pursuit_waypoint_tracking() {
        let mut car = AckermannVehicle::new(2.7, 0.6);
        let waypoints = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(20.0, 5.0),
            Vec2::new(30.0, 5.0),
        ];

        let mut controller = PurePursuitController::new(waypoints, 5.0, 10.0);

        // Run tracking for 4 seconds
        for _ in 0..200 {
            let reached = controller.update(&mut car);
            car.step(0.02);
            if reached {
                break;
            }
        }

        // Car should have moved past 15m towards final goal
        assert!(car.position.x > 15.0);
    }
}
