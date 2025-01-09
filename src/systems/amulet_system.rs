use crate::{
    wave::{handle_event, WaveEvent, WaveState},
    AmuletOfYendor, Map, MyTurn, Name, RunState,
};
use specs::prelude::*;

pub struct AmuletSystem {}

impl<'a> System<'a> for AmuletSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, AmuletOfYendor>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, MyTurn>,
        WriteExpect<'a, RunState>,
        WriteExpect<'a, WaveState>,
        ReadExpect<'a, Entity>, // The player
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut amulet_if_yendor,
            name,
            turns,
            mut runstate,
            mut wave_state,
            player_entity,
            entities,
        ) = data;

        for (entity, _my_turn) in (&entities, &turns).join() {
            if map.depth > 1 {
                match *wave_state {
                    WaveState::WaveInProgress {
                        amount_to_spawn,
                        depth,
                    } => {
                        if amount_to_spawn < 1 {
                            *wave_state = handle_event(&WaveEvent::Wait, *wave_state)
                        } else {
                            let curent_amount_to_spawn = amount_to_spawn - 1;
                            *wave_state = WaveState::WaveInProgress {
                                amount_to_spawn: curent_amount_to_spawn,
                                depth,
                            }
                        }
                    }

                    WaveState::WaitingToComplete => {
                        // count how many enemies left
                        *wave_state = handle_event(&WaveEvent::Complete, *wave_state)
                    }
                    WaveState::WaveCompleted => {
                        *wave_state = handle_event(&WaveEvent::WaitNextWave, *wave_state)
                    }
                    _ => {}
                }

                if entity == *player_entity {
                    match *wave_state {
                        WaveState::WaitingToStart { turns_left } => {
                            if turns_left < 1 {
                                *wave_state = handle_event(&WaveEvent::Start, *wave_state)
                            } else {
                                let curent_turn_left = turns_left - 1;
                                *wave_state = WaveState::WaitingToStart {
                                    turns_left: curent_turn_left,
                                }
                            }
                        }
                        _ => {}
                    }
                    rltk::console::log(format!("entered, current state:  {:?}", *wave_state));
                    rltk::console::log("#######################################################");
                }
            }
        }

        let mut finished = false;
        for (_amulet, n, _my_turn) in (&amulet_if_yendor, &name, &turns).join() {
            if map.depth <= 1 {
                rltk::console::log(format!("{} won the game!", n.name,));
                *runstate = RunState::FinishGame;
                finished = true;
            }
        }

        if finished {
            // Clean up
            amulet_if_yendor.clear();
        }
    }
}
