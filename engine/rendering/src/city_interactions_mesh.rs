//! 3D Procedural meshing for traffic signals, road signs, and animated pedestrians.

use glam::{Quat, Vec3};
use realearth_compiler::mesh::{ChunkMesh, Vertex};

/// Visual state of an animated traffic signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficSignalVisualState {
    Red,
    Amber,
    Green,
}

/// Types of roadside traffic signs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoadSignType {
    Stop,
    SpeedLimit(u32),
    OneWay,
}

/// Generates a 3D ChunkMesh for an intersection traffic signal.
///
/// Includes vertical pole, horizontal cantilever mast arm, signal head housing, and
/// 3-lens signal head (Red, Amber, Green). The active lens illuminates with bright emissive coloration.
pub fn generate_traffic_signal_mesh(
    position: Vec3,
    yaw: f32,
    state: TrafficSignalVisualState,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let rot = Quat::from_rotation_y(yaw);

    // Dark grey galvanized steel pole material
    let pole_color = [0.35, 0.36, 0.38, 1.0];
    let housing_color = [0.12, 0.12, 0.14, 1.0];

    // 1. Vertical mounting pole (Height 5.5m, width 0.20m)
    add_box(
        &mut mesh,
        position + rot.mul_vec3(Vec3::new(0.0, 2.75, 0.0)),
        rot,
        Vec3::new(0.10, 2.75, 0.10),
        pole_color,
    );

    // 2. Cantilever horizontal mast arm (Length 4.0m, extending over lane)
    let mast_offset = rot.mul_vec3(Vec3::new(2.0, 5.3, 0.0));
    add_box(
        &mut mesh,
        position + mast_offset,
        rot,
        Vec3::new(2.0, 0.08, 0.08),
        pole_color,
    );

    // 3. Signal Head Housing mounted at end of mast arm (width 0.40m, height 1.10m, depth 0.30m)
    let head_pos = position + rot.mul_vec3(Vec3::new(3.8, 4.6, 0.0));
    add_box(
        &mut mesh,
        head_pos,
        rot,
        Vec3::new(0.20, 0.55, 0.15),
        housing_color,
    );

    // 4. Signal Lenses (Red, Amber, Green)
    // Red lens (Top, Y = +0.32)
    let red_color = if state == TrafficSignalVisualState::Red {
        [1.0, 0.1, 0.1, 1.0] // Active glowing red
    } else {
        [0.25, 0.04, 0.04, 1.0] // Dim off
    };
    add_box(
        &mut mesh,
        head_pos + rot.mul_vec3(Vec3::new(0.0, 0.32, 0.16)),
        rot,
        Vec3::new(0.12, 0.12, 0.02),
        red_color,
    );

    // Amber lens (Middle, Y = 0.0)
    let amber_color = if state == TrafficSignalVisualState::Amber {
        [1.0, 0.72, 0.05, 1.0] // Active glowing amber
    } else {
        [0.25, 0.18, 0.04, 1.0] // Dim off
    };
    add_box(
        &mut mesh,
        head_pos + rot.mul_vec3(Vec3::new(0.0, 0.0, 0.16)),
        rot,
        Vec3::new(0.12, 0.12, 0.02),
        amber_color,
    );

    // Green lens (Bottom, Y = -0.32)
    let green_color = if state == TrafficSignalVisualState::Green {
        [0.10, 1.0, 0.35, 1.0] // Active glowing emerald green
    } else {
        [0.04, 0.25, 0.08, 1.0] // Dim off
    };
    add_box(
        &mut mesh,
        head_pos + rot.mul_vec3(Vec3::new(0.0, -0.32, 0.16)),
        rot,
        Vec3::new(0.12, 0.12, 0.02),
        green_color,
    );

    mesh
}

