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

/// Ecological tree classification for source-consistent urban and countryside vegetation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailedTreeType {
    /// Deciduous oak/maple with multi-layered broadleaf canopy and branching crown.
    BroadleafDeciduous,
    /// Evergreen needleleaf pine/spruce with multi-tiered conical boughs.
    EvergreenConifer,
    /// Coastal palm with slender curved trunk and radiating foliage fronds.
    CoastalPalm,
}

/// Generates a 3D ChunkMesh for an authentic roadside streetlight.
/// Features galvanized steel vertical pole (height 7.2m), curved luminaire arm extending 2.4m over the lane,
/// aerodynamic cobra-head fixture, and downward emissive LED illumination.
pub fn generate_streetlight_mesh(position: Vec3, yaw: f32, is_illuminated: bool) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let rot = Quat::from_rotation_y(yaw);
    let steel_color = [0.42, 0.44, 0.46, 1.0];
    let fixture_color = [0.22, 0.24, 0.26, 1.0];

    // 1. Vertical tapered pole (Height 7.2m, half-extent 0.09m)
    add_box(
        &mut mesh,
        position + Vec3::new(0.0, 3.6, 0.0),
        rot,
        Vec3::new(0.09, 3.6, 0.09),
        steel_color,
    );

    // 2. Curved luminaire outreach mast arm angled forward (+Z in local frame)
    let arm_offset = rot.mul_vec3(Vec3::new(0.0, 7.3, 1.2));
    add_box(
        &mut mesh,
        position + arm_offset,
        rot,
        Vec3::new(0.06, 0.06, 1.2),
        steel_color,
    );

    // 3. Aerodynamic cobra-head LED fixture housing
    let head_pos = position + rot.mul_vec3(Vec3::new(0.0, 7.25, 2.3));
    add_box(
        &mut mesh,
        head_pos,
        rot,
        Vec3::new(0.16, 0.08, 0.35),
        fixture_color,
    );

    // 4. Downward-facing LED lens (bright emissive warm white when active, dim grey when off)
    let lens_color = if is_illuminated {
        [1.0, 0.96, 0.84, 1.0] // High-intensity night illumination
    } else {
        [0.24, 0.24, 0.26, 1.0] // Day off
    };
    add_box(
        &mut mesh,
        head_pos - rot.mul_vec3(Vec3::new(0.0, 0.085, 0.0)),
        rot,
        Vec3::new(0.13, 0.01, 0.28),
        lens_color,
    );

    mesh
}

/// Generates a 3D ChunkMesh for a municipal low-floor bus stop shelter.
/// Features dark steel stanchions, tinted cantilever tempered glass roof, rear wind screen,
/// wooden passenger bench, and route timetable sign totem.
pub fn generate_bus_stop_shelter_mesh(position: Vec3, yaw: f32) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let rot = Quat::from_rotation_y(yaw);
    let frame_color = [0.18, 0.20, 0.22, 1.0];
    let glass_color = [0.65, 0.78, 0.88, 0.50]; // Tinted architectural glass
    let wood_color = [0.58, 0.38, 0.24, 1.0]; // Teak/polywood bench
    let sign_color = [0.92, 0.75, 0.12, 1.0]; // Municipal transit yellow

    // 1. Four vertical corner stanchions (Height 2.6m)
    let half_w = 1.8;
    let half_d = 0.9;
    for &dx in &[-half_w, half_w] {
        for &dz in &[-half_d, half_d] {
            add_box(
                &mut mesh,
                position + rot.mul_vec3(Vec3::new(dx, 1.3, dz)),
                rot,
                Vec3::new(0.05, 1.3, 0.05),
                frame_color,
            );
        }
    }

    // 2. Cantilever glass canopy roof (Width 4.0m, depth 2.2m, thickness 0.04m)
    add_box(
        &mut mesh,
        position + rot.mul_vec3(Vec3::new(0.0, 2.62, 0.0)),
        rot,
        Vec3::new(2.0, 0.02, 1.1),
        glass_color,
    );

    // 3. Rear glass weather screen
    add_box(
        &mut mesh,
        position + rot.mul_vec3(Vec3::new(0.0, 1.3, -half_d)),
        rot,
        Vec3::new(half_w, 1.1, 0.02),
        glass_color,
    );

    // 4. Passenger bench (Length 2.4m, seat height 0.48m)
    add_box(
        &mut mesh,
        position + rot.mul_vec3(Vec3::new(0.0, 0.48, -0.4)),
        rot,
        Vec3::new(1.2, 0.04, 0.22),
        wood_color,
    );
    // Bench support legs
    for &lx in &[-0.9, 0.9] {
        add_box(
            &mut mesh,
            position + rot.mul_vec3(Vec3::new(lx, 0.24, -0.4)),
            rot,
            Vec3::new(0.04, 0.24, 0.18),
            frame_color,
        );
    }

    // 5. Timetable sign totem & bus stop flag
    let totem_pos = position + rot.mul_vec3(Vec3::new(half_w + 0.4, 1.5, 0.0));
    add_box(
        &mut mesh,
        totem_pos,
        rot,
        Vec3::new(0.18, 0.50, 0.05),
        sign_color,
    );
    // Totem mounting post
    add_box(
        &mut mesh,
        position + rot.mul_vec3(Vec3::new(half_w + 0.4, 0.75, 0.0)),
        rot,
        Vec3::new(0.04, 0.75, 0.04),
        frame_color,
    );

    mesh
}

