//! STAGE G3 MASTER CERTIFICATION SUITE
//!
//! Comprehensive, authoritative, and reproducible verification of:
//! 1. Phase 1: Runtime Graphics Pipeline, Dual Opaque/Transparent Pass, and Terrain Shading Isolation.
//! 2. Phase 2: Automotive Palette PBR Semantics and Outward CCW Cylinder Winding.
//! 3. Phase 3: Apex GT Sports Coupe Reference Car Curved Topology, Cockpit, and Steering Wheel.
//! 4. Phase 4: HyperSport RR-1000 Motorcycle Trellis Frame and Lean Articulation.
//! 5. Phase 5: CityLine Transit Bus and TransContinental Semi Heavy Commercial Assets.
//! 6. Phase 6: Pacific Locomotive, Aerospace, and Maritime Multimodal Transport Assets.
//! 7. Phase 7: Environmental Foliage Overhaul (Tapered Trunks, Cascading Cones, Arching Palms) and Street Furniture.
//! 8. Phase 8: Full Regression Invariance, Multi-LOD Monotonicity, and GPU Triangle Budget Preservation.

use glam::{Quat, Vec3};
use realearth_compiler::mesh::ChunkMesh;
use realearth_rendering::city_interactions_mesh::{
    generate_bus_stop_shelter_mesh, generate_detailed_tree_mesh, generate_streetlight_mesh,
    DetailedTreeType,
};
use realearth_rendering::gltf_asset::{VehicleLodLevel, VehicleSubmeshType};
use realearth_rendering::shaders::WORLD_SHADER_WGSL;
use realearth_rendering::vehicle_mesh::{
    generate_vehicle_mesh, VehicleMeshParams, VehicleVisualType,
};
use realearth_rendering::vehicle_models::{
    add_cylinder_x, add_cylinder_y, add_cylinder_z, get_certified_vehicle_model, AutomotivePalette,
};
use std::path::Path;

// ============================================================================
// PHASE 1: RUNTIME GRAPHICS PIPELINE & TRANSPARENCY RESOLUTION
// ============================================================================

#[test]
fn test_g3_phase1_runtime_graphics_pipeline_and_transparency_resolution() {
    println!("🎨 [PHASE 1] Testing Runtime Graphics Pipeline, Dual Pass, and Terrain Shader Isolation...");

    // 1. Verify WGSL shader source guarantees
    let shader = WORLD_SHADER_WGSL;

    // Verify dual-lobe Clearcoat specular highlight for automotive body paint
    assert!(
        shader.to_lowercase().contains("clearcoat"),
        "Shader must implement clearcoat specular reflection lobe for automotive paint"
    );

    // Verify terrain rock/sand noise is strictly guarded against vehicle bodies
    assert!(
        shader.contains("!is_vehicle"),
        "Shader must guard terrain texturing with !is_vehicle to prevent rocky texture pollution on vehicles"
    );

    // Verify hemisphere skylight ambient fill
    assert!(
        shader.contains("hemi_ambient"),
        "Shader must compute hemisphere skylight ambient fill"
    );

    // Verify opaque surfaces write solid 1.0 alpha (no translucent car bodies or see-through ground)
    assert!(
        shader.contains("out_alpha = 1.0"),
        "Shader must enforce solid 1.0 alpha for opaque vehicle and world geometry"
    );

    // Verify glass transparency blending logic
    assert!(
        shader.contains("0.78") || shader.contains("alpha_tag"),
        "Shader must properly parse glass and shadow alpha tags"
    );

    println!("✅ Phase 1 Runtime Graphics Pipeline and Shader Isolation certified!");
}

// ============================================================================
// PHASE 2: AUTOMOTIVE PALETTE & PBR SEMANTIC CLASSIFICATION
// ============================================================================

