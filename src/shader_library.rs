#![allow(dead_code)]
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use wgpu::{Device, ShaderModule};

crate::macros::parallel_enum_values!(
    (
        ShaderId,
        SHADER_PATH_PAIRS,
        str,
    )
    VertexShader -> "shader/vertex_shader.vsspirv",
    FragmentShader -> "shader/fragment_shader.fsspirv",
);

#[derive(Debug)]
pub struct Shader {
    name: String,
    source_path: PathBuf,

    entry_point: String,

    handle: ShaderModule,
}

impl Shader {
    pub fn new(device: &Device, source_path: &Path) -> Self {
        ShaderBuilder::new(source_path).build(device)
    }

    pub fn all(device: &Device, source_path: &Path, name: &str, entry_point: &str) -> Self {
        let mut file = File::open(source_path).unwrap_or_else(|e| {
            panic!(
                "failed to open shader file {}: {}",
                source_path.display(),
                e
            )
        });

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap_or_else(|e| {
            panic!(
                "failed to read shader file {}: {}",
                source_path.display(),
                e
            )
        });

        assert!(
            contents.len() % 4 == 0,
            "shader source file missing alignment possibly wrong filepath"
        );
        let data = bytemuck::cast_slice(&contents);

        let handle = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::SpirV(Cow::Borrowed(data)),
        });

        Self {
            name: name.to_string(),
            source_path: source_path.to_owned(),
            entry_point: entry_point.to_string(),
            handle,
        }
    }

    pub fn handle(&self) -> &ShaderModule {
        &self.handle
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn entry_point(&self) -> &str {
        &self.entry_point
    }
}

pub struct ShaderBuilder {
    name: String,
    source_path: PathBuf,

    entry_point: String,
}

impl ShaderBuilder {
    pub fn new(source_path: &Path) -> Self {
        let name = source_path
            .file_name()
            .expect("no file name in shader source path")
            .to_str()
            .expect("failed to convert os string to string")
            .to_string();
        let entry_point = "main".to_string();
        Self {
            name,
            source_path: source_path.to_owned(),
            entry_point,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn source_path(mut self, source_path: &Path) -> Self {
        self.source_path = source_path.to_owned();
        self
    }

    pub fn entry_point(mut self, entry_point: &str) -> Self {
        self.entry_point = entry_point.to_string();
        self
    }

    pub fn build(self, device: &Device) -> Shader {
        let ShaderBuilder {
            name,
            source_path,
            entry_point,
        } = self;
        Shader::all(device, &source_path, &name, &entry_point)
    }
}

#[derive(Default)]
pub struct ShaderLibrary {
    shaders: HashMap<ShaderId, Arc<Shader>>,
}

impl ShaderLibrary {
    // TODO: implement on the fly shader loading and unloading.
    pub fn load_as_needed() -> Self {
        todo!();
    }

    pub fn load_all(device: &Device) -> Self {
        let build_out_dir = Path::new(&env!("OUT_DIR"));

        let shaders = SHADER_PATH_PAIRS
            .iter()
            .map(|(id, s)| {
                (
                    *id,
                    Arc::new(ShaderBuilder::new(&build_out_dir.join(s)).build(device)),
                )
            })
            .collect();

        Self { shaders }
    }

    pub fn get(&self, id: ShaderId) -> &Shader {
        &self
            .shaders
            .get(&id)
            .expect("tried to access shader with bad id")
    }
}

// tests are outdated shaders use features that aren't requested
// TODO: update test to either not actually create module or use wgpu features

/*
#[cfg(test)]
mod tests {

    use crate::util::BlockOn;

    use super::*;

    use wgpu::{Adapter, Device, Instance, Queue};

    async fn init_wgpu() -> (Instance, Adapter, Device, Queue) {
        let instance = Instance::new(wgpu::Backends::all());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("failed to find appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("failed to create appropriate device");

        (instance, adapter, device, queue)
    }

    #[test]
    fn shader_builder_generic_test() {
        let (_instance, _adapter, device, _queue) = init_wgpu().block_on();

        let path = PathBuf::from("shader/vertex_shader.vsspirv");
        let builder = ShaderBuilder::new(&path);
        let shader = builder.build(&device);

        assert_eq!(
            shader.source_path.to_str().unwrap(),
            "shader/vertex_shader.vs"
        );

        assert_eq!(shader.name, "vertex_shader.vsspirv");
        assert_eq!(shader.entry_point, "main");
    }

    #[test]
    fn shader_builder_specific_test() {
        let (_instance, _adapter, device, _queue) = init_wgpu().block_on();

        let path = PathBuf::from("shader/vertex_shader.vsspirv");
        let builder = ShaderBuilder::new(&path)
            .name("Joblin")
            .entry_point("main2");
        let shader = builder.build(&device);

        assert_eq!(
            shader.source_path.to_str().unwrap(),
            "shader/vertex_shader.vs"
        );

        assert_eq!(shader.name, "Joblin");
        assert_eq!(shader.entry_point, "main2");
    }

    #[test]
    fn shader_library_builder_generic_test() {
        let (_instance, _adapter, device, _queue) = init_wgpu().block_on();

        let path = PathBuf::from("shader/vertex_shader.vsspirv");
        let path2 = PathBuf::from("shader/fragment_shader.fsspirv");

        let mut builder = ShaderLibraryBuilder::new();
        let shader = builder.add(&path);
        let shader2 = builder.add(&path2);
        let library = builder.build(&device);

        let shader = library.get(shader);

        assert_eq!(
            shader.source_path.to_str().unwrap(),
            "shader/vertex_shader.vsspirv"
        );

        let shader = library.get(shader2);
        assert_eq!(
            shader.source_path.to_str().unwrap(),
            "shader/fragment_shader.fsspirv"
        );
    }
}
*/
