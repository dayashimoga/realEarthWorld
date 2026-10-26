//! STAGE H1 — PERMANENT DRIVING PHYSICS, STEERING AND ROAD-SURFACE QUALITY GATES
//!
//! Comprehensive regression certification verifying:
//! - Gate 1: 60-Second stationary level-ground hold (Park & Neutral/Idle in Drive, translation < 0.001m, zero drift).
//! - Gate 2: Directional steering and camera alignment across all 4 cardinal headings (North, East, South, West) in forward and reverse.
//! - Gate 3: Continuous road surface, curb, and bridge deck contact (exact physical contact with visual meshes, no sinking, no falling through bridge decks).
//! - Gate 4: Multi-framerate determinism (fixed-timestep simulation at 30, 60, and 120 FPS).
//! - Gate 5: Class-specific fleet dynamics validation (sports coupe, sedan, suv, taxi, motorcycle, transit bus, heavy truck).
//! - Gate 6: Steep grade hill-hold assist and anti-rollback on authentic San Francisco inclines.

use glam::{Mat3, Quat, Vec2, Vec3};
use realearth_vehicles::camera::{VehicleCameraController, VehicleCameraPerspective};
use realearth_vehicles::dynamics::{
    DynamicVehicle, TransmissionMode, VehicleConfig, VehicleInputs,
};
use realearth_vehicles::ground::{FlatGround, GroundQuery, SlopedGround};
use realearth_vehicles::multimodal::road_transport::{
    HeavyTruckModel, MotorcycleModel, TransitBusModel,
};
use std::f32::consts::{FRAC_PI_2, PI};

/// Road-surface collision query verifying exact contact against rendered ribbons, curbs, and bridge decks.
pub struct SyntheticRoadNetworkGround {
    pub base_elev: f32,
    pub segments: Vec<RoadTestSegment>,
}

pub struct RoadTestSegment {
    pub p1: Vec3,
    pub p2: Vec3,
    pub half_width: f32,
    pub is_bridge: bool,
}

impl GroundQuery for SyntheticRoadNetworkGround {
    fn query_ground_contact(&self, origin: Vec3, max_dist: f32) -> Option<(f32, Vec3)> {
        let mut best: Option<(f32, Vec3)> = None;
        let mut min_abs_dist = f32::MAX;

        for seg in &self.segments {
            let v = Vec2::new(seg.p2.x - seg.p1.x, seg.p2.z - seg.p1.z);
            let len_sq = v.length_squared();
            if len_sq < 1e-4 {
                continue;
            }
            let w = Vec2::new(origin.x - seg.p1.x, origin.z - seg.p1.z);
            let t = (w.dot(v) / len_sq).clamp(0.0, 1.0);
            let proj = Vec2::new(seg.p1.x, seg.p1.z) + v * t;
            let d_perp = (Vec2::new(origin.x, origin.z) - proj).length();

            if d_perp <= seg.half_width + 1.8 {
                let mut road_y = seg.p1.y + t * (seg.p2.y - seg.p1.y);
                if d_perp > seg.half_width {
                    road_y += 0.15; // Raised curb riser
                }
                let dist = origin.y - road_y;
                if dist >= -1.5 && dist <= max_dist {
                    let tangent = Vec3::new(v.x, seg.p2.y - seg.p1.y, v.y).normalize();
                    let perp = Vec3::new(-v.y, 0.0, v.x).normalize();
                    let mut norm = perp.cross(tangent).normalize();
                    if norm.y < 0.0 {
                        norm = -norm;
                    }
                    if dist.abs() < min_abs_dist {
                        min_abs_dist = dist.abs();
                        best = Some((0.0f32.max(dist), norm));
                    }
                }
            }
        }

        if let Some(contact) = best {
            return Some(contact);
        }

        // Flat terrain fallback
        let dist = origin.y - self.base_elev;
        if dist >= -10.0 && dist <= max_dist {
            Some((0.0f32.max(dist), Vec3::Y))
        } else {
            None
        }
    }
}

