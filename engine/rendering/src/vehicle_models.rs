//! High-precision procedural 3D automotive vehicle modeling and glTF 2.0 / GLB compilation.
//!
//! Provides certified, aerodynamically sculpted geometries with curved body panels,
//! greenhouse glass, articulated wheel assemblies (rim, tire, brake rotor, caliper),
//! cockpit interior, dynamic lighting, and ground contact shadow planes.

use crate::gltf_asset::{
    GltfAssetBuilder, GltfAssetError, LoadedVehicleModel, VehicleAlphaMode, VehicleCameraAnchors,
    VehicleLodLevel, VehiclePbrMaterial, VehicleSubmesh, VehicleSubmeshType,
};
use crate::vehicle_mesh::VehicleVisualType;
use glam::{Affine3A, Vec3};
use realearth_compiler::mesh::{ChunkMesh, Vertex};
use std::f32::consts::PI;
use std::path::Path;
use std::sync::OnceLock;

// =============================================================================
// Automotive Modeling Primitives & Helpers
// =============================================================================

/// Material color presets matching premium automotive finishes.
pub struct AutomotivePalette;

impl AutomotivePalette {
    // Exterior Body Finishes
    pub const APEX_CHERRY_RED: [f32; 4] = [0.92, 0.10, 0.12, 1.0];
    pub const SEDAN_SAPPHIRE_BLUE: [f32; 4] = [0.12, 0.30, 0.82, 1.0];
    pub const SUV_GRANITE_GRAY: [f32; 4] = [0.22, 0.24, 0.28, 1.0];
    pub const TAXI_MUNICIPAL_YELLOW: [f32; 4] = [0.96, 0.80, 0.10, 1.0];
    pub const TRUCK_CRIMSON_HAULER: [f32; 4] = [0.85, 0.20, 0.15, 1.0];
    pub const BUS_TRANSIT_GREEN: [f32; 4] = [0.10, 0.65, 0.40, 1.0];
    pub const BIKE_RACING_ORANGE: [f32; 4] = [0.95, 0.40, 0.08, 1.0];

    // Automotive Materials
    pub const TINTED_GLASS: [f32; 4] = [0.12, 0.18, 0.25, 0.85];
    pub const CARBON_FIBER: [f32; 4] = [0.08, 0.08, 0.09, 1.0];
    pub const TIRE_RUBBER: [f32; 4] = [0.10, 0.10, 0.10, 1.0];
    pub const ALLOY_SILVER: [f32; 4] = [0.75, 0.78, 0.82, 1.0];
    pub const BRAKE_DISC_STEEL: [f32; 4] = [0.65, 0.66, 0.68, 1.0];
    pub const BREMBO_RED_CALIPER: [f32; 4] = [0.95, 0.05, 0.05, 1.0];
    pub const POLISHED_CHROME: [f32; 4] = [0.92, 0.94, 0.96, 1.0];
    pub const INTERIOR_CHARCOAL: [f32; 4] = [0.14, 0.15, 0.17, 1.0];
    pub const LEATHER_ACCENT: [f32; 4] = [0.22, 0.18, 0.15, 1.0];
    pub const CONTACT_SHADOW: [f32; 4] = [0.02, 0.02, 0.03, 0.70];

    // Lighting Emissives
    pub const HEADLIGHT_XENON_ON: [f32; 4] = [1.0, 0.98, 0.90, 1.0];
    pub const HEADLIGHT_OFF: [f32; 4] = [0.35, 0.35, 0.38, 1.0];
    pub const BRAKELIGHT_ON: [f32; 4] = [1.0, 0.05, 0.05, 1.0];
    pub const TAILLIGHT_RUNNING: [f32; 4] = [0.55, 0.04, 0.04, 1.0];
    pub const REVERSE_WHITE_ON: [f32; 4] = [0.98, 0.98, 1.0, 1.0];
    pub const REVERSE_OFF: [f32; 4] = [0.30, 0.30, 0.32, 1.0];
    pub const TURN_SIGNAL_AMBER: [f32; 4] = [1.0, 0.65, 0.05, 1.0];
}

/// Helper to add a flat quad with custom normal, UVs, and color.
pub fn add_quad(
    mesh: &mut ChunkMesh,
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    normal: Vec3,
    color: [f32; 4],
) {
    let base_idx = mesh.vertices.len() as u32;
    let n = normal.to_array();

    mesh.vertices
        .push(Vertex::new(p0.to_array(), n, [0.0, 0.0], color));
    mesh.vertices
        .push(Vertex::new(p1.to_array(), n, [1.0, 0.0], color));
    mesh.vertices
        .push(Vertex::new(p2.to_array(), n, [1.0, 1.0], color));
    mesh.vertices
        .push(Vertex::new(p3.to_array(), n, [0.0, 1.0], color));

    mesh.indices.extend_from_slice(&[
        base_idx,
        base_idx + 1,
        base_idx + 2,
        base_idx,
        base_idx + 2,
        base_idx + 3,
    ]);
}

/// Helper to add a triangle.
pub fn add_tri(mesh: &mut ChunkMesh, p0: Vec3, p1: Vec3, p2: Vec3, normal: Vec3, color: [f32; 4]) {
    let base_idx = mesh.vertices.len() as u32;
    let n = normal.to_array();

    mesh.vertices
        .push(Vertex::new(p0.to_array(), n, [0.0, 0.0], color));
    mesh.vertices
        .push(Vertex::new(p1.to_array(), n, [1.0, 0.0], color));
    mesh.vertices
        .push(Vertex::new(p2.to_array(), n, [0.5, 1.0], color));

    mesh.indices
        .extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
}

/// Adds a curved cylindrical extrusion along the X-axis (axle orientation).
pub fn add_cylinder_x(
    mesh: &mut ChunkMesh,
    center: Vec3,
    radius: f32,
    half_width: f32,
    segments: usize,
    color: [f32; 4],
) {
    let angle_step = 2.0 * PI / segments as f32;
    let x_left = center.x - half_width;
    let x_right = center.x + half_width;

    for i in 0..segments {
        let a0 = i as f32 * angle_step;
        let a1 = (i + 1) as f32 * angle_step;

        let y0 = center.y + radius * a0.cos();
        let z0 = center.z + radius * a0.sin();
        let y1 = center.y + radius * a1.cos();
        let z1 = center.z + radius * a1.sin();

        let n0 = Vec3::new(0.0, a0.cos(), a0.sin()).normalize();
        let n1 = Vec3::new(0.0, a1.cos(), a1.sin()).normalize();
        let face_n = (n0 + n1) * 0.5;

        // Outer surface quad
        add_quad(
            mesh,
            Vec3::new(x_left, y0, z0),
            Vec3::new(x_right, y0, z0),
            Vec3::new(x_right, y1, z1),
            Vec3::new(x_left, y1, z1),
            face_n,
            color,
        );

        // Left endcap face (-X)
        add_tri(
            mesh,
            Vec3::new(x_left, center.y, center.z),
            Vec3::new(x_left, y1, z1),
            Vec3::new(x_left, y0, z0),
            -Vec3::X,
            color,
        );

        // Right endcap face (+X)
        add_tri(
            mesh,
            Vec3::new(x_right, center.y, center.z),
            Vec3::new(x_right, y0, z0),
            Vec3::new(x_right, y1, z1),
            Vec3::X,
            color,
        );
    }
}