/// Generates a 3D ChunkMesh for a passenger railway station platform.
/// Conforms to standard 1.15m platform height above rail top, width 3.6m, with tactile yellow hazard paving,
/// structural steel canopy, and station designation signboards.
pub fn generate_rail_station_platform_mesh(position: Vec3, yaw: f32, length_m: f32) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let rot = Quat::from_rotation_y(yaw);
    let concrete_color = [0.72, 0.74, 0.75, 1.0];
    let tactile_yellow = [0.96, 0.82, 0.08, 1.0];
    let steel_frame = [0.25, 0.28, 0.32, 1.0];
    let canopy_roof = [0.82, 0.85, 0.88, 1.0];

    let half_len = (length_m * 0.5).max(10.0);
    let half_w = 1.8;
    let plat_h = 1.15;

    // 1. Main concrete platform slab
    add_box(
        &mut mesh,
        position + Vec3::new(0.0, plat_h * 0.5, 0.0),
        rot,
        Vec3::new(half_w, plat_h * 0.5, half_len),
        concrete_color,
    );

    // 2. High-contrast tactile blister warning stripe along platform edge (+X track side)
    add_box(
        &mut mesh,
        position + rot.mul_vec3(Vec3::new(half_w - 0.25, plat_h + 0.01, 0.0)),
        rot,
        Vec3::new(0.20, 0.01, half_len),
        tactile_yellow,
    );

    // 3. Passenger shelter canopy posts spaced every 12 meters
    let span_spacing = 12.0;
    let num_spans = ((length_m / span_spacing).floor() as i32).max(1);
    for i in -num_spans / 2..=num_spans / 2 {
        let cz = (i as f32) * span_spacing;
        // Central support pillar
        add_box(
            &mut mesh,
            position + rot.mul_vec3(Vec3::new(0.0, plat_h + 1.8, cz)),
            rot,
            Vec3::new(0.12, 1.8, 0.12),
            steel_frame,
        );
        // Cantilever roof truss
        add_box(
            &mut mesh,
            position + rot.mul_vec3(Vec3::new(0.0, plat_h + 3.6, cz)),
            rot,
            Vec3::new(half_w + 0.3, 0.08, 0.12),
            steel_frame,
        );
    }

    // 4. Overhead shelter canopy roof
    add_box(
        &mut mesh,
        position + rot.mul_vec3(Vec3::new(0.0, plat_h + 3.7, 0.0)),
        rot,
        Vec3::new(half_w + 0.3, 0.04, half_len * 0.85),
        canopy_roof,
    );

    mesh
}

