use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Copy)]
pub enum WaveState {
    WaitingToStart { turns_left: i32 },
    WaveInProgress { amount_to_spawn: i32, depth: i32 },
    WaitingToComplete,
    WaveCompleted,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Copy)]
pub enum WaveEvent {
    Start,
    Wait,
    Complete,
    WaitNextWave,
    Restart,
}

pub fn handle_event(event: &WaveEvent, mut wave_state: WaveState) -> WaveState {
    match (&wave_state, event) {
        (WaveState::WaitingToStart { .. }, WaveEvent::Start) => {
            rltk::console::log(format!(
                "moved from {:?} to  {:?} with event {:?}",
                wave_state,
                WaveState::WaveInProgress {
                    amount_to_spawn: 1,
                    depth: 1
                },
                event,
            ));
            wave_state = WaveState::WaveInProgress {
                amount_to_spawn: 1,
                depth: 1,
            };
        }

        (WaveState::WaveInProgress { .. }, WaveEvent::Wait) => {
            rltk::console::log(format!(
                "moved from {:?} to  {:?} with event {:?}",
                wave_state,
                WaveState::WaitingToComplete,
                event,
            ));
            wave_state = WaveState::WaitingToComplete;
        }
        (WaveState::WaitingToComplete, WaveEvent::Complete) => {
            rltk::console::log(format!(
                "moved from {:?} to  {:?} with event {:?}",
                wave_state,
                WaveState::WaveCompleted,
                event,
            ));
            wave_state = WaveState::WaveCompleted;
        }
        (WaveState::WaveCompleted, WaveEvent::WaitNextWave) => {
            rltk::console::log(format!(
                "moved from {:?} to  {:?} with event {:?}",
                wave_state,
                WaveState::WaitingToStart { turns_left: 100 },
                event,
            ));
            wave_state = WaveState::WaitingToStart { turns_left: 100 };
        }
        _ => {
            rltk::console::log(format!(
                "No valid transition for event: {:?} from state {:?}",
                event, wave_state
            ));
        }
    }
    wave_state
}
