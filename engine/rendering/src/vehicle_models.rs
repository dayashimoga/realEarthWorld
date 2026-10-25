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

/// Adds a curved cylindrical extrusion along the Y-axis (vertical orientation).
pub fn add_cylinder_y(
    mesh: &mut ChunkMesh,
    center: Vec3,
    radius: f32,
    half_height: f32,
    segments: usize,
    color: [f32; 4],
) {
    let angle_step = 2.0 * PI / segments as f32;
    let y_bottom = center.y - half_height;
    let y_top = center.y + half_height;

    for i in 0..segments {
        let a0 = i as f32 * angle_step;
        let a1 = (i + 1) as f32 * angle_step;

        let x0 = center.x + radius * a0.cos();
        let z0 = center.z + radius * a0.sin();
        let x1 = center.x + radius * a1.cos();
        let z1 = center.z + radius * a1.sin();

        let n0 = Vec3::new(a0.cos(), 0.0, a0.sin()).normalize();
        let n1 = Vec3::new(a1.cos(), 0.0, a1.sin()).normalize();
        let face_n = (n0 + n1) * 0.5;

        // Outer surface quad
        add_quad(
            mesh,
            Vec3::new(x0, y_bottom, z0),
            Vec3::new(x1, y_bottom, z1),
            Vec3::new(x1, y_top, z1),
            Vec3::new(x0, y_top, z0),
            face_n,
            color,
        );

        // Bottom endcap face (-Y)
        add_tri(
            mesh,
            Vec3::new(center.x, y_bottom, center.z),
            Vec3::new(x1, y_bottom, z1),
            Vec3::new(x0, y_bottom, z0),
            -Vec3::Y,
            color,
        );

        // Top endcap face (+Y)
        add_tri(
            mesh,
            Vec3::new(center.x, y_top, center.z),
            Vec3::new(x0, y_top, z0),
            Vec3::new(x1, y_top, z1),
            Vec3::Y,
            color,
        );
    }
}

