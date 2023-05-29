use super::plans::{self,*};
use super::inputs::*;
use super::movement;
use super::simulator;
use super::view::{self,*};

const DISCOUNT_RATE: f32 = 1.07;

pub fn rollout(plan: &Vec<Milestone>, num_ticks: u32, view: &View, state: &State) -> f32 {
    let mut payoff = 0.0;

    let mut state = state.clone();
    for age in 0..num_ticks {
        let actions = plans::enact_plan(ME, plan, view, &state);

        let assignments = [
            movement::actions_to_assignments(ME, view, &state.num_ants, actions.iter()),
            movement::keep_assignments(ENEMY, &state.num_ants),
        ];

        let initial_crystals = state.crystals.clone();
        simulator::forward(&assignments, view, &mut state);

        for player in 0..NUM_PLAYERS {
            payoff += evaluate_harvesting(player, state.crystals[player], initial_crystals[player], age);
        }

        if let Some(_) = view::find_winner(&state.crystals, view) {
            break;
        }
    }

    payoff
}

fn evaluate_harvesting(player: usize, num_crystals: i32, previous_crystals: i32, age: u32) -> f32 {
    let mined = num_crystals - previous_crystals;
    let payoff = mined as f32 / DISCOUNT_RATE.powf(age as f32);
    if player == ME {
        payoff
    } else {
        -payoff
    }
}