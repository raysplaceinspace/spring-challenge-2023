use super::plans::{self,*};
use super::inputs::*;
use super::movement;
use super::simulator;
use super::view::{self,*};

const NUM_TICKS: u32 = 100;
const DISCOUNT_RATE: f32 = 1.02;

pub fn rollout(plan: &Vec<Milestone>, view: &View, state: &State) -> f32 {
    let mut payoff = 0.0;

    let mut state = state.clone();
    for age in 0..NUM_TICKS {
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

        if let Some(winner) = view::find_winner(&state.crystals, view) {
            payoff += evaluate_win(winner, age);
            break;
        }
    }

    payoff
}

fn discount(payoff: f32, age: u32) -> f32 {
    payoff / DISCOUNT_RATE.powf(age as f32)
}

fn evaluate_harvesting(player: usize, num_crystals: i32, previous_crystals: i32, age: u32) -> f32 {
    let mined = num_crystals - previous_crystals;
    evaluate_player(player) * discount(mined as f32, age)
}

fn evaluate_win(winner: usize, age: u32) -> f32 {
    const WINNER_PAYOFF: f32 = 100.0;
    evaluate_player(winner) * discount(WINNER_PAYOFF, age)
}

fn evaluate_player(player: usize) -> f32 {
    if player == ME {
        1.0
    } else {
        -1.0
    }
}