/// Adds a curved cylindrical extrusion along the Z-axis (longitudinal orientation).
pub fn add_cylinder_z(
    mesh: &mut ChunkMesh,
    center: Vec3,
    radius: f32,
    half_length: f32,
    segments: usize,
    color: [f32; 4],
) {
    let angle_step = 2.0 * PI / segments as f32;
    let z_rear = center.z - half_length;
    let z_front = center.z + half_length;

    for i in 0..segments {
        let a0 = i as f32 * angle_step;
        let a1 = (i + 1) as f32 * angle_step;

        let x0 = center.x + radius * a0.cos();
        let y0 = center.y + radius * a0.sin();
        let x1 = center.x + radius * a1.cos();
        let y1 = center.y + radius * a1.sin();

        let n0 = Vec3::new(a0.cos(), a0.sin(), 0.0).normalize();
        let n1 = Vec3::new(a1.cos(), a1.sin(), 0.0).normalize();
        let face_n = (n0 + n1) * 0.5;

        // Outer surface quad
        add_quad(
            mesh,
            Vec3::new(x0, y0, z_rear),
            Vec3::new(x0, y0, z_front),
            Vec3::new(x1, y1, z_front),
            Vec3::new(x1, y1, z_rear),
            face_n,
            color,
        );

        // Rear endcap face (-Z)
        add_tri(
            mesh,
            Vec3::new(center.x, center.y, z_rear),
            Vec3::new(x1, y1, z_rear),
            Vec3::new(x0, y0, z_rear),
            -Vec3::Z,
            color,
        );

        // Front endcap face (+Z)
        add_tri(
            mesh,
            Vec3::new(center.x, center.y, z_front),
            Vec3::new(x0, y0, z_front),
            Vec3::new(x1, y1, z_front),
            Vec3::Z,
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

/// Builds an articulated, flanged steel railway wheelset (axle, two flanged wheels conforming to standard gauge).
pub fn build_rail_wheelset(tread_radius: f32, gauge_half: f32, lod: VehicleLodLevel) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let segments = match lod {
        VehicleLodLevel::Lod0High => 24,
        VehicleLodLevel::Lod1Medium => 16,
        VehicleLodLevel::Lod2Low => 10,
    };

    let steel_tread = [0.85, 0.88, 0.92, 1.0]; // Polished machined steel contact band
    let steel_flange = [0.25, 0.28, 0.32, 1.0]; // Cast railway steel hub and flange
    let axle_steel = [0.20, 0.22, 0.25, 1.0]; // Heavy solid steel axle

    // 1. Center solid axle along X-axis
    let axle_radius = tread_radius * 0.22;
    add_cylinder_x(
        &mut mesh,
        Vec3::ZERO,
        axle_radius,
        gauge_half + 0.12,
        segments,
        axle_steel,
    );

    // 2. Left and Right flanged railway wheels conforming to 1435mm gauge
    let flange_radius = tread_radius * 1.06;
    let wheel_w = 0.14;
    let flange_w = 0.035;

    for sign in [-1.0f32, 1.0f32] {
        let wheel_center_x = sign * gauge_half;
        // Tread cylinder
        add_cylinder_x(
            &mut mesh,
            Vec3::new(wheel_center_x, 0.0, 0.0),
            tread_radius,
            wheel_w * 0.5,
            segments,
            steel_tread,
        );
        // Inboard flange ring (facing towards center of track)
        let flange_x = wheel_center_x - sign * (wheel_w * 0.5 + flange_w * 0.5);
        add_cylinder_x(
            &mut mesh,
            Vec3::new(flange_x, 0.0, 0.0),
            flange_radius,
            flange_w * 0.5,
            segments,
            steel_flange,
        );
    }

    mesh
}

/// Builds a heavy-duty 2-axle railway bogie (truck) frame with secondary coil springs and brake rigging.
pub fn build_railway_bogie(
    wheelbase: f32,
    wheel_radius: f32,
    gauge_half: f32,
    lod: VehicleLodLevel,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let bogie_steel = [0.22, 0.24, 0.28, 1.0];
    let spring_steel = [0.15, 0.15, 0.17, 1.0];

    let half_wb = wheelbase * 0.5;

    // Sideframes on Left (+X) and Right (-X) outside the wheels
    let sideframe_x = gauge_half + 0.18;
    for sign in [-1.0f32, 1.0f32] {
        let sx = sign * sideframe_x;
        // Longitudinal main beam
        add_box_at(
            &mut mesh,
            Vec3::new(sx, wheel_radius * 0.70, 0.0),
            Vec3::new(0.06, 0.12, half_wb + 0.35),
            bogie_steel,
        );
        // Journal bearing pedestal drop links over both axles
        for az in [-half_wb, half_wb] {
            add_box_at(
                &mut mesh,
                Vec3::new(sx, wheel_radius * 0.55, az),
                Vec3::new(0.08, 0.16, 0.18),
                bogie_steel,
            );
        }
        // Secondary coil spring clusters in the center of the sideframe
        add_box_at(
            &mut mesh,
            Vec3::new(sx, wheel_radius * 0.85, 0.0),
            Vec3::new(0.07, 0.10, 0.25),
            spring_steel,
        );
    }

    // Transverse center bolster & pivot crossmember
    add_box_at(
        &mut mesh,
        Vec3::new(0.0, wheel_radius * 0.85, 0.0),
        Vec3::new(sideframe_x, 0.10, 0.30),
        bogie_steel,
    );
    // Center pivot bearing cup
    add_cylinder_y(
        &mut mesh,
        Vec3::new(0.0, wheel_radius * 0.95, 0.0),
        0.22,
        0.05,
        12,
        bogie_steel,
    );

    // Add 2 flanged wheelsets at front and rear axles
    for az in [-half_wb, half_wb] {
        let ws = build_rail_wheelset(wheel_radius, gauge_half, lod);
        let base_idx = mesh.vertices.len() as u32;
        for v in ws.vertices {
            let p = Vec3::from_array(v.position) + Vec3::new(0.0, wheel_radius, az);
            mesh.vertices
                .push(Vertex::new(p.to_array(), v.normal, v.uv, v.color));
        }
        for idx in ws.indices {
            mesh.indices.push(idx + base_idx);
        }
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

    // 1e. Flared Front Fenders with Authentic Non-Clipping Wheel Arches & Dark Wheel Liners
    // Front wheel axle is at z = 1.25, wheel_r = 0.33, tire top at y = 0.66. Wheel extends z in [0.92, 1.58].
    for sign in [-1.0f32, 1.0f32] {
        let fx = sign * half_w;
        let fnorm = Vec3::new(sign, 0.0, 0.0);
        let inner_x = sign * (half_w - 0.12);
        let wheel_liner_color = [0.05, 0.05, 0.06, 1.0];

        // 1e-i. Forward Fender Section (Ahead of wheel arch: z = 1.58 to 1.75)
        add_quad(
            &mut chassis_mesh,
            Vec3::new(fx, 0.32, 1.75),
            Vec3::new(sign * 0.80, 0.58, 1.75),
            Vec3::new(sign * 0.82, 0.62, 1.58),
            Vec3::new(fx, 0.32, 1.58),
            fnorm,
            AutomotivePalette::APEX_CHERRY_RED,
        );

        // 1e-ii. Sculpted Wheel Arch Cutout (Over wheel: z = 0.92 to 1.58, arch apex y = 0.72)
        // Forward arch quadrant (z = 1.25 to 1.58)
        add_quad(
            &mut chassis_mesh,
            Vec3::new(fx, 0.32, 1.58),
            Vec3::new(sign * 0.82, 0.62, 1.58),
            Vec3::new(sign * 0.85, 0.68, 1.25),
            Vec3::new(fx, 0.72, 1.25), // Crest clears tire (0.72m > 0.66m)
            fnorm,
            AutomotivePalette::APEX_CHERRY_RED,
        );
        // Aft arch quadrant (z = 0.92 to 1.25)
        add_quad(
            &mut chassis_mesh,
            Vec3::new(fx, 0.72, 1.25),
            Vec3::new(sign * 0.85, 0.68, 1.25),
            Vec3::new(sign * 0.85, 0.65, 0.92),
            Vec3::new(fx, 0.32, 0.92),
            fnorm,
            AutomotivePalette::APEX_CHERRY_RED,
        );

        // 1e-iii. Aft Fender Section (Behind wheel arch: z = 0.65 to 0.92)
        add_quad(
            &mut chassis_mesh,
            Vec3::new(fx, 0.32, 0.92),
            Vec3::new(sign * 0.85, 0.65, 0.92),
            Vec3::new(sign * 0.85, 0.65, 0.65),
            Vec3::new(fx, 0.35, 0.65),
            fnorm,
            AutomotivePalette::APEX_CHERRY_RED,
        );

        // 1e-iv. Matte Dark Inner Wheel Well Cavity Liner
        add_quad(
            &mut chassis_mesh,
            Vec3::new(inner_x, 0.70, 0.92),
            Vec3::new(fx, 0.72, 0.92),
            Vec3::new(fx, 0.72, 1.58),
            Vec3::new(inner_x, 0.70, 1.58),
            -Vec3::Y,
            wheel_liner_color,
        );
    }

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

    // 1g. Flared Rear Muscular Haunches with Non-Clipping Wheel Arches & Dark Wheel Liners
    // Rear wheel axle is at z = -1.25, wheel_r = 0.33, tire top at y = 0.66. Wheel extends z in [-1.58, -0.92].
    for sign in [-1.0f32, 1.0f32] {
        let rx = sign * half_w;
        let rnorm = Vec3::new(sign, 0.0, 0.0);
        let inner_x = sign * (half_w - 0.12);
        let wheel_liner_color = [0.05, 0.05, 0.06, 1.0];

        // 1g-i. Forward Haunch Section (Ahead of rear wheel arch: z = -0.65 to -0.92)
        add_quad(
            &mut chassis_mesh,
            Vec3::new(sign * half_w * 0.92, 0.30, -0.65),
            Vec3::new(sign * half_w * 0.88, 0.68, -0.65),
            Vec3::new(sign * 0.92, 0.70, -0.92),
            Vec3::new(rx, 0.30, -0.92),
            rnorm,
            AutomotivePalette::APEX_CHERRY_RED,
        );

        // 1g-ii. Sculpted Rear Wheel Arch Cutout (Over rear wheel: z = -0.92 to -1.58, arch apex y = 0.74)
        // Forward arch quadrant (z = -0.92 to -1.25)
        add_quad(
            &mut chassis_mesh,
            Vec3::new(rx, 0.30, -0.92),
            Vec3::new(sign * 0.92, 0.70, -0.92),
            Vec3::new(sign * 0.95, 0.75, -1.25),
            Vec3::new(rx, 0.74, -1.25), // Crest clears tire (0.74m > 0.66m)
            rnorm,
            AutomotivePalette::APEX_CHERRY_RED,
        );
        // Aft arch quadrant (z = -1.25 to -1.58)
        add_quad(
            &mut chassis_mesh,
            Vec3::new(rx, 0.74, -1.25),
            Vec3::new(sign * 0.95, 0.75, -1.25),
            Vec3::new(sign * 0.94, 0.72, -1.58),
            Vec3::new(rx, 0.30, -1.58),
            rnorm,
            AutomotivePalette::APEX_CHERRY_RED,
        );

        // 1g-iii. Aft Haunch Section (Behind rear wheel arch: z = -1.58 to -1.85)
        add_quad(
            &mut chassis_mesh,
            Vec3::new(rx, 0.30, -1.58),
            Vec3::new(sign * 0.94, 0.72, -1.58),
            Vec3::new(sign * 0.90, 0.72, -1.85),
            Vec3::new(rx, 0.30, -1.85),
            rnorm,
            AutomotivePalette::APEX_CHERRY_RED,
        );

        // 1g-iv. Matte Dark Inner Wheel Well Cavity Liner
        add_quad(
            &mut chassis_mesh,
            Vec3::new(inner_x, 0.72, -1.58),
            Vec3::new(rx, 0.74, -1.58),
            Vec3::new(rx, 0.74, -0.92),
            Vec3::new(inner_x, 0.72, -0.92),
            -Vec3::Y,
            wheel_liner_color,
        );
    }

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

    // 1j. Solid Rear Fascia & Embedded Bumper (Permanently eliminates floating rear components)
    // Upper fascia connecting decklid trailing edge to taillight shelf
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.75, 0.82, -half_l + 0.10),
        Vec3::new(0.75, 0.82, -half_l + 0.10),
        Vec3::new(0.76, 0.79, -half_l + 0.015),
        Vec3::new(-0.76, 0.79, -half_l + 0.015),
        -Vec3::Z,
        AutomotivePalette::APEX_CHERRY_RED,
    );
    // Mid rear bumper face (Left and Right wings beneath taillights)
    for sign in [-1.0f32, 1.0f32] {
        let x0 = sign * 0.22;
        let x1 = sign * 0.80;
        let (left_x, right_x) = if sign < 0.0 { (x1, x0) } else { (x0, x1) };
        add_quad(
            &mut chassis_mesh,
            Vec3::new(left_x, 0.44, -half_l + 0.015),
            Vec3::new(right_x, 0.44, -half_l + 0.015),
            Vec3::new(right_x, 0.74, -half_l + 0.015),
            Vec3::new(left_x, 0.74, -half_l + 0.015),
            -Vec3::Z,
            AutomotivePalette::APEX_CHERRY_RED,
        );
    }
    // Recessed center license plate cove
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.22, 0.44, -half_l + 0.035),
        Vec3::new(0.22, 0.44, -half_l + 0.035),
        Vec3::new(0.22, 0.72, -half_l + 0.035),
        Vec3::new(-0.22, 0.72, -half_l + 0.035),
        -Vec3::Z,
        [0.08, 0.08, 0.10, 1.0], // Dark composite recess
    );
    // Embossed registration license plate
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-0.18, 0.48, -half_l + 0.032),
        Vec3::new(0.18, 0.48, -half_l + 0.032),
        Vec3::new(0.18, 0.66, -half_l + 0.032),
        Vec3::new(-0.18, 0.66, -half_l + 0.032),
        -Vec3::Z,
        [0.92, 0.92, 0.95, 1.0], // White stamped plate
    );
    // Lower rear bumper apron connecting down to carbon diffuser
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-half_w * 0.85, 0.32, -half_l),
        Vec3::new(half_w * 0.85, 0.32, -half_l),
        Vec3::new(0.80, 0.44, -half_l + 0.015),
        Vec3::new(-0.80, 0.44, -half_l + 0.015),
        -Vec3::Z,
        AutomotivePalette::APEX_CHERRY_RED,
    );

    // 1k. Aerodynamic Side Mirrors (Mounted on Doors)
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
        let p0 = Vec3::new(rim_r * a0.cos(), rim_r * a0.sin(), 0.0);
        let p1 = Vec3::new(rim_r * a1.cos(), rim_r * a1.sin(), 0.0);
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
        Vec3::ZERO,
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
    // 6. Elevated Aerodynamic GT Rear Wing / Spoiler (3D Solid Carbon Geometry)
    // -------------------------------------------------------------------------
    let mut spoiler_mesh = ChunkMesh::default();
    let wing_z = -1.95;
    let wing_y = 1.05;
    let wing_span = 0.85;

    // Dual 3D solid carbon fiber mounting stanchions (with authentic thickness)
    for sign in [-1.0f32, 1.0f32] {
        let sx = sign * 0.42;
        add_box_at(
            &mut spoiler_mesh,
            Vec3::new(sx, (0.82 + wing_y) * 0.5, wing_z),
            Vec3::new(0.02, (wing_y - 0.82) * 0.5, 0.07),
            AutomotivePalette::CARBON_FIBER,
        );
    }
    // Main 3D enclosed carbon fiber airfoil wing element
    add_box_at(
        &mut spoiler_mesh,
        Vec3::new(0.0, wing_y + 0.015, wing_z),
        Vec3::new(wing_span, 0.018, 0.13),
        AutomotivePalette::CARBON_FIBER,
    );
    // Trailing edge Gurney flap lip for high aerodynamic downforce
    add_box_at(
        &mut spoiler_mesh,
        Vec3::new(0.0, wing_y + 0.035, wing_z - 0.125),
        Vec3::new(wing_span, 0.015, 0.01),
        AutomotivePalette::CARBON_FIBER,
    );
    // Dual 3D aerodynamic endplates
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
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::CARBON_FIBER,
            metallic: 0.90,
            roughness: 0.20,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // -------------------------------------------------------------------------
    // 7. Quad Polished Stainless Steel Exhaust System (Longitudinal Z-Axis)
    // -------------------------------------------------------------------------
    let mut exhaust_mesh = ChunkMesh::default();
    for ex_x in [-0.72, -0.60, 0.60, 0.72] {
        add_cylinder_z(
            &mut exhaust_mesh,
            Vec3::new(ex_x, 0.28, -half_l + 0.01),
            0.045,
            0.05,
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
    // 8. Dynamic Amber Turn Signal Indicator Modules (Front & Rear Clusters)
    // -------------------------------------------------------------------------
    let mut turn_signal_mesh = ChunkMesh::default();
    // Front amber indicator modules integrated into outer headlight brows
    for sign in [-1.0f32, 1.0f32] {
        let tx = sign * 0.78;
        let ty = 0.52;
        let tz = half_l - 0.13;
        add_quad(
            &mut turn_signal_mesh,
            Vec3::new(tx - sign * 0.03, ty - 0.035, tz),
            Vec3::new(tx + sign * 0.03, ty - 0.035, tz - 0.04),
            Vec3::new(tx + sign * 0.03, ty + 0.035, tz - 0.04),
            Vec3::new(tx - sign * 0.03, ty + 0.035, tz),
            Vec3::Z,
            AutomotivePalette::TURN_SIGNAL_AMBER,
        );
    }
    // Rear amber indicator modules integrated into outer taillight bar
    for sign in [-1.0f32, 1.0f32] {
        let rx = sign * 0.72;
        add_quad(
            &mut turn_signal_mesh,
            Vec3::new(rx - 0.04, 0.74, -half_l + 0.018),
            Vec3::new(rx + 0.04, 0.74, -half_l + 0.018),
            Vec3::new(rx + 0.04, 0.79, -half_l + 0.018),
            Vec3::new(rx - 0.04, 0.79, -half_l + 0.018),
            -Vec3::Z,
            AutomotivePalette::TURN_SIGNAL_AMBER,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "TurnSignals".into(),
        submesh_type: VehicleSubmeshType::TurnSignals,
        mesh: turn_signal_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::TURN_SIGNAL_AMBER,
            metallic: 0.10,
            roughness: 0.10,
            emissive: [1.0, 0.65, 0.05],
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
            chase_target: Vec3::new(0.0, 2.00, -3.0),
            chase_distance: 26.0,
            chase_height: 5.20,
            cockpit_eye: Vec3::new(-0.75, 2.30, 0.80),
            cockpit_look: Vec3::new(-0.75, 2.30, 4.5),
            hood_eye: Vec3::new(0.0, 2.05, 2.60),
            orbit_pivot: Vec3::new(0.0, 1.50, 0.0),
        });

    let mut chassis_mesh = ChunkMesh::default();
    let half_w = 1.27;

    // 1. Heavy Conventional Sleeper Cab & Sloped Hood
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.95, 0.50),
        Vec3::new(half_w, 1.25, 1.30),
        AutomotivePalette::TRUCK_CRIMSON_HAULER,
    );
    // Aerodynamic Sleeper Roof Fairing
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 3.40, 0.30),
        Vec3::new(half_w * 0.95, 0.25, 1.05),
        AutomotivePalette::TRUCK_CRIMSON_HAULER,
    );
    // Long Sloped Engine Hood
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.35, 2.45),
        Vec3::new(0.95, 0.65, 0.85),
        AutomotivePalette::TRUCK_CRIMSON_HAULER,
    );
    // Massive Chrome Grille Surround with Vertical Cooling Slats
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.35, 3.32),
        Vec3::new(0.88, 0.60, 0.04),
        AutomotivePalette::POLISHED_CHROME,
    );
    // Heavy Aerodynamic Front Bumper with Fog Lamps
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.52, 3.25),
        Vec3::new(half_w * 1.02, 0.22, 0.15),
        AutomotivePalette::POLISHED_CHROME,
    );
    // Chrome Dual Vertical Exhaust Stacks with Heat Shields
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_y(
            &mut chassis_mesh,
            Vec3::new(sign * 1.15, 2.75, -0.75),
            0.08,
            1.10,
            12,
            AutomotivePalette::POLISHED_CHROME,
        );
    }
    // Heavy Chassis Frame Rails & Fifth-Wheel Coupling Plate
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.85, -0.60),
        Vec3::new(0.55, 0.12, 1.80),
        [0.18, 0.20, 0.22, 1.0],
    );
    add_cylinder_y(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.98, -1.05),
        0.35,
        0.04,
        14,
        [0.12, 0.12, 0.14, 1.0], // Fifth wheel plate
    );
    // Dual Cylindrical Aluminum Fuel Tanks with Serrated Steps
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_z(
            &mut chassis_mesh,
            Vec3::new(sign * 1.05, 0.68, 0.80),
            0.32,
            0.65,
            14,
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

    // 2. Windshield & Sleeper Glass
    let mut glass_mesh = ChunkMesh::default();
    add_quad(
        &mut glass_mesh,
        Vec3::new(-half_w * 0.92, 2.05, 1.76),
        Vec3::new(half_w * 0.92, 2.05, 1.76),
        Vec3::new(half_w * 0.90, 2.85, 1.62),
        Vec3::new(-half_w * 0.90, 2.85, 1.62),
        Vec3::new(0.0, 0.25, 0.95).normalize(),
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

    // 3. Articulated 53ft Dry Van Freight Trailer
    let mut trailer_mesh = ChunkMesh::default();
    let tr_l = 7.5;
    let tr_w = 1.25;
    let tr_h = 1.45;
    let tr_pivot = Vec3::new(0.0, 0.98, -1.05);
    let rel_center = Vec3::new(0.0, (1.10 + tr_h) - tr_pivot.y, -tr_l);
    // Main container body
    add_box_at(
        &mut trailer_mesh,
        rel_center,
        Vec3::new(tr_w, tr_h, tr_l),
        [0.90, 0.92, 0.95, 1.0], // Clean white aluminum freight trailer
    );
    // Aerodynamic side skirts
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut trailer_mesh,
            Vec3::new(
                sign * (tr_w - 0.05),
                rel_center.y - tr_h * 0.65,
                rel_center.z,
            ),
            Vec3::new(0.02, 0.35, tr_l * 0.65),
            [0.15, 0.15, 0.18, 1.0],
        );
    }
    // Rear ICC crash bar and mudflaps
    add_box_at(
        &mut trailer_mesh,
        Vec3::new(0.0, 0.55 - tr_pivot.y, rel_center.z - tr_l + 0.05),
        Vec3::new(tr_w * 0.95, 0.08, 0.05),
        AutomotivePalette::POLISHED_CHROME,
    );
    // Trailer dual tandem wheels
    let trailer_wheel_r = 0.50;
    for az in [-tr_l * 0.65, -tr_l * 0.85] {
        for sign in [-1.0f32, 1.0f32] {
            add_cylinder_x(
                &mut trailer_mesh,
                Vec3::new(
                    sign * (tr_w - 0.18),
                    trailer_wheel_r - tr_pivot.y,
                    rel_center.z + az,
                ),
                trailer_wheel_r,
                0.22,
                14,
                AutomotivePalette::TIRE_RUBBER,
            );
        }
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Articulated_Trailer".into(),
        submesh_type: VehicleSubmeshType::TruckArticulatedTrailer,
        mesh: trailer_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: tr_pivot, // Articulates around fifth-wheel kingpin
        material: VehiclePbrMaterial {
            base_color: [0.90, 0.92, 0.95, 1.0],
            metallic: 0.40,
            roughness: 0.35,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 4. Heavy Truck Steer and Drive Wheels
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

    // 5. Headlights and Brakelights
    let mut hl_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_z(
            &mut hl_mesh,
            Vec3::new(sign * 0.98, 0.72, 3.32),
            0.12,
            0.02,
            12,
            AutomotivePalette::HEADLIGHT_XENON_ON,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Headlights".into(),
        submesh_type: VehicleSubmeshType::Headlights,
        mesh: hl_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::HEADLIGHT_XENON_ON,
            metallic: 0.10,
            roughness: 0.10,
            emissive: [1.0, 0.98, 0.90],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    builder.build()
}

// =============================================================================
// CityLine Transit Bus — High-Fidelity 40ft Low-Floor Urban Transit Bus
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

    let half_w = 1.25;
    let half_l = 5.90;
    let mut chassis_mesh = ChunkMesh::default();

    // 1. Aerodynamic Front Cap, Bumper & Lower Skirts
    // Lower body transit skirt panels in transit green
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.85, 0.0),
        Vec3::new(half_w, 0.55, half_l),
        AutomotivePalette::BUS_TRANSIT_GREEN,
    );
    // Aerodynamic curved front bumper fascia with fog lights
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.52, half_l - 0.05),
        Vec3::new(half_w * 0.98, 0.22, 0.15),
        [0.15, 0.16, 0.18, 1.0], // Dark charcoal bumper
    );
    // Front Destination Header Box above windshield with amber LED sign
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 2.85, half_l - 0.08),
        Vec3::new(half_w * 0.95, 0.22, 0.16),
        [0.12, 0.14, 0.16, 1.0],
    );
    add_quad(
        &mut chassis_mesh,
        Vec3::new(-1.05, 2.74, half_l + 0.08),
        Vec3::new(1.05, 2.74, half_l + 0.08),
        Vec3::new(1.05, 2.96, half_l + 0.08),
        Vec3::new(-1.05, 2.96, half_l + 0.08),
        Vec3::Z,
        [1.0, 0.75, 0.10, 1.0], // Amber LED destination matrix
    );
    // Vertical Black Window Pillars along passenger saloon
    for pz in [-4.2, -2.8, -1.4, 0.0, 1.4, 2.8, 4.2] {
        for sign in [-1.0f32, 1.0f32] {
            add_box_at(
                &mut chassis_mesh,
                Vec3::new(sign * (half_w - 0.02), 2.05, pz),
                Vec3::new(0.02, 0.65, 0.06),
                [0.08, 0.08, 0.10, 1.0],
            );
        }
    }
    // Streamlined White Fiberglass Roof Cap with Curved Cant-rails
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 2.95, 0.0),
        Vec3::new(half_w * 0.98, 0.15, half_l),
        [0.92, 0.94, 0.96, 1.0], // Crisp white roof
    );
    // Rooftop Dual HVAC Air Conditioning Units
    for hz in [-2.5f32, 2.0f32] {
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(0.0, 3.25, hz),
            Vec3::new(0.85, 0.16, 1.10),
            [0.85, 0.88, 0.92, 1.0],
        );
    }
    // Rooftop Aerodynamic CNG Fuel Tank Enclosure Pod
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 3.32, -0.20),
        Vec3::new(0.92, 0.22, 1.60),
        [0.92, 0.94, 0.96, 1.0],
    );
    // High-Mounted Aerodynamic Transit Mirrors on Tubular Arms
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_y(
            &mut chassis_mesh,
            Vec3::new(sign * (half_w + 0.18), 2.10, half_l - 0.35),
            0.025,
            0.35,
            8,
            [0.15, 0.15, 0.15, 1.0],
        );
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * (half_w + 0.22), 2.15, half_l - 0.35),
            Vec3::new(0.06, 0.20, 0.12),
            [0.18, 0.20, 0.22, 1.0],
        );
    }
    // Rear Engine Ventilation Louvers and Rear Bumper
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.25, -half_l + 0.02),
        Vec3::new(0.95, 0.55, 0.04),
        [0.15, 0.15, 0.18, 1.0],
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

    // 2. Tinted Panoramic Glass (Windshield, Side Windows, Rear Window)
    let mut glass_mesh = ChunkMesh::default();
    // Front curved panoramic windshield raked at 54 degrees
    add_quad(
        &mut glass_mesh,
        Vec3::new(-half_w * 0.95, 1.35, half_l - 0.05),
        Vec3::new(half_w * 0.95, 1.35, half_l - 0.05),
        Vec3::new(half_w * 0.95, 2.70, half_l - 0.35),
        Vec3::new(-half_w * 0.95, 2.70, half_l - 0.35),
        Vec3::new(0.0, 0.28, 0.96).normalize(),
        AutomotivePalette::TINTED_GLASS,
    );
    // Panoramic side window ribbons (Left & Right)
    for sign in [-1.0f32, 1.0f32] {
        let x = sign * (half_w - 0.01);
        let n = Vec3::new(sign, 0.0, 0.0);
        add_quad(
            &mut glass_mesh,
            Vec3::new(x, 1.40, -half_l * 0.92),
            Vec3::new(x, 1.40, half_l * 0.88),
            Vec3::new(x, 2.70, half_l * 0.88),
            Vec3::new(x, 2.70, -half_l * 0.92),
            n,
            AutomotivePalette::TINTED_GLASS,
        );
    }
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

    // 3. Passenger Saloon & Driver Cabin Interior
    let mut interior_mesh = ChunkMesh::default();
    // Non-slip low floor walkway
    add_box_at(
        &mut interior_mesh,
        Vec3::new(0.0, 0.42, 0.0),
        Vec3::new(half_w * 0.92, 0.05, half_l * 0.95),
        [0.22, 0.24, 0.26, 1.0],
    );
    // Driver cabin console binnacle & seat
    add_box_at(
        &mut interior_mesh,
        Vec3::new(-0.75, 1.05, half_l - 1.10),
        Vec3::new(0.35, 0.35, 0.45),
        AutomotivePalette::INTERIOR_CHARCOAL,
    );
    add_box_at(
        &mut interior_mesh,
        Vec3::new(-0.75, 0.95, half_l - 1.65),
        Vec3::new(0.24, 0.45, 0.24),
        [0.15, 0.18, 0.22, 1.0], // Driver high-back seat
    );
    // Stainless Steel Vertical Stanchion Grab Poles
    for sz in [-3.8, -1.8, 1.2, 3.2] {
        add_cylinder_y(
            &mut interior_mesh,
            Vec3::new(0.35, 1.55, sz),
            0.02,
            1.10,
            8,
            AutomotivePalette::POLISHED_CHROME,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Interior_Cockpit".into(),
        submesh_type: VehicleSubmeshType::InteriorCockpit,
        mesh: interior_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial::default(),
    });

    // 16 Contoured Modular Passenger Bucket Seats in Transit Blue
    let mut seats_mesh = ChunkMesh::default();
    for sz in [-4.5, -3.5, -2.5, -1.5, 0.8, 1.8, 2.8, 3.8] {
        for sx in [-0.78, 0.78] {
            add_box_at(
                &mut seats_mesh,
                Vec3::new(sx, 0.88, sz),
                Vec3::new(0.22, 0.40, 0.20),
                [0.18, 0.35, 0.65, 1.0], // Transit Blue
            );
        }
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Passenger_Seats".into(),
        submesh_type: VehicleSubmeshType::InteriorPassengerSeats,
        mesh: seats_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: [0.18, 0.35, 0.65, 1.0],
            metallic: 0.10,
            roughness: 0.70,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 4. Articulated Passenger Doors (Front Entrance & Center Exit)
    let mut front_door_mesh = ChunkMesh::default();
    add_box_at(
        &mut front_door_mesh,
        Vec3::ZERO,
        Vec3::new(0.03, 0.90, 0.55),
        [0.95, 0.85, 0.10, 1.0], // Yellow safety door edges
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Door_Front".into(),
        submesh_type: VehicleSubmeshType::BusDoorFront,
        mesh: front_door_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::new(half_w, 1.35, half_l - 1.40),
        material: VehiclePbrMaterial::default(),
    });

    let mut center_door_mesh = ChunkMesh::default();
    add_box_at(
        &mut center_door_mesh,
        Vec3::ZERO,
        Vec3::new(0.03, 0.90, 0.55),
        [0.95, 0.85, 0.10, 1.0],
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Door_Center".into(),
        submesh_type: VehicleSubmeshType::BusDoorCenter,
        mesh: center_door_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::new(half_w, 1.35, -0.40),
        material: VehiclePbrMaterial::default(),
    });

    // 5. Heavy-Duty Transit Bus Wheels
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

    // 6. Projector Headlights
    let mut hl_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_z(
            &mut hl_mesh,
            Vec3::new(sign * 0.95, 0.75, half_l + 0.02),
            0.12,
            0.02,
            12,
            AutomotivePalette::HEADLIGHT_XENON_ON,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Headlights".into(),
        submesh_type: VehicleSubmeshType::Headlights,
        mesh: hl_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::HEADLIGHT_XENON_ON,
            metallic: 0.10,
            roughness: 0.10,
            emissive: [1.0, 0.98, 0.90],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    builder.build()
}

