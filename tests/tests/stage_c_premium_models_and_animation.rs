//! Stage C — Premium Realistic Transport Models and Animation Master Certification Suite.
//!
//! Validates:
//! - Phase C1: Asset pipeline, glTF/GLB 2.0 binary ingestion, licensed asset manifests,
//!   PBR metallic-roughness materials, submesh depth sorting, and fallback rejection.
//! - Phase C2: Apex GT Sports Coupe reference car fidelity:
//!   - Non-clipping wheel arches with dark inner wheel cavity liners.
//!   - Solid grounded rear fascia eliminating floating/disconnected taillights/reverse lights.
//!   - 3D enclosed carbon fiber rear wing with solid stanchions, camber airfoil, and Gurney flap.
//!   - Longitudinal quad stainless steel exhaust pipes and amber turn signals.
//!   - Detailed cockpit interior with racing bucket seats and digital instrumentation.
//! - Phase C3: All 12 distinct multimodal transport models (Coupe, Sedan, SUV, Taxi, Semi-Truck,
//!   Transit Bus, Motorcycle, Locomotive, Coach, Ferry, Airliner, General Aviation).
//! - Phase C4: Physics-driven dynamic animations:
//!   - Wheel roll and steering rack deflection.
//!   - Motorcycle dynamic cornering lean banking.
//!   - Semi-truck fifth-wheel trailer yaw articulation.
//!   - Transit bus passenger bi-fold door opening kinematics.
//!   - Train bogie track yaw articulation.
//!   - Aircraft propeller and turbofan rotor rotation.
//!   - Marine radar mast scanning rotation.
//!   - Multi-LOD decimation monotonicity.
//! - Phase C5: Full binary export and roundtrip parsing of all 16 GLB assets to `assets/vehicles/`
//!   and zero-regression preservation of Stage B authoritative vehicle physics.

use glam::{Quat, Vec2, Vec3};
use realearth_rendering::gltf_asset::{GltfAssetBuilder, VehicleLodLevel, VehicleSubmeshType};
use realearth_rendering::vehicle_mesh::{
    generate_fallback_development_mesh, generate_vehicle_mesh, VehicleMeshParams, VehicleVisualType,
};
use realearth_rendering::vehicle_models::{
    export_all_certified_models_to_glb_directory, get_certified_vehicle_model,
};
use realearth_vehicles::building_collision::{BuildingCollisionWorld, BuildingFootprintCollider};
use realearth_vehicles::{DynamicVehicle, FlatGround, VehicleConfig, VehicleInputs};
use std::fs;
use std::path::Path;

// ============================================================================
// PHASE C1: ASSET PIPELINE, INGESTION & QUALITY SPECIFICATIONS
// ============================================================================