/// Builds an articulated, high-detail wheel assembly (treaded tire, alloy rim, disc rotor, caliper).
pub fn build_wheel_assembly(
    radius: f32,
    width: f32,
    is_left_side: bool,
    lod: VehicleLodLevel,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let segments = match lod {
        VehicleLodLevel::Lod0High => 24,
        VehicleLodLevel::Lod1Medium => 14,
        VehicleLodLevel::Lod2Low => 8,
    };

    let half_w = width * 0.5;
    let rim_radius = radius * 0.68;
    let rotor_radius = radius * 0.52;

    let center = Vec3::ZERO;
    let angle_step = 2.0 * PI / segments as f32;

    // 1. Treaded Rubber Tire with curved profile
    let x_inboard = if is_left_side { half_w } else { -half_w };
    let x_outboard = if is_left_side { -half_w } else { half_w };

    for i in 0..segments {
        let a0 = i as f32 * angle_step;
        let a1 = (i + 1) as f32 * angle_step;

        let cos0 = a0.cos();
        let sin0 = a0.sin();
        let cos1 = a1.cos();
        let sin1 = a1.sin();

        let y0 = center.y + radius * cos0;
        let z0 = center.z + radius * sin0;
        let y1 = center.y + radius * cos1;
        let z1 = center.z + radius * sin1;

        let rim_y0 = center.y + rim_radius * cos0;
        let rim_z0 = center.z + rim_radius * sin0;
        let rim_y1 = center.y + rim_radius * cos1;
        let rim_z1 = center.z + rim_radius * sin1;

        let norm0 = Vec3::new(0.0, cos0, sin0);
        let norm1 = Vec3::new(0.0, cos1, sin1);
        let avg_n = (norm0 + norm1).normalize_or_zero();

        // 1a. Outer tread face
        add_quad(
            &mut mesh,
            Vec3::new(x_inboard, y0, z0),
            Vec3::new(x_outboard, y0, z0),
            Vec3::new(x_outboard, y1, z1),
            Vec3::new(x_inboard, y1, z1),
            avg_n,
            AutomotivePalette::TIRE_RUBBER,
        );

        // 1b. Outboard sidewall
        let out_n = if is_left_side { -Vec3::X } else { Vec3::X };
        add_quad(
            &mut mesh,
            Vec3::new(x_outboard, rim_y0, rim_z0),
            Vec3::new(x_outboard, y0, z0),
            Vec3::new(x_outboard, y1, z1),
            Vec3::new(x_outboard, rim_y1, rim_z1),
            out_n,
            AutomotivePalette::TIRE_RUBBER,
        );

        // 1c. Inboard sidewall
        let in_n = if is_left_side { Vec3::X } else { -Vec3::X };
        add_quad(
            &mut mesh,
            Vec3::new(x_inboard, y0, z0),
            Vec3::new(x_inboard, rim_y0, rim_z0),
            Vec3::new(x_inboard, rim_y1, rim_z1),
            Vec3::new(x_inboard, y1, z1),
            in_n,
            AutomotivePalette::TIRE_RUBBER,
        );
    }

    // 2. Alloy Rim & Spokes (Outboard facing)
    let rim_x = x_outboard + (if is_left_side { 0.015 } else { -0.015 });
    let hub_radius = rim_radius * 0.25;

    // Central hub
    for i in 0..segments {
        let a0 = i as f32 * angle_step;
        let a1 = (i + 1) as f32 * angle_step;
        let p0 = Vec3::new(
            rim_x,
            center.y + hub_radius * a0.cos(),
            center.z + hub_radius * a0.sin(),
        );
        let p1 = Vec3::new(
            rim_x,
            center.y + hub_radius * a1.cos(),
            center.z + hub_radius * a1.sin(),
        );
        let pc = Vec3::new(rim_x, center.y, center.z);
        let n = if is_left_side { -Vec3::X } else { Vec3::X };
        add_tri(&mut mesh, pc, p0, p1, n, AutomotivePalette::ALLOY_SILVER);
    }

    // 5 Aerodynamic Twin-Spokes
    let spoke_count = 5;
    let spoke_angle_step = 2.0 * PI / spoke_count as f32;
    for i in 0..spoke_count {
        let base_ang = i as f32 * spoke_angle_step;
        let spoke_w = 0.08; // Angular half-width

        let a0 = base_ang - spoke_w;
        let a1 = base_ang + spoke_w;

        let hub_p0 = Vec3::new(
            rim_x,
            center.y + hub_radius * a0.cos(),
            center.z + hub_radius * a0.sin(),
        );
        let hub_p1 = Vec3::new(
            rim_x,
            center.y + hub_radius * a1.cos(),
            center.z + hub_radius * a1.sin(),
        );
        let rim_p0 = Vec3::new(
            rim_x,
            center.y + rim_radius * a0.cos(),
            center.z + rim_radius * a0.sin(),
        );
        let rim_p1 = Vec3::new(
            rim_x,
            center.y + rim_radius * a1.cos(),
            center.z + rim_radius * a1.sin(),
        );

        let n = if is_left_side { -Vec3::X } else { Vec3::X };
        add_quad(
            &mut mesh,
            hub_p0,
            rim_p0,
            rim_p1,
            hub_p1,
            n,
            AutomotivePalette::ALLOY_SILVER,
        );
    }

    // 3. Ventilated Brake Disc Rotor (visible through spokes)
    if lod != VehicleLodLevel::Lod2Low {
        let rotor_x = x_outboard + (if is_left_side { 0.035 } else { -0.035 });
        for i in 0..segments {
            let a0 = i as f32 * angle_step;
            let a1 = (i + 1) as f32 * angle_step;
            let p0 = Vec3::new(
                rotor_x,
                center.y + rotor_radius * a0.cos(),
                center.z + rotor_radius * a0.sin(),
            );
            let p1 = Vec3::new(
                rotor_x,
                center.y + rotor_radius * a1.cos(),
                center.z + rotor_radius * a1.sin(),
            );
            let pc = Vec3::new(rotor_x, center.y, center.z);
            let n = if is_left_side { -Vec3::X } else { Vec3::X };
            add_tri(
                &mut mesh,
                pc,
                p0,
                p1,
                n,
                AutomotivePalette::BRAKE_DISC_STEEL,
            );
        }

        // 4. Brembo-Style High-Performance Brake Caliper
        let caliper_x = rotor_x + (if is_left_side { -0.015 } else { 0.015 });
        let cal_y = center.y + rotor_radius * 0.75;
        let cal_z = center.z + rotor_radius * 0.35;
        let cal_half_w = 0.02;
        let cal_half_h = 0.055;
        let cal_half_l = 0.075;

        add_box_at(
            &mut mesh,
            Vec3::new(caliper_x, cal_y, cal_z),
            Vec3::new(cal_half_w, cal_half_h, cal_half_l),
            AutomotivePalette::BREMBO_RED_CALIPER,
        );
    }

    mesh
}

/// Helper to add an oriented box.
pub fn add_box_at(mesh: &mut ChunkMesh, center: Vec3, half_extents: Vec3, color: [f32; 4]) {
    let (hx, hy, hz) = (half_extents.x, half_extents.y, half_extents.z);
    let c0 = center + Vec3::new(-hx, -hy, hz);
    let c1 = center + Vec3::new(hx, -hy, hz);
    let c2 = center + Vec3::new(hx, hy, hz);
    let c3 = center + Vec3::new(-hx, hy, hz);
    let c4 = center + Vec3::new(-hx, -hy, -hz);
    let c5 = center + Vec3::new(hx, -hy, -hz);
    let c6 = center + Vec3::new(hx, hy, -hz);
    let c7 = center + Vec3::new(-hx, hy, -hz);

    add_quad(mesh, c0, c1, c2, c3, Vec3::Z, color); // Front
    add_quad(mesh, c5, c4, c7, c6, -Vec3::Z, color); // Rear
    add_quad(mesh, c3, c2, c6, c7, Vec3::Y, color); // Top
    add_quad(mesh, c4, c5, c1, c0, -Vec3::Y, color); // Bottom
    add_quad(mesh, c1, c5, c6, c2, Vec3::X, color); // Right
    add_quad(mesh, c4, c0, c3, c7, -Vec3::X, color); // Left
}

// =============================================================================
// Apex GT Sports Coupe — High-Fidelity 3D Vehicle Builder
// =============================================================================

