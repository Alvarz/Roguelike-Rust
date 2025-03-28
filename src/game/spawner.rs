use crate::attributes::{attr_bonus, mana_at_level, player_hp_at_level};
use crate::spatial::is_blocked;
use crate::{
    random_table::MasterTable, raws::*, Attribute, AttributeBonus, Attributes, Duration,
    EntryTrigger, EquipmentChanged, Faction, HungerClock, HungerState, Initiative, KnownSpells,
    LightSource, Map, MasterDungeonMap, Name, OtherLevelPosition, Player, Pool, Pools, Position,
    Rect, Renderable, SerializeMe, SingleActivation, Skill, Skills, StatusEffect, TeleportTo,
    TileType, Viewshed,
};
use crate::{rng, tile_walkable, MAX_MONSTERS_BY_WAVE, MIN_MONSTERS_BY_WAVE};
use bracket_lib::prelude::RGB;
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

use super::{Chasing, HordeMember, HordeMode};

/// Spawns the player and returns his/her entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    spawn_all_spells(ecs);

    let mut skills = Skills {
        skills: HashMap::new(),
    };
    skills.skills.insert(Skill::Melee, 1);
    skills.skills.insert(Skill::Defense, 1);
    skills.skills.insert(Skill::Magic, 1);

    let player = ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('@'),
            fg: RGB::named(bracket_lib::terminal::YELLOW),
            bg: RGB::named(bracket_lib::terminal::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(HungerClock {
            state: HungerState::WellFed,
            duration: 20,
        })
        .with(Attributes {
            might: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attr_bonus(11),
            },
            fitness: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attr_bonus(11),
            },
            quickness: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attr_bonus(11),
            },
            intelligence: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attr_bonus(11),
            },
        })
        .with(skills)
        .with(Pools {
            hit_points: Pool {
                current: player_hp_at_level(11, 1),
                max: player_hp_at_level(11, 1),
            },
            mana: Pool {
                current: mana_at_level(11, 1),
                max: mana_at_level(11, 1),
            },
            xp: 0,
            level: 1,
            total_weight: 0.0,
            total_initiative_penalty: 0.0,
            gold: 0.0,
            god_mode: false,
        })
        .with(EquipmentChanged {})
        .with(LightSource {
            color: bracket_lib::prelude::RGB::from_f32(1.0, 1.0, 0.5),
            range: 8,
        })
        .with(Initiative { current: 0 })
        .with(Faction {
            name: "Player".to_string(),
        })
        .with(KnownSpells { spells: Vec::new() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Starting equipment
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Rusty Longsword",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Dried Sausage",
        SpawnType::Carried { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Beer",
        SpawnType::Carried { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Stained Tunic",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Torn Trousers",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Old Boots",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Shortbow",
        SpawnType::Carried { by: player },
    );

    // Starting hangover
    ecs.create_entity()
        .with(StatusEffect { target: player })
        .with(Duration { turns: 10 })
        .with(Name {
            name: "Hangover".to_string(),
        })
        .with(AttributeBonus {
            might: Some(-1),
            fitness: None,
            quickness: Some(-1),
            intelligence: Some(-1),
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    player
}

const MAX_MONSTERS: i32 = 4;

pub fn room_table(map_depth: i32) -> MasterTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(), map_depth)
}

/// Fills a room with stuff!
pub fn spawn_room(map: &Map, room: &Rect, map_depth: i32, spawn_list: &mut Vec<(usize, String)>) {
    let mut possible_targets: Vec<usize> = Vec::new();
    {
        // Borrow scope - to keep access to the map separated
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(map, &possible_targets, map_depth, spawn_list);
}

/// Fills a region with stuff!
pub fn spawn_region(
    _map: &Map,
    area: &[usize],
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        let num_spawns = i32::min(
            areas.len() as i32,
            crate::rng::roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3,
        );
        if num_spawns == 0 {
            return;
        }

        for _i in 0..num_spawns {
            let array_index = if areas.len() == 1 {
                0usize
            } else {
                (crate::rng::roll_dice(1, areas.len() as i32) - 1) as usize
            };

            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll());
            areas.remove(array_index);
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) -> Option<Entity> {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    let spawn_result = spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        &spawn.1,
        SpawnType::AtPosition { x, y },
    );
    if spawn_result.is_some() {
        return spawn_result;
    }

    if spawn.1 != "None" {
        bracket_lib::prelude::console::log(format!(
            "WARNING: We don't know how to spawn [{}]!",
            spawn.1
        ));
    }
    None
}

pub fn spawn_town_portal(ecs: &mut World) {
    // Get current position & depth
    let map = ecs.fetch::<Map>();
    let player_depth = map.depth;
    let player_pos = ecs.fetch::<bracket_lib::prelude::Point>();
    let player_x = player_pos.x;
    let player_y = player_pos.y;
    std::mem::drop(player_pos);
    std::mem::drop(map);

    // Find part of the town for the portal
    let dm = ecs.fetch::<MasterDungeonMap>();
    let town_map = dm.get_map(1).unwrap();
    let mut stairs_idx = 0;
    for (idx, tt) in town_map.tiles.iter().enumerate() {
        if *tt == TileType::DownStairs {
            stairs_idx = idx;
        }
    }
    // let portal_x = (stairs_idx as i32 % town_map.width) - 2;
    // let portal_y = stairs_idx as i32 / town_map.width;
    let (portal_x, portal_y) = town_map.idx_xy(stairs_idx);

    std::mem::drop(dm);

    // Spawn the portal itself
    ecs.create_entity()
        .with(OtherLevelPosition {
            x: portal_x,
            y: portal_y,
            depth: 1,
        })
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('♥'),
            fg: RGB::named(bracket_lib::terminal::CYAN),
            bg: RGB::named(bracket_lib::terminal::BLACK),
            render_order: 0,
        })
        .with(EntryTrigger {})
        .with(TeleportTo {
            x: player_x,
            y: player_y,
            depth: player_depth,
            player_only: true,
        })
        .with(SingleActivation {})
        .with(Name {
            name: "Town Portal".to_string(),
        })
        .build();
}

