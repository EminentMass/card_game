#![allow(dead_code)]
use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use wgpu::{Device, ShaderModule};

type ShaderId = usize;

#[derive(Debug, Clone)]
pub struct Shader {
    name: String,
    source_path: PathBuf,

    entry_point: String,

    handle: Arc<ShaderModule>,
}

impl Shader {
    pub fn new(device: &Device, source_path: &Path) -> Self {
        ShaderBuilder::new(source_path).build(device)
    }

    pub fn all(device: &Device, source_path: &Path, name: &str, entry_point: &str) -> Self {
        let mut file = match File::open(source_path) {
            Err(why) => {
                panic!(
                    "failed to open shader file {}: {}",
                    source_path.display(),
                    why
                );
            }
            Ok(file) => file,
        };

        let mut contents = Vec::new();
        match file.read_to_end(&mut contents) {
            Err(why) => {
                panic!(
                    "failed to read shader file {}: {}",
                    source_path.display(),
                    why
                );
            }
            Ok(_) => (),
        }

        assert!(
            contents.len() % 4 == 0,
            "shader source file missing alignment possibly wrong filepath"
        );
        let data = bytemuck::cast_slice(&contents);

        let handle = Arc::new(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::SpirV(Cow::Borrowed(data)),
        }));

        Self {
            name: name.to_string(),
            source_path: source_path.to_owned(),
            entry_point: entry_point.to_string(),
            handle,
        }
    }

    pub fn handle(&self) -> Arc<ShaderModule> {
        self.handle.clone()
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
    shaders: Vec<Shader>,
}

impl ShaderLibrary {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get(&self, id: ShaderId) -> &Shader {
        &self
            .shaders
            .get(id)
            .expect("tried to access shader with bad id")
    }
}

pub struct ShaderLibraryBuilder {
    builders: Vec<ShaderBuilder>,
}

impl ShaderLibraryBuilder {
    pub fn new() -> Self {
        Self {
            builders: Vec::new(),
        }
    }

    // Both add and add_builders cant use standard builder format as the ShaderId (Index within library) has to be available to the caller.
    pub fn add(&mut self, source_path: &Path) -> ShaderId {
        self.builders.push(ShaderBuilder::new(source_path));
        self.builders.len() - 1
    }

    pub fn add_builder(&mut self, builder: ShaderBuilder) -> ShaderId {
        self.builders.push(builder);
        self.builders.len() - 1
    }

    pub fn build(self, device: &Device) -> ShaderLibrary {
        ShaderLibrary {
            shaders: self.builders.into_iter().map(|b| b.build(device)).collect(),
        }
    }
}

// tests are outdated shaders use features that aren't requested
// TODO: update test to either not actually create module or use wgpu features

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
