use std::path::PathBuf;

use wgpu::{Adapter, Device, Instance, Queue, Surface};

use winit::{dpi::PhysicalSize, window::Window};

use crate::shader_library::{ShaderLibrary, ShaderLibraryBuilder};

pub struct Renderer {
    instance: Instance,
    surface: Surface,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    shader_library: ShaderLibrary,
    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
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
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
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
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader.handle(),
                entry_point: fragment_shader.entry_point(),
                targets: &[swapchain_format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
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
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
            shader_library,
            render_pipeline,
        }
    }

    pub fn render(&mut self) {
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
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
