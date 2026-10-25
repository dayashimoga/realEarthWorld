//! STAGE B Master Certification Test Suite
//! Comprehensive quantitative certification of authoritative vehicle physics,
//! canonical coordinate conventions, control input correctness, static equilibria,
//! multimodal handling, and non-regression guarantees.

use glam::{Mat3, Quat, Vec2, Vec3};
use realearth_vehicles::building_collision::{BuildingCollisionWorld, BuildingFootprintCollider};
use realearth_vehicles::dynamics::{DynamicVehicle, TransmissionMode, VehicleConfig, VehicleInputs};
use realearth_vehicles::ground::{FlatGround, SlopedGround};
use realearth_vehicles::multimodal::road_transport::{HeavyTruckModel, MotorcycleModel, TransitBusModel};
use std::f32::consts::{FRAC_PI_2, PI};

const DT: f32 = 1.0 / 60.0;

// ============================================================================
// 1. CANONICAL CONTROL SIGNS ACROSS ALL CARDINAL HEADINGS
// ============================================================================

#[test]
fn test_canonical_control_signs_across_all_cardinal_headings() {
    println!("\n🔍 [STAGE B.1] Auditing Canonical Control Signs Across All 4 Cardinal Headings...");
    let ground = FlatGround::new(0.0);

    let headings = [
        ("North", 0.0f32, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
        ("East", FRAC_PI_2, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
        ("South", PI, Vec3::new(0.0, 0.0, -1.0), Vec3::new(-1.0, 0.0, 0.0)),
        ("West", -FRAC_PI_2, Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
    ];

    let config = VehicleConfig::sports_coupe();
    let eq_y = config.equilibrium_ride_height();

    for (name, yaw, expected_fwd, expected_right) in headings {
        // A. Forward Motion [W]
        let mut car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), yaw);
        let rot = Mat3::from_quat(car.orientation);
        let fwd = rot.col(2);
        let right = rot.col(0);

        assert!((fwd - expected_fwd).length() < 0.01, "{}: Forward vector mismatch", name);
        assert!((right - expected_right).length() < 0.01, "{}: Right vector mismatch", name);

        let inputs_fwd = VehicleInputs {
            throttle: 1.0,
            brake: 0.0,
            steer: 0.0,
            handbrake: false,
            gear_mode: TransmissionMode::Drive,
        };

        for _ in 0..60 {
            car.step(&inputs_fwd, &ground, DT);
        }

        let fwd_displacement = car.position.dot(expected_fwd);
        assert!(fwd_displacement > 1.0, "{}: [W] must produce forward displacement (got {:.2}m)", name, fwd_displacement);
        let lat_drift = car.position.dot(expected_right).abs();
        assert!(lat_drift < 0.05, "{}: Straight forward acceleration must not drift laterally (got {:.3}m)", name, lat_drift);

        // B. Reverse Motion [S]
        let mut car_rev = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), yaw);
        let inputs_rev = VehicleInputs {
            throttle: 0.85,
            brake: 0.0,
            steer: 0.0,
            handbrake: false,
            gear_mode: TransmissionMode::Reverse,
        };

        for _ in 0..60 {
            car_rev.step(&inputs_rev, &ground, DT);
        }

        let rev_displacement = car_rev.position.dot(expected_fwd);
        assert!(rev_displacement < -1.0, "{}: [S] Reverse must produce backward displacement (got {:.2}m)", name, rev_displacement);

        // C. Steer Left [A]
        let mut car_left = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), yaw);
        let inputs_left = VehicleInputs {
            throttle: 0.5,
            brake: 0.0,
            steer: -0.65, // Full left
            handbrake: false,
            gear_mode: TransmissionMode::Drive,
        };

        for _ in 0..30 {
            car_left.step(&inputs_left, &ground, DT);
        }

        // Steer left produces negative yaw rate in right-handed convention
        assert!(car_left.angular_velocity.y < -0.05, "{}: Steer Left [A] must produce negative yaw rate (got {:.3} rad/s)", name, car_left.angular_velocity.y);
        let left_displacement = car_left.position.dot(expected_right);
        assert!(left_displacement < -0.02, "{}: Steer Left [A] must deflect vehicle to the left (got {:.3}m)", name, left_displacement);

        // D. Steer Right [D]
        let mut car_right = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), yaw);
        let inputs_right = VehicleInputs {
            throttle: 0.5,
            brake: 0.0,
            steer: 0.65, // Full right
            handbrake: false,
            gear_mode: TransmissionMode::Drive,
        };

        for _ in 0..30 {
            car_right.step(&inputs_right, &ground, DT);
        }

        // Steer right produces positive yaw rate in right-handed convention
        assert!(car_right.angular_velocity.y > 0.05, "{}: Steer Right [D] must produce positive yaw rate (got {:.3} rad/s)", name, car_right.angular_velocity.y);
        let right_displacement = car_right.position.dot(expected_right);
        assert!(right_displacement > 0.02, "{}: Steer Right [D] must deflect vehicle to the right (got {:.3}m)", name, right_displacement);

        println!("  ✓ Cardinal Heading {} ({:.1} rad): W/S/A/D Verified Cleanly", name, yaw);
    }
}