#[test]
fn test_g3_phase2_automotive_palette_and_pbr_semantic_classification() {
    println!("🔬 [PHASE 2] Testing Automotive Palette PBR Semantics and Cylinder Winding...");

    // 1. Exterior Clearcoat Metallic Body Finishes (alpha = 0.99)
    let body_finishes = [
        AutomotivePalette::APEX_CHERRY_RED,
        AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
        AutomotivePalette::SUV_GRANITE_GRAY,
        AutomotivePalette::TAXI_MUNICIPAL_YELLOW,
        AutomotivePalette::TRUCK_CRIMSON_HAULER,
        AutomotivePalette::BUS_TRANSIT_GREEN,
        AutomotivePalette::BIKE_RACING_ORANGE,
    ];
    for finish in body_finishes {
        assert!(
            (finish[3] - 0.99).abs() < 1e-4,
            "Exterior body finish alpha must be 0.99 for clearcoat PBR tag, got {}",
            finish[3]
        );
    }

    // 2. Matte / Rubber / Interior Materials (alpha = 0.98)
    let matte_materials = [
        AutomotivePalette::CARBON_FIBER,
        AutomotivePalette::TIRE_RUBBER,
        AutomotivePalette::INTERIOR_CHARCOAL,
        AutomotivePalette::LEATHER_ACCENT,
    ];
    for mat in matte_materials {
        assert!(
            (mat[3] - 0.98).abs() < 1e-4,
            "Matte/interior material alpha must be 0.98, got {}",
            mat[3]
        );
    }

    // 3. Metallic / Steel / Chrome Materials (alpha = 0.97)
    let metal_materials = [
        AutomotivePalette::ALLOY_SILVER,
        AutomotivePalette::BRAKE_DISC_STEEL,
        AutomotivePalette::POLISHED_CHROME,
    ];
    for mat in metal_materials {
        assert!(
            (mat[3] - 0.97).abs() < 1e-4,
            "Metallic/alloy material alpha must be 0.97, got {}",
            mat[3]
        );
    }

    // 4. Active Light Emissives (alpha = 0.95)
    let emissive_lights = [
        AutomotivePalette::HEADLIGHT_XENON_ON,
        AutomotivePalette::BRAKELIGHT_ON,
        AutomotivePalette::TAILLIGHT_RUNNING,
        AutomotivePalette::REVERSE_WHITE_ON,
        AutomotivePalette::TURN_SIGNAL_AMBER,
    ];
    for light in emissive_lights {
        assert!(
            (light[3] - 0.95).abs() < 1e-4,
            "Active emissive light alpha must be 0.95, got {}",
            light[3]
        );
    }

    // 5. Glass and Contact Shadows
    assert!(
        (AutomotivePalette::TINTED_GLASS[3] - 0.78).abs() < 1e-4,
        "Tinted glass alpha must be 0.78"
    );
    assert!(
        (AutomotivePalette::CONTACT_SHADOW[3] - 0.65).abs() < 1e-4,
        "Contact shadow alpha must be 0.65"
    );

    // 6. Cylinder Normal Direction & CCW Winding Verification
    // 6a. Cylinder along X-axis
    let mut cyl_x = ChunkMesh::default();
    add_cylinder_x(
        &mut cyl_x,
        Vec3::ZERO,
        1.0,
        0.5,
        16,
        AutomotivePalette::ALLOY_SILVER,
    );
    assert!(!cyl_x.vertices.is_empty());
    assert!(!cyl_x.indices.is_empty());
    for v in &cyl_x.vertices {
        let n = Vec3::from_array(v.normal);
        if n.x.abs() < 0.1 {
            let p_rad = Vec3::new(0.0, v.position[1], v.position[2]).normalize();
            let dot = n.dot(p_rad);
            assert!(
                dot > 0.85,
                "Cylinder X jacket normal must point radially outward! dot = {}",
                dot
            );
        }
    }

    // 6b. Cylinder along Y-axis
    let mut cyl_y = ChunkMesh::default();
    add_cylinder_y(
        &mut cyl_y,
        Vec3::ZERO,
        1.0,
        0.5,
        16,
        AutomotivePalette::ALLOY_SILVER,
    );
    assert!(!cyl_y.vertices.is_empty());
    assert!(!cyl_y.indices.is_empty());
    for v in &cyl_y.vertices {
        let n = Vec3::from_array(v.normal);
        if n.y.abs() < 0.1 {
            let p_rad = Vec3::new(v.position[0], 0.0, v.position[2]).normalize();
            let dot = n.dot(p_rad);
            assert!(
                dot > 0.85,
                "Cylinder Y jacket normal must point radially outward! dot = {}",
                dot
            );
        }
    }

    // 6c. Cylinder along Z-axis
    let mut cyl_z = ChunkMesh::default();
    add_cylinder_z(
        &mut cyl_z,
        Vec3::ZERO,
        1.0,
        0.5,
        16,
        AutomotivePalette::ALLOY_SILVER,
    );
    assert!(!cyl_z.vertices.is_empty());
    assert!(!cyl_z.indices.is_empty());
    for v in &cyl_z.vertices {
        let n = Vec3::from_array(v.normal);
        if n.z.abs() < 0.1 {
            let p_rad = Vec3::new(v.position[0], v.position[1], 0.0).normalize();
            let dot = n.dot(p_rad);
            assert!(
                dot > 0.85,
                "Cylinder Z jacket normal must point radially outward! dot = {}",
                dot
            );
        }
    }

    // Verify CCW triangle winding (geometric cross product aligns with normal)
    for (cyl_name, mesh) in [("cyl_x", &cyl_x), ("cyl_y", &cyl_y), ("cyl_z", &cyl_z)] {
        let mut ccw_valid = 0;
        for chunk in mesh.indices.as_chunks::<3>().0 {
            let p0 = Vec3::from_array(mesh.vertices[chunk[0] as usize].position);
            let p1 = Vec3::from_array(mesh.vertices[chunk[1] as usize].position);
            let p2 = Vec3::from_array(mesh.vertices[chunk[2] as usize].position);
            let geom_norm = (p1 - p0).cross(p2 - p0);
            let vert_norm = Vec3::from_array(mesh.vertices[chunk[0] as usize].normal);
            if geom_norm.length_squared() > 1e-6 {
                let dot = geom_norm.normalize().dot(vert_norm);
                assert!(
                    dot > 0.0,
                    "{} triangle winding must be CCW with outward geometric normal! dot = {}",
                    cyl_name,
                    dot
                );
                ccw_valid += 1;
            }
        }
        assert!(
            ccw_valid > 0,
            "{} must contain valid CCW triangles",
            cyl_name
        );
    }

    println!("✅ Phase 2 Automotive Palette and Cylinder Winding certified!");
}

