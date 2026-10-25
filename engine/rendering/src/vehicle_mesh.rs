//! 3D Procedural vehicle mesh generation for drivable cars and AI traffic.

use crate::gltf_asset::VehicleLodLevel;
use crate::vehicle_models::get_certified_vehicle_model;
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
    Motorcycle,
    TrainLocomotive,
    TrainWagon,
    MarineVessel,
    CommercialAirliner,
    GeneralAviation,
}

impl VehicleVisualType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sedan => "Executive Sedan",
            Self::SportsCoupe => "Apex GT Sports Coupe",
            Self::Suv => "Overland Explorer SUV",
            Self::TransitBus => "CityLine Transit Bus",
            Self::HeavyTruck => "TransContinental Semi",
            Self::Taxi => "Metro Yellow Cab",
            Self::Motorcycle => "HyperSport RR-1000",
            Self::TrainLocomotive => "Pacific Locomotive",
            Self::TrainWagon => "Passenger Coach",
            Self::MarineVessel => "Golden Gate Ferry",
            Self::CommercialAirliner => "SkyCourier B737",
            Self::GeneralAviation => "Skyhawk C172",
        }
    }
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
    pub trailer_angle: f32,
    pub wheel_spin_angle: f32,
    pub wheel_compressions: [f32; 4],
    pub lod: VehicleLodLevel,
    /// Passenger door opening animation ratio [0.0 = closed, 1.0 = fully open].
    pub door_open_ratio: f32,
    /// Front and rear railway truck / bogie yaw articulation angles [rad].
    pub bogie_angles: [f32; 2],
    /// Propeller or turbofan fan blade rotation angle [rad].
    pub propeller_angle: f32,
    /// Aircraft flight control surfaces: [elevator pitch, aileron roll, rudder yaw] [rad].
    pub control_surfaces: [f32; 3],
    /// Motorcycle dynamic cornering bank / roll lean angle [rad].
    pub motorcycle_lean_angle: f32,
}

impl Default for VehicleMeshParams {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            color: [0.85, 0.15, 0.15, 1.0], // Sport red
            vehicle_type: VehicleVisualType::SportsCoupe,
            wheel_steer_angle: 0.0,
            headlights_on: true,
            brake_lights_on: false,
            reverse_lights_on: false,
            trailer_angle: 0.0,
            wheel_spin_angle: 0.0,
            wheel_compressions: [0.0; 4],
            lod: VehicleLodLevel::Lod0High,
            door_open_ratio: 0.0,
            bogie_angles: [0.0, 0.0],
            propeller_angle: 0.0,
            control_surfaces: [0.0, 0.0, 0.0],
            motorcycle_lean_angle: 0.0,
        }
    }
}

/// Generates a complete 3D ChunkMesh for a vehicle using certified production models
/// and multimodal physical animations.
pub fn generate_vehicle_mesh(params: &VehicleMeshParams) -> ChunkMesh {
    let model = get_certified_vehicle_model(params.vehicle_type, params.lod);
    model.compose_multimodal_world_mesh(
        params.position,
        params.orientation,
        Some(params.color),
        params.wheel_steer_angle,
        params.wheel_spin_angle,
        params.wheel_compressions,
        params.headlights_on,
        params.brake_lights_on,
        params.reverse_lights_on,
        params.door_open_ratio,
        params.bogie_angles,
        params.propeller_angle,
        params.control_surfaces,
        params.motorcycle_lean_angle,
        params.trailer_angle,
    )
}

