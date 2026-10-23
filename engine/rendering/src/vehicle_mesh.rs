//! 3D Procedural vehicle mesh generation for drivable cars and AI traffic.

use glam::{Quat, Vec3};
use realearth_compiler::mesh::{ChunkMesh, Vertex};

/// Type classification for procedural vehicle rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleVisualType {
    Sedan,
    SportsCoupe,
    Suv,
    TransitBus,
    HeavyTruck,
    Taxi,
}

/// Visual parameters for rendering a dynamic or traffic vehicle.
#[derive(Debug, Clone)]
pub struct VehicleMeshParams {
    pub position: Vec3,
    pub orientation: Quat,
    pub color: [f32; 4],
    pub vehicle_type: VehicleVisualType,
    pub wheel_steer_angle: f32,
    pub headlights_on: bool,
    pub brake_lights_on: bool,
}

impl Default for VehicleMeshParams {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            color: [0.85, 0.15, 0.15, 1.0], // Sport red
            vehicle_type: VehicleVisualType::Sedan,
            wheel_steer_angle: 0.0,
            headlights_on: true,
            brake_lights_on: false,
        }
    }
}

/// Generates a complete 3D ChunkMesh for a vehicle.
pub fn generate_vehicle_mesh(params: &VehicleMeshParams) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();

    let (length, width, height, cabin_len, cabin_w, cabin_h, cabin_offset_z) =
        match params.vehicle_type {
            VehicleVisualType::SportsCoupe => (4.3, 1.85, 0.70, 2.0, 1.55, 0.45, -0.2),
            VehicleVisualType::Sedan | VehicleVisualType::Taxi => {
                (4.6, 1.80, 0.80, 2.4, 1.60, 0.55, -0.1)
            }
            VehicleVisualType::Suv => (4.8, 1.95, 1.00, 3.0, 1.70, 0.70, -0.1),
            VehicleVisualType::TransitBus => (11.5, 2.50, 2.20, 10.0, 2.40, 1.60, 0.0),
            VehicleVisualType::HeavyTruck => (14.0, 2.55, 2.80, 4.0, 2.40, 1.80, 3.5),
        };

    let chassis_color = match params.vehicle_type {
        VehicleVisualType::Taxi => [0.95, 0.78, 0.12, 1.0], // Classic Yellow Taxi
        _ => params.color,
    };
    let glass_color = [0.15, 0.22, 0.30, 0.95];
    let wheel_rubber = [0.12, 0.12, 0.12, 1.0];
    let wheel_rim = [0.70, 0.72, 0.75, 1.0];
    let headlight_color = if params.headlights_on {
        [1.0, 1.0, 0.85, 1.0]
    } else {
        [0.4, 0.4, 0.4, 1.0]
    };
    let taillight_color = if params.brake_lights_on {
        [1.0, 0.05, 0.05, 1.0] // Intense braking red
    } else {
        [0.6, 0.05, 0.05, 1.0]
    };

    // 1. Lower Chassis Box
    let half_w = width * 0.5;
    let half_l = length * 0.5;
    let chassis_bottom = 0.25;
    let chassis_top = chassis_bottom + height * 0.6;
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, (chassis_bottom + chassis_top) * 0.5, 0.0),
        Vec3::new(half_w, (chassis_top - chassis_bottom) * 0.5, half_l),
        chassis_color,
    );

    // 2. Cabin / Greenhouse
    let cabin_bottom = chassis_top;
    let cabin_top = cabin_bottom + cabin_h;
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, (cabin_bottom + cabin_top) * 0.5, cabin_offset_z),
        Vec3::new(cabin_w * 0.5, cabin_h * 0.5, cabin_len * 0.5),
        glass_color,
    );

    // 3. For Heavy Trucks: Trailer Cargo Container behind cab
    if params.vehicle_type == VehicleVisualType::HeavyTruck {
        let trailer_l = 8.5;
        let trailer_w = 2.45;
        let trailer_h = 2.4;
        let trailer_center_z = -2.2;
        add_box(
            &mut mesh,
            params,
            Vec3::new(0.0, chassis_bottom + trailer_h * 0.5, trailer_center_z),
            Vec3::new(trailer_w * 0.5, trailer_h * 0.5, trailer_l * 0.5),
            [0.85, 0.88, 0.92, 1.0], // Clean white freight container
        );
    }

    // 4. Wheels (FL, FR, RL, RR)
    let wheel_r = 0.35;
    let wheel_w = 0.22;
    let wheel_z_front = length * 0.32;
    let wheel_z_rear = -length * 0.32;
    let wheel_x = half_w;

    let wheel_positions = [
        (Vec3::new(-wheel_x, wheel_r, wheel_z_front), true),
        (Vec3::new(wheel_x, wheel_r, wheel_z_front), true),
        (Vec3::new(-wheel_x, wheel_r, wheel_z_rear), false),
        (Vec3::new(wheel_x, wheel_r, wheel_z_rear), false),
    ];

    for (pos, is_steerable) in wheel_positions {
        let wheel_rot = if is_steerable {
            Quat::from_rotation_y(params.wheel_steer_angle)
        } else {
            Quat::IDENTITY
        };
        // Rubber tire
        add_oriented_box(
            &mut mesh,
            params,
            pos,
            wheel_rot,
            Vec3::new(wheel_w * 0.5, wheel_r, wheel_r),
            wheel_rubber,
        );
        // Rim center
        add_oriented_box(
            &mut mesh,
            params,
            pos,
            wheel_rot,
            Vec3::new(wheel_w * 0.55, wheel_r * 0.5, wheel_r * 0.5),
            wheel_rim,
        );
    }

    // 5. Headlights (facing forward: +Z in vehicle local space)
    let hl_w = 0.22;
    let hl_h = 0.12;
    let hl_y = chassis_bottom + (chassis_top - chassis_bottom) * 0.65;
    let hl_z = half_l + 0.02;
    // Left headlight
    add_quad(
        &mut mesh,
        params,
        Vec3::new(-half_w * 0.65 - hl_w * 0.5, hl_y, hl_z),
        Vec3::new(-half_w * 0.65 + hl_w * 0.5, hl_y, hl_z),
        Vec3::new(-half_w * 0.65 + hl_w * 0.5, hl_y + hl_h, hl_z),
        Vec3::new(-half_w * 0.65 - hl_w * 0.5, hl_y + hl_h, hl_z),
        Vec3::Z,
        headlight_color,
    );
    // Right headlight
    add_quad(
        &mut mesh,
        params,
        Vec3::new(half_w * 0.65 - hl_w * 0.5, hl_y, hl_z),
        Vec3::new(half_w * 0.65 + hl_w * 0.5, hl_y, hl_z),
        Vec3::new(half_w * 0.65 + hl_w * 0.5, hl_y + hl_h, hl_z),
        Vec3::new(half_w * 0.65 - hl_w * 0.5, hl_y + hl_h, hl_z),
        Vec3::Z,
        headlight_color,
    );

    // 6. Taillights (facing backward: -Z in vehicle local space)
    let tl_w = 0.25;
    let tl_h = 0.10;
    let tl_z = -half_l - 0.02;
    // Left taillight
    add_quad(
        &mut mesh,
        params,
        Vec3::new(-half_w * 0.70 + tl_w * 0.5, hl_y, tl_z),
        Vec3::new(-half_w * 0.70 - tl_w * 0.5, hl_y, tl_z),
        Vec3::new(-half_w * 0.70 - tl_w * 0.5, hl_y + tl_h, tl_z),
        Vec3::new(-half_w * 0.70 + tl_w * 0.5, hl_y + tl_h, tl_z),
        -Vec3::Z,
        taillight_color,
    );
    // Right taillight
    add_quad(
        &mut mesh,
        params,
        Vec3::new(half_w * 0.70 + tl_w * 0.5, hl_y, tl_z),
        Vec3::new(half_w * 0.70 - tl_w * 0.5, hl_y, tl_z),
        Vec3::new(half_w * 0.70 - tl_w * 0.5, hl_y + tl_h, tl_z),
        Vec3::new(half_w * 0.70 + tl_w * 0.5, hl_y + tl_h, tl_z),
        -Vec3::Z,
        taillight_color,
    );

    mesh
}

