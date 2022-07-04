use bevy_ecs::system::{Query, ResMut};
use nalgebra::Matrix4;
use wgpu::{Adapter, Device, Instance, Queue, Surface};

use winit::{dpi::PhysicalSize, window::Window};

use crate::common_component::{Camera, MainCamera, RenderGeometry, Transform};
use crate::geometry_library::{GeometryId, GeometryLibrary};
use crate::shader_library::{ShaderId, ShaderLibrary};

use crate::data_types::Vertex;
use crate::texture_library::{TextureId, TextureLibrary};
use crate::util::BlockOn;

// Render System
pub fn render(
    mut state: ResMut<RenderState>,
    camera: Query<(&Camera, &Transform, &MainCamera)>,
    geoms: Query<(&RenderGeometry, &Transform)>,
) {
    match camera.get_single() {
        Ok((cam, cam_pos, _)) => {
            // update transform info to transform buffer on gpu
            let geoms: Vec<_> = geoms
                .iter()
                .map(
                    |(
                        RenderGeometry {
                            geom_type: _geom_type,
                        },
                        pos,
                    )| (pos.isometry.to_matrix()),
                )
                .collect();

            let view_projection: Matrix4<f32> =
                cam.projection.as_matrix() * cam_pos.isometry.inverse().to_matrix();
            state.render(view_projection, geoms);
        }
        Err(e) => log::error!("failed to access main camera entity for render call: {}", e),
    }
}

pub struct RenderState {
    _instance: Instance,
    surface: Surface,
    surface_config: wgpu::SurfaceConfiguration,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    render_pipeline: wgpu::RenderPipeline,

    _depth_stencil_texture: wgpu::Texture,
    depth_stencil_view: wgpu::TextureView,
    _depth_stencil_sampler: wgpu::Sampler,

    texture_library: TextureLibrary,

    _shader_library: ShaderLibrary,
    geometry_library: GeometryLibrary,
}

impl RenderState {
    pub fn init(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .expect("failed to find appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::PUSH_CONSTANTS,
                    limits: wgpu::Limits {
                        max_push_constant_size: std::mem::size_of::<Matrix4<f32>>() as u32,
                        ..Default::default()
                    }
                    .using_resolution(adapter.limits()), //wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .block_on()
            .expect("failed to create appropriate device");

        let shader_library = ShaderLibrary::load_all(&device);

        let fragment_shader = shader_library.get(ShaderId::FragmentShader).clone();
        let vertex_shader = shader_library.get(ShaderId::VertexShader).clone();

        let geometry_library = GeometryLibrary::load_all(&device);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_library = TextureLibrary::load_all(&device, &queue, &texture_bind_group_layout);

        let depth_stencil_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let depth_stencil_view =
            depth_stencil_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_stencil_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::all(),
                range: 0..(std::mem::size_of::<Matrix4<f32>>() as u32),
            }],
        });

        let swapchain_format = surface
            .get_preferred_format(&adapter)
            .expect("failed to get swapchain format");

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader.handle(),
                entry_point: vertex_shader.entry_point(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader.handle(),
                entry_point: fragment_shader.entry_point(),
                targets: &[swapchain_format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &surface_config);

        Self {
            _instance: instance,
            surface,
            surface_config,
            _adapter: adapter,
            device,
            queue,
            render_pipeline,

            _depth_stencil_texture: depth_stencil_texture,
            depth_stencil_view,
            _depth_stencil_sampler: depth_stencil_sampler,

            texture_library,

            _shader_library: shader_library,
            geometry_library,
        }
    }

    pub fn render(&mut self, view_projection: Matrix4<f32>, objects: Vec<Matrix4<f32>>) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => return, // Redraw is sometimes sent before resize
            Err(e) => panic!("failed to acquire next swap chain texture: {}", e),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(
                0,
                &self.texture_library.get(TextureId::CrabTexture).bind_group,
                &[],
            );

            // Draw torus
            let torus = self.geometry_library.get(GeometryId::TorusGeometry);
            for t in objects {
                // There is a matrix multiplication for each object. It may scale worse the gpu side multiplication for each.
                // This could be moved to the gpu with either a larger push constant with view_projection and t matrix, or using a bound buffer for view_projection.
                let mvp = view_projection * t;

                rpass.set_push_constants(
                    wgpu::ShaderStages::all(),
                    0,
                    bytemuck::cast_slice(&mvp.as_slice()),
                );
                rpass.set_vertex_buffer(0, torus.vertices.slice(..));
                rpass.set_index_buffer(torus.indices.slice(..), wgpu::IndexFormat::Uint16);
                rpass.draw_indexed(0..torus.index_len, 0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn resize_if_needed(&mut self, size: &PhysicalSize<u32>, window: &Window) -> () {
        if size.width > 0 && size.height > 0 {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);

            window.request_redraw();
        }
    }
}
