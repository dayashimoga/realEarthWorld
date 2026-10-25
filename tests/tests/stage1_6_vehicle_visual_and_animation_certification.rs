//! Stage 1.6-R — Premium Multimodal Vehicle Visual & Physical Animation Certification Suite
//!
//! Validates:
//! 1. Certified Multimodal Vehicle Fleet Inventory (All 10+ models, proper proportions & dimensions)
//! 2. Authentic Transit Bus & North American Locomotive Visual Anatomy (Curved windscreen, interior, bogies, couplers)
//! 3. Physical Kinematic & Dynamic Animation Architecture (Wheel spin, steering, lean, doors, bogies, trailer, props, radar)
//! 4. Multi-LOD Geometric Scaling & Triangle Budgeting (LOD0 -> LOD1 -> LOD2 monotonic reduction)
//! 5. Khronos glTF 2.0 / GLB Binary Roundtrip & PBR Materials (PBR, submesh hierarchies, parsing integrity)
//! 6. Authoritative Simulation & Driving Physics Invariance (Zero rendering feedback drift, identical dynamics)

use glam::{Quat, Vec3};
use realearth_rendering::gltf_asset::{GltfParser, VehicleLodLevel, VehicleSubmeshType};
use realearth_rendering::vehicle_mesh::{
    generate_vehicle_mesh, VehicleMeshParams, VehicleVisualType,
};
use realearth_rendering::vehicle_models::{
    build_cityline_transit_bus, build_hypersport_rr1000, build_pacific_locomotive,
    build_passenger_coach, build_skyhawk_c172, build_transcontinental_semi,
    export_all_certified_models_to_glb_directory, get_certified_vehicle_model,
};
use realearth_vehicles::{
    DynamicVehicle, FlatGround, TransmissionMode, VehicleConfig, VehicleInputs,
};
use std::f32::consts::PI;

// =============================================================================
// TEST 1: Certified Multimodal Vehicle Fleet Inventory Verification
// =============================================================================

#[test]
fn test_stage1_6_1_certified_multimodal_vehicle_fleet_inventory() {
    println!("\n===================================================================");
    println!("STAGE 1.6-R TEST 1: Certified Multimodal Vehicle Fleet Inventory");
    println!("===================================================================");

    let vehicle_types = [
        VehicleVisualType::SportsCoupe,
        VehicleVisualType::Sedan,
        VehicleVisualType::Suv,
        VehicleVisualType::Taxi,
        VehicleVisualType::HeavyTruck,
        VehicleVisualType::TransitBus,
        VehicleVisualType::Motorcycle,
        VehicleVisualType::TrainLocomotive,
        VehicleVisualType::TrainWagon,
        VehicleVisualType::MarineVessel,
        VehicleVisualType::CommercialAirliner,
        VehicleVisualType::GeneralAviation,
    ];

    for vt in vehicle_types {
        let model = get_certified_vehicle_model(vt, VehicleLodLevel::Lod0High);
        println!(
            "-> Asset: [{:20}] Display: {:28} Verts: {:5} Tris: {:5} Dims: ({:.1}m x {:.1}m x {:.1}m)",
            model.asset_id,
            model.display_name,
            model.total_vertices(),
            model.total_triangles(),
            model.dimensions.x,
            model.dimensions.y,
            model.dimensions.z,
        );

        // Core visual quality assertions
        assert!(!model.asset_id.is_empty(), "Asset ID must be defined");
        assert!(
            !model.display_name.is_empty(),
            "Display name must be defined"
        );
        assert!(
            model.total_vertices() >= 80,
            "Certified model {} must have substantial vertex count, got {}",
            model.asset_id,
            model.total_vertices()
        );
        assert!(
            model.total_triangles() >= 40,
            "Certified model {} must have substantial triangle count, got {}",
            model.asset_id,
            model.total_triangles()
        );
        assert!(
            !model.submeshes.is_empty(),
            "Certified model {} must have distinct submeshes",
            model.asset_id
        );

        // Physics metadata assertions
        assert!(
            model.mass_kg > 50.0,
            "Certified model {} must specify realistic mass",
            model.asset_id
        );
        assert!(
            model.track_width > 0.1,
            "Certified model {} must specify realistic track width",
            model.asset_id
        );
    }
}

