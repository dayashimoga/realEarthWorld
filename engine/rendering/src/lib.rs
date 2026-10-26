//! Native wgpu rendering engine and floating-origin camera system for Real Earth.

pub mod camera;
pub mod city_interactions_mesh;
pub mod culling;
pub mod gltf_asset;
pub mod hud;
pub mod preset;
pub mod renderer;
pub mod shaders;
pub mod typography;
pub mod uniforms;
pub mod vehicle_mesh;
pub mod vehicle_models;

pub use camera::FloatingOriginCamera;
pub use city_interactions_mesh::{
    generate_bus_stop_shelter_mesh, generate_detailed_tree_mesh, generate_marine_ferry_dock_mesh,
    generate_pedestrian_mesh, generate_rail_station_platform_mesh, generate_road_sign_mesh,
    generate_runway_papi_lights_mesh, generate_streetlight_mesh, generate_traffic_signal_mesh,
    DetailedTreeType, RoadSignType, TrafficSignalVisualState,
};
pub use culling::{Frustum, FrustumPlane};
pub use gltf_asset::{
    GltfAssetBuilder, GltfAssetError, GltfParser, LoadedVehicleModel, VehicleAlphaMode,
    VehicleAssetRegistry, VehicleCameraAnchors, VehicleLodLevel, VehiclePbrMaterial,
    VehicleSubmesh, VehicleSubmeshType,
};
pub use hud::{
    CatalogHit, CatalogLayout, DebriefHit, DebriefLayout, GarageHit, GarageLayout, HudMesh,
    HudVertex, InGameHudHit, InGameHudLayout, JobBrowserHit, JobBrowserLayout, MultiplayerHit,
    MultiplayerLayout, PauseMenuHit, PauseMenuLayout, RailRouteHit, RailRouteLayout, SettingsHit,
    SettingsLayout, StartScreenHit, StartScreenLayout, UiButtonVisualState, UiRect,
    GARAGE_PAINT_PALETTE,
};
pub use preset::{GraphicsPresetLevel, GraphicsSettings};
pub use renderer::{
    generate_coarse_horizon_mesh, GpuChunkBuffers, GpuChunkMesh, RealEarthRenderer,
};
pub use shaders::{HUD_SHADER_WGSL, WORLD_SHADER_WGSL};
pub use typography::{
    measure_proportional_text, render_proportional_text, FontSize, FontWeight, VectorGlyph,
};
pub use uniforms::SceneUniforms;
pub use vehicle_mesh::{
    generate_fallback_development_mesh, generate_vehicle_mesh, VehicleMeshParams, VehicleVisualType,
};
pub use vehicle_models::{
    build_apex_gt_sports_coupe, build_cityline_transit_bus, build_executive_sedan,
    build_hypersport_rr1000, build_metro_yellow_cab, build_overland_explorer_suv,
    build_transcontinental_semi, export_all_certified_models_to_glb_directory,
    get_certified_vehicle_model, AutomotivePalette,
};

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_compiler::chunk::{ChunkHeader, WorldChunk, CHUNK_MAGIC, CHUNK_VERSION};
    use realearth_compiler::mesh::{ChunkMesh, Vertex};
    use realearth_shared::bounds::GeoBounds;
    use realearth_shared::coordinates::{EcefCoord, Wgs84Coord};

    #[test]
    fn test_floating_origin_camera_calculations() {
        let vidhana = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let mut camera = FloatingOriginCamera::new(vidhana, 16.0 / 9.0);

        let vp = camera.view_proj_matrix();
        let cols = vp.to_cols_array();
        for val in cols {
            assert!(!val.is_nan() && !val.is_infinite());
        }

        // Test rebasing
        let initial_anchor = camera.anchor_ecef;
        // Move camera 6,000 meters away
        camera.pos_ecef = EcefCoord::new(
            initial_anchor.x + 6000.0,
            initial_anchor.y,
            initial_anchor.z,
        );
        camera.maybe_rebase_origin(5000.0);

        assert_ne!(camera.anchor_ecef, initial_anchor);
        assert_eq!(camera.anchor_ecef, camera.pos_ecef);
    }

    #[test]
    fn test_wgpu_headless_offscreen_render_and_cached_buffers() {
        let renderer = match RealEarthRenderer::new_headless(wgpu::TextureFormat::Rgba8UnormSrgb) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "Skipping hardware rendering test in headless runner without GPU: {}",
                    e
                );
                return;
            }
        };

        let width = 256;
        let height = 256;

        let color_tex = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Color Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let color_view = color_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_tex = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut terrain = ChunkMesh::default();
        terrain.vertices.push(Vertex::new(
            [0.0, 10.0, -20.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0],
            [0.3, 0.6, 0.2, 1.0],
        ));
        terrain.vertices.push(Vertex::new(
            [-10.0, -10.0, -20.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0],
            [0.3, 0.6, 0.2, 1.0],
        ));
        terrain.vertices.push(Vertex::new(
            [10.0, -10.0, -20.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0],
            [0.3, 0.6, 0.2, 1.0],
        ));
        terrain.indices.extend_from_slice(&[0, 1, 2]);

        let vidhana = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let camera = FloatingOriginCamera::new(vidhana, 1.0);
        renderer.update_uniforms(&camera, [0.0, 1.0, 0.0], 1.0);

        let chunk = WorldChunk {
            header: ChunkHeader {
                magic: *CHUNK_MAGIC,
                version: CHUNK_VERSION,
                chunk_x: 0,
                chunk_y: 0,
                lod_level: 0,
                bounds: GeoBounds::new(12.97, 12.98, 77.59, 77.60),
                anchor: vidhana,
                min_elevation: 0.0,
                max_elevation: 50.0,
            },
            terrain,
            roads: ChunkMesh::default(),
            buildings: ChunkMesh::default(),
            water: ChunkMesh::default(),
            vegetation: ChunkMesh::default(),
            building_metadata: Vec::new(),
        };

        // Render with persistent GPU buffer cache
        let rendered =
            renderer.render_chunks_culled_cached(&[&chunk], &camera, &color_view, &depth_view);
        assert_eq!(rendered, 1);
        assert!(renderer.gpu_memory_bytes() > 0);

        // Second render frame: reuses persistent GPU cache
        let rendered_frame2 =
            renderer.render_chunks_culled_cached(&[&chunk], &camera, &color_view, &depth_view);
        assert_eq!(rendered_frame2, 1);
    }
}