/// Builds the certified Apex GT Sports Coupe with authentic curved body panels,
/// aerodynamic wing, greenhouse glass, detailed cockpit interior, and transformable wheels.
pub fn build_apex_gt_sports_coupe(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("apex_gt_sports_coupe", "Apex GT Sports Coupe")
        .with_physics(2.50, 1.65, 0.33, 1250.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 0.45, 0.0),
            chase_distance: 6.0,
            chase_height: 1.75,
            cockpit_eye: Vec3::new(-0.38, 0.88, -0.15),
            cockpit_look: Vec3::new(-0.38, 0.88, 2.5),
            hood_eye: Vec3::new(0.0, 0.78, 1.55),
            orbit_pivot: Vec3::new(0.0, 0.45, 0.0),
        });

    let half_w = 0.96; // Track width half + fender flares
    let half_l = 2.22; // 4.45m overall length
    let beltline_y = 0.65;
    let hood_y = 0.58;
    let nose_y = 0.32;
    let roof_y = 1.25;

    // -------------------------------------------------------------------------
    // 1. Aerodynamic Chassis & Curved Body Panels
    // -------------------------------------------------------------------------
    let mut chassis_mesh = ChunkMesh::default();

    // 1a. Low Front Splitter & Air Curtain (Carbon Fiber)
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-half_w, 0.18, 1.80),
        Vec3::new(half_w, 0.18, 1.80),
        Vec3::new(half_w * 0.92, 0.18, half_l),
        Vec3::new(-half_w * 0.92, 0.18, half_l),
        -Vec3::Y,
        AutomotivePalette::CARBON_FIBER,
    );
    // Splitter front lip
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-half_w * 0.92, 0.18, half_l),
        Vec3::new(half_w * 0.92, 0.18, half_l),
        Vec3::new(half_w * 0.90, 0.22, half_l),
        Vec3::new(-half_w * 0.90, 0.22, half_l),
        Vec3::Z,
        AutomotivePalette::CARBON_FIBER,
    );

    // 1b. Front Bumper Fascia & Radiator Intake Grille
    // Lower front intake opening
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.65, 0.22, half_l - 0.02),
        Vec3::new(0.65, 0.22, half_l - 0.02),
        Vec3::new(0.60, 0.42, half_l - 0.05),
        Vec3::new(-0.60, 0.42, half_l - 0.05),
        Vec3::Z,
        AutomotivePalette::CARBON_FIBER, // Honeycomb grille
    );

    // 1c. Sloped Front Nose Cone
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.80, 0.42, half_l - 0.08),
        Vec3::new(0.80, 0.42, half_l - 0.08),
        Vec3::new(0.75, nose_y + 0.25, half_l - 0.15),
        Vec3::new(-0.75, nose_y + 0.25, half_l - 0.15),
        Vec3::new(0.0, 0.35, 0.93).normalize(),
        AutomotivePalette::APEX_CHERRY_RED,
    );

    // 1d. Curved Sculpted Hood with Power Bulge & Air Extraction Recesses
    let hood_slices = match lod {
        VehicleLodLevel::Lod0High => 5,
        VehicleLodLevel::Lod1Medium => 3,
        VehicleLodLevel::Lod2Low => 2,
    };
    for s in 0..hood_slices {
        let t0 = s as f32 / hood_slices as f32;
        let t1 = (s + 1) as f32 / hood_slices as f32;

        let z0 = (half_l - 0.15) * (1.0 - t0) + 0.65 * t0;
        let z1 = (half_l - 0.15) * (1.0 - t1) + 0.65 * t1;

        let y0 = (nose_y + 0.25) * (1.0 - t0) + hood_y * t0 + (t0 * (1.0 - t0) * 0.08); // Parabolic crown
        let y1 = (nose_y + 0.25) * (1.0 - t1) + hood_y * t1 + (t1 * (1.0 - t1) * 0.08);

        let w0 = 0.75 * (1.0 - t0) + 0.88 * t0;
        let w1 = 0.75 * (1.0 - t1) + 0.88 * t1;

        let n = Vec3::new(0.0, z0 - z1, y1 - y0).normalize_or_zero();

        // Left hood quadrant
        add_quad(
            &mut chassis_mesh,
            Vec3::new(-w0, y0, z0),
            Vec3::new(0.0, y0 + 0.02, z0), // Center power ridge
            Vec3::new(0.0, y1 + 0.02, z1),
            Vec3::new(-w1, y1, z1),
            n,
            AutomotivePalette::APEX_CHERRY_RED,
        );
        // Right hood quadrant
        add_quad(
            &mut chassis_mesh,
            Vec3::new(0.0, y0 + 0.02, z0),
            Vec3::new(w0, y0, z0),
            Vec3::new(w1, y1, z1),
            Vec3::new(0.0, y1 + 0.02, z1),
            n,
            AutomotivePalette::APEX_CHERRY_RED,
        );
    }

    // 1e. Flared Front and Rear Fenders
    // Front-Left Fender
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-half_w, 0.32, 1.75),
        Vec3::new(-0.80, 0.58, 1.75),
        Vec3::new(-0.85, 0.65, 0.65),
        Vec3::new(-half_w, 0.35, 0.65),
        -Vec3::X,
        AutomotivePalette::APEX_CHERRY_RED,
    );
    // Front-Right Fender
    add_quad(
        &mut chassis_mesh,
        Vec3::new(0.80, 0.58, 1.75),
        Vec3::new(half_w, 0.32, 1.75),
        Vec3::new(half_w, 0.35, 0.65),
        Vec3::new(0.85, 0.65, 0.65),
        Vec3::X,
        AutomotivePalette::APEX_CHERRY_RED,
    );

    // 1f. Sculpted Door Sills and Coke-Bottle Waistline
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-half_w * 0.92, 0.22, 0.65),
        Vec3::new(-half_w * 0.88, 0.65, 0.65),
        Vec3::new(-half_w * 0.88, 0.68, -0.65),
        Vec3::new(-half_w * 0.92, 0.22, -0.65),
        -Vec3::X,
        AutomotivePalette::APEX_CHERRY_RED,
    );
    add_quad(
        &mut chassis_mesh,
        Vec3::new(half_w * 0.88, 0.65, 0.65),
        Vec3::new(half_w * 0.92, 0.22, 0.65),
        Vec3::new(half_w * 0.92, 0.22, -0.65),
        Vec3::new(half_w * 0.88, 0.68, -0.65),
        Vec3::X,
        AutomotivePalette::APEX_CHERRY_RED,
    );

    // 1g. Flared Rear Muscular Haunches
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-half_w * 0.92, 0.30, -0.65),
        Vec3::new(-half_w * 0.88, 0.68, -0.65),
        Vec3::new(-half_w, 0.72, -1.85),
        Vec3::new(-half_w, 0.30, -1.85),
        -Vec3::X,
        AutomotivePalette::APEX_CHERRY_RED,
    );
    add_quad(
        &mut chassis_mesh,
        Vec3::new(half_w * 0.88, 0.68, -0.65),
        Vec3::new(half_w * 0.92, 0.30, -0.65),
        Vec3::new(half_w, 0.30, -1.85),
        Vec3::new(half_w, 0.72, -1.85),
        Vec3::X,
        AutomotivePalette::APEX_CHERRY_RED,
    );

    // 1h. Aerodynamic Rear Diffuser with Airflow Strakes
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-half_w * 0.90, 0.20, -1.85),
        Vec3::new(half_w * 0.90, 0.20, -1.85),
        Vec3::new(half_w * 0.85, 0.32, -half_l),
        Vec3::new(-half_w * 0.85, 0.32, -half_l),
        Vec3::new(0.0, 0.5, -0.86).normalize(),
        AutomotivePalette::CARBON_FIBER,
    );
    // 4 Vertical Diffuser Strakes
    for s_x in [-0.45, -0.15, 0.15, 0.45] {
        add_quad(
            &mut chassis_mesh,
            Vec3::new(s_x, 0.16, -1.90),
            Vec3::new(s_x, 0.28, -1.90),
            Vec3::new(s_x, 0.34, -half_l),
            Vec3::new(s_x, 0.20, -half_l),
            Vec3::X,
            AutomotivePalette::CARBON_FIBER,
        );
    }

    // 1i. Rear Decklid & Ducktail Lip
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.78, 0.78, -1.65),
        Vec3::new(0.78, 0.78, -1.65),
        Vec3::new(0.75, 0.82, -half_l + 0.10),
        Vec3::new(-0.75, 0.82, -half_l + 0.10),
        Vec3::Y,
        AutomotivePalette::APEX_CHERRY_RED,
    );

    // 1j. Aerodynamic Side Mirrors (Mounted on Doors)
    for sign in [-1.0f32, 1.0f32] {
        let mir_x = sign * 0.88;
        let mir_stalk_x = sign * 0.82;
        let mir_y = beltline_y + 0.12;
        let mir_z = 0.55;

        // Stalk
        add_box_at(
            &mut chassis_mesh,
            Vec3::new((mir_x + mir_stalk_x) * 0.5, mir_y - 0.04, mir_z),
            Vec3::new(0.04, 0.015, 0.02),
            AutomotivePalette::CARBON_FIBER,
        );
        // Teardrop Housing
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(mir_x + sign * 0.06, mir_y, mir_z),
            Vec3::new(0.07, 0.045, 0.08),
            AutomotivePalette::APEX_CHERRY_RED,
        );
        // Reflective Mirror Glass Face (-Z facing)
        add_quad(
            &mut chassis_mesh,
            Vec3::new(mir_x + sign * 0.01, mir_y - 0.035, mir_z - 0.075),
            Vec3::new(mir_x + sign * 0.11, mir_y - 0.035, mir_z - 0.075),
            Vec3::new(mir_x + sign * 0.11, mir_y + 0.035, mir_z - 0.075),
            Vec3::new(mir_x + sign * 0.01, mir_y + 0.035, mir_z - 0.075),
            -Vec3::Z,
            AutomotivePalette::POLISHED_CHROME,
        );
    }

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Chassis".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::APEX_CHERRY_RED,
            metallic: 0.88,
            roughness: 0.16,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // -------------------------------------------------------------------------
    // 2. Greenhouse Glass (Windshield, Roof, Rear Fastback Glass, Side Windows)
    // -------------------------------------------------------------------------
    let mut glass_mesh = ChunkMesh::default();

    // 2a. Sloped Panoramic Windshield (58 deg rake)
    add_quad(
        &mut glass_mesh,
        Vec3::new(-0.76, hood_y + 0.02, 0.62),
        Vec3::new(0.76, hood_y + 0.02, 0.62),
        Vec3::new(0.62, roof_y, -0.05),
        Vec3::new(-0.62, roof_y, -0.05),
        Vec3::new(0.0, 0.62, 0.78).normalize(),
        AutomotivePalette::TINTED_GLASS,
    );

    // 2b. Contoured Roof Panel
    add_quad(
        &mut glass_mesh,
        Vec3::new(-0.62, roof_y, -0.05),
        Vec3::new(0.62, roof_y, -0.05),
        Vec3::new(0.60, roof_y - 0.02, -0.75),
        Vec3::new(-0.60, roof_y - 0.02, -0.75),
        Vec3::Y,
        AutomotivePalette::CARBON_FIBER, // Carbon fiber roof panel
    );

    // 2c. Sloping Fastback Rear Window
    add_quad(
        &mut glass_mesh,
        Vec3::new(-0.60, roof_y - 0.02, -0.75),
        Vec3::new(0.60, roof_y - 0.02, -0.75),
        Vec3::new(0.70, 0.78, -1.62),
        Vec3::new(-0.70, 0.78, -1.62),
        Vec3::new(0.0, 0.65, -0.76).normalize(),
        AutomotivePalette::TINTED_GLASS,
    );

    // 2d. Left Side Windows (Driver side)
    add_quad(
        &mut glass_mesh,
        Vec3::new(-0.76, beltline_y + 0.02, 0.58),
        Vec3::new(-0.62, roof_y - 0.02, -0.05),
        Vec3::new(-0.60, roof_y - 0.04, -0.75),
        Vec3::new(-0.72, beltline_y + 0.02, -0.75),
        -Vec3::X,
        AutomotivePalette::TINTED_GLASS,
    );
    // 2e. Right Side Windows (Passenger side)
    add_quad(
        &mut glass_mesh,
        Vec3::new(0.62, roof_y - 0.02, -0.05),
        Vec3::new(0.76, beltline_y + 0.02, 0.58),
        Vec3::new(0.72, beltline_y + 0.02, -0.75),
        Vec3::new(0.60, roof_y - 0.04, -0.75),
        Vec3::X,
        AutomotivePalette::TINTED_GLASS,
    );

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Glass".into(),
        submesh_type: VehicleSubmeshType::GlassGreenhouse,
        mesh: glass_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::TINTED_GLASS,
            metallic: 0.10,
            roughness: 0.05,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Blend,
        },
    });

    // -------------------------------------------------------------------------
    // 3. Cockpit Interior (Dashboard, Center Console, Racing Bucket Seats)
    // -------------------------------------------------------------------------
    let mut interior_mesh = ChunkMesh::default();

    // 3a. Sculpted Dashboard with Instrument Binnacle & Infotainment Screen
    add_box_at(
        &mut interior_mesh,
        Vec3::new(0.0, 0.68, 0.40),
        Vec3::new(0.65, 0.10, 0.18),
        AutomotivePalette::INTERIOR_CHARCOAL,
    );
    // Digital Instrument Cluster Display Screen
    add_quad(
        &mut interior_mesh,
        Vec3::new(-0.52, 0.72, 0.32),
        Vec3::new(-0.24, 0.72, 0.32),
        Vec3::new(-0.24, 0.80, 0.32),
        Vec3::new(-0.52, 0.80, 0.32),
        -Vec3::Z,
        [0.05, 0.45, 0.85, 1.0], // Glowing telemetry screen
    );
    // Center Infotainment Navigation Display
    add_quad(
        &mut interior_mesh,
        Vec3::new(-0.12, 0.66, 0.34),
        Vec3::new(0.12, 0.66, 0.34),
        Vec3::new(0.12, 0.76, 0.34),
        Vec3::new(-0.12, 0.76, 0.34),
        -Vec3::Z,
        [0.10, 0.70, 0.40, 1.0], // GPS map glow
    );

    // 3b. Center Tunnel Console & Gear Shifter
    add_box_at(
        &mut interior_mesh,
        Vec3::new(0.0, 0.46, -0.15),
        Vec3::new(0.12, 0.12, 0.45),
        AutomotivePalette::INTERIOR_CHARCOAL,
    );
    // Shift knob
    add_box_at(
        &mut interior_mesh,
        Vec3::new(0.0, 0.62, 0.05),
        Vec3::new(0.025, 0.045, 0.025),
        AutomotivePalette::POLISHED_CHROME,
    );

    // 3c. Driver Racing Bucket Seat (Left-hand drive, x = -0.38)
    // Seat Cushion
    add_box_at(
        &mut interior_mesh,
        Vec3::new(-0.38, 0.44, -0.22),
        Vec3::new(0.24, 0.08, 0.25),
        AutomotivePalette::INTERIOR_CHARCOAL,
    );
    // Sculpted Backrest with Side Thigh Bolsters
    add_box_at(
        &mut interior_mesh,
        Vec3::new(-0.38, 0.72, -0.42),
        Vec3::new(0.22, 0.22, 0.06),
        AutomotivePalette::LEATHER_ACCENT,
    );
    // Integrated Headrest
    add_box_at(
        &mut interior_mesh,
        Vec3::new(-0.38, 0.98, -0.46),
        Vec3::new(0.14, 0.08, 0.05),
        AutomotivePalette::INTERIOR_CHARCOAL,
    );

    // 3d. Passenger Racing Bucket Seat (Right side, x = +0.38)
    add_box_at(
        &mut interior_mesh,
        Vec3::new(0.38, 0.44, -0.22),
        Vec3::new(0.24, 0.08, 0.25),
        AutomotivePalette::INTERIOR_CHARCOAL,
    );
    add_box_at(
        &mut interior_mesh,
        Vec3::new(0.38, 0.72, -0.42),
        Vec3::new(0.22, 0.22, 0.06),
        AutomotivePalette::LEATHER_ACCENT,
    );
    add_box_at(
        &mut interior_mesh,
        Vec3::new(0.38, 0.98, -0.46),
        Vec3::new(0.14, 0.08, 0.05),
        AutomotivePalette::INTERIOR_CHARCOAL,
    );

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Interior".into(),
        submesh_type: VehicleSubmeshType::InteriorCockpit,
        mesh: interior_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::INTERIOR_CHARCOAL,
            metallic: 0.15,
            roughness: 0.85,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // -------------------------------------------------------------------------
    // 4. Articulated Steering Wheel
    // -------------------------------------------------------------------------
    let mut steer_mesh = ChunkMesh::default();
    let steer_pivot = Vec3::new(-0.38, 0.74, 0.18);

    // 3-Spoke Racing Steering Wheel Rim (torus approximation)
    let rim_r = 0.16;
    let segs = 16;
    for i in 0..segs {
        let a0 = i as f32 * (2.0 * PI / segs as f32);
        let a1 = (i + 1) as f32 * (2.0 * PI / segs as f32);
        let p0 = steer_pivot + Vec3::new(rim_r * a0.cos(), rim_r * a0.sin(), 0.0);
        let p1 = steer_pivot + Vec3::new(rim_r * a1.cos(), rim_r * a1.sin(), 0.0);
        add_box_at(
            &mut steer_mesh,
            (p0 + p1) * 0.5,
            Vec3::new(0.015, 0.015, 0.015),
            AutomotivePalette::CARBON_FIBER,
        );
    }
    // Center boss & spokes
    add_box_at(
        &mut steer_mesh,
        steer_pivot,
        Vec3::new(0.04, 0.04, 0.02),
        AutomotivePalette::ALLOY_SILVER,
    );

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Steering_Wheel".into(),
        submesh_type: VehicleSubmeshType::InteriorSteeringWheel,
        mesh: steer_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: steer_pivot,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::CARBON_FIBER,
            metallic: 0.30,
            roughness: 0.70,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // -------------------------------------------------------------------------
    // 5. High-Intensity Dynamic Lighting Assemblies
    // -------------------------------------------------------------------------
    // 5a. Projector LED Headlights & DRL Brows
    let mut headlight_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        let hx = sign * 0.62;
        let hy = 0.52;
        let hz = half_l - 0.14;
        add_quad(
            &mut headlight_mesh,
            Vec3::new(hx - sign * 0.16, hy - 0.05, hz),
            Vec3::new(hx + sign * 0.16, hy - 0.05, hz - 0.06),
            Vec3::new(hx + sign * 0.16, hy + 0.05, hz - 0.06),
            Vec3::new(hx - sign * 0.16, hy + 0.05, hz),
            Vec3::Z,
            AutomotivePalette::HEADLIGHT_XENON_ON,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Headlights".into(),
        submesh_type: VehicleSubmeshType::Headlights,
        mesh: headlight_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::HEADLIGHT_XENON_ON,
            metallic: 0.10,
            roughness: 0.10,
            emissive: [1.0, 0.98, 0.88],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 5b. Rear Taillight & Brake Light Full-Width OLED Bar
    let mut brake_mesh = ChunkMesh::default();
    add_quad(
        &mut brake_mesh,
        Vec3::new(-0.76, 0.74, -half_l + 0.02),
        Vec3::new(0.76, 0.74, -half_l + 0.02),
        Vec3::new(0.76, 0.79, -half_l + 0.02),
        Vec3::new(-0.76, 0.79, -half_l + 0.02),
        -Vec3::Z,
        AutomotivePalette::BRAKELIGHT_ON,
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "BrakeLights".into(),
        submesh_type: VehicleSubmeshType::BrakeLights,
        mesh: brake_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::BRAKELIGHT_ON,
            metallic: 0.10,
            roughness: 0.10,
            emissive: [1.0, 0.05, 0.05],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 5c. Dual Cool-White Reverse Lights
    let mut reverse_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        let rx = sign * 0.28;
        add_quad(
            &mut reverse_mesh,
            Vec3::new(rx - 0.08, 0.42, -half_l + 0.02),
            Vec3::new(rx + 0.08, 0.42, -half_l + 0.02),
            Vec3::new(rx + 0.08, 0.46, -half_l + 0.02),
            Vec3::new(rx - 0.08, 0.46, -half_l + 0.02),
            -Vec3::Z,
            AutomotivePalette::REVERSE_WHITE_ON,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "ReverseLights".into(),
        submesh_type: VehicleSubmeshType::ReverseLights,
        mesh: reverse_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::REVERSE_WHITE_ON,
            metallic: 0.10,
            roughness: 0.10,
            emissive: [0.98, 0.98, 1.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // -------------------------------------------------------------------------
    // 6. Elevated Aerodynamic GT Rear Wing / Spoiler
    // -------------------------------------------------------------------------
    let mut spoiler_mesh = ChunkMesh::default();
    let wing_z = -1.95;
    let wing_y = 1.05;
    let wing_span = 0.85;

    // Dual carbon fiber mounting stanchions
    for sign in [-1.0f32, 1.0f32] {
        let sx = sign * 0.42;
        add_quad(
            &mut spoiler_mesh,
            Vec3::new(sx, 0.82, wing_z + 0.05),
            Vec3::new(sx, 0.82, wing_z - 0.05),
            Vec3::new(sx, wing_y, wing_z - 0.05),
            Vec3::new(sx, wing_y, wing_z + 0.05),
            Vec3::X,
            AutomotivePalette::CARBON_FIBER,
        );
    }
    // Main airfoil wing element
    add_quad(
        &mut spoiler_mesh,
        Vec3::new(-wing_span, wing_y, wing_z + 0.12),
        Vec3::new(wing_span, wing_y, wing_z + 0.12),
        Vec3::new(wing_span, wing_y + 0.03, wing_z - 0.12),
        Vec3::new(-wing_span, wing_y + 0.03, wing_z - 0.12),
        Vec3::Y,
        AutomotivePalette::CARBON_FIBER,
    );
    // Endplates
    for sign in [-1.0f32, 1.0f32] {
        let ex = sign * wing_span;
        add_box_at(
            &mut spoiler_mesh,
            Vec3::new(ex, wing_y + 0.02, wing_z),
            Vec3::new(0.015, 0.08, 0.15),
            AutomotivePalette::CARBON_FIBER,
        );
    }

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Spoiler".into(),
        submesh_type: VehicleSubmeshType::AerodynamicSpoiler,
        mesh: spoiler_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::new(0.0, wing_y, wing_z),
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::CARBON_FIBER,
            metallic: 0.90,
            roughness: 0.20,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // -------------------------------------------------------------------------
    // 7. Quad Polished Stainless Steel Exhaust System
    // -------------------------------------------------------------------------
    let mut exhaust_mesh = ChunkMesh::default();
    for ex_x in [-0.72, -0.60, 0.60, 0.72] {
        add_cylinder_x(
            &mut exhaust_mesh,
            Vec3::new(ex_x, 0.26, -half_l - 0.04),
            0.045,
            0.02,
            12,
            AutomotivePalette::POLISHED_CHROME,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Exhaust".into(),
        submesh_type: VehicleSubmeshType::Exhaust,
        mesh: exhaust_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::POLISHED_CHROME,
            metallic: 0.98,
            roughness: 0.08,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // -------------------------------------------------------------------------
    // 8. Ground Contact Shadow Plane
    // -------------------------------------------------------------------------
    let mut shadow_mesh = ChunkMesh::default();
    add_quad(
        &mut shadow_mesh,
        Vec3::new(-half_w * 1.05, 0.02, half_l * 0.98),
        Vec3::new(half_w * 1.05, 0.02, half_l * 0.98),
        Vec3::new(half_w * 1.05, 0.02, -half_l * 0.98),
        Vec3::new(-half_w * 1.05, 0.02, -half_l * 0.98),
        Vec3::Y,
        AutomotivePalette::CONTACT_SHADOW,
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "ContactShadow".into(),
        submesh_type: VehicleSubmeshType::ContactShadow,
        mesh: shadow_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::CONTACT_SHADOW,
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Blend,
        },
    });

    // -------------------------------------------------------------------------
    // 9. Separate Transformable 4-Wheel Assemblies (FL, FR, RL, RR)
    // -------------------------------------------------------------------------
    let wheel_r = 0.33;
    let wheel_w = 0.245;
    let half_wb = 1.25; // 2.50m wheelbase
    let half_tw = 0.825; // 1.65m track width

    // Front-Left Wheel
    let fl_pivot = Vec3::new(-half_tw, wheel_r, half_wb);
    let fl_mesh = build_wheel_assembly(wheel_r, wheel_w, true, lod);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Wheel_FL".into(),
        submesh_type: VehicleSubmeshType::WheelFrontLeft,
        mesh: fl_mesh,
        local_transform: Affine3A::from_translation(fl_pivot),
        pivot: fl_pivot,
        material: VehiclePbrMaterial::default(),
    });

    // Front-Right Wheel
    let fr_pivot = Vec3::new(half_tw, wheel_r, half_wb);
    let fr_mesh = build_wheel_assembly(wheel_r, wheel_w, false, lod);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Wheel_FR".into(),
        submesh_type: VehicleSubmeshType::WheelFrontRight,
        mesh: fr_mesh,
        local_transform: Affine3A::from_translation(fr_pivot),
        pivot: fr_pivot,
        material: VehiclePbrMaterial::default(),
    });

    // Rear-Left Wheel
    let rl_pivot = Vec3::new(-half_tw, wheel_r, -half_wb);
    let rl_mesh = build_wheel_assembly(wheel_r, wheel_w, true, lod);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Wheel_RL".into(),
        submesh_type: VehicleSubmeshType::WheelRearLeft,
        mesh: rl_mesh,
        local_transform: Affine3A::from_translation(rl_pivot),
        pivot: rl_pivot,
        material: VehiclePbrMaterial::default(),
    });

    // Rear-Right Wheel
    let rr_pivot = Vec3::new(half_tw, wheel_r, -half_wb);
    let rr_mesh = build_wheel_assembly(wheel_r, wheel_w, false, lod);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Wheel_RR".into(),
        submesh_type: VehicleSubmeshType::WheelRearRight,
        mesh: rr_mesh,
        local_transform: Affine3A::from_translation(rr_pivot),
        pivot: rr_pivot,
        material: VehiclePbrMaterial::default(),
    });

    builder.build()
}