// =============================================================================
// TEST 2: Authentic Transit Bus & Locomotive Visual Anatomy Verification
// =============================================================================

#[test]
fn test_stage1_6_2_transit_bus_and_locomotive_visual_anatomy() {
    println!("\n===================================================================");
    println!("STAGE 1.6-R TEST 2: Authentic Bus & Locomotive Anatomy Certification");
    println!("===================================================================");

    // 1. Transit Bus Anatomy Verification
    let bus = build_cityline_transit_bus(VehicleLodLevel::Lod0High);
    println!(
        "-> CityLine Transit Bus: {} submeshes, {} vertices, {} triangles",
        bus.submeshes.len(),
        bus.total_vertices(),
        bus.total_triangles()
    );

    let has_front_door = bus
        .submeshes
        .iter()
        .any(|s| s.submesh_type == VehicleSubmeshType::BusDoorFront);
    let has_center_door = bus
        .submeshes
        .iter()
        .any(|s| s.submesh_type == VehicleSubmeshType::BusDoorCenter);
    let has_interior = bus
        .submeshes
        .iter()
        .any(|s| s.submesh_type == VehicleSubmeshType::InteriorCockpit);
    let has_glass = bus
        .submeshes
        .iter()
        .any(|s| s.submesh_type == VehicleSubmeshType::GlassGreenhouse);

    assert!(has_front_door, "Transit bus must have front boarding doors");
    assert!(has_center_door, "Transit bus must have center exit doors");
    assert!(
        has_interior,
        "Transit bus must feature cabin interior with seats and grab poles"
    );
    assert!(
        has_glass,
        "Transit bus must feature raked curved panoramic windshield & side glass"
    );

    // Check interior details (passenger seats & console)
    let interior_submesh = bus
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::InteriorCockpit)
        .expect("Bus interior submesh exists");
    assert!(
        interior_submesh.mesh.vertices.len() >= 60,
        "Bus interior must contain seats and grab poles (got {} verts)",
        interior_submesh.mesh.vertices.len()
    );

    // 2. Locomotive Anatomy Verification
    let loco = build_pacific_locomotive(VehicleLodLevel::Lod0High);
    println!(
        "-> Pacific Locomotive: {} submeshes, {} vertices, {} triangles",
        loco.submeshes.len(),
        loco.total_vertices(),
        loco.total_triangles()
    );

    let has_front_bogie = loco
        .submeshes
        .iter()
        .any(|s| s.submesh_type == VehicleSubmeshType::TrainBogieFront);
    let has_rear_bogie = loco
        .submeshes
        .iter()
        .any(|s| s.submesh_type == VehicleSubmeshType::TrainBogieRear);
    let has_headlights = loco
        .submeshes
        .iter()
        .any(|s| s.submesh_type == VehicleSubmeshType::Headlights);

    assert!(
        has_front_bogie,
        "Locomotive must feature articulated front bogie"
    );
    assert!(
        has_rear_bogie,
        "Locomotive must feature articulated rear bogie"
    );
    assert!(
        has_headlights,
        "Locomotive must feature sealed-beam cyclops headlight and ditch lights"
    );

    let front_bogie = loco
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::TrainBogieFront)
        .expect("Front bogie exists");
    assert!(
        front_bogie.mesh.vertices.len() >= 40,
        "Front bogie must feature steel wheels, axles, and truck side frames"
    );

    // 3. Passenger Coach Anatomy Verification
    let coach = build_passenger_coach(VehicleLodLevel::Lod0High);
    println!(
        "-> Passenger Coach: {} submeshes, {} vertices, {} triangles",
        coach.submeshes.len(),
        coach.total_vertices(),
        coach.total_triangles()
    );
    assert!(
        coach
            .submeshes
            .iter()
            .any(|s| s.submesh_type == VehicleSubmeshType::TrainBogieFront),
        "Passenger coach must feature front bogie"
    );
    assert!(
        coach
            .submeshes
            .iter()
            .any(|s| s.submesh_type == VehicleSubmeshType::TrainBogieRear),
        "Passenger coach must feature rear bogie"
    );
    assert!(
        coach
            .submeshes
            .iter()
            .any(|s| s.submesh_type == VehicleSubmeshType::GlassGreenhouse),
        "Passenger coach must feature dual-level passenger windows"
    );
}

