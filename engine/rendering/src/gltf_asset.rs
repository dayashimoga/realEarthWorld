//! Standards-compliant glTF 2.0 / GLB binary asset import, validation, caching, and hierarchy management.

use glam::{Affine3A, Mat3, Quat, Vec3};
use realearth_compiler::mesh::{ChunkMesh, Vertex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Error conditions encountered during glTF/GLB import and validation.
#[derive(Debug, Error)]
pub enum GltfAssetError {
    #[error("I/O error reading asset: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid GLB header magic: expected 0x46546C67 ('glTF'), got 0x{got:08X}")]
    InvalidMagic { got: u32 },

    #[error("Unsupported glTF version: {0} (only glTF 2.0 is supported)")]
    UnsupportedVersion(u32),

    #[error("GLB file truncated or buffer size mismatch: header specifies {expected} bytes, got {actual}")]
    TruncatedFile { expected: usize, actual: usize },

    #[error("Corrupt GLB chunk: {0}")]
    CorruptChunk(String),

    #[error("Failed to parse glTF JSON manifest: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Buffer access out of bounds: offset {offset} + len {length} exceeds buffer size {buffer_size}")]
    BufferOutOfBounds {
        offset: usize,
        length: usize,
        buffer_size: usize,
    },

    #[error("Invalid accessor index {index} (manifest contains {count} accessors)")]
    InvalidAccessor { index: usize, count: usize },

    #[error("Missing mandatory vertex attribute: {0}")]
    MissingAttribute(String),

    #[error("Vehicle asset '{0}' was not found in asset registry")]
    MissingAsset(String),

    #[error("Asset validation failure: {0}")]
    Validation(String),
}

/// Level of detail (LOD) tier for 3D vehicle geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VehicleLodLevel {
    /// LOD0: High-fidelity cinematic detail for player vehicle and close-up garage inspection.
    #[default]
    Lod0High,
    /// LOD1: Medium-fidelity for nearby AI traffic (< 200m).
    Lod1Medium,
    /// LOD2: Streamlined low-polygon silhouette for distant AI traffic (> 200m).
    Lod2Low,
}

/// Functional semantic classification of vehicle submeshes for dynamic rigging and articulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VehicleSubmeshType {
    /// Main aerodynamic chassis, body panels, fenders, hood, doors, roof.
    ChassisBody,
    /// Front-Left wheel assembly (steered and rolled).
    WheelFrontLeft,
    /// Front-Right wheel assembly (steered and rolled).
    WheelFrontRight,
    /// Rear-Left wheel assembly (rolled).
    WheelRearLeft,
    /// Rear-Right wheel assembly (rolled).
    WheelRearRight,
    /// Transparent/tinted glass greenhouse, windshield, side and rear windows.
    GlassGreenhouse,
    /// Detailed cockpit interior, dashboard, seats, console.
    InteriorCockpit,
    /// Steering wheel (rotates with steering deflection).
    InteriorSteeringWheel,
    /// High-intensity projector headlights and DRLs.
    Headlights,
    /// Rear taillights and high-intensity brake lights.
    BrakeLights,
    /// White reverse light modules.
    ReverseLights,
    /// Amber dynamic turn indicator signals.
    TurnSignals,
    /// Ventilated brake discs and high-performance calipers.
    BrakeHardware,
    /// Polished exhaust tips and diffuser pipes.
    Exhaust,
    /// Aerodynamic GT wing or lip spoiler.
    AerodynamicSpoiler,
    /// Soft ambient occlusion contact shadow plane beneath tires.
    ContactShadow,
    /// Front railway truck / bogie assembly articulating with track curves.
    TrainBogieFront,
    /// Rear railway truck / bogie assembly articulating with track curves.
    TrainBogieRear,
    /// Flanged steel railway wheelsets rotating with track linear speed.
    TrainWheelset,
    /// Railway AAR Type E knuckle couplers and end draft gear.
    TrainCoupler,
    /// Transit bus front boarding doors animating open/closed.
    BusDoorFront,
    /// Transit bus center exit doors animating open/closed.
    BusDoorCenter,
    /// Interior passenger seating rows in transit buses and rail coaches.
    InteriorPassengerSeats,
    /// Articulated highway freight trailer.
    TruckArticulatedTrailer,
    /// Aircraft propulsion propeller.
    AircraftPropeller,
    /// Aircraft high-bypass turbofan intake fan blades.
    AircraftTurbofanFan,
    /// Aircraft aerodynamic control surfaces (elevators, ailerons, rudder).
    AircraftControlSurface,
    /// Marine vessel high-velocity waterjet propulsion nozzles.
    MarineWaterjets,
    /// Marine radar scanner mast rotating in azimuth.
    MarineRadarMast,
    /// Motorcycle frame, engine, and rider banking dynamically into turns.
    MotorcycleLeanBody,
}

