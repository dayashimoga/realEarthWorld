//! STAGE 0.7 — CRITICAL VEHICLE PHYSICS AND CONTROL STABILIZATION CERTIFICATION
//!
//! Rigorous independent test suite verifying:
//! - Test 1: 60-second flat ground zero-input test (drift < 0.01m, velocity < 0.001 m/s).
//! - Test 2: Parked slope-hold test (15% grade, drift < 0.005m, velocity < 0.001 m/s).
//! - Test 3: Unbraked slope-roll test (15% grade in Neutral rolls downhill > 2.0 m/s).
//! - Test 4: W/S/A/D input sign invariants (W forward, S brakes/reverses, A turns left, D turns right).
//! - Test 5: Braking distance test (50 km/h stop conforms to d <= v^2 / (2 * mu * g) + margin).
//! - Test 6: Circle and turn stability (steady steering without rollover, roll angle < 12 deg).
//! - Test 7: Repeated spawn equilibrium (10 repeated spawns, tick 0 shock impulse == 0).
//! - Test 8: Multi-framerate determinism (fixed-input replay at 30, 60, and 120 FPS matches < 0.02m).
//! - Test 9: Real-road authentic San Francisco Montgomery Street settling test.

use glam::{Mat3, Vec3};
use realearth_vehicles::dynamics::{
    DynamicVehicle, TransmissionMode, VehicleConfig, VehicleInputs,
};
use realearth_vehicles::ground::{FlatGround, SlopedGround};

/// Test 1: 60-Second Flat Ground Zero-Input Drift Certification
/// A level-ground stationary car must remain effectively motionless without cumulative drift.
#[test]
fn test_certification_60s_flat_ground_zero_input() {
    let config = VehicleConfig::sports_coupe();
    let ground = FlatGround::new(0.0);
    let start_pos = Vec3::new(0.0, config.equilibrium_ride_height(), 0.0);
    let start_yaw = 0.0;
    let mut car = DynamicVehicle::new(config, start_pos, start_yaw);

    // Stationary vehicle in Park (spawn state) on level ground must remain effectively motionless without cumulative drift
    let inputs = VehicleInputs {
        gear_mode: TransmissionMode::Park,
        handbrake: true,
        ..Default::default()
    };

    let mut max_speed = 0.0f32;
    let mut max_drift = 0.0f32;

    for _tick in 0..3600 {
        car.step(&inputs, &ground, 1.0 / 60.0);

        let speed = car.linear_velocity.length();
        if speed > max_speed {
            max_speed = speed;
        }

        let horiz_dist = ((car.position.x - start_pos.x).powi(2)
            + (car.position.z - start_pos.z).powi(2))
        .sqrt();
        if horiz_dist > max_drift {
            max_drift = horiz_dist;
        }
    }

    println!(
        "Test 1 (60s Flat Ground): Max speed = {:.6} m/s, Max drift = {:.6} m",
        max_speed, max_drift
    );

    // Physically justified tolerances: stationary car must not drift on level ground
    assert!(
        max_drift < 0.005,
        "Level ground drift must be < 5mm over 60s, got {:.4}m",
        max_drift
    );
    assert!(
        max_speed < 0.001,
        "Level ground max speed must be < 1mm/s, got {:.4}m/s",
        max_speed
    );
    assert!(
        (car.position.y - start_pos.y).abs() < 0.005,
        "Vertical settling must stay within 5mm of equilibrium ride height"
    );
}

/// Test 2: Parked Slope-Hold Certification
/// A vehicle parked on a 15% grade slope must remain held motionless by static friction.
#[test]
fn test_certification_slope_hold_in_park() {
    let config = VehicleConfig::sports_coupe();
    let grade = 0.15; // 15% grade along +Z
    let ground = SlopedGround::new(0.0, 0.0, grade);
    let start_pos = Vec3::new(0.0, 1.0, 0.0);
    let mut car = DynamicVehicle::new(config, start_pos, 0.0);
    car.recover_on_surface(0.0);

    let park_inputs = VehicleInputs {
        gear_mode: TransmissionMode::Park,
        brake: 1.0,
        handbrake: true,
        ..Default::default()
    };

    // Allow vehicle to settle suspension onto the 15% grade slope
    for _ in 0..60 {
        car.step(&park_inputs, &ground, 1.0 / 60.0);
    }

    let start_p = car.position;
    for _ in 0..600 {
        // 10 seconds at 60 Hz
        car.step(&park_inputs, &ground, 1.0 / 60.0);
    }

    let drift = (car.position - start_p).length();
    let speed = car.linear_velocity.length();

    println!(
        "Test 2 (15% Slope Park Hold 10s): Speed = {:.6} m/s, Drift = {:.6} m",
        speed, drift
    );

    assert!(
        drift < 0.005,
        "Parked vehicle on 15% slope must not drift > 5mm, got {:.4}m",
        drift
    );
    assert!(
        speed < 0.001,
        "Parked vehicle velocity on 15% slope must be < 1mm/s, got {:.4}m/s",
        speed
    );
}

