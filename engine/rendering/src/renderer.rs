//! Native wgpu rendering engine and persistent GPU buffer pipeline manager.

use crate::camera::FloatingOriginCamera;
use crate::hud::{HudMesh, HudVertex};
use crate::preset::{GraphicsPresetLevel, GraphicsSettings};
use crate::shaders::{HUD_SHADER_WGSL, WORLD_SHADER_WGSL};
use crate::uniforms::SceneUniforms;
use bytemuck::cast_slice;
use realearth_compiler::chunk::WorldChunk;
use realearth_compiler::mesh::{ChunkMesh, Vertex};
use realearth_ingestion::dem_grid::DemGrid;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::{RealEarthError, Result};
use std::collections::HashMap;
use std::sync::RwLock;
use wgpu::util::DeviceExt;

/// Persistent GPU buffers allocated for an individual mesh (terrain, building, road, etc.).
pub struct GpuChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub vram_bytes: usize,
}

/// Set of persistent GPU buffers allocated for a complete world chunk.
pub struct GpuChunkBuffers {
    pub terrain: Option<GpuChunkMesh>,
    pub vegetation: Option<GpuChunkMesh>,
    pub roads: Option<GpuChunkMesh>,
    pub buildings: Option<GpuChunkMesh>,
    pub water: Option<GpuChunkMesh>,
    pub total_vram_bytes: usize,
}

/// Native wgpu graphics renderer supporting both headless and surface rendering,
/// with persistent GPU buffer caching, dynamic graphics presets, and integrated 2D HUD overlays.
pub struct RealEarthRenderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub pipeline: wgpu::RenderPipeline,
    pub transparent_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub hud_pipeline: wgpu::RenderPipeline,
    pub hud_uniform_buffer: wgpu::Buffer,
    pub hud_bind_group: wgpu::BindGroup,
    pub surface_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
    pub gpu_cache: RwLock<HashMap<(i32, i32, u32), GpuChunkBuffers>>,
    pub horizon_mesh_gpu: RwLock<Option<GpuChunkMesh>>,
    pub settings: GraphicsSettings,
    pub start_time: std::time::Instant,
}

impl RealEarthRenderer {
    /// Creates a headless renderer instance for automated testing and off-screen rendering.
    pub fn new_headless(surface_format: wgpu::TextureFormat) -> Result<Self> {
        Self::new_headless_with_settings(
            surface_format,
            GraphicsSettings::from_preset(GraphicsPresetLevel::High),
        )
    }

    /// Creates a headless renderer instance with explicit graphics settings preset.
    pub fn new_headless_with_settings(
        surface_format: wgpu::TextureFormat,
        settings: GraphicsSettings,
    ) -> Result<Self> {
        pollster::block_on(Self::init_device(surface_format, settings, None))
    }

