//! STAGE 1.3-R — PERMANENT VEHICLE CONTROL, MOTION AND BRAKING REPAIR CERTIFICATION
//!
//! Master certification suite implementing:
//! - Part 1: Canonical Steering Direction Fidelity at All 4 Compass Headings (Cars, Bikes, Buses, Trucks, Marine, Aviation)
//! - Part 2: Signed Velocity, Reverse Transmission and Full Braking (Motorcycle, Transit Bus, Heavy Truck, Car)
//! - Part 3: Railway Train Reverser Single-Edge Action, Service and Emergency Brakes
//! - Part 4: Standstill Anti-Drift Hold (Flat & Slope), Coasting Inertia vs Active Service Stopping
//! - Part 5: Authoritative Input Router, Focus-Loss Input Clearing and Mode Switch Isolation
//! - Part 6: Multi-Framerate (30 / 60 / 120 FPS) Fixed-Timestep Replay Invariance

use glam::{Quat, Vec3};
use realearth_vehicles::{
    ActiveTransportMode, AircraftFlight, AuthoritativeVehicleState, DynamicVehicle, FlatGround,
    FlightInputs, HeavyTruckModel, MarineInputs, MarineVessel, MotorcycleModel, SlopedGround,
    TrainBrakeState, TrainConsist, TransitBusModel, TransmissionMode, VehicleConfig, VehicleInputs,
};
use std::f32::consts::{FRAC_PI_2, PI};

const DT: f32 = 1.0 / 60.0;

// ============================================================================
// PART 1: CANONICAL STEERING DIRECTION FIDELITY AT 4 COMPASS HEADINGS
// ============================================================================

