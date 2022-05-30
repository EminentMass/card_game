mod common_component;
mod data_types;
mod game;
mod import_mesh;
mod render_system;
mod shader_library;
mod time;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init().unwrap();

    game::run()
}