#[allow(clippy::too_many_arguments)]
fn add_quad(
    mesh: &mut ChunkMesh,
    params: &VehicleMeshParams,
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    normal: Vec3,
    color: [f32; 4],
) {
    let base_idx = mesh.vertices.len() as u32;

    let w_p0 = params.position + params.orientation.mul_vec3(p0);
    let w_p1 = params.position + params.orientation.mul_vec3(p1);
    let w_p2 = params.position + params.orientation.mul_vec3(p2);
    let w_p3 = params.position + params.orientation.mul_vec3(p3);
    let w_norm = (params.orientation.mul_vec3(normal)).normalize();

    mesh.vertices.push(Vertex::new(
        [w_p0.x, w_p0.y, w_p0.z],
        [w_norm.x, w_norm.y, w_norm.z],
        [0.0, 0.0],
        color,
    ));
    mesh.vertices.push(Vertex::new(
        [w_p1.x, w_p1.y, w_p1.z],
        [w_norm.x, w_norm.y, w_norm.z],
        [1.0, 0.0],
        color,
    ));
    mesh.vertices.push(Vertex::new(
        [w_p2.x, w_p2.y, w_p2.z],
        [w_norm.x, w_norm.y, w_norm.z],
        [1.0, 1.0],
        color,
    ));
    mesh.vertices.push(Vertex::new(
        [w_p3.x, w_p3.y, w_p3.z],
        [w_norm.x, w_norm.y, w_norm.z],
        [0.0, 1.0],
        color,
    ));

    mesh.indices.extend_from_slice(&[
        base_idx,
        base_idx + 1,
        base_idx + 2,
        base_idx,
        base_idx + 2,
        base_idx + 3,
    ]);
}