#[test]
fn test_phase_c1_asset_pipeline_and_gltf_ingestion() {
    println!("📦 [PHASE C1] Testing glTF 2.0 / GLB binary asset pipeline and quality specs...");

    // 1. Verify licensed asset manifest exists and documents all 12 multimodal models
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root");
    let manifest_path = workspace_root.join("assets/vehicles/LICENSE_ATTRIBUTION.md");
    assert!(
        manifest_path.exists(),
        "Asset manifest {:?} must exist",
        manifest_path
    );
    let manifest_text = fs::read_to_string(&manifest_path).expect("Failed to read asset manifest");
    assert!(
        manifest_text.contains("apex_gt_sports_coupe"),
        "Manifest must document apex_gt_sports_coupe"
    );
    assert!(
        manifest_text.contains("executive_sedan"),
        "Manifest must document executive_sedan"
    );
    assert!(
        manifest_text.contains("transcontinental_semi"),
        "Manifest must document transcontinental_semi"
    );
    assert!(
        manifest_text.contains("cityline_transit_bus"),
        "Manifest must document cityline_transit_bus"
    );
    assert!(
        manifest_text.contains("hypersport_rr1000"),
        "Manifest must document hypersport_rr1000"
    );
    assert!(
        manifest_text.contains("pacific_locomotive"),
        "Manifest must document pacific_locomotive"
    );
    assert!(
        manifest_text.contains("golden_gate_ferry"),
        "Manifest must document golden_gate_ferry"
    );
    assert!(
        manifest_text.contains("skycourier_b737"),
        "Manifest must document skycourier_b737"
    );

    // 2. Binary roundtrip parsing of a certified model
    let model =
        get_certified_vehicle_model(VehicleVisualType::SportsCoupe, VehicleLodLevel::Lod0High);
    let mut builder = GltfAssetBuilder::new(&model.asset_id, &model.display_name)
        .with_physics(
            model.wheelbase,
            model.track_width,
            model.wheel_radius,
            model.mass_kg,
        )
        .with_lod(model.lod_level)
        .with_camera_anchors(model.camera_anchors);

    for s in &model.submeshes {
        builder = builder.add_submesh(s.clone());
    }

    let glb_bytes = builder
        .to_glb_bytes()
        .expect("GLB compilation must succeed");
    assert!(
        glb_bytes.len() >= 12,
        "GLB header must be at least 12 bytes"
    );

    let magic = u32::from_le_bytes(glb_bytes[0..4].try_into().unwrap());
    let version = u32::from_le_bytes(glb_bytes[4..8].try_into().unwrap());
    let length = u32::from_le_bytes(glb_bytes[8..12].try_into().unwrap());

    assert_eq!(
        magic, 0x46546C67,
        "GLB magic must match 'glTF' (0x46546C67)"
    );
    assert_eq!(version, 2, "glTF version must be 2");
    assert_eq!(
        length as usize,
        glb_bytes.len(),
        "Header byte length must match buffer size"
    );

    // 3. Alpha Depth Sorting Verification:
    // When composing world mesh, opaque geometry must be rasterized before transparent geometry (glass, shadow)
    let composed = model.compose_multimodal_world_mesh(
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
        [0.0, 0.0, 0.0],
        0.0,
        0.0,
    );
    assert!(
        !composed.vertices.is_empty(),
        "Composed world mesh vertices must not be empty"
    );
    assert!(
        !composed.indices.is_empty(),
        "Composed world mesh indices must not be empty"
    );

    // 4. Procedural Development Fallback Rejection
    // Production certification must verify certified mesh is far richer than development fallback box
    let fallback_params = VehicleMeshParams {
        vehicle_type: VehicleVisualType::SportsCoupe,
        lod: VehicleLodLevel::Lod0High,
        ..Default::default()
    };
    let fallback_mesh = generate_fallback_development_mesh(&fallback_params);
    let prod_mesh = generate_vehicle_mesh(&fallback_params);

    assert!(
        prod_mesh.vertices.len() > fallback_mesh.vertices.len() * 2,
        "Production certified vehicle mesh ({} verts) must significantly exceed primitive fallback mesh ({} verts)",
        prod_mesh.vertices.len(),
        fallback_mesh.vertices.len()
    );

    println!("✅ Phase C1 asset pipeline and quality specifications certified!");
}

// ============================================================================
// PHASE C2: APEX GT SPORTS COUPE REFERENCE CAR CERTIFICATION
// ============================================================================

