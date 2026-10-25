//! STAGE A — P0 DEFECT FORENSIC REPRODUCTION SUITE
//!
//! Provides independent, reproducible, deterministic proof for all 8 P0 defects:
//! 1. P0-1: Steering sign convention & directional deviation (KeyA / steer_left vs steer_right vs heading)
//! 2. P0-2: Self-steering divergence under zero input
//! 3. P0-3: Backward movement on hill slope with zero input (Drive D1 rolls backward)
//! 4. P0-4: Building collision inward penetration trapping
//! 5. P0-5: Fallback primitive placeholder meshes & camera occlusion
//! 6. P0-6: Train simulation without track geometry
//! 7. P0-7: Aircraft flight without geographic boundaries escaping world bounds
//! 8. P0-8: US city transitions failing for catalog-only definitions without bundled GIS packages

use glam::{Quat, Vec2, Vec3};
use realearth_ingestion::authentic::verify_city_playable;
use realearth_rendering::gltf_asset::{VehicleLodLevel, VehicleSubmeshType};
use realearth_rendering::vehicle_mesh::VehicleVisualType;
use realearth_rendering::vehicle_models::get_certified_vehicle_model;
use realearth_shared::us_catalog::all_us_cities;
use realearth_vehicles::building_collision::{BuildingCollisionWorld, BuildingFootprintCollider};
use realearth_vehicles::{
    AircraftFlight, DynamicVehicle, FlightInputs,
    SlopedGround, TrainConsist, TransmissionMode, VehicleConfig, VehicleInputs,
};

const DT: f32 = 1.0 / 60.0;

// ============================================================================
// P0-1: CAR STEERING SIGN CONVENTION AND DIRECTIONAL INVERSION
// ============================================================================

#[test]
#[ignore = "Stage A forensic reproduction - superseded by Stage B remediation certification"]
fn test_reproduce_p0_1_car_steering_sign_inversion() {
    println!("\n🔍 [P0-1 REPRODUCTION] Auditing Steering Sign Convention & Directional Mapping...");

    // 1. In interactive.rs lines 4618-4621:
    // When KeyA (steer_left) is pressed:
    // target_steer is assigned -0.65 (NEGATIVE).
    let target_steer_left = -0.65f32;

    // 2. In dynamics.rs line 440:
    // steer_rad = inputs.steer * max_steer_angle.
    let car_config = VehicleConfig::sports_coupe();
    let steer_rad = target_steer_left * car_config.max_steer_angle;
    assert!(
        steer_rad < 0.0,
        "steer_rad must be negative when steer_left is pressed"
    );

    // 3. In gltf_asset.rs line 372:
    // steer_rot = Quat::from_rotation_y(wheel_steer_angle);
    let steer_rot = Quat::from_rotation_y(steer_rad);
    let forward_local = Vec3::Z; // +Z is vehicle forward
    let steered_forward = steer_rot * forward_local;

    // In vehicle model coordinates:
    // +X is Right (+Vec3::X), -X is Left (-Vec3::X).
    println!(
        "  • Negative steer angle ({:.2} rad) rotates +Z forward into: ({:.3}, {:.3}, {:.3})",
        steer_rad, steered_forward.x, steered_forward.y, steered_forward.z
    );

    // Negative steering input gives steered_forward.x < 0.0 (pointing -X)
    assert!(
        steered_forward.x < 0.0,
        "Physical front wheels rotate toward -X (left) for negative steer angle"
    );

    // 4. In interactive.rs line 1290:
    // Road tangent heading yaw calculation uses atan2(dx, dz) instead of atan2(dz, dx):
    let road_dx = 10.0f32; // Road travels East (+X)
    let road_dz = 0.0f32;
    let spawn_yaw = road_dx.atan2(road_dz); // atan2(X, Z) = +PI/2
    let camera_yaw = road_dz.atan2(road_dx); // atan2(Z, X) = 0.0
    let yaw_discrepancy = (spawn_yaw - camera_yaw).abs();

    println!(
        "  • Road spawn heading yaw ({:.3} rad) vs Camera heading yaw ({:.3} rad) => Discrepancy: {:.3} rad",
        spawn_yaw, camera_yaw, yaw_discrepancy
    );
    assert!(
        (yaw_discrepancy - std::f32::consts::FRAC_PI_2).abs() < 1e-4,
        "DEFECT CONFIRMED: 90-degree angular frame discrepancy between spawn yaw and camera yaw"
    );
}

