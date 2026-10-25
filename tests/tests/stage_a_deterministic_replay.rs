//! STAGE A — DETERMINISTIC INPUT REPLAY & NUMERICAL COMPARISON HARNESS
//!
//! Provides permanent test infrastructure verifying:
//! 1. Fixed-timestep (60 Hz) input replay determinism across multiple runs.
//! 2. Exact numerical snapshot capture and restore fidelity (rewind & branch).
//! 3. Bitwise/tight numerical convergence tolerances (Delta pos < 1e-5, Delta vel < 1e-5).
//! 4. Multimodal vehicle consistency (Car, Motorcycle, Transit Bus, Heavy Truck).

use glam::Vec3;
use realearth_vehicles::{
    DynamicVehicle, FlatGround, HeavyTruckModel, MotorcycleModel, TransitBusModel, TransmissionMode,
    VehicleConfig, VehicleInputs, VehicleStateSnapshot,
};

const DT: f32 = 1.0 / 60.0;
const REPLAY_FRAMES: usize = 300;

fn generate_test_input_sequence() -> Vec<VehicleInputs> {
    let mut inputs = Vec::with_capacity(REPLAY_FRAMES);
    for frame in 0..REPLAY_FRAMES {
        let input = if frame < 60 {
            // Accelerate forward
            VehicleInputs {
                throttle: 0.8,
                brake: 0.0,
                steer: 0.0,
                gear_mode: TransmissionMode::Drive,
                handbrake: false,
            }
        } else if frame < 120 {
            // Steer left while throttling
            VehicleInputs {
                throttle: 0.6,
                brake: 0.0,
                steer: -0.4,
                gear_mode: TransmissionMode::Drive,
                handbrake: false,
            }
        } else if frame < 180 {
            // Counter-steer right
            VehicleInputs {
                throttle: 0.5,
                brake: 0.0,
                steer: 0.45,
                gear_mode: TransmissionMode::Drive,
                handbrake: false,
            }
        } else if frame < 240 {
            // Heavy braking to stop
            VehicleInputs {
                throttle: 0.0,
                brake: 0.9,
                steer: 0.0,
                gear_mode: TransmissionMode::Drive,
                handbrake: false,
            }
        } else {
            // Reverse acceleration
            VehicleInputs {
                throttle: 0.7,
                brake: 0.0,
                steer: -0.2,
                gear_mode: TransmissionMode::Reverse,
                handbrake: false,
            }
        };
        inputs.push(input);
    }
    inputs
}

#[test]
fn test_deterministic_input_replay_bitwise_consistency() {
    println!("\n🔁 [TEST INFRASTRUCTURE] Auditing Fixed-Timestep Replay Determinism across Multiple Runs...");

    let ground = FlatGround::new(0.0);
    let car_config = VehicleConfig::sports_coupe();
    let eq_y = car_config.equilibrium_ride_height();
    let input_sequence = generate_test_input_sequence();

    // Run 1: Primary Run
    let mut car1 = DynamicVehicle::new(car_config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);
    let mut history1: Vec<VehicleStateSnapshot> = Vec::with_capacity(REPLAY_FRAMES);

    for input in &input_sequence {
        car1.step(input, &ground, DT);
        history1.push(car1.snapshot());
    }

    // Run 2: Replay Run
    let mut car2 = DynamicVehicle::new(car_config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);
    let mut history2: Vec<VehicleStateSnapshot> = Vec::with_capacity(REPLAY_FRAMES);

    for input in &input_sequence {
        car2.step(input, &ground, DT);
        history2.push(car2.snapshot());
    }

    // Verify numerical comparison harness
    let mut max_pos_delta = 0.0f32;
    let mut max_vel_delta = 0.0f32;

    for (f, (s1, s2)) in history1.iter().zip(history2.iter()).enumerate() {
        let pos_d = (s1.position - s2.position).length();
        let vel_d = (s1.linear_velocity - s2.linear_velocity).length();

        if pos_d > max_pos_delta {
            max_pos_delta = pos_d;
        }
        if vel_d > max_vel_delta {
            max_vel_delta = vel_d;
        }

        assert!(
            pos_d < 1e-5,
            "Frame {} position divergence: {:.7}m >= 1e-5m tolerance",
            f,
            pos_d
        );
        assert!(
            vel_d < 1e-5,
            "Frame {} velocity divergence: {:.7}m/s >= 1e-5m/s tolerance",
            f,
            vel_d
        );
    }

    println!(
        "  ✓ Car Replay (300 frames @ 60Hz): Max pos delta = {:.2e}m, Max vel delta = {:.2e}m/s",
        max_pos_delta, max_vel_delta
    );
}

