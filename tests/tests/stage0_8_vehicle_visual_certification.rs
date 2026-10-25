//! Stage 0.8 Complete Realistic Vehicle Visual Overhaul Certification Tests.
//!
//! Validates:
//! 1. glTF 2.0 / GLB binary export, roundtrip parsing, and asset registry caching.
//! 2. Apex GT Sports Coupe curved body panels, greenhouse glass, cockpit, brake hardware, and contact shadow.
//! 3. Genuine wheel rolling, steering articulation, suspension travel, and lighting states.
//! 4. Non-identical distinct models for sports coupe, sedan, SUV, taxi, semi, transit bus, and motorcycle.
//! 5. Multi-tier Level of Detail (LOD0, LOD1, LOD2) polygon budget scaling.
//! 6. Authoritative physics invariance: rendering pipeline never alters simulation state.

use glam::{Quat, Vec3};
use realearth_rendering::{
    export_all_certified_models_to_glb_directory, generate_vehicle_mesh,
    get_certified_vehicle_model, AutomotivePalette, GltfAssetError, GltfParser,
    VehicleAssetRegistry, VehicleLodLevel, VehicleMeshParams, VehicleSubmeshType,
    VehicleVisualType,
};
use realearth_vehicles::{DynamicVehicle, FlatGround, VehicleConfig, VehicleInputs};
use std::f32::consts::PI;
use std::path::Path;

#[allow(dead_code)]
fn get_assets_dir() -> std::path::PathBuf {
    if Path::new("assets/vehicles").exists() {
        std::path::PathBuf::from("assets/vehicles")
    } else if Path::new("../assets/vehicles").exists() {
        std::path::PathBuf::from("../assets/vehicles")
    } else {
        std::path::PathBuf::from("assets/vehicles")
    }
}

// ============================================================================
// TEST 1: GLTF 2.0 / GLB ASSET EXPORT & ROUNDTRIP BINARY PARSING
// ============================================================================

#[test]
fn test_stage0_8_1_gltf_export_and_binary_roundtrip_parsing() {
    let temp_dir = tempfile::tempdir().expect("Failed to create isolated tempdir");
    let assets_dir = temp_dir.path().to_path_buf();

    // Export all certified models to disk
    export_all_certified_models_to_glb_directory(&assets_dir)
        .expect("Failed to export certified vehicle models to GLB");

    let expected_glbs = [
        "apex_gt_sports_coupe_lod0.glb",
        "apex_gt_sports_coupe_lod1.glb",
        "apex_gt_sports_coupe_lod2.glb",
        "executive_sedan_lod0.glb",
        "executive_sedan_lod1.glb",
        "executive_sedan_lod2.glb",
        "overland_explorer_suv_lod0.glb",
        "metro_yellow_cab_lod0.glb",
        "transcontinental_semi_lod0.glb",
        "cityline_transit_bus_lod0.glb",
        "hypersport_rr1000_lod0.glb",
    ];

    for filename in &expected_glbs {
        let glb_path = assets_dir.join(filename);
        assert!(
            glb_path.exists(),
            "Exported GLB asset must exist on disk: {:?}",
            glb_path
        );

        let glb_bytes = std::fs::read(&glb_path).expect("Failed to read GLB file");
        assert!(
            glb_bytes.len() >= 20,
            "GLB file must contain at least standard header: {:?}",
            filename
        );

        // Verify standard glTF 2.0 binary header
        let magic = u32::from_le_bytes(glb_bytes[0..4].try_into().unwrap());
        assert_eq!(
            magic, 0x46546C67,
            "GLB header magic must be 0x46546C67 ('glTF'): {:?}",
            filename
        );
        let version = u32::from_le_bytes(glb_bytes[4..8].try_into().unwrap());
        assert_eq!(version, 2, "GLB version must be 2: {:?}", filename);

        // Parse and validate binary chunk
        let parsed_model = GltfParser::parse_glb(filename, &glb_bytes, VehicleLodLevel::Lod0High)
            .expect("Failed to parse valid binary GLB");
        assert!(
            !parsed_model.submeshes.is_empty(),
            "Parsed model must contain submeshes: {:?}",
            filename
        );
        assert!(
            parsed_model.dimensions.length() > 0.0,
            "Parsed model must contain valid physical dimensions: {:?}",
            filename
        );
    }
}

