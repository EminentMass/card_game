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
    pub projection: Perspective3<f32>,
}
#[derive(Copy, Clone, Debug, Component)]
pub struct MainCamera;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum GeometryType {
    Plane,
    Cube,
    Tetrahedron,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct RenderGeometry {
    pub geom_type: GeometryType,
}

impl RenderGeometry {
    pub fn new(geom_type: GeometryType) -> Self {
        Self { geom_type }
    }
}