#[test]
fn test_snapshot_rewind_and_branch_determinism() {
    println!("\n⏪ [TEST INFRASTRUCTURE] Auditing Snapshot Rewind and Branching Invariance...");

    let ground = FlatGround::new(0.0);
    let car_config = VehicleConfig::sports_coupe();
    let eq_y = car_config.equilibrium_ride_height();
    let input_sequence = generate_test_input_sequence();

    let mut car = DynamicVehicle::new(car_config, Vec3::new(0.0, eq_y, 0.0), 0.0);

    // Step to frame 150 and capture checkpoint
    for input in &input_sequence[..150] {
        car.step(input, &ground, DT);
    }
    let checkpoint_150 = car.snapshot();

    // Continue to frame 300
    for input in &input_sequence[150..300] {
        car.step(input, &ground, DT);
    }
    let end_state_run_a = car.snapshot();

    // Rewind back to checkpoint 150
    car.restore_snapshot(&checkpoint_150);
    assert_eq!(car.position, checkpoint_150.position);
    assert_eq!(car.linear_velocity, checkpoint_150.linear_velocity);

    // Step to frame 300 again along identical inputs
    for input in &input_sequence[150..300] {
        car.step(input, &ground, DT);
    }
    let end_state_run_b = car.snapshot();

    let pos_diff = (end_state_run_a.position - end_state_run_b.position).length();
    let vel_diff = (end_state_run_a.linear_velocity - end_state_run_b.linear_velocity).length();

    println!(
        "  ✓ Snapshot Rewind Invariance: Delta pos = {:.2e}m, Delta vel = {:.2e}m/s",
        pos_diff, vel_diff
    );

    assert!(
        pos_diff < 1e-5,
        "Snapshot rewind produced non-deterministic position: diff={}",
        pos_diff
    );
    assert!(
        vel_diff < 1e-5,
        "Snapshot rewind produced non-deterministic velocity: diff={}",
        vel_diff
    );
}

#[test]
fn test_multimodal_replay_determinism() {
    println!("\n🏍️ [TEST INFRASTRUCTURE] Auditing Multimodal Transport Replay Determinism...");

    let ground = FlatGround::new(0.0);
    let inputs = generate_test_input_sequence();

    // 1. Motorcycle
    let mut bike1 = MotorcycleModel::new(Vec3::new(0.0, 0.35, 0.0), 0.0);
    let mut bike2 = MotorcycleModel::new(Vec3::new(0.0, 0.35, 0.0), 0.0);
    for inp in &inputs {
        bike1.step(inp, &ground, DT);
        bike2.step(inp, &ground, DT);
    }
    let bike_diff = (bike1.position - bike2.position).length();
    assert!(
        bike_diff < 1e-5,
        "Motorcycle replay diverged: diff={}",
        bike_diff
    );
    println!("  ✓ Motorcycle Replay: Delta pos = {:.2e}m", bike_diff);

    // 2. Transit Bus
    let mut bus1 = TransitBusModel::new(Vec3::new(0.0, 0.95, 0.0), 0.0);
    let mut bus2 = TransitBusModel::new(Vec3::new(0.0, 0.95, 0.0), 0.0);
    for inp in &inputs {
        bus1.step(inp, &ground, DT);
        bus2.step(inp, &ground, DT);
    }
    let bus_diff = (bus1.position - bus2.position).length();
    assert!(bus_diff < 1e-5, "Transit Bus replay diverged: diff={}", bus_diff);
    println!("  ✓ Transit Bus Replay: Delta pos = {:.2e}m", bus_diff);

    // 3. Heavy Truck
    let mut truck1 = HeavyTruckModel::new(Vec3::new(0.0, 1.25, 0.0), 0.0);
    let mut truck2 = HeavyTruckModel::new(Vec3::new(0.0, 1.25, 0.0), 0.0);
    for inp in &inputs {
        truck1.step(inp, &ground, DT);
        truck2.step(inp, &ground, DT);
    }
    let truck_diff = (truck1.tractor_position - truck2.tractor_position).length();
    assert!(
        truck_diff < 1e-5,
        "Heavy Truck replay diverged: diff={}",
        truck_diff
    );
    println!("  ✓ Heavy Truck Replay: Delta pos = {:.2e}m", truck_diff);
}
