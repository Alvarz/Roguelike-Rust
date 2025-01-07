use crate::{UseOnPickUp, WantsToUseItem};

use super::{
    EquipmentChanged, InBackpack, MagicItem, MasterDungeonMap, Name, ObfuscatedName, Position,
    WantsToPickupItem,
};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, MagicItem>,
        ReadStorage<'a, ObfuscatedName>,
        ReadExpect<'a, MasterDungeonMap>,
        WriteStorage<'a, UseOnPickUp>,
        WriteStorage<'a, WantsToUseItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut wants_pickup,
            mut positions,
            names,
            mut backpack,
            mut dirty,
            magic_items,
            obfuscated_names,
            dm,
            mut use_on_pickup,
            mut wants_to_use,
        ) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");
            dirty
                .insert(pickup.collected_by, EquipmentChanged {})
                .expect("Unable to insert");

            if pickup.collected_by == *player_entity {
                crate::gamelog::Logger::new()
                    .append("You pick up the")
                    .item_name(super::obfuscate_name(
                        pickup.item,
                        &names,
                        &magic_items,
                        &obfuscated_names,
                        &dm,
                    ))
                    .log();
            }

            if use_on_pickup.get(pickup.item).is_some() {
                let _ = wants_to_use.insert(
                    pickup.collected_by,
                    WantsToUseItem {
                        item: pickup.item,
                        target: None,
                    },
                );
                use_on_pickup.remove(pickup.item);
            }
        }

        wants_pickup.clear();
    }
}
