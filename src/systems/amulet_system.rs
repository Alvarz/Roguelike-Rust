use crate::{AmuletOfYendor, HordeMode, Map, MyTurn, Position, RunState};
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
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            amulet_of_yendor,
            turns,
            mut runstate,
            horde_modes,
            player_entity,
            entities,
            positions,
        ) = data;

        if !amulet_of_yendor.contains(*player_entity) {
            return;
        }

        for (_my_turn, entity, _pos) in (&turns, &entities, &positions).join() {
            // player doesn't have the AOY

            if map.depth <= 1 && entity == *player_entity {
                *runstate = RunState::FinishGame;
                return;
            }
        }

        let horde_modes = (&horde_modes, &positions).join();
        if horde_modes.into_iter().count() < 1 {
            *runstate = RunState::SpawnHordeMode;
        }
    }
}
