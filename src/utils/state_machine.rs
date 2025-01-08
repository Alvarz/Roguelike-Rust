use std::collections::HashMap;
use std::hash::Hash;

type TransitionAction<S, E> =
    Box<dyn Fn(&mut StateMachine<S, E>, &S, &E) -> Result<(), String> + Send + Sync>;

pub struct Transition<S, E> {
    pub from: S,
    pub event: E,
    pub to: S,
    pub action: Option<TransitionAction<S, E>>,
}

pub struct StateMachine<S, E> {
    current_state: S,
    transitions: HashMap<(S, E), Transition<S, E>>,
    initial_state: S, // Store the initial state for restarting
}

impl<S: Clone + Eq + Hash + std::fmt::Debug, E: Clone + Eq + Hash + std::fmt::Debug>
    StateMachine<S, E>
{
    pub fn new(initial_state: S) -> Self {
        StateMachine {
            current_state: initial_state.clone(),
            transitions: HashMap::new(),
            initial_state,
        }
    }

    pub fn add_transition(&mut self, transition: Transition<S, E>) {
        self.transitions.insert(
            (transition.from.clone(), transition.event.clone()),
            transition,
        );
    }

    pub fn handle_event(&mut self, event: E) -> Result<(), String> {
        if let Some(transition) = self
            .transitions
            .get(&(self.current_state.clone(), event.clone()))
        {
            self.current_state = transition.to.clone();

            if let Some(ref action) = transition.action {
                // Pass "self" to the action closure
                action(self, &self.current_state, &event)?;
            }

            Ok(())
        } else {
            Err(format!(
                "Invalid transition: {:?} from state {:?}",
                event, self.current_state
            ))
        }
    }

    pub fn get_current_state(&self) -> &S {
        &self.current_state
    }
}