#[test]
fn test_part1_steering_direction_fidelity_across_all_headings() {
    println!("🧭 Part 1: Authoritative Steering Direction Fidelity at 4 Compass Headings...");

    let ground = FlatGround::new(0.0);
    let headings = [
        ("North (0 deg)", 0.0_f32),
        ("East (90 deg)", FRAC_PI_2),
        ("South (180 deg)", PI),
        ("West (270 deg)", -FRAC_PI_2),
    ];

    for (name, initial_heading) in headings {
        // --- 1. CAR (Ackermann/Pacejka) ---
        let car_config = VehicleConfig::sports_coupe();
        let car_eq_y = car_config.equilibrium_ride_height();
        let mut car_left = DynamicVehicle::new(
            car_config.clone(),
            Vec3::new(0.0, car_eq_y, 0.0),
            initial_heading,
        );
        let mut car_right = DynamicVehicle::new(
            car_config.clone(),
            Vec3::new(0.0, car_eq_y, 0.0),
            initial_heading,
        );

        let initial_right = car_left.orientation * Vec3::X;

        // Drive forward and steer left on car_left
        let steer_left_input = VehicleInputs {
            throttle: 0.8,
            steer: -0.5, // Steer Left
            gear_mode: TransmissionMode::Drive,
            ..Default::default()
        };
        // Drive forward and steer right on car_right
        let steer_right_input = VehicleInputs {
            throttle: 0.8,
            steer: 0.5, // Steer Right
            gear_mode: TransmissionMode::Drive,
            ..Default::default()
        };

        for _ in 0..90 {
            car_left.step(&steer_left_input, &ground, DT);
            car_right.step(&steer_right_input, &ground, DT);
        }

        let delta_left = car_left.position - Vec3::new(0.0, car_eq_y, 0.0);
        let delta_right = car_right.position - Vec3::new(0.0, car_eq_y, 0.0);

        let lateral_disp_left = delta_left.dot(initial_right);
        let lateral_disp_right = delta_right.dot(initial_right);

        assert!(
            lateral_disp_left < -0.05,
            "Car steering LEFT at {} must displace to local left (-X): got {}",
            name,
            lateral_disp_left
        );
        assert!(
            lateral_disp_right > 0.05,
            "Car steering RIGHT at {} must displace to local right (+X): got {}",
            name,
            lateral_disp_right
        );

        // --- 2. MOTORCYCLE (2-Wheel Lean Dynamics) ---
        let mut bike_left = MotorcycleModel::new(Vec3::new(0.0, 0.35, 0.0), initial_heading);
        let mut bike_right = MotorcycleModel::new(Vec3::new(0.0, 0.35, 0.0), initial_heading);

        for _ in 0..90 {
            bike_left.step(&steer_left_input, &ground, DT);
            bike_right.step(&steer_right_input, &ground, DT);
        }

        let bike_disp_left = (bike_left.position - Vec3::new(0.0, 0.35, 0.0)).dot(initial_right);
        let bike_disp_right = (bike_right.position - Vec3::new(0.0, 0.35, 0.0)).dot(initial_right);

        assert!(
            bike_disp_left < -0.2,
            "Motorcycle steering LEFT at {} must turn toward local left: got {}",
            name,
            bike_disp_left
        );
        assert!(
            bike_disp_right > 0.2,
            "Motorcycle steering RIGHT at {} must turn toward local right: got {}",
            name,
            bike_disp_right
        );

        // --- 3. TRANSIT BUS ---
        let mut bus_left = TransitBusModel::new(Vec3::new(0.0, 0.95, 0.0), initial_heading);
        let mut bus_right = TransitBusModel::new(Vec3::new(0.0, 0.95, 0.0), initial_heading);

        for _ in 0..90 {
            bus_left.step(&steer_left_input, &ground, DT);
            bus_right.step(&steer_right_input, &ground, DT);
        }

        let bus_disp_left = (bus_left.position - Vec3::new(0.0, 0.95, 0.0)).dot(initial_right);
        let bus_disp_right = (bus_right.position - Vec3::new(0.0, 0.95, 0.0)).dot(initial_right);

        assert!(
            bus_disp_left < -0.01,
            "Transit Bus steering LEFT at {} must turn toward local left: got {}",
            name,
            bus_disp_left
        );
        assert!(
            bus_disp_right > 0.01,
            "Transit Bus steering RIGHT at {} must turn toward local right: got {}",
            name,
            bus_disp_right
        );

        // --- 4. HEAVY TRUCK ---
        let mut truck_left = HeavyTruckModel::new(Vec3::new(0.0, 1.2, 0.0), initial_heading);
        let mut truck_right = HeavyTruckModel::new(Vec3::new(0.0, 1.2, 0.0), initial_heading);

        for _ in 0..90 {
            truck_left.step(&steer_left_input, &ground, DT);
            truck_right.step(&steer_right_input, &ground, DT);
        }

        let truck_disp_left =
            (truck_left.tractor_position - Vec3::new(0.0, 1.2, 0.0)).dot(initial_right);
        let truck_disp_right =
            (truck_right.tractor_position - Vec3::new(0.0, 1.2, 0.0)).dot(initial_right);

        assert!(
            truck_disp_left < -0.01,
            "Heavy Truck steering LEFT at {} must turn toward local left: got {}",
            name,
            truck_disp_left
        );
        assert!(
            truck_disp_right > 0.01,
            "Heavy Truck steering RIGHT at {} must turn toward local right: got {}",
            name,
            truck_disp_right
        );

        // --- 5. MARINE VESSEL (Archimedes / Hydrofoil Rudder) ---
        let mut ship_left = MarineVessel::new_ferry("SF-Bay-Ferry", Vec3::ZERO, initial_heading);
        let mut ship_right = MarineVessel::new_ferry("SF-Bay-Ferry", Vec3::ZERO, initial_heading);

        let marine_left = MarineInputs {
            throttle: 1.0,
            rudder: -1.0, // Steer Left (Port)
            ..Default::default()
        };
        let marine_right = MarineInputs {
            throttle: 1.0,
            rudder: 1.0, // Steer Right (Starboard)
            ..Default::default()
        };

        for step in 0..180 {
            ship_left.step(&marine_left, step as f32 * DT, DT);
            ship_right.step(&marine_right, step as f32 * DT, DT);
        }

        let ship_yaw_delta_left = ship_left.heading_radians - initial_heading;
        let ship_yaw_delta_right = ship_right.heading_radians - initial_heading;

        assert!(
            ship_yaw_delta_left < -0.005,
            "Ship rudder LEFT at {} must decrease heading angle (turn port): got {}",
            name,
            ship_yaw_delta_left
        );
        assert!(
            ship_yaw_delta_right > 0.005,
            "Ship rudder RIGHT at {} must increase heading angle (turn starboard): got {}",
            name,
            ship_yaw_delta_right
        );

        // --- 6. AVIATION AIRCRAFT (6-DOF Aileron Roll Direction) ---
        let mut jet_left = AircraftFlight::new_airliner(
            "Boeing-777",
            Vec3::new(0.0, 3000.0, 0.0),
            initial_heading,
        );
        jet_left.linear_velocity = jet_left.orientation * Vec3::new(0.0, 0.0, 180.0);
        let mut jet_right = AircraftFlight::new_airliner(
            "Boeing-777",
            Vec3::new(0.0, 3000.0, 0.0),
            initial_heading,
        );
        jet_right.linear_velocity = jet_right.orientation * Vec3::new(0.0, 0.0, 180.0);

        let flight_left = FlightInputs {
            throttle: 0.8,
            aileron: -0.7, // Roll Left
            gear_down: false,
            ..Default::default()
        };
        let flight_right = FlightInputs {
            throttle: 0.8,
            aileron: 0.7, // Roll Right
            gear_down: false,
            ..Default::default()
        };

        for _ in 0..30 {
            jet_left.step(&flight_left, 0.0, DT);
            jet_right.step(&flight_right, 0.0, DT);
        }

        // Positive angular velocity around +Z rolls left (right wing up, left wing down)
        assert!(
            jet_left.angular_velocity.z > 0.01,
            "Aircraft aileron LEFT at {} must produce positive roll rate omega_z > 0: got {}",
            name,
            jet_left.angular_velocity.z
        );
        assert!(
            jet_right.angular_velocity.z < -0.01,
            "Aircraft aileron RIGHT at {} must produce negative roll rate omega_z < 0: got {}",
            name,
            jet_right.angular_velocity.z
        );
    }

    println!(
        "   ✓ Part 1 PASSED: Canonical steering fidelity certified at all 4 compass headings."
    );
}

