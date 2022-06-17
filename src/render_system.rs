use std::ops::Range;
use std::path::PathBuf;

use bevy_ecs::system::{Query, ResMut};
use nalgebra::Matrix4;
use wgpu::{util::DeviceExt, Adapter, Device, Instance, Queue, Surface};

use winit::{dpi::PhysicalSize, window::Window};

use crate::common_component::{Camera, GeometryType, MainCamera, RenderGeometry, Transform};
use crate::geometry_library::GeometryLibrary;
use crate::shader_library::{ShaderLibrary, ShaderLibraryBuilder};

use crate::data_types::Vertex;
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
            let mut geoms: Vec<_> = geoms
                .iter()
                .map(|(RenderGeometry { geom_type }, pos)| (*geom_type, pos.isometry.to_matrix()))
                .collect();

            geoms.sort_by(|(g1, _), (g2, _)| g1.cmp(g2));

            let (plane_count, cube_count, tetrahedron_count): (u32, u32, u32) =
                geoms.iter().fold((0, 0, 0), |a, (g, _)| match g {
                    GeometryType::Plane => (a.0 + 1, a.1, a.2),
                    GeometryType::Cube => (a.0, a.1 + 1, a.2),
                    GeometryType::Tetrahedron => (a.0, a.1, a.2 + 1),
                });

            let (plane_range, cube_range, tetrahedron_range) = {
                let mat_size = std::mem::size_of::<Matrix4<f32>>() as u32;
                let plane_offset = plane_count * mat_size;
                let cube_offset = plane_offset + cube_count * mat_size;
                let tetrahedron_offset = cube_offset + tetrahedron_count * mat_size;

                (
                    0..plane_offset as u64,
                    plane_offset as u64..cube_offset as u64,
                    cube_offset as u64..tetrahedron_offset as u64,
                )
            };

            let data: Vec<_> = geoms.iter().map(|(_, m)| *m).collect();

            state.write_to_instance_buffer(&data);

            let view_projection: Matrix4<f32> =
                cam.projection.as_matrix() * cam_pos.isometry.inverse().to_matrix();
            state.render(
                view_projection,
                plane_count,
                cube_count,
                tetrahedron_count,
                plane_range,
                cube_range,
                tetrahedron_range,
            );
        }
        Err(e) => log::error!("failed to access main camera entity for render call: {}", e),
    }
}

pub struct RenderState {
    instance: Instance,
    surface: Surface,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_buffer_len: u64,

    depth_stencil_texture: wgpu::Texture,
    depth_stencil_view: wgpu::TextureView,
    depth_stencil_sampler: wgpu::Sampler,

    shader_library: ShaderLibrary,
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

        let mut builder = ShaderLibraryBuilder::new();
        let vertex_shader_id = builder.add(&PathBuf::from("shader/vertex_shader.vsspirv"));
        let fragment_shader_id = builder.add(&PathBuf::from("shader/fragment_shader.fsspirv"));
        let shader_library = builder.build(&device);

        let fragment_shader = shader_library.get(fragment_shader_id).clone();
        let vertex_shader = shader_library.get(vertex_shader_id).clone();

        let geometry_library = GeometryLibrary::new();

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
                buffers: &[Vertex::desc(), crate::data_types::Instance::desc()],
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(geometry_library.geometry_vertex_data()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(geometry_library.geometry_index_data()),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer_len = 5;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Matrix4<f32>>() as u64 * instance_buffer_len as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instance_buffer_len,

            depth_stencil_texture,
            depth_stencil_view,
            depth_stencil_sampler,

            shader_library,
            geometry_library,
        }
    }

    pub fn write_to_instance_buffer(&mut self, data: &[Matrix4<f32>]) {
        let bytes = bytemuck::cast_slice(&data);
        if data.len() > self.instance_buffer_len as usize {
            // double
            let new_len: u64 = {
                let mut len = self.instance_buffer_len * 2;
                while data.len() > len as usize {
                    len *= 2;
                }
                len as u64
            };
            self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<Matrix4<f32>>() as u64 * new_len,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });

            self.instance_buffer.slice(..).get_mapped_range_mut()[0..bytes.len()]
                .copy_from_slice(bytes);
            self.instance_buffer.unmap();

            self.instance_buffer_len = new_len;
        } else {
            self.queue
                .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&data));
        }
    }

    pub fn render(
        &mut self,
        mvp: Matrix4<f32>,
        plane_count: u32,
        cube_count: u32,
        tetrahedron_count: u32,
        plane_range: Range<u64>,
        cube_range: Range<u64>,
        tetrahedron_range: Range<u64>,
    ) {
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

            // Draw planes
            rpass.set_vertex_buffer(
                0,
                self.vertex_buffer
                    .slice(GeometryLibrary::PLANE_VERTEX_OFFSET..),
            );
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(plane_range));
            rpass.set_index_buffer(
                self.index_buffer
                    .slice(GeometryLibrary::PLANE_INDEX_OFFSET..),
                wgpu::IndexFormat::Uint16,
            );

            rpass.draw_indexed(0..GeometryLibrary::PLANE_INDEX_COUNT, 0, 0..plane_count);

            // Draw Cubes
            rpass.set_vertex_buffer(
                0,
                self.vertex_buffer
                    .slice(GeometryLibrary::CUBE_VERTEX_OFFSET..),
            );
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(cube_range));
            rpass.set_index_buffer(
                self.index_buffer
                    .slice(GeometryLibrary::CUBE_INDEX_OFFSET..),
                wgpu::IndexFormat::Uint16,
            );

            rpass.draw_indexed(0..GeometryLibrary::CUBE_INDEX_COUNT, 0, 0..cube_count);

            // Draw tetrahedrons
            rpass.set_vertex_buffer(
                0,
                self.vertex_buffer
                    .slice(GeometryLibrary::TETRAHEDRON_VERTEX_OFFSET..),
            );
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(tetrahedron_range));
            rpass.set_index_buffer(
                self.index_buffer
                    .slice(GeometryLibrary::TETRAHEDRON_INDEX_OFFSET..),
                wgpu::IndexFormat::Uint16,
            );

            rpass.draw_indexed(
                0..GeometryLibrary::TETRAHEDRON_INDEX_COUNT,
                0,
                0..tetrahedron_count,
            );
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
