mod common_component;
mod data_types;
mod game;
mod geometry_library;
mod macros;
mod render_system;
mod shader_library;
mod texture_library;
mod tile_world;
mod time;
mod util;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    game::run()
}