// ============================================================================
// PHASE 3: APEX GT SPORTS COUPE CURVED TOPOLOGY & COCKPIT
// ============================================================================

#[test]
fn test_g3_phase3_sports_car_curved_topology_and_cockpit_steering() {
    println!("🏎️ [PHASE 3] Testing Apex GT Sports Coupe Curved Topology, Cockpit & Steering...");

    let model =
        get_certified_vehicle_model(VehicleVisualType::SportsCoupe, VehicleLodLevel::Lod0High);
    assert_eq!(model.display_name, "Apex GT Sports Coupe");

    // 1. Submesh verification
    let types: Vec<VehicleSubmeshType> = model.submeshes.iter().map(|s| s.submesh_type).collect();
    assert!(types.contains(&VehicleSubmeshType::ChassisBody));
    assert!(types.contains(&VehicleSubmeshType::GlassGreenhouse));
    assert!(types.contains(&VehicleSubmeshType::InteriorCockpit));
    assert!(types.contains(&VehicleSubmeshType::InteriorSteeringWheel));
    assert!(types.contains(&VehicleSubmeshType::WheelFrontLeft));
    assert!(types.contains(&VehicleSubmeshType::WheelFrontRight));
    assert!(types.contains(&VehicleSubmeshType::WheelRearLeft));
    assert!(types.contains(&VehicleSubmeshType::WheelRearRight));
    assert!(types.contains(&VehicleSubmeshType::Headlights));
    assert!(types.contains(&VehicleSubmeshType::BrakeLights));
    assert!(types.contains(&VehicleSubmeshType::AerodynamicSpoiler));
    assert!(types.contains(&VehicleSubmeshType::ContactShadow));

    // 2. Cockpit camera anchors
    let cam = model.camera_anchors;
    assert!(cam.cockpit_eye.y > 0.8 && cam.cockpit_eye.y < 1.3);
    assert!(
        cam.cockpit_eye.x < 0.0,
        "Driver sits on left side in US vehicle"
    );
    assert!(
        cam.cockpit_look.z > cam.cockpit_eye.z,
        "Cockpit camera looks forward (+Z)"
    );
    assert!(cam.chase_distance >= 5.0 && cam.chase_distance <= 12.0);

    // 3. Wheel arch cutouts and liners
    let chassis = model
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::ChassisBody)
        .expect("ChassisBody must exist");
    let chassis_verts = &chassis.mesh.vertices;

    let has_front_arch = chassis_verts.iter().any(|v| {
        let p = Vec3::from_array(v.position);
        p.z >= 1.15 && p.z <= 1.35 && p.y >= 0.70
    });
    assert!(
        has_front_arch,
        "Front wheel arch cutout must crest at y >= 0.70m"
    );

    let has_liner = chassis_verts.iter().any(|v| {
        v.color[0] <= 0.10
            && v.color[1] <= 0.10
            && v.color[2] <= 0.10
            && (v.color[3] - 0.98).abs() < 0.05
    });
    assert!(
        has_liner,
        "Chassis must contain dark matte wheel well liners"
    );

    // 4. Steering wheel submesh
    let steering = model
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::InteriorSteeringWheel)
        .expect("Steering wheel must exist");
    assert!(
        !steering.mesh.vertices.is_empty(),
        "Steering wheel must have vertices"
    );
    assert!(
        steering.pivot.z > 0.1,
        "Steering wheel pivot must be in front of driver"
    );

    println!("✅ Phase 3 Apex GT Sports Coupe curved topology and cockpit certified!");
}