impl VehicleSubmeshType {
    pub fn from_node_name(name: &str) -> Self {
        let n = name.to_lowercase();
        if n.contains("bogie_f") || n.contains("front_bogie") || n.contains("truck_front") {
            Self::TrainBogieFront
        } else if n.contains("bogie_r") || n.contains("rear_bogie") || n.contains("truck_rear") {
            Self::TrainBogieRear
        } else if n.contains("wheelset") || n.contains("train_wheel") || n.contains("rail_wheel") {
            Self::TrainWheelset
        } else if n.contains("coupler") || n.contains("knuckle") {
            Self::TrainCoupler
        } else if n.contains("door_front") || n.contains("bus_door_f") {
            Self::BusDoorFront
        } else if n.contains("door_center") || n.contains("bus_door_c") || n.contains("exit_door") {
            Self::BusDoorCenter
        } else if n.contains("passenger_seat") || n.contains("saloon_seat") {
            Self::InteriorPassengerSeats
        } else if n.contains("propeller") || n.contains("airscrew") {
            Self::AircraftPropeller
        } else if n.contains("turbofan") || n.contains("jet_fan") || n.contains("engine_fan") {
            Self::AircraftTurbofanFan
        } else if n.contains("aileron")
            || n.contains("elevator")
            || n.contains("rudder")
            || n.contains("control_surface")
        {
            Self::AircraftControlSurface
        } else if n.contains("waterjet") || n.contains("propulsion") {
            Self::MarineWaterjets
        } else if n.contains("radar") || n.contains("mast") {
            Self::MarineRadarMast
        } else if n.contains("trailer") || n.contains("semitrailer") {
            Self::TruckArticulatedTrailer
        } else if n.contains("bike_body") || n.contains("motorcycle_frame") {
            Self::MotorcycleLeanBody
        } else if n.contains("wheel_fl") || n.contains("front_left_wheel") || n.contains("wheel.fl")
        {
            Self::WheelFrontLeft
        } else if n.contains("wheel_fr")
            || n.contains("front_right_wheel")
            || n.contains("wheel.fr")
        {
            Self::WheelFrontRight
        } else if n.contains("wheel_rl") || n.contains("rear_left_wheel") || n.contains("wheel.rl")
        {
            Self::WheelRearLeft
        } else if n.contains("wheel_rr") || n.contains("rear_right_wheel") || n.contains("wheel.rr")
        {
            Self::WheelRearRight
        } else if n.contains("glass") || n.contains("windshield") || n.contains("window") {
            Self::GlassGreenhouse
        } else if n.contains("steering_wheel") || n.contains("steer_wheel") {
            Self::InteriorSteeringWheel
        } else if n.contains("interior")
            || n.contains("cockpit")
            || n.contains("seat")
            || n.contains("dash")
        {
            Self::InteriorCockpit
        } else if n.contains("headlight") || n.contains("front_light") {
            Self::Headlights
        } else if n.contains("brakelight") || n.contains("tail_light") || n.contains("rear_light") {
            Self::BrakeLights
        } else if n.contains("reverselight") || n.contains("reverse_light") {
            Self::ReverseLights
        } else if n.contains("indicator") || n.contains("turn_signal") {
            Self::TurnSignals
        } else if n.contains("brake_caliper") || n.contains("rotor") || n.contains("disc") {
            Self::BrakeHardware
        } else if n.contains("exhaust") || n.contains("muffler") || n.contains("tailpipe") {
            Self::Exhaust
        } else if n.contains("spoiler") || n.contains("wing") {
            Self::AerodynamicSpoiler
        } else if n.contains("shadow") || n.contains("contact_ao") {
            Self::ContactShadow
        } else {
            Self::ChassisBody
        }
    }
}

/// Physically-Based Rendering (PBR) metallic-roughness material definition.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VehiclePbrMaterial {
    /// Base RGBA albedo color factor (linear sRGB + alpha tag).
    pub base_color: [f32; 4],
    /// Metallic reflectance factor [0.0 = dielectric, 1.0 = metal].
    pub metallic: f32,
    /// Surface microfacet roughness factor [0.0 = mirror, 1.0 = completely matte].
    pub roughness: f32,
    /// Linear RGB emissive emission intensity factor.
    pub emissive: [f32; 3],
    /// Alpha transparency rendering mode.
    pub alpha_mode: VehicleAlphaMode,
}

impl Default for VehiclePbrMaterial {
    fn default() -> Self {
        Self {
            base_color: [0.85, 0.15, 0.15, 1.0], // Cherry Red
            metallic: 0.85,                      // High metallic car paint
            roughness: 0.20,                     // Glossy clearcoat
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: VehicleAlphaMode::Opaque,
        }
    }
}

/// Alpha blending mode for vehicle materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VehicleAlphaMode {
    Opaque,
    Mask,
    Blend,
}

/// An articulated submesh within a vehicle visual hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleSubmesh {
    pub name: String,
    pub submesh_type: VehicleSubmeshType,
    pub mesh: ChunkMesh,
    pub local_transform: Affine3A,
    pub pivot: Vec3,
    pub material: VehiclePbrMaterial,
}

/// Verified camera attachment and framing anchors for cockpit, chase, hood, and orbit perspectives.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VehicleCameraAnchors {
    /// Follow focus target relative to vehicle center.
    pub chase_target: Vec3,
    /// Distance behind vehicle for third-person chase camera.
    pub chase_distance: f32,
    /// Height above vehicle for third-person chase camera.
    pub chase_height: f32,
    /// Driver eye viewpoint inside cockpit (left-hand drive position).
    pub cockpit_eye: Vec3,
    /// Driver forward look target from cockpit.
    pub cockpit_look: Vec3,
    /// Forward hood / bumper camera viewpoint.
    pub hood_eye: Vec3,
    /// Centered orbit pivot for showroom garage turntable.
    pub orbit_pivot: Vec3,
}

impl Default for VehicleCameraAnchors {
    fn default() -> Self {
        Self {
            chase_target: Vec3::new(0.0, 0.5, 0.0),
            chase_distance: 6.2,
            chase_height: 1.8,
            cockpit_eye: Vec3::new(-0.38, 0.95, -0.15),
            cockpit_look: Vec3::new(-0.38, 0.95, 2.5),
            hood_eye: Vec3::new(0.0, 0.85, 1.6),
            orbit_pivot: Vec3::new(0.0, 0.5, 0.0),
        }
    }
}

/// Fully-loaded and validated vehicle 3D model asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedVehicleModel {
    pub asset_id: String,
    pub display_name: String,
    pub submeshes: Vec<VehicleSubmesh>,
    pub dimensions: Vec3, // length (Z), width (X), height (Y)
    pub wheelbase: f32,
    pub track_width: f32,
    pub wheel_radius: f32,
    pub mass_kg: f32,
    pub lod_level: VehicleLodLevel,
    pub camera_anchors: VehicleCameraAnchors,
}

impl LoadedVehicleModel {
    /// Composes a single articulated `ChunkMesh` transformed into world space with active animations.
    #[allow(clippy::too_many_arguments)]
    pub fn compose_world_mesh(
        &self,
        world_pos: Vec3,
        world_orient: Quat,
        override_paint_color: Option<[f32; 4]>,
        wheel_steer_angle: f32,
        wheel_spin_angle: f32,
        wheel_compressions: [f32; 4],
        headlights_on: bool,
        brake_lights_on: bool,
        reverse_lights_on: bool,
    ) -> ChunkMesh {
        self.compose_multimodal_world_mesh(
            world_pos,
            world_orient,
            override_paint_color,
            wheel_steer_angle,
            wheel_spin_angle,
            wheel_compressions,
            headlights_on,
            brake_lights_on,
            reverse_lights_on,
            0.0,
            [0.0, 0.0],
            0.0,
            [0.0, 0.0, 0.0],
            0.0,
            0.0,
        )
    }

