use super::planning::{self,*};
use super::inputs::*;
use super::simulation;
use super::view::{self,*};

const NUM_TICKS: u32 = 25;
const DECAY_RATE: f32 = 0.98;
const WIN_PAYOFF: f32 = 0.0;

#[derive(Clone,Debug)]
pub struct Endgame {
    pub tick: u32,
    pub crystals: CrystalsPerPlayer,
    pub total_ants: [i32; NUM_PLAYERS],
}

pub fn rollout(me: usize, plans: [&Vec<Milestone>; NUM_PLAYERS], view: &View, state: &State) -> (f32,Endgame) {
    let enemy = (me + 1) % NUM_PLAYERS;
    let mut payoff = (state.crystals[me] - state.crystals[enemy]) as f32;

    let mut state = state.clone();
    for age in 0..NUM_TICKS {
        let Commands { assignments: my_assignments, .. } = planning::enact_plan(ME, plans[ME], view, &state);
        let Commands { assignments: enemy_assignments, .. } = planning::enact_plan(ENEMY, plans[ENEMY], view, &state);

        let assignments = [
            my_assignments,
            enemy_assignments,
        ];

        let previous_crystals = state.crystals.clone();
        simulation::forward(&assignments, view, &mut state);

        for player in 0..NUM_PLAYERS {
            payoff += player_sign(me, player) * evaluate_harvesting(state.crystals[player], previous_crystals[player], age);
        }

        if let Some(winner) = view::find_winner(&state.crystals, view) {
            payoff += player_sign(me, winner) * evaluate_win(age);
            break;
        }

        if state.tick >= MAX_TICKS { break; }
    }

    let crystals_to_win = view.initial_crystals / 2 - state.crystals.iter().sum::<i32>();
    if crystals_to_win > 0 {
        // Apportion final crystals according to number of eggs
        let total_ants: i32 = state.total_ants.iter().sum();
        let remaining_crystal_amounts = [
            crystals_to_win as f32 * (state.total_ants[ME] as f32 / total_ants as f32),
            crystals_to_win as f32 * (state.total_ants[ENEMY] as f32 / total_ants as f32),
        ];
        for player in 0..NUM_PLAYERS {
            payoff += player_sign(me, player) * remaining_crystal_amounts[player] * discount(MAX_TICKS);
        }
    }

    let endgame = Endgame {
        tick: state.tick,
        crystals: state.crystals,
        total_ants: state.total_ants,
    };
    (payoff, endgame)
}

fn discount(age: u32) -> f32 {
    DECAY_RATE.powi(age as i32)
}

fn evaluate_harvesting(num_crystals: i32, previous_crystals: i32, age: u32) -> f32 {
    let mined = num_crystals - previous_crystals;
    mined as f32 * discount(age)
}

fn evaluate_win(age: u32) -> f32 {
    WIN_PAYOFF * discount(age)
}

fn player_sign(me: usize, player: usize) -> f32 {
    if player == me {
        1.0
    } else {
        -1.0
    }
}