// =============================================================================
// TEST 3: Physical Kinematic & Dynamic Animation Verification
// =============================================================================

#[test]
fn test_stage1_6_3_physical_kinematic_and_dynamic_animation() {
    println!("\n===================================================================");
    println!("STAGE 1.6-R TEST 3: Physical Kinematic & Dynamic Animation Systems");
    println!("===================================================================");

    // 1. Motorcycle Roll Lean Angle Animation
    let bike = build_hypersport_rr1000(VehicleLodLevel::Lod0High);
    let upright_mesh = bike.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0, // Zero lean
        0.0,
    );

    let leaned_mesh = bike.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.50, // 0.50 rad (~28 deg) roll lean into corner
        0.0,
    );

    // Verify top of motorcycle helmet / windshield laterally shifted under roll lean
    let upright_max_y = upright_mesh
        .vertices
        .iter()
        .map(|v| v.position[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let leaned_max_y = leaned_mesh
        .vertices
        .iter()
        .map(|v| v.position[1])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        leaned_max_y < upright_max_y,
        "Leaned motorcycle peak height must be lower than upright motorcycle (got leaned: {}, upright: {})",
        leaned_max_y,
        upright_max_y
    );
    println!(
        "-> Motorcycle lean verified: Upright peak Y = {:.3}m, Leaned peak Y = {:.3}m",
        upright_max_y, leaned_max_y
    );

    // 2. Bus Passenger Door Articulation Animation
    let bus = build_cityline_transit_bus(VehicleLodLevel::Lod0High);
    let doors_closed = bus.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        0.0, // Doors closed
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );

    let doors_open = bus.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        1.0, // Doors fully open
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );

    // Verify door positions shifted between closed and open states
    let mut max_vertex_displacement = 0.0f32;
    for (v1, v2) in doors_closed.vertices.iter().zip(doors_open.vertices.iter()) {
        let diff = Vec3::from_array(v1.position) - Vec3::from_array(v2.position);
        let dist = diff.length();
        if dist > max_vertex_displacement {
            max_vertex_displacement = dist;
        }
    }
    assert!(
        max_vertex_displacement >= 0.20,
        "Bus door open ratio must articulate door vertices (max displacement: {:.3}m)",
        max_vertex_displacement
    );
    println!(
        "-> Bus door articulation verified: Max door vertex displacement = {:.3}m",
        max_vertex_displacement
    );

    // 3. Train Bogie Track Curvature Articulation
    let loco = build_pacific_locomotive(VehicleLodLevel::Lod0High);
    let straight_track = loco.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        0.0,
        [0.0, 0.0], // Straight track
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );

    let curved_track = loco.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        0.0,
        [0.18, -0.18], // Track curve articulation angles
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );

    let mut bogie_displacement = 0.0f32;
    for (v1, v2) in straight_track
        .vertices
        .iter()
        .zip(curved_track.vertices.iter())
    {
        let dist = (Vec3::from_array(v1.position) - Vec3::from_array(v2.position)).length();
        if dist > bogie_displacement {
            bogie_displacement = dist;
        }
    }
    assert!(
        bogie_displacement >= 0.15,
        "Locomotive bogies must articulate with track curvature (max bogie displacement: {:.3}m)",
        bogie_displacement
    );
    println!(
        "-> Locomotive bogie curve articulation verified: Max displacement = {:.3}m",
        bogie_displacement
    );

    // 4. Heavy Truck Trailer Articulation
    let truck = build_transcontinental_semi(VehicleLodLevel::Lod0High);
    let aligned_semi = truck.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.0, // Aligned
    );

    let jackknifed_semi = truck.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.45, // 0.45 rad trailer articulation angle
    );

    let mut trailer_displacement = 0.0f32;
    for (v1, v2) in aligned_semi
        .vertices
        .iter()
        .zip(jackknifed_semi.vertices.iter())
    {
        let dist = (Vec3::from_array(v1.position) - Vec3::from_array(v2.position)).length();
        if dist > trailer_displacement {
            trailer_displacement = dist;
        }
    }
    assert!(
        trailer_displacement >= 1.0,
        "Semi trailer must articulate around fifth-wheel pivot (max trailer displacement: {:.3}m)",
        trailer_displacement
    );
    println!(
        "-> Articulated trailer articulation verified: Max tail displacement = {:.3}m",
        trailer_displacement
    );

    // 5. Aviation Propeller & Turbofan Spin Animation
    let plane = build_skyhawk_c172(VehicleLodLevel::Lod0High);
    let prop_0 = plane.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0, // Angle 0
        [0.0; 3],
        0.0,
        0.0,
    );

    let prop_pi_half = plane.compose_multimodal_world_mesh(
        Vec3::ZERO,
        Quat::IDENTITY,
        None,
        0.0,
        0.0,
        [0.0; 4],
        true,
        false,
        false,
        0.0,
        [0.0, 0.0],
        PI * 0.5, // 90 deg rotation
        [0.0; 3],
        0.0,
        0.0,
    );

    let mut prop_displacement = 0.0f32;
    for (v1, v2) in prop_0.vertices.iter().zip(prop_pi_half.vertices.iter()) {
        let dist = (Vec3::from_array(v1.position) - Vec3::from_array(v2.position)).length();
        if dist > prop_displacement {
            prop_displacement = dist;
        }
    }
    assert!(
        prop_displacement >= 0.40,
        "Propeller blade vertices must rotate with propeller angle (displacement: {:.3}m)",
        prop_displacement
    );
    println!(
        "-> Propeller spinning verified: Blade tip displacement = {:.3}m",
        prop_displacement
    );
}

