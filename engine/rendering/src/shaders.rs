//! WGSL PBR terrain, building, and road shader definitions.

pub const WORLD_SHADER_WGSL: &str = r#"
struct SceneUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color_intensity: vec4<f32>,
    ambient_light: vec4<f32>,
    fog_params: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: SceneUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = in.position;
    out.normal = normalize(in.normal);
    out.uv = in.uv;
    out.color = in.color;
    out.clip_position = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sun_dir = normalize(uniforms.sun_direction.xyz);
    let normal = normalize(in.normal);
    let view_dir = normalize(uniforms.camera_pos.xyz - in.world_pos);

    // Directional sunlight diffuse factor
    let n_dot_l = max(dot(normal, sun_dir), 0.0);
    let diffuse = uniforms.sun_color_intensity.rgb * n_dot_l * uniforms.sun_color_intensity.w;
    let ambient = uniforms.ambient_light.rgb;

    // Specular highlight (Blinn-Phong) for reflective materials (e.g. water, glass)
    let half_dir = normalize(sun_dir + view_dir);
    let spec_factor = pow(max(dot(normal, half_dir), 0.0), 48.0);
    let specular = uniforms.sun_color_intensity.rgb * spec_factor * 0.45 * uniforms.sun_color_intensity.w;

    let lit_color = in.color.rgb * (ambient + diffuse) + specular;

    // Atmospheric distance fog
    let dist = length(uniforms.camera_pos.xyz - in.world_pos);
    let fog_start = uniforms.fog_params.y;
    let fog_end = uniforms.fog_params.z;
    let fog_factor = clamp((dist - fog_start) / (fog_end - fog_start), 0.0, 0.85);

    let sky_fog_color = vec3<f32>(0.65, 0.76, 0.92);
    let final_rgb = mix(lit_color, sky_fog_color, fog_factor);

    return vec4<f32>(final_rgb, in.color.a);
}
"#;
