mod common_component;
mod data_types;
mod game;
mod geometry_library;
mod pvnrt;
mod render_system;
mod shader_library;
mod time;
mod util;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Error).unwrap();

    game::run()
}