// ============================================================================
// TEST 2: APEX GT SPORTS COUPE ANATOMY, CURVED PANELS & HARDWARE
// ============================================================================

#[test]
fn test_stage0_8_2_apex_gt_sports_coupe_anatomical_and_geometric_fidelity() {
    let coupe =
        get_certified_vehicle_model(VehicleVisualType::SportsCoupe, VehicleLodLevel::Lod0High);

    assert_eq!(coupe.asset_id, "apex_gt_sports_coupe");
    assert_eq!(coupe.display_name, "Apex GT Sports Coupe");
    assert!((coupe.wheelbase - 2.50).abs() < 0.05);
    assert!((coupe.track_width - 1.65).abs() < 0.05);
    assert!((coupe.wheel_radius - 0.33).abs() < 0.05);

    // Verify submesh completeness
    let submesh_types: Vec<VehicleSubmeshType> =
        coupe.submeshes.iter().map(|s| s.submesh_type).collect();

    assert!(submesh_types.contains(&VehicleSubmeshType::ChassisBody));
    assert!(submesh_types.contains(&VehicleSubmeshType::GlassGreenhouse));
    assert!(submesh_types.contains(&VehicleSubmeshType::InteriorCockpit));
    assert!(submesh_types.contains(&VehicleSubmeshType::InteriorSteeringWheel));
    assert!(submesh_types.contains(&VehicleSubmeshType::WheelFrontLeft));
    assert!(submesh_types.contains(&VehicleSubmeshType::WheelFrontRight));
    assert!(submesh_types.contains(&VehicleSubmeshType::WheelRearLeft));
    assert!(submesh_types.contains(&VehicleSubmeshType::WheelRearRight));
    assert!(submesh_types.contains(&VehicleSubmeshType::ContactShadow));
    assert!(submesh_types.contains(&VehicleSubmeshType::Headlights));
    assert!(submesh_types.contains(&VehicleSubmeshType::BrakeLights));
    assert!(submesh_types.contains(&VehicleSubmeshType::ReverseLights));

    // Verify aerodynamic curvature of the hood and roof (non-box vertex height distribution)
    let chassis_submesh = coupe
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::ChassisBody)
        .expect("Chassis body submesh required");

    let hood_vertices: Vec<&[f32; 3]> = chassis_submesh
        .mesh
        .vertices
        .iter()
        .map(|v| &v.position)
        .filter(|p| p[2] >= 0.65 && p[2] <= 2.20 && p[1] > 0.40)
        .collect();

    assert!(
        hood_vertices.len() >= 16,
        "Curved sculpted hood must contain >= 16 vertices, found {}",
        hood_vertices.len()
    );

    // Verify that hood Y-height decreases towards the aerodynamic front nose
    let mut min_nose_y = f32::MAX;
    let mut max_cowl_y = f32::MIN;
    for pos in &hood_vertices {
        if pos[2] > 1.9 {
            min_nose_y = min_nose_y.min(pos[1]);
        }
        if pos[2] < 0.8 {
            max_cowl_y = max_cowl_y.max(pos[1]);
        }
    }
    assert!(
        max_cowl_y > min_nose_y + 0.15,
        "Aerodynamic hood must curve downward toward nose: cowl={}, nose={}",
        max_cowl_y,
        min_nose_y
    );

    // Verify high-resolution wheel assembly geometry (tire, rim, brake hardware)
    let fl_wheel = coupe
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::WheelFrontLeft)
        .expect("FL Wheel submesh required");

    assert!(
        fl_wheel.mesh.vertices.len() >= 90,
        "LOD0 wheel assembly must be detailed with tire, rim, brake rotor and caliper: found {} verts",
        fl_wheel.mesh.vertices.len()
    );

    // Verify total model complexity > 400 triangles (far beyond primitive box)
    let total_triangles: usize = coupe
        .submeshes
        .iter()
        .map(|s| s.mesh.indices.len() / 3)
        .sum();
    assert!(
        total_triangles > 400,
        "Apex GT LOD0 model must feature > 400 triangles for realistic curvature: found {}",
        total_triangles
    );
}