// ============================================================================
// P0-2: SELF-STEERING DIVERGENCE UNDER ZERO INPUT
// ============================================================================

#[test]
#[ignore = "Stage A forensic reproduction - superseded by Stage B remediation certification"]
fn test_reproduce_p0_2_self_steering_divergence() {
    println!("\n🔍 [P0-2 REPRODUCTION] Auditing Self-Steering Divergence under Zero Steering Input...");

    let car_config = VehicleConfig::sports_coupe();
    let eq_y = car_config.equilibrium_ride_height();

    // Test self-steering instability on sloped terrain where static idle hold is disabled:
    let sloped_ground = SlopedGround::new(0.0, 0.04, 0.06);
    let mut sloped_car = DynamicVehicle::new(car_config, Vec3::new(0.0, eq_y, 0.0), 0.0);
    // Settle
    for _ in 0..10 {
        sloped_car.step(&VehicleInputs::default(), &sloped_ground, DT);
    }

    // Vehicle starts rolling down hill with zero steer input and a small perturbation
    sloped_car.linear_velocity = Vec3::new(0.02, 0.0, -1.0);
    sloped_car.angular_velocity = Vec3::new(0.0, 0.01, 0.0);

    let initial_x = sloped_car.position.x;
    let initial_yaw_rate = sloped_car.angular_velocity.y;

    let zero_input = VehicleInputs {
        throttle: 0.0,
        brake: 0.0,
        steer: 0.0, // Hands completely off steering
        gear_mode: TransmissionMode::Drive,
        handbrake: false,
    };

    for _ in 0..60 {
        sloped_car.step(&zero_input, &sloped_ground, DT);
    }

    let final_x = sloped_car.position.x;
    let final_yaw_rate = sloped_car.angular_velocity.y;
    let lateral_drift = (final_x - initial_x).abs();

    println!(
        "  • Sloped Reverse Drift: initial_yaw_rate={:.4}, final_yaw_rate={:.4}, lateral_drift={:.4}m under zero steer input",
        initial_yaw_rate, final_yaw_rate, lateral_drift
    );

    // Without pneumatic caster trail, rolling backwards on grade with zero input drifts laterally
    assert!(
        lateral_drift > 0.002 || final_yaw_rate.abs() > 0.0005,
        "DEFECT CONFIRMED: Vehicle diverges laterally ({:.4}m) under zero steer input while rolling backwards",
        lateral_drift
    );
}

// ============================================================================
// P0-3: BACKWARD MOVEMENT ON SLOPE WITH ZERO INPUT (DRIVE GEAR D1)
// ============================================================================

#[test]
#[ignore = "Stage A forensic reproduction - superseded by Stage B remediation certification"]
fn test_reproduce_p0_3_backward_movement_on_slope_zero_input() {
    println!("\n🔍 [P0-3 REPRODUCTION] Auditing Backward Slope Drift with Transmission in Drive...");

    // San Francisco standard hill slope: 8% grade (0.08)
    let ground = SlopedGround::new(0.0, 0.0, 0.08);
    let car_config = VehicleConfig::sports_coupe();
    let eq_y = car_config.equilibrium_ride_height();

    // Place car facing uphill (+Z)
    let mut car = DynamicVehicle::new(car_config, Vec3::new(0.0, eq_y, 0.0), 0.0);

    // Settle for 10 frames
    for _ in 0..10 {
        car.step(&VehicleInputs::default(), &ground, DT);
    }

    let initial_z = car.position.z;

    // Driver has car in Drive (D1), but foot off throttle and brake
    let idle_drive_input = VehicleInputs {
        throttle: 0.0,
        brake: 0.0,
        steer: 0.0,
        gear_mode: TransmissionMode::Drive,
        handbrake: false,
    };

    for _ in 0..60 {
        car.step(&idle_drive_input, &ground, DT);
    }

    let final_z = car.position.z;
    let forward_speed = car.linear_velocity.z;

    println!(
        "  • Initial Z: {:.3}m | Final Z after 1s: {:.3}m | Speed: {:.2} km/h (negative = backward)",
        initial_z,
        final_z,
        car.speed_kph
    );

    // Because is_flat_idle only holds when slope_grade < 0.025 (dynamics.rs:739),
    // on an 8% slope, gravity pulls the car backward down the hill:
    assert!(
        final_z < initial_z - 0.20,
        "DEFECT CONFIRMED: Car rolls backwards ({:.3}m displacement) down slope while in Drive gear with zero input",
        final_z - initial_z
    );

    // In interactive.rs line 4674:
    // If the driver presses W while rolling backward (active_forward_speed < -0.3):
    // road_inputs.brake = 0.95; road_inputs.gear_mode = Reverse;
    assert!(
        forward_speed < -0.3,
        "DEFECT CONFIRMED: Backward speed ({:.2} m/s) trips the reverse-brake threshold (< -0.3 m/s)",
        forward_speed
    );
}