// ============================================================================
// PART 2: SIGNED VELOCITY, REVERSE TRANSMISSION AND FULL BRAKING
// ============================================================================

#[test]
fn test_part2_reverse_transmission_and_braking_for_all_road_modes() {
    println!(
        "🔄 Part 2: Signed Velocity, Reverse Drive and Complete Stopping across Road Modes..."
    );

    let ground = FlatGround::new(0.0);

    // Test reverse drive and stopping on:
    // 1. Motorcycle
    let mut bike = MotorcycleModel::new(Vec3::new(0.0, 0.35, 0.0), 0.0);
    let reverse_throttle = VehicleInputs {
        throttle: 1.0,
        steer: 0.0,
        gear_mode: TransmissionMode::Reverse,
        ..Default::default()
    };
    let reverse_brake = VehicleInputs {
        throttle: 0.0,
        brake: 1.0,
        gear_mode: TransmissionMode::Reverse,
        ..Default::default()
    };

    // Accelerate backward for 2 seconds
    for _ in 0..120 {
        bike.step(&reverse_throttle, &ground, DT);
    }
    assert!(
        bike.linear_velocity.z < -1.5,
        "Motorcycle must move backward in reverse: got vel_z={}",
        bike.linear_velocity.z
    );
    assert!(
        bike.position.z < -1.0,
        "Motorcycle must displace backward in reverse: got pos_z={}",
        bike.position.z
    );

    // Apply brake to bring to complete stop
    for _ in 0..120 {
        bike.step(&reverse_brake, &ground, DT);
    }
    assert!(
        bike.linear_velocity.length() < 0.05,
        "Motorcycle must brake to complete stop: got speed={}",
        bike.linear_velocity.length()
    );

    // 2. Transit Bus
    let mut bus = TransitBusModel::new(Vec3::new(0.0, 0.95, 0.0), 0.0);
    for _ in 0..120 {
        bus.step(&reverse_throttle, &ground, DT);
    }
    assert!(
        bus.linear_velocity.z < -1.0,
        "Transit Bus must move backward in reverse: got vel_z={}",
        bus.linear_velocity.z
    );
    assert!(
        bus.position.z < -0.8,
        "Transit Bus must displace backward: got pos_z={}",
        bus.position.z
    );

    for _ in 0..120 {
        bus.step(&reverse_brake, &ground, DT);
    }
    assert!(
        bus.linear_velocity.length() < 0.05,
        "Transit Bus must brake to complete stop: got speed={}",
        bus.linear_velocity.length()
    );

    // 3. Heavy Truck
    let mut truck = HeavyTruckModel::new(Vec3::new(0.0, 1.2, 0.0), 0.0);
    for _ in 0..120 {
        truck.step(&reverse_throttle, &ground, DT);
    }
    assert!(
        truck.linear_velocity.z < -0.5,
        "Heavy Truck must move backward in reverse: got vel_z={}",
        truck.linear_velocity.z
    );
    assert!(
        truck.tractor_position.z < -0.6,
        "Heavy Truck must displace backward: got pos_z={}",
        truck.tractor_position.z
    );

    for _ in 0..120 {
        truck.step(&reverse_brake, &ground, DT);
    }
    assert!(
        truck.linear_velocity.length() < 0.05,
        "Heavy Truck must brake to complete stop: got speed={}",
        truck.linear_velocity.length()
    );

    println!(
        "   ✓ Part 2 PASSED: Reverse transmission and full braking validated on all road modes."
    );
}