// =============================================================================
// HyperSport RR-1000 — High-Fidelity 1000cc Supersport Motorcycle
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

    // 1. Aerodynamic MotoGP-style Front Fairing with Cat-Eye Lights & Winglets
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.78, 0.50),
        Vec3::new(0.20, 0.16, 0.28),
        AutomotivePalette::BIKE_RACING_ORANGE,
    );
    // Aerodynamic downforce winglets (Left & Right)
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * 0.26, 0.74, 0.48),
            Vec3::new(0.06, 0.015, 0.12),
            AutomotivePalette::CARBON_FIBER,
        );
    }
    // 2. Sculpted 17L Fuel Tank with Ergonomic Knee Recesses
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.82, 0.05),
        Vec3::new(0.18, 0.18, 0.25),
        AutomotivePalette::BIKE_RACING_ORANGE,
    );
    // 3. Exposed Liquid-Cooled 998cc Inline-4 Engine & Aluminum Trellis Frame
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.42, 0.0),
        Vec3::new(0.16, 0.16, 0.24),
        [0.15, 0.16, 0.18, 1.0], // Black crankcases
    );
    // 4 Curved Stainless Exhaust Headers merging to collector
    for sx in [-0.09, -0.03, 0.03, 0.09] {
        add_cylinder_y(
            &mut chassis_mesh,
            Vec3::new(sx, 0.32, 0.24),
            0.02,
            0.12,
            8,
            AutomotivePalette::POLISHED_CHROME,
        );
    }
    // High-Exit Titanium Exhaust Canister
    add_cylinder_z(
        &mut chassis_mesh,
        Vec3::new(0.18, 0.48, -0.45),
        0.055,
        0.28,
        12,
        AutomotivePalette::POLISHED_CHROME,
    );
    // 4. Split-Level Rider Saddle & Upswept Aerodynamic Tail Cowl
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.70, -0.22),
        Vec3::new(0.14, 0.04, 0.18),
        AutomotivePalette::INTERIOR_CHARCOAL, // Rider seat
    );
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.76, -0.52),
        Vec3::new(0.11, 0.08, 0.22),
        AutomotivePalette::BIKE_RACING_ORANGE, // Tail cowl
    );
    // 5. Inverted Telescopic Front Forks (Gold Anodized) & Triple Clamp
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_y(
            &mut chassis_mesh,
            Vec3::new(sign * 0.10, 0.52, 0.62),
            0.025,
            0.28,
            10,
            [0.85, 0.70, 0.15, 1.0], // Gold anodized forks
        );
    }
    // Clip-on Handlebars with Bar-End Mirrors
    add_cylinder_x(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.84, 0.42),
        0.015,
        0.28,
        8,
        AutomotivePalette::INTERIOR_CHARCOAL,
    );
    // 6. Cast Aluminum Gull-Wing Rear Swingarm
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * 0.10, 0.35, -0.35),
            Vec3::new(0.025, 0.05, 0.35),
            AutomotivePalette::ALLOY_SILVER,
        );
    }

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Motorcycle_Body".into(),
        submesh_type: VehicleSubmeshType::MotorcycleLeanBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::new(0.0, 0.31, 0.0), // Banks into turns around ground roll axis
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::BIKE_RACING_ORANGE,
            metallic: 0.85,
            roughness: 0.20,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 7. Clear Aerodynamic Bubble Windscreen
    let mut glass_mesh = ChunkMesh::default();
    add_quad(
        &mut glass_mesh,
        Vec3::new(-0.14, 0.82, 0.48),
        Vec3::new(0.14, 0.82, 0.48),
        Vec3::new(0.10, 1.02, 0.32),
        Vec3::new(-0.10, 1.02, 0.32),
        Vec3::new(0.0, 0.50, 0.86).normalize(),
        [0.85, 0.90, 0.95, 0.40],
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Windscreen".into(),
        submesh_type: VehicleSubmeshType::GlassGreenhouse,
        mesh: glass_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: [0.85, 0.90, 0.95, 0.40],
            metallic: 0.10,
            roughness: 0.05,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Blend,
        },
    });

    // 8. Front and Rear Superbike Wheels
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
// Pacific Locomotive — 4,000 HP Diesel-Electric Passenger/Freight Locomotive
// =============================================================================

