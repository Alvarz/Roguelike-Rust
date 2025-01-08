use specs::World;

use crate::state_machine::{StateMachine, Transition};

const TURNS_BETWEEN_WAVES: i32 = 10;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum WaveState {
    WaitingToStart,
    WaveInProgress,
    WaitingToComplete,
    WaveComplete,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum WaveEvent {
    Start,
    Waiting,
    Complete,
    WaitingNextWave,
    Restart,
}

pub struct Waves {
    machine: StateMachine<WaveState, WaveEvent>,
}

impl Waves {
    pub fn new() -> Self {
        Waves {
            machine: StateMachine::new(WaveState::WaitingToStart),
        }
    }

    pub fn configure_transitions(&mut self, _ecs: &mut World) {
        // Define transitions

        self.machine.add_transition(Transition {
            from: WaveState::WaitingToStart,
            event: WaveEvent::Start,
            to: WaveState::WaveInProgress,
            action: Some(Box::new(|machine, _to, _event| {
                rltk::console::log("Transition action executed Start");
                machine.handle_event(WaveEvent::Waiting)?;
                // Trigger Waiting event after some turns
                // You'll need a way to track game turns (e.g., a game timer component)
                // Here's a simplified example:
                // schedule_event_after_turns(machine, WaveEvent::Waiting, TURNS_BETWEEN_WAVES);
                Ok(())
            })),
        });
        self.machine.add_transition(Transition {
            from: WaveState::WaveInProgress,
            event: WaveEvent::Waiting,
            to: WaveState::WaitingToComplete,
            action: Some(Box::new(|machine, _to, _event| {
                rltk::console::log("Transition action executed Waiting");
                machine.handle_event(WaveEvent::Complete)?;
                Ok(())
            })),
        });
        self.machine.add_transition(Transition {
            from: WaveState::WaitingToComplete,
            event: WaveEvent::Complete,
            to: WaveState::WaveComplete,
            action: Some(Box::new(|machine, _to, _event| {
                rltk::console::log("Transition action executed Complete");
                machine.handle_event(WaveEvent::WaitingNextWave)?;
                Ok(())
            })),
        });
        self.machine.add_transition(Transition {
            from: WaveState::WaveComplete,
            event: WaveEvent::WaitingNextWave,
            to: WaveState::WaitingToStart,
            action: Some(Box::new(|machine, _to, _event| {
                rltk::console::log("Transition action executed WaitingNextWave");
                // Trigger Start event for the next wave
                machine.handle_event(WaveEvent::Start)?;
                Ok(())
            })),
        });
    }

    pub fn restart(&mut self) {
        self.machine.handle_event(WaveEvent::Restart).unwrap();
    }

    pub fn trigger_event(&mut self, event: WaveEvent) {
        // Handle invalid event
        match self.machine.handle_event(event) {
            Ok(_) => rltk::console::log("Transition successful (unexpected)"),
            Err(e) => rltk::console::log(format!("Error: {}", e)),
        }
    }
}

// Example of scheduling an event after turns (simplified)
// fn schedule_event_after_turns(machine: &mut StateMachine<WaveState, WaveEvent>, event: WaveEvent, turns: i32) {
//     // In a real game, you would use a game timer or scheduling system
//     // For this example, we'll simulate it:
//     if turns > 0 {
//         // Simulate turn passing (replace with actual game logic)
//         // ...
//         schedule_event_after_turns(machine, event, turns - 1);
//     } else {
//         machine.handle_event(event).unwrap();
//     }
// }