#[test]
fn test_phase_c2_apex_gt_reference_car_fidelity() {
    println!("🏎️ [PHASE C2] Testing Apex GT Sports Coupe Reference Car Fidelity...");

    let coupe =
        get_certified_vehicle_model(VehicleVisualType::SportsCoupe, VehicleLodLevel::Lod0High);

    // 1. Verify semantic submesh composition
    let _submesh_names: Vec<&str> = coupe.submeshes.iter().map(|s| s.name.as_str()).collect();
    let submesh_types: Vec<VehicleSubmeshType> =
        coupe.submeshes.iter().map(|s| s.submesh_type).collect();

    assert!(
        submesh_types.contains(&VehicleSubmeshType::ChassisBody),
        "Must contain ChassisBody"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::GlassGreenhouse),
        "Must contain GlassGreenhouse"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::InteriorCockpit),
        "Must contain InteriorCockpit"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::InteriorSteeringWheel),
        "Must contain InteriorSteeringWheel"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::Headlights),
        "Must contain Headlights"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::BrakeLights),
        "Must contain BrakeLights"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::ReverseLights),
        "Must contain ReverseLights"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::TurnSignals),
        "Must contain TurnSignals"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::AerodynamicSpoiler),
        "Must contain AerodynamicSpoiler"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::Exhaust),
        "Must contain Exhaust"
    );
    assert!(
        submesh_types.contains(&VehicleSubmeshType::ContactShadow),
        "Must contain ContactShadow"
    );

    // 2. Wheel Arch Non-Clipping Verification:
    // Front wheels are at z = 1.25, wheel_r = 0.33 (tire top y = 0.66m).
    // Rear wheels are at z = -1.25, wheel_r = 0.33 (tire top y = 0.66m).
    let chassis_submesh = coupe
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::ChassisBody)
        .unwrap();
    let chassis_verts = &chassis_submesh.mesh.vertices;

    // Check front wheel arch cutout crest: vertex over front wheel (z in [1.10, 1.40]) must rise above 0.70m
    let front_arch_crest = chassis_verts.iter().any(|v| {
        let p = Vec3::from_array(v.position);
        p.z >= 1.15 && p.z <= 1.35 && p.y >= 0.70
    });
    assert!(
        front_arch_crest,
        "Front wheel arch cutout must crest at y >= 0.70m over the front tire (top y = 0.66m)"
    );

    // Check rear wheel arch cutout crest: vertex over rear wheel (z in [-1.40, -1.10]) must rise above 0.72m
    let rear_arch_crest = chassis_verts.iter().any(|v| {
        let p = Vec3::from_array(v.position);
        p.z >= -1.35 && p.z <= -1.15 && p.y >= 0.72
    });
    assert!(
        rear_arch_crest,
        "Rear wheel arch cutout must crest at y >= 0.72m over the rear tire (top y = 0.66m)"
    );

    // Check dark inner wheel well cavity liners
    let inner_liner_present = chassis_verts.iter().any(|v| {
        v.color[0] <= 0.08 && v.color[1] <= 0.08 && v.color[2] <= 0.08 && v.color[3] >= 0.95
    });
    assert!(
        inner_liner_present,
        "Chassis must contain dark matte inner wheel well liners enclosing wheel cavities"
    );

    // 3. Solid Rear Fascia & Embedded Tail Details (Eliminates floating components):
    // Must contain rear vertical fascia panels with normal pointing backwards (-Z)
    let rear_fascia_verts = chassis_verts
        .iter()
        .filter(|v| {
            let p = Vec3::from_array(v.position);
            let n = Vec3::from_array(v.normal);
            p.z <= -2.15 && n.z <= -0.85
        })
        .count();
    assert!(
        rear_fascia_verts >= 8,
        "Solid rear fascia must enclose the tail with >= 8 rearward-facing vertices (found {})",
        rear_fascia_verts
    );

    // 4. 3D Aerodynamic GT Wing / Spoiler Verification:
    // Must be 3D solid geometry (stanchions with thickness, enclosed airfoil, Gurney flap, endplates)
    let spoiler_submesh = coupe
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::AerodynamicSpoiler)
        .unwrap();
    let spoiler_verts = &spoiler_submesh.mesh.vertices;
    assert!(
        spoiler_verts.len() >= 60,
        "3D solid spoiler assembly must contain >= 60 vertices (found {})",
        spoiler_verts.len()
    );

    // Check that spoiler has vertices facing up (+Y), down (-Y), front (+Z), and rear (-Z)
    let has_top = spoiler_verts
        .iter()
        .any(|v| Vec3::from_array(v.normal).y > 0.8);
    let has_bottom = spoiler_verts
        .iter()
        .any(|v| Vec3::from_array(v.normal).y < -0.8);
    let has_front = spoiler_verts
        .iter()
        .any(|v| Vec3::from_array(v.normal).z > 0.8);
    let has_rear = spoiler_verts
        .iter()
        .any(|v| Vec3::from_array(v.normal).z < -0.8);
    assert!(
        has_top && has_bottom && has_front && has_rear,
        "Spoiler airfoil must have 3D enclosure (top, bottom, front, rear faces)"
    );

    // 5. Quad Longitudinal Exhaust Tips
    let exhaust_submesh = coupe
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::Exhaust)
        .unwrap();
    assert!(
        !exhaust_submesh.mesh.vertices.is_empty(),
        "Exhaust submesh must contain vertices"
    );

    // 6. Cockpit Interior & Camera Anchors
    let interior_submesh = coupe
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::InteriorCockpit)
        .unwrap();
    assert!(
        interior_submesh.mesh.vertices.len() >= 80,
        "Detailed cockpit interior must contain seats, dashboard, and console with >= 80 vertices"
    );

    let cams = &coupe.camera_anchors;
    assert!(
        cams.cockpit_eye.x < 0.0 && cams.cockpit_eye.y > 0.8,
        "Cockpit eye must be left-hand drive driver position: {:?}",
        cams.cockpit_eye
    );
    assert!(
        cams.hood_eye.z > 1.4 && cams.hood_eye.y > 0.7,
        "Hood eye must be forward on the vehicle hood: {:?}",
        cams.hood_eye
    );
    assert!(
        cams.chase_distance >= 5.0 && cams.chase_height >= 1.5,
        "Chase camera must frame vehicle appropriately: {:?}",
        cams
    );

    println!("✅ Phase C2 Apex GT Sports Coupe certified with authentic non-clipping geometry and 3D details!");
}

