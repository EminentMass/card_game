use std::env;
use std::path::{Path, PathBuf};

use ct_spirv::Compiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_path = "shader";
    let mut binary_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    binary_path.push(source_path);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", source_path);

    let cmp = Compiler::new(Path::new(source_path), &binary_path);

    cmp.compile().unwrap();

    Ok(())
}