/// Development fallback generator creating coarse primitive boxes.
/// Labeled for diagnostics, automated regression comparisons, and fallback pipelines.
pub fn generate_fallback_development_mesh(params: &VehicleMeshParams) -> ChunkMesh {
    match params.vehicle_type {
        VehicleVisualType::Motorcycle => return generate_motorcycle_mesh(params),
        VehicleVisualType::TrainLocomotive => return generate_train_locomotive_mesh(params),
        VehicleVisualType::TrainWagon => return generate_train_wagon_mesh(params),
        VehicleVisualType::MarineVessel => return generate_marine_vessel_mesh(params),
        VehicleVisualType::CommercialAirliner => return generate_commercial_airliner_mesh(params),
        VehicleVisualType::GeneralAviation => return generate_general_aviation_mesh(params),
        _ => {}
    }

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
            _ => (4.6, 1.80, 0.80, 2.4, 1.60, 0.55, -0.1),
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
            let hitch_z = 1.0;
            let trailer_rot = Quat::from_rotation_y(params.trailer_angle);
            let trailer_center = Vec3::new(0.0, chassis_bottom + trailer_h * 0.5, hitch_z)
                + trailer_rot.mul_vec3(Vec3::new(0.0, 0.0, -trailer_l * 0.5));
            add_oriented_box(
                &mut mesh,
                params,
                trailer_center,
                trailer_rot,
                Vec3::new(trailer_w * 0.5, trailer_h * 0.5, trailer_l * 0.5),
                [0.85, 0.88, 0.92, 1.0], // Clean white freight container
            );

            // Trailer rear wheels (dual bogie axles at tail of trailer)
            let trailer_axle_z1 = trailer_center
                + trailer_rot.mul_vec3(Vec3::new(0.0, -trailer_h * 0.5 + 0.35, -trailer_l * 0.35));
            let trailer_axle_z2 = trailer_center
                + trailer_rot.mul_vec3(Vec3::new(0.0, -trailer_h * 0.5 + 0.35, -trailer_l * 0.45));
            for axle_pos in [trailer_axle_z1, trailer_axle_z2] {
                add_oriented_box(
                    &mut mesh,
                    params,
                    axle_pos + trailer_rot.mul_vec3(Vec3::new(-trailer_w * 0.5, 0.0, 0.0)),
                    trailer_rot,
                    Vec3::new(0.12, 0.35, 0.35),
                    wheel_rubber,
                );
                add_oriented_box(
                    &mut mesh,
                    params,
                    axle_pos + trailer_rot.mul_vec3(Vec3::new(trailer_w * 0.5, 0.0, 0.0)),
                    trailer_rot,
                    Vec3::new(0.12, 0.35, 0.35),
                    wheel_rubber,
                );
            }

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
        _ => {}
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

fn generate_motorcycle_mesh(params: &VehicleMeshParams) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let frame_color = [0.15, 0.16, 0.18, 1.0];
    let body_color = params.color;
    let chrome_color = [0.85, 0.88, 0.92, 1.0];
    let rubber_color = [0.10, 0.10, 0.12, 1.0];
    let rim_color = [0.75, 0.76, 0.78, 1.0];
    let headlight_color = if params.headlights_on {
        [1.0, 1.0, 0.88, 1.0]
    } else {
        [0.3, 0.3, 0.3, 1.0]
    };
    let taillight_color = if params.brake_lights_on {
        [1.0, 0.05, 0.05, 1.0]
    } else {
        [0.6, 0.05, 0.05, 1.0]
    };

    // 1. Tubular Double-Cradle Frame & Engine Block
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.42, 0.0),
        Vec3::new(0.16, 0.18, 0.35),
        frame_color,
    );
    // Metallic engine crankcase & finned cylinders
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.38, 0.05),
        Vec3::new(0.18, 0.14, 0.22),
        chrome_color,
    );

    // 2. Teardrop Fuel Tank & Fairing
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.68, 0.20),
        Vec3::new(0.18, 0.12, 0.28),
        body_color,
    );

    // 3. Saddle Seat & Rear Tail Cowl
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.60, -0.22),
        Vec3::new(0.15, 0.06, 0.22),
        [0.12, 0.12, 0.14, 1.0],
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.64, -0.50),
        Vec3::new(0.12, 0.08, 0.15),
        body_color,
    );

    // 4. Rider Figure (Torso, Helmet, Limbs)
    // Rider Torso leaning forward slightly into wind
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.95, -0.15),
        Vec3::new(0.20, 0.24, 0.14),
        [0.18, 0.22, 0.28, 1.0],
    );
    // Rider Helmet
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 1.28, -0.05),
        Vec3::new(0.12, 0.12, 0.13),
        body_color,
    );
    // Dark helmet visor
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 1.28, 0.07),
        Vec3::new(0.10, 0.05, 0.03),
        [0.05, 0.05, 0.06, 1.0],
    );

    // 5. Front Steering Fork & Handlebars
    let steer_rot = Quat::from_rotation_y(params.wheel_steer_angle);
    let fork_tilt = Quat::from_rotation_x(0.38); // ~22 degree rake angle
    let combined_fork_rot = steer_rot * fork_tilt;

    // Dual front fork tubes
    add_oriented_box(
        &mut mesh,
        params,
        Vec3::new(-0.11, 0.52, 0.58),
        combined_fork_rot,
        Vec3::new(0.025, 0.35, 0.025),
        chrome_color,
    );
    add_oriented_box(
        &mut mesh,
        params,
        Vec3::new(0.11, 0.52, 0.58),
        combined_fork_rot,
        Vec3::new(0.025, 0.35, 0.025),
        chrome_color,
    );

    // Handlebars with rubber grips
    let bar_center = Vec3::new(0.0, 0.88, 0.45);
    add_oriented_box(
        &mut mesh,
        params,
        bar_center,
        steer_rot,
        Vec3::new(0.38, 0.02, 0.02),
        chrome_color,
    );
    // Left & right rubber grips
    add_oriented_box(
        &mut mesh,
        params,
        bar_center + steer_rot.mul_vec3(Vec3::new(-0.35, 0.0, 0.0)),
        steer_rot,
        Vec3::new(0.06, 0.025, 0.025),
        rubber_color,
    );
    add_oriented_box(
        &mut mesh,
        params,
        bar_center + steer_rot.mul_vec3(Vec3::new(0.35, 0.0, 0.0)),
        steer_rot,
        Vec3::new(0.06, 0.025, 0.025),
        rubber_color,
    );

    // 6. Front Wheel (Single steered 17" motorcycle wheel, r=0.30m, w=0.12m)
    let front_wheel_pos = Vec3::new(0.0, 0.30, 0.72);
    add_oriented_box(
        &mut mesh,
        params,
        front_wheel_pos,
        steer_rot,
        Vec3::new(0.06, 0.30, 0.30),
        rubber_color,
    );
    add_oriented_box(
        &mut mesh,
        params,
        front_wheel_pos,
        steer_rot,
        Vec3::new(0.07, 0.18, 0.18),
        rim_color,
    );

    // 7. Rear Wheel & Swingarm (r=0.30m, w=0.16m)
    let rear_wheel_pos = Vec3::new(0.0, 0.30, -0.65);
    add_box(
        &mut mesh,
        params,
        rear_wheel_pos,
        Vec3::new(0.08, 0.30, 0.30),
        rubber_color,
    );
    add_box(
        &mut mesh,
        params,
        rear_wheel_pos,
        Vec3::new(0.09, 0.18, 0.18),
        rim_color,
    );
    // Rear swingarm bars
    add_box(
        &mut mesh,
        params,
        Vec3::new(-0.10, 0.30, -0.35),
        Vec3::new(0.02, 0.04, 0.32),
        chrome_color,
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.10, 0.30, -0.35),
        Vec3::new(0.02, 0.04, 0.32),
        chrome_color,
    );

    // 8. Chrome Swept Exhaust Pipe & Muffler
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.18, 0.32, -0.30),
        Vec3::new(0.05, 0.05, 0.42),
        chrome_color,
    );

    // 9. Front Headlight & Rear Taillight
    add_quad(
        &mut mesh,
        params,
        Vec3::new(-0.10, 0.75, 0.62),
        Vec3::new(0.10, 0.75, 0.62),
        Vec3::new(0.10, 0.90, 0.62),
        Vec3::new(-0.10, 0.90, 0.62),
        Vec3::Z,
        headlight_color,
    );
    add_quad(
        &mut mesh,
        params,
        Vec3::new(0.08, 0.65, -0.66),
        Vec3::new(-0.08, 0.65, -0.66),
        Vec3::new(-0.08, 0.72, -0.66),
        Vec3::new(0.08, 0.72, -0.66),
        -Vec3::Z,
        taillight_color,
    );

    mesh
}