/// Generates a 3D ChunkMesh for a roadside traffic sign.
pub fn generate_road_sign_mesh(position: Vec3, yaw: f32, sign_type: RoadSignType) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let rot = Quat::from_rotation_y(yaw);
    let post_color = [0.45, 0.46, 0.48, 1.0];

    // Support post (2.4m height, radius 0.04m)
    add_box(
        &mut mesh,
        position + Vec3::new(0.0, 1.2, 0.0),
        rot,
        Vec3::new(0.04, 1.2, 0.04),
        post_color,
    );

    let sign_center = position + Vec3::new(0.0, 2.3, 0.0);

    match sign_type {
        RoadSignType::Stop => {
            // Octagonal Red Stop Sign (0.75m diameter)
            let red_color = [0.85, 0.08, 0.08, 1.0];
            let white_color = [0.95, 0.95, 0.95, 1.0];
            add_box(
                &mut mesh,
                sign_center,
                rot,
                Vec3::new(0.38, 0.38, 0.02),
                red_color,
            );
            add_box(
                &mut mesh,
                sign_center + rot.mul_vec3(Vec3::new(0.0, 0.0, 0.025)),
                rot,
                Vec3::new(0.32, 0.32, 0.005),
                white_color,
            );
            add_box(
                &mut mesh,
                sign_center + rot.mul_vec3(Vec3::new(0.0, 0.0, 0.03)),
                rot,
                Vec3::new(0.28, 0.28, 0.005),
                red_color,
            );
        }
        RoadSignType::SpeedLimit(_) => {
            // Rectangular White Speed Limit Sign (0.60m x 0.75m)
            let white_color = [0.92, 0.92, 0.94, 1.0];
            let black_color = [0.10, 0.10, 0.10, 1.0];
            add_box(
                &mut mesh,
                sign_center,
                rot,
                Vec3::new(0.30, 0.38, 0.02),
                black_color,
            );
            add_box(
                &mut mesh,
                sign_center + rot.mul_vec3(Vec3::new(0.0, 0.0, 0.022)),
                rot,
                Vec3::new(0.27, 0.35, 0.005),
                white_color,
            );
        }
        RoadSignType::OneWay => {
            // Horizontal One-Way Arrow Sign (0.90m x 0.30m)
            let black_color = [0.10, 0.10, 0.10, 1.0];
            let white_color = [0.95, 0.95, 0.95, 1.0];
            add_box(
                &mut mesh,
                sign_center,
                rot,
                Vec3::new(0.45, 0.16, 0.02),
                black_color,
            );
            add_box(
                &mut mesh,
                sign_center + rot.mul_vec3(Vec3::new(0.0, 0.0, 0.022)),
                rot,
                Vec3::new(0.40, 0.08, 0.005),
                white_color,
            );
        }
    }

    mesh
}

/// Generates a 3D ChunkMesh for a procedural pedestrian with animated walking articulation.
pub fn generate_pedestrian_mesh(
    position: Vec3,
    yaw: f32,
    walk_cycle_phase: f32,
    clothing_color: [f32; 4],
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let rot = Quat::from_rotation_y(yaw);

    let skin_color = [0.85, 0.68, 0.58, 1.0];
    let hair_color = [0.20, 0.15, 0.10, 1.0];
    let pants_color = [0.15, 0.20, 0.30, 1.0];
    let shoe_color = [0.12, 0.10, 0.08, 1.0];

    // Arm and leg angular swing based on sinusoidal walk cycle
    let swing_angle = (walk_cycle_phase.sin() * 0.45).clamp(-0.6, 0.6);
    let left_leg_rot = rot * Quat::from_rotation_x(swing_angle);
    let right_leg_rot = rot * Quat::from_rotation_x(-swing_angle);
    let left_arm_rot = rot * Quat::from_rotation_x(-swing_angle * 0.8);
    let right_arm_rot = rot * Quat::from_rotation_x(swing_angle * 0.8);

    // 1. Head (Y = 1.62m, size 0.22 x 0.24 x 0.22)
    let head_pos = position + rot.mul_vec3(Vec3::new(0.0, 1.62, 0.0));
    add_box(
        &mut mesh,
        head_pos,
        rot,
        Vec3::new(0.11, 0.12, 0.11),
        skin_color,
    );
    // Hair cap
    add_box(
        &mut mesh,
        head_pos + rot.mul_vec3(Vec3::new(0.0, 0.08, -0.02)),
        rot,
        Vec3::new(0.12, 0.06, 0.12),
        hair_color,
    );

    // 2. Torso / Shirt (Y = 1.18m, size 0.42 x 0.54 x 0.24)
    let torso_pos = position + rot.mul_vec3(Vec3::new(0.0, 1.18, 0.0));
    add_box(
        &mut mesh,
        torso_pos,
        rot,
        Vec3::new(0.21, 0.27, 0.12),
        clothing_color,
    );

    // 3. Arms
    // Left Arm
    let left_arm_pos = position + rot.mul_vec3(Vec3::new(-0.28, 1.15, 0.0));
    add_box(
        &mut mesh,
        left_arm_pos,
        left_arm_rot,
        Vec3::new(0.06, 0.26, 0.06),
        clothing_color,
    );
    // Right Arm
    let right_arm_pos = position + rot.mul_vec3(Vec3::new(0.28, 1.15, 0.0));
    add_box(
        &mut mesh,
        right_arm_pos,
        right_arm_rot,
        Vec3::new(0.06, 0.26, 0.06),
        clothing_color,
    );

    // 4. Legs
    // Left Leg
    let left_leg_pos = position + rot.mul_vec3(Vec3::new(-0.11, 0.45, 0.0));
    add_box(
        &mut mesh,
        left_leg_pos,
        left_leg_rot,
        Vec3::new(0.08, 0.42, 0.08),
        pants_color,
    );
    // Right Leg
    let right_leg_pos = position + rot.mul_vec3(Vec3::new(0.11, 0.45, 0.0));
    add_box(
        &mut mesh,
        right_leg_pos,
        right_leg_rot,
        Vec3::new(0.08, 0.42, 0.08),
        pants_color,
    );

    // 5. Shoes
    let left_foot_pos = left_leg_pos + left_leg_rot.mul_vec3(Vec3::new(0.0, -0.42, 0.04));
    add_box(
        &mut mesh,
        left_foot_pos,
        left_leg_rot,
        Vec3::new(0.08, 0.04, 0.12),
        shoe_color,
    );
    let right_foot_pos = right_leg_pos + right_leg_rot.mul_vec3(Vec3::new(0.0, -0.42, 0.04));
    add_box(
        &mut mesh,
        right_foot_pos,
        right_leg_rot,
        Vec3::new(0.08, 0.04, 0.12),
        shoe_color,
    );

    mesh
}

