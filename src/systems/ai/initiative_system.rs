use crate::{
    Attributes, DamageOverTime, Duration, EquipmentChanged, HordeMember, HordeMode, Initiative,
    MyTurn, Pools, Position, RunState, StatusEffect,
};
use specs::prelude::*;

pub struct InitiativeSystem {}

impl<'a> System<'a> for InitiativeSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, Initiative>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MyTurn>,
        Entities<'a>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, bracket_lib::prelude::Point>,
        ReadStorage<'a, Pools>,
        WriteStorage<'a, Duration>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, StatusEffect>,
        ReadStorage<'a, DamageOverTime>,
        ReadStorage<'a, HordeMode>,
        ReadStorage<'a, HordeMember>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut initiatives,
            positions,
            mut turns,
            entities,
            attributes,
            mut runstate,
            player,
            player_pos,
            pools,
            mut durations,
            mut dirty,
            statuses,
            dots,
            horde_modes,
            horde_members,
        ) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        // Clear any remaining MyTurn we left by mistkae
        turns.clear();

        // Roll initiative
        for (entity, initiative, pos) in (&entities, &mut initiatives, &positions).join() {
            initiative.current -= 1;
            if initiative.current < 1 {
                let mut myturn = true;

                // Re-roll
                initiative.current = 6 + crate::rng::roll_dice(1, 6);

                // Give a bonus for quickness
                if let Some(attr) = attributes.get(entity) {
                    initiative.current -= attr.quickness.bonus;
                }

                // Apply pool penalty
                if let Some(pools) = pools.get(entity) {
                    initiative.current += f32::floor(pools.total_initiative_penalty) as i32;
                }

                // TODO: More initiative granting boosts/penalties will go here later

                // If its the player, we want to go to an AwaitingInput state
                if entity == *player {
                    // Give control to the player
                    *runstate = RunState::AwaitingInput;
                } else {
                    // if it is a horde mode entity or a horde member, we don't care about it's distance
                    if !horde_modes.contains(entity) && !horde_members.contains(entity) {
                        let distance = bracket_lib::prelude::DistanceAlg::PythagorasSquared
                            .distance2d(*player_pos, bracket_lib::prelude::Point::new(pos.x, pos.y));
                        if distance > 20.0 {
                            myturn = false;
                        }
                    }
                }

                // It's my turn!
                if myturn {
                    turns
                        .insert(entity, MyTurn {})
                        .expect("Unable to insert turn");
                }
            }
        }

        // Handle durations
        if *runstate == RunState::AwaitingInput {
            use crate::effects::*;
            for (effect_entity, duration, status) in (&entities, &mut durations, &statuses).join() {
                if entities.is_alive(status.target) {
                    duration.turns -= 1;
                    if let Some(dot) = dots.get(effect_entity) {
                        add_effect(
                            None,
                            EffectType::Damage { amount: dot.damage },
                            Targets::Single {
                                target: status.target,
                            },
                        );
                    }
                    if duration.turns < 1 {
                        dirty
                            .insert(status.target, EquipmentChanged {})
                            .expect("Unable to insert");
                        entities.delete(effect_entity).expect("Unable to delete");
                    }
                }
            }
        }
    }
}