fn generate_train_locomotive_mesh(params: &VehicleMeshParams) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let body_color = params.color;
    let chassis_steel = [0.18, 0.20, 0.22, 1.0];
    let glass_color = [0.15, 0.25, 0.35, 0.95];
    let wheel_steel = [0.35, 0.38, 0.42, 1.0];
    let roof_equipment = [0.55, 0.58, 0.62, 1.0];
    let yellow_warning = [0.95, 0.80, 0.12, 1.0];

    // Locomotive Dimensions: Length 18.0m, Width 3.0m, Height 3.8m
    let half_l = 9.0;
    let half_w = 1.5;

    // 1. Main Locomotive Carbody
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 2.3, 0.0),
        Vec3::new(half_w, 1.3, half_l * 0.95),
        body_color,
    );
    // Lower Steel Underframe Chassis Sill
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.90, 0.0),
        Vec3::new(half_w * 1.02, 0.25, half_l),
        chassis_steel,
    );

    // 2. High Cab Windshield (Front & Rear Dual-Cab design)
    // Forward Cab Windshield (+Z)
    add_quad(
        &mut mesh,
        params,
        Vec3::new(-half_w * 0.75, 2.7, half_l * 0.96),
        Vec3::new(half_w * 0.75, 2.7, half_l * 0.96),
        Vec3::new(half_w * 0.75, 3.4, half_l * 0.96),
        Vec3::new(-half_w * 0.75, 3.4, half_l * 0.96),
        Vec3::Z,
        glass_color,
    );
    // Rear Cab Windshield (-Z)
    add_quad(
        &mut mesh,
        params,
        Vec3::new(half_w * 0.75, 2.7, -half_l * 0.96),
        Vec3::new(-half_w * 0.75, 2.7, -half_l * 0.96),
        Vec3::new(-half_w * 0.75, 3.4, -half_l * 0.96),
        Vec3::new(half_w * 0.75, 3.4, -half_l * 0.96),
        -Vec3::Z,
        glass_color,
    );

    // 3. Roof Pantograph / Dynamic Brake Blister & Resistor Radiators
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 3.75, 0.0),
        Vec3::new(1.1, 0.15, 6.0),
        roof_equipment,
    );
    // Pantographs at front and rear roof
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 4.05, 5.5),
        Vec3::new(0.9, 0.20, 0.8),
        [0.85, 0.20, 0.20, 1.0],
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 4.05, -5.5),
        Vec3::new(0.9, 0.20, 0.8),
        [0.85, 0.20, 0.20, 1.0],
    );

    // 4. Cowcatcher / Pilot Plow Wedge (Front & Rear)
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.40, half_l + 0.35),
        Vec3::new(half_w * 0.90, 0.25, 0.35),
        yellow_warning,
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.40, -half_l - 0.35),
        Vec3::new(half_w * 0.90, 0.25, 0.35),
        chassis_steel,
    );

    // 5. Heavy Steel Wheel Bogies (Two 3-axle bogies: front Z=+5.0, rear Z=-5.0)
    let bogie_z_centers = [5.0, -5.0];
    let wheel_r = 0.52;
    let wheel_w = 0.16;
    for &bz in &bogie_z_centers {
        // Bogie frame
        add_box(
            &mut mesh,
            params,
            Vec3::new(0.0, 0.60, bz),
            Vec3::new(half_w * 0.85, 0.15, 2.4),
            chassis_steel,
        );
        // 3 Axles per bogie (-1.8m, 0.0m, +1.8m)
        for &axle_offset in &[-1.8, 0.0, 1.8] {
            let wz = bz + axle_offset;
            // Left steel wheel
            add_box(
                &mut mesh,
                params,
                Vec3::new(-half_w * 0.88, wheel_r, wz),
                Vec3::new(wheel_w * 0.5, wheel_r, wheel_r),
                wheel_steel,
            );
            // Right steel wheel
            add_box(
                &mut mesh,
                params,
                Vec3::new(half_w * 0.88, wheel_r, wz),
                Vec3::new(wheel_w * 0.5, wheel_r, wheel_r),
                wheel_steel,
            );
            // Steel axle rod
            add_box(
                &mut mesh,
                params,
                Vec3::new(0.0, wheel_r, wz),
                Vec3::new(half_w * 0.85, 0.06, 0.06),
                chassis_steel,
            );
        }
    }

    // 6. High-Intensity Locomotive Twin Headlights (Front & Marker Lights Rear)
    let hl_color = if params.headlights_on {
        [1.0, 1.0, 0.90, 1.0]
    } else {
        [0.35, 0.35, 0.35, 1.0]
    };
    add_quad(
        &mut mesh,
        params,
        Vec3::new(-0.55, 2.1, half_l + 0.02),
        Vec3::new(-0.15, 2.1, half_l + 0.02),
        Vec3::new(-0.15, 2.5, half_l + 0.02),
        Vec3::new(-0.55, 2.5, half_l + 0.02),
        Vec3::Z,
        hl_color,
    );
    add_quad(
        &mut mesh,
        params,
        Vec3::new(0.15, 2.1, half_l + 0.02),
        Vec3::new(0.55, 2.1, half_l + 0.02),
        Vec3::new(0.55, 2.5, half_l + 0.02),
        Vec3::new(0.15, 2.5, half_l + 0.02),
        Vec3::Z,
        hl_color,
    );

    mesh
}