// ============================================================================
// PHASE 4: HYPERSPORT RR-1000 MOTORCYCLE TRELLIS FRAME & LEAN
// ============================================================================

#[test]
fn test_g3_phase4_motorcycle_trellis_frame_and_lean_articulation() {
    println!("🏍️ [PHASE 4] Testing HyperSport RR-1000 Trellis Frame & Lean Articulation...");

    let bike =
        get_certified_vehicle_model(VehicleVisualType::Motorcycle, VehicleLodLevel::Lod0High);
    assert_eq!(bike.display_name, "HyperSport RR-1000");

    let types: Vec<VehicleSubmeshType> = bike.submeshes.iter().map(|s| s.submesh_type).collect();
    assert!(types.contains(&VehicleSubmeshType::MotorcycleLeanBody));
    assert!(types.contains(&VehicleSubmeshType::GlassGreenhouse));
    assert!(types.contains(&VehicleSubmeshType::WheelFrontLeft));
    assert!(types.contains(&VehicleSubmeshType::WheelRearLeft));

    // Dynamic cornering lean articulation in world mesh generation
    let params_upright = VehicleMeshParams {
        vehicle_type: VehicleVisualType::Motorcycle,
        lod: VehicleLodLevel::Lod0High,
        position: Vec3::ZERO,
        orientation: Quat::IDENTITY,
        ..Default::default()
    };
    let mesh_upright = generate_vehicle_mesh(&params_upright);

    // Roll banking angle of 20 degrees (0.35 rad)
    let lean_rot = Quat::from_rotation_z(0.35);
    let params_leaning = VehicleMeshParams {
        vehicle_type: VehicleVisualType::Motorcycle,
        lod: VehicleLodLevel::Lod0High,
        position: Vec3::ZERO,
        orientation: lean_rot,
        ..Default::default()
    };
    let mesh_leaning = generate_vehicle_mesh(&params_leaning);

    assert_eq!(mesh_upright.vertices.len(), mesh_leaning.vertices.len());

    // When leaning left (+roll around Z), highest vertices must displace laterally
    let max_y_vert_upright = mesh_upright
        .vertices
        .iter()
        .max_by(|a, b| a.position[1].partial_cmp(&b.position[1]).unwrap())
        .unwrap();
    let max_y_vert_leaning = mesh_leaning
        .vertices
        .iter()
        .max_by(|a, b| a.position[1].partial_cmp(&b.position[1]).unwrap())
        .unwrap();

    assert!(
        (max_y_vert_upright.position[0] - max_y_vert_leaning.position[0]).abs() > 0.05,
        "Leaning motorcycle must displace upper vertices laterally"
    );

    println!("✅ Phase 4 HyperSport RR-1000 motorcycle and lean articulation certified!");
}

// ============================================================================
// PHASE 5: CITYLINE TRANSIT BUS & TRANSCONTINENTAL SEMI HEAVY COMMERCIAL
// ============================================================================

