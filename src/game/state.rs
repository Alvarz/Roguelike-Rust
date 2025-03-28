extern crate serde;
use bracket_lib::prelude::{BTerm, GameState};
use specs::prelude::*;

use crate::{
    camera, freeze_level_entities, gamelog, gui, map, saveload, systems, Map, MasterDungeonMap,
    PROJECT_NAME, SHOW_DEPTH, SHOW_FPS, SHOW_MAPGEN_VISUALIZER, SHOW_SEED,
};

use super::{components::*, spawner};
use super::{damage, player};

#[derive(PartialEq, Copy, Clone)]
pub enum VendorMode {
    Buy,
    Sell,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    Ticking,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    PreviousLevel,
    TownPortal,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
    ShowCheatMenu,
    ShowHelpMenu,
    ShowVendor {
        vendor: Entity,
        mode: VendorMode,
    },
    TeleportingToOtherLevel {
        x: i32,
        y: i32,
        depth: i32,
    },
    ShowRemoveCurse,
    ShowIdentify,
    ShowOptionMenu,
    FinishGame,
    SpawnWave,
    SpawnHordeMode,
}

pub struct State {
    pub ecs: World,
    pub mapgen_next_state: Option<RunState>,
    pub mapgen_history: Vec<Map>,
    pub mapgen_index: usize,
    pub mapgen_timer: f32,
    pub dispatcher: Box<dyn systems::UnifiedDispatcher + 'static>,
}