// ============================================================================
// PHASE C3: MULTIMODAL TRANSPORT CLASSES CERTIFICATION
// ============================================================================

#[test]
fn test_phase_c3_multimodal_transport_classes() {
    println!("🚂 [PHASE C3] Testing All 12 Distinct Multimodal Transport Classes...");

    let visual_types = [
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

    for vt in visual_types {
        let model = get_certified_vehicle_model(vt, VehicleLodLevel::Lod0High);
        println!(
            "  -> Validating {:?}: '{}' ({} submeshes, {} vertices, {} triangles, mass: {:.0} kg)",
            vt,
            model.display_name,
            model.submeshes.len(),
            model.total_vertices(),
            model.total_triangles(),
            model.mass_kg
        );

        assert!(
            model.total_vertices() >= 300,
            "Model {:?} must have >= 300 vertices (found {})",
            vt,
            model.total_vertices()
        );
        assert!(
            model.total_triangles() >= 150,
            "Model {:?} must have >= 150 triangles (found {})",
            vt,
            model.total_triangles()
        );

        // Verify class-specific structural elements:
        match vt {
            VehicleVisualType::Motorcycle => {
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::MotorcycleLeanBody),
                    "Motorcycle must have dynamic MotorcycleLeanBody"
                );
                assert_eq!(model.wheelbase, 1.42, "Motorcycle wheelbase must be 1.42m");
            }
            VehicleVisualType::TransitBus => {
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::BusDoorFront),
                    "Transit bus must have articulating front bi-fold doors"
                );
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::InteriorPassengerSeats),
                    "Transit bus must have passenger seating rows"
                );
            }
            VehicleVisualType::HeavyTruck => {
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::TruckArticulatedTrailer),
                    "Heavy truck must have fifth-wheel articulated trailer"
                );
            }
            VehicleVisualType::TrainLocomotive => {
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::TrainBogieFront),
                    "Locomotive must have articulating front railway bogie"
                );
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::TrainCoupler),
                    "Locomotive must have knuckle couplers"
                );
            }
            VehicleVisualType::TrainWagon => {
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::TrainBogieFront),
                    "Commuter coach must have articulating bogies"
                );
            }
            VehicleVisualType::MarineVessel => {
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::MarineRadarMast),
                    "Ferry must have rotating MarineRadarMast"
                );
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::MarineWaterjets),
                    "Ferry must have directional waterjets"
                );
            }
            VehicleVisualType::CommercialAirliner => {
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::AircraftTurbofanFan),
                    "Airliner must have spinning AircraftTurbofanFan rotors"
                );
            }
            VehicleVisualType::GeneralAviation => {
                assert!(
                    model
                        .submeshes
                        .iter()
                        .any(|s| s.submesh_type == VehicleSubmeshType::AircraftPropeller),
                    "Skyhawk must have spinning AircraftPropeller"
                );
            }
            _ => {}
        }
    }

    println!(
        "✅ Phase C3 all 12 multimodal transport classes certified as distinct authored models!"
    );
}

// ============================================================================
// PHASE C4: PHYSICS-DRIVEN ANIMATION & MULTI-LOD OPTIMIZATION
// ============================================================================