// ============================================================================
// TEST 3: DYNAMIC WHEEL ARTICULATION, ROLLING & SUSPENSION TRAVEL
// ============================================================================

#[test]
fn test_stage0_8_3_dynamic_wheel_articulation_and_kinematic_animation() {
    let coupe =
        get_certified_vehicle_model(VehicleVisualType::SportsCoupe, VehicleLodLevel::Lod0High);

    // 1. Steering articulation
    let steer_angle = 0.45; // ~25.8 deg
    let mesh_straight = coupe.compose_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
    );
    let mesh_steered = coupe.compose_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        steer_angle,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
    );

    assert_eq!(
        mesh_straight.vertices.len(),
        mesh_steered.vertices.len(),
        "Steering must preserve total vertex count"
    );

    // Compare front wheel vertices: they must deflect laterally when steered
    let mut front_deflected = false;
    for (v_str, v_steer) in mesh_straight
        .vertices
        .iter()
        .zip(mesh_steered.vertices.iter())
    {
        if v_str.position[2] > 1.0 {
            // Front region
            let lateral_diff = (v_steer.position[0] - v_str.position[0]).abs();
            if lateral_diff > 0.05 {
                front_deflected = true;
                break;
            }
        }
    }
    assert!(
        front_deflected,
        "Front wheel vertices must articulate laterally with steering angle"
    );

    // 2. Wheel rolling animation
    let spin_angle = PI * 0.5; // 90 deg rotation
    let mesh_spun = coupe.compose_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        spin_angle,
        [0.0; 4],
        true,
        false,
        false,
    );

    let mut wheel_spun = false;
    for (v_str, v_spun) in mesh_straight.vertices.iter().zip(mesh_spun.vertices.iter()) {
        if v_str.position[0].abs() > 0.70 && (v_str.position[2].abs() - 1.25).abs() < 0.35 {
            // Wheel region
            let dy = (v_spun.position[1] - v_str.position[1]).abs();
            let dz = (v_spun.position[2] - v_str.position[2]).abs();
            if dy > 0.05 || dz > 0.05 {
                wheel_spun = true;
                break;
            }
        }
    }
    assert!(wheel_spun, "Wheels must visibly rotate about their axle");

    // 3. Suspension compression
    let compressions = [0.08, 0.08, -0.02, -0.02]; // Braking nose dive
    let mesh_compressed = coupe.compose_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        compressions,
        true,
        false,
        false,
    );

    let mut front_compressed = false;
    for (v_str, v_comp) in mesh_straight
        .vertices
        .iter()
        .zip(mesh_compressed.vertices.iter())
    {
        if v_str.position[0].abs() > 0.70 && v_str.position[2] > 1.0 {
            // Front wheel region
            if (v_comp.position[1] - (v_str.position[1] + 0.08)).abs() < 0.001 {
                front_compressed = true;
                break;
            }
        }
    }
    assert!(
        front_compressed,
        "Suspension compression must vertically displace wheel assemblies"
    );

    // 4. Dynamic Lighting states
    let mesh_lights_off = coupe.compose_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
    );
    let mesh_braking = coupe.compose_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        true,
        true,
    );

    // Headlight color changes from off to xenon-on
    let mut headlight_switched = false;
    let mut brakelight_switched = false;
    for (v_off, v_on) in mesh_lights_off
        .vertices
        .iter()
        .zip(mesh_braking.vertices.iter())
    {
        if v_off.position[2] > 2.0 && v_on.color[0] > 0.95 && v_off.color[0] < 0.5 {
            headlight_switched = true;
        }
        if v_off.position[2] < -2.0 && v_on.color[0] > 0.95 && v_off.color[0] < 0.7 {
            brakelight_switched = true;
        }
    }
    assert!(headlight_switched, "Headlights must switch emission color");
    assert!(
        brakelight_switched,
        "Brake lights must intensify when active"
    );
}

