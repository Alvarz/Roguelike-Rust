// use crate::state_machine::{StateMachine, Transition};

const TURNS_BETWEEN_WAVES: i32 = 10;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum WaveState {
    WaitingToStart,
    WaveInProgress,
    WaitingToComplete,
    WaveCompleted,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub enum WaveEvent {
    Start,
    Wait,
    Complete,
    WaitNextWave,
    Restart,
}

pub fn handle_event(event: &WaveEvent, mut wave_state: WaveState) -> WaveState {
    match (&wave_state, event) {
        (WaveState::WaitingToStart, WaveEvent::Start) => {
            rltk::console::log(format!(
                "moved from {:?} to  {:?} with event {:?}",
                wave_state,
                WaveState::WaveInProgress,
                event,
            ));
            wave_state = WaveState::WaveInProgress;
        }

        (WaveState::WaveInProgress, WaveEvent::Wait) => {
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
                WaveState::WaitingToStart,
                event,
            ));
            wave_state = WaveState::WaitingToStart;
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