    /// Composes a rich multimodal articulated `ChunkMesh` with physics-driven animations:
    /// wheels, bogies, doors, propellers, control surfaces, trailers, and motorcycle banking.
    #[allow(clippy::too_many_arguments)]
    pub fn compose_multimodal_world_mesh(
        &self,
        world_pos: Vec3,
        world_orient: Quat,
        override_paint_color: Option<[f32; 4]>,
        wheel_steer_angle: f32,
        wheel_spin_angle: f32,
        wheel_compressions: [f32; 4],
        headlights_on: bool,
        brake_lights_on: bool,
        reverse_lights_on: bool,
        door_open_ratio: f32,
        bogie_angles: [f32; 2],
        propeller_spin_angle: f32,
        control_surfaces: [f32; 3], // [elevator, aileron, rudder]
        motorcycle_lean_angle: f32,
        trailer_angle: f32,
    ) -> ChunkMesh {
        let mut combined_mesh = ChunkMesh::default();
        let world_rot_mat = Mat3::from_quat(world_orient);

        // Alpha / depth sorting: Emit opaque submeshes first, mask second, and blend/transparent
        // submeshes (e.g. greenhouse glass, contact shadows) last so the depth buffer properly resolves.
        let mut sorted_submeshes: Vec<&VehicleSubmesh> = self.submeshes.iter().collect();
        sorted_submeshes.sort_by_key(|s| match s.material.alpha_mode {
            VehicleAlphaMode::Opaque => 0,
            VehicleAlphaMode::Mask => 1,
            VehicleAlphaMode::Blend => 2,
        });

        for submesh in sorted_submeshes {
            if submesh.mesh.is_empty() {
                continue;
            }

            // Compute local dynamic articulation transform based on submesh semantic type
            let (local_offset, local_rotation, override_color) = match submesh.submesh_type {
                VehicleSubmeshType::WheelFrontLeft => {
                    let comp = wheel_compressions[0];
                    let offset = Vec3::new(0.0, comp, 0.0);
                    let steer_rot = Quat::from_rotation_y(wheel_steer_angle);
                    let roll_rot = Quat::from_rotation_x(wheel_spin_angle);
                    (offset, steer_rot * roll_rot, None)
                }
                VehicleSubmeshType::WheelFrontRight => {
                    let comp = wheel_compressions[1];
                    let offset = Vec3::new(0.0, comp, 0.0);
                    let steer_rot = Quat::from_rotation_y(wheel_steer_angle);
                    let roll_rot = Quat::from_rotation_x(wheel_spin_angle);
                    (offset, steer_rot * roll_rot, None)
                }
                VehicleSubmeshType::WheelRearLeft => {
                    let comp = wheel_compressions[2];
                    let offset = Vec3::new(0.0, comp, 0.0);
                    let roll_rot = Quat::from_rotation_x(wheel_spin_angle);
                    (offset, roll_rot, None)
                }
                VehicleSubmeshType::WheelRearRight => {
                    let comp = wheel_compressions[3];
                    let offset = Vec3::new(0.0, comp, 0.0);
                    let roll_rot = Quat::from_rotation_x(wheel_spin_angle);
                    (offset, roll_rot, None)
                }
                VehicleSubmeshType::TrainBogieFront => {
                    let yaw_rot = Quat::from_rotation_y(bogie_angles[0]);
                    (Vec3::ZERO, yaw_rot, None)
                }
                VehicleSubmeshType::TrainBogieRear => {
                    let yaw_rot = Quat::from_rotation_y(bogie_angles[1]);
                    (Vec3::ZERO, yaw_rot, None)
                }
                VehicleSubmeshType::TrainWheelset => {
                    let roll_rot = Quat::from_rotation_x(wheel_spin_angle);
                    (Vec3::ZERO, roll_rot, None)
                }
                VehicleSubmeshType::BusDoorFront => {
                    let open_offset =
                        Vec3::new(-0.25 * door_open_ratio, 0.0, 0.12 * door_open_ratio);
                    let open_rot = Quat::from_rotation_y(door_open_ratio * 0.35);
                    (open_offset, open_rot, None)
                }
                VehicleSubmeshType::BusDoorCenter => {
                    let open_offset =
                        Vec3::new(-0.25 * door_open_ratio, 0.0, -0.12 * door_open_ratio);
                    let open_rot = Quat::from_rotation_y(-door_open_ratio * 0.35);
                    (open_offset, open_rot, None)
                }
                VehicleSubmeshType::TruckArticulatedTrailer => {
                    let trailer_rot = Quat::from_rotation_y(trailer_angle);
                    (Vec3::ZERO, trailer_rot, None)
                }
                VehicleSubmeshType::AircraftPropeller => {
                    let spin_rot = Quat::from_rotation_z(propeller_spin_angle);
                    (Vec3::ZERO, spin_rot, None)
                }
                VehicleSubmeshType::AircraftTurbofanFan => {
                    let spin_rot = Quat::from_rotation_z(propeller_spin_angle * 1.5);
                    (Vec3::ZERO, spin_rot, None)
                }
                VehicleSubmeshType::AircraftControlSurface => {
                    let pitch_rot = Quat::from_rotation_x(control_surfaces[0]);
                    (Vec3::ZERO, pitch_rot, None)
                }
                VehicleSubmeshType::MarineRadarMast => {
                    let scan_rot = Quat::from_rotation_y(propeller_spin_angle * 0.20);
                    (Vec3::ZERO, scan_rot, None)
                }
                VehicleSubmeshType::MotorcycleLeanBody => {
                    let lean_rot = Quat::from_rotation_z(motorcycle_lean_angle);
                    (Vec3::ZERO, lean_rot, None)
                }
                VehicleSubmeshType::InteriorSteeringWheel => {
                    let steer_rot = Quat::from_rotation_z(-wheel_steer_angle * 1.5);
                    (Vec3::ZERO, steer_rot, None)
                }
                VehicleSubmeshType::Headlights => {
                    let color = if headlights_on {
                        [1.0, 0.98, 0.88, 0.95] // Brilliant xenon white (alpha 0.95 = emissive light tag)
                    } else {
                        [0.35, 0.35, 0.38, 0.98] // Inactive lens
                    };
                    (Vec3::ZERO, Quat::IDENTITY, Some(color))
                }
                VehicleSubmeshType::BrakeLights => {
                    let color = if brake_lights_on {
                        [1.0, 0.05, 0.05, 0.95] // Brilliant brake red (alpha 0.95 = emissive light tag)
                    } else {
                        [0.75, 0.05, 0.05, 0.95] // Running taillight
                    };
                    (Vec3::ZERO, Quat::IDENTITY, Some(color))
                }
                VehicleSubmeshType::ReverseLights => {
                    let color = if reverse_lights_on {
                        [0.98, 0.98, 1.0, 0.95] // Reverse white (alpha 0.95 = emissive light tag)
                    } else {
                        [0.30, 0.30, 0.32, 0.98] // Inactive reverse
                    };
                    (Vec3::ZERO, Quat::IDENTITY, Some(color))
                }
                VehicleSubmeshType::ChassisBody => {
                    let paint = override_paint_color.map(|mut c| {
                        c[3] = 0.99; // Automotive Clearcoat Paint tag
                        c
                    });
                    (Vec3::ZERO, Quat::IDENTITY, paint)
                }
                VehicleSubmeshType::TurnSignals => (Vec3::ZERO, Quat::IDENTITY, None),
                _ => (Vec3::ZERO, Quat::IDENTITY, None),
            };

            let index_offset = combined_mesh.vertices.len() as u32;

            for v in &submesh.mesh.vertices {
                let local_pos = Vec3::from_array(v.position);
                let local_normal = Vec3::from_array(v.normal);

                // Articulate around submesh pivot
                let p_rel = local_pos;
                let articulated_p = submesh.pivot + local_offset + local_rotation * p_rel;
                let articulated_norm = local_rotation * local_normal;

                // Transform to vehicle world space
                let world_v_pos = world_pos + world_rot_mat * articulated_p;
                let world_v_norm = (world_rot_mat * articulated_norm).normalize_or_zero();

                let final_color = if let Some(ov) = override_color {
                    if submesh.submesh_type == VehicleSubmeshType::ChassisBody {
                        let is_accent_or_light = (v.color[3] - 0.95).abs() < 0.01
                            || (v.color[3] - 0.98).abs() < 0.005
                            || (v.color[0] > 0.90 && v.color[1] > 0.90 && v.color[2] > 0.90);
                        if is_accent_or_light {
                            v.color
                        } else {
                            ov
                        }
                    } else {
                        ov
                    }
                } else {
                    v.color
                };

                combined_mesh.vertices.push(Vertex::new(
                    world_v_pos.to_array(),
                    world_v_norm.to_array(),
                    v.uv,
                    final_color,
                ));
            }

            for idx in &submesh.mesh.indices {
                combined_mesh.indices.push(idx + index_offset);
            }
        }

        combined_mesh
    }