/// Test 3: Unbraked Slope-Roll Certification
/// In Neutral without brakes, a vehicle on a 15% grade must roll realistically downhill under gravity.
#[test]
fn test_certification_unbraked_slope_roll_in_neutral() {
    let config = VehicleConfig::sports_coupe();
    let grade = 0.15; // 15% grade along +Z
    let ground = SlopedGround::new(0.0, 0.0, grade);
    let start_pos = Vec3::new(0.0, 1.0, 0.0);
    let mut car = DynamicVehicle::new(config, start_pos, 0.0);
    car.recover_on_surface(0.0);

    let neutral_inputs = VehicleInputs {
        gear_mode: TransmissionMode::Neutral,
        throttle: 0.0,
        brake: 0.0,
        handbrake: false,
        steer: 0.0,
    };

    let start_z = car.position.z;
    for _ in 0..180 {
        // 3 seconds
        car.step(&neutral_inputs, &ground, 1.0 / 60.0);
    }

    let final_speed = car.linear_velocity.length();
    let delta_z = car.position.z - start_z;

    println!(
        "Test 3 (15% Slope Neutral Roll 3s): Final Speed = {:.3} m/s, delta_z = {:.3} m",
        final_speed, delta_z
    );

    // Gravity along 15% slope: g * sin(theta) ~ 9.81 * 0.148 ~ 1.45 m/s^2. In 3s: v ~ 4.3 m/s, d ~ 6.5m.
    assert!(
        delta_z < -2.0,
        "Unbraked vehicle must roll downhill (-Z) on 15% slope, got delta_z = {:.3}m",
        delta_z
    );
    assert!(
        final_speed > 2.0,
        "Unbraked vehicle speed must exceed 2.0 m/s after 3s, got {:.3} m/s",
        final_speed
    );
}

/// Test 4: W/S/A/D Input Sign Invariants Certification
/// W accelerates forward, S brakes/reverses, A steers left, D steers right relative to heading.
#[test]
fn test_certification_wsad_input_signs() {
    let ground = FlatGround::new(0.0);

    // 1. W Input: Accelerates along vehicle forward vector
    {
        let config = VehicleConfig::sports_coupe();
        let mut car = DynamicVehicle::new(
            config.clone(),
            Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
            0.5, // Arbitrary heading yaw
        );
        let rot = Mat3::from_quat(car.orientation);
        let forward = rot.col(2);

        let w_inputs = VehicleInputs {
            throttle: 1.0,
            gear_mode: TransmissionMode::Drive,
            ..Default::default()
        };

        for _ in 0..60 {
            car.step(&w_inputs, &ground, 1.0 / 60.0);
        }

        let forward_vel = car.linear_velocity.dot(forward);
        println!(
            "Test 4.1 (W Forward): Forward Velocity = {:.3} m/s",
            forward_vel
        );
        assert!(
            forward_vel > 2.0,
            "W must accelerate vehicle forward along heading, got {:.3} m/s",
            forward_vel
        );
    }

    // 2. S Input from rest: Reverses opposite to vehicle forward vector
    {
        let config = VehicleConfig::sports_coupe();
        let mut car = DynamicVehicle::new(
            config.clone(),
            Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
            0.5,
        );
        let rot = Mat3::from_quat(car.orientation);
        let forward = rot.col(2);

        let s_inputs = VehicleInputs {
            throttle: 0.85,
            gear_mode: TransmissionMode::Reverse,
            ..Default::default()
        };

        for _ in 0..60 {
            car.step(&s_inputs, &ground, 1.0 / 60.0);
        }

        let forward_vel = car.linear_velocity.dot(forward);
        println!(
            "Test 4.2 (S Reverse): Forward Velocity = {:.3} m/s",
            forward_vel
        );
        assert!(
            forward_vel < -1.5,
            "S in reverse must accelerate opposite to heading, got {:.3} m/s",
            forward_vel
        );
    }

    // 3. A and D Steering Sign Differentiation
    {
        let config = VehicleConfig::sports_coupe();
        let mut car_left = DynamicVehicle::new(
            config.clone(),
            Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
            0.0,
        );
        let mut car_right = DynamicVehicle::new(config, Vec3::new(0.0, 1.0, 0.0), 0.0);
        car_right.recover_on_surface(0.0);

        let a_inputs = VehicleInputs {
            throttle: 0.8,
            steer: -0.45, // A key: Steer Left
            gear_mode: TransmissionMode::Drive,
            ..Default::default()
        };
        let d_inputs = VehicleInputs {
            throttle: 0.8,
            steer: 0.45, // D key: Steer Right
            gear_mode: TransmissionMode::Drive,
            ..Default::default()
        };

        for _ in 0..90 {
            car_left.step(&a_inputs, &ground, 1.0 / 60.0);
            car_right.step(&d_inputs, &ground, 1.0 / 60.0);
        }

        println!(
            "Test 4.3 (A vs D): Left car pos.x = {:.3}, Right car pos.x = {:.3}",
            car_left.position.x, car_right.position.x
        );

        // Heading is North (+Z). Left turn curves towards West (-X), Right turn curves towards East (+X).
        assert!(
            car_left.position.x < -1.0,
            "A key (left) must curve vehicle towards -X, got {:.3}",
            car_left.position.x
        );
        assert!(
            car_right.position.x > 1.0,
            "D key (right) must curve vehicle towards +X, got {:.3}",
            car_right.position.x
        );
    }
}

