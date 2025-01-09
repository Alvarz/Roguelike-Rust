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
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut amulet_if_yendor, name, turns, mut runstate, mut wave_state) = data;

        if map.depth > 1 {
            match *wave_state {
                WaveState::WaitingToStart => {
                    *wave_state = handle_event(&WaveEvent::Start, *wave_state)
                }

                WaveState::WaveInProgress => {
                    *wave_state = handle_event(&WaveEvent::Wait, *wave_state)
                }

                WaveState::WaitingToComplete => {
                    *wave_state = handle_event(&WaveEvent::Complete, *wave_state)
                }

                WaveState::WaveCompleted => {
                    *wave_state = handle_event(&WaveEvent::WaitNextWave, *wave_state)
                }
            }
        }

        if map.depth > 1 && *wave_state == WaveState::WaitingToStart {
            *wave_state = handle_event(&WaveEvent::Start, *wave_state);
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
