use std::time::Duration;

use bevy_ecs::{
    schedule::{Schedule, Stage, SystemStage},
    system::{Query, Res},
    world::World,
};
use nalgebra::{Isometry3, Perspective3, UnitQuaternion, Vector3};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    common_component::{Camera, GeometryType, MainCamera, RenderGeometry, Transform},
    render_system::{self, RenderState},
    time::{frame_criteria, update_criteria, TimeResource},
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
    frame_schedule: Schedule,
    update_schedule: Schedule,
}

impl Game {
    fn new(window: Window) -> Self {
        let mut world = World::new();
        let render_state = RenderState::init(&window);
        world.insert_resource(render_state);
        world.insert_resource(TimeResource::new(
            Duration::from_secs_f64(1.0 / 60.0),
            Duration::from_secs_f64(1.0 / 60.0),
        ));

        let size = window.inner_size();
        let aspect = size.width as f32 / size.height as f32;

        world
            .spawn()
            .insert(Transform {
                isometry: Isometry3::translation(3.0, 0.0, 0.0),
                parent: None,
                children: vec![],
            })
            .insert(Camera {
                projection: Perspective3::new(aspect, 3.14 / 2.0, 0.05, 1000.0),
            })
            .insert(MainCamera);
        world
            .spawn()
            .insert(Transform {
                isometry: Isometry3::translation(0.0, 0.0, -5.0),
                parent: None,
                children: vec![],
            })
            .insert(RenderGeometry::new(GeometryType::Plane));
        world
            .spawn()
            .insert(Transform {
                isometry: Isometry3::translation(3.0, 0.0, -5.0),
                parent: None,
                children: vec![],
            })
            .insert(RenderGeometry::new(GeometryType::Cube));
        world
            .spawn()
            .insert(Transform {
                isometry: Isometry3::translation(6.0, 0.0, -5.0),
                parent: None,
                children: vec![],
            })
            .insert(RenderGeometry::new(GeometryType::Tetrahedron));

        for i in 0..10 {
            world
                .spawn()
                .insert(Transform {
                    isometry: Isometry3::translation(i as f32, 3.0, -5.0),
                    parent: None,
                    children: vec![],
                })
                .insert(RenderGeometry::new(GeometryType::Tetrahedron));
        }

        let update_stage = SystemStage::parallel()
            .with_run_criteria(update_criteria)
            .with_system(rotate_geometries);
        let mut update_schedule = Schedule::default();
        update_schedule.add_stage("update", update_stage);

        let frame_stage = SystemStage::parallel()
            .with_run_criteria(frame_criteria)
            .with_system(render_system::render);

        let mut frame_schedule = Schedule::default();
        frame_schedule.add_stage("frame", frame_stage);

        Self {
            window,
            world,
            update_schedule,
            frame_schedule,
        }
    }

    fn update_as_needed(&mut self) {
        self.update_schedule.run(&mut self.world);
    }

    fn render(&mut self) {
        self.frame_schedule.run(&mut self.world);
    }

    fn handle_event<E>(&mut self, event: &Event<E>) -> ControlFlow {
        self.window.request_redraw();
        match event {
            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::Resized(size) => {
                    if *window_id == self.window.id() {
                        self.world
                            .resource_mut::<RenderState>()
                            .resize_if_needed(&size, &self.window);
                    }
                }
                WindowEvent::CloseRequested => {
                    if *window_id == self.window.id() {
                        return ControlFlow::Exit;
                    }
                }
                _ => (),
            },
            Event::RedrawRequested(_) => self.render(),
            _ => (), //todo!(),
        }

        self.update_as_needed();

        ControlFlow::Poll
    }
}

fn rotate_geometries(time: Res<TimeResource>, mut geoms: Query<(&RenderGeometry, &mut Transform)>) {
    let dt = time.update_dt.as_secs_f32();
    for (RenderGeometry { geom_type }, mut trans) in geoms.iter_mut() {
        let axis = match geom_type {
            GeometryType::Plane => Vector3::x(),
            GeometryType::Cube => Vector3::y(),
            GeometryType::Tetrahedron => Vector3::z(),
        };
        let rot = UnitQuaternion::new(axis * dt);
        trans.isometry.append_rotation_wrt_center_mut(&rot);
    }
}