pub fn build_pacific_locomotive(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("pacific_locomotive", "Pacific Locomotive")
        .with_physics(11.20, 1.435, 0.55, 130000.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 2.40, 0.0),
            chase_distance: 28.0,
            chase_height: 6.50,
            cockpit_eye: Vec3::new(-0.85, 3.20, 6.80),
            cockpit_look: Vec3::new(-0.85, 3.20, 15.0),
            hood_eye: Vec3::new(0.0, 2.20, 9.20),
            orbit_pivot: Vec3::new(0.0, 2.40, 0.0),
        });

    let half_w = 1.575;
    let half_l = 9.25;
    let mut chassis_mesh = ChunkMesh::default();

    // 1. Heavy Cast Steel Underframe Platform Sill
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.15, 0.0),
        Vec3::new(half_w, 0.15, half_l),
        [0.18, 0.20, 0.22, 1.0], // Deep graphite underframe steel
    );
    // 2. Streamlined Comfort Cab & Collision Deflection Nose
    // Forward Collision Nose
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 2.10, 7.80),
        Vec3::new(half_w * 0.95, 0.80, 1.25),
        AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
    );
    // High Isolated Crew Cab
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 2.85, 5.80),
        Vec3::new(half_w * 0.98, 1.55, 1.25),
        AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
    );
    // Rooftop Cyclops Headlight Binnacle & 5-Chime Nathan Air Horn
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 4.45, 6.50),
        Vec3::new(0.35, 0.15, 0.45),
        [0.22, 0.24, 0.28, 1.0],
    );
    // Cast Steel Pilot (Snowplow / Cowcatcher) with Ditch Light Housings
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.55, half_l - 0.15),
        Vec3::new(half_w * 0.95, 0.35, 0.18),
        [0.15, 0.16, 0.18, 1.0],
    );
    // Knuckle Coupler and Pin Lifter Drawbar (Front and Rear)
    let mut coupler_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut coupler_mesh,
            Vec3::new(0.0, 0.85, sign * (half_l + 0.35)),
            Vec3::new(0.18, 0.18, 0.35),
            [0.25, 0.26, 0.28, 1.0], // AAR Type E Knuckle Coupler
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Locomotive_Couplers".into(),
        submesh_type: VehicleSubmeshType::TrainCoupler,
        mesh: coupler_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: [0.25, 0.26, 0.28, 1.0],
            metallic: 0.90,
            roughness: 0.35,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });
    // Long Machinery Hood with Dynamic Brake Blister & Exhaust Stack
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 2.65, -0.80),
        Vec3::new(half_w * 0.88, 1.35, 5.50),
        AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
    );
    // Rooftop Exhaust Silencer Stack
    add_cylinder_y(
        &mut chassis_mesh,
        Vec3::new(0.0, 4.25, 1.80),
        0.28,
        0.22,
        14,
        [0.15, 0.15, 0.16, 1.0],
    );
    // Massive 2,200-Gallon Belly Diesel Fuel Tank and Air Reservoirs
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.65, 0.0),
        Vec3::new(half_w * 0.82, 0.35, 2.80),
        [0.12, 0.14, 0.16, 1.0],
    );
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_z(
            &mut chassis_mesh,
            Vec3::new(sign * 1.05, 0.85, 0.0),
            0.18,
            2.20,
            12,
            [0.25, 0.28, 0.32, 1.0], // Main reservoir tanks
        );
    }
    // Full Perimeter Safety Handrails along Walkways (Yellow Safety Steel)
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * (half_w - 0.05), 1.85, -0.80),
            Vec3::new(0.02, 0.50, 5.50),
            [0.95, 0.80, 0.10, 1.0], // Yellow handrails
        );
    }

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Locomotive_Chassis".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::SEDAN_SAPPHIRE_BLUE,
            metallic: 0.70,
            roughness: 0.30,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 2. Cab Windshields & Side Windows
    let mut glass_mesh = ChunkMesh::default();
    add_quad(
        &mut glass_mesh,
        Vec3::new(-half_w * 0.85, 3.10, 7.02),
        Vec3::new(half_w * 0.85, 3.10, 7.02),
        Vec3::new(half_w * 0.80, 3.85, 6.75),
        Vec3::new(-half_w * 0.80, 3.85, 6.75),
        Vec3::new(0.0, 0.35, 0.94).normalize(),
        AutomotivePalette::TINTED_GLASS,
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Cab_Glass".into(),
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

    // 3. Track-Articulated Railway Bogies (Front & Rear)
    let bogie_wb = 2.80;
    let rail_wheel_r = 0.55;
    let gauge_half = 1.435 * 0.5;

    let f_bogie = build_railway_bogie(bogie_wb, rail_wheel_r, gauge_half, lod);
    let f_bogie_pivot = Vec3::new(0.0, 0.0, 5.60);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Bogie_Front".into(),
        submesh_type: VehicleSubmeshType::TrainBogieFront,
        mesh: f_bogie,
        local_transform: Affine3A::from_translation(f_bogie_pivot),
        pivot: f_bogie_pivot, // Pivots around kingpin with track curvature
        material: VehiclePbrMaterial::default(),
    });

    let r_bogie = build_railway_bogie(bogie_wb, rail_wheel_r, gauge_half, lod);
    let r_bogie_pivot = Vec3::new(0.0, 0.0, -5.60);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Bogie_Rear".into(),
        submesh_type: VehicleSubmeshType::TrainBogieRear,
        mesh: r_bogie,
        local_transform: Affine3A::from_translation(r_bogie_pivot),
        pivot: r_bogie_pivot,
        material: VehiclePbrMaterial::default(),
    });

    // 4. Sealed-Beam High-Intensity Cyclops Headlight & Ditch Lights
    let mut hl_mesh = ChunkMesh::default();
    add_cylinder_z(
        &mut hl_mesh,
        Vec3::new(0.0, 4.45, 6.72),
        0.18,
        0.02,
        14,
        AutomotivePalette::HEADLIGHT_XENON_ON,
    );
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_z(
            &mut hl_mesh,
            Vec3::new(sign * 0.85, 0.85, half_l + 0.02),
            0.10,
            0.02,
            10,
            AutomotivePalette::HEADLIGHT_XENON_ON,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Locomotive_Headlights".into(),
        submesh_type: VehicleSubmeshType::Headlights,
        mesh: hl_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: AutomotivePalette::HEADLIGHT_XENON_ON,
            metallic: 0.10,
            roughness: 0.10,
            emissive: [1.0, 0.98, 0.90],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    builder.build()
}

