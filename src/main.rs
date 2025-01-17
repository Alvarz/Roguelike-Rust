extern crate serde;
use bracket_lib::prelude::{BError, Point};
use specs::prelude::*;
use specs::saveload::SimpleMarkerAllocator;

mod config;
pub use config::*;
mod map;
pub use map::*;
mod utils;
pub use utils::*;
pub mod game;
pub use game::*;
pub mod effects;
mod gamelog;
mod gui;
pub mod map_builders;
pub mod raws;
#[macro_use]
extern crate lazy_static;
pub mod spatial;
mod systems;

fn main() -> BError {
    use bracket_lib::prelude::BTermBuilder;
    let mut context = BTermBuilder::simple(80, 60)
        .unwrap()
        .with_title(PROJECT_NAME)
        .with_font("vga8x16.png", 8, 16)
        .with_fullscreen(true)
        .with_fps_cap(30.0)
        .with_sparse_console(80, 30, "vga8x16.png")
        .with_vsync(false)
        .build()?;
    context.with_post_scanlines(true);
    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
        dispatcher: systems::build(),
    };
    gs = register_components(gs);
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();

    gs.ecs.insert(map::MasterDungeonMap::new());
    gs.ecs.insert(Map::new(1, 64, 64, "New Map"));
    gs.ecs.insert(Point::new(0, 0));
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MapGeneration {});
    gs.ecs
        .insert(systems::particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    bracket_lib::prelude::main_loop(context, gs)
}
