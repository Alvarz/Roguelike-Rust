use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};
use specs::prelude::*;
mod area_ending_point;
mod area_starting_points;
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod cull_unreachable;
mod distant_exit;
mod dla;
mod door_placement;
mod drunkard;
mod dwarf_fort_builder;
mod forest;
mod limestone_cavern;
mod maze;
mod prefab_builder;
mod room_based_spawner;
mod room_based_stairs;
mod room_based_starting_position;
mod room_corner_rounding;
mod room_corridor_spawner;
mod room_draw;
mod room_exploder;
mod room_sorter;
mod rooms_corridors_bsp;
mod rooms_corridors_dogleg;
mod rooms_corridors_lines;
mod rooms_corridors_nearest;
mod simple_map;
mod town;
mod voronoi;
mod voronoi_spawning;
mod waveform_collapse;
use area_ending_point::*;
use area_starting_points::{AreaStartingPosition, XStart, YStart};
use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use common::*;
use cull_unreachable::CullUnreachable;
use distant_exit::DistantExit;
use dla::DLABuilder;
use door_placement::DoorPlacement;
use drunkard::DrunkardsWalkBuilder;
use dwarf_fort_builder::*;
use forest::forest_builder;
use limestone_cavern::*;
use maze::MazeBuilder;
use prefab_builder::PrefabBuilder;
use room_based_spawner::RoomBasedSpawner;
use room_based_stairs::RoomBasedStairs;
use room_based_starting_position::RoomBasedStartingPosition;
use room_corner_rounding::RoomCornerRounder;
use room_corridor_spawner::CorridorSpawner;
use room_draw::RoomDrawer;
use room_exploder::RoomExploder;
use room_sorter::{RoomSort, RoomSorter};
use rooms_corridors_bsp::BspCorridors;
use rooms_corridors_dogleg::DoglegCorridors;
use rooms_corridors_lines::StraightLineCorridors;
use rooms_corridors_nearest::NearestCorridors;
use simple_map::SimpleMapBuilder;
use town::town_builder;
use voronoi::VoronoiCellBuilder;
use voronoi_spawning::VoronoiSpawning;
use waveform_collapse::WaveformCollapseBuilder;
mod mushroom_forest;
use mushroom_forest::*;
mod dark_elves;
use dark_elves::*;

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub history: Vec<Map>,
    pub width: i32,
    pub height: i32,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new<S: ToString>(new_depth: i32, width: i32, height: i32, name: S) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth, width, height, name),
                starting_position: None,
                rooms: None,
                corridors: None,
                history: Vec::new(),
                width,
                height,
            },
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder."),
        };
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(&mut self.build_data);
            }
        }

        // Build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(&mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            let _ = spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap);
}

fn random_start_position() -> (XStart, YStart) {
    let x;
    let xroll = crate::rng::roll_dice(1, 3);
    match xroll {
        1 => x = XStart::LEFT,
        2 => x = XStart::CENTER,
        _ => x = XStart::RIGHT,
    }

    let y;
    let yroll = crate::rng::roll_dice(1, 3);
    match yroll {
        1 => y = YStart::BOTTOM,
        2 => y = YStart::CENTER,
        _ => y = YStart::TOP,
    }

    (x, y)
}