fn add_box(
    mesh: &mut ChunkMesh,
    params: &VehicleMeshParams,
    center: Vec3,
    half_extents: Vec3,
    color: [f32; 4],
) {
    add_oriented_box(mesh, params, center, Quat::IDENTITY, half_extents, color);
}

fn add_oriented_box(
    mesh: &mut ChunkMesh,
    params: &VehicleMeshParams,
    center: Vec3,
    local_rot: Quat,
    half_extents: Vec3,
    color: [f32; 4],
) {
    let (hx, hy, hz) = (half_extents.x, half_extents.y, half_extents.z);

    let corners = [
        center + local_rot.mul_vec3(Vec3::new(-hx, -hy, hz)), // 0: front-bottom-left
        center + local_rot.mul_vec3(Vec3::new(hx, -hy, hz)),  // 1: front-bottom-right
        center + local_rot.mul_vec3(Vec3::new(hx, hy, hz)),   // 2: front-top-right
        center + local_rot.mul_vec3(Vec3::new(-hx, hy, hz)),  // 3: front-top-left
        center + local_rot.mul_vec3(Vec3::new(-hx, -hy, -hz)), // 4: rear-bottom-left
        center + local_rot.mul_vec3(Vec3::new(hx, -hy, -hz)), // 5: rear-bottom-right
        center + local_rot.mul_vec3(Vec3::new(hx, hy, -hz)),  // 6: rear-top-right
        center + local_rot.mul_vec3(Vec3::new(-hx, hy, -hz)), // 7: rear-top-left
    ];

    // Front face (+Z)
    add_quad(
        mesh,
        params,
        corners[0],
        corners[1],
        corners[2],
        corners[3],
        local_rot.mul_vec3(Vec3::Z),
        color,
    );
    // Rear face (-Z)
    add_quad(
        mesh,
        params,
        corners[5],
        corners[4],
        corners[7],
        corners[6],
        local_rot.mul_vec3(-Vec3::Z),
        color,
    );
    // Top face (+Y)
    add_quad(
        mesh,
        params,
        corners[3],
        corners[2],
        corners[6],
        corners[7],
        local_rot.mul_vec3(Vec3::Y),
        color,
    );
    // Bottom face (-Y)
    add_quad(
        mesh,
        params,
        corners[4],
        corners[5],
        corners[1],
        corners[0],
        local_rot.mul_vec3(-Vec3::Y),
        color,
    );
    // Right face (+X)
    add_quad(
        mesh,
        params,
        corners[1],
        corners[5],
        corners[6],
        corners[2],
        local_rot.mul_vec3(Vec3::X),
        color,
    );
    // Left face (-X)
    add_quad(
        mesh,
        params,
        corners[4],
        corners[0],
        corners[3],
        corners[7],
        local_rot.mul_vec3(-Vec3::X),
        color,
    );
}