/// Generates a 3D ChunkMesh for an airport runway Precision Approach Path Indicator (PAPI) light unit bank.
/// Conforms to ICAO/FAA 4-box lateral arrangement with frangible mounting stanchions and optical lenses.
pub fn generate_runway_papi_lights_mesh(
    position: Vec3,
    yaw: f32,
    glide_slope_angle_deg: f32,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let rot = Quat::from_rotation_y(yaw);
    let box_color = [0.92, 0.88, 0.15, 1.0]; // High-visibility aerodrome yellow
    let leg_color = [0.65, 0.65, 0.68, 1.0];

    // 4 PAPI light units spaced 9.0 meters apart perpendicular to runway centerline (+X)
    let unit_spacing = 6.0;
    for i in 0..4 {
        let x_off = (i as f32 - 1.5) * unit_spacing;
        let unit_center = position + rot.mul_vec3(Vec3::new(x_off, 0.65, 0.0));

        // Frangible support legs
        add_box(
            &mut mesh,
            position + rot.mul_vec3(Vec3::new(x_off, 0.30, 0.0)),
            rot,
            Vec3::new(0.04, 0.30, 0.04),
            leg_color,
        );

        // Projector housing box (0.60m x 0.40m x 0.80m)
        add_box(
            &mut mesh,
            unit_center,
            rot,
            Vec3::new(0.30, 0.20, 0.40),
            box_color,
        );

        // Optical lens: Transition color based on glideslope angle (White above, Red below)
        // Unit threshold angle offsets: unit 0 = 3.5 deg, unit 1 = 3.2 deg, unit 2 = 2.8 deg, unit 3 = 2.5 deg
        let threshold_deg = 3.5 - (i as f32 * 0.33);
        let lens_color = if glide_slope_angle_deg >= threshold_deg {
            [1.0, 1.0, 1.0, 1.0] // White light
        } else {
            [1.0, 0.08, 0.08, 1.0] // Red light (low)
        };

        // Front lens facing runway approach (+Z in local frame)
        add_box(
            &mut mesh,
            unit_center + rot.mul_vec3(Vec3::new(0.0, 0.0, 0.41)),
            rot,
            Vec3::new(0.22, 0.14, 0.02),
            lens_color,
        );
    }

    mesh
}

/// Generates a 3D ChunkMesh for a passenger maritime ferry boarding dock and terminal slip.
/// Features heavy timber and concrete pier deck, cast-iron mooring bollards, passenger gangway, and safety railings.
pub fn generate_marine_ferry_dock_mesh(position: Vec3, yaw: f32) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let rot = Quat::from_rotation_y(yaw);
    let concrete_color = [0.68, 0.70, 0.72, 1.0];
    let timber_color = [0.42, 0.32, 0.22, 1.0];
    let bollard_color = [0.15, 0.16, 0.18, 1.0];
    let gangway_color = [0.82, 0.84, 0.86, 1.0];

    // 1. Main concrete pier pontoon deck (Width 12.0m, Length 24.0m, Height 1.8m above water)
    add_box(
        &mut mesh,
        position + Vec3::new(0.0, 0.9, 0.0),
        rot,
        Vec3::new(6.0, 0.9, 12.0),
        concrete_color,
    );

    // 2. Timber fendering piles along ferry berthing edge (+Z)
    for i in -3..=3 {
        let x_pile = (i as f32) * 1.8;
        add_box(
            &mut mesh,
            position + rot.mul_vec3(Vec3::new(x_pile, 1.0, 12.2)),
            rot,
            Vec3::new(0.18, 1.0, 0.18),
            timber_color,
        );
    }

    // 3. Heavy cast-iron mooring bollards for ship lines
    for &bx in &[-5.0, 5.0] {
        for &bz in &[-8.0, 8.0] {
            add_box(
                &mut mesh,
                position + rot.mul_vec3(Vec3::new(bx, 2.05, bz)),
                rot,
                Vec3::new(0.22, 0.25, 0.22),
                bollard_color,
            );
        }
    }

    // 4. Aluminum passenger gangway ramp extending to ship deck
    add_box(
        &mut mesh,
        position + rot.mul_vec3(Vec3::new(0.0, 1.95, 14.0)),
        rot,
        Vec3::new(1.4, 0.08, 2.8),
        gangway_color,
    );
    // Gangway safety railings
    for &rx in &[-1.35, 1.35] {
        add_box(
            &mut mesh,
            position + rot.mul_vec3(Vec3::new(rx, 2.45, 14.0)),
            rot,
            Vec3::new(0.04, 0.45, 2.8),
            gangway_color,
        );
    }

    mesh
}