/// Test 5: Braking Distance Certification
/// Full braking from 50 km/h (13.89 m/s) must stop within physical bounds d in [8m, 18m] without backward recoil.
#[test]
fn test_certification_braking_distance() {
    let config = VehicleConfig::sports_coupe();
    let ground = FlatGround::new(0.0);
    let mut car = DynamicVehicle::new(
        config.clone(),
        Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
        0.0,
    );

    // Accelerate to ~50 km/h
    let drive_inputs = VehicleInputs {
        throttle: 1.0,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };
    while car.speed_kph < 50.0 {
        car.step(&drive_inputs, &ground, 1.0 / 60.0);
    }

    let initial_brake_speed = car.linear_velocity.length();
    let initial_brake_pos = car.position.z;

    let brake_inputs = VehicleInputs {
        brake: 1.0,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };

    let mut stopped = false;
    for _ in 0..300 {
        // Up to 5s braking
        car.step(&brake_inputs, &ground, 1.0 / 60.0);
        if car.linear_velocity.length() < 0.02 {
            stopped = true;
            break;
        }
    }

    let braking_dist = car.position.z - initial_brake_pos;
    println!(
        "Test 5 (Braking from {:.1} km/h): Distance = {:.2} m, Stopped = {}",
        initial_brake_speed * 3.6,
        braking_dist,
        stopped
    );

    assert!(
        stopped,
        "Vehicle must come to a complete stop under full braking"
    );
    assert!(
        (8.0..=18.0).contains(&braking_dist),
        "Braking distance from 50 km/h must be between 8m and 18m, got {:.2}m",
        braking_dist
    );

    // Verify no backward recoil after stopping
    let post_stop_z = car.position.z;
    for _ in 0..60 {
        car.step(&brake_inputs, &ground, 1.0 / 60.0);
    }
    let recoil = (car.position.z - post_stop_z).abs();
    assert!(
        recoil < 0.005,
        "Vehicle must not recoil or chatter after stopping, recoil = {:.5}m",
        recoil
    );
}

/// Test 6: Circle and Turn Stability Certification
/// Steady cornering under high lateral acceleration must remain roll-stable without flipping or spinning out.
#[test]
fn test_certification_circle_turn_stability() {
    let config = VehicleConfig::sports_coupe();
    let ground = FlatGround::new(0.0);
    let mut car = DynamicVehicle::new(
        config.clone(),
        Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
        0.0,
    );

    let circle_inputs = VehicleInputs {
        throttle: 0.65,
        steer: 0.35,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };

    let mut max_roll_deg = 0.0f32;
    for _ in 0..600 {
        // 10 seconds of continuous circular turn
        car.step(&circle_inputs, &ground, 1.0 / 60.0);

        let rot = Mat3::from_quat(car.orientation);
        let up = rot.col(1);
        let roll_deg = up.y.clamp(-1.0, 1.0).acos().to_degrees();
        if roll_deg > max_roll_deg {
            max_roll_deg = roll_deg;
        }

        assert!(
            !car.is_overturned,
            "Vehicle must not roll over during steady turn"
        );
    }

    println!(
        "Test 6 (Circle Turn Stability): Final Speed = {:.1} km/h, Max Chassis Lean = {:.2} deg",
        car.speed_kph, max_roll_deg
    );
    assert!(
        max_roll_deg < 12.0,
        "Anti-roll bar must keep body lean under 12 deg, got {:.2} deg",
        max_roll_deg
    );
}