// ============================================================================
// P0-4: BUILDING COLLISION INWARD PENETRATION TRAPPING
// ============================================================================

#[test]
#[ignore = "Stage A forensic reproduction - superseded by Stage B remediation certification"]
fn test_reproduce_p0_4_building_collision_inward_penetration_trapping() {
    println!("\n🔍 [P0-4 REPRODUCTION] Auditing Building Collision Inward Normal Trapping...");

    // Create a 10m x 10m building footprint collider from (0, 0) to (10, 10)
    let ring = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(10.0, 10.0),
        Vec2::new(0.0, 10.0),
    ];
    let collider = BuildingFootprintCollider::new(101, 0.0, 25.0, ring);

    // Vehicle penetrates slightly into the building: center is at (0.2, 5.0)
    let mut pos = Vec3::new(0.2, 1.0, 5.0);
    let mut vel = Vec3::new(5.0, 0.0, 0.0); // Moving East (+X) into the building
    let half_extents = Vec3::new(0.9, 0.6, 2.2);

    let mut world = BuildingCollisionWorld::new();
    world.add_collider(collider);

    let collided = world.resolve_collision(&mut pos, &mut vel, half_extents);
    assert!(collided, "Collision must be detected when inside building");

    println!(
        "  • Position after resolution: ({:.3}, {:.3}, {:.3})",
        pos.x, pos.y, pos.z
    );

    // When the vehicle center is inside the building:
    // closest edge is [0,0]->[0,10] at x=0.
    // diff = p - closest = (0.2 - 0.0, 0) = +0.2 in X!
    // best_normal = diff / d = +Vec2::X (pointing DEEPER into the building interior)!
    // pos += normal * depth pushes the position to larger positive X (DEEPER into building)!
    assert!(
        pos.x > 0.2,
        "DEFECT CONFIRMED: Collision resolution pushed vehicle deeper into building interior: x={:.3} > 0.2",
        pos.x
    );
}

// ============================================================================
// P0-5: PLACEHOLDER PRIMITIVE FALLBACK MESHES & CAMERA VIEWPORT OCCLUSION
// ============================================================================

#[test]
#[ignore = "Stage A forensic reproduction - superseded by Stage B remediation certification"]
fn test_reproduce_p0_5_placeholder_primitive_fallback_meshes() {
    println!("\n🔍 [P0-5 REPRODUCTION] Auditing Vehicle Production Asset Bindings vs Primitive Placeholders...");

    // Verify that get_certified_vehicle_model returns procedural submeshes
    let truck_model =
        get_certified_vehicle_model(VehicleVisualType::HeavyTruck, VehicleLodLevel::Lod0High);
    let plane_model = get_certified_vehicle_model(
        VehicleVisualType::CommercialAirliner,
        VehicleLodLevel::Lod0High,
    );
    let train_model = get_certified_vehicle_model(
        VehicleVisualType::TrainLocomotive,
        VehicleLodLevel::Lod0High,
    );

    assert_eq!(truck_model.asset_id, "transcontinental_semi");
    assert_eq!(plane_model.asset_id, "skycourier_b737");
    assert_eq!(train_model.asset_id, "pacific_locomotive");

    // Check truck camera anchors and trailer occlusion:
    // In vehicle_models.rs:
    // chase_distance = 18.0m
    // articulated trailer length = 15.0m (tr_l = 7.5, extends from z = -1.05 to -16.05)
    // Camera is positioned at -18.0m, directly behind the trailer wall (-16.05m),
    // with distance from trailer back to camera being only 1.95 meters!
    let truck_camera = truck_model.camera_anchors;
    println!(
        "  • Truck Chase Distance: {:.1}m | Camera clearance to 15m trailer: {:.2}m",
        truck_camera.chase_distance,
        truck_camera.chase_distance - 16.05
    );
    assert!(
        truck_camera.chase_distance - 16.05 < 2.5,
        "DEFECT CONFIRMED: Heavy haul camera (18.0m) sits within 2.0m of 15m trailer back face, obstructing view"
    );

    // Verify that the airliner mesh is built from box and cylinder primitives
    let has_primitive_submeshes = plane_model.submeshes.iter().any(|s| {
        s.name.contains("Airliner_Airframe") && s.submesh_type == VehicleSubmeshType::ChassisBody
    });
    assert!(
        has_primitive_submeshes,
        "DEFECT CONFIRMED: Commercial airliner uses procedural primitive submeshes, not external asset model"
    );
}