impl State {
    fn run_systems(&mut self) {
        self.dispatcher.run_now(&mut self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    #[allow(clippy::cognitive_complexity)]
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();
        systems::particle_system::update_particles(&mut self.ecs, ctx);

        match newrunstate {
            RunState::MainMenu { .. } => {}
            RunState::GameOver { .. } => {}
            RunState::FinishGame { .. } => {}
            _ => {
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
        }

        match newrunstate {
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    newrunstate = self.mapgen_next_state.unwrap();
                } else {
                    ctx.cls();
                    if self.mapgen_index < self.mapgen_history.len()
                        && self.mapgen_index < self.mapgen_history.len()
                    {
                        camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);
                    }

                    self.mapgen_timer += ctx.frame_time_ms;
                    if self.mapgen_timer > 250.0 {
                        self.mapgen_timer = 0.0;
                        self.mapgen_index += 1;
                        if self.mapgen_index >= self.mapgen_history.len() {
                            //self.mapgen_index -= 1;
                            newrunstate = self.mapgen_next_state.unwrap();
                        }
                    }
                }
            }
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player::player_input(self, ctx);
                if newrunstate != RunState::AwaitingInput {
                    crate::gamelog::record_event("Turn", 1);
                }
            }
            RunState::Ticking => {
                let mut should_change_target = false;

                while newrunstate == RunState::Ticking {
                    self.run_systems();

                    // all these is meant to avoid a infinite loop (while) when change state in systems
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => {
                            newrunstate = RunState::AwaitingInput;
                            should_change_target = true;
                        }
                        RunState::MagicMapReveal { .. } => {
                            newrunstate = RunState::MagicMapReveal { row: 0 }
                        }
                        RunState::TownPortal => newrunstate = RunState::TownPortal,
                        RunState::TeleportingToOtherLevel { x, y, depth } => {
                            newrunstate = RunState::TeleportingToOtherLevel { x, y, depth }
                        }
                        RunState::ShowRemoveCurse => newrunstate = RunState::ShowRemoveCurse,
                        RunState::ShowIdentify => newrunstate = RunState::ShowIdentify,
                        RunState::FinishGame => newrunstate = RunState::FinishGame,
                        RunState::SpawnWave => newrunstate = RunState::SpawnWave,
                        RunState::SpawnHordeMode => newrunstate = RunState::SpawnHordeMode,
                        _ => newrunstate = RunState::Ticking,
                    }
                }
                if should_change_target {
                    player::end_turn_targeting(&mut self.ecs);
                }
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting {
                                range: is_item_ranged.range,
                                item: item_entity,
                            };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::ShowOptionMenu => {
                let result = gui::show_option_menu(self, ctx);
                match result {
                    gui::OptionMenuResult::Continue => newrunstate = RunState::AwaitingInput,
                    gui::OptionMenuResult::NoResponse => {}
                    gui::OptionMenuResult::ExitWithoutSaving => {
                        self.game_over_cleanup(true);
                        newrunstate = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        };
                    }
                    gui::OptionMenuResult::SaveAndExit => newrunstate = RunState::SaveGame,
                }
            }
            RunState::ShowHelpMenu => {
                let result = gui::show_help_menu(self, ctx);
                match result {
                    gui::HelpMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    _ => {}
                }
            }
            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_mode(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::TeleportToExit => {
                        self.goto_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        newrunstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::Heal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::Reveal => {
                        let mut map = self.ecs.fetch_mut::<Map>();
                        for v in map.revealed_tiles.iter_mut() {
                            *v = true;
                        }
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.god_mode = true;
                        newrunstate = RunState::AwaitingInput;
                    }

                    gui::CheatMenuResult::ListSpawnedMobs => {
                        let initiatives = self.ecs.read_storage::<Initiative>();
                        let entities = self.ecs.entities();
                        let positions = self.ecs.read_storage::<Position>();

                        let mut filtered_mobs: Vec<String> = Vec::new();

                        for (entity, _initiative, _pos) in
                            (&entities, &initiatives, &positions).join()
                        {
                            filtered_mobs.push(gui::get_item_display_name(&self.ecs, entity))
                        }

                        bracket_lib::prelude::console::log(format!(
                            "{}.",
                            filtered_mobs.join(", ")
                        ));
                        crate::gamelog::Logger::new()
                            .color(bracket_lib::terminal::GRAY)
                            .append(format!(
                                "[DEBUG] Enemies in current map: {:?}",
                                filtered_mobs.join(", ")
                            ))
                            .log();
                        newrunstate = RunState::AwaitingInput;
                    }

                    gui::CheatMenuResult::ListSpawnedItems => {
                        let items = self.ecs.read_storage::<Item>();
                        let entities = self.ecs.entities();
                        let positions = self.ecs.read_storage::<Position>();

                        let mut filtered_items: Vec<String> = Vec::new();

                        for (entity, _item, _pos) in (&entities, &items, &positions).join() {
                            filtered_items.push(gui::get_item_display_name(&self.ecs, entity));
                        }

                        bracket_lib::prelude::console::log(format!(
                            "{}.",
                            filtered_items.join(", ")
                        ));
                        crate::gamelog::Logger::new()
                            .color(bracket_lib::terminal::GRAY)
                            .append(format!(
                                "[DEBUG] Items in current map: {:?}",
                                filtered_items.join(", ")
                            ))
                            .log();

                        newrunstate = RunState::AwaitingInput;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveCurse => {
                let result = gui::remove_curse_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        self.ecs.write_storage::<CursedItem>().remove(item_entity);
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowIdentify => {
                let result = gui::identify_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        if let Some(name) = self.ecs.read_storage::<Name>().get(item_entity) {
                            let mut dm = self.ecs.fetch_mut::<MasterDungeonMap>();
                            dm.identified_items.insert(name.name.clone());
                        }
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        if self.ecs.read_storage::<SpellTemplate>().get(item).is_some() {
                            let mut intent = self.ecs.write_storage::<WantsToCastSpell>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToCastSpell {
                                        spell: item,
                                        target: result.1,
                                    },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item,
                                        target: result.1,
                                    },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::ShowVendor { vendor, mode } => {
                use crate::raws::*;
                let result = gui::show_vendor_menu(self, ctx, vendor, mode);
                match result.0 {
                    gui::VendorResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::VendorResult::NoResponse => {}
                    gui::VendorResult::Sell => {
                        let price = self
                            .ecs
                            .read_storage::<Item>()
                            .get(result.1.unwrap())
                            .unwrap()
                            .base_value
                            * 0.8;
                        self.ecs
                            .write_storage::<Pools>()
                            .get_mut(*self.ecs.fetch::<Entity>())
                            .unwrap()
                            .gold += price;
                        self.ecs
                            .delete_entity(result.1.unwrap())
                            .expect("Unable to delete");
                    }
                    gui::VendorResult::Buy => {
                        let tag = result.2.unwrap();
                        let price = result.3.unwrap();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_entity = self.ecs.fetch::<Entity>();
                        let mut identified = self.ecs.write_storage::<IdentifiedItem>();
                        identified
                            .insert(*player_entity, IdentifiedItem { name: tag.clone() })
                            .expect("Unable to insert");
                        std::mem::drop(identified);
                        let player_pools = pools.get_mut(*player_entity).unwrap();
                        std::mem::drop(player_entity);
                        if player_pools.gold >= price {
                            player_pools.gold -= price;
                            std::mem::drop(pools);
                            let player_entity = *self.ecs.fetch::<Entity>();
                            crate::raws::spawn_named_item(
                                &RAWS.lock().unwrap(),
                                &mut self.ecs,
                                &tag,
                                SpawnType::Carried { by: player_entity },
                            );
                        }
                    }
                    gui::VendorResult::BuyMode => {
                        newrunstate = RunState::ShowVendor {
                            vendor,
                            mode: VendorMode::Buy,
                        }
                    }
                    gui::VendorResult::SellMode => {
                        newrunstate = RunState::ShowVendor {
                            vendor,
                            mode: VendorMode::Sell,
                        }
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        newrunstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => {
                            self.game_over_cleanup(false);
                            newrunstate = RunState::PreRun
                        }
                        gui::MainMenuSelection::LoadGame => {
                            saveload::load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                            saveload::delete_save();
                        }
                        gui::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::FinishGame => {
                let result = gui::finish_game(ctx);
                match result {
                    gui::FinishGameResult::NoSelection => {}
                    gui::FinishGameResult::QuitToMenu => {
                        self.game_over_cleanup(true);
                        newrunstate = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        };
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup(true);
                        newrunstate = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        };
                    }
                }
            }
            RunState::SaveGame => {
                saveload::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
                };
            }
            RunState::NextLevel => {
                self.goto_level(1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::PreviousLevel => {
                self.goto_level(-1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::TownPortal => {
                // Spawn the portal
                spawner::spawn_town_portal(&mut self.ecs);

                // Transition
                let map_depth = self.ecs.fetch::<Map>().depth;
                let destination_offset = 0 - (map_depth - 1);
                self.goto_level(destination_offset);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::TeleportingToOtherLevel { x, y, depth } => {
                self.goto_level(depth - 1);
                let player_entity = self.ecs.fetch::<Entity>();
                if let Some(pos) = self.ecs.write_storage::<Position>().get_mut(*player_entity) {
                    pos.x = x;
                    pos.y = y;
                }
                let mut ppos = self.ecs.fetch_mut::<bracket_lib::prelude::Point>();
                ppos.x = x;
                ppos.y = y;
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..map.width {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row == map.height - 1 {
                    newrunstate = RunState::Ticking;
                } else {
                    newrunstate = RunState::MagicMapReveal { row: row + 1 };
                }
            }
            RunState::SpawnWave => {
                spawner::spawn_horde_mobs_by_depth(&mut self.ecs, crate::raws::SpawnTableType::Mob);
                newrunstate = RunState::Ticking;
            }
            RunState::SpawnHordeMode => {
                spawner::spawn_horde_mode(&mut self.ecs);
                newrunstate = RunState::Ticking;
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage::delete_the_dead(&mut self.ecs);

        let _ = bracket_lib::prelude::render_draw_buffer(ctx);
        if SHOW_FPS {
            ctx.print(1, 59, &format!("FPS: {}", ctx.fps));
        }

        ctx.print(1, 59, &format!("press <Space>: for help "));

        if SHOW_SEED {
            ctx.print(
                10,
                59,
                &format!("Seed: {}", crate::rng::get_current_seed().to_string()),
            );
        }

        if SHOW_DEPTH {
            let current_depth = self.ecs.fetch::<Map>().depth;
            ctx.print(30, 59, &format!("Depth: {}", current_depth));
        }
    }
}

impl State {
    fn goto_level(&mut self, offset: i32) {
        freeze_level_entities(&mut self.ecs);

        // Build a new map and place the player
        let current_depth = self.ecs.fetch::<Map>().depth;
        self.generate_world_map(current_depth + offset, offset);
        // Notify the player
        gamelog::Logger::new().append("You change level.").log();
    }

    fn game_over_cleanup(&mut self, only_clean_up: bool) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        if only_clean_up {
            return;
        }

        // Spawn a new player
        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        // Replace the world maps
        self.ecs.insert(map::MasterDungeonMap::new());

        // Build a new map and place the player
        self.generate_world_map(1, 0);
    }

    fn generate_world_map(&mut self, new_depth: i32, offset: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let map_building_info = map::level_transition(&mut self.ecs, new_depth, offset);
        if let Some(history) = map_building_info {
            self.mapgen_history = history;
        } else {
            map::thaw_level_entities(&mut self.ecs);
        }
        gamelog::clear_log();
        gamelog::Logger::new()
            .append("Welcome to")
            .color(bracket_lib::terminal::CYAN)
            .append(PROJECT_NAME)
            .log();

        gamelog::clear_events();
    }
}
