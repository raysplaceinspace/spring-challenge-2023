use super::planning::{self,*};
use super::inputs::*;
use super::opponents;
use super::simulation;
use super::view::{self,*};

const NUM_TICKS: u32 = 100;
const DISCOUNT_RATE: f32 = 1.02;
const WIN_PAYOFF: f32 = 1.0;

#[derive(Clone,Debug)]
pub struct Endgame {
    pub tick: u32,
    pub crystals: CrystalsPerPlayer,
}

pub fn rollout(plan: &Vec<Milestone>, view: &View, state: &State) -> (f32,Endgame) {
    let mut payoff = 0.0;

    let mut state = state.clone();
    for age in 0..NUM_TICKS {
        let Commands { assignments: my_assignments, .. } = planning::enact_plan(ME, plan, view, &state);
        let countermoves = opponents::enact_countermoves(ENEMY, view, &state);

        let assignments = [
            my_assignments,
            countermoves,
        ];

        let initial_crystals = state.crystals.clone();
        simulation::forward(&assignments, view, &mut state);

        for player in 0..NUM_PLAYERS {
            payoff += evaluate_harvesting(player, state.crystals[player], initial_crystals[player], age);
        }

        if let Some(winner) = view::find_winner(&state.crystals, view) {
            payoff += evaluate_win(winner, age);
            break;
        }
    }

    let endgame = Endgame {
        tick: state.tick,
        crystals: state.crystals,
    };
    (payoff, endgame)
}

fn discount(payoff: f32, age: u32) -> f32 {
    payoff / DISCOUNT_RATE.powf(age as f32)
}

fn evaluate_harvesting(player: usize, num_crystals: i32, previous_crystals: i32, age: u32) -> f32 {
    let mined = num_crystals - previous_crystals;
    evaluate_player(player) * discount(mined as f32, age)
}

fn evaluate_win(player: usize, age: u32) -> f32 {
    evaluate_player(player) * discount(WIN_PAYOFF, age)
}

fn evaluate_player(player: usize) -> f32 {
    if player == ME {
        1.0
    } else {
        -1.0
    }
}