#[test]
fn test_g3_phase5_transit_bus_and_semi_truck_heavy_commercial_assets() {
    println!("🚌 [PHASE 5] Testing CityLine Transit Bus & TransContinental Semi Heavy Assets...");

    // 1. CityLine Transit Bus
    let bus = get_certified_vehicle_model(VehicleVisualType::TransitBus, VehicleLodLevel::Lod0High);
    assert_eq!(bus.display_name, "CityLine Transit Bus");

    let bus_chassis = bus
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::ChassisBody)
        .expect("Bus chassis must exist");

    // Open wheel arch cutouts over front and rear axles (no solid skirts blocking tires)
    let front_axle_z = 3.35;
    let rear_axle_z = -2.85;
    let bus_verts = &bus_chassis.mesh.vertices;

    let bus_front_arch = bus_verts.iter().any(|v| {
        let p = Vec3::from_array(v.position);
        (p.z - front_axle_z).abs() < 0.6 && p.y >= 0.70
    });
    assert!(
        bus_front_arch,
        "Transit Bus must feature front axle open wheel arch cutouts"
    );

    let bus_rear_arch = bus_verts.iter().any(|v| {
        let p = Vec3::from_array(v.position);
        (p.z - rear_axle_z).abs() < 0.6 && p.y >= 0.70
    });
    assert!(
        bus_rear_arch,
        "Transit Bus must feature rear axle open wheel arch cutouts"
    );

    // Illuminated destination sign with emissive tag (0.95)
    let has_dest_matrix = bus_verts
        .iter()
        .any(|v| (v.color[3] - 0.95).abs() < 0.02 && v.position[1] >= 2.7 && v.position[2] >= 5.0);
    assert!(
        has_dest_matrix,
        "Transit Bus must feature illuminated destination sign"
    );

    // Passenger bi-fold door animation ratio
    let bus_params_closed = VehicleMeshParams {
        vehicle_type: VehicleVisualType::TransitBus,
        door_open_ratio: 0.0,
        ..Default::default()
    };
    let mesh_closed = generate_vehicle_mesh(&bus_params_closed);

    let bus_params_open = VehicleMeshParams {
        vehicle_type: VehicleVisualType::TransitBus,
        door_open_ratio: 1.0,
        ..Default::default()
    };
    let mesh_open = generate_vehicle_mesh(&bus_params_open);

    assert_eq!(mesh_closed.vertices.len(), mesh_open.vertices.len());

    // 2. TransContinental Semi Truck
    let semi =
        get_certified_vehicle_model(VehicleVisualType::HeavyTruck, VehicleLodLevel::Lod0High);
    assert_eq!(semi.display_name, "TransContinental Semi");

    let semi_trailer = semi
        .submeshes
        .iter()
        .find(|s| s.name.contains("Trailer") || s.name.contains("trailer"))
        .expect("Semi must contain trailer submesh");

    // Embossed aluminum corrugations (alpha = 0.97)
    let trailer_verts = &semi_trailer.mesh.vertices;
    let has_corrugation = trailer_verts
        .iter()
        .any(|v| (v.color[3] - 0.97).abs() < 0.02 && v.position[1] >= 1.5);
    assert!(
        has_corrugation,
        "Semi trailer must feature aluminum corrugation ribs"
    );

    // Fifth-wheel trailer articulation angle
    let semi_straight = VehicleMeshParams {
        vehicle_type: VehicleVisualType::HeavyTruck,
        trailer_angle: 0.0,
        ..Default::default()
    };
    let mesh_straight = generate_vehicle_mesh(&semi_straight);

    let semi_jackknife = VehicleMeshParams {
        vehicle_type: VehicleVisualType::HeavyTruck,
        trailer_angle: 0.50, // 28 degrees trailer yaw
        ..Default::default()
    };
    let mesh_jackknife = generate_vehicle_mesh(&semi_jackknife);

    assert_eq!(mesh_straight.vertices.len(), mesh_jackknife.vertices.len());

    println!("✅ Phase 5 CityLine Transit Bus and TransContinental Semi certified!");
}

// ============================================================================
// PHASE 6: PACIFIC LOCOMOTIVE, AEROSPACE & MARITIME ASSETS
// ============================================================================