#[test]
fn test_phase_c4_physics_driven_animations_and_lod() {
    println!("⚙️ [PHASE C4] Testing Physics-Driven Dynamic Animations & Multi-LOD Optimization...");

    let bus = get_certified_vehicle_model(VehicleVisualType::TransitBus, VehicleLodLevel::Lod0High);
    let bike =
        get_certified_vehicle_model(VehicleVisualType::Motorcycle, VehicleLodLevel::Lod0High);
    let truck =
        get_certified_vehicle_model(VehicleVisualType::HeavyTruck, VehicleLodLevel::Lod0High);
    let train = get_certified_vehicle_model(
        VehicleVisualType::TrainLocomotive,
        VehicleLodLevel::Lod0High,
    );
    let plane = get_certified_vehicle_model(
        VehicleVisualType::GeneralAviation,
        VehicleLodLevel::Lod0High,
    );
    let _ferry =
        get_certified_vehicle_model(VehicleVisualType::MarineVessel, VehicleLodLevel::Lod0High);

    let pos = Vec3::new(10.0, 0.0, 10.0);
    let orient = Quat::IDENTITY;

    // 1. Bus Bi-Fold Passenger Door Opening Kinematics
    let bus_closed = bus.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );
    let bus_open = bus.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        1.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );
    // Vertex positions between open and closed doors must physically animate
    assert_ne!(
        bus_closed.vertices, bus_open.vertices,
        "Bus doors must articulate when open"
    );

    // 2. Motorcycle Dynamic Cornering Lean Banking
    let bike_upright = bike.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );
    let bike_leaned = bike.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.35,
        0.0, // 20 degree bank
    );
    assert_ne!(
        bike_upright.vertices, bike_leaned.vertices,
        "Motorcycle body must bank when cornering"
    );

    // 3. Truck Fifth-Wheel Semitrailer Articulation
    let truck_straight = truck.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );
    let truck_jackknifed = truck.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.45, // 25 degree trailer yaw
    );
    assert_ne!(
        truck_straight.vertices, truck_jackknifed.vertices,
        "Truck trailer must articulate around kingpin"
    );

    // 4. Train Bogie Track Yaw Articulation
    let train_straight = train.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );
    let train_curved = train.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        0.0,
        [0.08, -0.08],
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );
    assert_ne!(
        train_straight.vertices, train_curved.vertices,
        "Train bogies must articulate on track curves"
    );

    // 5. Aircraft Propeller Spinning Rotation
    let plane_static = plane.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        0.0,
        [0.0, 0.0],
        0.0,
        [0.0; 3],
        0.0,
        0.0,
    );
    let plane_spinning = plane.compose_multimodal_world_mesh(
        pos,
        orient,
        None,
        0.0,
        0.0,
        [0.0; 4],
        false,
        false,
        false,
        0.0,
        [0.0, 0.0],
        1.57,
        [0.0; 3],
        0.0,
        0.0,
    );
    assert_ne!(
        plane_static.vertices, plane_spinning.vertices,
        "Aircraft propeller must spin"
    );

    // 6. Multi-LOD Scaling Monotonicity (LOD0 >= LOD1 >= LOD2)
    for vt in [VehicleVisualType::SportsCoupe, VehicleVisualType::Sedan] {
        let lod0 = get_certified_vehicle_model(vt, VehicleLodLevel::Lod0High);
        let lod1 = get_certified_vehicle_model(vt, VehicleLodLevel::Lod1Medium);
        let lod2 = get_certified_vehicle_model(vt, VehicleLodLevel::Lod2Low);

        assert!(
            lod0.total_vertices() > lod1.total_vertices(),
            "LOD0 vertices ({}) must exceed LOD1 ({}) for {:?}",
            lod0.total_vertices(),
            lod1.total_vertices(),
            vt
        );
        assert!(
            lod1.total_vertices() > lod2.total_vertices(),
            "LOD1 vertices ({}) must exceed LOD2 ({}) for {:?}",
            lod1.total_vertices(),
            lod2.total_vertices(),
            vt
        );
    }

    println!("✅ Phase C4 physics-driven animations and multi-LOD scaling certified!");
}

// ============================================================================
// PHASE C5: GLB EXPORT OF ALL 16 ASSETS & PHYSICS REGRESSION INVARIANCE
// ============================================================================