// =============================================================================
// Bombardier BiLevel Passenger Coach — Commuter Rail Car
// =============================================================================

pub fn build_passenger_coach(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("passenger_coach", "Passenger Coach")
        .with_physics(17.60, 1.435, 0.46, 55375.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 2.40, 0.0),
            chase_distance: 32.0,
            chase_height: 7.00,
            cockpit_eye: Vec3::new(0.0, 3.50, 0.0),
            cockpit_look: Vec3::new(0.0, 3.50, 10.0),
            hood_eye: Vec3::new(0.0, 2.0, 12.0),
            orbit_pivot: Vec3::new(0.0, 2.40, 0.0),
        });

    let half_w = 1.50;
    let half_l = 12.95;
    let mut chassis_mesh = ChunkMesh::default();
    let stainless_steel = [0.82, 0.85, 0.88, 1.0];

    // 1. BiLevel Octagonal Profile Passenger Carbody
    // Lower Commuter Level
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.85, 0.0),
        Vec3::new(half_w, 0.85, half_l),
        stainless_steel,
    );
    // Upper Observation Level with Angled Cant-Rails
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 3.55, 0.0),
        Vec3::new(half_w * 0.92, 0.85, half_l),
        stainless_steel,
    );
    // End Vestibule Walk-Through Bellows Diaphragms & Knuckle Couplers
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(0.0, 2.40, sign * (half_l + 0.15)),
            Vec3::new(0.65, 1.20, 0.15),
            [0.15, 0.15, 0.16, 1.0], // Flexible rubber diaphragm
        );
    }

    let mut coach_coupler_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut coach_coupler_mesh,
            Vec3::new(0.0, 0.85, sign * (half_l + 0.35)),
            Vec3::new(0.18, 0.18, 0.35),
            [0.25, 0.26, 0.28, 1.0], // Knuckle Coupler
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Coach_Couplers".into(),
        submesh_type: VehicleSubmeshType::TrainCoupler,
        mesh: coach_coupler_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: [0.25, 0.26, 0.28, 1.0],
            metallic: 0.90,
            roughness: 0.35,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Coach_Chassis".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: stainless_steel,
            metallic: 0.90,
            roughness: 0.20,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 2. Dual-Level Tinted Passenger Windows
    let mut glass_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        let x = sign * (half_w + 0.01);
        let n = Vec3::new(sign, 0.0, 0.0);
        // Lower level window ribbon
        add_quad(
            &mut glass_mesh,
            Vec3::new(x, 1.45, -half_l * 0.85),
            Vec3::new(x, 1.45, half_l * 0.85),
            Vec3::new(x, 2.25, half_l * 0.85),
            Vec3::new(x, 2.25, -half_l * 0.85),
            n,
            AutomotivePalette::TINTED_GLASS,
        );
        // Upper level window ribbon
        add_quad(
            &mut glass_mesh,
            Vec3::new(x * 0.92, 3.15, -half_l * 0.85),
            Vec3::new(x * 0.92, 3.15, half_l * 0.85),
            Vec3::new(x * 0.92, 3.95, half_l * 0.85),
            Vec3::new(x * 0.92, 3.95, -half_l * 0.85),
            n,
            AutomotivePalette::TINTED_GLASS,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Coach_Glass".into(),
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

    // 3. High-Speed 2-Axle Passenger Bogies
    let bogie_wb = 2.60;
    let rail_wheel_r = 0.46;
    let gauge_half = 1.435 * 0.5;

    let f_bogie = build_railway_bogie(bogie_wb, rail_wheel_r, gauge_half, lod);
    let f_bogie_pivot = Vec3::new(0.0, 0.0, 8.80);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Coach_Bogie_Front".into(),
        submesh_type: VehicleSubmeshType::TrainBogieFront,
        mesh: f_bogie,
        local_transform: Affine3A::from_translation(f_bogie_pivot),
        pivot: f_bogie_pivot,
        material: VehiclePbrMaterial::default(),
    });

    let r_bogie = build_railway_bogie(bogie_wb, rail_wheel_r, gauge_half, lod);
    let r_bogie_pivot = Vec3::new(0.0, 0.0, -8.80);
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Coach_Bogie_Rear".into(),
        submesh_type: VehicleSubmeshType::TrainBogieRear,
        mesh: r_bogie,
        local_transform: Affine3A::from_translation(r_bogie_pivot),
        pivot: r_bogie_pivot,
        material: VehiclePbrMaterial::default(),
    });

    builder.build()
}