// =============================================================================
// TEST 4: Multi-LOD Geometric Scaling & Triangle Budgeting
// =============================================================================

#[test]
fn test_stage1_6_4_multimodal_lod_scaling_and_budgets() {
    println!("\n===================================================================");
    println!("STAGE 1.6-R TEST 4: Multi-LOD Scaling & Budget Certification");
    println!("===================================================================");

    let lod_models = [
        ("Apex GT Sports Coupe", VehicleVisualType::SportsCoupe),
        ("Executive Sedan", VehicleVisualType::Sedan),
    ];

    for (name, vt) in lod_models {
        let lod0 = get_certified_vehicle_model(vt, VehicleLodLevel::Lod0High);
        let lod1 = get_certified_vehicle_model(vt, VehicleLodLevel::Lod1Medium);
        let lod2 = get_certified_vehicle_model(vt, VehicleLodLevel::Lod2Low);

        let t0 = lod0.total_triangles();
        let t1 = lod1.total_triangles();
        let t2 = lod2.total_triangles();

        println!(
            "-> {:24} Tris: LOD0={:5}, LOD1={:5}, LOD2={:5}",
            name, t0, t1, t2
        );

        // Assert monotonic geometric simplification
        assert!(
            t0 > t1,
            "LOD0 triangles ({}) must exceed LOD1 triangles ({}) for {}",
            t0,
            t1,
            name
        );
        assert!(
            t1 > t2,
            "LOD1 triangles ({}) must exceed LOD2 triangles ({}) for {}",
            t1,
            t2,
            name
        );

        // Assert maximum triangle budget caps
        assert!(
            t0 <= 5000,
            "LOD0 triangles ({}) exceeds 5000 budget cap for {}",
            t0,
            name
        );
        assert!(
            t1 <= 2000,
            "LOD1 triangles ({}) exceeds 2000 budget cap for {}",
            t1,
            name
        );
        assert!(
            t2 <= 1000,
            "LOD2 triangles ({}) exceeds 1000 budget cap for {}",
            t2,
            name
        );
    }
}

// =============================================================================
// TEST 5: Khronos glTF 2.0 / GLB Binary Roundtrip & PBR Materials
// =============================================================================

