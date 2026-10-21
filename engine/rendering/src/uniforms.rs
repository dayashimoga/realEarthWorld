//! GPU uniform buffer definitions for lighting, camera matrices, atmospheric fog, and materials.

use bytemuck::{Pod, Zeroable};

/// Uniforms passed to vertex and fragment shaders.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SceneUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
    pub sun_direction: [f32; 4],
    pub sun_color_intensity: [f32; 4],
    pub ambient_light: [f32; 4],
    pub fog_params: [f32; 4],      // x: density, y: start, z: end, w: unused
    pub material_params: [f32; 4], // x: specular_intensity, y: road_markings (0/1), z: anim_time, w: unused
}

impl Default for SceneUniforms {
    fn default() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0, 0.0, 0.0, 1.0],
            sun_direction: [0.0, 1.0, 0.0, 0.0],
            sun_color_intensity: [1.0, 0.98, 0.92, 1.0],
            ambient_light: [0.22, 0.24, 0.28, 1.0],
            fog_params: [0.0002, 100.0, 4000.0, 0.0],
            material_params: [0.8, 1.0, 0.0, 0.0],
        }
    }
}