    /// Initializes wgpu device and queue from system graphics adapter, then constructs the renderer.
    async fn init_device(
        surface_format: wgpu::TextureFormat,
        settings: GraphicsSettings,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface,
                force_fallback_adapter: false,
            })
            .await
        {
            Some(a) => a,
            None => match instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: surface,
                    force_fallback_adapter: true,
                })
                .await
            {
                Some(a) => a,
                None => {
                    return Err(RealEarthError::Geospatial(
                        "Failed to acquire any suitable wgpu graphics adapter".to_string(),
                    ));
                }
            },
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("RealEarth Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| {
                RealEarthError::Geospatial(format!("Failed to create wgpu device: {}", e))
            })?;

        Ok(Self::from_device(device, queue, surface_format, settings))
    }

    /// Synchronously creates a RealEarthRenderer instance given an existing wgpu Device and Queue.
    /// This is the primary constructor for visible native window surfaces.
    pub fn from_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        settings: GraphicsSettings,
    ) -> Self {
        let world_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("World Shader"),
            source: wgpu::ShaderSource::Wgsl(WORLD_SHADER_WGSL.into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Uniforms Buffer"),
            size: std::mem::size_of::<SceneUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let depth_format = wgpu::TextureFormat::Depth32Float;

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3, // position
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3, // normal
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2, // uv
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4, // color
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("World Render Pipeline (Opaque)"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &world_shader,
                entry_point: Some("vs_main"),
                buffers: std::slice::from_ref(&vertex_layout),
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &world_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let transparent_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("World Render Pipeline (Transparent/Glass/Shadow)"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &world_shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &world_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false, // CRITICAL: NEVER write depth for blended/transparent geometry!
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // 2D Screen-space HUD Pipeline
        let hud_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("HUD Shader"),
            source: wgpu::ShaderSource::Wgsl(HUD_SHADER_WGSL.into()),
        });

        let hud_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("HUD Uniform Buffer"),
            size: 16, // vec2<f32> screen_size + vec2<f32> padding
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let hud_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("HUD Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let hud_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("HUD Bind Group"),
            layout: &hud_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: hud_uniform_buffer.as_entire_binding(),
            }],
        });

        let hud_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("HUD Pipeline Layout"),
            bind_group_layouts: &[&hud_bind_group_layout],
            push_constant_ranges: &[],
        });

        let hud_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<HudVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2, // 2D pixel position
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4, // RGBA color
                },
            ],
        };

        let hud_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("HUD Render Pipeline"),
            layout: Some(&hud_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &hud_shader,
                entry_point: Some("vs_main"),
                buffers: &[hud_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &hud_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // No depth testing for 2D overlay
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            device,
            queue,
            pipeline,
            transparent_pipeline,
            uniform_buffer,
            uniform_bind_group,
            hud_pipeline,
            hud_uniform_buffer,
            hud_bind_group,
            surface_format,
            depth_format,
            gpu_cache: RwLock::new(HashMap::new()),
            horizon_mesh_gpu: RwLock::new(None),
            settings,
            start_time: std::time::Instant::now(),
        }
    }

    /// Renders a 2D HUD and telemetry overlay on top of the rendered frame.
    pub fn render_hud(
        &self,
        hud_mesh: &HudMesh,
        color_view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) {
        if hud_mesh.is_empty() || width == 0 || height == 0 {
            return;
        }

        let hud_uniforms = [width as f32, height as f32, 0.0, 0.0];
        self.queue
            .write_buffer(&self.hud_uniform_buffer, 0, cast_slice(&hud_uniforms));

        let vertex_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("HUD Vertex Buffer"),
                contents: cast_slice(&hud_mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("HUD Index Buffer"),
                contents: cast_slice(&hud_mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("HUD Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("HUD Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.hud_pipeline);
            pass.set_bind_group(0, &self.hud_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buf.slice(..));
            pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..hud_mesh.indices.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Uploads an individual ChunkMesh to persistent GPU buffers.
    fn upload_mesh(&self, label: &str, mesh: &ChunkMesh) -> Option<GpuChunkMesh> {
        if mesh.is_empty() {
            return None;
        }

        let vertex_bytes = mesh.vertices.len() * std::mem::size_of::<Vertex>();
        let index_bytes = mesh.indices.len() * std::mem::size_of::<u32>();

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Vertex Buffer", label)),
                contents: cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Index Buffer", label)),
                contents: cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Some(GpuChunkMesh {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
            vram_bytes: vertex_bytes + index_bytes,
        })
    }

    /// Uploads an entire WorldChunk to persistent GPU VRAM buffers.
    pub fn upload_chunk_buffers(&self, chunk: &WorldChunk) -> GpuChunkBuffers {
        let tag = format!(
            "Chunk_{}_{}_L{}",
            chunk.header.chunk_x, chunk.header.chunk_y, chunk.header.lod_level
        );

        let terrain = self.upload_mesh(&format!("{}_Terrain", tag), &chunk.terrain);
        let vegetation = self.upload_mesh(&format!("{}_Vegetation", tag), &chunk.vegetation);
        let roads = self.upload_mesh(&format!("{}_Roads", tag), &chunk.roads);
        let buildings = self.upload_mesh(&format!("{}_Buildings", tag), &chunk.buildings);
        let water = self.upload_mesh(&format!("{}_Water", tag), &chunk.water);

        let mut total_vram_bytes = 0;
        if let Some(ref m) = terrain {
            total_vram_bytes += m.vram_bytes;
        }
        if let Some(ref m) = vegetation {
            total_vram_bytes += m.vram_bytes;
        }
        if let Some(ref m) = roads {
            total_vram_bytes += m.vram_bytes;
        }
        if let Some(ref m) = buildings {
            total_vram_bytes += m.vram_bytes;
        }
        if let Some(ref m) = water {
            total_vram_bytes += m.vram_bytes;
        }

        GpuChunkBuffers {
            terrain,
            vegetation,
            roads,
            buildings,
            water,
            total_vram_bytes,
        }
    }

    /// Total VRAM bytes resident in the persistent GPU chunk cache.
    pub fn gpu_memory_bytes(&self) -> usize {
        let cache = self.gpu_cache.read().unwrap();
        cache.values().map(|b| b.total_vram_bytes).sum()
    }

    /// Prunes GPU buffers for chunks that are no longer active in CPU memory.
    pub fn prune_gpu_cache(&self, active_keys: &[(i32, i32, u32)]) -> usize {
        let mut cache = self.gpu_cache.write().unwrap();
        let before = cache.len();
        cache.retain(|k, _| active_keys.contains(k));
        before - cache.len()
    }

    /// Clears all persistent GPU chunk buffer allocations from the cache.
    /// Used when transitioning between regions or cities to prevent chunk coordinate collision.
    pub fn clear_gpu_cache(&self) -> usize {
        let mut cache = self.gpu_cache.write().unwrap();
        let count = cache.len();
        cache.clear();
        *self.horizon_mesh_gpu.write().unwrap() = None;
        count
    }

    /// Uploads and assigns a coarse regional terrain horizon mesh to the GPU.
    pub fn set_horizon_mesh(&self, mesh: &ChunkMesh) {
        let gpu = self.upload_mesh("Coarse_Horizon", mesh);
        *self.horizon_mesh_gpu.write().unwrap() = gpu;
    }

    /// Updates uniform buffer data with active camera and lighting conditions.
    pub fn update_uniforms(
        &self,
        camera: &FloatingOriginCamera,
        sun_dir: [f32; 3],
        sun_intensity: f32,
    ) {
        self.update_uniforms_with_settings(camera, sun_dir, sun_intensity, &self.settings);
    }

    /// Updates uniform buffer data with active camera, sun position, and explicit graphics settings.
    pub fn update_uniforms_with_settings(
        &self,
        camera: &FloatingOriginCamera,
        sun_dir: [f32; 3],
        sun_intensity: f32,
        settings: &GraphicsSettings,
    ) {
        let view_proj = camera.view_proj_matrix();
        let rel_pos = camera.relative_pos_f32();
        let elapsed_time = self.start_time.elapsed().as_secs_f32();

        let fog_start = (settings.view_distance_meters as f32 * 0.30).max(300.0);
        let uniforms = SceneUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [rel_pos.x, rel_pos.y, rel_pos.z, 1.0],
            sun_direction: [sun_dir[0], sun_dir[1], sun_dir[2], 0.0],
            sun_color_intensity: [1.0, 0.98, 0.92, sun_intensity],
            ambient_light: [0.22, 0.24, 0.28, 1.0],
            fog_params: [
                settings.fog_density,
                fog_start,
                settings.view_distance_meters as f32,
                0.0,
            ],
            material_params: [
                settings.specular_intensity,
                if settings.road_markings_enabled {
                    1.0
                } else {
                    0.0
                },
                elapsed_time,
                1.0,
            ],
        };

        self.queue
            .write_buffer(&self.uniform_buffer, 0, cast_slice(&[uniforms]));
    }

    /// Renders a single ChunkMesh directly into a target color view and depth view.
    pub fn render_mesh(
        &self,
        mesh: &ChunkMesh,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        clear: bool,
    ) {
        if mesh.is_empty() {
            return;
        }

        // Partition triangles into opaque and transparent passes to eliminate alpha occlusion bugs
        let mut opaque_indices = Vec::with_capacity(mesh.indices.len());
        let mut transparent_indices = Vec::new();

        for tri in mesh.indices.as_chunks::<3>().0 {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;
            if i0 < mesh.vertices.len() && i1 < mesh.vertices.len() && i2 < mesh.vertices.len() {
                let a0 = mesh.vertices[i0].color[3];
                let a1 = mesh.vertices[i1].color[3];
                let a2 = mesh.vertices[i2].color[3];
                // True transparent geometries: vehicle glass, contact shadows, transparent props
                let is_transparent = a0 < 0.80 || a1 < 0.80 || a2 < 0.80;
                if is_transparent {
                    transparent_indices.extend_from_slice(tri);
                } else {
                    opaque_indices.extend_from_slice(tri);
                }
            } else {
                opaque_indices.extend_from_slice(tri);
            }
        }

        let vertex_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Vertex Buffer"),
                contents: cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let load_op = if clear {
                wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.65,
                    g: 0.76,
                    b: 0.92,
                    a: 1.0,
                })
            } else {
                wgpu::LoadOp::Load
            };

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesh Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: if clear {
                            wgpu::LoadOp::Clear(1.0)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buf.slice(..));

            // Pass 1: Opaque geometry with full depth writes
            if !opaque_indices.is_empty() {
                let opaque_buf =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Opaque Mesh Index Buffer"),
                            contents: cast_slice(&opaque_indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });
                pass.set_pipeline(&self.pipeline);
                pass.set_index_buffer(opaque_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..opaque_indices.len() as u32, 0, 0..1);
            }

            // Pass 2: Transparent geometry (glass, shadows) with depth writing disabled
            if !transparent_indices.is_empty() {
                let trans_buf = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Transparent Mesh Index Buffer"),
                        contents: cast_slice(&transparent_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });
                pass.set_pipeline(&self.transparent_pipeline);
                pass.set_index_buffer(trans_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..transparent_indices.len() as u32, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Renders active world chunks with frustum culling and PERSISTENT GPU buffers.
    /// Buffers are uploaded once upon chunk arrival and reused for all subsequent frames with zero reallocation!
    pub fn render_chunks_culled_cached(
        &self,
        chunks: &[&WorldChunk],
        camera: &FloatingOriginCamera,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) -> usize {
        let frustum = camera.frustum();

        // 1. Filter visible chunks
        let visible: Vec<&&WorldChunk> = chunks
            .iter()
            .filter(|chunk| {
                let conv = realearth_geospatial::transforms::EnuConverter::new(chunk.header.anchor);
                let enu_c = conv.wgs84_to_enu(chunk.header.bounds.center());
                let rel_center = glam::Vec3::new(
                    enu_c.east as f32,
                    (chunk.header.min_elevation + chunk.header.max_elevation) * 0.5,
                    enu_c.north as f32,
                );
                let radius = 800.0;
                let cam_pos = camera.relative_pos_f32();
                (cam_pos - rel_center).length() < radius
                    || frustum.intersects_sphere(rel_center, radius)
            })
            .collect();

        // 2. Ensure all visible chunks are resident in persistent GPU cache
        if !visible.is_empty() {
            let mut cache = self.gpu_cache.write().unwrap();
            for chunk in &visible {
                let key = (
                    chunk.header.chunk_x,
                    chunk.header.chunk_y,
                    chunk.header.lod_level,
                );
                cache
                    .entry(key)
                    .or_insert_with(|| self.upload_chunk_buffers(chunk));
            }
        }

        // 3. Render all visible chunks and distant terrain horizon in a SINGLE batched render pass!
        let cache = self.gpu_cache.read().unwrap();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Batched Render Pass Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("World Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.65,
                            g: 0.76,
                            b: 0.92,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            // Draw distant horizon terrain mesh if present (eliminating flight boundary voids!)
            if let Some(ref horizon) = *self.horizon_mesh_gpu.read().unwrap() {
                pass.set_vertex_buffer(0, horizon.vertex_buffer.slice(..));
                pass.set_index_buffer(horizon.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..horizon.index_count, 0, 0..1);
            }

            // Render submeshes in optimal depth order: terrain -> vegetation -> roads -> buildings -> water
            for chunk in &visible {
                let key = (
                    chunk.header.chunk_x,
                    chunk.header.chunk_y,
                    chunk.header.lod_level,
                );
                if let Some(gpu_bufs) = cache.get(&key) {
                    if let Some(ref m) = gpu_bufs.terrain {
                        pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                        pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..m.index_count, 0, 0..1);
                    }
                    if let Some(ref m) = gpu_bufs.vegetation {
                        pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                        pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..m.index_count, 0, 0..1);
                    }
                    if let Some(ref m) = gpu_bufs.roads {
                        pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                        pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..m.index_count, 0, 0..1);
                    }
                    if let Some(ref m) = gpu_bufs.buildings {
                        pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                        pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..m.index_count, 0, 0..1);
                    }
                    if let Some(ref m) = gpu_bufs.water {
                        pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                        pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..m.index_count, 0, 0..1);
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        visible.len()
    }

    /// Renders all active world chunks into the target view (legacy compatibility).
    pub fn render_chunks(
        &self,
        chunks: &[&WorldChunk],
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        let mut first = true;
        for chunk in chunks {
            self.render_mesh(&chunk.terrain, color_view, depth_view, first);
            first = false;
            self.render_mesh(&chunk.vegetation, color_view, depth_view, false);
            self.render_mesh(&chunk.roads, color_view, depth_view, false);
            self.render_mesh(&chunk.buildings, color_view, depth_view, false);
            self.render_mesh(&chunk.water, color_view, depth_view, false);
        }
    }

    /// Legacy frustum-culled rendering allocating temporary buffers.
    pub fn render_chunks_culled(
        &self,
        chunks: &[&WorldChunk],
        camera: &FloatingOriginCamera,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) -> usize {
        self.render_chunks_culled_cached(chunks, camera, color_view, depth_view)
    }
}