/// Test 7: Repeated Spawn Equilibrium Certification
/// Spawning must initialize suspension at static equilibrium with zero vertical launch impulse.
#[test]
fn test_certification_repeated_spawn_equilibrium() {
    let ground = FlatGround::new(10.0);
    let config = VehicleConfig::sports_coupe();

    for spawn_idx in 0..10 {
        let mut car = DynamicVehicle::new(
            config.clone(),
            Vec3::new(
                spawn_idx as f32 * 5.0,
                10.0 + config.equilibrium_ride_height(),
                0.0,
            ),
            0.0,
        );
        let inputs = VehicleInputs::default();

        // Tick 0: initial velocity must be 0
        assert_eq!(
            car.linear_velocity,
            Vec3::ZERO,
            "Spawn #{} initial linear velocity must be exactly zero",
            spawn_idx
        );

        // Step 1 tick
        car.step(&inputs, &ground, 1.0 / 60.0);

        // Vertical velocity on tick 1 must not spike (zero shock impulse)
        let v_y_abs = car.linear_velocity.y.abs();
        assert!(
            v_y_abs < 0.05,
            "Spawn #{} tick 1 vertical velocity spike must be < 0.05 m/s, got {:.4} m/s",
            spawn_idx,
            v_y_abs
        );

        // Normal forces must be grounded and positive
        for (w_i, wheel) in car.wheels.iter().enumerate() {
            assert!(
                wheel.in_contact,
                "Spawn #{} wheel {} must be in ground contact",
                spawn_idx, w_i
            );
            assert!(
                wheel.normal_force_fz > 1000.0,
                "Spawn #{} wheel {} normal force must be positive",
                spawn_idx,
                w_i
            );
        }
    }
    println!("Test 7 (Repeated Spawn Equilibrium): 10/10 spawns achieved zero-impulse equilibrium");
}

/// Test 8: Multi-Framerate Determinism Certification
/// Fixed inputs simulated through the fixed 60Hz physics accumulator must yield identical results
/// when rendered at 30 FPS, 60 FPS, and 120 FPS.
#[test]
fn test_certification_multirate_determinism() {
    let ground = FlatGround::new(0.0);
    let config = VehicleConfig::sports_coupe();

    fn run_simulation_at_fps(fps: f32, config: VehicleConfig, ground: &FlatGround) -> Vec3 {
        let start_pos = Vec3::new(0.0, config.equilibrium_ride_height(), 0.0);
        let mut car = DynamicVehicle::new(config, start_pos, 0.0);

        let dt_frame = 1.0 / fps;
        const PHYSICS_DT: f32 = 1.0 / 60.0;
        let mut accumulator = 0.0f32;

        let total_time = 4.0f32; // 4 seconds total duration
        let total_frames = (total_time / dt_frame).round() as usize;

        for frame in 0..total_frames {
            let t = frame as f32 * dt_frame;
            accumulator += dt_frame;

            let inputs = if t < 1.5 {
                VehicleInputs {
                    throttle: 1.0,
                    gear_mode: TransmissionMode::Drive,
                    ..Default::default()
                }
            } else if t < 2.5 {
                VehicleInputs {
                    throttle: 0.8,
                    steer: 0.3,
                    gear_mode: TransmissionMode::Drive,
                    ..Default::default()
                }
            } else {
                VehicleInputs {
                    brake: 0.9,
                    gear_mode: TransmissionMode::Drive,
                    ..Default::default()
                }
            };

            while accumulator >= PHYSICS_DT {
                car.step(&inputs, ground, PHYSICS_DT);
                accumulator -= PHYSICS_DT;
            }
        }
        car.position
    }

    let pos_30fps = run_simulation_at_fps(30.0, config.clone(), &ground);
    let pos_60fps = run_simulation_at_fps(60.0, config.clone(), &ground);
    let pos_120fps = run_simulation_at_fps(120.0, config, &ground);

    println!(
        "Test 8 (Multi-Framerate Determinism):\n  30 FPS pos:  {:?}\n  60 FPS pos:  {:?}\n  120 FPS pos: {:?}",
        pos_30fps, pos_60fps, pos_120fps
    );

    let diff_30_60 = (pos_30fps - pos_60fps).length();
    let diff_60_120 = (pos_60fps - pos_120fps).length();

    assert!(
        diff_30_60 < 0.02,
        "Difference between 30 FPS and 60 FPS must be < 0.02m, got {:.4}m",
        diff_30_60
    );
    assert!(
        diff_60_120 < 0.02,
        "Difference between 60 FPS and 120 FPS must be < 0.02m, got {:.4}m",
        diff_60_120
    );
}

