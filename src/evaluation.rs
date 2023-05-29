use super::plans::{self,*};
use super::inputs::*;
use super::movement;
use super::simulator::{self,Event};
use super::view::{self,*};

pub fn rollout(plan: &Vec<Milestone>, num_ticks: u32, view: &View, state: &State, mut events: Option<&mut Vec<Event>>) -> i32 {
    let mut state = state.clone();
    for _ in 0..num_ticks {
        let actions = plans::enact_plan(ME, plan, view, &state);

        let assignments = [
            movement::actions_to_assignments(ME, view, &state.num_ants, actions.iter()),
            movement::keep_assignments(ENEMY, &state.num_ants),
        ];

        simulator::forward(&assignments, view, &mut state, match &mut events {
            Some(events) => Some(events),
            None => None,
        });

        if let Some(_) = view::find_winner(&state.crystals, view) {
            break;
        }
    }
    evaluate(&state)
}

pub fn evaluate(state: &State) -> i32 {
    state.crystals[ME] - state.crystals[ENEMY]
}