// =============================================================================
// Executive Sedan — High-Fidelity 3D Vehicle Builder
// =============================================================================

pub fn build_executive_sedan(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("executive_sedan", "Executive Luxury Sedan")
        .with_physics(2.70, 1.60, 0.33, 1450.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 0.50, 0.0),
            chase_distance: 6.5,
            chase_height: 1.90,
            cockpit_eye: Vec3::new(-0.38, 0.98, -0.10),
            cockpit_look: Vec3::new(-0.38, 0.98, 2.5),
            hood_eye: Vec3::new(0.0, 0.88, 1.70),
            orbit_pivot: Vec3::new(0.0, 0.50, 0.0),
        });

    let half_w = 0.93;
    let half_l = 2.42; // 4.85m overall length
    let beltline_y = 0.72;
    let roof_y = 1.45;

    let mut chassis_mesh = ChunkMesh::default();

    // 1. Sleek 3-Box Sedan Chassis (Bumper, Hood, Trunk, Doors)
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.48, 0.0),
        Vec3::new(half_w, 0.24, half_l),
        AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
    );
    // Chrome Front Grille
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.55, 0.32, half_l + 0.01),
        Vec3::new(0.55, 0.32, half_l + 0.01),
        Vec3::new(0.50, 0.65, half_l + 0.01),
        Vec3::new(-0.50, 0.65, half_l + 0.01),
        Vec3::Z,
        AutomotivePalette::POLISHED_CHROME,
    );
    // Sloped Hood
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.75, 0.65, half_l - 0.05),
        Vec3::new(0.75, 0.65, half_l - 0.05),
        Vec3::new(0.82, beltline_y, 0.75),
        Vec3::new(-0.82, beltline_y, 0.75),
        Vec3::new(0.0, 0.25, 0.96).normalize(),
        AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
    );
    // Trunk Decklid
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.78, beltline_y, -1.35),
        Vec3::new(0.78, beltline_y, -1.35),
        Vec3::new(0.75, beltline_y - 0.05, -half_l + 0.05),
        Vec3::new(-0.75, beltline_y - 0.05, -half_l + 0.05),
        Vec3::Y,
        AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
    );

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Chassis".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
            metallic: 0.82,
            roughness: 0.22,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 2. Sedan Greenhouse Glass
    let mut glass_mesh = ChunkMesh::default();
    // Windshield
    add_quad(
        &mut glass_mesh,
        Vec3::new(-0.78, beltline_y + 0.01, 0.75),
        Vec3::new(0.78, beltline_y + 0.01, 0.75),
        Vec3::new(0.65, roof_y, 0.15),
        Vec3::new(-0.65, roof_y, 0.15),
        Vec3::new(0.0, 0.60, 0.80).normalize(),
        AutomotivePalette::TINTED_GLASS,
    );
    // Roof
    add_quad(
        &mut glass_mesh,
        Vec3::new(-0.65, roof_y, 0.15),
        Vec3::new(0.65, roof_y, 0.15),
        Vec3::new(0.65, roof_y, -0.65),
        Vec3::new(-0.65, roof_y, -0.65),
        Vec3::Y,
        AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
    );
    // Rear Window
    add_quad(
        &mut glass_mesh,
        Vec3::new(-0.65, roof_y, -0.65),
        Vec3::new(0.65, roof_y, -0.65),
        Vec3::new(0.72, beltline_y + 0.01, -1.35),
        Vec3::new(-0.72, beltline_y + 0.01, -1.35),
        Vec3::new(0.0, 0.60, -0.80).normalize(),
        AutomotivePalette::TINTED_GLASS,
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Glass".into(),
        submesh_type: VehicleSubmeshType::GlassGreenhouse,
        mesh: glass_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::TINTED_GLASS,
            metallic: 0.10,
            roughness: 0.05,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Blend,
        },
    });

    // 3. Transformable 4 Wheels
    let wheel_r = 0.33;
    let wheel_w = 0.235;
    let half_wb = 1.35;
    let half_tw = 0.80;

    for (name, st, px, pz, is_left) in [
        (
            "Wheel_FL",
            VehicleSubmeshType::WheelFrontLeft,
            -half_tw,
            half_wb,
            true,
        ),
        (
            "Wheel_FR",
            VehicleSubmeshType::WheelFrontRight,
            half_tw,
            half_wb,
            false,
        ),
        (
            "Wheel_RL",
            VehicleSubmeshType::WheelRearLeft,
            -half_tw,
            -half_wb,
            true,
        ),
        (
            "Wheel_RR",
            VehicleSubmeshType::WheelRearRight,
            half_tw,
            -half_wb,
            false,
        ),
    ] {
        let pivot = Vec3::new(px, wheel_r, pz);
        let w_mesh = build_wheel_assembly(wheel_r, wheel_w, is_left, lod);
        builder = builder.add_submesh(VehicleSubmesh {
            name: name.into(),
            submesh_type: st,
            mesh: w_mesh,
            local_transform: Affine3A::from_translation(pivot),
            pivot,
            material: VehiclePbrMaterial::default(),
        });
    }

    builder.build()
}