/// Generates a coarse regional terrain horizon apron mesh around the regional anchor.
/// Spans from inner_radius (e.g. 1,500m) to outer_radius (e.g. 50,000m) around the regional anchor.
/// Conforms to real elevations sampled from the regional DemGrid, smoothly tapering to base MSL.
pub fn generate_coarse_horizon_mesh(
    anchor: Wgs84Coord,
    dem: &DemGrid,
    inner_radius_m: f32,
    outer_radius_m: f32,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let num_angles = 24;
    let num_rings = 8;

    let conv = realearth_geospatial::transforms::EnuConverter::new(anchor);

    // Build concentric rings of vertices from inner_radius_m to outer_radius_m
    let mut ring_radii = Vec::with_capacity(num_rings);
    for r in 0..num_rings {
        let t = r as f32 / (num_rings - 1) as f32;
        // Non-linear geometric spacing provides higher detail near city core and expands outwards
        let radius = inner_radius_m + (outer_radius_m - inner_radius_m) * t.powf(1.6);
        ring_radii.push(radius);
    }

    for (ring_idx, &radius) in ring_radii.iter().enumerate() {
        let t_outer = ring_idx as f32 / (num_rings - 1) as f32;
        for a in 0..num_angles {
            let theta = (a as f32 / num_angles as f32) * std::f32::consts::TAU;
            let x = radius * theta.sin();
            let z = radius * theta.cos();

            // Convert local ENU (x, 0, z) back to Wgs84 to sample regional DEM
            let enu = realearth_shared::coordinates::EnuCoord::new(x as f64, z as f64, 0.0);
            let wgs = conv.enu_to_wgs84(enu);

            let elev = if dem.bounds.contains(&wgs) {
                dem.sample_bilinear(&wgs)
            } else {
                // Taper smoothly towards anchor altitude or MSL sea level at distant boundary
                let edge_dist = ((x * x + z * z).sqrt() - inner_radius_m).max(0.0);
                let taper = (1.0 - (edge_dist / (outer_radius_m - inner_radius_m))).clamp(0.0, 1.0);
                (anchor.alt as f32 * taper).max(0.0)
            };

            // Color based on elevation and distance
            let (color, normal) = if elev <= 0.8 {
                // Ocean / Bay water
                ([0.18, 0.32, 0.48, 1.0], [0.0, 1.0, 0.0])
            } else if elev > 300.0 {
                // High ground / hills / mountains
                ([0.45, 0.44, 0.40, 1.0], [0.0, 1.0, 0.0])
            } else {
                // Valley / rolling vegetation
                ([0.34, 0.46, 0.28, 1.0], [0.0, 1.0, 0.0])
            };

            mesh.vertices.push(Vertex {
                position: [x, elev, z],
                normal,
                color,
                uv: [t_outer, a as f32 / num_angles as f32],
            });
        }
    }

    // Connect rings into triangle quads
    for r in 0..(num_rings - 1) {
        let r0_base = (r * num_angles) as u32;
        let r1_base = ((r + 1) * num_angles) as u32;

        for a in 0..num_angles {
            let next_a = ((a + 1) % num_angles) as u32;
            let a_curr = a as u32;

            let v0 = r0_base + a_curr;
            let v1 = r0_base + next_a;
            let v2 = r1_base + next_a;
            let v3 = r1_base + a_curr;

            // Quad: (v0, v1, v2) and (v0, v2, v3)
            mesh.indices.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
        }
    }

    mesh
}