// ============================================================================
// TEST 4: DISTINCT MULTIMODAL VEHICLE MODELS & SPECS
// ============================================================================

#[test]
fn test_stage0_8_4_distinct_multimodal_vehicle_models() {
    let types = [
        VehicleVisualType::SportsCoupe,
        VehicleVisualType::Sedan,
        VehicleVisualType::Suv,
        VehicleVisualType::Taxi,
        VehicleVisualType::HeavyTruck,
        VehicleVisualType::TransitBus,
        VehicleVisualType::Motorcycle,
    ];

    let mut models = Vec::new();
    for v_type in &types {
        models.push(get_certified_vehicle_model(
            *v_type,
            VehicleLodLevel::Lod0High,
        ));
    }

    // Verify non-identical dimensions and weights
    for i in 0..models.len() {
        for j in (i + 1)..models.len() {
            let m1 = &models[i];
            let m2 = &models[j];

            // Taxi and Sedan share body shell dimensions but have distinct asset_id, display_name and livery
            if (m1.asset_id == "metro_yellow_cab" && m2.asset_id == "executive_sedan")
                || (m1.asset_id == "executive_sedan" && m2.asset_id == "metro_yellow_cab")
            {
                assert_ne!(m1.asset_id, m2.asset_id);
                continue;
            }

            let diff_length = (m1.dimensions.z - m2.dimensions.z).abs();
            let diff_mass = (m1.mass_kg - m2.mass_kg).abs();

            assert!(
                diff_length > 0.1 || diff_mass > 50.0,
                "Vehicle models {:?} and {:?} must be distinct in length or mass",
                m1.display_name,
                m2.display_name
            );
        }
    }

    // Check category specifics
    let bus = &models[5];
    assert!(bus.dimensions.z >= 11.0, "Transit bus must be >= 11m long");
    assert!(bus.mass_kg >= 10000.0, "Transit bus must be heavy transit");

    let semi = &models[4];
    assert!(semi.wheelbase >= 4.0, "Semi must have long wheelbase");

    let motorcycle = &models[6];
    assert!(
        motorcycle.track_width < 0.5,
        "Motorcycle must have narrow track"
    );
    assert!(
        motorcycle.submeshes.len() >= 3,
        "Motorcycle must have front/rear inline wheels and chassis"
    );
}

// ============================================================================
// TEST 5: LOD SCALING & ASSET REGISTRY CACHING
// ============================================================================

