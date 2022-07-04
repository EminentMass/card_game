#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use std::mem::size_of;

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
