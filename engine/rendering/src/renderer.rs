//! Native wgpu rendering engine and persistent GPU buffer pipeline manager.

use crate::camera::FloatingOriginCamera;
use crate::preset::{GraphicsPresetLevel, GraphicsSettings};
use crate::shaders::WORLD_SHADER_WGSL;
use crate::uniforms::SceneUniforms;
use bytemuck::cast_slice;
use realearth_compiler::chunk::WorldChunk;
use realearth_compiler::mesh::{ChunkMesh, Vertex};
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
/// with persistent GPU buffer caching and dynamic graphics quality presets.
pub struct RealEarthRenderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub surface_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
    pub gpu_cache: RwLock<HashMap<(i32, i32, u32), GpuChunkBuffers>>,
    pub settings: GraphicsSettings,
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

    /// Initializes wgpu device, queue, shaders, and render pipeline.
    async fn init_device(
        surface_format: wgpu::TextureFormat,
        settings: GraphicsSettings,
        _surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap_or_else(|| {
                // Fallback to secondary/software adapter
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: None,
                    force_fallback_adapter: true,
                }))
                .expect("Failed to acquire any suitable wgpu graphics adapter")
            });

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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
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
            label: Some("World Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
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
                cull_mode: None, // Render both sides of walls/ribbons
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

        Ok(Self {
            device,
            queue,
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            surface_format,
            depth_format,
            gpu_cache: RwLock::new(HashMap::new()),
            settings,
        })
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

        let uniforms = SceneUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [rel_pos.x, rel_pos.y, rel_pos.z, 1.0],
            sun_direction: [sun_dir[0], sun_dir[1], sun_dir[2], 0.0],
            sun_color_intensity: [1.0, 0.98, 0.92, sun_intensity],
            ambient_light: [0.22, 0.24, 0.28, 1.0],
            fog_params: [
                settings.fog_density,
                100.0,
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
                0.0,
                0.0,
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

        let vertex_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Vertex Buffer"),
                contents: cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Index Buffer"),
                contents: cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
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

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buf.slice(..));
            pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
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
                let rel_center = glam::Vec3::new(
                    0.0,
                    (chunk.header.min_elevation + chunk.header.max_elevation) * 0.5,
                    0.0,
                );
                let radius = 750.0;
                frustum.intersects_sphere(rel_center, radius)
            })
            .collect();

        if visible.is_empty() {
            return 0;
        }

        // 2. Ensure all visible chunks are resident in persistent GPU cache
        {
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

        // 3. Render all visible chunks in a SINGLE batched render pass!
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