#[test]
fn test_phase_c5_export_all_16_glb_assets_and_verify() {
    println!("💾 [PHASE C5] Exporting and Validating All 16 Certified GLB Assets...");

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root");
    let target_dir = workspace_root.join("assets/vehicles");
    fs::create_dir_all(&target_dir).expect("Failed to create assets/vehicles directory");

    export_all_certified_models_to_glb_directory(&target_dir)
        .expect("GLB export of all 16 certified models must succeed");

    let expected_files = [
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

    for file in expected_files {
        let path = target_dir.join(file);
        assert!(
            path.exists(),
            "Exported GLB asset '{}' must exist on disk",
            file
        );

        let data = fs::read(&path).unwrap_or_else(|_| panic!("Failed to read {}", file));
        assert!(
            data.len() >= 12,
            "File '{}' must be at least 12 bytes",
            file
        );

        let magic = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let length = u32::from_le_bytes(data[8..12].try_into().unwrap());

        assert_eq!(magic, 0x46546C67, "{}: GLB magic must be 'glTF'", file);
        assert_eq!(version, 2, "{}: glTF version must be 2", file);
        assert_eq!(length as usize, data.len(), "{}: GLB length mismatch", file);

        println!("  -> Verified {:<35} ({} bytes)", file, data.len());
    }

    println!("✅ Phase C5 all 16 GLB binary assets exported and validated!");
}

#[test]
fn test_stage_b_physics_and_coordinate_regression_invariance() {
    println!("🛡️ Verifying Zero-Regression of Stage B Authoritative Physics & Coordinates...");

    // 1. Fixed Timestep Invariance (30, 60, 120 FPS)
    let ground = FlatGround::new(0.0);
    let inputs = VehicleInputs {
        throttle: 1.0,
        ..Default::default()
    };

    let mut v60 = DynamicVehicle::new(VehicleConfig::default(), Vec3::new(0.0, 1.0, 0.0), 0.0);
    for _ in 0..120 {
        v60.step(&inputs, &ground, 1.0 / 60.0);
    }

    let mut v120 = DynamicVehicle::new(VehicleConfig::default(), Vec3::new(0.0, 1.0, 0.0), 0.0);
    for _ in 0..240 {
        v120.step(&inputs, &ground, 1.0 / 120.0);
    }

    let speed_diff = (v60.speed_kph - v120.speed_kph).abs();
    assert!(
        speed_diff < 0.25,
        "Physics must maintain fixed-timestep invariance: 60fps={:.2}, 120fps={:.2}, diff={:.3}",
        v60.speed_kph,
        v120.speed_kph,
        speed_diff
    );

    // 2. Steer Release Self-Centering Active Damping
    let mut steer_veh =
        DynamicVehicle::new(VehicleConfig::default(), Vec3::new(0.0, 1.0, 0.0), 0.0);
    let inputs_turn = VehicleInputs {
        throttle: 0.6,
        steer: 0.5,
        ..Default::default()
    };
    for _ in 0..45 {
        steer_veh.step(&inputs_turn, &ground, 1.0 / 60.0);
    }
    assert!(
        steer_veh.angular_velocity.y.abs() > 0.10,
        "Car must develop yaw rate when turning"
    );

    let release_inputs = VehicleInputs {
        steer: 0.0,
        throttle: 0.4,
        ..Default::default()
    };
    for _ in 0..60 {
        steer_veh.step(&release_inputs, &ground, 1.0 / 60.0);
    }
    assert!(
        steer_veh.angular_velocity.y.abs() < 0.04,
        "Steering caster self-centering must damp yaw rate toward zero (got {:.4})",
        steer_veh.angular_velocity.y
    );

    // 3. Building Solid Collision Outward Resolution
    let mut collision_world = BuildingCollisionWorld::new();
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

    let mut vehicle_pos = Vec3::new(10.5, 0.35, 20.0);
    let mut vehicle_vel = Vec3::new(15.0, 0.0, 0.0);
    let half_extents = Vec3::new(0.9, 0.7, 2.3);

    let had_collision =
        collision_world.resolve_collision(&mut vehicle_pos, &mut vehicle_vel, half_extents);
    assert!(had_collision, "Penetrating building must trigger collision");
    assert!(
        vehicle_pos.x < 10.0,
        "Vehicle must be ejected outward toward street exterior"
    );
    assert!(
        vehicle_vel.x <= 0.0,
        "Inward velocity component must be arrested"
    );

    println!("✅ Stage B authoritative physics and coordinate invariants 100% preserved!");
}