// ============================================================================
// 2. PARK MODE STANDSTILL HOLDING ON LEVEL AND SLOPED ROADS
// ============================================================================

#[test]
fn test_park_mode_hold_stationary_level_and_slopes() {
    println!("\n🔍 [STAGE B.2] Auditing Park Mode Standstill Stability (Level & Slopes)...");

    let test_slopes = [
        ("Level Ground (0%)", 0.0f32),
        ("San Francisco Hill (8%)", 0.08f32),
        ("Steep Lombard Grade (20%)", 0.20f32),
    ];

    let inputs_park = VehicleInputs {
        throttle: 0.0,
        brake: 1.0,
        steer: 0.0,
        handbrake: true,
        gear_mode: TransmissionMode::Park,
    };

    let config = VehicleConfig::sports_coupe();
    let eq_y = config.equilibrium_ride_height();

    for (desc, grade) in test_slopes {
        let ground = SlopedGround::new(0.0, 0.0, grade);
        let mut car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);
        car.orientation = Quat::from_rotation_x(-grade.atan());
        let initial_pos = car.position;

        // Step for 2 full seconds (120 frames) in Park
        for _ in 0..120 {
            car.step(&inputs_park, &ground, DT);
        }

        let delta = car.position - initial_pos;
        let displacement = delta.length();
        let speed = car.linear_velocity.length();

        println!("  • {}: Δx={:.6}m, Δy={:.6}m, Δz={:.6}m, Total={:.6}m, Speed={:.6} m/s", desc, delta.x, delta.y, delta.z, displacement, speed);
        assert!(displacement < 1e-4, "Park mode on {} must remain motionless (moved {:.6}m)", desc, displacement);
        assert!(speed < 1e-4, "Park mode on {} velocity must be zero (got {:.6} m/s)", desc, speed);
        assert!(car.angular_velocity.length() < 1e-4, "Park mode angular velocity must be zero");
    }
}

// ============================================================================
// 3. HILL-HOLD ASSIST AND IDLE CREEP IN DRIVE (D1)
// ============================================================================

#[test]
fn test_hill_hold_assist_and_idle_creep_in_drive() {
    println!("\n🔍 [STAGE B.3] Auditing Hill-Hold Assist and Idle Creep in Drive...");

    let config = VehicleConfig::sports_coupe();
    let eq_y = config.equilibrium_ride_height();

    // A. Hill-Hold Assist on 8% slope in Drive with zero throttle
    let sf_ground = SlopedGround::new(0.0, 0.0, 0.08);
    let mut car_hill = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);
    let initial_z = car_hill.position.z;

    let inputs_idle_drive = VehicleInputs {
        throttle: 0.0,
        brake: 0.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    for _ in 0..120 {
        car_hill.step(&inputs_idle_drive, &sf_ground, DT);
    }

    let rollback = car_hill.position.z - initial_z;
    println!("  • 8% Hill in Drive (zero throttle): Z-displacement = {:.6}m (speed={:.2} km/h)", rollback, car_hill.speed_kph);
    assert!(rollback >= -0.01, "Hill-Hold Assist must prevent backward rollback (got {:.4}m)", rollback);

    // B. Idle Forward Creep on Level Ground in Drive
    let flat_ground = FlatGround::new(0.0);
    let mut car_flat = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);

    for _ in 0..180 { // 3 seconds
        car_flat.step(&inputs_idle_drive, &flat_ground, DT);
    }

    let creep_speed = car_flat.linear_velocity.z;
    println!("  • Flat Ground Idle Creep in Drive: Speed = {:.2} km/h (vz={:.3} m/s)", car_flat.speed_kph, creep_speed);
    assert!(creep_speed > 0.05, "Idle in Drive must produce gentle forward creep (got vz={:.3} m/s)", creep_speed);
    assert!(creep_speed < 2.5, "Idle creep speed must not exceed ~8 km/h (got vz={:.3} m/s)", creep_speed);
}

