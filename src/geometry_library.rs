use std::mem::size_of;

use crate::data_types::Vertex as Vert;

macro_rules! v {
    ($($x:expr, $y:expr, $z:expr),*) => {
        [
            $(Vert::pos(&[$x, $y, $z].into()),)*
        ]
    };
}

macro_rules! vt {
    ($($x:expr, $y:expr, $z:expr, $u:expr, $v:expr),*) => {
        [
            $(Vert::pos_and_tex(&[$x, $y, $z].into(), &[$u, $v].into()),)*
        ]
    };
}

#[rustfmt::skip]
pub fn plane_vertices() -> [Vert; 4] {
    vt![
         1.0,  1.0, 0.0, 1.0, 1.0,
        -1.0,  1.0, 0.0, 1.0, 0.0,
         1.0, -1.0, 0.0, 0.0, 1.0,
        -1.0, -1.0, 0.0, 0.0, 0.0
    ]
}
#[rustfmt::skip]
pub fn plane_indices() -> [u16; 6] {
    [
        3, 1, 0,
        2, 3, 0,
    ]
}

#[rustfmt::skip]
pub fn cube_vertices() -> [Vert; 8] {
    vt![
        -1.0, -1.0,  1.0, 1.0, 1.0, // Front four
         1.0, -1.0,  1.0, 1.0, 0.0,
         1.0,  1.0,  1.0, 0.0, 1.0,
        -1.0,  1.0,  1.0, 0.0, 0.0,
        -1.0,  1.0, -1.0, 1.0, 1.0, // back four
         1.0,  1.0, -1.0, 1.0, 0.0,
         1.0, -1.0, -1.0, 0.0, 1.0,
        -1.0, -1.0, -1.0, 0.0, 0.0
    ]
}

#[rustfmt::skip]
pub fn cube_indices() -> [u16; 12 * 3] {
    [
        2, 1, 0, // Front
        3, 2, 0,
        6, 5, 7, // Back
        5, 4, 7,
        1, 6, 0, // Bottom
        6, 7, 0,
        3, 4, 2, // Top
        4, 5, 2,
        4, 3, 0, // Left
        7, 4, 0,
        2, 5, 1, // Right
        5, 6, 1,
    ]
}

#[rustfmt::skip]
pub fn tetrahedron_vertices() -> [Vert; 4] {
    // origin is center of object. base is under the y plane with the point sticking up
    v![ 
        0.0, -0.57735, -1.15470, // base
       -1.0, -0.57735,  0.57735,
        1.0, -0.57735,  0.57735,
        0.0,  1.15470,  0.0      // point sticking up along y
    ]
}

#[rustfmt::skip]
pub fn tetrahedron_indices() -> [u16; 12] {
    [
        0, 1, 2,
        0, 3, 1, 
        3, 0, 2, 
        2, 1, 3
    ]
}

pub struct GeometryLibrary {
    vertex_data: Vec<u8>,
    index_data: Vec<u8>,
}

impl GeometryLibrary {
    pub const PLANE_VERTEX_OFFSET: u64 = 0;
    pub const CUBE_VERTEX_OFFSET: u64 = size_of::<[Vert; 4]>() as u64;
    pub const TETRAHEDRON_VERTEX_OFFSET: u64 = size_of::<[Vert; 4 + 8]>() as u64; // plane plus cube vertices

    pub const PLANE_INDEX_OFFSET: u64 = 0;
    pub const CUBE_INDEX_OFFSET: u64 = size_of::<[u16; 6]>() as u64;
    pub const TETRAHEDRON_INDEX_OFFSET: u64 = size_of::<[u16; 6 + 12 * 3]>() as u64; // plane plus cube vertices

    pub const PLANE_INDEX_COUNT: u32 = 6;
    pub const CUBE_INDEX_COUNT: u32 = 12 * 3;
    pub const TETRAHEDRON_INDEX_COUNT: u32 = 12;

    pub fn new() -> Self {
        use bytemuck::cast_slice as to_u8;

        let vertex_data = to_u8(&plane_vertices())
            .iter()
            .chain(to_u8(&cube_vertices()))
            .chain(to_u8(&tetrahedron_vertices()))
            .cloned()
            .collect();

        let index_data = to_u8(&plane_indices())
            .iter()
            .chain(to_u8(&cube_indices()))
            .chain(to_u8(&tetrahedron_indices()))
            .cloned()
            .collect();

        Self {
            vertex_data,
            index_data,
        }
    }

    pub fn geometry_vertex_data(&self) -> &[u8] {
        &self.vertex_data
    }

    pub fn geometry_index_data(&self) -> &[u8] {
        &self.index_data
    }
}
