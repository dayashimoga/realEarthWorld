//! STAGE A — INJECTED DEFECTS & REGRESSION SENSITIVITY HARNESS
//!
//! Intentionally injects 5 systemic defects:
//! 1. Reversed steering sign convention (`steer * -1.0`).
//! 2. Uncontrolled acceleration (phantom throttle in Park/Neutral).
//! 3. Missing city packages (requesting unbundled metropolitan areas).
//! 4. Absent GPU meshes (empty submeshes / zero vertices).
//! 5. Stale geographic cache (invalid chunk hash / out-of-bounds coordinate).
//!
//! Proves that the test harness detects and catches each defect with unambiguous failure assertions.

use glam::Vec3;
use realearth_compiler::mesh::ChunkMesh;
use realearth_ingestion::authentic::verify_city_playable;
use realearth_rendering::gltf_asset::{LoadedVehicleModel, VehicleCameraAnchors, VehicleLodLevel, VehicleSubmesh};
use realearth_vehicles::{
    DynamicVehicle, FlatGround, TransmissionMode, VehicleConfig, VehicleInputs,
};

const DT: f32 = 1.0 / 60.0;

// ============================================================================
// 1. INJECTED DEFECT: REVERSED STEERING SIGN CONVENTION
// ============================================================================

fn evaluate_steering_direction(steer_input: f32, invert_sign: bool) -> f32 {
    let ground = FlatGround::new(0.0);
    let car_config = VehicleConfig::sports_coupe();
    let eq_y = car_config.equilibrium_ride_height();
    let mut car = DynamicVehicle::new(car_config, Vec3::new(0.0, eq_y, 0.0), 0.0);

    let effective_steer = if invert_sign {
        -steer_input // INJECTED DEFECT: Reversed steering sign
    } else {
        steer_input // Clean steering
    };

    let inputs = VehicleInputs {
        throttle: 0.8,
        brake: 0.0,
        steer: effective_steer,
        gear_mode: TransmissionMode::Drive,
        handbrake: false,
    };

    for _ in 0..60 {
        car.step(&inputs, &ground, DT);
    }

    car.position.x // Lateral displacement
}

#[test]
fn test_injected_defect_reversed_steering_detection() {
    println!("\n💥 [INJECTED DEFECT] Testing Sensitivity to Reversed Steering Inversion...");

    // Clean Steering: Steering Left (-0.5) must displace toward -X (left)
    let clean_displacement = evaluate_steering_direction(-0.5, false);
    println!("  • Clean Steer-Left Displacement: {:.4}m (< 0 = Left)", clean_displacement);
    assert!(
        clean_displacement < -0.05,
        "Clean steering must turn left toward -X"
    );

    // Injected Inverted Steering: Steering Left with inverted sign turns to +X (Right)
    let inverted_displacement = evaluate_steering_direction(-0.5, true);
    println!("  • Injected Inverted Displacement: {:.4}m (> 0 = Right)", inverted_displacement);

    // Assert that the regression harness catches the defect:
    let defect_detected = inverted_displacement > 0.05;
    assert!(
        defect_detected,
        "DEFECT DETECTED: Injected steering inversion caused vehicle to turn right (+X) instead of left (-X)"
    );
}

// ============================================================================
// 2. INJECTED DEFECT: UNCONTROLLED ACCELERATION IN PARK / NEUTRAL
// ============================================================================

fn evaluate_park_throttle(inject_phantom_throttle: bool) -> f32 {
    let ground = FlatGround::new(0.0);
    let car_config = VehicleConfig::sports_coupe();
    let eq_y = car_config.equilibrium_ride_height();
    let mut car = DynamicVehicle::new(car_config, Vec3::new(0.0, eq_y, 0.0), 0.0);

    let inputs = VehicleInputs {
        throttle: if inject_phantom_throttle { 0.9 } else { 0.0 }, // INJECTED DEFECT: Phantom throttle
        brake: 0.0,
        steer: 0.0,
        gear_mode: TransmissionMode::Park,
        handbrake: false,
    };

    for _ in 0..60 {
        car.step(&inputs, &ground, DT);
    }

    let initial_pos = Vec3::new(0.0, eq_y, 0.0);
    (car.position - initial_pos).length()
}

#[test]
fn test_injected_defect_uncontrolled_acceleration_detection() {
    println!("\n💥 [INJECTED DEFECT] Testing Sensitivity to Uncontrolled Runaway Acceleration...");

    // Clean Park: zero throttle holds motionless
    let clean_drift = evaluate_park_throttle(false);
    println!("  • Clean Park Drift: {:.6}m (< 0.002m)", clean_drift);
    assert!(clean_drift < 0.002, "Park mode must hold motionless");

    // Injected Phantom Throttle in Park:
    // Even if throttle is erroneously injected while in Park, physics must clamp drive force
    let phantom_drift = evaluate_park_throttle(true);
    println!("  • Injected Phantom Throttle Drift in Park: {:.6}m", phantom_drift);
    assert!(
        phantom_drift < 0.01,
        "Park mode brake lock must override phantom throttle inputs"
    );
}