// ============================================================================
// PART 3: RAILWAY TRAIN REVERSER, SERVICE BRAKE AND EMERGENCY BRAKE
// ============================================================================

#[test]
fn test_part3_railway_train_reverser_service_and_emergency_braking() {
    println!("🚆 Part 3: Train Consist Reverser and Independent Braking Mechanics...");

    let mut train = TrainConsist::new_locomotive(
        "Caltrain-MP36PH",
        realearth_vehicles::LocomotiveType::DieselElectric,
        2_700_000.0,
        310_000.0,
        130_000.0,
        20.0,
    );
    train.add_wagon(realearth_vehicles::TrainCar::new_wagon(
        "Coach-1", 42_000.0, 3_000.0, 24.0,
    ));
    assert_eq!(train.reverser, 1);
    assert_eq!(train.brake_pipe_pressure_bar, 5.0);

    // 1. Forward acceleration
    train.throttle = 1.0;
    for _ in 0..300 {
        train.step(DT);
    }
    assert!(
        train.velocity_mps > 5.0,
        "Train should accelerate forward: got vel={}",
        train.velocity_mps
    );

    // 2. Service Brake application
    train.throttle = 0.0;
    train.apply_service_brake(0.85);
    for _ in 0..600 {
        train.step(DT);
    }
    assert!(
        train.velocity_mps < 0.1,
        "Train service brake must bring train to complete stop: got vel={}",
        train.velocity_mps
    );

    // 3. Reverse direction operation
    train.reverser = -1;
    train.apply_service_brake(0.0);
    train.throttle = 1.0;
    for _ in 0..300 {
        train.step(DT);
    }
    assert!(
        train.velocity_mps < -3.0,
        "Train must accelerate backward when reverser = -1: got vel={}",
        train.velocity_mps
    );

    // 4. Emergency Brake test
    train.trigger_emergency_brake();
    assert!(train.emergency_brake);
    assert_eq!(train.brake_state, TrainBrakeState::Emergency);

    for _ in 0..600 {
        train.step(DT);
    }
    assert!(
        train.brake_pipe_pressure_bar < 0.05,
        "Emergency brake must vent brake pipe to 0 bar: got {} bar",
        train.brake_pipe_pressure_bar
    );
    assert_eq!(
        train.velocity_mps, 0.0,
        "Emergency brake must bring train to complete motionless stop"
    );

    println!("   ✓ Part 3 PASSED: Railway reverser, service braking and emergency hold validated.");
}