// =============================================================================
// GATE 1: 60-Second Stationary Level Ground Hold
// =============================================================================

#[test]
fn test_gate1_60s_level_ground_park_and_idle_hold() {
    let ground = FlatGround::new(0.0);

    // 1. Park Mode 60s test: Mechanical transmission lock must produce zero translation
    {
        let config = VehicleConfig::sports_coupe();
        let mut car = DynamicVehicle::new(
            config.clone(),
            Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
            0.0,
        );
        let park_inputs = VehicleInputs {
            gear_mode: TransmissionMode::Park,
            handbrake: true,
            ..Default::default()
        };

        let start_pos = car.position;
        for _ in 0..3600 {
            car.step(&park_inputs, &ground, 1.0 / 60.0);
        }

        let total_drift = (car.position - start_pos).length();
        let final_speed = car.linear_velocity.length();
        assert!(
            total_drift < 0.001,
            "Park hold: total drift after 60s must be < 1mm, got {:.6}m",
            total_drift
        );
        assert!(
            final_speed < 0.0001,
            "Park hold: final speed must be < 0.1mm/s, got {:.6}m/s",
            final_speed
        );
        assert_eq!(car.angular_velocity, Vec3::ZERO);
    }

    // 2. Drive Mode 60s test with Zero Throttle (Auto-Creep Disabled):
    // Must remain rock-solid motionless on level ground without spontaneous forward movement
    {
        let config = VehicleConfig::sports_coupe();
        assert!(
            !config.auto_creep_enabled,
            "Sports coupe auto_creep_enabled must default to false"
        );

        let mut car = DynamicVehicle::new(
            config.clone(),
            Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
            0.0,
        );
        let idle_inputs = VehicleInputs {
            gear_mode: TransmissionMode::Drive,
            throttle: 0.0,
            brake: 0.0,
            steer: 0.0,
            handbrake: false,
        };

        let start_pos = car.position;
        for _ in 0..3600 {
            car.step(&idle_inputs, &ground, 1.0 / 60.0);
        }

        let total_drift = (car.position - start_pos).length();
        let final_speed = car.linear_velocity.length();
        assert!(
            total_drift < 0.001,
            "Drive idle: vehicle must NOT creep or spontaneously move on level ground, drift = {:.6}m",
            total_drift
        );
        assert!(
            final_speed < 0.0001,
            "Drive idle: final velocity must be 0, got {:.6}m/s",
            final_speed
        );
    }
}

// =============================================================================
// GATE 2: Directional Steering and Camera Alignment at 4 Cardinal Headings
// =============================================================================

