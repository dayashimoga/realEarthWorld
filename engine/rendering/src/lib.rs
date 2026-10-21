//! Native wgpu rendering engine and floating-origin camera system for Real Earth.

pub mod camera;
pub mod renderer;
pub mod shaders;
pub mod uniforms;

pub use camera::FloatingOriginCamera;
pub use renderer::RealEarthRenderer;
pub use shaders::WORLD_SHADER_WGSL;
pub use uniforms::SceneUniforms;

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_compiler::mesh::{ChunkMesh, Vertex};
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
    fn test_wgpu_headless_offscreen_render() {
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

        let mut mesh = ChunkMesh::default();
        mesh.vertices.push(Vertex::new(
            [0.0, 10.0, -20.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0],
            [1.0, 0.0, 0.0, 1.0],
        ));
        mesh.vertices.push(Vertex::new(
            [-10.0, -10.0, -20.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
        ));
        mesh.vertices.push(Vertex::new(
            [10.0, -10.0, -20.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ));
        mesh.indices.extend_from_slice(&[0, 1, 2]);

        let vidhana = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let camera = FloatingOriginCamera::new(vidhana, 1.0);
        renderer.update_uniforms(&camera, [0.0, 1.0, 0.0], 1.0);

        // Render mesh to offscreen texture
        renderer.render_mesh(&mesh, &color_view, &depth_view, true);
    }
}
