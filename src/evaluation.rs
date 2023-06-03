use super::planning::{self,*};
use super::inputs::*;
use super::simulation;
use super::view::{self,*};

const NUM_TICKS: u32 = 100;
const DECAY_RATE: f32 = 0.98;
const WIN_PAYOFF: f32 = 0.0;

#[derive(Clone,Debug)]
pub struct Endgame {
    pub tick: u32,
    pub crystals: CrystalsPerPlayer,
    pub total_ants: [i32; NUM_PLAYERS],
}

pub fn rollout(me: usize, plans: [&Vec<Milestone>; NUM_PLAYERS], view: &View, state: &State) -> (f32,Endgame) {
    let mut payoff = 0.0;

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