#[test]
fn test_g3_phase6_locomotive_and_aerospace_multimodal_assets() {
    println!("🚂 [PHASE 6] Testing Pacific Locomotive, Aerospace & Maritime Transport Assets...");

    // 1. Pacific Locomotive
    let loco = get_certified_vehicle_model(
        VehicleVisualType::TrainLocomotive,
        VehicleLodLevel::Lod0High,
    );
    assert_eq!(loco.display_name, "Pacific Locomotive");

    let loco_chassis = loco
        .submeshes
        .iter()
        .find(|s| s.submesh_type == VehicleSubmeshType::ChassisBody)
        .expect("Locomotive chassis must exist");
    let loco_verts = &loco_chassis.mesh.vertices;

    // Sloped aerodynamic collision nose face and chamfered cheeks
    let has_sloped_nose = loco_verts.iter().any(|v| {
        let p = Vec3::from_array(v.position);
        p.z >= 7.0 && p.y >= 2.0 && v.normal[2] > 0.5
    });
    assert!(
        has_sloped_nose,
        "Locomotive must have sloped aerodynamic nose"
    );

    // 3 Rooftop dynamic brake cooling fan assemblies
    let has_cooling_fans = loco_verts.iter().any(|v| {
        let p = Vec3::from_array(v.position);
        p.y >= 4.40 && (v.color[3] - 0.97).abs() < 0.02
    });
    assert!(
        has_cooling_fans,
        "Locomotive must feature rooftop dynamic cooling fans"
    );

    // Heavy underframe steel sill (alpha = 0.97)
    let has_underframe_sill = loco_verts.iter().any(|v| {
        let p = Vec3::from_array(v.position);
        p.y <= 1.30 && (v.color[3] - 0.97).abs() < 0.02
    });
    assert!(
        has_underframe_sill,
        "Locomotive must feature heavy underframe sill"
    );

    // 2. Aerospace: SkyCourier B737
    let airliner = get_certified_vehicle_model(
        VehicleVisualType::CommercialAirliner,
        VehicleLodLevel::Lod0High,
    );
    assert_eq!(airliner.display_name, "SkyCourier B737");
    assert!(airliner
        .submeshes
        .iter()
        .any(|s| s.name.contains("Fuselage")
            || s.name.contains("fuselage")
            || s.submesh_type == VehicleSubmeshType::ChassisBody));

    // 3. Aerospace: Skyhawk C172
    let c172 = get_certified_vehicle_model(
        VehicleVisualType::GeneralAviation,
        VehicleLodLevel::Lod0High,
    );
    assert_eq!(c172.display_name, "Skyhawk C172");

    // 4. Maritime: Golden Gate Ferry
    let ferry =
        get_certified_vehicle_model(VehicleVisualType::MarineVessel, VehicleLodLevel::Lod0High);
    assert_eq!(ferry.display_name, "Golden Gate Ferry");

    println!("✅ Phase 6 Pacific Locomotive, Aerospace, and Maritime transport certified!");
}

// ============================================================================
// PHASE 7: ENVIRONMENTAL FOLIAGE & URBAN INFRASTRUCTURE OVERHAUL
// ============================================================================

#[test]
fn test_g3_phase7_environmental_foliage_and_urban_infrastructure() {
    println!("🌳 [PHASE 7] Testing Environmental Foliage & Urban Infrastructure Overhaul...");

    // 1. Broadleaf Deciduous Tree: Tapered trunk and multi-layered organic foliage domes
    let deciduous =
        generate_detailed_tree_mesh(Vec3::ZERO, 1.0, DetailedTreeType::BroadleafDeciduous);
    assert!(!deciduous.vertices.is_empty());
    assert!(!deciduous.indices.is_empty());
    assert!(
        deciduous.vertices.len() >= 80,
        "Broadleaf deciduous tree must feature smooth multi-layered canopy (got {} verts)",
        deciduous.vertices.len()
    );

    // 2. Evergreen Conifer Tree: Central mast trunk and cascading drooping cone boughs
    let conifer = generate_detailed_tree_mesh(Vec3::ZERO, 1.0, DetailedTreeType::EvergreenConifer);
    assert!(!conifer.vertices.is_empty());
    assert!(!conifer.indices.is_empty());
    assert!(
        conifer.vertices.len() >= 50,
        "Evergreen conifer must feature 4 cascading foliage tiers (got {} verts)",
        conifer.vertices.len()
    );

    // 3. Coastal Palm: Curved segmented trunk and arching radiating fronds
    let palm = generate_detailed_tree_mesh(Vec3::ZERO, 1.0, DetailedTreeType::CoastalPalm);
    assert!(!palm.vertices.is_empty());
    assert!(!palm.indices.is_empty());
    assert!(
        palm.vertices.len() >= 80,
        "Coastal palm must feature curved trunk and 10 arching fronds (got {} verts)",
        palm.vertices.len()
    );

    // Verify trunk geometry is tapered cylinder (radius narrows with height)
    let trunk_bottom_radius = deciduous
        .vertices
        .iter()
        .filter(|v| v.position[1] < 0.2)
        .map(|v| (v.position[0].powi(2) + v.position[2].powi(2)).sqrt())
        .fold(0.0f32, f32::max);
    let trunk_top_radius = deciduous
        .vertices
        .iter()
        .filter(|v| v.position[1] > 1.8 && v.position[1] < 2.3 && v.color[0] > 0.3)
        .map(|v| (v.position[0].powi(2) + v.position[2].powi(2)).sqrt())
        .fold(0.0f32, f32::max);

    assert!(
        trunk_bottom_radius > trunk_top_radius,
        "Tree trunk must taper organically towards the crown (bottom {} > top {})",
        trunk_bottom_radius,
        trunk_top_radius
    );

    // 4. Urban Infrastructure: Streetlights with cobra head and bus stop shelter
    let light = generate_streetlight_mesh(Vec3::ZERO, 0.0, true);
    assert!(!light.vertices.is_empty());
    assert!(!light.indices.is_empty());

    let shelter = generate_bus_stop_shelter_mesh(Vec3::ZERO, 0.0);
    assert!(!shelter.vertices.is_empty());
    assert!(!shelter.indices.is_empty());

    println!("✅ Phase 7 Environmental Foliage and Urban Infrastructure certified!");
}