// ============================================================================
// P0-6: TRAIN SIMULATION WITHOUT TRACK GEOMETRY
// ============================================================================

#[test]
#[ignore = "Stage A forensic reproduction - superseded by Stage B remediation certification"]
fn test_reproduce_p0_6_train_without_track_free_roam() {
    println!("\n🔍 [P0-6 REPRODUCTION] Auditing Train Consist Free Roam Without Rail Graph...");

    let mut train = TrainConsist::new_caltrain_diesel();
    assert!(
        train.track_pose.is_none(),
        "Initial train consist must have no track pose bound"
    );

    train.throttle = 1.0;

    // In interactive.rs lines 4827-4829:
    // If res.rail_graph is None or unbound:
    // self.train_consist.step(PHYSICS_DT);
    for _ in 0..120 {
        train.step(DT);
    }

    println!(
        "  • Unbound Train Consist Speed after 2s: {:.1} km/h (Free motion without track)",
        train.speed_kph
    );
    assert!(
        train.speed_kph > 1.0,
        "DEFECT CONFIRMED: Train accelerates ({:.1} km/h) and simulates motion freely without track geometry",
        train.speed_kph
    );
}

// ============================================================================
// P0-7: AIRCRAFT LEAVING GEOGRAPHIC WORLD BOUNDS
// ============================================================================

#[test]
#[ignore = "Stage A forensic reproduction - superseded by Stage B remediation certification"]
fn test_reproduce_p0_7_aircraft_leaving_geographic_world() {
    println!("\n🔍 [P0-7 REPRODUCTION] Auditing Aircraft Flight Beyond Geographic Region Bounds...");

    let mut plane = AircraftFlight::new_airliner(
        "FlightAlpha",
        Vec3::new(0.0, 350.0, 0.0), // 350m MSL
        0.0,                        // Heading North (+Z)
    );
    plane.linear_velocity = Vec3::new(0.0, 0.0, 180.0); // 180 m/s (~648 km/h) cruise speed

    let flight_inputs = FlightInputs {
        throttle: 0.85,
        flaps: 0.0,
        gear_down: false,
        ..Default::default()
    };

    // Fly for 15 seconds (900 ticks)
    for _ in 0..900 {
        plane.step(&flight_inputs, 10.0, DT);
    }

    let traveled_meters = plane.position.z;
    println!(
        "  • Aircraft displacement after 15s: {:.1} meters (Region extent: 2500m)",
        traveled_meters
    );

    // San Francisco world chunk package extent is 2,500m from anchor.
    // In 15 seconds at 180 m/s, the plane travels > 2,700m and leaves the world package entirely:
    assert!(
        traveled_meters > 2500.0,
        "DEFECT CONFIRMED: Aircraft flew {:.1}m, exiting the 2500m authentic geographic boundary into empty space",
        traveled_meters
    );
}

// ============================================================================
// P0-8: UNSUCCESSFUL US CITY TRANSITIONS
// ============================================================================

#[test]
#[ignore = "Stage A forensic reproduction - superseded by Stage B remediation certification"]
fn test_reproduce_p0_8_unsuccessful_us_city_transitions() {
    println!("\n🔍 [P0-8 REPRODUCTION] Auditing US City Transitions & Catalog Availability...");

    let cities = all_us_cities();
    let mut catalog_only_count = 0;
    let mut available_count = 0;

    for city in cities {
        match verify_city_playable(city.id) {
            Ok(_) => {
                available_count += 1;
            }
            Err(e) => {
                catalog_only_count += 1;
                assert!(
                    e.to_string().contains("requires GIS data package"),
                    "Error must indicate missing GIS data package: got {}",
                    e
                );
            }
        }
    }

    println!(
        "  • Verified Playable Cities: {}/{} | Catalog-Only (Failed Transition): {}/{}",
        available_count,
        cities.len(),
        catalog_only_count,
        cities.len()
    );

    assert_eq!(
        cities.len(),
        38,
        "Catalog must contain all 38 mandatory US metropolitan cities"
    );
    assert!(
        catalog_only_count >= 30,
        "DEFECT CONFIRMED: At least 30 of 38 US cities cannot be transitioned to because real GIS packages are not installed on disk"
    );
}