// =============================================================================
// Golden Gate Ferry — High-Speed Catamaran Passenger Ferry
// =============================================================================

pub fn build_golden_gate_ferry(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("golden_gate_ferry", "Golden Gate Ferry")
        .with_physics(22.0, 8.50, 0.0, 180000.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 4.50, 0.0),
            chase_distance: 85.0,
            chase_height: 22.0,
            cockpit_eye: Vec3::new(0.0, 7.80, 8.50),
            cockpit_look: Vec3::new(0.0, 7.80, 25.0),
            hood_eye: Vec3::new(0.0, 3.50, 20.0),
            orbit_pivot: Vec3::new(0.0, 4.50, 0.0),
        });

    let mut chassis_mesh = ChunkMesh::default();
    let hull_white = [0.95, 0.95, 0.98, 1.0];
    let hull_navy = [0.10, 0.18, 0.32, 1.0];

    // 1. Twin Catamaran Sponsons with Razor Wave-Piercing Bows
    let sponson_x = 4.25;
    for sign in [-1.0f32, 1.0f32] {
        let sx = sign * sponson_x;
        // Main sponson hull
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sx, 1.80, 0.0),
            Vec3::new(1.10, 1.20, 19.5),
            hull_navy,
        );
        // Razor wave-piercing knife bow
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sx, 2.10, 19.5),
            Vec3::new(0.70, 0.90, 2.5),
            hull_white,
        );
    }
    // 2. Central Tunnel Deck Bridge & Main Passenger Saloon
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 3.40, -1.5),
        Vec3::new(5.20, 0.40, 16.0),
        hull_white,
    );
    // Enclosed Main Passenger Saloon
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 5.00, -1.0),
        Vec3::new(4.80, 1.20, 13.5),
        hull_white,
    );
    // Upper Promenade Observation Sun Deck & Bridge Wheelhouse
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 6.85, 4.5),
        Vec3::new(3.80, 0.65, 3.5),
        hull_white,
    );

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Ferry_Hull".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: hull_white,
            metallic: 0.30,
            roughness: 0.35,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 3. Panoramic Ocean View Windows
    let mut glass_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        add_quad(
            &mut glass_mesh,
            Vec3::new(sign * 4.82, 4.20, -13.0),
            Vec3::new(sign * 4.82, 4.20, 11.5),
            Vec3::new(sign * 4.82, 5.80, 11.5),
            Vec3::new(sign * 4.82, 5.80, -13.0),
            Vec3::new(sign, 0.0, 0.0),
            AutomotivePalette::TINTED_GLASS,
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Ferry_Glass".into(),
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

    // 4. Rotating Marine Radar Scanner Mast
    let mut radar_mesh = ChunkMesh::default();
    add_cylinder_y(
        &mut radar_mesh,
        Vec3::new(0.0, -0.42, 0.0),
        0.08,
        0.42,
        8,
        AutomotivePalette::ALLOY_SILVER,
    );
    add_box_at(
        &mut radar_mesh,
        Vec3::ZERO,
        Vec3::new(1.10, 0.06, 0.12),
        [0.98, 0.98, 1.0, 1.0], // Open-array radar scanner
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Radar_Mast".into(),
        submesh_type: VehicleSubmeshType::MarineRadarMast,
        mesh: radar_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::new(0.0, 9.05, 4.5),
        material: VehiclePbrMaterial::default(),
    });

    // 5. High-Velocity Stern Waterjets
    let mut wj_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_z(
            &mut wj_mesh,
            Vec3::new(sign * sponson_x, 0.0, 0.0),
            0.45,
            0.45,
            14,
            [0.25, 0.28, 0.32, 1.0],
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Waterjets".into(),
        submesh_type: VehicleSubmeshType::MarineWaterjets,
        mesh: wj_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::new(0.0, 1.40, -19.8),
        material: VehiclePbrMaterial::default(),
    });

    builder.build()
}