fn generate_train_wagon_mesh(params: &VehicleMeshParams) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let body_color = [0.22, 0.45, 0.72, 1.0]; // Indian Railways / Amtrak Blue
    let chassis_steel = [0.18, 0.20, 0.22, 1.0];
    let roof_ribbed = [0.75, 0.78, 0.82, 1.0];
    let window_glass = [0.12, 0.18, 0.25, 0.95];
    let wheel_steel = [0.35, 0.38, 0.42, 1.0];

    // Passenger Coach: Length 21.0m, Width 3.0m, Height 3.6m
    let half_l = 10.5;
    let half_w = 1.5;

    // 1. Coach Carbody
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 2.2, 0.0),
        Vec3::new(half_w, 1.25, half_l * 0.98),
        body_color,
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.85, 0.0),
        Vec3::new(half_w * 1.01, 0.20, half_l),
        chassis_steel,
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 3.52, 0.0),
        Vec3::new(half_w * 0.98, 0.12, half_l * 0.98),
        roof_ribbed,
    );

    // 2. Long Window Band along left and right sides
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 2.3, 0.0),
        Vec3::new(half_w + 0.015, 0.38, half_l * 0.88),
        window_glass,
    );

    // 3. Dual Bogies (2-axle per bogie, Z=±7.2m)
    let wheel_r = 0.46;
    let wheel_w = 0.15;
    for &bz in &[7.2, -7.2] {
        add_box(
            &mut mesh,
            params,
            Vec3::new(0.0, 0.55, bz),
            Vec3::new(half_w * 0.85, 0.12, 1.6),
            chassis_steel,
        );
        for &axle_offset in &[-1.1, 1.1] {
            let wz = bz + axle_offset;
            add_box(
                &mut mesh,
                params,
                Vec3::new(-half_w * 0.88, wheel_r, wz),
                Vec3::new(wheel_w * 0.5, wheel_r, wheel_r),
                wheel_steel,
            );
            add_box(
                &mut mesh,
                params,
                Vec3::new(half_w * 0.88, wheel_r, wz),
                Vec3::new(wheel_w * 0.5, wheel_r, wheel_r),
                wheel_steel,
            );
        }
    }

    // 4. Gangway / Vestibule Connections at Coach Ends
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 2.0, half_l + 0.15),
        Vec3::new(0.65, 1.0, 0.15),
        [0.10, 0.10, 0.10, 1.0],
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 2.0, -half_l - 0.15),
        Vec3::new(0.65, 1.0, 0.15),
        [0.10, 0.10, 0.10, 1.0],
    );

    mesh
}