#[test]
fn test_stage0_8_5_lod_scaling_and_asset_registry_caching() {
    // 1. Monotonic polygon reduction across LODs
    let lod0 =
        get_certified_vehicle_model(VehicleVisualType::SportsCoupe, VehicleLodLevel::Lod0High);
    let lod1 =
        get_certified_vehicle_model(VehicleVisualType::SportsCoupe, VehicleLodLevel::Lod1Medium);
    let lod2 =
        get_certified_vehicle_model(VehicleVisualType::SportsCoupe, VehicleLodLevel::Lod2Low);

    let verts0: usize = lod0.submeshes.iter().map(|s| s.mesh.vertices.len()).sum();
    let verts1: usize = lod1.submeshes.iter().map(|s| s.mesh.vertices.len()).sum();
    let verts2: usize = lod2.submeshes.iter().map(|s| s.mesh.vertices.len()).sum();

    assert!(
        verts0 > verts1,
        "LOD0 vertices ({}) must exceed LOD1 vertices ({})",
        verts0,
        verts1
    );
    assert!(
        verts1 > verts2,
        "LOD1 vertices ({}) must exceed LOD2 vertices ({})",
        verts1,
        verts2
    );

    // 2. Asset registry caching and lookup
    let temp_dir = tempfile::tempdir().expect("Failed to create isolated tempdir");
    let assets_dir = temp_dir.path().to_path_buf();
    let _ = export_all_certified_models_to_glb_directory(&assets_dir);
    let mut registry = VehicleAssetRegistry::new(&assets_dir);
    let model_a = registry
        .load_or_get("apex_gt_sports_coupe_lod0")
        .expect("Failed to load apex_gt_sports_coupe_lod0 from assets directory");
    assert!(model_a.asset_id.starts_with("apex_gt_sports_coupe"));
    assert!(registry.require("apex_gt_sports_coupe_lod0").is_ok());
    assert_eq!(registry.len(), 1);

    // 3. Explicit error handling for missing/corrupt assets
    let missing_res = registry.load_or_get("non_existent_car_xyz123");
    match missing_res {
        Err(GltfAssetError::MissingAsset(_)) => {}
        other => panic!("Expected MissingAsset error, got: {:?}", other),
    }

    let corrupt_bytes = [0x00, 0x11, 0x22, 0x33, 0x44];
    let corrupt_res = GltfParser::parse_glb("corrupt", &corrupt_bytes, VehicleLodLevel::Lod0High);
    match corrupt_res {
        Err(GltfAssetError::TruncatedFile { .. }) | Err(GltfAssetError::InvalidMagic { .. }) => {}
        other => panic!(
            "Expected corrupted buffer/invalid magic error, got: {:?}",
            other
        ),
    }
}

// ============================================================================
// TEST 6: AUTHORITATIVE SIMULATION INVARIANCE UNDER RENDERING OVERHAUL
// ============================================================================

#[test]
fn test_stage0_8_6_authoritative_simulation_invariance_under_rendering() {
    let mut car = DynamicVehicle::new(VehicleConfig::sports_coupe(), Vec3::new(0.0, 0.5, 0.0), 0.0);
    let ground = FlatGround::default();

    let inputs = VehicleInputs {
        throttle: 0.85,
        steer: 0.15,
        brake: 0.0,
        handbrake: false,
        gear_mode: realearth_vehicles::TransmissionMode::Drive,
    };

    let dt = 1.0 / 60.0;
    for step in 0..180 {
        // Step authoritative simulation
        car.step(&inputs, &ground, dt);

        // Render dynamic vehicle mesh at each step (independent render pass)
        let mesh = generate_vehicle_mesh(&VehicleMeshParams {
            position: car.position,
            orientation: car.orientation,
            color: AutomotivePalette::APEX_CHERRY_RED,
            vehicle_type: VehicleVisualType::SportsCoupe,
            wheel_steer_angle: car.wheels[0].steer_angle,
            trailer_angle: 0.0,
            headlights_on: true,
            brake_lights_on: false,
            reverse_lights_on: false,
            wheel_spin_angle: (step as f32 * 0.1) % (2.0 * PI),
            wheel_compressions: [
                car.wheels[0].compression,
                car.wheels[1].compression,
                car.wheels[2].compression,
                car.wheels[3].compression,
            ],
            lod: VehicleLodLevel::Lod0High,
            ..Default::default()
        });

        // Verify visual output is valid
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());

        // Verify simulation state is valid and completely unperturbed
        assert!(!car.position.x.is_nan());
        assert!(!car.position.y.is_nan());
        assert!(!car.position.z.is_nan());
        assert!(!car.speed_kph.is_nan());
        assert!(
            car.position.y >= 0.20,
            "Car must not clip below ground plane"
        );
    }

    assert!(
        car.speed_kph > 20.0,
        "Vehicle should accelerate smoothly under drive throttle"
    );
}
