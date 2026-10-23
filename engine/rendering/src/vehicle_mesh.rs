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
    pub reverse_lights_on: bool,
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
            reverse_lights_on: false,
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
    let interior_dash_color = [0.12, 0.13, 0.15, 1.0];
    let interior_seat_color = [0.15, 0.16, 0.18, 1.0];
    let steering_wheel_color = [0.20, 0.21, 0.24, 1.0];
    let chrome_color = [0.85, 0.88, 0.92, 1.0];
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
    let reverse_light_color = if params.reverse_lights_on {
        [0.98, 0.98, 1.0, 1.0] // Bright cool white reverse light
    } else {
        [0.35, 0.35, 0.35, 1.0] // Inactive reverse lens
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

    // 3. Cockpit Interior (Dashboard, Steering Wheel, Seats, Mirrors)
    if params.vehicle_type != VehicleVisualType::HeavyTruck
        && params.vehicle_type != VehicleVisualType::TransitBus
    {
        // 3a. Dashboard console
        let dash_z = cabin_offset_z + cabin_len * 0.35;
        let dash_y = cabin_bottom + 0.16;
        add_box(
            &mut mesh,
            params,
            Vec3::new(0.0, dash_y, dash_z),
            Vec3::new(cabin_w * 0.42, 0.10, 0.16),
            interior_dash_color,
        );

        // 3b. Driver Bucket Seat (Left-hand drive, x = -0.38)
        let seat_z = cabin_offset_z - 0.05;
        let seat_y = cabin_bottom + 0.12;
        add_box(
            &mut mesh,
            params,
            Vec3::new(-0.38, seat_y, seat_z),
            Vec3::new(0.24, 0.08, 0.24),
            interior_seat_color,
        );
        // Backrest + headrest
        add_box(
            &mut mesh,
            params,
            Vec3::new(-0.38, seat_y + 0.26, seat_z - 0.20),
            Vec3::new(0.22, 0.20, 0.06),
            interior_seat_color,
        );

        // 3c. Passenger Bucket Seat (Right side, x = +0.38)
        add_box(
            &mut mesh,
            params,
            Vec3::new(0.38, seat_y, seat_z),
            Vec3::new(0.24, 0.08, 0.24),
            interior_seat_color,
        );
        add_box(
            &mut mesh,
            params,
            Vec3::new(0.38, seat_y + 0.26, seat_z - 0.20),
            Vec3::new(0.22, 0.20, 0.06),
            interior_seat_color,
        );

        // 3d. Steering Wheel & Steering Column (animated rotation with steer angle)
        let wheel_center = Vec3::new(-0.38, cabin_bottom + 0.28, dash_z - 0.15);
        // Steering wheel tilted back by 18 degrees around X, rotated by steer angle around its column axis
        let tilt_quat = Quat::from_rotation_x(0.31);
        let steer_yaw_quat = Quat::from_rotation_z(-params.wheel_steer_angle * 3.5);
        let sw_rot = tilt_quat * steer_yaw_quat;

        // Steering column
        add_oriented_box(
            &mut mesh,
            params,
            wheel_center + Vec3::new(0.0, -0.08, 0.10),
            tilt_quat,
            Vec3::new(0.03, 0.03, 0.12),
            interior_dash_color,
        );
        // Steering wheel center hub
        add_oriented_box(
            &mut mesh,
            params,
            wheel_center,
            sw_rot,
            Vec3::new(0.045, 0.045, 0.02),
            steering_wheel_color,
        );
        // Horizontal spokes
        add_oriented_box(
            &mut mesh,
            params,
            wheel_center,
            sw_rot,
            Vec3::new(0.16, 0.02, 0.015),
            steering_wheel_color,
        );
        // Top and bottom rim sections
        add_oriented_box(
            &mut mesh,
            params,
            wheel_center + sw_rot.mul_vec3(Vec3::new(0.0, 0.16, 0.0)),
            sw_rot,
            Vec3::new(0.12, 0.02, 0.02),
            steering_wheel_color,
        );
        add_oriented_box(
            &mut mesh,
            params,
            wheel_center + sw_rot.mul_vec3(Vec3::new(0.0, -0.16, 0.0)),
            sw_rot,
            Vec3::new(0.12, 0.02, 0.02),
            steering_wheel_color,
        );

        // 3e. Exterior Side Rear-View Mirrors
        let mirror_z = cabin_offset_z + cabin_len * 0.42;
        let mirror_y = cabin_bottom + 0.15;
        // Left mirror
        add_box(
            &mut mesh,
            params,
            Vec3::new(-half_w - 0.10, mirror_y, mirror_z),
            Vec3::new(0.08, 0.05, 0.08),
            chassis_color,
        );
        // Right mirror
        add_box(
            &mut mesh,
            params,
            Vec3::new(half_w + 0.10, mirror_y, mirror_z),
            Vec3::new(0.08, 0.05, 0.08),
            chassis_color,
        );
    }

    // 4. Vehicle Visual Variations (Spoilers, Roof Racks, Taxi Signs, Exhausts)
    match params.vehicle_type {
        VehicleVisualType::SportsCoupe => {
            // Aerodynamic Rear Wing / Spoiler
            let wing_z = -half_l * 0.88;
            let wing_y = chassis_top + 0.24;
            // Wing blade
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.0, wing_y, wing_z),
                Vec3::new(half_w * 0.95, 0.025, 0.15),
                [0.10, 0.10, 0.10, 1.0], // Carbon black wing
            );
            // Stanchions (left and right supports)
            add_box(
                &mut mesh,
                params,
                Vec3::new(-half_w * 0.55, (chassis_top + wing_y) * 0.5, wing_z),
                Vec3::new(0.025, (wing_y - chassis_top) * 0.5, 0.04),
                [0.15, 0.15, 0.15, 1.0],
            );
            add_box(
                &mut mesh,
                params,
                Vec3::new(half_w * 0.55, (chassis_top + wing_y) * 0.5, wing_z),
                Vec3::new(0.025, (wing_y - chassis_top) * 0.5, 0.04),
                [0.15, 0.15, 0.15, 1.0],
            );
            // Front Chin Aerodynamic Splitter
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.0, chassis_bottom + 0.02, half_l + 0.06),
                Vec3::new(half_w * 0.98, 0.02, 0.08),
                [0.08, 0.08, 0.08, 1.0],
            );
            // Dual Polished Chrome Exhaust Tips
            add_box(
                &mut mesh,
                params,
                Vec3::new(-0.45, chassis_bottom + 0.06, -half_l - 0.06),
                Vec3::new(0.06, 0.04, 0.06),
                chrome_color,
            );
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.45, chassis_bottom + 0.06, -half_l - 0.06),
                Vec3::new(0.06, 0.04, 0.06),
                chrome_color,
            );
        }
        VehicleVisualType::Suv => {
            // Dual Roof Utility Rack Bars
            let front_bar_z = cabin_offset_z + cabin_len * 0.28;
            let rear_bar_z = cabin_offset_z - cabin_len * 0.28;
            let bar_y = cabin_top + 0.04;
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.0, bar_y, front_bar_z),
                Vec3::new(cabin_w * 0.52, 0.025, 0.03),
                [0.22, 0.24, 0.26, 1.0],
            );
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.0, bar_y, rear_bar_z),
                Vec3::new(cabin_w * 0.52, 0.025, 0.03),
                [0.22, 0.24, 0.26, 1.0],
            );
            // Rugged Lower Front Skid Plate
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.0, chassis_bottom + 0.05, half_l + 0.03),
                Vec3::new(half_w * 0.65, 0.06, 0.04),
                chrome_color,
            );
        }
        VehicleVisualType::Taxi => {
            // Rooftop Illuminated "TAXI" Sign Dome
            let sign_y = cabin_top + 0.09;
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.0, sign_y, cabin_offset_z),
                Vec3::new(0.28, 0.07, 0.12),
                [1.0, 0.84, 0.20, 1.0], // Illuminated warm amber taxi dome
            );
            // Sign mounting base
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.0, cabin_top + 0.015, cabin_offset_z),
                Vec3::new(0.18, 0.015, 0.08),
                [0.10, 0.10, 0.10, 1.0],
            );
        }
        VehicleVisualType::Sedan => {
            // Executive Chrome Horizontal Grille Slats
            let grille_y = chassis_bottom + (chassis_top - chassis_bottom) * 0.50;
            let grille_z = half_l + 0.025;
            for i in -1..=1 {
                add_box(
                    &mut mesh,
                    params,
                    Vec3::new(0.0, grille_y + i as f32 * 0.05, grille_z),
                    Vec3::new(half_w * 0.40, 0.012, 0.015),
                    chrome_color,
                );
            }
        }
        VehicleVisualType::HeavyTruck => {
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
            // Vertical Dual Exhaust Stacks behind cab
            let stack_y = chassis_top + 1.2;
            let stack_z = 1.3;
            add_box(
                &mut mesh,
                params,
                Vec3::new(-half_w * 0.85, stack_y, stack_z),
                Vec3::new(0.06, 0.60, 0.06),
                chrome_color,
            );
            add_box(
                &mut mesh,
                params,
                Vec3::new(half_w * 0.85, stack_y, stack_z),
                Vec3::new(0.06, 0.60, 0.06),
                chrome_color,
            );
        }
        VehicleVisualType::TransitBus => {
            // Rooftop HVAC units
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.0, cabin_top + 0.12, 1.0),
                Vec3::new(0.80, 0.12, 1.2),
                [0.75, 0.78, 0.82, 1.0],
            );
        }
    }

    // 5. Wheels (FL, FR, RL, RR)
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

    // 6. Headlights (facing forward: +Z in vehicle local space)
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

    // 7. Taillights (facing backward: -Z in vehicle local space)
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

    // 8. Reverse Lights (white, inboard of taillights)
    let rev_w = 0.12;
    let rev_h = 0.08;
    // Left reverse light
    add_quad(
        &mut mesh,
        params,
        Vec3::new(-half_w * 0.35 + rev_w * 0.5, hl_y, tl_z),
        Vec3::new(-half_w * 0.35 - rev_w * 0.5, hl_y, tl_z),
        Vec3::new(-half_w * 0.35 - rev_w * 0.5, hl_y + rev_h, tl_z),
        Vec3::new(-half_w * 0.35 + rev_w * 0.5, hl_y + rev_h, tl_z),
        -Vec3::Z,
        reverse_light_color,
    );
    // Right reverse light
    add_quad(
        &mut mesh,
        params,
        Vec3::new(half_w * 0.35 + rev_w * 0.5, hl_y, tl_z),
        Vec3::new(half_w * 0.35 - rev_w * 0.5, hl_y, tl_z),
        Vec3::new(half_w * 0.35 - rev_w * 0.5, hl_y + rev_h, tl_z),
        Vec3::new(half_w * 0.35 + rev_w * 0.5, hl_y + rev_h, tl_z),
        -Vec3::Z,
        reverse_light_color,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_mesh_generation_all_types() {
        let types = [
            VehicleVisualType::SportsCoupe,
            VehicleVisualType::Sedan,
            VehicleVisualType::Suv,
            VehicleVisualType::Taxi,
            VehicleVisualType::TransitBus,
            VehicleVisualType::HeavyTruck,
        ];

        for vtype in types {
            let params = VehicleMeshParams {
                position: Vec3::new(10.0, 2.0, 30.0),
                orientation: Quat::IDENTITY,
                color: [0.8, 0.2, 0.2, 1.0],
                vehicle_type: vtype,
                wheel_steer_angle: 0.25,
                headlights_on: true,
                brake_lights_on: true,
                reverse_lights_on: true,
            };

            let mesh = generate_vehicle_mesh(&params);
            assert!(
                !mesh.vertices.is_empty(),
                "Mesh vertices should not be empty for {:?}",
                vtype
            );
            assert!(
                !mesh.indices.is_empty(),
                "Mesh indices should not be empty for {:?}",
                vtype
            );
            assert_eq!(
                mesh.indices.len() % 3,
                0,
                "Indices must form complete triangles"
            );

            // Verify vertices are positioned around the requested vehicle position
            for vert in &mesh.vertices {
                assert!(
                    (vert.position[0] - 10.0).abs() < 15.0,
                    "Vertex X within expected bounds"
                );
                assert!(
                    (vert.position[1] - 2.0).abs() < 10.0,
                    "Vertex Y within expected bounds"
                );
                assert!(
                    (vert.position[2] - 30.0).abs() < 20.0,
                    "Vertex Z within expected bounds"
                );
            }
        }
    }

    #[test]
    fn test_vehicle_lighting_states() {
        let mut params = VehicleMeshParams {
            reverse_lights_on: true,
            brake_lights_on: true,
            headlights_on: true,
            ..Default::default()
        };

        let mesh = generate_vehicle_mesh(&params);
        assert!(!mesh.vertices.is_empty());

        // Test with lights off
        params.reverse_lights_on = false;
        params.brake_lights_on = false;
        params.headlights_on = false;
        let mesh_off = generate_vehicle_mesh(&params);
        assert!(!mesh_off.vertices.is_empty());
        assert_eq!(mesh.vertices.len(), mesh_off.vertices.len());
    }
}
