use bevy_ecs::system::{Query, ResMut};
use nalgebra::{Matrix4, Vector4};
use wgpu::{Adapter, Device, Instance, Queue, Surface};

use winit::{dpi::PhysicalSize, window::Window};

use crate::common_component::{
    Camera, GlobalLight, MainCamera, PointLight, RenderGeometry, SpotLight, Texture, Transform,
};
use crate::geometry_library::{GeometryId, GeometryLibrary};
use crate::shader_library::{ShaderId, ShaderLibrary};

use crate::data_types::{
    self, GlobalLight as GlobalLightData, PointLight as PointLightData, SpotLight as SpotLightData,
    Vertex,
};
use crate::texture_library::{TextureId, TextureLibrary};
use crate::util::BlockOn;

const PUSH_CONSTANT_SIZE: u32 = std::mem::size_of::<Matrix4<f32>>() as u32;

const MAX_GLOBAL_LIGHTS: usize = 8;
const MAX_POINT_LIGHTS: usize = 8;
const MAX_SPOT_LIGHTS: usize = 8;

// Render System
pub fn render(
    mut state: ResMut<RenderState>,
    camera: Query<(&Camera, &Transform, &MainCamera)>,
    objects: Query<(&RenderGeometry, &Transform, Option<&Texture>)>,
    global_lights: Query<&GlobalLight>,
    point_lights: Query<(&PointLight, &Transform)>,
    spot_lights: Query<(&SpotLight, &Transform)>,
) {
    match camera.get_single() {
        Ok((cam, cam_pos, _)) => {
            // grab transformation matrices for push constants
            let mut objects = objects
                .iter()
                .map(|(RenderGeometry { geom_type }, pos, texture)| {
                    let t_id = match texture {
                        Some(s) => Some(s.texture_id),
                        None => None,
                    };

                    (*geom_type, pos.isometry.to_matrix(), t_id)
                });

            let view_projection: Matrix4<f32> =
                cam.projection.as_matrix() * cam_pos.isometry.inverse().to_matrix();

            let p = cam_pos.isometry.translation.vector;

            let cam = data_types::Camera {
                view_projection,
                position: Vector4::new(p.x, p.y, p.z, 1.0),
            };

            let global_lights: Box<[GlobalLightData]> = global_lights
                .iter()
                .map(|tuple| tuple.into())
                .take(MAX_GLOBAL_LIGHTS)
                .collect();

            let point_lights: Box<[PointLightData]> = point_lights
                .iter()
                .map(|tuple| tuple.into())
                .take(MAX_POINT_LIGHTS)
                .collect();

            let spot_lights: Box<[SpotLightData]> = spot_lights
                .iter()
                .map(|tuple| tuple.into())
                .take(MAX_POINT_LIGHTS)
                .collect();

            let mut point_light_data = [PointLightData::default(); MAX_POINT_LIGHTS];

            assert!(point_lights.len() <= MAX_POINT_LIGHTS); // This assert probably isn't needed
            unsafe {
                std::ptr::copy_nonoverlapping(
                    point_lights.as_ptr(),
                    point_light_data.as_mut_ptr(),
                    point_lights.len(),
                )
            }

            state
                .queue
                .write_buffer(&state.camera_buffer, 0, bytemuck::cast_slice(&[cam]));
            state.queue.write_buffer(
                &state.light_buffer,
                0,
                bytemuck::cast_slice(&point_light_data),
            );

            state.render(&mut objects);
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

    /*
    light_assignment_pipeline: wgpu::ComputePipeline,
    light_assignment_bind_group: wgpu::BindGroup,
     */
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,

    light_bind_group: wgpu::BindGroup,
    light_buffer: wgpu::Buffer,

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
                        max_push_constant_size: PUSH_CONSTANT_SIZE,
                        ..Default::default()
                    }
                    .using_resolution(adapter.limits()), //wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .block_on()
            .expect("failed to create appropriate device");

        let shader_library = ShaderLibrary::load_all(&device);

        //let light_assignment_shader = shader_library.get(ShaderId::LightAssignment).clone();
        let fragment_shader = shader_library.get(ShaderId::FragmentShader).clone();
        let vertex_shader = shader_library.get(ShaderId::VertexShader).clone();

        let geometry_library = GeometryLibrary::load_all(&device);

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: data_types::Camera::BINDING_SIZE,
                    },
                    count: None,
                }],
            });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: data_types::Camera::BINDING_SIZE.unwrap().into(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_buffer,
                    offset: 0,
                    size: data_types::Camera::BINDING_SIZE,
                }),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
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

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let global_light_size = (std::mem::size_of::<GlobalLightData>() * 8) as u64;
        let point_light_size = (std::mem::size_of::<PointLightData>() * 8) as u64;
        let spot_light_size = (std::mem::size_of::<SpotLightData>() * 8) as u64;

        let global_light_offset = 0;
        let point_light_offset = global_light_size;
        let spot_light_offset = point_light_offset + point_light_size;

        let light_buffer_size: u64 = global_light_size + point_light_size + spot_light_size;

        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size: light_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &light_buffer,
                        offset: global_light_offset,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &light_buffer,
                        offset: point_light_offset,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &light_buffer,
                        offset: spot_light_offset,
                        size: None,
                    }),
                },
            ],
        });

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

        /*
        let light_assignment_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Assignment Bind Group Layout"),
                entries: &[],
            });

        let light_assignment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Assignment Bind Group"),
            layout: &light_assignment_bind_group_layout,
            entries: &[],
        });

        let light_assignment_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Assignment Pipeline Layout"),
                bind_group_layouts: &[&global_light_bind_group_layout],
                push_constant_ranges: &[],
            });

        let light_assignment_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Light Assignment Pipeline"),
                layout: Some(&light_assignment_pipeline_layout),
                module: light_assignment_shader.handle(),
                entry_point: light_assignment_shader.entry_point(),
            });
        */

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &texture_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::all(),
                    range: 0..PUSH_CONSTANT_SIZE,
                }],
            });

        let swapchain_format = surface.get_supported_formats(&adapter)[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader.handle(),
                entry_point: vertex_shader.entry_point(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader.handle(),
                entry_point: fragment_shader.entry_point(),
                targets: &[Some(swapchain_format.into())],
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

            /*
            light_assignment_pipeline,
            light_assignment_bind_group,
             */
            camera_bind_group,
            camera_buffer,

            light_bind_group,
            light_buffer,

            _depth_stencil_texture: depth_stencil_texture,
            depth_stencil_view,
            _depth_stencil_sampler: depth_stencil_sampler,

            texture_library,

            _shader_library: shader_library,
            geometry_library,
        }
    }

    pub fn render(
        &mut self,
        objects: &mut dyn Iterator<Item = (GeometryId, Matrix4<f32>, Option<TextureId>)>,
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

        /* This would be the depth pre pass but as of now it is not implemented
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Depth Pre Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        }
        */

        /*
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Light Assignment Pass"),
            });

            cpass.set_pipeline(&self.light_assignment_pipeline);
            cpass.set_bind_group(0, &self.light_assignment_bind_group, &[]);
            cpass.dispatch_workgroups(8, 8, 8);
        }
         */

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
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
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_bind_group(2, &self.light_bind_group, &[]);

            // Draw geometry
            for (id, model_t, tex) in objects {
                rpass.set_bind_group(1, &self.texture_library.get(tex).bind_group, &[]);

                let mesh = self.geometry_library.get(id);
                rpass.set_push_constants(
                    wgpu::ShaderStages::all(),
                    0,
                    bytemuck::cast_slice(&[model_t]),
                );
                rpass.set_vertex_buffer(0, mesh.vertices.slice(..));
                rpass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint16);
                rpass.draw_indexed(0..mesh.index_len, 0, 0..1);
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
