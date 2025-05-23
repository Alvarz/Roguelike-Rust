use super::*;
use crate::attributes::{mana_at_level, player_hp_at_level};
use crate::map::Map;
use crate::{
    Attributes, Confusion, DamageOverTime, Duration, EquipmentChanged, Name, Player, Pools,
    SerializeMe, Skills, Slow, StatusEffect,
};
use specs::saveload::{MarkedBuilder, SimpleMarker};

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    let player_entity = ecs.fetch::<Entity>();
    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
            if let Some(creator) = damage.creator {
                if creator == target {
                    return;
                }
            }
            if let EffectType::Damage { amount } = damage.effect_type {
                pool.hit_points.current -= amount;
                add_effect(None, EffectType::Bloodstain, Targets::Single { target });
                add_effect(
                    None,
                    EffectType::Particle {
                        glyph: bracket_lib::prelude::to_cp437('‼'),
                        fg: bracket_lib::prelude::RGB::named(bracket_lib::terminal::ORANGE),
                        bg: bracket_lib::prelude::RGB::named(bracket_lib::terminal::BLACK),
                        lifespan: 200.0,
                    },
                    Targets::Single { target },
                );
                if target == *player_entity {
                    crate::gamelog::record_event("Damage Taken", amount);
                }
                if let Some(creator) = damage.creator {
                    if creator == *player_entity {
                        crate::gamelog::record_event("Damage Inflicted", amount);
                    }
                }

                if pool.hit_points.current < 1 {
                    add_effect(
                        damage.creator,
                        EffectType::EntityDeath,
                        Targets::Single { target },
                    );
                }
            }
        }
    }
}

pub fn bloodstain(ecs: &mut World, tile_idx: i32) {
    let mut map = ecs.fetch_mut::<Map>();
    map.bloodstains.insert(tile_idx as usize);
}

