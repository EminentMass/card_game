use nalgebra::Vector3;
use std::fs::File;
use std::path::Path;

struct Mesh {
    vertices: Vec<Vector3<f32>>,
}

fn import_stl(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;

    let stl = stl_io::read_stl(&mut file)?;
    stl.validate()?;

    Ok(())
}
