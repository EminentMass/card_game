use bevy_ecs::{entity::Entity, prelude::Component};
use nalgebra::{Isometry3, Perspective3, Vector3};

use crate::{geometry_library::GeometryId, texture_library::TextureId};

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

#[derive(Clone, Copy, Debug, Component)]
pub struct RenderGeometry {
    pub geom_type: GeometryId,
}

impl RenderGeometry {
    pub fn new(geom_type: GeometryId) -> Self {
        Self { geom_type }
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct Texture {
    pub texture_id: TextureId,
}

impl Texture {
    pub fn new(texture_id: TextureId) -> Self {
        Self { texture_id }
    }
}

trait GetTextureId {
    fn get_texture_id(&self) -> Option<TextureId>;
}

impl GetTextureId for Option<Texture> {
    fn get_texture_id(&self) -> Option<TextureId> {
        match self {
            Some(s) => Some(s.texture_id),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct PointLight {
    pub color: Vector3<f32>,
    pub power: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct SpotLight {
    pub color: Vector3<f32>,
    pub power: f32,
    pub radius: f32,
    pub direction: Vector3<f32>,
    pub cut_off: f32,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct GlobalLight {
    pub color: Vector3<f32>,
    pub power: f32,
    pub direction: Vector3<f32>,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct Rotate {
    pub axis: Vector3<f32>,
}