fn generate_marine_vessel_mesh(params: &VehicleMeshParams) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let hull_topside = [0.12, 0.18, 0.26, 1.0]; // Deep Marine Navy Blue
    let hull_antifouling = [0.72, 0.18, 0.15, 1.0]; // Oxide Red below waterline
    let deck_wood = [0.65, 0.55, 0.42, 1.0];
    let superstructure_white = [0.92, 0.94, 0.96, 1.0];
    let bridge_glass = [0.15, 0.28, 0.40, 0.95];

    // Vessel Dimensions: Length 50m, Beam 11m, Depth 7m
    let half_l = 25.0;
    let half_w = 5.5;

    // 1. Lower Hull (below water datum, y from -3.5 to 0.0)
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, -1.75, 0.0),
        Vec3::new(half_w * 0.92, 1.75, half_l * 0.95),
        hull_antifouling,
    );
    // Bulbous bow protruding forward at front waterline
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, -2.2, half_l + 1.8),
        Vec3::new(1.8, 1.2, 2.0),
        hull_antifouling,
    );

    // 2. Freeboard Topsides (above water datum, y from 0.0 to 3.5)
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 1.75, 0.0),
        Vec3::new(half_w, 1.75, half_l),
        hull_topside,
    );
    // Main weather deck
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 3.52, 0.0),
        Vec3::new(half_w * 0.98, 0.05, half_l * 0.98),
        deck_wood,
    );

    // 3. Multi-Tier Bridge Superstructure & Nav Deck (Located aft at Z = -10.0m)
    let bridge_z = -10.0;
    // Lower deckhouse
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 5.2, bridge_z),
        Vec3::new(half_w * 0.85, 1.6, 6.5),
        superstructure_white,
    );
    // Upper navigation bridge with panoramic bridge wings
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 7.8, bridge_z + 1.0),
        Vec3::new(half_w * 0.95, 1.0, 4.0),
        superstructure_white,
    );
    // Wrap-around bridge windows facing forward
    add_quad(
        &mut mesh,
        params,
        Vec3::new(-half_w * 0.88, 7.3, bridge_z + 3.02),
        Vec3::new(half_w * 0.88, 7.3, bridge_z + 3.02),
        Vec3::new(half_w * 0.88, 8.4, bridge_z + 3.02),
        Vec3::new(-half_w * 0.88, 8.4, bridge_z + 3.02),
        Vec3::Z,
        bridge_glass,
    );

    // Radar Mast and Communication Antennas on Bridge Roof
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 10.5, bridge_z + 1.0),
        Vec3::new(0.18, 1.8, 0.18),
        superstructure_white,
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 12.0, bridge_z + 1.0),
        Vec3::new(1.4, 0.08, 0.12),
        [0.85, 0.85, 0.15, 1.0],
    );

    // 4. Cargo Deck Container Stacks (Forward Deck Z = 2.0 to 18.0)
    let container_colors = [
        [0.85, 0.22, 0.18, 1.0], // Maersk Red
        [0.15, 0.42, 0.78, 1.0], // Ocean Blue
        [0.18, 0.65, 0.35, 1.0], // Evergreen Green
        [0.88, 0.62, 0.12, 1.0], // Hapag Orange
    ];
    for row in 0..3 {
        let cz = 3.0 + (row as f32 * 6.2);
        for col in -1_i32..=1_i32 {
            let cx = col as f32 * 2.8;
            let c_color =
                container_colors[(row + col.unsigned_abs() as usize) % container_colors.len()];
            // 2-tier stacked shipping containers (2.4m wide x 2.6m high x 6.0m long)
            add_box(
                &mut mesh,
                params,
                Vec3::new(cx, 4.85, cz),
                Vec3::new(1.2, 1.3, 2.9),
                c_color,
            );
            add_box(
                &mut mesh,
                params,
                Vec3::new(cx, 7.45, cz),
                Vec3::new(1.2, 1.3, 2.9),
                container_colors[(row + 2) % container_colors.len()],
            );
        }
    }

    // 5. Stern Rudder & Propeller Housing
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, -1.8, -half_l - 0.6),
        Vec3::new(0.12, 1.2, 0.8),
        [0.15, 0.15, 0.18, 1.0],
    );

    mesh
}