// ============================================================================
// 4. COASTING AND GRADUAL DECELERATION IN NEUTRAL
// ============================================================================

#[test]
fn test_coasting_and_gradual_deceleration_in_neutral() {
    println!("\n🔍 [STAGE B.4] Auditing Coasting and Monotonic Deceleration in Neutral...");
    let ground = FlatGround::new(0.0);

    let config = VehicleConfig::sports_coupe();
    let eq_y = config.equilibrium_ride_height();
    let mut car = DynamicVehicle::new(config, Vec3::new(0.0, eq_y, 0.0), 0.0);
    // Initialize car cruising at 54 km/h (15.0 m/s)
    car.linear_velocity = Vec3::new(0.0, 0.0, 15.0);
    car.speed_kph = 54.0;

    let inputs_neutral = VehicleInputs {
        throttle: 0.0,
        brake: 0.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Neutral,
    };

    let mut prev_speed = car.linear_velocity.z;

    for second in 1..=4 {
        for _ in 0..60 {
            car.step(&inputs_neutral, &ground, DT);
        }
        let current_speed = car.linear_velocity.z;
        println!("  • Second {}: Speed = {:.2} m/s ({:.1} km/h)", second, current_speed, car.speed_kph);
        assert!(current_speed < prev_speed, "Speed must decrease monotonically during coasting");
        assert!(current_speed > 0.0, "Car should not come to an abrupt dead stop while coasting");
        assert!(car.linear_velocity.x.abs() < 0.05, "Coasting on level road must not develop lateral drift");
        prev_speed = current_speed;
    }
}

// ============================================================================
// 5. PROGRESSIVE SERVICE BRAKING AND FULL STOP
// ============================================================================

