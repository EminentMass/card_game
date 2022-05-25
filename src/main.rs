mod renderer;
mod shader_library;

use std::path::PathBuf;

use renderer::Renderer;
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

    let mut renderer = Renderer::init(&window).await;

    let egui_ctx = egui::Context::default();
    let mut egui_winit_state = egui_winit::State::new(2000, &window);

    // setup egui rendering
    //let mut egui_rp = egui_wgpu::renderer::RenderPass::new(&device, swapchain_format, 1);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, window_id } => {
                egui_winit_state.on_event(&egui_ctx, &event);
                match event {
                    WindowEvent::Resized(size) => {
                        renderer.resize_if_needed(&size, &window);
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

                renderer.render();
            }
            _ => (),
        }
    });
}