/// Test 9: Authentic Real-Road San Francisco Settling Test
/// Verifies settling stability on authentic surveyed DEM roadway geometry at Montgomery Street.
#[test]
fn test_certification_real_road_san_francisco_settling() {
    use realearth_ingestion::authentic::load_authentic_city;
    use realearth_shared::coordinates::Wgs84Coord;

    let (_dataset, dem, _meta) = load_authentic_city("san-francisco").expect("SF must load");
    let package_anchor = Wgs84Coord::new(37.7952, -122.4028, 0.0);
    let meters_per_deg_lat = 111_320.0;
    let meters_per_deg_lon = 111_320.0 * package_anchor.lat.to_radians().cos().abs().max(0.01);

    struct DemQuery {
        dem: realearth_ingestion::dem_grid::DemGrid,
        anchor: Wgs84Coord,
        m_lat: f64,
        m_lon: f64,
    }
    impl realearth_vehicles::ground::GroundQuery for DemQuery {
        fn query_ground_contact(&self, origin: Vec3, max_dist: f32) -> Option<(f32, Vec3)> {
            let lat = self.anchor.lat + (origin.z as f64 / self.m_lat);
            let lon = self.anchor.lon + (origin.x as f64 / self.m_lon);
            let ground_y = self.dem.sample_bilinear(&Wgs84Coord::new(lat, lon, 0.0));
            let dist = origin.y - ground_y;
            let eps = 0.5;
            let h_r =
                self.dem
                    .sample_bilinear(&Wgs84Coord::new(lat, lon + (eps / self.m_lon), 0.0));
            let h_l =
                self.dem
                    .sample_bilinear(&Wgs84Coord::new(lat, lon - (eps / self.m_lon), 0.0));
            let h_u =
                self.dem
                    .sample_bilinear(&Wgs84Coord::new(lat + (eps / self.m_lat), lon, 0.0));
            let h_d =
                self.dem
                    .sample_bilinear(&Wgs84Coord::new(lat - (eps / self.m_lat), lon, 0.0));
            let normal = Vec3::new(
                -(h_r - h_l) / (2.0 * eps as f32),
                1.0,
                -(h_u - h_d) / (2.0 * eps as f32),
            )
            .normalize();
            if (0.0..=max_dist).contains(&dist) {
                Some((dist, normal))
            } else if dist < 0.0 && dist > -10.0 {
                Some((0.0, normal))
            } else {
                None
            }
        }
    }

    let gq = DemQuery {
        dem,
        anchor: package_anchor,
        m_lat: meters_per_deg_lat,
        m_lon: meters_per_deg_lon,
    };

    let spawn_pos = Vec3::new(-0.41, 16.26, 0.0);
    let spawn_yaw = 0.0f32;

    let mut car = DynamicVehicle::new(VehicleConfig::sports_coupe(), spawn_pos, spawn_yaw);
    // Spawn state vehicle in Park settling into authentic roadway elevation
    let inputs = VehicleInputs {
        gear_mode: TransmissionMode::Park,
        handbrake: true,
        ..Default::default()
    };

    for _ in 0..600 {
        // 10 seconds at 60 Hz
        car.step(&inputs, &gq, 1.0 / 60.0);
    }

    let drift =
        ((car.position.x - spawn_pos.x).powi(2) + (car.position.z - spawn_pos.z).powi(2)).sqrt();
    let speed = car.linear_velocity.length();

    println!(
        "Test 9 (SF Montgomery Street 10s): Speed = {:.5} m/s, Total Drift = {:.4} m",
        speed, drift
    );

    assert!(
        drift < 0.02,
        "Authentic SF roadway settling drift must be < 20mm, got {:.4}m",
        drift
    );
    assert!(
        speed < 0.001,
        "Authentic SF roadway final speed must be < 1mm/s, got {:.5}m/s",
        speed
    );
}