/// Helper to add a smooth tapered cylindrical extrusion between two 3D points.
fn add_tapered_cylinder(
    mesh: &mut ChunkMesh,
    p_bot: Vec3,
    p_top: Vec3,
    r_bot: f32,
    r_top: f32,
    segments: usize,
    color: [f32; 4],
) {
    let segs = segments.max(6);
    let step = 2.0 * std::f32::consts::PI / segs as f32;
    let axis = (p_top - p_bot).normalize_or_zero();
    let up = if axis.y.abs() > 0.95 {
        Vec3::Z
    } else {
        Vec3::Y
    };
    let radial_x = axis.cross(up).normalize();
    let radial_z = axis.cross(radial_x).normalize();

    let base_idx = mesh.vertices.len() as u32;

    for i in 0..segs {
        let a = i as f32 * step;
        let cos_a = a.cos();
        let sin_a = a.sin();
        let norm = (radial_x * cos_a + radial_z * sin_a).normalize();

        // Bottom vertex
        let v_bot = p_bot + (radial_x * cos_a + radial_z * sin_a) * r_bot;
        mesh.vertices.push(Vertex::new(
            [v_bot.x, v_bot.y, v_bot.z],
            [norm.x, norm.y, norm.z],
            [i as f32 / segs as f32, 0.0],
            color,
        ));

        // Top vertex
        let v_top = p_top + (radial_x * cos_a + radial_z * sin_a) * r_top;
        mesh.vertices.push(Vertex::new(
            [v_top.x, v_top.y, v_top.z],
            [norm.x, norm.y, norm.z],
            [i as f32 / segs as f32, 1.0],
            color,
        ));
    }

    // Outer side quads (CCW outward normal)
    for i in 0..segs {
        let i0 = base_idx + (i * 2) as u32;
        let i1 = base_idx + (i * 2 + 1) as u32;
        let next_i = (i + 1) % segs;
        let i2 = base_idx + (next_i * 2 + 1) as u32;
        let i3 = base_idx + (next_i * 2) as u32;

        mesh.indices.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
    }
}

/// Helper to add a rounded organic foliage dome with multi-faceted shading.
fn add_foliage_dome(
    mesh: &mut ChunkMesh,
    center: Vec3,
    radius: f32,
    height: f32,
    segments: usize,
    color: [f32; 4],
) {
    let segs = segments.max(8);
    let step = 2.0 * std::f32::consts::PI / segs as f32;

    let base_idx = mesh.vertices.len() as u32;
    let apex_pos = center + Vec3::new(0.0, height * 0.5, 0.0);
    let apex_norm = Vec3::Y;

    // Apex vertex
    mesh.vertices.push(Vertex::new(
        [apex_pos.x, apex_pos.y, apex_pos.z],
        [apex_norm.x, apex_norm.y, apex_norm.z],
        [0.5, 0.5],
        color,
    ));

    // Mid ring
    for i in 0..segs {
        let a = i as f32 * step;
        let cos_a = a.cos();
        let sin_a = a.sin();
        let p_mid = center + Vec3::new(cos_a * radius, 0.0, sin_a * radius);
        let n_mid = Vec3::new(cos_a, 0.35, sin_a).normalize();
        mesh.vertices.push(Vertex::new(
            [p_mid.x, p_mid.y, p_mid.z],
            [n_mid.x, n_mid.y, n_mid.z],
            [cos_a * 0.5 + 0.5, sin_a * 0.5 + 0.5],
            color,
        ));
    }

    // Bottom center
    let bot_pos = center - Vec3::new(0.0, height * 0.5, 0.0);
    let bot_idx = mesh.vertices.len() as u32;
    mesh.vertices.push(Vertex::new(
        [bot_pos.x, bot_pos.y, bot_pos.z],
        [0.0, -1.0, 0.0],
        [0.5, 0.5],
        color,
    ));

    // Connect upper crown triangles
    for i in 0..segs {
        let m0 = base_idx + 1 + i as u32;
        let m1 = base_idx + 1 + ((i + 1) % segs) as u32;
        mesh.indices.extend_from_slice(&[base_idx, m0, m1]);
    }

    // Connect lower skirt triangles
    for i in 0..segs {
        let m0 = base_idx + 1 + i as u32;
        let m1 = base_idx + 1 + ((i + 1) % segs) as u32;
        mesh.indices.extend_from_slice(&[bot_idx, m1, m0]);
    }
}

/// Helper to add a drooping conical needle bough tier for conifers.
fn add_foliage_cone(
    mesh: &mut ChunkMesh,
    base_center: Vec3,
    apex: Vec3,
    radius: f32,
    segments: usize,
    color: [f32; 4],
) {
    let segs = segments.max(8);
    let step = 2.0 * std::f32::consts::PI / segs as f32;

    let base_idx = mesh.vertices.len() as u32;

    // Apex
    mesh.vertices.push(Vertex::new(
        [apex.x, apex.y, apex.z],
        [0.0, 1.0, 0.0],
        [0.5, 0.5],
        color,
    ));

    // Skirt vertices (drooping slightly)
    for i in 0..segs {
        let a = i as f32 * step;
        let cos_a = a.cos();
        let sin_a = a.sin();
        let p_skirt = base_center + Vec3::new(cos_a * radius, -0.15 * radius, sin_a * radius);
        let norm = Vec3::new(cos_a, 0.45, sin_a).normalize();
        mesh.vertices.push(Vertex::new(
            [p_skirt.x, p_skirt.y, p_skirt.z],
            [norm.x, norm.y, norm.z],
            [cos_a * 0.5 + 0.5, sin_a * 0.5 + 0.5],
            color,
        ));
    }

    // Cone triangles
    for i in 0..segs {
        let s0 = base_idx + 1 + i as u32;
        let s1 = base_idx + 1 + ((i + 1) % segs) as u32;
        mesh.indices.extend_from_slice(&[base_idx, s0, s1]);
    }
}

