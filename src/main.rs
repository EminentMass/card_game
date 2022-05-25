mod renderer;
mod shader_library;

use std::path::PathBuf;

use shader_library::ShaderLibraryBuilder;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init().unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

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

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    surface.configure(&device, &config);

    let egui_ctx = egui::Context::default();
    let mut egui_winit_state = egui_winit::State::new(2000, &window);

    // setup egui rendering
    let mut egui_rp = egui_wgpu::renderer::RenderPass::new(&device, swapchain_format, 1);

    event_loop.run(move |event, _, control_flow| {
        let _ = (
            &instance,
            &adapter,
            &vertex_shader,
            &fragment_shader,
            &pipeline_layout,
        );

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, window_id } => {
                egui_winit_state.on_event(&egui_ctx, &event);
                match event {
                    WindowEvent::Resized(size) => {
                        if size.width > 0 && size.height > 0 {
                            config.width = size.width;
                            config.height = size.height;
                            surface.configure(&device, &config);

                            window.request_redraw();
                        }
                    }
                    WindowEvent::CloseRequested => {
                        if window_id == window.id() {
                            *control_flow = ControlFlow::Exit
                        }
                    }
                    _ => (),
                }
            }

            Event::RedrawRequested(_) => {
                let raw_input = egui_winit_state.take_egui_input(&window); //egui::RawInput::default();

                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => return, // Redraw is sometimes sent before resize
                    Err(e) => panic!("failed to acquire next swap chain texture: {}", e),
                };
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let output = egui_ctx.run(raw_input, |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::containers::Frame::none())
                        .show(&ctx, |ui| {
                            ui.label("Hello world!");
                            if ui.button("Click me").clicked() {
                                // take some action here
                            }
                        });
                });

                let clipped_primitives = egui_ctx.tessellate(output.shapes);

                egui_winit_state.handle_platform_output(&window, &egui_ctx, output.platform_output);

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

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
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);

                    for (id, delta) in &output.textures_delta.set {
                        egui_rp.update_texture(&device, &queue, *id, delta);
                    }

                    for id in &output.textures_delta.free {
                        egui_rp.free_texture(id);
                    }

                    let screen_descriptor = &egui_wgpu::renderer::ScreenDescriptor {
                        size_in_pixels: [config.width, config.height],
                        pixels_per_point: 1.0,
                    };

                    egui_rp.update_buffers(
                        &device,
                        &queue,
                        &clipped_primitives,
                        &screen_descriptor,
                    );

                    egui_rp.execute_with_renderpass(
                        &mut rpass,
                        &clipped_primitives,
                        &screen_descriptor,
                    );

                    /*
                    egui_rp.execute(
                        &mut encoder,
                        &view,
                        &clipped_primitives,
                        &egui_wgpu::renderer::ScreenDescriptor {
                            size_in_pixels: [500, 500],
                            pixels_per_point: 1.0,
                        },
                        Some(wgpu::Color {
                            r: 0.0,
                            g: 0.73,
                            b: 0.0,
                            a: 1.0,
                        }),
                    );
                    */
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => (),
        }
    });
}