fn generate_commercial_airliner_mesh(params: &VehicleMeshParams) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let fuselage_white = [0.95, 0.96, 0.98, 1.0];
    let wing_gray = [0.72, 0.74, 0.78, 1.0];
    let livery_accent = params.color;
    let glass_color = [0.12, 0.18, 0.28, 0.95];
    let engine_cowl = [0.88, 0.90, 0.92, 1.0];
    let engine_interior = [0.10, 0.10, 0.12, 1.0];
    let tire_rubber = [0.12, 0.12, 0.14, 1.0];

    // Airliner Dimensions (A320/B737): Length 34m, Wingspan 34m, Tail Height 11m
    let half_l = 17.0;
    let fuse_r = 1.9;

    // 1. Aerodynamic Main Fuselage Tube
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, fuse_r, 0.0),
        Vec3::new(fuse_r, fuse_r, half_l * 0.85),
        fuselage_white,
    );
    // Aerodynamic Nose Cone (+Z)
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, fuse_r * 0.9, half_l * 0.92),
        Vec3::new(fuse_r * 0.75, fuse_r * 0.75, half_l * 0.12),
        fuselage_white,
    );
    // Cockpit Windshield Visor
    add_quad(
        &mut mesh,
        params,
        Vec3::new(-fuse_r * 0.55, fuse_r * 1.35, half_l * 0.90),
        Vec3::new(fuse_r * 0.55, fuse_r * 1.35, half_l * 0.90),
        Vec3::new(fuse_r * 0.50, fuse_r * 1.65, half_l * 0.84),
        Vec3::new(-fuse_r * 0.50, fuse_r * 1.65, half_l * 0.84),
        Vec3::new(0.0, 0.5, 0.85).normalize(),
        glass_color,
    );

    // 2. Swept Main Wings (Span 34m, 25 degree sweep, dihedral)
    let wing_span_half = 16.5;
    let wing_root_chord = 5.2;
    // Left Wing (-X)
    add_oriented_box(
        &mut mesh,
        params,
        Vec3::new(-wing_span_half * 0.5, fuse_r * 0.65, -0.8),
        Quat::from_rotation_y(0.24) * Quat::from_rotation_z(0.05),
        Vec3::new(wing_span_half * 0.5, 0.22, wing_root_chord * 0.5),
        wing_gray,
    );
    // Right Wing (+X)
    add_oriented_box(
        &mut mesh,
        params,
        Vec3::new(wing_span_half * 0.5, fuse_r * 0.65, -0.8),
        Quat::from_rotation_y(-0.24) * Quat::from_rotation_z(-0.05),
        Vec3::new(wing_span_half * 0.5, 0.22, wing_root_chord * 0.5),
        wing_gray,
    );
    // Vertical Winglets at Wingtips
    add_box(
        &mut mesh,
        params,
        Vec3::new(-wing_span_half - 0.1, fuse_r * 0.65 + 0.8, -2.6),
        Vec3::new(0.06, 0.8, 0.6),
        livery_accent,
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(wing_span_half + 0.1, fuse_r * 0.65 + 0.8, -2.6),
        Vec3::new(0.06, 0.8, 0.6),
        livery_accent,
    );

    // 3. Turbofan Jet Engines on Wing Pylons (Left & Right)
    for &side in &[-5.5, 5.5] {
        // Engine Pylon
        add_box(
            &mut mesh,
            params,
            Vec3::new(side, fuse_r * 0.35, 0.5),
            Vec3::new(0.15, 0.40, 1.2),
            wing_gray,
        );
        // Engine Nacelle Cowling
        add_box(
            &mut mesh,
            params,
            Vec3::new(side, fuse_r * 0.05, 0.5),
            Vec3::new(1.0, 1.0, 2.2),
            engine_cowl,
        );
        // Engine Intake Front Face & Fan Spinner
        add_quad(
            &mut mesh,
            params,
            Vec3::new(side - 0.85, fuse_r * 0.05 - 0.85, 1.62),
            Vec3::new(side + 0.85, fuse_r * 0.05 - 0.85, 1.62),
            Vec3::new(side + 0.85, fuse_r * 0.05 + 0.85, 1.62),
            Vec3::new(side - 0.85, fuse_r * 0.05 + 0.85, 1.62),
            Vec3::Z,
            engine_interior,
        );
    }

    // 4. Empennage: Swept Vertical Tailfin & Horizontal Stabilizers
    // Vertical Stabilizer Fin (Height 6.5m)
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, fuse_r + 3.4, -half_l * 0.80),
        Vec3::new(0.20, 3.2, 3.2),
        livery_accent,
    );
    // Horizontal Stabilizers (Span 11m)
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, fuse_r + 0.8, -half_l * 0.90),
        Vec3::new(5.5, 0.16, 2.0),
        wing_gray,
    );

    // 5. Tricycle Landing Gear
    // Nose gear
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.40, half_l * 0.70),
        Vec3::new(0.08, 0.50, 0.08),
        [0.3, 0.3, 0.3, 1.0],
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.25, half_l * 0.70),
        Vec3::new(0.25, 0.25, 0.25),
        tire_rubber,
    );
    // Main dual gear
    for &side in &[-3.2, 3.2] {
        add_box(
            &mut mesh,
            params,
            Vec3::new(side, 0.40, -1.0),
            Vec3::new(0.10, 0.50, 0.10),
            [0.3, 0.3, 0.3, 1.0],
        );
        add_box(
            &mut mesh,
            params,
            Vec3::new(side, 0.35, -1.0),
            Vec3::new(0.40, 0.35, 0.35),
            tire_rubber,
        );
    }

    mesh
}