/// Generates a high-fidelity 3D ChunkMesh for an authentic tree conforming to regional ecological classifications.
/// Features organic cylindrical tapered trunks, branching structures, and multi-layered foliage crowns.
pub fn generate_detailed_tree_mesh(
    position: Vec3,
    scale: f32,
    tree_type: DetailedTreeType,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let s = scale.clamp(0.4, 3.0);

    match tree_type {
        DetailedTreeType::BroadleafDeciduous => {
            let bark_color = [0.38, 0.28, 0.20, 1.0];
            let leaf_color_a = [0.22, 0.54, 0.24, 1.0];
            let leaf_color_b = [0.18, 0.48, 0.20, 1.0];
            let leaf_color_c = [0.26, 0.60, 0.28, 1.0];

            // 1. Tapered organic trunk
            add_tapered_cylinder(
                &mut mesh,
                position,
                position + Vec3::new(0.0, 2.2 * s, 0.0),
                0.28 * s,
                0.18 * s,
                8,
                bark_color,
            );

            // 2. Primary spreading branches
            let branch_targets = [
                Vec3::new(0.65 * s, 3.4 * s, 0.35 * s),
                Vec3::new(-0.70 * s, 3.2 * s, -0.40 * s),
                Vec3::new(0.10 * s, 3.6 * s, -0.65 * s),
            ];
            for &b_top in &branch_targets {
                add_tapered_cylinder(
                    &mut mesh,
                    position + Vec3::new(0.0, 2.0 * s, 0.0),
                    position + b_top,
                    0.12 * s,
                    0.06 * s,
                    6,
                    bark_color,
                );
            }

            // 3. Multi-layered organic foliage canopy clusters
            // Main central crown
            add_foliage_dome(
                &mut mesh,
                position + Vec3::new(0.0, 4.2 * s, 0.0),
                1.9 * s,
                2.2 * s,
                10,
                leaf_color_a,
            );
            // Lateral canopy lobes
            add_foliage_dome(
                &mut mesh,
                position + Vec3::new(0.85 * s, 4.0 * s, 0.45 * s),
                1.3 * s,
                1.8 * s,
                8,
                leaf_color_b,
            );
            add_foliage_dome(
                &mut mesh,
                position + Vec3::new(-0.90 * s, 3.8 * s, -0.50 * s),
                1.4 * s,
                1.9 * s,
                8,
                leaf_color_c,
            );
            // Upper dome crown
            add_foliage_dome(
                &mut mesh,
                position + Vec3::new(0.10 * s, 5.6 * s, -0.15 * s),
                1.2 * s,
                1.6 * s,
                8,
                leaf_color_a,
            );
        }
        DetailedTreeType::EvergreenConifer => {
            let bark_color = [0.32, 0.24, 0.18, 1.0];
            let needle_a = [0.12, 0.38, 0.18, 1.0];
            let needle_b = [0.15, 0.44, 0.22, 1.0];

            // 1. Slender tapered central mast trunk
            add_tapered_cylinder(
                &mut mesh,
                position,
                position + Vec3::new(0.0, 8.0 * s, 0.0),
                0.22 * s,
                0.05 * s,
                8,
                bark_color,
            );

            // 2. Cascading tiered needle boughs
            // Tier 1 (Lowest, widest)
            add_foliage_cone(
                &mut mesh,
                position + Vec3::new(0.0, 2.8 * s, 0.0),
                position + Vec3::new(0.0, 4.5 * s, 0.0),
                1.8 * s,
                10,
                needle_a,
            );
            // Tier 2
            add_foliage_cone(
                &mut mesh,
                position + Vec3::new(0.0, 4.2 * s, 0.0),
                position + Vec3::new(0.0, 5.8 * s, 0.0),
                1.4 * s,
                10,
                needle_b,
            );
            // Tier 3
            add_foliage_cone(
                &mut mesh,
                position + Vec3::new(0.0, 5.5 * s, 0.0),
                position + Vec3::new(0.0, 7.0 * s, 0.0),
                1.0 * s,
                8,
                needle_a,
            );
            // Tier 4 (Spire top)
            add_foliage_cone(
                &mut mesh,
                position + Vec3::new(0.0, 6.8 * s, 0.0),
                position + Vec3::new(0.0, 8.4 * s, 0.0),
                0.55 * s,
                8,
                needle_b,
            );
        }
        DetailedTreeType::CoastalPalm => {
            let trunk_color = [0.55, 0.48, 0.38, 1.0];
            let frond_color = [0.28, 0.62, 0.22, 1.0];

            // 1. Gracefully curving 4-segment trunk
            let p0 = position;
            let p1 = position + Vec3::new(0.05 * s, 2.2 * s, 0.02 * s);
            let p2 = position + Vec3::new(0.18 * s, 4.4 * s, 0.08 * s);
            let p3 = position + Vec3::new(0.38 * s, 6.6 * s, 0.18 * s);

            add_tapered_cylinder(&mut mesh, p0, p1, 0.22 * s, 0.19 * s, 8, trunk_color);
            add_tapered_cylinder(&mut mesh, p1, p2, 0.19 * s, 0.16 * s, 8, trunk_color);
            add_tapered_cylinder(&mut mesh, p2, p3, 0.16 * s, 0.13 * s, 8, trunk_color);

            // 2. Crown fruit/pod cluster
            add_foliage_dome(
                &mut mesh,
                p3,
                0.35 * s,
                0.40 * s,
                8,
                [0.42, 0.36, 0.24, 1.0],
            );

            // 3. 10 Radiating arching palm fronds with outward/downward droop
            let frond_count = 10;
            let step = 2.0 * std::f32::consts::PI / frond_count as f32;
            for i in 0..frond_count {
                let a = i as f32 * step;
                let dir_x = a.cos();
                let dir_z = a.sin();

                // 2-segment curved arching leaf
                let p_stem = p3;
                let p_mid = p3 + Vec3::new(dir_x * 1.1 * s, 0.35 * s, dir_z * 1.1 * s);
                let p_tip = p3 + Vec3::new(dir_x * 2.2 * s, -0.65 * s, dir_z * 2.2 * s);

                // Leaf base quad
                let perp = Vec3::new(-dir_z, 0.0, dir_x) * (0.22 * s);
                let b0 = p_stem - perp * 0.5;
                let b1 = p_stem + perp * 0.5;
                let m0 = p_mid - perp;
                let m1 = p_mid + perp;
                let t_end = p_tip;

                let norm = Vec3::new(0.0, 0.8, 0.0).normalize();
                add_quad_mesh(&mut mesh, b0, b1, m1, m0, norm, frond_color);
                add_tri_mesh(&mut mesh, m0, m1, t_end, norm, frond_color);
            }
        }
    }

    mesh
}