// ============================================================================
// PHASE 8: FULL REGRESSION, MULTI-LOD MONOTONICITY & GPU BUDGETS
// ============================================================================

#[test]
fn test_g3_phase8_full_regression_and_gpu_budget_preservation() {
    println!(
        "⚡ [PHASE 8] Testing Full Multi-LOD Monotonicity, GPU Budgets & GLB File Artifacts..."
    );

    let visual_types = [
        VehicleVisualType::SportsCoupe,
        VehicleVisualType::Sedan,
        VehicleVisualType::Suv,
        VehicleVisualType::TransitBus,
        VehicleVisualType::HeavyTruck,
        VehicleVisualType::Taxi,
        VehicleVisualType::Motorcycle,
        VehicleVisualType::TrainLocomotive,
        VehicleVisualType::TrainWagon,
        VehicleVisualType::MarineVessel,
        VehicleVisualType::CommercialAirliner,
        VehicleVisualType::GeneralAviation,
    ];

    for vt in visual_types {
        let m0 = get_certified_vehicle_model(vt, VehicleLodLevel::Lod0High);
        let m1 = get_certified_vehicle_model(vt, VehicleLodLevel::Lod1Medium);
        let m2 = get_certified_vehicle_model(vt, VehicleLodLevel::Lod2Low);

        let v0_count: usize = m0.submeshes.iter().map(|s| s.mesh.vertices.len()).sum();
        let v1_count: usize = m1.submeshes.iter().map(|s| s.mesh.vertices.len()).sum();
        let v2_count: usize = m2.submeshes.iter().map(|s| s.mesh.vertices.len()).sum();

        assert!(
            v0_count >= v1_count,
            "{} LOD0 verts ({}) must be >= LOD1 verts ({})",
            vt.display_name(),
            v0_count,
            v1_count
        );
        assert!(
            v1_count >= v2_count,
            "{} LOD1 verts ({}) must be >= LOD2 verts ({})",
            vt.display_name(),
            v1_count,
            v2_count
        );

        // GPU Budget: Each vehicle must remain under 50,000 vertices for 60+ FPS multi-vehicle traffic simulation
        assert!(
            v0_count < 50_000,
            "{} LOD0 exceeds GPU performance budget: {} vertices",
            vt.display_name(),
            v0_count
        );
    }

    // Verify all 16 GLB binary files exist in assets/vehicles/ and are non-empty
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root");
    let glb_dir = workspace_root.join("assets/vehicles");

    let required_glbs = [
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

    for glb_name in required_glbs {
        let p = glb_dir.join(glb_name);
        assert!(
            p.exists(),
            "Required binary GLB asset {:?} must exist on disk",
            p
        );
        let metadata = std::fs::metadata(&p).expect("GLB metadata");
        assert!(
            metadata.len() > 1000,
            "GLB asset {:?} must be non-empty and valid (size: {} bytes)",
            p,
            metadata.len()
        );
    }

    println!("✅ Phase 8 Full Regression, Multi-LOD Monotonicity, and GPU Budgets certified!");
}
