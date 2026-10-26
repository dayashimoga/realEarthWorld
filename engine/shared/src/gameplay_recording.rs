//! Deterministic Gameplay Input Recording and Replay Infrastructure.
//!
//! Synchronizes raw inputs, vehicle state, physics step, render transforms,
//! camera poses, collision dynamics, and loaded geographic source IDs for bitwise
//! repeatable regression validation and audit trails.

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// High-precision vehicle physical telemetry snapshot synchronized with input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedVehicleState {
    pub speed_kph: f32,
    pub engine_rpm: f32,
    pub gear: i8,
    pub steer_angle_rad: f32,
    pub slip_ratio: f32,
    pub slip_angle_deg: f32,
    pub handbrake: bool,
    pub park_engaged: bool,
    pub sim_state_name: String,
}

impl Default for RecordedVehicleState {
    fn default() -> Self {
        Self {
            speed_kph: 0.0,
            engine_rpm: 800.0,
            gear: 1,
            steer_angle_rad: 0.0,
            slip_ratio: 0.0,
            slip_angle_deg: 0.0,
            handbrake: false,
            park_engaged: false,
            sim_state_name: "Idle".to_string(),
        }
    }
}

/// 3D spatial transformation pose.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RecordedTransform {
    pub position: [f32; 3],
    pub orientation_quat: [f32; 4],
}

impl RecordedTransform {
    pub fn new(pos: Vec3, rot: Quat) -> Self {
        Self {
            position: [pos.x, pos.y, pos.z],
            orientation_quat: [rot.x, rot.y, rot.z, rot.w],
        }
    }

    pub fn pos_vec3(&self) -> Vec3 {
        Vec3::from_slice(&self.position)
    }

    pub fn rot_quat(&self) -> Quat {
        Quat::from_array(self.orientation_quat)
    }
}

impl Default for RecordedTransform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            orientation_quat: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Camera spatial perspective snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedCameraState {
    pub pos_wgs84: [f64; 3], // [lat, lon, alt_meters]
    pub yaw_rad: f32,
    pub pitch_rad: f32,
    pub fov_deg: f32,
    pub mode: String,
}

impl Default for RecordedCameraState {
    fn default() -> Self {
        Self {
            pos_wgs84: [37.7952, -122.4028, 50.0],
            yaw_rad: 0.0,
            pitch_rad: -0.2,
            fov_deg: 60.0,
            mode: "Chase".to_string(),
        }
    }
}

/// Collision dynamics and ground interaction snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedCollisionState {
    pub in_collision: bool,
    pub building_hit_id: Option<String>,
    pub surface_elevation: f32,
    pub surface_normal: [f32; 3],
}

impl Default for RecordedCollisionState {
    fn default() -> Self {
        Self {
            in_collision: false,
            building_hit_id: None,
            surface_elevation: 0.0,
            surface_normal: [0.0, 1.0, 0.0],
        }
    }
}

/// Active geographic datasets and chunks loaded for the recorded frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedGeographicContext {
    pub region_slug: String,
    pub package_anchor: [f64; 3],
    pub loaded_chunk_ids: Vec<String>,
    pub source_osm_sha256: Option<String>,
}

impl Default for RecordedGeographicContext {
    fn default() -> Self {
        Self {
            region_slug: "san-francisco".to_string(),
            package_anchor: [37.7952, -122.4028, 0.0],
            loaded_chunk_ids: vec!["chunk_0_0".to_string(), "chunk_0_1".to_string()],
            source_osm_sha256: None,
        }
    }
}

/// Comprehensive, synchronized deterministic gameplay frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameplayInputFrame {
    pub frame_index: usize,
    pub timestamp_ms: f64,
    pub delta_time_sec: f32,
    pub raw_keys: Vec<String>,
    pub mouse_buttons: Vec<String>,
    pub mouse_cursor: Option<[f64; 2]>,
    pub active_mode: String,
    pub vehicle_state: RecordedVehicleState,
    pub render_transform: RecordedTransform,
    pub camera_state: RecordedCameraState,
    pub collision_state: RecordedCollisionState,
    pub geographic_context: RecordedGeographicContext,
}

/// Verification results when playing back a recorded session against active simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayVerificationSummary {
    pub session_id: String,
    pub total_frames_verified: usize,
    pub max_position_divergence_meters: f32,
    pub max_velocity_divergence_mps: f32,
    pub max_steer_divergence_rad: f32,
    pub collision_events_matched: usize,
    pub geographic_anchors_matched: bool,
    pub is_bitwise_deterministic: bool,
    pub passed: bool,
    pub failure_reasons: Vec<String>,
}