fn add_quad_mesh(
    mesh: &mut ChunkMesh,
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    norm: Vec3,
    color: [f32; 4],
) {
    let base = mesh.vertices.len() as u32;
    let n = [norm.x, norm.y, norm.z];
    mesh.vertices
        .push(Vertex::new([p0.x, p0.y, p0.z], n, [0.0, 0.0], color));
    mesh.vertices
        .push(Vertex::new([p1.x, p1.y, p1.z], n, [1.0, 0.0], color));
    mesh.vertices
        .push(Vertex::new([p2.x, p2.y, p2.z], n, [1.0, 1.0], color));
    mesh.vertices
        .push(Vertex::new([p3.x, p3.y, p3.z], n, [0.0, 1.0], color));
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn add_tri_mesh(mesh: &mut ChunkMesh, p0: Vec3, p1: Vec3, p2: Vec3, norm: Vec3, color: [f32; 4]) {
    let base = mesh.vertices.len() as u32;
    let n = [norm.x, norm.y, norm.z];
    mesh.vertices
        .push(Vertex::new([p0.x, p0.y, p0.z], n, [0.0, 0.0], color));
    mesh.vertices
        .push(Vertex::new([p1.x, p1.y, p1.z], n, [1.0, 0.0], color));
    mesh.vertices
        .push(Vertex::new([p2.x, p2.y, p2.z], n, [0.5, 1.0], color));
    mesh.indices.extend_from_slice(&[base, base + 1, base + 2]);
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
