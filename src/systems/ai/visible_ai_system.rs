use crate::{
    raws::Reaction, Chasing, Equipped, Faction, Map, MyTurn, Name, Position, SpecialAbilities,
    SpellTemplate, Viewshed, WantsToApproach, WantsToCastSpell, WantsToFlee, WantsToShoot, Weapon,
};
use specs::prelude::*;

pub struct VisibleAI {}

impl<'a> System<'a> for VisibleAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, MyTurn>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, WantsToFlee>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Viewshed>,
        WriteStorage<'a, Chasing>,
        ReadStorage<'a, SpecialAbilities>,
        WriteStorage<'a, WantsToCastSpell>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, SpellTemplate>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, Weapon>,
        WriteStorage<'a, WantsToShoot>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            turns,
            factions,
            positions,
            map,
            mut want_approach,
            mut want_flee,
            entities,
            player,
            viewsheds,
            mut chasing,
            abilities,
            mut casting,
            names,
            spells,
            equipped,
            weapons,
            mut wants_shoot,
        ) = data;

        for (entity, _turn, my_faction, pos, viewshed) in
            (&entities, &turns, &factions, &positions, &viewsheds).join()
        {
            if entity != *player {
                let my_idx = map.xy_idx(pos.x, pos.y);
                let mut reactions: Vec<(usize, Reaction, Entity)> = Vec::new();
                let mut flee: Vec<usize> = Vec::new();
                for visible_tile in viewshed.visible_tiles.iter() {
                    let idx = map.xy_idx(visible_tile.x, visible_tile.y);
                    if my_idx != idx {
                        evaluate(idx, &map, &factions, &my_faction.name, &mut reactions);
                    }
                }

                let mut done = false;
                for reaction in reactions.iter() {
                    match reaction.1 {
                        Reaction::Attack => {
                            let range = bracket_lib::prelude::DistanceAlg::Chebyshev.distance2d(
                                bracket_lib::prelude::Point::new(pos.x, pos.y),
                                bracket_lib::prelude::Point::new(
                                    reaction.0 as i32 % map.width,
                                    reaction.0 as i32 / map.width,
                                ),
                            );
                            if let Some(abilities) = abilities.get(entity) {
                                for ability in abilities.abilities.iter() {
                                    if range >= ability.min_range
                                        && range <= ability.range
                                        && crate::rng::roll_dice(1, 100)
                                            <= (ability.chance * 100.0) as i32
                                    {
                                        use crate::raws::find_spell_entity_by_name;
                                        casting
                                            .insert(
                                                entity,
                                                WantsToCastSpell {
                                                    spell: find_spell_entity_by_name(
                                                        &ability.spell,
                                                        &names,
                                                        &spells,
                                                        &entities,
                                                    )
                                                    .unwrap(),
                                                    target: Some(bracket_lib::prelude::Point::new(
                                                        reaction.0 as i32 % map.width,
                                                        reaction.0 as i32 / map.width,
                                                    )),
                                                },
                                            )
                                            .expect("Unable to insert");
                                        done = true;
                                    }
                                }
                            }

                            if !done {
                                for (weapon, equip) in (&weapons, &equipped).join() {
                                    if let Some(wrange) = weapon.range {
                                        if equip.owner == entity {
                                            if wrange >= range as i32 {
                                                wants_shoot
                                                    .insert(
                                                        entity,
                                                        WantsToShoot { target: reaction.2 },
                                                    )
                                                    .expect("Insert fail");
                                                done = true;
                                            }
                                        }
                                    }
                                }
                            }

                            if !done {
                                want_approach
                                    .insert(
                                        entity,
                                        WantsToApproach {
                                            idx: reaction.0 as i32,
                                        },
                                    )
                                    .expect("Unable to insert");
                                chasing
                                    .insert(entity, Chasing { target: reaction.2 })
                                    .expect("Unable to insert");
                                done = true;
                            }
                        }
                        Reaction::Flee => {
                            flee.push(reaction.0);
                        }
                        _ => {}
                    }
                }

                if !done && !flee.is_empty() {
                    want_flee
                        .insert(entity, WantsToFlee { indices: flee })
                        .expect("Unable to insert");
                }
            }
        }
    }
}

fn evaluate(
    idx: usize,
    _map: &Map,
    factions: &ReadStorage<Faction>,
    my_faction: &str,
    reactions: &mut Vec<(usize, Reaction, Entity)>,
) {
    crate::spatial::for_each_tile_content(idx, |other_entity| {
        if let Some(faction) = factions.get(other_entity) {
            reactions.push((
                idx,
                crate::raws::faction_reaction(
                    my_faction,
                    &faction.name,
                    &crate::raws::RAWS.lock().unwrap(),
                ),
                other_entity,
            ));
        }
    });
}
