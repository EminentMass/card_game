use std::path::PathBuf;

use bevy_ecs::system::{Query, Res, ResMut};
use nalgebra::{Matrix4, Vector3};
use wgpu::{util::DeviceExt, Adapter, Device, Instance, Queue, Surface};

use winit::{dpi::PhysicalSize, window::Window};

use crate::common_component::{Camera, Transform};
use crate::shader_library::{ShaderLibrary, ShaderLibraryBuilder};

use crate::data_types::Vertex;
use crate::time::TimeResource;

macro_rules! v {
    ($a:expr, $b:expr, $c:expr) => {
        Vertex {
            position: [$a, $b, $c, 1.0],
            normal: [0.0, 0.0, 0.0, 0.0],
            texture: [0.0, 0.0],
        }
    };
}

// origin is center of object. base is under the y plane with the point sticking up
const TETRAHEDRON_VERTICES: [Vertex; 4] = [
    v![0.0, -0.57735, -1.15470], // base
    v![-1.0, -0.57735, 0.57735],
    v![1.0, -0.57735, 0.57735],
    v![0.0, 1.15470, 0.0], // point sticking up along y
];
const TETRAHEDRON_INDICES: [u16; 12] = [0, 1, 2, 0, 3, 1, 3, 0, 2, 2, 1, 3];
//const TETRAHEDRON_INDICES: [u16; 12] = [2, 1, 0, 1, 3, 0, 2, 0, 3, 3, 1, 2];

// Render System
pub fn render(
    mut state: ResMut<RenderState>,
    time: Res<TimeResource>,
    cameras: Query<(&Camera, &Transform)>,
) {
    match cameras.get_single() {
        Ok((cam, pos)) => {
            let r = time.time.as_secs_f32() * 0.5;

            let t = Vector3::z() * -5.0;
            let model = Matrix4::<f32>::new_translation(&t)
                * Matrix4::new_rotation(Vector3::y() * r)
                * Matrix4::new_scaling(2.0);

            let mvp: Matrix4<f32> =
                cam.perspective.as_matrix() * pos.isometry.inverse().to_matrix() * model;
            state.render(mvp);
        }
        Err(e) => log::error!("failed to access camera entity for render call: {}", e),
    }
}

pub struct RenderState {
    instance: Instance,
    surface: Surface,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    shader_library: ShaderLibrary,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    depth_stencil_texture: wgpu::Texture,
    depth_stencil_view: wgpu::TextureView,
    depth_stencil_sampler: wgpu::Sampler,
}

impl RenderState {
    pub async fn init(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
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
            .await
            .expect("failed to create appropriate device");

        let mut builder = ShaderLibraryBuilder::new();
        let vertex_shader_id = builder.add(&PathBuf::from("shader/vertex_shader.vs"));
        let fragment_shader_id = builder.add(&PathBuf::from("shader/fragment_shader.fs"));
        let shader_library = builder.build(&device);

        let fragment_shader = shader_library.get(fragment_shader_id).clone();
        let vertex_shader = shader_library.get(vertex_shader_id).clone();

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
            bind_group_layouts: &[],
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
            primitive: wgpu::PrimitiveState::default(),
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&TETRAHEDRON_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&TETRAHEDRON_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
            shader_library,
            render_pipeline,
            vertex_buffer,
            index_buffer,

            depth_stencil_texture,
            depth_stencil_view,
            depth_stencil_sampler,
        }
    }

    pub fn render(&mut self, mvp: Matrix4<f32>) {
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
            rpass.set_push_constants(
                wgpu::ShaderStages::all(),
                0,
                bytemuck::cast_slice(&mvp.as_slice()),
            );
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..12, 0, 0..1);
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