// =============================================================================
// Overland Explorer SUV — High-Fidelity 3D Vehicle Builder
// =============================================================================

pub fn build_overland_explorer_suv(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("overland_explorer_suv", "Overland Explorer SUV")
        .with_physics(2.85, 1.72, 0.38, 1950.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 0.65, 0.0),
            chase_distance: 7.2,
            chase_height: 2.20,
            cockpit_eye: Vec3::new(-0.42, 1.25, -0.10),
            cockpit_look: Vec3::new(-0.42, 1.25, 2.8),
            hood_eye: Vec3::new(0.0, 1.15, 1.80),
            orbit_pivot: Vec3::new(0.0, 0.65, 0.0),
        });

    let half_w = 0.99;
    let half_l = 2.45;
    let roof_y = 1.75;

    let mut chassis_mesh = ChunkMesh::default();
    // Rugged 2-Box High-Clearance SUV Body
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.62, 0.0),
        Vec3::new(half_w, 0.32, half_l),
        AutomotivePalette::SUV_GRANITE_GRAY,
    );
    // Lower protective cladding
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.35, 0.0),
        Vec3::new(half_w + 0.02, 0.10, half_l + 0.02),
        AutomotivePalette::CARBON_FIBER,
    );
    // Roof rack rails
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * 0.75, roof_y + 0.05, -0.2),
            Vec3::new(0.025, 0.025, 1.1),
            AutomotivePalette::ALLOY_SILVER,
        );
    }

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Chassis".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::SUV_GRANITE_GRAY,
            metallic: 0.70,
            roughness: 0.35,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // SUV Wheels
    let wheel_r = 0.38;
    let wheel_w = 0.265;
    let half_wb = 1.425;
    let half_tw = 0.86;

    for (name, st, px, pz, is_left) in [
        (
            "Wheel_FL",
            VehicleSubmeshType::WheelFrontLeft,
            -half_tw,
            half_wb,
            true,
        ),
        (
            "Wheel_FR",
            VehicleSubmeshType::WheelFrontRight,
            half_tw,
            half_wb,
            false,
        ),
        (
            "Wheel_RL",
            VehicleSubmeshType::WheelRearLeft,
            -half_tw,
            -half_wb,
            true,
        ),
        (
            "Wheel_RR",
            VehicleSubmeshType::WheelRearRight,
            half_tw,
            -half_wb,
            false,
        ),
    ] {
        let pivot = Vec3::new(px, wheel_r, pz);
        let w_mesh = build_wheel_assembly(wheel_r, wheel_w, is_left, lod);
        builder = builder.add_submesh(VehicleSubmesh {
            name: name.into(),
            submesh_type: st,
            mesh: w_mesh,
            local_transform: Affine3A::from_translation(pivot),
            pivot,
            material: VehiclePbrMaterial::default(),
        });
    }

    builder.build()
}

