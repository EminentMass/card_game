#![allow(dead_code)]
use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use shaderc::{Compiler, ShaderKind};
use wgpu::{Device, ShaderModule};

type ShaderId = usize;

trait ToShaderKind {
    fn to_shader_kind(&self) -> ShaderKind;
}

impl ToShaderKind for &str {
    fn to_shader_kind(&self) -> ShaderKind {
        match *self {
            "vs" => shaderc::ShaderKind::Vertex,
            "fs" => shaderc::ShaderKind::Fragment,
            "cs" => shaderc::ShaderKind::Compute,
            "gs" => shaderc::ShaderKind::Geometry,
            _ => panic!(
                "unable to determine shader kind from file extension maybe specify in declaration"
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shader {
    name: String,
    source_path: PathBuf,

    entry_point: String,
    shader_kind: ShaderKind,

    handle: Arc<ShaderModule>,
}

unsafe impl Send for Shader {}
unsafe impl Sync for Shader {}

impl Shader {
    pub fn new(device: &Device, source_path: &Path) -> Self {
        ShaderBuilder::new(source_path).build(device)
    }

    pub fn all(
        device: &Device,
        source_path: &Path,
        name: &str,
        entry_point: &str,
        shader_kind: ShaderKind,
    ) -> Self {
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

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Err(why) => {
                panic!(
                    "failed to read shader file {}: {}",
                    source_path.display(),
                    why
                );
            }
            Ok(_) => (),
        }

        let artifact = Compiler::new()
            .expect("couldn't create shader spir-v compiler")
            .compile_into_spirv(&contents, shader_kind, name, entry_point, None)
            .unwrap_or_else(|err| {
                panic!(
                    "failed to compile shader at {}: {}",
                    source_path.display(),
                    err
                );
            });

        let handle = Arc::new(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::SpirV(Cow::Borrowed(artifact.as_binary())),
        }));

        Self {
            name: name.to_string(),
            source_path: source_path.to_owned(),
            entry_point: entry_point.to_string(),
            shader_kind,
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

    pub fn shader_kind(&self) -> ShaderKind {
        self.shader_kind
    }
}

pub struct ShaderBuilder {
    name: String,
    source_path: PathBuf,

    entry_point: String,
    shader_kind: ShaderKind,
}

unsafe impl Send for ShaderBuilder {}
unsafe impl Sync for ShaderBuilder {}

impl ShaderBuilder {
    pub fn new(source_path: &Path) -> Self {
        let name = source_path
            .file_name()
            .expect("no file name in shader source path")
            .to_str()
            .expect("failed to convert os string to string")
            .to_string();
        let entry_point = "main".to_string();
        let shader_kind = source_path
            .extension()
            .expect("no file extension in shader source path")
            .to_str()
            .expect("failed to convert os string to string")
            .to_shader_kind();
        Self {
            name,
            source_path: source_path.to_owned(),
            entry_point,
            shader_kind,
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

    pub fn shader_kind(mut self, shader_kind: ShaderKind) -> Self {
        self.shader_kind = shader_kind;
        self
    }

    pub fn build(self, device: &Device) -> Shader {
        let ShaderBuilder {
            name,
            source_path,
            entry_point,
            shader_kind,
        } = self;
        Shader::all(device, &source_path, &name, &entry_point, shader_kind)
    }
}

#[derive(Default)]
pub struct ShaderLibrary {
    shaders: Vec<Shader>,
}

unsafe impl Send for ShaderLibrary {}
unsafe impl Sync for ShaderLibrary {}

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

unsafe impl Send for ShaderLibraryBuilder {}
unsafe impl Sync for ShaderLibraryBuilder {}

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

#[cfg(test)]
mod tests {

    use super::*;

    use std::future::Future;
    use tokio::runtime::Runtime;

    use wgpu::{Adapter, Device, Instance, Queue};

    use shaderc::ShaderKind;

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

    fn run_async<F: Future>(func: F) {
        let rt = Runtime::new().expect("unable to create tokio runtime");

        rt.block_on(func);
    }

    #[test]
    fn shader_builder_generic_test() {
        run_async(async {
            let (_instance, _adapter, device, _queue) = init_wgpu().await;

            let path = PathBuf::from("shader/vertex_shader.vs");
            let builder = ShaderBuilder::new(&path);
            let shader = builder.build(&device);

            assert_eq!(
                shader.source_path.to_str().unwrap(),
                "shader/vertex_shader.vs"
            );

            assert_eq!(shader.name, "vertex_shader.vs");
            assert_eq!(shader.shader_kind, ShaderKind::Vertex);
            assert_eq!(shader.entry_point, "main");
        });
    }

    #[test]
    fn shader_builder_specific_test() {
        run_async(async {
            let (_instance, _adapter, device, _queue) = init_wgpu().await;

            let path = PathBuf::from("shader/vertex_shader.vs");
            let builder = ShaderBuilder::new(&path)
                .name("Joblin")
                .shader_kind(ShaderKind::Vertex)
                .entry_point("main2");
            let shader = builder.build(&device);

            assert_eq!(
                shader.source_path.to_str().unwrap(),
                "shader/vertex_shader.vs"
            );

            assert_eq!(shader.name, "Joblin");
            assert_eq!(shader.shader_kind, ShaderKind::Vertex);
            assert_eq!(shader.entry_point, "main2");
        });
    }

    #[test]
    fn shader_library_builder_generic_test() {
        run_async(async {
            let (_instance, _adapter, device, _queue) = init_wgpu().await;

            let path = PathBuf::from("shader/vertex_shader.vs");
            let path2 = PathBuf::from("shader/fragment_shader.fs");

            let mut builder = ShaderLibraryBuilder::new();
            let shader = builder.add(&path);
            let shader2 = builder.add(&path2);
            let library = builder.build(&device);

            let shader = library.get(shader);

            assert_eq!(
                shader.source_path.to_str().unwrap(),
                "shader/vertex_shader.vs"
            );

            let shader = library.get(shader2);
            assert_eq!(
                shader.source_path.to_str().unwrap(),
                "shader/fragment_shader.fs"
            );
        });
    }
}