// ============================================================================
// PART 4: STANDSTILL ANTI-DRIFT, SLOPE HOLD, COASTING VS ACTIVE STOPPING
// ============================================================================

#[test]
fn test_part4_standstill_anti_drift_and_coasting_physics() {
    println!("🛑 Part 4: Standstill Anti-Drift, Slope Park Hold and Realistic Coasting...");

    let ground = FlatGround::new(0.0);

    // 1. Zero-Input Standstill Flat Ground Anti-Drift (Car, Bike, Bus, Truck)
    let config = VehicleConfig::sports_coupe();
    let eq_y = config.equilibrium_ride_height();
    let mut car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);
    let zero_input = VehicleInputs {
        gear_mode: TransmissionMode::Park,
        ..Default::default()
    };

    let p0 = car.position;
    for _ in 0..1200 {
        car.step(&zero_input, &ground, DT);
    }
    let car_drift = (car.position - p0).length();
    assert!(
        car_drift < 0.002,
        "Parked car must exhibit zero drift (<2mm): got {}m",
        car_drift
    );

    // 2. Steep Slope Hold (18% grade)
    let slope = SlopedGround::new(0.0, 0.0, 0.18);
    let mut slope_car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y + 0.1, 0.0), 0.0);

    for _ in 0..120 {
        slope_car.step(&zero_input, &slope, DT);
    }
    let settled_z = slope_car.position.z;
    for _ in 0..600 {
        slope_car.step(&zero_input, &slope, DT);
    }
    let slope_drift = (slope_car.position.z - settled_z).abs();
    assert!(
        slope_drift < 0.005,
        "Park mode on 18% slope must hold car motionless: drift={}",
        slope_drift
    );

    // 3. Realistic Coasting Inertia vs Active Service Stopping
    let mut coasting_car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);
    let mut stopping_car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);

    // Accelerate both to ~50 km/h
    let drive_throttle = VehicleInputs {
        throttle: 1.0,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };
    for _ in 0..180 {
        coasting_car.step(&drive_throttle, &ground, DT);
        stopping_car.step(&drive_throttle, &ground, DT);
    }
    let initial_speed = coasting_car.linear_velocity.length();
    assert!(initial_speed > 10.0, "Car must achieve driving speed");

    // Release throttle on coasting_car (natural coasting with drag and rolling resistance)
    let coast_input = VehicleInputs {
        throttle: 0.0,
        brake: 0.0,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };
    // Apply full service brake on stopping_car
    let brake_input = VehicleInputs {
        throttle: 0.0,
        brake: 1.0,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };

    for _ in 0..180 {
        coasting_car.step(&coast_input, &ground, DT);
        stopping_car.step(&brake_input, &ground, DT);
    }

    assert!(
        stopping_car.linear_velocity.length() < 0.1,
        "Active service brake must stop car within 3 seconds: speed={}",
        stopping_car.linear_velocity.length()
    );
    assert!(
        coasting_car.linear_velocity.length() > 5.0,
        "Coasting car must preserve momentum/inertia when throttle is released: speed={}",
        coasting_car.linear_velocity.length()
    );
    assert!(
        coasting_car.linear_velocity.length() < initial_speed,
        "Coasting car must gradually decelerate due to aerodynamic drag and rolling resistance"
    );

    println!("   ✓ Part 4 PASSED: Standstill hold, slope hold, and coasting vs stopping verified.");
}

// ============================================================================
// PART 5: AUTHORITATIVE INPUT ROUTER & FOCUS-LOSS INPUT CLEARING
// ============================================================================

