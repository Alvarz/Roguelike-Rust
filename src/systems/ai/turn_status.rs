use crate::{
    effects::add_effect, effects::EffectType, effects::Targets, Confusion, MyTurn, RunState,
    StatusEffect,
};
use specs::prelude::*;
use std::collections::HashSet;

pub struct TurnStatusSystem {}

impl<'a> System<'a> for TurnStatusSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        ReadStorage<'a, Confusion>,
        Entities<'a>,
        ReadExpect<'a, RunState>,
        ReadStorage<'a, StatusEffect>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, confusion, entities, runstate, statuses) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        // Collect a set of all entities whose turn it is
        let mut entity_turns = HashSet::new();
        for (entity, _turn) in (&entities, &turns).join() {
            entity_turns.insert(entity);
        }

        // Find status effects affecting entities whose turn it is
        let mut not_my_turn: Vec<Entity> = Vec::new();
        for (effect_entity, status_effect) in (&entities, &statuses).join() {
            if entity_turns.contains(&status_effect.target) {
                // Skip turn for confusion
                if confusion.get(effect_entity).is_some() {
                    add_effect(
                        None,
                        EffectType::Particle {
                            glyph: bracket_lib::prelude::to_cp437('?'),
                            fg: bracket_lib::prelude::RGB::named(bracket_lib::terminal::CYAN),
                            bg: bracket_lib::prelude::RGB::named(bracket_lib::terminal::BLACK),
                            lifespan: 200.0,
                        },
                        Targets::Single {
                            target: status_effect.target,
                        },
                    );
                    not_my_turn.push(status_effect.target);
                }
            }
        }

        for e in not_my_turn {
            turns.remove(e);
        }
    }
}