pub fn death(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let mut xp_gain = 0;
    let mut gold_gain = 0.0f32;

    let mut pools = ecs.write_storage::<Pools>();
    let mut attributes = ecs.write_storage::<Attributes>();

    if let Some(pos) = entity_position(ecs, target) {
        crate::spatial::remove_entity(target, pos as usize);
    }

    if let Some(source) = effect.creator {
        if ecs.read_storage::<Player>().get(source).is_some() {
            if let Some(stats) = pools.get(target) {
                xp_gain += stats.level * 100;
                gold_gain += stats.gold;
            }

            if xp_gain != 0 || gold_gain != 0.0 {
                let player_stats = pools.get_mut(source).unwrap();
                let player_attributes = attributes.get_mut(source).unwrap();
                player_stats.xp += xp_gain;
                player_stats.gold += gold_gain;
                if player_stats.xp >= player_stats.level * 1000 {
                    // We've gone up a level!
                    player_stats.level += 1;
                    crate::gamelog::Logger::new()
                        .color(bracket_lib::terminal::MAGENTA)
                        .append("Congratulations, you are now level")
                        .append(format!("{}", player_stats.level))
                        .log();

                    // Improve a random attribute
                    let attr_to_boost = crate::rng::roll_dice(1, 4);
                    match attr_to_boost {
                        1 => {
                            player_attributes.might.base += 1;
                            crate::gamelog::Logger::new()
                                .color(bracket_lib::terminal::GREEN)
                                .append("You feel stronger!")
                                .log();
                        }
                        2 => {
                            player_attributes.fitness.base += 1;
                            crate::gamelog::Logger::new()
                                .color(bracket_lib::terminal::GREEN)
                                .append("You feel healthier!")
                                .log();
                        }
                        3 => {
                            player_attributes.quickness.base += 1;
                            crate::gamelog::Logger::new()
                                .color(bracket_lib::terminal::GREEN)
                                .append("You feel quicker!")
                                .log();
                        }
                        _ => {
                            player_attributes.intelligence.base += 1;
                            crate::gamelog::Logger::new()
                                .color(bracket_lib::terminal::GREEN)
                                .append("You feel smarter!")
                                .log();
                        }
                    }

                    // Improve all skills
                    let mut skills = ecs.write_storage::<Skills>();
                    let player_skills = skills.get_mut(*ecs.fetch::<Entity>()).unwrap();
                    for sk in player_skills.skills.iter_mut() {
                        *sk.1 += 1;
                    }

                    ecs.write_storage::<EquipmentChanged>()
                        .insert(*ecs.fetch::<Entity>(), EquipmentChanged {})
                        .expect("Insert Failed");

                    player_stats.hit_points.max = player_hp_at_level(
                        player_attributes.fitness.base + player_attributes.fitness.modifiers,
                        player_stats.level,
                    );
                    player_stats.hit_points.current = player_stats.hit_points.max;
                    player_stats.mana.max = mana_at_level(
                        player_attributes.intelligence.base
                            + player_attributes.intelligence.modifiers,
                        player_stats.level,
                    );
                    player_stats.mana.current = player_stats.mana.max;

                    let player_pos = ecs.fetch::<bracket_lib::prelude::Point>();
                    let map = ecs.fetch::<Map>();
                    for i in 0..10 {
                        if player_pos.y - i > 1 {
                            add_effect(
                                None,
                                EffectType::Particle {
                                    glyph: bracket_lib::prelude::to_cp437('░'),
                                    fg: bracket_lib::prelude::RGB::named(
                                        bracket_lib::terminal::GOLD,
                                    ),
                                    bg: bracket_lib::prelude::RGB::named(
                                        bracket_lib::terminal::BLACK,
                                    ),
                                    lifespan: 400.0,
                                },
                                Targets::Tile {
                                    tile_idx: map.xy_idx(player_pos.x, player_pos.y - i) as i32,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn heal_damage(ecs: &mut World, heal: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Healing { amount } = heal.effect_type {
            pool.hit_points.current =
                i32::min(pool.hit_points.max, pool.hit_points.current + amount);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: bracket_lib::prelude::to_cp437('‼'),
                    fg: bracket_lib::prelude::RGB::named(bracket_lib::terminal::GREEN),
                    bg: bracket_lib::prelude::RGB::named(bracket_lib::terminal::BLACK),
                    lifespan: 200.0,
                },
                Targets::Single { target },
            );
        }
    }
}

pub fn restore_mana(ecs: &mut World, mana: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Mana { amount } = mana.effect_type {
            pool.mana.current = i32::min(pool.mana.max, pool.mana.current + amount);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: bracket_lib::prelude::to_cp437('‼'),
                    fg: bracket_lib::prelude::RGB::named(bracket_lib::terminal::BLUE),
                    bg: bracket_lib::prelude::RGB::named(bracket_lib::terminal::BLACK),
                    lifespan: 200.0,
                },
                Targets::Single { target },
            );
        }
    }
}

pub fn add_confusion(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Confusion { turns } = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(Confusion {})
            .with(Duration { turns: *turns })
            .with(Name {
                name: "Confusion".to_string(),
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
    }
}

pub fn attribute_effect(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::AttributeEffect {
        bonus,
        name,
        duration,
    } = &effect.effect_type
    {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(bonus.clone())
            .with(Duration { turns: *duration })
            .with(Name { name: name.clone() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<EquipmentChanged>()
            .insert(target, EquipmentChanged {})
            .expect("Insert failed");
    }
}

pub fn slow(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Slow { initiative_penalty } = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(Slow {
                initiative_penalty: *initiative_penalty,
            })
            .with(Duration { turns: 5 })
            .with(if *initiative_penalty > 0.0 {
                Name {
                    name: "Slowed".to_string(),
                }
            } else {
                Name {
                    name: "Hasted".to_string(),
                }
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
    }
}

pub fn damage_over_time(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::DamageOverTime { damage } = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(DamageOverTime { damage: *damage })
            .with(Duration { turns: 5 })
            .with(Name {
                name: "Damage Over Time".to_string(),
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
    }
}
