use crate::{
    gamelog, HordeMember, HordeMode, Map, MyTurn, Position, RunState, WaveState, TURNS_BETWEEN_BASE,
};
use specs::prelude::*;

pub struct HordeModeSystem {}

impl<'a> System<'a> for HordeModeSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, HordeMode>,
        Entities<'a>,
        WriteExpect<'a, RunState>,
        ReadStorage<'a, HordeMember>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (_map, mut turns, mut horde_modes, entities, mut runstate, horde_members, positions) =
            data;

        let active_horde_members_count = (&horde_members, &positions).join().count();

        let mut turn_done: Vec<Entity> = Vec::new();
        for (horde_mode, entity, _my_turn, _pos) in
            (&mut horde_modes, &entities, &mut turns, &positions).join()
        {
            turn_done.push(entity);
            match horde_mode.state {
                WaveState::WaveInProgress {
                    amount_to_spawn,
                    depth,
                } => {
                    if amount_to_spawn < 1 {
                        gamelog::Logger::new()
                            .color(bracket_lib::terminal::RED)
                            .append("a horde of enemies approach you!.")
                            .log();
                        horde_mode.state = WaveState::WaitingToComplete;
                        *runstate = RunState::SpawnWave
                    } else {
                        let current_amount_to_spawn = amount_to_spawn - 1;
                        horde_mode.state = WaveState::WaveInProgress {
                            amount_to_spawn: current_amount_to_spawn,
                            depth,
                        }
                    }
                }

                WaveState::WaitingToComplete => {
                    // count how many enemies left
                    if active_horde_members_count < 1 {
                        horde_mode.state = WaveState::WaveCompleted;
                    }

                    bracket_lib::prelude::console::log(format!(
                        "amount of horde members. {}",
                        active_horde_members_count
                    ));
                }
                WaveState::WaveCompleted => {
                    horde_mode.state = WaveState::WaitingToStart {
                        turns_left: TURNS_BETWEEN_BASE,
                    };
                    gamelog::Logger::new()
                        .color(bracket_lib::terminal::RED)
                        .append("the horde is completed for now!.")
                        .log();
                }
                WaveState::WaitingToStart { turns_left } => {
                    if turns_left < 1 {
                        horde_mode.state = WaveState::WaveInProgress {
                            amount_to_spawn: 1,
                            depth: 1,
                        }
                    } else {
                        let curent_turn_left = turns_left - 1;
                        horde_mode.state = WaveState::WaitingToStart {
                            turns_left: curent_turn_left,
                        }
                    }
                }
            }
        }

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