// =============================================================================
// SkyCourier B737 — Twin-Turbofan Narrow-Body Transport Airliner
// =============================================================================

pub fn build_skycourier_b737(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("skycourier_b737", "SkyCourier B737")
        .with_physics(14.80, 5.72, 0.55, 41000.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 3.50, 0.0),
            chase_distance: 55.0,
            chase_height: 14.0,
            cockpit_eye: Vec3::new(-0.50, 2.40, 15.5),
            cockpit_look: Vec3::new(-0.50, 2.40, 25.0),
            hood_eye: Vec3::new(0.0, 2.0, 18.5),
            orbit_pivot: Vec3::new(0.0, 3.50, 0.0),
        });

    let mut chassis_mesh = ChunkMesh::default();
    let airliner_white = [0.96, 0.96, 0.98, 1.0];

    // 1. Aerodynamic Cylindrical Fuselage with Nose Radome Cone
    add_cylinder_z(
        &mut chassis_mesh,
        Vec3::new(0.0, 2.20, 0.0),
        1.85,
        15.5,
        18,
        airliner_white,
    );
    // Nose Radome
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.95, 16.8),
        Vec3::new(1.20, 1.10, 1.5),
        airliner_white,
    );
    // 2. Swept Low Wings (25 degree sweep, wingspan 34.3m) with Blended Winglets
    for sign in [-1.0f32, 1.0f32] {
        let span_half = 17.15;
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * (span_half * 0.5), 1.65, -0.80),
            Vec3::new(span_half * 0.5, 0.15, 2.20),
            [0.88, 0.90, 0.94, 1.0],
        );
        // Blended vertical winglets
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * span_half, 2.65, -1.80),
            Vec3::new(0.08, 0.95, 0.65),
            [0.10, 0.35, 0.85, 1.0], // Airline blue winglet
        );
    }
    // 3. Swept Vertical Stabilizer Tail Fin and Horizontal Tailplanes
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 6.20, -14.2),
        Vec3::new(0.20, 2.80, 2.80),
        [0.10, 0.35, 0.85, 1.0], // Airline livery tail fin
    );
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * 3.80, 2.80, -15.5),
            Vec3::new(3.80, 0.12, 1.40),
            airliner_white,
        );
    }
    // 4. Twin Underwing High-Bypass CFM Turbofan Engine Nacelles
    for sign in [-1.0f32, 1.0f32] {
        let ex = sign * 4.80;
        add_cylinder_z(
            &mut chassis_mesh,
            Vec3::new(ex, 0.65, 2.80),
            1.05,
            1.85,
            16,
            airliner_white,
        );
    }

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Airliner_Airframe".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: airliner_white,
            metallic: 0.50,
            roughness: 0.25,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 5. Cockpit Windshield & Cabin Window Strips
    let mut glass_mesh = ChunkMesh::default();
    add_box_at(
        &mut glass_mesh,
        Vec3::new(0.0, 2.85, 16.0),
        Vec3::new(1.10, 0.45, 0.85),
        AutomotivePalette::TINTED_GLASS,
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Cockpit_Glass".into(),
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

    // 6. Tricycle Landing Gear (Nose Gear & Dual Main Gear)
    let mut gear_mesh = ChunkMesh::default();
    add_box_at(
        &mut gear_mesh,
        Vec3::new(0.0, 0.35, 14.5),
        Vec3::new(0.25, 0.35, 0.35),
        [0.15, 0.15, 0.16, 1.0],
    );
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut gear_mesh,
            Vec3::new(sign * 2.85, 0.35, -0.65),
            Vec3::new(0.35, 0.35, 0.45),
            [0.15, 0.15, 0.16, 1.0],
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Landing_Gear".into(),
        submesh_type: VehicleSubmeshType::WheelFrontLeft,
        mesh: gear_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: [0.15, 0.15, 0.16, 1.0],
            metallic: 0.70,
            roughness: 0.30,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 5. Spinning High-Bypass Turbofan Fan Blades
    let mut fan_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        let ex = sign * 4.80;
        add_cylinder_z(
            &mut fan_mesh,
            Vec3::new(ex, 0.0, 0.0),
            0.92,
            0.08,
            14,
            [0.20, 0.22, 0.25, 1.0], // Titanium fan blades
        );
    }
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Turbofan_Fans".into(),
        submesh_type: VehicleSubmeshType::AircraftTurbofanFan,
        mesh: fan_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::new(0.0, 0.65, 4.20),
        material: VehiclePbrMaterial::default(),
    });

    builder.build()
}

// =============================================================================
// Skyhawk C172 — General Aviation High-Wing Single-Engine Aircraft
// =============================================================================