fn generate_general_aviation_mesh(params: &VehicleMeshParams) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let body_color = params.color;
    let wing_color = [0.94, 0.94, 0.96, 1.0];
    let glass_color = [0.15, 0.25, 0.38, 0.95];
    let prop_spinner = [0.85, 0.15, 0.15, 1.0];
    let tire_rubber = [0.12, 0.12, 0.14, 1.0];

    // Cessna 172 Dimensions: Length 8.2m, Wingspan 11.0m, Height 2.7m
    let half_l = 4.1;
    let cabin_w = 0.65;

    // 1. Fuselage
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 1.25, -0.2),
        Vec3::new(cabin_w, 0.65, half_l * 0.85),
        body_color,
    );
    // Nose engine cowling
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 1.15, half_l * 0.90),
        Vec3::new(0.55, 0.50, 0.55),
        body_color,
    );
    // Cabin windshield and side windows
    add_quad(
        &mut mesh,
        params,
        Vec3::new(-cabin_w * 0.95, 1.45, 1.4),
        Vec3::new(cabin_w * 0.95, 1.45, 1.4),
        Vec3::new(cabin_w * 0.85, 1.90, 0.8),
        Vec3::new(-cabin_w * 0.85, 1.90, 0.8),
        Vec3::new(0.0, 0.45, 0.89).normalize(),
        glass_color,
    );

    // 2. High Mounted Wings with Wing Struts (Span 11.0m, Y = 1.95m)
    let wing_span_half = 5.5;
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 1.95, 0.2),
        Vec3::new(wing_span_half, 0.08, 0.85),
        wing_color,
    );
    // Diagonal wing struts
    add_box(
        &mut mesh,
        params,
        Vec3::new(-2.2, 1.45, 0.2),
        Vec3::new(0.04, 0.45, 0.04),
        [0.3, 0.3, 0.3, 1.0],
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(2.2, 1.45, 0.2),
        Vec3::new(0.04, 0.45, 0.04),
        [0.3, 0.3, 0.3, 1.0],
    );

    // 3. Nose Propeller Spinner & 2-Blade Propeller
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 1.15, half_l + 0.35),
        Vec3::new(0.16, 0.16, 0.18),
        prop_spinner,
    );
    // Spinning 2-blade prop disk
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 1.15, half_l + 0.45),
        Vec3::new(0.95, 0.06, 0.02),
        [0.15, 0.15, 0.15, 1.0],
    );

    // 4. Tail Empennage
    // Vertical tailfin
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 2.05, -half_l * 0.88),
        Vec3::new(0.08, 0.85, 0.85),
        body_color,
    );
    // Horizontal tailplane
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 1.55, -half_l * 0.95),
        Vec3::new(1.8, 0.06, 0.55),
        wing_color,
    );

    // 5. Fixed Tricycle Landing Gear with Wheel Fairings
    // Nose gear
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.45, 2.2),
        Vec3::new(0.05, 0.35, 0.05),
        [0.3, 0.3, 0.3, 1.0],
    );
    add_box(
        &mut mesh,
        params,
        Vec3::new(0.0, 0.22, 2.2),
        Vec3::new(0.12, 0.22, 0.24),
        tire_rubber,
    );
    // Main dual gear
    for &side in &[-1.1, 1.1] {
        add_box(
            &mut mesh,
            params,
            Vec3::new(side, 0.45, 0.0),
            Vec3::new(0.06, 0.35, 0.06),
            [0.3, 0.3, 0.3, 1.0],
        );
        add_box(
            &mut mesh,
            params,
            Vec3::new(side, 0.22, 0.0),
            Vec3::new(0.12, 0.22, 0.24),
            tire_rubber,
        );
    }

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
            VehicleVisualType::Motorcycle,
            VehicleVisualType::TrainLocomotive,
            VehicleVisualType::TrainWagon,
            VehicleVisualType::MarineVessel,
            VehicleVisualType::CommercialAirliner,
            VehicleVisualType::GeneralAviation,
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
                trailer_angle: 0.15,
                ..Default::default()
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
                    (vert.position[0] - 10.0).abs() < 30.0,
                    "Vertex X within expected bounds for {:?}",
                    vtype
                );
                assert!(
                    (vert.position[1] - 2.0).abs() < 20.0,
                    "Vertex Y within expected bounds for {:?}",
                    vtype
                );
                assert!(
                    (vert.position[2] - 30.0).abs() < 35.0,
                    "Vertex Z within expected bounds for {:?}",
                    vtype
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