// =============================================================================
// Metro Yellow Cab — High-Fidelity 3D Vehicle Builder
// =============================================================================

pub fn build_metro_yellow_cab(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut model = build_executive_sedan(lod);
    model.asset_id = "metro_yellow_cab".into();
    model.display_name = "Metro Yellow Cab".into();

    // Re-color chassis to iconic yellow taxi livery & add illuminated roof lantern
    for submesh in &mut model.submeshes {
        if submesh.submesh_type == VehicleSubmeshType::ChassisBody {
            for v in &mut submesh.mesh.vertices {
                v.color = AutomotivePalette::TAXI_MUNICIPAL_YELLOW;
            }
            // Add rooftop taxi sign lantern
            add_box_at(
                &mut submesh.mesh,
                Vec3::new(0.0, 1.48, -0.15),
                Vec3::new(0.24, 0.065, 0.10),
                [1.0, 0.95, 0.60, 1.0], // Illuminated taxi sign
            );
        }
    }

    model
}

// =============================================================================
// TransContinental Semi (Heavy Truck) — High-Fidelity 3D Vehicle Builder
// =============================================================================

pub fn build_transcontinental_semi(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("transcontinental_semi", "TransContinental Semi")
        .with_physics(4.20, 2.05, 0.52, 7500.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 1.20, 0.0),
            chase_distance: 12.0,
            chase_height: 3.80,
            cockpit_eye: Vec3::new(-0.75, 2.30, 0.80),
            cockpit_look: Vec3::new(-0.75, 2.30, 4.5),
            hood_eye: Vec3::new(0.0, 2.05, 2.60),
            orbit_pivot: Vec3::new(0.0, 1.20, 0.0),
        });

    let mut chassis_mesh = ChunkMesh::default();
    let half_w = 1.27;

    // Heavy Conventional Cab
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.85, 0.60),
        Vec3::new(half_w, 1.30, 1.40),
        AutomotivePalette::TRUCK_CRIMSON_HAULER,
    );
    // Long Engine Hood
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.25, 2.45),
        Vec3::new(0.95, 0.70, 0.85),
        AutomotivePalette::TRUCK_CRIMSON_HAULER,
    );
    // Chrome Vertical Exhaust Stacks
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_x(
            &mut chassis_mesh,
            Vec3::new(sign * 1.15, 2.65, -0.65),
            0.08,
            0.04,
            12,
            AutomotivePalette::POLISHED_CHROME,
        );
    }

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Chassis".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::TRUCK_CRIMSON_HAULER,
            metallic: 0.85,
            roughness: 0.25,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // Heavy Truck Wheels
    let wheel_r = 0.52;
    let wheel_w = 0.32;
    for (name, st, px, pz, is_left) in [
        (
            "Wheel_FL",
            VehicleSubmeshType::WheelFrontLeft,
            -1.02,
            2.40,
            true,
        ),
        (
            "Wheel_FR",
            VehicleSubmeshType::WheelFrontRight,
            1.02,
            2.40,
            false,
        ),
        (
            "Wheel_RL",
            VehicleSubmeshType::WheelRearLeft,
            -1.02,
            -1.20,
            true,
        ),
        (
            "Wheel_RR",
            VehicleSubmeshType::WheelRearRight,
            1.02,
            -1.20,
            false,
        ),
    ] {
        let pivot = Vec3::new(px, wheel_r, pz);
        let w_mesh = build_wheel_assembly(wheel_r, wheel_w, is_left, lod);
        builder = builder.add_submesh(VehicleSubmesh {
            name: name.into(),
            submesh_type: st,
            mesh: w_mesh,
            local_transform: Affine3A::from_translation(pivot),
            pivot,
            material: VehiclePbrMaterial::default(),
        });
    }

    builder.build()
}

