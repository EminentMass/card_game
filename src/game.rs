use std::time::Duration;

use bevy_ecs::{
    schedule::{Schedule, Stage, SystemStage},
    world::World,
};
use futures::executor::block_on;
use nalgebra::{Isometry3, Perspective3};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    common_component::{Camera, Transform},
    render_system::{self, RenderState},
    time::{time_update, TimeResource},
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let mut game = Game::new(window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = game.handle_event(&event);
    });
}

struct Game {
    window: Window,
    world: World,
    //time_manager: TimeManager,
    update_sch: Schedule,
    frame_sch: Schedule,
}

impl Game {
    fn new(window: Window) -> Self {
        let mut world = World::new();
        world.insert_resource(TimeResource::new(
            Duration::from_secs_f64(1.0 / 60.0),
            Duration::from_secs_f64(1.0 / 60.0),
        ));
        let render_state = block_on(RenderState::init(&window));
        world.insert_resource(render_state);

        let size = window.inner_size();
        let aspect = size.width as f32 / size.height as f32;

        world
            .spawn()
            .insert(Transform {
                isometry: Isometry3::translation(1.0, 0.0, 0.0),
                parent: None,
                children: vec![],
            })
            .insert(Camera {
                perspective: Perspective3::new(aspect, 3.14 / 2.0, 0.05, 1000.0),
            });

        let mut update_sch = Schedule::default();
        let mut frame_sch = Schedule::default();

        update_sch.add_stage("physics", SystemStage::parallel().with_system(time_update));
        frame_sch.add_stage(
            "draw",
            SystemStage::parallel().with_system(render_system::render),
        );

        Self {
            window,
            update_sch,
            frame_sch,

            world,
            /*time_manager: TimeManager::new(
                Duration::from_secs_f64(1.0 / 60.0),
                Duration::from_secs_f64(1.0 / 60.0),
            ),*/
        }
    }
    fn do_update(&mut self) {
        self.update_sch.run(&mut self.world);
    }
    fn do_frame(&mut self) {
        self.frame_sch.run(&mut self.world);
    }
    fn handle_event<E>(&mut self, event: &Event<E>) -> ControlFlow {
        self.do_update();

        self.window.request_redraw();
        match event {
            Event::WindowEvent { event, window_id } => {
                //egui_winit_state.on_event(&egui_ctx, &event);
                match event {
                    WindowEvent::Resized(size) => {
                        self.world
                            .resource_mut::<RenderState>()
                            .resize_if_needed(&size, &self.window);
                    }
                    WindowEvent::CloseRequested => {
                        if *window_id == self.window.id() {
                            return ControlFlow::Exit;
                        }
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(_) => self.do_frame(),
            _ => (), //todo!(),
        }

        ControlFlow::Poll
    }
}