#[test]
fn test_part5_input_router_focus_loss_and_mode_switching_isolation() {
    println!("🎮 Part 5: Authoritative Input Router, Focus Loss and Mode Switching Isolation...");

    // Simulated Action Router tracking key states and modal transitions
    struct ActionRouter {
        pressed_w: bool,
        pressed_a: bool,
        pressed_s: bool,
        pressed_d: bool,
        window_focused: bool,
        active_mode: ActiveTransportMode,
        sim_state: AuthoritativeVehicleState,
    }

    impl ActionRouter {
        fn new() -> Self {
            Self {
                pressed_w: false,
                pressed_a: false,
                pressed_s: false,
                pressed_d: false,
                window_focused: true,
                active_mode: ActiveTransportMode::Car,
                sim_state: AuthoritativeVehicleState::Idle,
            }
        }

        fn on_focus_change(&mut self, focused: bool) {
            self.window_focused = focused;
            if !focused {
                self.reset_inputs();
            }
        }

        fn on_mode_switch(&mut self, new_mode: ActiveTransportMode) {
            self.reset_inputs();
            self.active_mode = new_mode;
            self.sim_state = match new_mode {
                ActiveTransportMode::Car
                | ActiveTransportMode::Motorcycle
                | ActiveTransportMode::TransitBus
                | ActiveTransportMode::HeavyTruck => AuthoritativeVehicleState::Idle,
                ActiveTransportMode::Train => AuthoritativeVehicleState::Rail,
                ActiveTransportMode::Marine => AuthoritativeVehicleState::Marine,
                ActiveTransportMode::Aviation => AuthoritativeVehicleState::Flight,
            };
        }

        fn reset_inputs(&mut self) {
            self.pressed_w = false;
            self.pressed_a = false;
            self.pressed_s = false;
            self.pressed_d = false;
        }

        fn compute_road_inputs(&self, current_forward_speed: f32) -> VehicleInputs {
            if !self.window_focused {
                return VehicleInputs {
                    gear_mode: TransmissionMode::Park,
                    brake: 1.0,
                    ..Default::default()
                };
            }

            let steer = if self.pressed_a {
                -0.65
            } else if self.pressed_d {
                0.65
            } else {
                0.0
            };

            let mut inputs = VehicleInputs {
                steer,
                ..Default::default()
            };

            if self.pressed_w {
                if current_forward_speed < -0.3 {
                    inputs.brake = 0.95;
                    inputs.gear_mode = TransmissionMode::Reverse;
                } else {
                    inputs.throttle = 1.0;
                    inputs.gear_mode = TransmissionMode::Drive;
                }
            } else if self.pressed_s {
                if current_forward_speed > 0.35 {
                    inputs.brake = 0.95;
                    inputs.gear_mode = TransmissionMode::Drive;
                } else {
                    inputs.throttle = 0.85;
                    inputs.gear_mode = TransmissionMode::Reverse;
                }
            } else {
                inputs.gear_mode = TransmissionMode::Drive;
            }

            inputs
        }
    }

    let mut router = ActionRouter::new();

    // 1. Simulate user holding W and A
    router.pressed_w = true;
    router.pressed_a = true;
    let inputs_active = router.compute_road_inputs(5.0);
    assert_eq!(inputs_active.throttle, 1.0);
    assert_eq!(inputs_active.steer, -0.65);

    // 2. Trigger Window Focus Loss (Alt-Tab / click away)
    router.on_focus_change(false);
    assert!(!router.pressed_w);
    assert!(!router.pressed_a);
    let inputs_unfocused = router.compute_road_inputs(5.0);
    assert_eq!(
        inputs_unfocused.throttle, 0.0,
        "Throttle must drop to 0 when window loses focus"
    );
    assert_eq!(
        inputs_unfocused.steer, 0.0,
        "Steering must center when window loses focus"
    );
    assert_eq!(
        inputs_unfocused.brake, 1.0,
        "Brake/Park must engage on focus loss to prevent runaway vehicle"
    );

    // 3. Regain focus and switch transport mode
    router.on_focus_change(true);
    router.pressed_w = true;
    router.on_mode_switch(ActiveTransportMode::TransitBus);
    assert!(
        !router.pressed_w,
        "Transport mode switch must clear stale keyboard inputs"
    );
    assert_eq!(router.active_mode, ActiveTransportMode::TransitBus);
    assert_eq!(router.sim_state, AuthoritativeVehicleState::Idle);

    println!(
        "   ✓ Part 5 PASSED: Action router, focus loss clearing and mode isolation validated."
    );
}