// =============================================================================
// CityLine Transit Bus — High-Fidelity 3D Vehicle Builder
// =============================================================================

pub fn build_cityline_transit_bus(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("cityline_transit_bus", "CityLine Transit Bus")
        .with_physics(6.80, 2.15, 0.50, 11500.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 1.40, 0.0),
            chase_distance: 16.0,
            chase_height: 4.20,
            cockpit_eye: Vec3::new(-0.75, 1.65, 4.80),
            cockpit_look: Vec3::new(-0.75, 1.65, 8.5),
            hood_eye: Vec3::new(0.0, 1.50, 5.80),
            orbit_pivot: Vec3::new(0.0, 1.40, 0.0),
        });

    let mut chassis_mesh = ChunkMesh::default();
    let half_w = 1.25;
    let half_l = 5.90;

    // 40ft Low-Floor Bus Monocoque
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.65, 0.0),
        Vec3::new(half_w, 1.35, half_l),
        AutomotivePalette::BUS_TRANSIT_GREEN,
    );
    // Destination LED sign matrix
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.95, 2.70, half_l + 0.02),
        Vec3::new(0.95, 2.70, half_l + 0.02),
        Vec3::new(0.95, 2.92, half_l + 0.02),
        Vec3::new(-0.95, 2.92, half_l + 0.02),
        Vec3::Z,
        [1.0, 0.75, 0.10, 1.0], // Amber LED destination display
    );

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Chassis".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::BUS_TRANSIT_GREEN,
            metallic: 0.60,
            roughness: 0.30,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    let wheel_r = 0.50;
    let wheel_w = 0.28;
    for (name, st, px, pz, is_left) in [
        (
            "Wheel_FL",
            VehicleSubmeshType::WheelFrontLeft,
            -1.07,
            3.40,
            true,
        ),
        (
            "Wheel_FR",
            VehicleSubmeshType::WheelFrontRight,
            1.07,
            3.40,
            false,
        ),
        (
            "Wheel_RL",
            VehicleSubmeshType::WheelRearLeft,
            -1.07,
            -3.40,
            true,
        ),
        (
            "Wheel_RR",
            VehicleSubmeshType::WheelRearRight,
            1.07,
            -3.40,
            false,
        ),
    ] {
        let pivot = Vec3::new(px, wheel_r, pz);
        let w_mesh = build_wheel_assembly(wheel_r, wheel_w, is_left, lod);
        builder = builder.add_submesh(VehicleSubmesh {
            name: name.into(),
            submesh_type: st,
            mesh: w_mesh,
            local_transform: Affine3A::from_translation(pivot),
            pivot,
            material: VehiclePbrMaterial::default(),
        });
    }

    builder.build()
}

