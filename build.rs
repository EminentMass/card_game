use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use shaderc::Compiler;

// These map source extensions to compiled extensions
const SHADER_SOURCE_EXTENSIONS: [&str; 2] = ["vs", "fs"];

const SHADER_SOURCE_DIRECTORY: &str = "shader/";

//
// Compile glsl shaders to spir-v
//

// Create an iterator over all the shader source files within directory.
// This is not recursive it only finds files directly in the directory
fn shader_files_at<'a>(
    path: &'a Path,
) -> Result<Box<dyn Iterator<Item = PathBuf> + 'a>, Box<dyn std::error::Error>> {
    let iter = std::fs::read_dir(path)?.filter_map(|direntry| {
        // Traverse files checking if they are valid source files.
        // This does not access the file and only uses os operations to check filetype and name etc

        // Print possible error info than transpose to option and send up
        let entry = direntry
            .or_else(|err| {
                println!(
                    "couldn't access directory when searching for shaders in {}: {}",
                    path.display(),
                    err
                );
                Err(err)
            })
            .ok()?;

        let file_type = entry
            .file_type()
            .or_else(|err| {
                println!(
                    "couldn't access filetype of directory while searching for shaders in {}: {}",
                    path.display(),
                    err
                );
                Err(err)
            })
            .ok()?;

        if file_type.is_file() {
            // Only use shader source files with valid extensions.
            // This stops compiled files from being picked up among other files that might end up in the directory.
            let path = entry.path();
            let extension = path
                .extension()
                .expect("couldn't get extension from path of shader")
                .to_str()
                .expect("couldn't cast ostr file extension of shader to str");

            if SHADER_SOURCE_EXTENSIONS.contains(&extension) {
                Some(path)
            } else {
                None
            }
        } else {
            None
        }
    });

    Ok(Box::new(iter))
}

fn map_file_extension(path: &Path, append: &str) -> PathBuf {
    let extension_out = {
        let extension = path
            .extension()
            .expect("failed to get extension from shader source path")
            .to_str()
            .unwrap();

        if SHADER_SOURCE_EXTENSIONS.contains(&extension) {
            format!("{}{}", extension, append)
        } else {
            panic!("file extension is not that of a shader");
        }
    };

    let mut path_out = path.to_owned();
    path_out.set_extension(extension_out);
    path_out
}

// Compiler shader at location path and write spir-v to path.concat("spirv")
//
// vertex_shader.vs -> vertex_shader.vsspirv
//
fn compile_shader(compiler: &Compiler, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("compiling {}", path.display());

    let mut file = File::open(&path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let artifact = compiler.compile_into_spirv(
        &contents,
        shaderc::ShaderKind::InferFromSource, // All shader sources must have #pragma shader_stage()
        &path.display().to_string(),
        "main",
        None,
    )?;

    // add compiled file extension
    let path_out = map_file_extension(path, "spirv");

    let mut file_out = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path_out)?;

    file_out.write_all(artifact.as_binary_u8())?;

    // also write out assembly for debugging
    let assembly = compiler.compile_into_spirv_assembly(
        &contents,
        shaderc::ShaderKind::InferFromSource, // All shader sources must have #pragma shader_stage()
        &path.display().to_string(),
        "main",
        None,
    )?;

    let path_out = map_file_extension(path, "spirvasm");

    let mut file_out = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path_out)?;

    file_out.write_all(assembly.as_text().as_bytes())?;

    Ok(())
}

fn compile_shaders(source_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let compiler = Compiler::new().expect("couldn't create shaderc compiler");

    for path in shader_files_at(source_path)? {
        compile_shader(&compiler, &path).unwrap();
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_path = PathBuf::from(SHADER_SOURCE_DIRECTORY);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", SHADER_SOURCE_DIRECTORY);

    compile_shaders(&source_path)?;

    Ok(())
}
