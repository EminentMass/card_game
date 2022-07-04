use std::{collections::HashMap, mem::size_of, path::Path, sync::Arc};

use wgpu::{util::DeviceExt, Device};

use crate::data_types::Vertex as Vert;

use bytemuck::{cast_slice, cast_slice_mut};

crate::macros::parallel_enum_values! {
    (
        GeometryId,
        GEOMETRY_PATH_PAIRS,
        str,
    )
    TorusGeometry -> "torus.obj",
}

pub struct MeshData {
    pub vertex_len: u32,
    pub index_len: u32,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
}

impl MeshData {
    fn from_file(device: &Device, path: &Path) -> Self {
        // TODO: use material data
        let (models, _material) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: true,
                ignore_lines: true,
            },
        )
        .unwrap_or_else(|e| panic!("failed to open obj file {}: {}", path.display(), e));

        let mesh = &models
            .first()
            .unwrap_or_else(|| panic!("failed to parse obj file no models {}", path.display()))
            .mesh;

        let mut index_data: Vec<u16> = mesh
            .indices
            .iter()
            .map(|i: &u32| {
                (*i).try_into()
                    .expect("obj file index out of bounds greater than 65536")
            })
            .collect();

        cast_slice_mut(&mut index_data)
            .iter_mut()
            .for_each(|a: &mut [u16; 3]| a.reverse());

        let vertex_data: Vec<Vert> = {
            let p = cast_slice::<f32, [f32; 3]>(&mesh.positions).iter();
            let n = cast_slice::<f32, [f32; 3]>(&mesh.normals).iter();
            let uv = cast_slice(&mesh.texcoords).iter();

            p.zip(n)
                .zip(uv)
                .map(|(([x, y, z], [nx, ny, nz]), uv)| Vert {
                    position: [*x, *y, *z, 1.0].into(),
                    normal: [*nx, *ny, *nz, 0.0].into(),
                    texture: *uv,
                })
                .collect()
        };

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertices,
            indices,
            vertex_len: vertex_data.len() as u32,
            index_len: index_data.len() as u32,
        }
    }
}

pub struct GeometryLibrary {
    geometries: HashMap<GeometryId, Arc<MeshData>>,
}

impl GeometryLibrary {
    pub fn load_as_needed() -> Self {
        todo!();
    }

    pub fn load_all(device: &Device) -> Self {
        let geometries = GEOMETRY_PATH_PAIRS
            .iter()
            .map(|(id, g)| (*id, Arc::new(MeshData::from_file(device, Path::new(g)))))
            .collect();

        Self { geometries }
    }

    pub fn get(&self, id: GeometryId) -> &MeshData {
        &self
            .geometries
            .get(&id)
            .expect("tried to access texture with bad id")
    }
}
