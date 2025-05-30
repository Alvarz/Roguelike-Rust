use super::{get_item_display_name, item_result_menu, ItemMenuResult};
use crate::{Equipped, InBackpack, Item, MasterDungeonMap, Name, ObfuscatedName, State};
use bracket_lib::prelude::*;
use specs::prelude::*;

pub fn identify_menu(gs: &mut State, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();

    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let item_components = gs.ecs.read_storage::<Item>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();
    let obfuscated = gs.ecs.read_storage::<ObfuscatedName>();

    let mut items: Vec<(Entity, String)> = Vec::new();
    (&entities, &item_components)
        .join()
        .filter(|(item_entity, _item)| {
            let mut keep = false;
            if let Some(bp) = backpack.get(*item_entity) {
                if bp.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        if obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name)
                        {
                            keep = true;
                        }
                    }
                }
            }
            // It's equipped, so we know it's cursed
            if let Some(equip) = equipped.get(*item_entity) {
                if equip.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        if obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name)
                        {
                            keep = true;
                        }
                    }
                }
            }
            keep
        })
        .for_each(|item| items.push((item.0, get_item_display_name(&gs.ecs, item.0))));

    let result = item_result_menu(&mut draw_batch, "Inventory", items.len(), &items, ctx.key);
    let _ = draw_batch.submit(6000);
    result
}
