use bevy_ecs::{entity::Entity, prelude::Component};
use nalgebra::{Isometry3, Perspective3};

#[derive(Clone, Debug, Component)]
pub struct Transform {
    pub isometry: Isometry3<f32>,

    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}
#[derive(Clone, Debug, Component)]
pub struct Camera {
    pub perspective: Perspective3<f32>,
}
#[derive(Clone, Debug, Component)]
pub struct MainCamera;