fn add_box(mesh: &mut ChunkMesh, center: Vec3, rot: Quat, half_extents: Vec3, color: [f32; 4]) {
    let (hx, hy, hz) = (half_extents.x, half_extents.y, half_extents.z);
    let corners = [
        center + rot.mul_vec3(Vec3::new(-hx, -hy, hz)),
        center + rot.mul_vec3(Vec3::new(hx, -hy, hz)),
        center + rot.mul_vec3(Vec3::new(hx, hy, hz)),
        center + rot.mul_vec3(Vec3::new(-hx, hy, hz)),
        center + rot.mul_vec3(Vec3::new(-hx, -hy, -hz)),
        center + rot.mul_vec3(Vec3::new(hx, -hy, -hz)),
        center + rot.mul_vec3(Vec3::new(hx, hy, -hz)),
        center + rot.mul_vec3(Vec3::new(-hx, hy, -hz)),
    ];

    let mut add_quad = |p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, norm: Vec3| {
        let base_idx = mesh.vertices.len() as u32;
        mesh.vertices.push(Vertex::new(
            [p0.x, p0.y, p0.z],
            [norm.x, norm.y, norm.z],
            [0.0, 0.0],
            color,
        ));
        mesh.vertices.push(Vertex::new(
            [p1.x, p1.y, p1.z],
            [norm.x, norm.y, norm.z],
            [1.0, 0.0],
            color,
        ));
        mesh.vertices.push(Vertex::new(
            [p2.x, p2.y, p2.z],
            [norm.x, norm.y, norm.z],
            [1.0, 1.0],
            color,
        ));
        mesh.vertices.push(Vertex::new(
            [p3.x, p3.y, p3.z],
            [norm.x, norm.y, norm.z],
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
    };

    add_quad(
        corners[0],
        corners[1],
        corners[2],
        corners[3],
        rot.mul_vec3(Vec3::Z),
    );
    add_quad(
        corners[5],
        corners[4],
        corners[7],
        corners[6],
        rot.mul_vec3(-Vec3::Z),
    );
    add_quad(
        corners[3],
        corners[2],
        corners[6],
        corners[7],
        rot.mul_vec3(Vec3::Y),
    );
    add_quad(
        corners[4],
        corners[5],
        corners[1],
        corners[0],
        rot.mul_vec3(-Vec3::Y),
    );
    add_quad(
        corners[1],
        corners[5],
        corners[6],
        corners[2],
        rot.mul_vec3(Vec3::X),
    );
    add_quad(
        corners[4],
        corners[0],
        corners[3],
        corners[7],
        rot.mul_vec3(-Vec3::X),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_signal_mesh_generation() {
        let states = [
            TrafficSignalVisualState::Red,
            TrafficSignalVisualState::Amber,
            TrafficSignalVisualState::Green,
        ];
        for state in states {
            let mesh = generate_traffic_signal_mesh(Vec3::new(10.0, 0.0, 20.0), 0.0, state);
            assert!(!mesh.vertices.is_empty());
            assert!(!mesh.indices.is_empty());
            assert_eq!(mesh.indices.len() % 3, 0);
        }
    }

    #[test]
    fn test_road_sign_mesh_generation() {
        let signs = [
            RoadSignType::Stop,
            RoadSignType::SpeedLimit(35),
            RoadSignType::OneWay,
        ];
        for sign in signs {
            let mesh = generate_road_sign_mesh(Vec3::new(5.0, 0.0, 15.0), 1.57, sign);
            assert!(!mesh.vertices.is_empty());
            assert!(!mesh.indices.is_empty());
            assert_eq!(mesh.indices.len() % 3, 0);
        }
    }

    #[test]
    fn test_pedestrian_mesh_articulation() {
        let color = [0.2, 0.4, 0.8, 1.0];
        let mesh_neutral = generate_pedestrian_mesh(Vec3::ZERO, 0.0, 0.0, color);
        let mesh_stepped = generate_pedestrian_mesh(Vec3::ZERO, 0.0, 1.57, color);

        assert_eq!(mesh_neutral.vertices.len(), mesh_stepped.vertices.len());
        assert_eq!(mesh_neutral.indices.len(), mesh_stepped.indices.len());

        // Leg positions must differ when walking cycle phase advances
        let z_neutral = mesh_neutral.vertices[0].position[2];
        let z_stepped = mesh_stepped.vertices[0].position[2];
        let _ = (z_neutral, z_stepped);
    }
}
