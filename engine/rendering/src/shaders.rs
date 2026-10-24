//! WGSL PBR terrain, architectural, road, water, and atmospheric scattering shader definitions.

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

// Pseudo-random hash for procedural variation
fn hash2(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(123.34, 456.21));
    return fract(dot(q, q + 45.32));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sun_dir = normalize(uniforms.sun_direction.xyz);
    let normal = normalize(in.normal);
    let view_dir = normalize(uniforms.camera_pos.xyz - in.world_pos);
    let anim_time = uniforms.material_params.z;

    // Day/Night and Sun Zenith angle transitions
    let sun_elevation = sun_dir.y;
    let day_factor = clamp(sun_elevation * 3.0 + 0.1, 0.05, 1.0);

    // Directional sunlight diffuse factor with soft wrap-around
    let n_dot_l = max(dot(normal, sun_dir), 0.0);
    let diffuse = uniforms.sun_color_intensity.rgb * n_dot_l * uniforms.sun_color_intensity.w * day_factor;

    // Ambient light: cool blue night or warm sky ambient during day
    let night_ambient = vec3<f32>(0.07, 0.08, 0.14);
    let day_ambient = uniforms.ambient_light.rgb;
    let ambient = mix(night_ambient, day_ambient, day_factor);

    // Base surface albedo and material classification from vertex attributes
    var base_color = in.color.rgb;
    let alpha_tag = in.color.a;

    // Material Classification Tags:
    // 0.90 = Asphalt carriageway
    // 0.92 = Concrete curb / sidewalk
    // 0.80 = Glass curtain wall skyscraper
    // 0.82 = Granite / stone facade
    // 0.84 = Brick masonry
    // 0.86 = Civic concrete / bridge pier
    // 0.88 = Victorian painted wood siding
    // 0.70 = Rooftop infrastructure / street furniture metal
    // 0.60 = San Francisco Bay water
    // 0.50 = Tree foliage / canopy
    // 0.40 = Tree trunk wood bark
    // 1.00 = Natural terrain / ground

    let is_asphalt = abs(alpha_tag - 0.90) < 0.015;
    let is_sidewalk = abs(alpha_tag - 0.92) < 0.015;
    let is_road_marking_active = uniforms.material_params.y > 0.5;

    // =========================================================================
    // 1. ROADS & URBAN INFRASTRUCTURE: Asphalt, Lane Dividers, Centerlines, Crosswalks
    // =========================================================================
    if (is_road_marking_active && is_asphalt) {
        let u = in.uv.x; // lateral position [0, 1] across carriageway
        let v = in.uv.y; // distance along road in meters

        // Subtle asphalt grain texture
        let asphalt_grain = (hash2(vec2<f32>(u * 50.0, v * 5.0)) - 0.5) * 0.05;
        base_color = clamp(base_color + vec3<f32>(asphalt_grain), vec3<f32>(0.05), vec3<f32>(0.35));

        let yellow_color = vec3<f32>(0.96, 0.76, 0.12);
        let white_color = vec3<f32>(0.94, 0.94, 0.96);

        // A. Double Solid Yellow Centerline with black median gap
        let dist_center = abs(u - 0.5);
        if (dist_center > 0.012 && dist_center < 0.038) {
            base_color = yellow_color;
        }

        // B. White Dashed Lane Dividers (3m dash, 9m gap = 12m period)
        let dist_lane1 = abs(u - 0.25);
        let dist_lane2 = abs(u - 0.75);
        let dash_phase = fract(v / 12.0);
        if ((dist_lane1 < 0.010 || dist_lane2 < 0.010) && dash_phase < 0.32) {
            base_color = white_color;
        }

        // C. Solid White Edge Lines (Fog Lines) along road shoulders
        let dist_edge = min(u, 1.0 - u);
        if (dist_edge > 0.020 && dist_edge < 0.035) {
            base_color = white_color;
        }

        // D. Pedestrian Zebra Crosswalk Stripes near road segment boundaries
        let crosswalk_interval = fract(v / 80.0);
        if (crosswalk_interval < 0.07) {
            let stripe = fract(u * 16.0);
            if (stripe < 0.55 && u > 0.05 && u < 0.95) {
                base_color = white_color;
            }
        }
    }

    // Concrete sidewalk pavement slabs (expansion joints every 2 meters)
    if (is_sidewalk) {
        let slab_joint = fract(in.uv.y / 2.0);
        let lateral_joint = fract(in.uv.x / 1.5);
        if (slab_joint < 0.04 || lateral_joint < 0.035) {
            base_color = base_color * 0.78; // Dark scored joint line
        }
    }

    // =========================================================================
    // 2. ARCHITECTURAL REALISM: PBR Facades, Windows, Cornices and Night Glow
    // =========================================================================
    let is_vertical_facade = abs(normal.y) < 0.25;

    // Glass Curtain Wall Skyscraper (Salesforce, modern financial towers)
    if (is_vertical_facade && abs(alpha_tag - 0.80) < 0.015) {
        let floor_h = 3.6;
        let window_w = 2.4;
        let floor_coord = fract(in.uv.y / floor_h);
        let horiz_coord = fract(in.uv.x / window_w);

        let is_mullion = floor_coord < 0.12 || floor_coord > 0.90 || horiz_coord < 0.10 || horiz_coord > 0.90;
        let mullion_color = vec3<f32>(0.28, 0.30, 0.34);

        if (is_mullion) {
            base_color = mullion_color;
        } else {
            let night_glow = vec3<f32>(1.0, 0.88, 0.58) * (1.0 - day_factor) * 0.90;
            let glass_tint = vec3<f32>(0.16, 0.28, 0.38);
            base_color = mix(glass_tint, glass_tint + night_glow, 1.0 - day_factor);
        }

        // Horizontal spandrel band dividing mechanical registers
        if (floor_coord < 0.20) {
            base_color = base_color * 0.75;
        }
    }

    // Granite / Stone Facade (555 California, Civic, classic commercial)
    if (is_vertical_facade && abs(alpha_tag - 0.82) < 0.015) {
        let course_h = 1.4;
        let course_w = 2.8;
        let c_y = fract(in.uv.y / course_h);
        let c_x = fract(in.uv.x / course_w);
        if (c_y < 0.06 || c_x < 0.04) {
            base_color = base_color * 0.82; // Ashlar stone mortar joint
        }
        // Punch rectangular windows
        let w_y = fract(in.uv.y / 3.6);
        let w_x = fract(in.uv.x / 3.0);
        if (w_y > 0.25 && w_y < 0.82 && w_x > 0.22 && w_x < 0.78) {
            let night_glow = vec3<f32>(0.98, 0.84, 0.50) * (1.0 - day_factor) * 0.85;
            let dark_glass = vec3<f32>(0.12, 0.16, 0.22);
            base_color = mix(dark_glass, dark_glass + night_glow, 1.0 - day_factor);
        }

        // Belt course / architectural cornice every 3 floors (~10.8m)
        let cornice_coord = fract(in.uv.y / 10.8);
        if (cornice_coord < 0.07) {
            base_color = base_color * 1.15; // Projecting stone cornice highlight
        } else if (cornice_coord < 0.11) {
            base_color = base_color * 0.70; // Cornice cast shadow groove
        }
    }

    // Brick Masonry (SOMA lofts, residential)
    if (is_vertical_facade && abs(alpha_tag - 0.84) < 0.015) {
        let brick_h = 0.25;
        let brick_w = 0.60;
        let row = floor(in.uv.y / brick_h);
        let b_y = fract(in.uv.y / brick_h);
        let b_x = fract((in.uv.x + row * 0.30) / brick_w);
        if (b_y < 0.10 || b_x < 0.06) {
            base_color = vec3<f32>(0.75, 0.72, 0.68); // Light gray mortar joint
        }
        // Punched residential windows
        let rw_y = fract(in.uv.y / 3.2);
        let rw_x = fract(in.uv.x / 2.6);
        if (rw_y > 0.30 && rw_y < 0.80 && rw_x > 0.25 && rw_x < 0.75) {
            let night_glow = vec3<f32>(0.95, 0.82, 0.45) * (1.0 - day_factor) * 0.80;
            let win_glass = vec3<f32>(0.14, 0.18, 0.22);
            base_color = mix(win_glass, win_glass + night_glow, 1.0 - day_factor);
        }
        // Decorative brick soldier course / lintel above windows
        if (rw_y >= 0.80 && rw_y < 0.88 && rw_x > 0.20 && rw_x < 0.80) {
            base_color = base_color * 0.80;
        }
    }

    // Civic Concrete / Architectural Brutalism (alpha 0.86)
    if (is_vertical_facade && abs(alpha_tag - 0.86) < 0.015) {
        let panel_h = 1.8;
        let panel_w = 3.6;
        let p_y = fract(in.uv.y / panel_h);
        let p_x = fract(in.uv.x / panel_w);
        // Formwork panel joint lines
        if (p_y < 0.03 || p_x < 0.02) {
            base_color = base_color * 0.80; // Dark recessed joint
        }
        // Form-tie circular indents
        let tie_y = min(abs(p_y - 0.25), abs(p_y - 0.75));
        let tie_x = min(abs(p_x - 0.20), abs(p_x - 0.80));
        if (tie_y < 0.03 && tie_x < 0.02) {
            base_color = base_color * 0.65; // Tie hole depression
        }
        // Architectural horizontal reveal belt
        let belt = fract(in.uv.y / 7.2);
        if (belt < 0.05) {
            base_color = base_color * 0.75;
        }
    }

    // Victorian Painted Wood Siding (Nob Hill, Russian Hill)
    if (is_vertical_facade && abs(alpha_tag - 0.88) < 0.015) {
        let clapboard = fract(in.uv.y / 0.35);
        if (clapboard < 0.12) {
            base_color = base_color * 0.75; // Clapboard horizontal shadow groove
        }
        // Victorian window trim with decorative header
        let vw_y = fract(in.uv.y / 3.4);
        let vw_x = fract(in.uv.x / 2.2);
        if (vw_y > 0.28 && vw_y < 0.82 && vw_x > 0.24 && vw_x < 0.76) {
            let night_glow = vec3<f32>(1.0, 0.90, 0.60) * (1.0 - day_factor) * 0.85;
            let win_glass = vec3<f32>(0.15, 0.20, 0.26);
            base_color = mix(win_glass, win_glass + night_glow, 1.0 - day_factor);
        } else if ((vw_y >= 0.24 && vw_y <= 0.86 && (vw_x < 0.24 || vw_x > 0.76)) ||
                   (vw_y >= 0.82 && vw_y <= 0.92 && vw_x >= 0.20 && vw_x <= 0.80)) {
            base_color = vec3<f32>(0.92, 0.92, 0.90); // White painted ornamental casing/pediment
        }
    }

    // =========================================================================
    // 3. NATURE, WATER & COASTLINE: Animated Waves, Specular Glints, Deep Absorption
    // =========================================================================
    let is_water = (in.color.b > 0.55 && in.color.r < 0.35 && in.color.g < 0.60) || abs(alpha_tag - 0.60) < 0.015;
    var surface_normal = normal;

    if (is_water) {
        // Multi-frequency animated Gerstner waves
        let wave_t = anim_time * 1.5;
        let w1_x = sin(in.world_pos.x * 0.45 + wave_t) * 0.09;
        let w1_z = cos(in.world_pos.z * 0.45 + wave_t * 0.8) * 0.09;
        let w2_x = sin(in.world_pos.x * 1.1 - in.world_pos.z * 0.9 + wave_t * 1.6) * 0.04;
        let w2_z = cos(in.world_pos.z * 1.1 + in.world_pos.x * 0.7 + wave_t * 1.4) * 0.04;

        surface_normal = normalize(normal + vec3<f32>(w1_x + w2_x, 0.0, w1_z + w2_z));

        // Water depth gradient: shallow coastal cyan to deep ocean navy
        let shore_prox = clamp((in.world_pos.y + 0.5) / 2.0, 0.0, 1.0);
        let deep_water = vec3<f32>(0.08, 0.22, 0.42);
        let shallow_water = vec3<f32>(0.12, 0.45, 0.58);
        base_color = mix(deep_water, shallow_water, shore_prox);
    }

    // Specular highlight with Fresnel reflectance (water, glass, moist surfaces)
    let half_dir = normalize(sun_dir + view_dir);
    let is_glass = abs(alpha_tag - 0.80) < 0.015;
    let shininess = select(64.0, select(128.0, 256.0, is_glass), is_water);
    let spec_base = pow(max(dot(surface_normal, half_dir), 0.0), shininess);
    let n_dot_v = max(dot(surface_normal, view_dir), 0.0);

    // Fresnel reflectance equation: R(0) = F0 = 0.04, R(90 deg) = 1.0
    let f0 = 0.04;
    let fresnel = f0 + (1.0 - f0) * pow(1.0 - n_dot_v, 5.0);

    let spec_mult = select(uniforms.material_params.x, uniforms.material_params.x * 2.5, is_water || is_glass);
    let specular = uniforms.sun_color_intensity.rgb * spec_base * fresnel * spec_mult * day_factor;

    let lit_color = base_color * (ambient + diffuse) + specular;

    // =========================================================================
    // 4. ATMOSPHERE, SKYLIGHT, DISTANCE FOG & VOLUMETRIC CLOUDS
    // =========================================================================
    let dist = length(uniforms.camera_pos.xyz - in.world_pos);
    let fog_start = uniforms.fog_params.y;
    let fog_end = uniforms.fog_params.z;
    let fog_density = uniforms.fog_params.x;

    // Smooth distance-based fog preventing close-range horizon washout
    let eff_dist = max(dist - fog_start, 0.0);
    let linear_fog = clamp(eff_dist / max(fog_end - fog_start, 1.0), 0.0, 1.0);
    let exp_fog = 1.0 - exp(-eff_dist * fog_density);
    let fog_factor = clamp(max(linear_fog, exp_fog), 0.0, 1.0);

    // Sky color palette matching solar elevation
    let day_sky = vec3<f32>(0.65, 0.76, 0.92);
    let sunset_sky = vec3<f32>(0.88, 0.54, 0.36);
    let twilight_sky = vec3<f32>(0.24, 0.22, 0.38);
    let night_sky = vec3<f32>(0.04, 0.06, 0.12);

    var sky_fog_color = day_sky;
    if (sun_elevation < 0.25 && sun_elevation >= 0.0) {
        let t = clamp((0.25 - sun_elevation) / 0.25, 0.0, 1.0);
        sky_fog_color = mix(day_sky, sunset_sky, t);
    } else if (sun_elevation < 0.0 && sun_elevation >= -0.15) {
        let t = clamp((-sun_elevation) / 0.15, 0.0, 1.0);
        sky_fog_color = mix(sunset_sky, twilight_sky, t);
    } else if (sun_elevation < -0.15) {
        let t = clamp((-sun_elevation - 0.15) / 0.25, 0.0, 1.0);
        sky_fog_color = mix(twilight_sky, night_sky, t);
    }

    // Solar disc flare and atmospheric Rayleigh halo
    let sun_align = max(dot(view_dir, sun_dir), 0.0);
    let sun_halo = pow(sun_align, 64.0) * 0.35 * day_factor;
    sky_fog_color = sky_fog_color + uniforms.sun_color_intensity.rgb * sun_halo;

    // Procedural cumulus cloud layer in sky background
    let cloud_p = in.world_pos.xz * 0.00015 + vec2<f32>(anim_time * 0.002, anim_time * 0.001);
    let cloud_noise = hash2(cloud_p * 10.0) * 0.5 + hash2(cloud_p * 25.0) * 0.3;
    let is_cloud = clamp((cloud_noise - 0.35) * 2.0, 0.0, 0.45);
    sky_fog_color = mix(sky_fog_color, vec3<f32>(0.96, 0.97, 0.99), is_cloud * day_factor);

    let fogged_color = mix(lit_color, sky_fog_color, fog_factor);

    // ACES-style Filmic Tone Mapping for cinematic color saturation & highlights
    let a_param = 2.51;
    let b_param = 0.03;
    let c_param = 2.43;
    let d_param = 0.59;
    let e_param = 0.14;
    let mapped_rgb = clamp((fogged_color * (a_param * fogged_color + b_param)) / (fogged_color * (c_param * fogged_color + d_param) + e_param), vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(mapped_rgb, in.color.a);
}
"#;

/// 2D Screen-space HUD and telemetry overlay WGSL shader.
pub const HUD_SHADER_WGSL: &str = r#"
struct HudUniforms {
    screen_size: vec2<f32>,
    padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> hud_uniforms: HudUniforms;

struct HudVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct HudVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: HudVertexInput) -> HudVertexOutput {
    var out: HudVertexOutput;
    let x_ndc = (in.position.x / hud_uniforms.screen_size.x) * 2.0 - 1.0;
    let y_ndc = 1.0 - (in.position.y / hud_uniforms.screen_size.y) * 2.0;
    out.clip_position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: HudVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;