#[test]
fn test_progressive_service_braking_and_full_stop() {
    println!("\n🔍 [STAGE B.5] Auditing Progressive Service Braking Deceleration...");
    let ground = FlatGround::new(0.0);

    let config = VehicleConfig::sports_coupe();
    let eq_y = config.equilibrium_ride_height();
    let mut car = DynamicVehicle::new(config, Vec3::new(0.0, eq_y, 0.0), 0.0);
    // Initialize cruising at 72 km/h (20.0 m/s)
    car.linear_velocity = Vec3::new(0.0, 0.0, 20.0);
    car.speed_kph = 72.0;

    let inputs_brake = VehicleInputs {
        throttle: 0.0,
        brake: 0.85,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    let mut stopped = false;
    let mut stop_time = 0.0f32;

    for frame in 0..180 {
        car.step(&inputs_brake, &ground, DT);
        if car.speed_kph.abs() < 0.05 && !stopped {
            stopped = true;
            stop_time = frame as f32 * DT;
        }
    }

    println!("  • Full stop achieved in {:.2} seconds. Final Speed = {:.3} km/h, Final Vel = {:?}", stop_time, car.speed_kph, car.linear_velocity);
    assert!(stopped, "Vehicle must come to a complete stop under 0.85 service brake");
    assert!(stop_time > 0.8 && stop_time < 2.5, "Stopping time ({:.2}s) must be physically realistic (0.8s - 2.5s)", stop_time);
    assert!(car.linear_velocity.z >= 0.0, "Service braking in Drive must not trigger backward reverse motion");
    assert!(car.linear_velocity.length() < 0.01, "Vehicle must remain stopped under sustained brake");
}

// ============================================================================
// 6. STEERING RELEASE AND ACTIVE CASTER SELF-CENTERING
// ============================================================================

#[test]
fn test_steer_release_self_centering() {
    println!("\n🔍 [STAGE B.6] Auditing Steering Wheel Release Self-Centering Active Damping...");
    let ground = FlatGround::new(0.0);

    let config = VehicleConfig::sports_coupe();
    let eq_y = config.equilibrium_ride_height();
    let mut car = DynamicVehicle::new(config, Vec3::new(0.0, eq_y, 0.0), 0.0);
    // Turn aggressively into a corner
    let inputs_turn = VehicleInputs {
        throttle: 0.6,
        brake: 0.0,
        steer: 0.5,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    for _ in 0..45 {
        car.step(&inputs_turn, &ground, DT);
    }

    let initial_yaw_rate = car.angular_velocity.y;
    println!("  • Peak Yaw Rate during cornering: {:.3} rad/s", initial_yaw_rate);
    assert!(initial_yaw_rate.abs() > 0.15, "Car must develop turning yaw rate under steering");

    // Driver releases the steering wheel
    let inputs_release = VehicleInputs {
        throttle: 0.4,
        brake: 0.0,
        steer: 0.0, // Hands off
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    for _ in 0..60 {
        car.step(&inputs_release, &ground, DT);
    }

    let settled_yaw_rate = car.angular_velocity.y;
    println!("  • Settled Yaw Rate 1.0s after releasing steer: {:.4} rad/s", settled_yaw_rate);
    assert!(settled_yaw_rate.abs() < 0.04, "Caster self-centering must damp yaw rate toward zero (got {:.4} rad/s)", settled_yaw_rate);
}

// ============================================================================
// 7. MULTIMODAL CLASS-APPROPRIATE PHYSICS
// ============================================================================

#[test]
fn test_multimodal_class_appropriate_physics() {
    println!("\n🔍 [STAGE B.7] Auditing Multimodal Fleet Class-Appropriate Physics...");
    let ground = FlatGround::new(0.0);

    // A. Motorcycle: dynamic lean into turns & Park stability
    let mut moto = MotorcycleModel::new(Vec3::new(0.0, 0.35, 0.0), 0.0);
    let inputs_moto_turn = VehicleInputs {
        throttle: 0.7,
        brake: 0.0,
        steer: 0.4, // Right turn
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    for _ in 0..60 {
        moto.step(&inputs_moto_turn, &ground, DT);
    }
    println!("  • Motorcycle: Speed={:.1} km/h, Lean Angle={:.2}°", moto.speed_kph, moto.lean_angle.to_degrees());
    assert!(moto.speed_kph > 10.0, "Motorcycle must accelerate cleanly");
    assert!(moto.lean_angle.abs() > 0.05, "Motorcycle must lean dynamically into corner");

    // Park mode for motorcycle
    let mut moto_park = MotorcycleModel::new(Vec3::new(0.0, 0.35, 0.0), 0.0);
    let inputs_park = VehicleInputs {
        throttle: 0.0,
        brake: 1.0,
        steer: 0.0,
        handbrake: true,
        gear_mode: TransmissionMode::Park,
    };
    for _ in 0..60 {
        moto_park.step(&inputs_park, &ground, DT);
    }
    assert!(moto_park.speed_kph < 0.01, "Motorcycle in Park must remain motionless");

    // B. Transit Bus: heavy inertia & air brake response
    let mut bus = TransitBusModel::new(Vec3::new(0.0, 0.95, 0.0), 0.0);
    let inputs_bus = VehicleInputs {
        throttle: 1.0,
        brake: 0.0,
        steer: 0.0,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };
    for _ in 0..60 {
        bus.step(&inputs_bus, &ground, DT);
    }
    println!("  • Transit Bus (9,500 kg): Speed after 1.0s = {:.1} km/h, Comfort={:.2}", bus.speed_kph, bus.passenger_comfort_rating);
    assert!(bus.speed_kph > 2.0 && bus.speed_kph < 25.0, "Bus heavy mass must accelerate gradually");

    // C. Heavy Truck: articulated trailer yaw & rollover safety
    let mut truck = HeavyTruckModel::new(Vec3::new(0.0, 1.2, 0.0), 0.0);
    let inputs_truck_turn = VehicleInputs {
        throttle: 0.8,
        brake: 0.0,
        steer: 0.3,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };
    for _ in 0..90 {
        truck.step(&inputs_truck_turn, &ground, DT);
    }
    println!("  • Heavy Truck (14,000 kg + trailer): Speed={:.1} km/h, Articulation Angle={:.2}°", truck.speed_kph, truck.trailer_articulation_angle.to_degrees());
    assert!(!truck.is_rolled_over, "Truck must remain stable without rollover during normal turn");
    assert!(truck.trailer_articulation_angle.abs() > 0.02, "Trailer must articulate around fifth wheel");
}

// ============================================================================
// 8. FIXED-TIMESTEP INVARIANCE ACROSS FRAME RATES (30 / 60 / 120 FPS)
// ============================================================================

#[test]
fn test_fixed_timestep_invariance_30_60_120_fps() {
    println!("\n🔍 [STAGE B.8] Auditing Fixed-Timestep Physics Invariance Across Presentation Rates...");
    let ground = FlatGround::new(0.0);

    let inputs = VehicleInputs {
        throttle: 0.8,
        brake: 0.0,
        steer: 0.15,
        handbrake: false,
        gear_mode: TransmissionMode::Drive,
    };

    let config = VehicleConfig::sports_coupe();
    let eq_y = config.equilibrium_ride_height();

    // Helper: simulate 1.0 second of presentation delta time through fixed 60Hz physics accumulator
    let simulate_accumulated = |frame_dt: f32, frame_count: usize| -> (Vec3, Quat) {
        let mut car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);
        let mut accumulator = 0.0f32;
        const PHYSICS_DT: f32 = 1.0 / 60.0;

        for _ in 0..frame_count {
            accumulator = (accumulator + frame_dt).min(0.20);
            while accumulator >= PHYSICS_DT {
                car.step(&inputs, &ground, PHYSICS_DT);
                accumulator -= PHYSICS_DT;
            }
        }
        (car.position, car.orientation)
    };

    let (pos_30, orient_30) = simulate_accumulated(1.0 / 30.0, 30);   // 30 FPS for 1.0s
    let (pos_60, orient_60) = simulate_accumulated(1.0 / 60.0, 60);   // 60 FPS for 1.0s
    let (pos_120, orient_120) = simulate_accumulated(1.0 / 120.0, 120); // 120 FPS for 1.0s

    let diff_30_60 = (pos_30 - pos_60).length();
    let diff_60_120 = (pos_60 - pos_120).length();

    println!("  • Pos at 30 FPS:  {:?}", pos_30);
    println!("  • Pos at 60 FPS:  {:?}", pos_60);
    println!("  • Pos at 120 FPS: {:?}", pos_120);
    println!("  • Δ(30 vs 60):  {:.6}m", diff_30_60);
    println!("  • Δ(60 vs 120): {:.6}m", diff_60_120);

    assert!(diff_30_60 < 1e-4, "Fixed-step simulation must be identical between 30 and 60 FPS (got {:.6}m)", diff_30_60);
    assert!(diff_60_120 < 1e-4, "Fixed-step simulation must be identical between 60 and 120 FPS (got {:.6}m)", diff_60_120);
    assert!((orient_30.dot(orient_60) - 1.0).abs() < 1e-5, "Orientations must match between 30 and 60 FPS");
    assert!((orient_60.dot(orient_120) - 1.0).abs() < 1e-5, "Orientations must match between 60 and 120 FPS");
}

// ============================================================================
// 9. BUILDING COLLISION ZERO PENETRATION OUTWARD RESOLUTION
// ============================================================================

#[test]
fn test_building_collision_zero_penetration_outward_resolution() {
    println!("\n🔍 [STAGE B.9] Auditing Building Collision Zero Penetration Outward Resolution...");

    let mut collision_world = BuildingCollisionWorld::new();
    // Building located in the positive X/Z quadrant: x in [10, 30], z in [10, 30]
    let collider = BuildingFootprintCollider::new(
        101,
        0.0,
        45.0,
        vec![
            Vec2::new(10.0, 10.0),
            Vec2::new(30.0, 10.0),
            Vec2::new(30.0, 30.0),
            Vec2::new(10.0, 30.0),
        ],
    );
    collision_world.add_collider(collider);

    // Car attempts to penetrate from street (x=9.5) into building interior (x=10.5)
    let mut vehicle_pos = Vec3::new(10.5, 0.35, 20.0); // Inside building by 0.5m
    let mut vehicle_vel = Vec3::new(15.0, 0.0, 0.0);   // Velocity heading deeper inside (+X)
    let half_extents = Vec3::new(0.9, 0.7, 2.3);

    let had_collision = collision_world.resolve_collision(&mut vehicle_pos, &mut vehicle_vel, half_extents);

    println!("  • Collision Resolved: {} | New Pos: x={:.2}, z={:.2} | New Vel: vx={:.2}", had_collision, vehicle_pos.x, vehicle_pos.z, vehicle_vel.x);
    assert!(had_collision, "Collision must be detected when inside building polygon");
    assert!(vehicle_pos.x < 10.0, "Vehicle must be ejected OUTWARD toward street exterior (x < 10.0, got x={:.2})", vehicle_pos.x);
    assert!(vehicle_vel.x <= 0.0, "Inward velocity component must be zeroed or repelled (got vx={:.2})", vehicle_vel.x);
}