// =============================================================================
// HyperSport RR-1000 — High-Fidelity 3D Motorcycle Builder
// =============================================================================

pub fn build_hypersport_rr1000(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("hypersport_rr1000", "HyperSport RR-1000")
        .with_physics(1.42, 0.16, 0.31, 195.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 0.55, 0.0),
            chase_distance: 4.5,
            chase_height: 1.45,
            cockpit_eye: Vec3::new(0.0, 0.88, 0.10),
            cockpit_look: Vec3::new(0.0, 0.88, 2.0),
            hood_eye: Vec3::new(0.0, 0.75, 0.85),
            orbit_pivot: Vec3::new(0.0, 0.55, 0.0),
        });

    let mut chassis_mesh = ChunkMesh::default();

    // Aerodynamic Supersport Fairing & Fuel Tank
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.75, 0.20),
        Vec3::new(0.22, 0.18, 0.42),
        AutomotivePalette::BIKE_RACING_ORANGE,
    );
    // Exposed Engine Block & Trellis Frame
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.42, 0.0),
        Vec3::new(0.18, 0.16, 0.25),
        AutomotivePalette::ALLOY_SILVER,
    );
    // Tail cowl & rider saddle
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.68, -0.45),
        Vec3::new(0.14, 0.12, 0.35),
        AutomotivePalette::INTERIOR_CHARCOAL,
    );

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Chassis".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::BIKE_RACING_ORANGE,
            metallic: 0.85,
            roughness: 0.20,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // Front and Rear Wheels
    let wheel_r = 0.31;
    let wheel_w = 0.12;

    let f_pivot = Vec3::new(0.0, wheel_r, 0.71);
    let f_mesh = build_wheel_assembly(wheel_r, wheel_w, true, lod);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Wheel_Front".into(),
        submesh_type: VehicleSubmeshType::WheelFrontLeft,
        mesh: f_mesh,
        local_transform: Affine3A::from_translation(f_pivot),
        pivot: f_pivot,
        material: VehiclePbrMaterial::default(),
    });

    let r_pivot = Vec3::new(0.0, wheel_r, -0.71);
    let r_mesh = build_wheel_assembly(wheel_r, wheel_w, true, lod);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Wheel_Rear".into(),
        submesh_type: VehicleSubmeshType::WheelRearLeft,
        mesh: r_mesh,
        local_transform: Affine3A::from_translation(r_pivot),
        pivot: r_pivot,
        material: VehiclePbrMaterial::default(),
    });

    builder.build()
}

// =============================================================================
// Static Asset Cache & GLB Disk Publication
// =============================================================================

static APEX_GT_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static APEX_GT_LOD1: OnceLock<LoadedVehicleModel> = OnceLock::new();
static APEX_GT_LOD2: OnceLock<LoadedVehicleModel> = OnceLock::new();
static SEDAN_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static SUV_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static TAXI_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static TRUCK_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static BUS_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static BIKE_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();

/// Retrieves the certified, high-fidelity vehicle model for the specified type and LOD tier.
pub fn get_certified_vehicle_model(
    v_type: VehicleVisualType,
    lod: VehicleLodLevel,
) -> &'static LoadedVehicleModel {
    match v_type {
        VehicleVisualType::SportsCoupe => {
            match lod {
                VehicleLodLevel::Lod0High => APEX_GT_LOD0
                    .get_or_init(|| build_apex_gt_sports_coupe(VehicleLodLevel::Lod0High)),
                VehicleLodLevel::Lod1Medium => APEX_GT_LOD1
                    .get_or_init(|| build_apex_gt_sports_coupe(VehicleLodLevel::Lod1Medium)),
                VehicleLodLevel::Lod2Low => APEX_GT_LOD2
                    .get_or_init(|| build_apex_gt_sports_coupe(VehicleLodLevel::Lod2Low)),
            }
        }
        VehicleVisualType::Sedan => SEDAN_LOD0.get_or_init(|| build_executive_sedan(lod)),
        VehicleVisualType::Suv => SUV_LOD0.get_or_init(|| build_overland_explorer_suv(lod)),
        VehicleVisualType::Taxi => TAXI_LOD0.get_or_init(|| build_metro_yellow_cab(lod)),
        VehicleVisualType::HeavyTruck => {
            TRUCK_LOD0.get_or_init(|| build_transcontinental_semi(lod))
        }
        VehicleVisualType::TransitBus => BUS_LOD0.get_or_init(|| build_cityline_transit_bus(lod)),
        VehicleVisualType::Motorcycle => BIKE_LOD0.get_or_init(|| build_hypersport_rr1000(lod)),
        _ => APEX_GT_LOD0.get_or_init(|| build_apex_gt_sports_coupe(VehicleLodLevel::Lod0High)),
    }
}

/// Compiles and serializes all certified vehicle models into binary GLB files in `target_dir`.
pub fn export_all_certified_models_to_glb_directory(
    target_dir: &Path,
) -> Result<(), GltfAssetError> {
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)?;
    }

    let models = [
        (
            "apex_gt_sports_coupe_lod0.glb",
            build_apex_gt_sports_coupe(VehicleLodLevel::Lod0High),
        ),
        (
            "apex_gt_sports_coupe_lod1.glb",
            build_apex_gt_sports_coupe(VehicleLodLevel::Lod1Medium),
        ),
        (
            "apex_gt_sports_coupe_lod2.glb",
            build_apex_gt_sports_coupe(VehicleLodLevel::Lod2Low),
        ),
        (
            "executive_sedan_lod0.glb",
            build_executive_sedan(VehicleLodLevel::Lod0High),
        ),
        (
            "executive_sedan_lod1.glb",
            build_executive_sedan(VehicleLodLevel::Lod1Medium),
        ),
        (
            "executive_sedan_lod2.glb",
            build_executive_sedan(VehicleLodLevel::Lod2Low),
        ),
        (
            "overland_explorer_suv_lod0.glb",
            build_overland_explorer_suv(VehicleLodLevel::Lod0High),
        ),
        (
            "metro_yellow_cab_lod0.glb",
            build_metro_yellow_cab(VehicleLodLevel::Lod0High),
        ),
        (
            "transcontinental_semi_lod0.glb",
            build_transcontinental_semi(VehicleLodLevel::Lod0High),
        ),
        (
            "cityline_transit_bus_lod0.glb",
            build_cityline_transit_bus(VehicleLodLevel::Lod0High),
        ),
        (
            "hypersport_rr1000_lod0.glb",
            build_hypersport_rr1000(VehicleLodLevel::Lod0High),
        ),
    ];

    for (filename, model) in models {
        let mut builder = GltfAssetBuilder::new(&model.asset_id, &model.display_name)
            .with_physics(
                model.wheelbase,
                model.track_width,
                model.wheel_radius,
                model.mass_kg,
            )
            .with_lod(model.lod_level)
            .with_camera_anchors(model.camera_anchors);

        for s in model.submeshes {
            builder = builder.add_submesh(s);
        }

        let glb_bytes = builder.to_glb_bytes()?;
        let path = target_dir.join(filename);
        std::fs::write(&path, glb_bytes)?;
    }

    Ok(())
}