pub fn build_skyhawk_c172(lod: VehicleLodLevel) -> LoadedVehicleModel {
    let mut builder = GltfAssetBuilder::new("skyhawk_c172", "Skyhawk C172")
        .with_physics(1.65, 2.22, 0.28, 760.0)
        .with_lod(lod)
        .with_camera_anchors(VehicleCameraAnchors {
            chase_target: Vec3::new(0.0, 1.20, 0.0),
            chase_distance: 14.0,
            chase_height: 3.50,
            cockpit_eye: Vec3::new(-0.35, 1.25, 0.20),
            cockpit_look: Vec3::new(-0.35, 1.25, 3.0),
            hood_eye: Vec3::new(0.0, 1.10, 1.80),
            orbit_pivot: Vec3::new(0.0, 1.20, 0.0),
        });

    let mut chassis_mesh = ChunkMesh::default();
    let plane_white = [0.95, 0.96, 0.98, 1.0];

    // 1. Fuselage and Cabin
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.10, 0.0),
        Vec3::new(0.60, 0.60, 2.20),
        plane_white,
    );
    // Forward Engine Cowl with Air Intakes
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.95, 2.65),
        Vec3::new(0.48, 0.42, 0.55),
        plane_white,
    );
    // 2. High Wings with Lift Struts (11m wingspan)
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.85, 0.20),
        Vec3::new(5.50, 0.08, 0.85),
        plane_white,
    );
    // Dual Wing Lift Struts
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * 2.20, 1.15, 0.20),
            Vec3::new(0.04, 0.45, 0.04),
            AutomotivePalette::ALLOY_SILVER,
        );
    }
    // 3. Empennage (Vertical Fin, Rudder, Horizontal Stabilizer)
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 2.15, -3.85),
        Vec3::new(0.08, 0.95, 0.95),
        [0.85, 0.15, 0.15, 1.0], // Red tail accent
    );
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 1.20, -4.10),
        Vec3::new(1.85, 0.06, 0.55),
        plane_white,
    );
    // 4. Fixed Tricycle Landing Gear with Wheel Pants (Fairings)
    for sign in [-1.0f32, 1.0f32] {
        add_box_at(
            &mut chassis_mesh,
            Vec3::new(sign * 1.11, 0.28, 0.0),
            Vec3::new(0.12, 0.25, 0.35),
            plane_white,
        );
    }
    add_box_at(
        &mut chassis_mesh,
        Vec3::new(0.0, 0.28, 1.65),
        Vec3::new(0.10, 0.25, 0.32),
        plane_white,
    );

    builder = builder.add_submesh(VehicleSubmesh {
        name: "Airframe".into(),
        submesh_type: VehicleSubmeshType::ChassisBody,
        mesh: chassis_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: plane_white,
            metallic: 0.40,
            roughness: 0.30,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // Cabin Windshield & Side Windows
    let mut glass_mesh = ChunkMesh::default();
    add_box_at(
        &mut glass_mesh,
        Vec3::new(0.0, 1.45, 0.45),
        Vec3::new(0.55, 0.35, 0.75),
        AutomotivePalette::TINTED_GLASS,
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Cabin_Glass".into(),
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

    // Tricycle Landing Gear Wheels
    let mut gear_mesh = ChunkMesh::default();
    for sign in [-1.0f32, 1.0f32] {
        add_cylinder_z(
            &mut gear_mesh,
            Vec3::new(sign * 1.11, 0.20, 0.0),
            0.18,
            0.10,
            12,
            [0.15, 0.15, 0.16, 1.0],
        );
    }
    add_cylinder_z(
        &mut gear_mesh,
        Vec3::new(0.0, 0.20, 1.65),
        0.16,
        0.08,
        12,
        [0.15, 0.15, 0.16, 1.0],
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Landing_Gear".into(),
        submesh_type: VehicleSubmeshType::WheelRearLeft,
        mesh: gear_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::ZERO,
        material: VehiclePbrMaterial {
            base_color: [0.15, 0.15, 0.16, 1.0],
            metallic: 0.80,
            roughness: 0.25,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        },
    });

    // 5. Spinning Dual-Blade Propeller
    let mut prop_mesh = ChunkMesh::default();
    add_cylinder_z(
        &mut prop_mesh,
        Vec3::ZERO,
        0.14,
        0.10,
        12,
        AutomotivePalette::POLISHED_CHROME, // Nose spinner cone
    );
    add_box_at(
        &mut prop_mesh,
        Vec3::new(0.0, 0.0, 0.03),
        Vec3::new(0.85, 0.05, 0.02),
        [0.15, 0.15, 0.16, 1.0], // Composite propeller blades
    );
    builder = builder.add_submesh(VehicleSubmesh {
        name: "Propeller".into(),
        submesh_type: VehicleSubmeshType::AircraftPropeller,
        mesh: prop_mesh,
        local_transform: Affine3A::IDENTITY,
        pivot: Vec3::new(0.0, 0.95, 3.25),
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
static SEDAN_LOD1: OnceLock<LoadedVehicleModel> = OnceLock::new();
static SEDAN_LOD2: OnceLock<LoadedVehicleModel> = OnceLock::new();
static SUV_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static SUV_LOD1: OnceLock<LoadedVehicleModel> = OnceLock::new();
static SUV_LOD2: OnceLock<LoadedVehicleModel> = OnceLock::new();
static TAXI_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static TRUCK_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static BUS_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static BIKE_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static LOCO_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static COACH_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static FERRY_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static AIRLINER_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();
static SKYHAWK_LOD0: OnceLock<LoadedVehicleModel> = OnceLock::new();

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
        VehicleVisualType::Sedan => match lod {
            VehicleLodLevel::Lod0High => {
                SEDAN_LOD0.get_or_init(|| build_executive_sedan(VehicleLodLevel::Lod0High))
            }
            VehicleLodLevel::Lod1Medium => {
                SEDAN_LOD1.get_or_init(|| build_executive_sedan(VehicleLodLevel::Lod1Medium))
            }
            VehicleLodLevel::Lod2Low => {
                SEDAN_LOD2.get_or_init(|| build_executive_sedan(VehicleLodLevel::Lod2Low))
            }
        },
        VehicleVisualType::Suv => match lod {
            VehicleLodLevel::Lod0High => {
                SUV_LOD0.get_or_init(|| build_overland_explorer_suv(VehicleLodLevel::Lod0High))
            }
            VehicleLodLevel::Lod1Medium => {
                SUV_LOD1.get_or_init(|| build_overland_explorer_suv(VehicleLodLevel::Lod1Medium))
            }
            VehicleLodLevel::Lod2Low => {
                SUV_LOD2.get_or_init(|| build_overland_explorer_suv(VehicleLodLevel::Lod2Low))
            }
        },
        VehicleVisualType::Taxi => TAXI_LOD0.get_or_init(|| build_metro_yellow_cab(lod)),
        VehicleVisualType::HeavyTruck => {
            TRUCK_LOD0.get_or_init(|| build_transcontinental_semi(lod))
        }
        VehicleVisualType::TransitBus => BUS_LOD0.get_or_init(|| build_cityline_transit_bus(lod)),
        VehicleVisualType::Motorcycle => BIKE_LOD0.get_or_init(|| build_hypersport_rr1000(lod)),
        VehicleVisualType::TrainLocomotive => {
            LOCO_LOD0.get_or_init(|| build_pacific_locomotive(lod))
        }
        VehicleVisualType::TrainWagon => COACH_LOD0.get_or_init(|| build_passenger_coach(lod)),
        VehicleVisualType::MarineVessel => FERRY_LOD0.get_or_init(|| build_golden_gate_ferry(lod)),
        VehicleVisualType::CommercialAirliner => {
            AIRLINER_LOD0.get_or_init(|| build_skycourier_b737(lod))
        }
        VehicleVisualType::GeneralAviation => SKYHAWK_LOD0.get_or_init(|| build_skyhawk_c172(lod)),
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
        (
            "pacific_locomotive_lod0.glb",
            build_pacific_locomotive(VehicleLodLevel::Lod0High),
        ),
        (
            "passenger_coach_lod0.glb",
            build_passenger_coach(VehicleLodLevel::Lod0High),
        ),
        (
            "golden_gate_ferry_lod0.glb",
            build_golden_gate_ferry(VehicleLodLevel::Lod0High),
        ),
        (
            "skycourier_b737_lod0.glb",
            build_skycourier_b737(VehicleLodLevel::Lod0High),
        ),
        (
            "skyhawk_c172_lod0.glb",
            build_skyhawk_c172(VehicleLodLevel::Lod0High),
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