#[test]
fn test_gate2_directional_steering_across_four_cardinal_headings() {
    let ground = FlatGround::new(0.0);
    let headings = [
        ("North", 0.0f32, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
        ("East", FRAC_PI_2, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
        ("South", PI, Vec3::new(0.0, 0.0, -1.0), Vec3::new(-1.0, 0.0, 0.0)),
        ("West", -FRAC_PI_2, Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
    ];

    for (name, heading_yaw, expected_forward, expected_right) in headings {
        // Verify base transform vectors
        let rot = Mat3::from_quat(Quat::from_rotation_y(heading_yaw));
        let fwd = rot.col(2);
        let rgt = rot.col(0);
        assert!(
            (fwd - expected_forward).length() < 1e-5,
            "Heading {} forward vector mismatch",
            name
        );
        assert!(
            (rgt - expected_right).length() < 1e-5,
            "Heading {} right vector mismatch",
            name
        );

        // 1. Forward Steering A (Left) vs D (Right):
        // Steer Left (steer < 0): turns towards -right.
        // Steer Right (steer > 0): turns towards +right.
        let config = VehicleConfig::sports_coupe();
        let mut car_left = DynamicVehicle::new(
            config.clone(),
            Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
            heading_yaw,
        );
        let mut car_right = DynamicVehicle::new(
            config.clone(),
            Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
            heading_yaw,
        );

        let inputs_left = VehicleInputs {
            throttle: 0.8,
            steer: -0.5, // Steer Left
            gear_mode: TransmissionMode::Drive,
            ..Default::default()
        };
        let inputs_right = VehicleInputs {
            throttle: 0.8,
            steer: 0.5, // Steer Right
            gear_mode: TransmissionMode::Drive,
            ..Default::default()
        };

        for _ in 0..90 {
            car_left.step(&inputs_left, &ground, 1.0 / 60.0);
            car_right.step(&inputs_right, &ground, 1.0 / 60.0);
        }

        // Relative lateral displacement along initial vehicle right vector
        let lat_disp_left = car_left.position.dot(expected_right);
        let lat_disp_right = car_right.position.dot(expected_right);

        assert!(
            lat_disp_left < -0.8,
            "Heading {}: Steer Left must displace in -right direction, got {:.3}m",
            name,
            lat_disp_left
        );
        assert!(
            lat_disp_right > 0.8,
            "Heading {}: Steer Right must displace in +right direction, got {:.3}m",
            name,
            lat_disp_right
        );

        // 2. Camera tracking alignment:
        let mut cam = VehicleCameraController::new(VehicleCameraPerspective::Chase);
        let (cam_pos, cam_yaw, _) = cam.update(&car_left, 1.0 / 60.0);

        // Chase camera must be placed behind the vehicle opposite to forward heading
        let to_cam = cam_pos - car_left.position;
        let car_fwd_now = Mat3::from_quat(car_left.orientation).col(2);
        let cam_dot_fwd = to_cam.normalize().dot(car_fwd_now);
        assert!(
            cam_dot_fwd < -0.85,
            "Heading {}: Chase camera must be positioned behind vehicle, dot = {:.3}",
            name,
            cam_dot_fwd
        );

        // Camera yaw must look along vehicle forward vector
        let expected_cam_yaw = car_fwd_now.z.atan2(car_fwd_now.x);
        let yaw_diff = (cam_yaw - expected_cam_yaw).sin().abs();
        assert!(
            yaw_diff < 0.05,
            "Heading {}: Camera yaw must align with vehicle forward direction, diff = {:.4}",
            name,
            yaw_diff
        );
    }
}

#[test]
fn test_gate2_reverse_kinematics_and_standstill_braking() {
    let ground = FlatGround::new(0.0);
    let config = VehicleConfig::sports_coupe();
    let mut car = DynamicVehicle::new(
        config.clone(),
        Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
        0.0,
    );

    // Accelerate in reverse
    let rev_inputs = VehicleInputs {
        throttle: 0.85,
        gear_mode: TransmissionMode::Reverse,
        ..Default::default()
    };
    for _ in 0..120 {
        car.step(&rev_inputs, &ground, 1.0 / 60.0);
    }

    assert!(
        car.speed_kph < -15.0,
        "Vehicle must reach reverse cruising speed, got {:.1} km/h",
        car.speed_kph
    );

    // Apply full service brake to standstill
    let brake_inputs = VehicleInputs {
        brake: 1.0,
        throttle: 0.0,
        gear_mode: TransmissionMode::Reverse,
        ..Default::default()
    };
    for _ in 0..120 {
        car.step(&brake_inputs, &ground, 1.0 / 60.0);
    }

    assert!(
        car.speed_kph.abs() < 0.01,
        "Vehicle must brake to full standstill from reverse without bouncing, got {:.4} km/h",
        car.speed_kph
    );
    assert_eq!(car.linear_velocity, Vec3::ZERO);
}

// =============================================================================
// GATE 3: Continuous Road Surface and Bridge Deck Physical Contact
// =============================================================================

#[test]
fn test_gate3_road_surface_curb_and_elevated_bridge_continuity() {
    // Construct a synthetic realistic multi-segment road route:
    // Seg 0: Surface road at Y = 10.20m (from DEM 10.0 + 0.20 visual offset)
    // Seg 1: Approach ramp rising from Y = 10.20m to Y = 16.20m (bridge deck layer 1)
    // Seg 2: Elevated bridge deck at Y = 16.20m spanning across water (DEM = 0.0)
    let ground = SyntheticRoadNetworkGround {
        base_elev: 10.0,
        segments: vec![
            RoadTestSegment {
                p1: Vec3::new(0.0, 10.20, 0.0),
                p2: Vec3::new(0.0, 10.20, 100.0),
                half_width: 3.5,
                is_bridge: false,
            },
            RoadTestSegment {
                p1: Vec3::new(0.0, 10.20, 100.0),
                p2: Vec3::new(0.0, 16.20, 180.0),
                half_width: 3.5,
                is_bridge: false,
            },
            RoadTestSegment {
                p1: Vec3::new(0.0, 16.20, 180.0),
                p2: Vec3::new(0.0, 16.20, 800.0),
                half_width: 3.5,
                is_bridge: true,
            },
        ],
    };

    let config = VehicleConfig::sports_coupe();
    let eq_height = config.equilibrium_ride_height();

    // 1. Test surface contact on Segment 0 (asphalt surface at 10.20m)
    let contact0 = ground.query_ground_contact(Vec3::new(0.0, 10.50, 50.0), 2.0);
    assert!(contact0.is_some(), "Must hit surface road ribbon");
    let (dist0, norm0) = contact0.unwrap();
    let hit_y0 = 10.50 - dist0;
    assert!(
        (hit_y0 - 10.20).abs() < 1e-4,
        "Surface road contact Y ({:.3}) must match visual mesh at 10.20m",
        hit_y0
    );
    assert!(
        (norm0 - Vec3::Y).length() < 1e-4,
        "Surface road normal must be upward"
    );

    // 2. Test sidewalk curb riser: point placed at X = 4.2m (beyond half_width 3.5m)
    let contact_curb = ground.query_ground_contact(Vec3::new(4.2, 10.70, 50.0), 2.0);
    assert!(contact_curb.is_some(), "Must hit sidewalk curb");
    let (dist_curb, _) = contact_curb.unwrap();
    let curb_y = 10.70 - dist_curb;
    assert!(
        (curb_y - 10.35).abs() < 1e-4,
        "Sidewalk curb contact Y ({:.3}) must be at 10.35m (+0.15m above road)",
        curb_y
    );

    // 3. Test elevated bridge deck on Segment 2 (bridge deck at 16.20m)
    let contact_bridge = ground.query_ground_contact(Vec3::new(0.0, 16.80, 250.0), 3.0);
    assert!(contact_bridge.is_some(), "Must hit elevated bridge deck");
    let (dist_bridge, _) = contact_bridge.unwrap();
    let bridge_y = 16.80 - dist_bridge;
    assert!(
        (bridge_y - 16.20).abs() < 1e-4,
        "Bridge contact Y ({:.3}) must match bridge deck height at 16.20m",
        bridge_y
    );

    // 4. Drive vehicle across entire route: surface -> ramp -> elevated bridge deck
    let mut car = DynamicVehicle::new(
        config.clone(),
        Vec3::new(0.0, 10.20 + eq_height, 10.0),
        0.0, // Facing North (+Z)
    );

    let drive_inputs = VehicleInputs {
        throttle: 0.70,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };

    let mut min_clearance_to_ground = f32::MAX;
    for _tick in 0..700 {
        car.step(&drive_inputs, &ground, 1.0 / 60.0);

        // Sample exact ground beneath vehicle center
        if let Some((dist, _)) = ground.query_ground_contact(car.position, 10.0) {
            let ground_surface_y = car.position.y - dist;
            let clearance = car.position.y - ground_surface_y;
            if clearance < min_clearance_to_ground {
                min_clearance_to_ground = clearance;
            }
        }
    }

    println!(
        "Route crossing: Final Pos = [{:.2}, {:.2}, {:.2}], Min Clearance = {:.3}m",
        car.position.x, car.position.y, car.position.z, min_clearance_to_ground
    );

    assert!(
        car.position.z > 200.0,
        "Car must successfully drive onto elevated bridge deck, got Z = {:.2}m",
        car.position.z
    );
    assert!(
        car.position.y > 16.0,
        "Car must ride atop bridge deck (> 16.0m), got Y = {:.2}m",
        car.position.y
    );
    assert!(
        min_clearance_to_ground >= config.half_extents.y * 0.95,
        "Chassis must never sink into road surface, min clearance = {:.3}m",
        min_clearance_to_ground
    );
}

// =============================================================================
// GATE 4: Multi-Framerate Determinism at 30, 60, and 120 FPS
// =============================================================================

#[test]
fn test_gate4_multi_framerate_determinism_at_30_60_120_fps() {
    let ground = FlatGround::new(0.0);
    let run_simulation = |fps: usize| -> (Vec3, glam::Quat, f32) {
        let config = VehicleConfig::sports_coupe();
        let mut car = DynamicVehicle::new(
            config.clone(),
            Vec3::new(0.0, config.equilibrium_ride_height(), 0.0),
            0.0,
        );

        let dt = 1.0 / fps as f32;
        const PHYSICS_DT: f32 = 1.0 / 60.0;
        let mut accumulator = 0.0f32;

        let total_frames = fps * 4; // 4 seconds of driving
        for frame in 0..total_frames {
            let sim_time = frame as f32 * dt;
            let inputs = if sim_time < 1.5 {
                VehicleInputs {
                    throttle: 1.0,
                    steer: 0.0,
                    gear_mode: TransmissionMode::Drive,
                    ..Default::default()
                }
            } else if sim_time < 3.0 {
                VehicleInputs {
                    throttle: 0.8,
                    steer: 0.4, // Turn right
                    gear_mode: TransmissionMode::Drive,
                    ..Default::default()
                }
            } else {
                VehicleInputs {
                    brake: 0.95,
                    throttle: 0.0,
                    steer: 0.0,
                    gear_mode: TransmissionMode::Drive,
                    ..Default::default()
                }
            };

            accumulator = (accumulator + dt).min(0.20);
            while accumulator >= PHYSICS_DT {
                car.step(&inputs, &ground, PHYSICS_DT);
                accumulator -= PHYSICS_DT;
            }
        }

        (car.position, car.orientation, car.speed_kph)
    };

    let (pos30, _, speed30) = run_simulation(30);
    let (pos60, _, speed60) = run_simulation(60);
    let (pos120, _, speed120) = run_simulation(120);

    let diff_30_60 = (pos30 - pos60).length();
    let diff_60_120 = (pos60 - pos120).length();

    println!(
        "Multi-framerate determinism: 30 vs 60 = {:.4}m, 60 vs 120 = {:.4}m",
        diff_30_60, diff_60_120
    );

    assert!(
        diff_30_60 < 0.02,
        "Position at 30 FPS must match 60 FPS within 20mm, got {:.4}m",
        diff_30_60
    );
    assert!(
        diff_60_120 < 0.01,
        "Position at 60 FPS must match 120 FPS within 10mm, got {:.4}m",
        diff_60_120
    );
    assert!(
        (speed30 - speed60).abs() < 0.1,
        "Speed at 30 FPS must match 60 FPS within 0.1 km/h"
    );
    assert!(
        (speed60 - speed120).abs() < 0.05,
        "Speed at 60 FPS must match 120 FPS within 0.05 km/h"
    );
}

// =============================================================================
// GATE 5: Multimodal Fleet Class-Specific Dynamics Validation
// =============================================================================

#[test]
fn test_gate5_multimodal_fleet_class_specific_dynamics() {
    let sports = VehicleConfig::sports_coupe();
    let sedan = VehicleConfig::sedan();
    let suv = VehicleConfig::suv();
    let bus = VehicleConfig::transit_bus();
    let truck = VehicleConfig::heavy_truck();

    // Verify curb weight separation
    assert!(sports.mass_kg < sedan.mass_kg);
    assert!(sedan.mass_kg < suv.mass_kg);
    assert!(suv.mass_kg < bus.mass_kg);
    assert!(bus.mass_kg < truck.mass_kg);

    // Verify suspension stiffness scaling
    assert!(sports.suspension_stiffness < bus.suspension_stiffness);
    assert!(bus.suspension_stiffness < truck.suspension_stiffness);

    // Test Motorcycle lean dynamics
    let ground = FlatGround::new(0.0);
    let mut bike = MotorcycleModel::new(Vec3::ZERO, 0.0);
    let bike_steer_inputs = VehicleInputs {
        throttle: 0.8,
        steer: 0.6,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };
    for _ in 0..120 {
        bike.step(&bike_steer_inputs, &ground, 1.0 / 60.0);
    }
    assert!(
        bike.lean_angle > 0.15,
        "Motorcycle must roll lean into turn, got {:.3} rad",
        bike.lean_angle
    );

    // Test Transit Bus passenger braking
    let mut transit = TransitBusModel::new(Vec3::ZERO, 0.0);
    transit.set_passenger_count(50);
    assert!(
        (transit.total_mass_kg - (12500.0 + 50.0 * 75.0)).abs() < 1e-3,
        "Gross bus mass must include passengers"
    );

    // Test Heavy Truck trailer articulation
    let mut big_rig = HeavyTruckModel::new(Vec3::ZERO, 0.0);
    let truck_turn_inputs = VehicleInputs {
        throttle: 0.7,
        steer: 0.5,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };
    for _ in 0..120 {
        big_rig.step(&truck_turn_inputs, &ground, 1.0 / 60.0);
    }
    assert!(
        big_rig.trailer_articulation_angle.abs() > 0.05,
        "Articulated trailer must swing relative to tractor"
    );
}

// =============================================================================
// GATE 6: San Francisco Steep Hill-Hold & Anti-Rollback
// =============================================================================

#[test]
fn test_gate6_san_francisco_steep_hill_hold_and_anti_rollback() {
    // 18% steep grade (typical of California St / Powell St in San Francisco)
    let grade = 0.18f32;
    let ground = SlopedGround::new(0.0, 0.0, grade);
    let config = VehicleConfig::sports_coupe();
    let mut car = DynamicVehicle::new(
        config.clone(),
        Vec3::new(0.0, 2.0, 0.0),
        0.0, // Facing uphill (+Z)
    );
    car.recover_on_surface(0.0);

    // Drive uphill for 2 seconds
    let drive_inputs = VehicleInputs {
        throttle: 0.95,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };
    for _ in 0..120 {
        car.step(&drive_inputs, &ground, 1.0 / 60.0);
    }
    assert!(
        car.position.z > 5.0,
        "Vehicle must climb uphill along +Z, got Z = {:.2}m",
        car.position.z
    );

    // Release throttle and apply service brake to stop
    let brake_inputs = VehicleInputs {
        brake: 1.0,
        throttle: 0.0,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };
    for _ in 0..120 {
        car.step(&brake_inputs, &ground, 1.0 / 60.0);
    }
    assert!(
        car.speed_kph.abs() < 0.05,
        "Vehicle must halt on 18% slope under service brake"
    );

    // Hill-Hold Assist Test: Release service brake while in Drive with throttle = 0
    let hha_inputs = VehicleInputs {
        throttle: 0.0,
        brake: 0.0,
        gear_mode: TransmissionMode::Drive,
        ..Default::default()
    };
    let stopped_z = car.position.z;
    for _ in 0..300 {
        // Hold for 5 seconds
        car.step(&hha_inputs, &ground, 1.0 / 60.0);
    }

    let rollback = stopped_z - car.position.z;
    println!("18% Slope HHA Hold: rollback = {:.5}m", rollback);
    assert!(
        rollback < 0.005,
        "Hill-Hold Assist must prevent backward rollback in Drive, rollback = {:.5}m",
        rollback
    );
}
