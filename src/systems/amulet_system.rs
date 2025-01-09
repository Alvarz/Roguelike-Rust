use crate::{AmuletOfYendor, HordeMode, Map, MyTurn, RunState, WaveState};
use specs::prelude::*;

pub struct AmuletSystem {}

impl<'a> System<'a> for AmuletSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadStorage<'a, AmuletOfYendor>,
        ReadStorage<'a, MyTurn>,
        WriteExpect<'a, RunState>,
        WriteStorage<'a, HordeMode>,
        ReadExpect<'a, Entity>, // The player
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, amulet_if_yendor, turns, mut runstate, mut horde_modes, player_entity, entities) =
            data;

        for (_amulet, _my_turn, entity) in (&amulet_if_yendor, &turns, &entities).join() {
            if map.depth <= 1 && entity == *player_entity {
                rltk::console::log("{} You won the game!");
                *runstate = RunState::FinishGame;
            } else if horde_modes.get(entity).is_none() {
                let _ = horde_modes.insert(
                    entity,
                    HordeMode {
                        state: WaveState::WaitingToStart { turns_left: 1 },
                    },
                );
            }
        }
    }
}
