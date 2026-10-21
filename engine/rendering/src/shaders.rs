//! WGSL PBR terrain, building, road, and water shader definitions.

pub const WORLD_SHADER_WGSL: &str = r#"
struct SceneUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color_intensity: vec4<f32>,
    ambient_light: vec4<f32>,
    fog_params: vec4<f32>,
    material_params: vec4<f32>,
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

    // Day/Night and Sun Zenith angle transitions
    let sun_elevation = sun_dir.y;
    let day_factor = clamp(sun_elevation * 3.0 + 0.1, 0.05, 1.0);

    // Directional sunlight diffuse factor
    let n_dot_l = max(dot(normal, sun_dir), 0.0);
    let diffuse = uniforms.sun_color_intensity.rgb * n_dot_l * uniforms.sun_color_intensity.w * day_factor;

    // Ambient light: cool blue-tinted night or warm ambient day
    let night_ambient = vec3<f32>(0.08, 0.09, 0.14);
    let day_ambient = uniforms.ambient_light.rgb;
    let ambient = mix(night_ambient, day_ambient, day_factor);

    // Base surface color
    var base_color = in.color.rgb;

    // Procedural Road Markings (white dashed lane dividers & double yellow centerlines)
    let is_road_marking_active = uniforms.material_params.y > 0.5;
    // Check if this fragment belongs to asphalt road ribbon (dark base tone)
    let is_asphalt = in.color.r < 0.35 && in.color.g < 0.35 && in.color.b < 0.35 && in.color.a > 0.9;

    if (is_road_marking_active && is_asphalt) {
        let v = in.uv.y;
        let u = in.uv.x;

        // Double yellow centerline along median (v approx 0.5)
        let dist_center = abs(v - 0.5);
        let yellow_color = vec3<f32>(0.95, 0.78, 0.12);
        if ((dist_center > 0.015 && dist_center < 0.035)) {
            base_color = yellow_color;
        }

        // White dashed lane dividers at 1/4 and 3/4 roadway width
        let dist_lane1 = abs(v - 0.25);
        let dist_lane2 = abs(v - 0.75);
        let dash_pattern = fract(u * 8.0);
        let white_color = vec3<f32>(0.95, 0.95, 0.95);
        if ((dist_lane1 < 0.012 || dist_lane2 < 0.012) && dash_pattern < 0.55) {
            base_color = white_color;
        }
    }

    // Specular highlight with Fresnel reflectance (water, glass, moist asphalt)
    let half_dir = normalize(sun_dir + view_dir);
    let spec_base = pow(max(dot(normal, half_dir), 0.0), 64.0);
    let n_dot_v = max(dot(normal, view_dir), 0.0);
    let fresnel = 0.04 + 0.96 * pow(1.0 - n_dot_v, 5.0);
    let spec_intensity = uniforms.material_params.x;
    let specular = uniforms.sun_color_intensity.rgb * spec_base * fresnel * spec_intensity * day_factor;

    let lit_color = base_color * (ambient + diffuse) + specular;

    // Atmospheric Rayleigh Haze & Distance Fog
    let dist = length(uniforms.camera_pos.xyz - in.world_pos);
    let fog_start = uniforms.fog_params.y;
    let fog_end = uniforms.fog_params.z;
    let fog_density = uniforms.fog_params.x;

    let linear_fog = clamp((dist - fog_start) / max(fog_end - fog_start, 1.0), 0.0, 0.88);
    let exp_fog = 1.0 - exp(-dist * fog_density);
    let fog_factor = max(linear_fog, exp_fog);

    // Sky haze gradient matching time of day
    let day_sky = vec3<f32>(0.65, 0.76, 0.92);
    let sunset_sky = vec3<f32>(0.85, 0.52, 0.38);
    let night_sky = vec3<f32>(0.04, 0.06, 0.12);

    var sky_fog_color = day_sky;
    if (sun_elevation < 0.2) {
        let t = clamp((sun_elevation + 0.1) / 0.3, 0.0, 1.0);
        sky_fog_color = mix(sunset_sky, day_sky, t);
    }
    if (sun_elevation < 0.0) {
        let t = clamp((sun_elevation + 0.2) / 0.2, 0.0, 1.0);
        sky_fog_color = mix(night_sky, sunset_sky, t);
    }

    let final_rgb = mix(lit_color, sky_fog_color, fog_factor);

    return vec4<f32>(final_rgb, in.color.a);
}
"#;
