use super::{EffectSpawner, EffectType};
use crate::AmuletOfYendor;
use specs::{Entity, World, WorldExt};

pub fn add_amulet_of_yendor(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::AmuletOfYendor = &effect.effect_type {
        ecs.write_storage::<AmuletOfYendor>()
            .insert(target, AmuletOfYendor {})
            .expect("Unable to insert status");
    }
}
