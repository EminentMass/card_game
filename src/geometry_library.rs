use std::{collections::HashMap, ops::Range, path::Path, sync::Arc};

use wgpu::{util::DeviceExt, BufferAddress, Device};

use crate::data_types::Vertex as Vert;

use bytemuck::cast_slice;

crate::macros::parallel_enum_values! {
    (
        GeometryId,
        GEOMETRY_PATH_PAIRS,
        str,
    )
    TorusGeometry -> "model/torus.obj",
    SceneTestGeometry -> "model/scene_test.obj",
}

#[allow(dead_code)]
pub struct MultiMeshData {
    pub index_ranges: Vec<Range<BufferAddress>>,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
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

        reverse_indices(&mut index_data);

        let vertex_data: Vec<Vert> = transmute_vertex_data(mesh);

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
    #[allow(dead_code)]
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

// transmute vertex data from tobj mesh representation to internal rendering engine representation
fn transmute_vertex_data(mesh: &tobj::Mesh) -> Vec<Vert> {
    // the creation of tobj mesh should create proper length data
    let p = mesh.positions.chunks(3);
    let n = mesh.normals.chunks(3);
    let t = mesh.texcoords.chunks(2);

    p.zip(n)
        .zip(t)
        .map(|((p, n), t)| Vert {
            position: [p[0], p[1], p[2], 1.0].into(),
            normal: [n[0], n[1], n[2], 0.0].into(),
            texture: [t[0], t[1]].into(),
        })
        .collect()
}

fn reverse_indices<T>(indices: &mut [T]) {
    assert!(
        indices.len() % 3 == 0,
        "tried to reverse index data with incorrect length"
    );
    indices.chunks_mut(3).for_each(|a: &mut [T]| a.reverse());
}