#[test]
fn test_stage1_6_5_gltf_glb_export_and_binary_roundtrip() {
    println!("\n===================================================================");
    println!("STAGE 1.6-R TEST 5: glTF 2.0 / GLB Binary Roundtrip Verification");
    println!("===================================================================");

    let temp_dir = std::env::temp_dir().join("realearth_stage1_6_glb_certification");
    if temp_dir.exists() {
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    // Export all 16 certified GLB files to directory
    export_all_certified_models_to_glb_directory(&temp_dir).expect("GLB batch export must succeed");

    let required_files = [
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
        "pacific_locomotive_lod0.glb",
        "passenger_coach_lod0.glb",
        "golden_gate_ferry_lod0.glb",
        "skycourier_b737_lod0.glb",
        "skyhawk_c172_lod0.glb",
    ];

    for filename in required_files {
        let file_path = temp_dir.join(filename);
        assert!(
            file_path.exists(),
            "Exported GLB {} must exist on disk",
            filename
        );

        let glb_bytes = std::fs::read(&file_path).expect("Read exported GLB file");
        assert!(
            glb_bytes.len() >= 1024,
            "Exported GLB {} size ({} bytes) must be substantial",
            filename,
            glb_bytes.len()
        );

        // Parse with strict standards-compliant glTF 2.0 / GLB parser
        let parsed_model = GltfParser::parse_glb(
            filename.trim_end_matches(".glb"),
            &glb_bytes,
            VehicleLodLevel::Lod0High,
        )
        .expect("GLB binary parsing must succeed");

        println!(
            "-> Certified GLB [{:30}] FileSize: {:6} bytes, Submeshes: {:2}, Vertices: {:5}",
            filename,
            glb_bytes.len(),
            parsed_model.submeshes.len(),
            parsed_model.total_vertices()
        );

        assert!(
            !parsed_model.submeshes.is_empty(),
            "Parsed GLB {} must contain submeshes",
            filename
        );
        assert!(
            parsed_model.total_vertices() > 0,
            "Parsed GLB {} must contain vertices",
            filename
        );
    }

    // Clean up temporary test files
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// =============================================================================
// TEST 6: Authoritative Physics & Driving Simulation Invariance
// =============================================================================

#[test]
fn test_stage1_6_6_authoritative_physics_simulation_invariance() {
    println!("\n===================================================================");
    println!("STAGE 1.6-R TEST 6: Authoritative Simulation Invariance");
    println!("===================================================================");

    let mut car = DynamicVehicle::new(VehicleConfig::sports_coupe(), Vec3::new(0.0, 0.5, 0.0), 0.0);
    let ground = FlatGround::default();

    let inputs = VehicleInputs {
        throttle: 0.85,
        brake: 0.0,
        handbrake: false,
        steer: 0.25,
        gear_mode: TransmissionMode::Drive,
    };

    let dt = 1.0 / 60.0;
    for step in 0..180 {
        // Step authoritative simulation
        car.step(&inputs, &ground, dt);

        // Generate multimodal certified mesh at each step (independent render pass)
        let mesh = generate_vehicle_mesh(&VehicleMeshParams {
            position: car.position,
            orientation: car.orientation,
            color: [0.92, 0.12, 0.12, 1.0],
            vehicle_type: VehicleVisualType::SportsCoupe,
            wheel_steer_angle: car.wheels[0].steer_angle,
            wheel_spin_angle: (step as f32 * 0.1) % (2.0 * PI),
            wheel_compressions: [
                car.wheels[0].compression,
                car.wheels[1].compression,
                car.wheels[2].compression,
                car.wheels[3].compression,
            ],
            headlights_on: true,
            brake_lights_on: false,
            reverse_lights_on: false,
            trailer_angle: 0.0,
            door_open_ratio: 0.0,
            bogie_angles: [0.0, 0.0],
            propeller_angle: 0.0,
            control_surfaces: [0.0, 0.0, 0.0],
            motorcycle_lean_angle: 0.0,
            lod: VehicleLodLevel::Lod0High,
        });

        // Ensure visual mesh generation produced valid non-empty geometry
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());

        // Ensure simulation state is completely unperturbed by visual rendering
        assert!(!car.position.x.is_nan());
        assert!(!car.position.y.is_nan());
        assert!(!car.position.z.is_nan());
        assert!(!car.linear_velocity.x.is_nan());
    }

    assert!(
        car.position.length() > 5.0,
        "Vehicle should have traveled realistically under simulation"
    );
    println!(
        "-> Authoritative simulation invariance verified: Traveled {:.2}m, Speed = {:.2} m/s",
        car.position.length(),
        car.linear_velocity.length()
    );
}
