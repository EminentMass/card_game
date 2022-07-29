#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use std::{mem::size_of, num::NonZeroU64};

use crate::common_component::{
    GlobalLight as GlobalLightComponent, PointLight as PointLightComponent,
    SpotLight as SpotLightComponent, Transform,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: Vector4<f32>,
    pub normal: Vector4<f32>,
    pub texture: Vector2<f32>,
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn pos(pos: &Vector3<f32>) -> Self {
        Self {
            position: [pos.x, pos.y, pos.z, 1.0].into(),
            normal: Vector4::zeros(),
            texture: Vector2::zeros(),
        }
    }

    pub fn pos_and_tex(pos: &Vector3<f32>, tex: &Vector2<f32>) -> Self {
        Self {
            position: [pos.x, pos.y, pos.z, 1.0].into(),
            normal: Vector4::zeros(),
            texture: *tex,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Camera {
    pub view_projection: Matrix4<f32>,
    pub position: Vector4<f32>,
}

impl Camera {
    pub const BINDING_SIZE: Option<NonZeroU64> =
        NonZeroU64::new(std::mem::size_of::<Self>() as u64);
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GlobalLight {
    pub color: Vector4<f32>,     // w is power
    pub direction: Vector4<f32>, // w is always 0 as this is a direction not point
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PointLight {
    pub position: Vector4<f32>, // w is radius
    pub color: Vector4<f32>,    // w is power
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SpotLight {
    pub position: Vector4<f32>,  // w is radius
    pub color: Vector4<f32>,     // w is power
    pub direction: Vector4<f32>, // w is cut off
}

impl Default for GlobalLight {
    fn default() -> Self {
        Self {
            color: [0.0, 0.0, 0.0, 0.0].into(),
            direction: Vector4::x(),
        }
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 0.0].into(),
            color: [0.0, 0.0, 0.0, 0.0].into(),
        }
    }
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 0.0].into(),
            color: [0.0, 0.0, 0.0, 0.0].into(),
            direction: [1.0, 0.0, 0.0, 1.0].into(),
        }
    }
}

impl From<&GlobalLightComponent> for GlobalLight {
    fn from(gl: &GlobalLightComponent) -> Self {
        Self {
            direction: [gl.direction.x, gl.direction.y, gl.direction.z, 0.0].into(),
            color: [gl.color.x, gl.color.y, gl.color.z, gl.power].into(),
        }
    }
}

impl From<(&PointLightComponent, &Transform)> for PointLight {
    fn from((pl, t): (&PointLightComponent, &Transform)) -> Self {
        let t = &t.isometry.translation;

        Self {
            position: [t.x, t.y, t.z, pl.radius].into(),
            color: [pl.color.x, pl.color.y, pl.color.z, pl.power].into(),
        }
    }
}

impl From<(&SpotLightComponent, &Transform)> for SpotLight {
    fn from((sl, t): (&SpotLightComponent, &Transform)) -> Self {
        let t = &t.isometry.translation;

        Self {
            position: [t.x, t.y, t.z, sl.radius].into(),
            color: [sl.color.x, sl.color.y, sl.color.z, sl.power].into(),
            direction: [sl.direction.x, sl.direction.y, sl.direction.z, sl.cut_off].into(),
        }
    }
}

pub struct Instance {
    pub model: Matrix4<f32>,
}

impl Instance {
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] = [
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: size_of::<Vector4<f32>>() as u64,
            shader_location: 4,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: size_of::<Vector4<f32>>() as u64 * 2,
            shader_location: 5,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: size_of::<Vector4<f32>>() as u64 * 3,
            shader_location: 6,
            format: wgpu::VertexFormat::Float32x4,
        },
    ];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