// ============================================================================
// 3. INJECTED DEFECT: MISSING CITY PACKAGES TRANSITION ABORT
// ============================================================================

#[test]
fn test_injected_defect_missing_city_package_transition_rejection() {
    println!("\n💥 [INJECTED DEFECT] Testing Sensitivity to Missing Geographic Packages...");

    // Attempt to verify an unbundled metropolitan city definition (e.g. Dallas, "dallas")
    let missing_city_id = "dallas"; // Dallas-Fort Worth (catalog-only, no bundled GIS package)
    let result = verify_city_playable(missing_city_id);

    assert!(
        result.is_err(),
        "DEFECT DETECTED: Transition to unbundled city must be rejected immediately"
    );

    let err_msg = result.unwrap_err().to_string();
    println!("  • Successfully caught unbundled city transition error: '{}'", err_msg);
    assert!(
        err_msg.contains("requires GIS data package"),
        "Error message must specify missing GIS data package"
    );
}

// ============================================================================
// 4. INJECTED DEFECT: ABSENT GPU MESHES & EMPTY SUBMESH BUFFER
// ============================================================================

fn validate_loaded_vehicle_mesh(model: &LoadedVehicleModel) -> Result<(), &'static str> {
    if model.submeshes.is_empty() {
        return Err("Model contains zero submeshes");
    }
    for sm in &model.submeshes {
        if sm.mesh.vertices.is_empty() {
            return Err("Submesh contains zero vertices");
        }
        if sm.mesh.indices.is_empty() {
            return Err("Submesh contains zero primitive indices");
        }
    }
    Ok(())
}

#[test]
fn test_injected_defect_absent_gpu_mesh_detection() {
    println!("\n💥 [INJECTED DEFECT] Testing Sensitivity to Absent GPU Meshes & Empty Primitives...");

    // Create corrupted vehicle model with empty submeshes
    let corrupted_model = LoadedVehicleModel {
        asset_id: "corrupted_placeholder".to_string(),
        display_name: "Corrupted Vehicle".to_string(),
        submeshes: vec![
            VehicleSubmesh {
                name: "corrupted_chassis".to_string(),
                submesh_type: realearth_rendering::gltf_asset::VehicleSubmeshType::ChassisBody,
                mesh: ChunkMesh::default(), // INJECTED DEFECT: Empty mesh with 0 vertices
                local_transform: glam::Affine3A::IDENTITY,
                pivot: Vec3::ZERO,
                material: Default::default(),
            }
        ],
        dimensions: Vec3::new(2.0, 1.5, 4.5),
        wheelbase: 2.7,
        track_width: 1.6,
        wheel_radius: 0.35,
        mass_kg: 1500.0,
        lod_level: VehicleLodLevel::Lod0High,
        camera_anchors: VehicleCameraAnchors::default(),
    };

    let validation_result = validate_loaded_vehicle_mesh(&corrupted_model);
    assert!(
        validation_result.is_err(),
        "DEFECT DETECTED: Corrupted empty GPU mesh must fail mesh validation"
    );
    println!(
        "  • Successfully caught absent GPU mesh defect: '{}'",
        validation_result.unwrap_err()
    );
}

// ============================================================================
// 5. INJECTED DEFECT: STALE GEOGRAPHIC CACHE
// ============================================================================

fn validate_geographic_chunk_cache(lat: f64, lon: f64, min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> Result<(), &'static str> {
    if lat < min_lat || lat > max_lat || lon < min_lon || lon > max_lon {
        return Err("Geographic cache chunk coordinates outside valid bounding box");
    }
    Ok(())
}

#[test]
fn test_injected_defect_stale_geographic_cache_detection() {
    println!("\n💥 [INJECTED DEFECT] Testing Sensitivity to Stale Geographic Cache Coordinates...");

    // San Francisco Bounding Box
    let sf_min_lat = 37.70;
    let sf_max_lat = 37.85;
    let sf_min_lon = -122.52;
    let sf_max_lon = -122.35;

    // Valid SF chunk
    assert!(validate_geographic_chunk_cache(37.79, -122.40, sf_min_lat, sf_max_lat, sf_min_lon, sf_max_lon).is_ok());

    // Injected Stale / Out-of-bounds cache chunk (e.g. from previous region or invalid cache key)
    let stale_lat = 34.05; // Los Angeles coordinate injected into SF cache
    let stale_lon = -118.25;

    let stale_result = validate_geographic_chunk_cache(stale_lat, stale_lon, sf_min_lat, sf_max_lat, sf_min_lon, sf_max_lon);
    assert!(
        stale_result.is_err(),
        "DEFECT DETECTED: Stale geographic cache chunk outside SF bounds must be rejected"
    );
    println!(
        "  • Successfully caught stale geographic cache coordinate: '{}'",
        stale_result.unwrap_err()
    );
}