/// High-fidelity gameplay recording session for regression replay and mutation tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameplayRecordingSession {
    pub session_id: String,
    pub recording_name: String,
    pub target_region: String,
    pub vehicle_type: String,
    pub recorded_at_utc: String,
    pub target_fps: f32,
    pub frames: Vec<GameplayInputFrame>,
}

impl GameplayRecordingSession {
    pub fn new(
        session_id: impl Into<String>,
        recording_name: impl Into<String>,
        target_region: impl Into<String>,
        vehicle_type: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            recording_name: recording_name.into(),
            target_region: target_region.into(),
            vehicle_type: vehicle_type.into(),
            recorded_at_utc: "2026-10-26T00:00:00Z".to_string(),
            target_fps: 60.0,
            frames: Vec::new(),
        }
    }

    pub fn push_frame(&mut self, frame: GameplayInputFrame) {
        self.frames.push(frame);
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Serializes session to JSON file on disk.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file =
            File::create(path).map_err(|e| format!("Failed to create recording file: {}", e))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)
            .map_err(|e| format!("Failed to serialize recording: {}", e))
    }

    /// Deserializes recording session from JSON file on disk.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let file = File::open(path.as_ref())
            .map_err(|e| format!("Failed to open recording file: {}", e))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| format!("Failed to parse recording: {}", e))
    }

    /// Validates determinism against an active playback run with strict tolerances.
    pub fn compare_replay(
        &self,
        replayed_frames: &[GameplayInputFrame],
        pos_tolerance_m: f32,
        vel_tolerance_mps: f32,
    ) -> ReplayVerificationSummary {
        let mut max_pos_div = 0.0f32;
        let mut max_vel_div = 0.0f32;
        let mut max_steer_div = 0.0f32;
        let mut col_matched = 0;
        let mut reasons = Vec::new();

        let count = self.frames.len().min(replayed_frames.len());
        if self.frames.len() != replayed_frames.len() {
            reasons.push(format!(
                "Frame count mismatch: recorded {} vs replayed {}",
                self.frames.len(),
                replayed_frames.len()
            ));
        }

        let mut anchors_matched = true;

        for (orig, replay) in self.frames.iter().zip(replayed_frames.iter()) {
            let p1 = orig.render_transform.pos_vec3();
            let p2 = replay.render_transform.pos_vec3();
            let pos_delta = (p1 - p2).length();
            if pos_delta > max_pos_div {
                max_pos_div = pos_delta;
            }

            let v1 = orig.vehicle_state.speed_kph * (1000.0 / 3600.0);
            let v2 = replay.vehicle_state.speed_kph * (1000.0 / 3600.0);
            let vel_delta = (v1 - v2).abs();
            if vel_delta > max_vel_div {
                max_vel_div = vel_delta;
            }

            let steer_delta =
                (orig.vehicle_state.steer_angle_rad - replay.vehicle_state.steer_angle_rad).abs();
            if steer_delta > max_steer_div {
                max_steer_div = steer_delta;
            }

            if orig.collision_state.in_collision == replay.collision_state.in_collision {
                col_matched += 1;
            }

            if orig.geographic_context.region_slug != replay.geographic_context.region_slug {
                anchors_matched = false;
            }
        }

        if max_pos_div > pos_tolerance_m {
            reasons.push(format!(
                "Max position divergence ({:.4}m) exceeded tolerance ({:.4}m)",
                max_pos_div, pos_tolerance_m
            ));
        }

        if max_vel_div > vel_tolerance_mps {
            reasons.push(format!(
                "Max velocity divergence ({:.4}m/s) exceeded tolerance ({:.4}m/s)",
                max_vel_div, vel_tolerance_mps
            ));
        }

        let bitwise = max_pos_div < 1e-6 && max_vel_div < 1e-6;
        let passed = reasons.is_empty();

        ReplayVerificationSummary {
            session_id: self.session_id.clone(),
            total_frames_verified: count,
            max_position_divergence_meters: max_pos_div,
            max_velocity_divergence_mps: max_vel_div,
            max_steer_divergence_rad: max_steer_div,
            collision_events_matched: col_matched,
            geographic_anchors_matched: anchors_matched,
            is_bitwise_deterministic: bitwise,
            passed,
            failure_reasons: reasons,
        }
    }
}