fn random_room_builder(builder: &mut BuilderChain) {
    let build_roll = crate::rng::roll_dice(1, 3);
    match build_roll {
        1 => builder.start_with(SimpleMapBuilder::new()),
        2 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new()),
    }

    // BSP Interior still makes holes in the walls
    if build_roll != 3 {
        // Sort by one of the 5 available algorithms
        let sort_roll = crate::rng::roll_dice(1, 5);
        match sort_roll {
            1 => builder.with(RoomSorter::new(RoomSort::LEFTMOST)),
            2 => builder.with(RoomSorter::new(RoomSort::RIGHTMOST)),
            3 => builder.with(RoomSorter::new(RoomSort::TOPMOST)),
            4 => builder.with(RoomSorter::new(RoomSort::BOTTOMMOST)),
            _ => builder.with(RoomSorter::new(RoomSort::CENTRAL)),
        }

        builder.with(RoomDrawer::new());

        let corridor_roll = crate::rng::roll_dice(1, 4);
        match corridor_roll {
            1 => builder.with(DoglegCorridors::new()),
            2 => builder.with(NearestCorridors::new()),
            3 => builder.with(StraightLineCorridors::new()),
            _ => builder.with(BspCorridors::new()),
        }

        let cspawn_roll = crate::rng::roll_dice(1, 2);
        if cspawn_roll == 1 {
            builder.with(CorridorSpawner::new());
        }

        let modifier_roll = crate::rng::roll_dice(1, 6);
        match modifier_roll {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            _ => {}
        }
    }

    let start_roll = crate::rng::roll_dice(1, 2);
    match start_roll {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position();
            builder.with(AreaStartingPosition::new(start_x, start_y));
        }
    }

    let exit_roll = crate::rng::roll_dice(1, 2);
    match exit_roll {
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new()),
    }

    let spawn_roll = crate::rng::roll_dice(1, 2);
    match spawn_roll {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new()),
    }
}

fn random_shape_builder(builder: &mut BuilderChain) {
    let builder_roll = crate::rng::roll_dice(1, 16);
    match builder_roll {
        1 => builder.start_with(CellularAutomataBuilder::new()),
        2 => builder.start_with(DrunkardsWalkBuilder::open_area()),
        3 => builder.start_with(DrunkardsWalkBuilder::open_halls()),
        4 => builder.start_with(DrunkardsWalkBuilder::winding_passages()),
        5 => builder.start_with(DrunkardsWalkBuilder::fat_passages()),
        6 => builder.start_with(DrunkardsWalkBuilder::fearful_symmetry()),
        7 => builder.start_with(MazeBuilder::new()),
        8 => builder.start_with(DLABuilder::walk_inwards()),
        9 => builder.start_with(DLABuilder::walk_outwards()),
        10 => builder.start_with(DLABuilder::central_attractor()),
        11 => builder.start_with(DLABuilder::insectoid()),
        12 => builder.start_with(VoronoiCellBuilder::pythagoras()),
        13 => builder.start_with(VoronoiCellBuilder::manhattan()),
        _ => builder.start_with(PrefabBuilder::constant(
            prefab_builder::prefab_levels::WFC_POPULATED,
        )),
    }

    // Set the start to the center and cull
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());

    // Now set the start to a random starting area
    let (start_x, start_y) = random_start_position();
    builder.with(AreaStartingPosition::new(start_x, start_y));

    // Setup an exit and spawn mobs
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
}

pub fn random_builder(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height, "New Map");
    let type_roll = crate::rng::roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(&mut builder),
        _ => random_shape_builder(&mut builder),
    }

    if crate::rng::roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());

        // Now set the start to a random starting area
        let (start_x, start_y) = random_start_position();
        builder.with(AreaStartingPosition::new(start_x, start_y));

        // Setup an exit and spawn mobs
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    if crate::rng::roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(
            prefab_builder::prefab_sections::UNDERGROUND_FORT,
        ));
    }

    builder.with(DoorPlacement::new());
    builder.with(PrefabBuilder::vaults());

    builder
}

pub fn level_builder(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    match new_depth {
        1 => town_builder(new_depth, width, height),
        2 => forest_builder(new_depth, width, height),
        3 => limestone_cavern_builder(new_depth, width, height),
        4 => limestone_deep_cavern_builder(new_depth, width, height),
        5 => limestone_transition_builder(new_depth, width, height),
        6 => dwarf_fort_builder(new_depth, width, height),
        7 => mushroom_entrance(new_depth, width, height),
        8 => mushroom_builder(new_depth, width, height),
        9 => mushroom_exit(new_depth, width, height),
        10 => dark_elf_city(new_depth, width, height),
        11 => dark_elf_plaza(new_depth, width, height),
        _ => random_builder(new_depth, width, height),
    }
}