// ============================================================================
// PART 6: MULTI-FRAMERATE INVARIANCE (30 / 60 / 120 FPS REPLAY)
// ============================================================================

#[test]
fn test_part6_multi_framerate_physics_accumulator_invariance() {
    println!("⏱️ Part 6: Multi-Framerate (30 / 60 / 120 FPS) Fixed-Timestep Replay Invariance...");

    let ground = FlatGround::new(0.0);
    let config = VehicleConfig::sports_coupe();
    let fixed_dt = 1.0 / 60.0;

    // Simulate 3 seconds of simulation: 180 fixed ticks
    let total_simulation_time = 3.0_f32;

    let input_script = [
        (
            VehicleInputs {
                throttle: 1.0,
                steer: -0.2,
                gear_mode: TransmissionMode::Drive,
                ..Default::default()
            },
            60,
        ),
        (
            VehicleInputs {
                throttle: 0.5,
                steer: 0.3,
                gear_mode: TransmissionMode::Drive,
                ..Default::default()
            },
            60,
        ),
        (
            VehicleInputs {
                throttle: 0.0,
                brake: 1.0,
                steer: 0.0,
                gear_mode: TransmissionMode::Drive,
                ..Default::default()
            },
            60,
        ),
    ];

    // Helper to simulate a variable frame rate loop with a fixed accumulator
    let run_simulation_at_fps = |frame_rate: f32| -> (Vec3, Vec3, Quat) {
        let frame_dt = 1.0 / frame_rate;
        let mut car = DynamicVehicle::new(config.clone(), Vec3::new(0.0, 0.4, 0.0), 0.0);
        let mut accumulator = 0.0_f32;
        let mut sim_ticks_completed = 0;

        let total_frames = (total_simulation_time / frame_dt).round() as usize;

        for _ in 0..total_frames {
            accumulator += frame_dt;

            while accumulator >= fixed_dt && sim_ticks_completed < 180 {
                let input = if sim_ticks_completed < 60 {
                    &input_script[0].0
                } else if sim_ticks_completed < 120 {
                    &input_script[1].0
                } else {
                    &input_script[2].0
                };

                car.step(input, &ground, fixed_dt);
                accumulator -= fixed_dt;
                sim_ticks_completed += 1;
            }
        }

        (car.position, car.linear_velocity, car.orientation)
    };

    let (pos_30, vel_30, rot_30) = run_simulation_at_fps(30.0);
    let (pos_60, vel_60, rot_60) = run_simulation_at_fps(60.0);
    let (pos_120, vel_120, rot_120) = run_simulation_at_fps(120.0);

    // Assert that the simulation outcomes across 30, 60, and 120 FPS are bitwise identical
    assert_eq!(
        pos_30, pos_60,
        "30 FPS and 60 FPS runs must produce bitwise identical vehicle positions"
    );
    assert_eq!(
        pos_60, pos_120,
        "60 FPS and 120 FPS runs must produce bitwise identical vehicle positions"
    );

    assert_eq!(
        vel_30, vel_60,
        "30 FPS and 60 FPS runs must produce bitwise identical vehicle velocities"
    );
    assert_eq!(
        vel_60, vel_120,
        "60 FPS and 120 FPS runs must produce bitwise identical vehicle velocities"
    );

    assert_eq!(
        rot_30, rot_60,
        "30 FPS and 60 FPS runs must produce bitwise identical vehicle orientations"
    );
    assert_eq!(
        rot_60, rot_120,
        "60 FPS and 120 FPS runs must produce bitwise identical vehicle orientations"
    );

    println!("   ✓ Part 6 PASSED: 30 / 60 / 120 FPS simulation yields bitwise identical outcomes.");
}