pub fn spawn_horde_mobs_by_depth(ecs: &mut World, table_type: SpawnTableType) {
    let map = ecs.get_mut::<crate::map::Map>().unwrap().clone();
    let spawn_list: &mut Vec<(usize, String)> = &mut Vec::new();

    let max_spawn = rng::range(MIN_MONSTERS_BY_WAVE, MAX_MONSTERS_BY_WAVE);
    let mut current_spawn = 0;
    let mut spawn_table = room_table(map.depth);

    while current_spawn < max_spawn {
        let x = crate::rng::range(0, map.width - 1);
        let y = crate::rng::range(0, map.height - 1);
        let map_idx = map.xy_idx(x, y);

        if can_spawn_at_position(map_idx, ecs, x, y) {
            spawn_list.push((map_idx, spawn_table.roll_by_type(table_type.clone())));
            current_spawn += 1;
        }
    }
    for entity in spawn_list.iter() {
        let entity = spawn_entity(ecs, &(&entity.0, &entity.1)).unwrap();
        add_horde_member_components_to_entity(entity, ecs);
    }
}

pub fn add_horde_member_components_to_entity(entity: Entity, ecs: &mut World) {
    let player_entity = ecs.fetch::<Entity>();
    let mut horde_members = ecs.write_storage::<HordeMember>();
    let mut chasing = ecs.write_storage::<Chasing>();

    // debug horde members changing their color
    // let mut renderers = ecs.write_storage::<Renderable>();
    // let entity_renderer = renderers.get_mut(entity);
    // if let Some(entity_renderer) = entity_renderer {
    //     entity_renderer.bg = bracket_lib::prelude::RGB::named(bracket_lib::REBECCAPURPLE);
    // }

    let _ = horde_members.insert(entity, HordeMember {});
    let _ = chasing.insert(
        entity,
        Chasing {
            target: *player_entity,
        },
    );
}

pub fn spawn_horde_mode(ecs: &mut World) {
    ecs.create_entity()
        .with(Position { x: 1, y: 1 })
        .with(HordeMode {
            state: super::WaveState::WaitingToStart { turns_left: 1 },
        })
        .with(Initiative { current: 1 })
        .build();
}

fn can_spawn_at_position(map_idx: usize, ecs: &mut World, x: i32, y: i32) -> bool {
    let map = ecs.get_mut::<crate::map::Map>().unwrap().clone();
    let player_pos = ecs.fetch::<bracket_lib::prelude::Point>();
    let distance =
        bracket_lib::prelude::DistanceAlg::Manhattan.distance2d(*player_pos, bracket_lib::prelude::Point::new(x, y));

    return !is_blocked(map_idx)
        && tile_walkable(map.tiles[map_idx])
        && !map.visible_tiles[map_idx]
        && distance > 18.0
        && distance < 20.0;
}