    /// Total vertex count across all submeshes.
    pub fn total_vertices(&self) -> usize {
        self.submeshes.iter().map(|s| s.mesh.vertices.len()).sum()
    }

    /// Total triangle count across all submeshes.
    pub fn total_triangles(&self) -> usize {
        self.submeshes
            .iter()
            .map(|s| s.mesh.indices.len() / 3)
            .sum()
    }
}

// =============================================================================
// glTF 2.0 / GLB Binary Parser & Serializer Implementation
// =============================================================================

const GLB_MAGIC: u32 = 0x4654_6C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_TYPE_JSON: u32 = 0x4E4F_534A; // "JSON"
const CHUNK_TYPE_BIN: u32 = 0x004E_4942; // "BIN\0"

#[derive(Debug, Serialize, Deserialize)]
struct GltfJsonRoot {
    asset: GltfAssetHeader,
    #[serde(default)]
    scene: usize,
    #[serde(default)]
    scenes: Vec<GltfScene>,
    #[serde(default)]
    nodes: Vec<GltfNode>,
    #[serde(default)]
    meshes: Vec<GltfMesh>,
    #[serde(default)]
    accessors: Vec<GltfAccessor>,
    #[serde(default)]
    #[serde(rename = "bufferViews")]
    buffer_views: Vec<GltfBufferView>,
    #[serde(default)]
    buffers: Vec<GltfBuffer>,
    #[serde(default)]
    materials: Vec<GltfMaterial>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfAssetHeader {
    version: String,
    #[serde(default)]
    generator: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfScene {
    #[serde(default)]
    nodes: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfNode {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    mesh: Option<usize>,
    #[serde(default)]
    translation: Option<[f32; 3]>,
    #[serde(default)]
    rotation: Option<[f32; 4]>,
    #[serde(default)]
    scale: Option<[f32; 3]>,
    #[serde(default)]
    children: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfMesh {
    #[serde(default)]
    name: Option<String>,
    primitives: Vec<GltfPrimitive>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfPrimitive {
    attributes: HashMap<String, usize>,
    #[serde(default)]
    indices: Option<usize>,
    #[serde(default)]
    material: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfAccessor {
    #[serde(rename = "bufferView")]
    buffer_view: usize,
    #[serde(default)]
    #[serde(rename = "byteOffset")]
    byte_offset: usize,
    #[serde(rename = "componentType")]
    component_type: u32,
    count: usize,
    #[serde(rename = "type")]
    type_str: String,
    #[serde(default)]
    min: Option<Vec<f32>>,
    #[serde(default)]
    max: Option<Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfBufferView {
    buffer: usize,
    #[serde(default)]
    #[serde(rename = "byteOffset")]
    byte_offset: usize,
    #[serde(rename = "byteLength")]
    byte_length: usize,
    #[serde(default)]
    target: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfBuffer {
    #[serde(rename = "byteLength")]
    byte_length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfMaterial {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    #[serde(rename = "pbrMetallicRoughness")]
    pbr_metallic_roughness: Option<GltfPbr>,
    #[serde(default)]
    #[serde(rename = "emissiveFactor")]
    emissive_factor: Option<[f32; 3]>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltfPbr {
    #[serde(default = "default_base_color")]
    #[serde(rename = "baseColorFactor")]
    base_color_factor: [f32; 4],
    #[serde(default = "default_factor_1")]
    #[serde(rename = "metallicFactor")]
    metallic_factor: f32,
    #[serde(default = "default_factor_1")]
    #[serde(rename = "roughnessFactor")]
    roughness_factor: f32,
}

fn default_base_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_factor_1() -> f32 {
    1.0
}

/// Standards-compliant parser for glTF 2.0 / GLB files.
pub struct GltfParser;

impl GltfParser {
    /// Parses a complete binary GLB byte buffer into a `LoadedVehicleModel`.
    pub fn parse_glb(
        asset_id: &str,
        bytes: &[u8],
        lod: VehicleLodLevel,
    ) -> Result<LoadedVehicleModel, GltfAssetError> {
        if bytes.len() < 12 {
            return Err(GltfAssetError::TruncatedFile {
                expected: 12,
                actual: bytes.len(),
            });
        }

        let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        if magic != GLB_MAGIC {
            return Err(GltfAssetError::InvalidMagic { got: magic });
        }

        let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        if version != GLB_VERSION {
            return Err(GltfAssetError::UnsupportedVersion(version));
        }

        let total_length = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        if bytes.len() < total_length {
            return Err(GltfAssetError::TruncatedFile {
                expected: total_length,
                actual: bytes.len(),
            });
        }

        let mut offset = 12;
        let mut json_chunk_data: Option<&[u8]> = None;
        let mut bin_chunk_data: Option<&[u8]> = None;

        while offset + 8 <= total_length {
            let chunk_len =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            let chunk_type = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            offset += 8;

            if offset + chunk_len > total_length {
                return Err(GltfAssetError::CorruptChunk(format!(
                    "Chunk extends beyond file length: offset {} + len {} > {}",
                    offset, chunk_len, total_length
                )));
            }

            let chunk_slice = &bytes[offset..offset + chunk_len];
            offset += chunk_len;

            // Pad to 4-byte boundary
            let padding = (4 - (chunk_len % 4)) % 4;
            offset += padding;

            if chunk_type == CHUNK_TYPE_JSON && json_chunk_data.is_none() {
                json_chunk_data = Some(chunk_slice);
            } else if chunk_type == CHUNK_TYPE_BIN && bin_chunk_data.is_none() {
                bin_chunk_data = Some(chunk_slice);
            }
        }

        let json_bytes = json_chunk_data
            .ok_or_else(|| GltfAssetError::CorruptChunk("Missing mandatory JSON chunk 0".into()))?;
        let bin_bytes = bin_chunk_data
            .ok_or_else(|| GltfAssetError::CorruptChunk("Missing mandatory BIN chunk 1".into()))?;

        let root: GltfJsonRoot = serde_json::from_slice(json_bytes)?;

        let mut submeshes = Vec::new();

        // Process nodes referencing meshes
        for (node_idx, node) in root.nodes.iter().enumerate() {
            if let Some(mesh_idx) = node.mesh {
                if mesh_idx >= root.meshes.len() {
                    continue;
                }
                let mesh_def = &root.meshes[mesh_idx];
                let node_name = node
                    .name
                    .clone()
                    .or_else(|| mesh_def.name.clone())
                    .unwrap_or_else(|| format!("Node_{}", node_idx));
                let submesh_type = VehicleSubmeshType::from_node_name(&node_name);

                let translation = node.translation.map(Vec3::from_array).unwrap_or(Vec3::ZERO);
                let rotation = node
                    .rotation
                    .map(|r| Quat::from_xyzw(r[0], r[1], r[2], r[3]))
                    .unwrap_or(Quat::IDENTITY);
                let scale = node.scale.map(Vec3::from_array).unwrap_or(Vec3::ONE);

                let local_transform =
                    Affine3A::from_scale_rotation_translation(scale, rotation, translation);
                let pivot = translation;

                for prim in &mesh_def.primitives {
                    let mut chunk_mesh = ChunkMesh::default();

                    // 1. POSITION (Accessor)
                    let pos_acc_idx = *prim
                        .attributes
                        .get("POSITION")
                        .ok_or_else(|| GltfAssetError::MissingAttribute("POSITION".into()))?;
                    let positions = Self::read_vec3_accessor(pos_acc_idx, &root, bin_bytes)?;

                    // 2. NORMAL (Optional, computed if missing)
                    let normals = if let Some(&norm_acc_idx) = prim.attributes.get("NORMAL") {
                        Self::read_vec3_accessor(norm_acc_idx, &root, bin_bytes)?
                    } else {
                        vec![[0.0, 1.0, 0.0]; positions.len()]
                    };

                    // 3. TEXCOORD_0 (Optional)
                    let uvs = if let Some(&uv_acc_idx) = prim.attributes.get("TEXCOORD_0") {
                        Self::read_vec2_accessor(uv_acc_idx, &root, bin_bytes)?
                    } else {
                        vec![[0.0, 0.0]; positions.len()]
                    };

                    // 4. COLOR_0 (Optional)
                    let colors = if let Some(&col_acc_idx) = prim.attributes.get("COLOR_0") {
                        Self::read_vec4_accessor(col_acc_idx, &root, bin_bytes)?
                    } else {
                        vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]
                    };

                    // 5. INDICES
                    let indices = if let Some(ind_acc_idx) = prim.indices {
                        Self::read_indices_accessor(ind_acc_idx, &root, bin_bytes)?
                    } else {
                        (0..positions.len() as u32).collect()
                    };

                    for i in 0..positions.len() {
                        chunk_mesh.vertices.push(Vertex::new(
                            positions[i],
                            normals[i],
                            uvs[i],
                            colors[i],
                        ));
                    }
                    chunk_mesh.indices = indices;

                    // Material
                    let material = if let Some(mat_idx) = prim.material {
                        if mat_idx < root.materials.len() {
                            let m = &root.materials[mat_idx];
                            let pbr = m.pbr_metallic_roughness.as_ref();
                            VehiclePbrMaterial {
                                base_color: pbr
                                    .map(|p| p.base_color_factor)
                                    .unwrap_or([1.0, 1.0, 1.0, 1.0]),
                                metallic: pbr.map(|p| p.metallic_factor).unwrap_or(0.5),
                                roughness: pbr.map(|p| p.roughness_factor).unwrap_or(0.5),
                                emissive: m.emissive_factor.unwrap_or([0.0, 0.0, 0.0]),
                                alpha_mode: VehicleAlphaMode::Opaque,
                            }
                        } else {
                            VehiclePbrMaterial::default()
                        }
                    } else {
                        VehiclePbrMaterial::default()
                    };

                    submeshes.push(VehicleSubmesh {
                        name: node_name.clone(),
                        submesh_type,
                        mesh: chunk_mesh,
                        local_transform,
                        pivot,
                        material,
                    });
                }
            }
        }

        // Calculate bounding box dimensions
        let mut min_bound = Vec3::splat(f32::INFINITY);
        let mut max_bound = Vec3::splat(f32::NEG_INFINITY);

        for s in &submeshes {
            for v in &s.mesh.vertices {
                let p = Vec3::from_array(v.position) + s.pivot;
                min_bound = min_bound.min(p);
                max_bound = max_bound.max(p);
            }
        }

        let dimensions = if min_bound.x.is_finite() && max_bound.x.is_finite() {
            max_bound - min_bound
        } else {
            Vec3::new(1.8, 1.4, 4.5)
        };

        Ok(LoadedVehicleModel {
            asset_id: asset_id.to_string(),
            display_name: asset_id.replace('_', " ").to_uppercase(),
            submeshes,
            dimensions,
            wheelbase: 2.50,
            track_width: 1.65,
            wheel_radius: 0.33,
            mass_kg: 1250.0,
            lod_level: lod,
            camera_anchors: VehicleCameraAnchors::default(),
        })
    }

    fn read_vec3_accessor(
        acc_idx: usize,
        root: &GltfJsonRoot,
        bin: &[u8],
    ) -> Result<Vec<[f32; 3]>, GltfAssetError> {
        let acc = root
            .accessors
            .get(acc_idx)
            .ok_or(GltfAssetError::InvalidAccessor {
                index: acc_idx,
                count: root.accessors.len(),
            })?;
        let bv = root
            .buffer_views
            .get(acc.buffer_view)
            .ok_or_else(|| GltfAssetError::CorruptChunk("Invalid bufferView index".into()))?;

        let start = bv.byte_offset + acc.byte_offset;
        let count = acc.count;
        let byte_len = count * 12;

        if start + byte_len > bin.len() {
            return Err(GltfAssetError::BufferOutOfBounds {
                offset: start,
                length: byte_len,
                buffer_size: bin.len(),
            });
        }

        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let offset = start + i * 12;
            let x = f32::from_le_bytes(bin[offset..offset + 4].try_into().unwrap());
            let y = f32::from_le_bytes(bin[offset + 4..offset + 8].try_into().unwrap());
            let z = f32::from_le_bytes(bin[offset + 8..offset + 12].try_into().unwrap());
            result.push([x, y, z]);
        }
        Ok(result)
    }

    fn read_vec2_accessor(
        acc_idx: usize,
        root: &GltfJsonRoot,
        bin: &[u8],
    ) -> Result<Vec<[f32; 2]>, GltfAssetError> {
        let acc = root
            .accessors
            .get(acc_idx)
            .ok_or(GltfAssetError::InvalidAccessor {
                index: acc_idx,
                count: root.accessors.len(),
            })?;
        let bv = &root.buffer_views[acc.buffer_view];
        let start = bv.byte_offset + acc.byte_offset;
        let count = acc.count;
        let mut result = Vec::with_capacity(count);

        for i in 0..count {
            let offset = start + i * 8;
            let u = f32::from_le_bytes(bin[offset..offset + 4].try_into().unwrap());
            let v = f32::from_le_bytes(bin[offset + 4..offset + 8].try_into().unwrap());
            result.push([u, v]);
        }
        Ok(result)
    }

    fn read_vec4_accessor(
        acc_idx: usize,
        root: &GltfJsonRoot,
        bin: &[u8],
    ) -> Result<Vec<[f32; 4]>, GltfAssetError> {
        let acc = root
            .accessors
            .get(acc_idx)
            .ok_or(GltfAssetError::InvalidAccessor {
                index: acc_idx,
                count: root.accessors.len(),
            })?;
        let bv = &root.buffer_views[acc.buffer_view];
        let start = bv.byte_offset + acc.byte_offset;
        let count = acc.count;
        let mut result = Vec::with_capacity(count);

        for i in 0..count {
            let offset = start + i * 16;
            let r = f32::from_le_bytes(bin[offset..offset + 4].try_into().unwrap());
            let g = f32::from_le_bytes(bin[offset + 4..offset + 8].try_into().unwrap());
            let b = f32::from_le_bytes(bin[offset + 8..offset + 12].try_into().unwrap());
            let a = f32::from_le_bytes(bin[offset + 12..offset + 16].try_into().unwrap());
            result.push([r, g, b, a]);
        }
        Ok(result)
    }

    fn read_indices_accessor(
        acc_idx: usize,
        root: &GltfJsonRoot,
        bin: &[u8],
    ) -> Result<Vec<u32>, GltfAssetError> {
        let acc = root
            .accessors
            .get(acc_idx)
            .ok_or(GltfAssetError::InvalidAccessor {
                index: acc_idx,
                count: root.accessors.len(),
            })?;
        let bv = &root.buffer_views[acc.buffer_view];
        let start = bv.byte_offset + acc.byte_offset;
        let count = acc.count;
        let mut result = Vec::with_capacity(count);

        match acc.component_type {
            5123 => {
                // UNSIGNED_SHORT (u16)
                for i in 0..count {
                    let offset = start + i * 2;
                    let val =
                        u16::from_le_bytes(bin[offset..offset + 2].try_into().unwrap()) as u32;
                    result.push(val);
                }
            }
            5125 => {
                // UNSIGNED_INT (u32)
                for i in 0..count {
                    let offset = start + i * 4;
                    let val = u32::from_le_bytes(bin[offset..offset + 4].try_into().unwrap());
                    result.push(val);
                }
            }
            other => {
                return Err(GltfAssetError::CorruptChunk(format!(
                    "Unsupported index component type: {}",
                    other
                )));
            }
        }
        Ok(result)
    }
}

/// Standards-compliant GLB binary builder for authoring certified vehicle assets.
pub struct GltfAssetBuilder {
    asset_id: String,
    display_name: String,
    submeshes: Vec<VehicleSubmesh>,
    dimensions: Vec3,
    wheelbase: f32,
    track_width: f32,
    wheel_radius: f32,
    mass_kg: f32,
    lod: VehicleLodLevel,
    camera_anchors: VehicleCameraAnchors,
}

impl GltfAssetBuilder {
    pub fn new(asset_id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            asset_id: asset_id.into(),
            display_name: display_name.into(),
            submeshes: Vec::new(),
            dimensions: Vec3::new(1.85, 1.25, 4.45),
            wheelbase: 2.50,
            track_width: 1.65,
            wheel_radius: 0.33,
            mass_kg: 1250.0,
            lod: VehicleLodLevel::Lod0High,
            camera_anchors: VehicleCameraAnchors::default(),
        }
    }

    pub fn with_physics(
        mut self,
        wheelbase: f32,
        track_width: f32,
        wheel_radius: f32,
        mass_kg: f32,
    ) -> Self {
        self.wheelbase = wheelbase;
        self.track_width = track_width;
        self.wheel_radius = wheel_radius;
        self.mass_kg = mass_kg;
        self
    }

    pub fn with_camera_anchors(mut self, anchors: VehicleCameraAnchors) -> Self {
        self.camera_anchors = anchors;
        self
    }

    pub fn with_lod(mut self, lod: VehicleLodLevel) -> Self {
        self.lod = lod;
        self
    }

    pub fn add_submesh(mut self, submesh: VehicleSubmesh) -> Self {
        self.submeshes.push(submesh);
        self
    }

    /// Serializes the submeshes and metadata into a certified Khronos glTF 2.0 binary GLB byte buffer.
    pub fn to_glb_bytes(&self) -> Result<Vec<u8>, GltfAssetError> {
        let mut bin_buffer = Vec::new();

        let mut gltf_nodes = Vec::new();
        let mut gltf_meshes = Vec::new();
        let mut gltf_accessors = Vec::new();
        let mut gltf_buffer_views = Vec::new();
        let mut gltf_materials = Vec::new();

        for submesh in &self.submeshes {
            let vert_count = submesh.mesh.vertices.len();
            let ind_count = submesh.mesh.indices.len();

            if vert_count == 0 || ind_count == 0 {
                continue;
            }

            // Material
            let mat_idx = gltf_materials.len();
            gltf_materials.push(GltfMaterial {
                name: Some(format!("{}_Mat", submesh.name)),
                pbr_metallic_roughness: Some(GltfPbr {
                    base_color_factor: submesh.material.base_color,
                    metallic_factor: submesh.material.metallic,
                    roughness_factor: submesh.material.roughness,
                }),
                emissive_factor: Some(submesh.material.emissive),
            });

            // 1. Pack POSITION (VEC3 FLOAT)
            let pos_offset = bin_buffer.len();
            for v in &submesh.mesh.vertices {
                bin_buffer.extend_from_slice(&v.position[0].to_le_bytes());
                bin_buffer.extend_from_slice(&v.position[1].to_le_bytes());
                bin_buffer.extend_from_slice(&v.position[2].to_le_bytes());
            }
            let pos_bv_idx = gltf_buffer_views.len();
            gltf_buffer_views.push(GltfBufferView {
                buffer: 0,
                byte_offset: pos_offset,
                byte_length: vert_count * 12,
                target: Some(34962), // ARRAY_BUFFER
            });
            let pos_acc_idx = gltf_accessors.len();
            gltf_accessors.push(GltfAccessor {
                buffer_view: pos_bv_idx,
                byte_offset: 0,
                component_type: 5126, // FLOAT
                count: vert_count,
                type_str: "VEC3".into(),
                min: None,
                max: None,
            });

            // 2. Pack NORMAL (VEC3 FLOAT)
            let norm_offset = bin_buffer.len();
            for v in &submesh.mesh.vertices {
                bin_buffer.extend_from_slice(&v.normal[0].to_le_bytes());
                bin_buffer.extend_from_slice(&v.normal[1].to_le_bytes());
                bin_buffer.extend_from_slice(&v.normal[2].to_le_bytes());
            }
            let norm_bv_idx = gltf_buffer_views.len();
            gltf_buffer_views.push(GltfBufferView {
                buffer: 0,
                byte_offset: norm_offset,
                byte_length: vert_count * 12,
                target: Some(34962),
            });
            let norm_acc_idx = gltf_accessors.len();
            gltf_accessors.push(GltfAccessor {
                buffer_view: norm_bv_idx,
                byte_offset: 0,
                component_type: 5126,
                count: vert_count,
                type_str: "VEC3".into(),
                min: None,
                max: None,
            });

            // 3. Pack TEXCOORD_0 (VEC2 FLOAT)
            let uv_offset = bin_buffer.len();
            for v in &submesh.mesh.vertices {
                bin_buffer.extend_from_slice(&v.uv[0].to_le_bytes());
                bin_buffer.extend_from_slice(&v.uv[1].to_le_bytes());
            }
            let uv_bv_idx = gltf_buffer_views.len();
            gltf_buffer_views.push(GltfBufferView {
                buffer: 0,
                byte_offset: uv_offset,
                byte_length: vert_count * 8,
                target: Some(34962),
            });
            let uv_acc_idx = gltf_accessors.len();
            gltf_accessors.push(GltfAccessor {
                buffer_view: uv_bv_idx,
                byte_offset: 0,
                component_type: 5126,
                count: vert_count,
                type_str: "VEC2".into(),
                min: None,
                max: None,
            });

            // 4. Pack COLOR_0 (VEC4 FLOAT)
            let col_offset = bin_buffer.len();
            for v in &submesh.mesh.vertices {
                bin_buffer.extend_from_slice(&v.color[0].to_le_bytes());
                bin_buffer.extend_from_slice(&v.color[1].to_le_bytes());
                bin_buffer.extend_from_slice(&v.color[2].to_le_bytes());
                bin_buffer.extend_from_slice(&v.color[3].to_le_bytes());
            }
            let col_bv_idx = gltf_buffer_views.len();
            gltf_buffer_views.push(GltfBufferView {
                buffer: 0,
                byte_offset: col_offset,
                byte_length: vert_count * 16,
                target: Some(34962),
            });
            let col_acc_idx = gltf_accessors.len();
            gltf_accessors.push(GltfAccessor {
                buffer_view: col_bv_idx,
                byte_offset: 0,
                component_type: 5126,
                count: vert_count,
                type_str: "VEC4".into(),
                min: None,
                max: None,
            });

            // 5. Pack INDICES (SCALAR UNSIGNED_INT)
            let ind_offset = bin_buffer.len();
            for idx in &submesh.mesh.indices {
                bin_buffer.extend_from_slice(&idx.to_le_bytes());
            }
            let ind_bv_idx = gltf_buffer_views.len();
            gltf_buffer_views.push(GltfBufferView {
                buffer: 0,
                byte_offset: ind_offset,
                byte_length: ind_count * 4,
                target: Some(34963), // ELEMENT_ARRAY_BUFFER
            });
            let ind_acc_idx = gltf_accessors.len();
            gltf_accessors.push(GltfAccessor {
                buffer_view: ind_bv_idx,
                byte_offset: 0,
                component_type: 5125, // UNSIGNED_INT
                count: ind_count,
                type_str: "SCALAR".into(),
                min: None,
                max: None,
            });

            let mut attributes = HashMap::new();
            attributes.insert("POSITION".into(), pos_acc_idx);
            attributes.insert("NORMAL".into(), norm_acc_idx);
            attributes.insert("TEXCOORD_0".into(), uv_acc_idx);
            attributes.insert("COLOR_0".into(), col_acc_idx);

            let mesh_idx = gltf_meshes.len();
            gltf_meshes.push(GltfMesh {
                name: Some(submesh.name.clone()),
                primitives: vec![GltfPrimitive {
                    attributes,
                    indices: Some(ind_acc_idx),
                    material: Some(mat_idx),
                }],
            });

            gltf_nodes.push(GltfNode {
                name: Some(submesh.name.clone()),
                mesh: Some(mesh_idx),
                translation: Some(submesh.pivot.to_array()),
                rotation: None,
                scale: None,
                children: Vec::new(),
            });
        }

        let root_scene = GltfScene {
            nodes: (0..gltf_nodes.len()).collect(),
        };

        let root = GltfJsonRoot {
            asset: GltfAssetHeader {
                version: "2.0".into(),
                generator: Some("RealEarth Production Vehicle Pipeline 2.0".into()),
            },
            scene: 0,
            scenes: vec![root_scene],
            nodes: gltf_nodes,
            meshes: gltf_meshes,
            accessors: gltf_accessors,
            buffer_views: gltf_buffer_views,
            buffers: vec![GltfBuffer {
                byte_length: bin_buffer.len(),
            }],
            materials: gltf_materials,
        };

        let json_text = serde_json::to_string(&root)?;
        let mut json_bytes = json_text.into_bytes();

        // 4-byte boundary padding for JSON chunk (space character 0x20)
        let json_padding = (4 - (json_bytes.len() % 4)) % 4;
        json_bytes.extend(std::iter::repeat_n(b' ', json_padding));

        // 4-byte boundary padding for BIN chunk (null byte 0x00)
        let bin_padding = (4 - (bin_buffer.len() % 4)) % 4;
        bin_buffer.extend(std::iter::repeat_n(0u8, bin_padding));

        let total_file_length = 12 + 8 + json_bytes.len() + 8 + bin_buffer.len();

        let mut glb_file = Vec::with_capacity(total_file_length);

        // 1. 12-byte GLB Header
        glb_file.extend_from_slice(&GLB_MAGIC.to_le_bytes());
        glb_file.extend_from_slice(&GLB_VERSION.to_le_bytes());
        glb_file.extend_from_slice(&(total_file_length as u32).to_le_bytes());

        // 2. Chunk 0: JSON Chunk
        glb_file.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb_file.extend_from_slice(&CHUNK_TYPE_JSON.to_le_bytes());
        glb_file.extend_from_slice(&json_bytes);

        // 3. Chunk 1: BIN Chunk
        glb_file.extend_from_slice(&(bin_buffer.len() as u32).to_le_bytes());
        glb_file.extend_from_slice(&CHUNK_TYPE_BIN.to_le_bytes());
        glb_file.extend_from_slice(&bin_buffer);

        Ok(glb_file)
    }

    /// Explicitly sets vehicle dimensions.
    pub fn with_dimensions(mut self, dimensions: Vec3) -> Self {
        self.dimensions = dimensions;
        self
    }

    /// Builds and returns the in-memory `LoadedVehicleModel`.
    pub fn build(self) -> LoadedVehicleModel {
        let mut min_bound = Vec3::splat(f32::INFINITY);
        let mut max_bound = Vec3::splat(f32::NEG_INFINITY);

        for s in &self.submeshes {
            for v in &s.mesh.vertices {
                let p = Vec3::from_array(v.position) + s.pivot;
                min_bound = min_bound.min(p);
                max_bound = max_bound.max(p);
            }
        }

        let dimensions = if min_bound.x.is_finite() && max_bound.x.is_finite() {
            max_bound - min_bound
        } else {
            self.dimensions
        };

        LoadedVehicleModel {
            asset_id: self.asset_id,
            display_name: self.display_name,
            submeshes: self.submeshes,
            dimensions,
            wheelbase: self.wheelbase,
            track_width: self.track_width,
            wheel_radius: self.wheel_radius,
            mass_kg: self.mass_kg,
            lod_level: self.lod,
            camera_anchors: self.camera_anchors,
        }
    }
}

/// Global asset registry providing validated caching, hot-loading, and LOD selection.
pub struct VehicleAssetRegistry {
    models: HashMap<String, LoadedVehicleModel>,
    asset_dir: PathBuf,
}

impl VehicleAssetRegistry {
    pub fn new(asset_dir: impl Into<PathBuf>) -> Self {
        Self {
            models: HashMap::new(),
            asset_dir: asset_dir.into(),
        }
    }

    /// Loads and registers a model from raw binary GLB bytes.
    pub fn register_glb_bytes(
        &mut self,
        asset_id: &str,
        bytes: &[u8],
        lod: VehicleLodLevel,
    ) -> Result<&LoadedVehicleModel, GltfAssetError> {
        let model = GltfParser::parse_glb(asset_id, bytes, lod)?;
        self.models.insert(asset_id.to_string(), model);
        Ok(self.models.get(asset_id).unwrap())
    }

    /// Registers an in-memory preconstructed `LoadedVehicleModel`.
    pub fn register_model(&mut self, model: LoadedVehicleModel) {
        self.models.insert(model.asset_id.clone(), model);
    }

    /// Loads a model from disk if not already cached.
    pub fn load_or_get(&mut self, asset_id: &str) -> Result<&LoadedVehicleModel, GltfAssetError> {
        if !self.models.contains_key(asset_id) {
            let filename = format!("{}.glb", asset_id);
            let path = self.asset_dir.join(&filename);
            if !path.exists() {
                return Err(GltfAssetError::MissingAsset(format!(
                    "Asset '{}' not found at path '{}'",
                    asset_id,
                    path.display()
                )));
            }
            let bytes = fs::read(&path)?;
            let model = GltfParser::parse_glb(asset_id, &bytes, VehicleLodLevel::Lod0High)?;
            self.models.insert(asset_id.to_string(), model);
        }
        Ok(self.models.get(asset_id).unwrap())
    }

    /// Retrieves a cached model or returns an explicit error if missing.
    pub fn require(&self, asset_id: &str) -> Result<&LoadedVehicleModel, GltfAssetError> {
        self.models
            .get(asset_id)
            .ok_or_else(|| GltfAssetError::MissingAsset(asset_id.to_string()))
    }

    /// Total number of cached models.
    pub fn len(&self) -> usize {
        self.models.len()
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}
