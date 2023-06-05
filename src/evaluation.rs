use super::planning::{self,*};
use super::inputs::*;
use super::simulation;
use super::view::{self,*};

const NUM_TICKS: u32 = 100;
const DECAY_RATE: f32 = 0.95;
const WIN_PAYOFF: f32 = 1.0;

#[derive(Clone,Debug)]
pub struct Endgame {
    pub tick: u32,
    pub crystals: CrystalsPerPlayer,
    pub total_ants: [i32; NUM_PLAYERS],
    pub winner: Option<usize>,
}

pub fn rollout(plans: [&Vec<Milestone>; NUM_PLAYERS], view: &View, state: &State) -> (f32,Endgame) {
    let mut payoff = 0.0;

    let mut state = state.clone();
    let mut winner = None;
    for age in 0..NUM_TICKS {
        let Commands { assignments: my_assignments, .. } = planning::enact_plan(ME, &plans[ME], view, &state);
        let Commands { assignments: enemy_assignments, .. } = planning::enact_plan(ENEMY, &plans[ENEMY], view, &state);

        let assignments = [
            my_assignments,
            enemy_assignments,
        ];

        let initial_crystals = state.crystals.clone();
        simulation::forward(&assignments, view, &mut state);

        for player in 0..NUM_PLAYERS {
            payoff += evaluate_harvesting(player, state.crystals[player], initial_crystals[player], age);
        }

        if let Some(w) = view::find_winner(view, &state) {
            payoff += evaluate_win(w, age);
            winner = Some(w);
            break;
        }

        if state.tick >= MAX_TICKS { break; }
    }

    let endgame = Endgame {
        tick: state.tick,
        crystals: state.crystals,
        total_ants: state.total_ants,
        winner,
    };
    (payoff, endgame)
}

fn discount(age: u32) -> f32 {
    DECAY_RATE.powi(age as i32)
}

fn evaluate_harvesting(player: usize, num_crystals: i32, previous_crystals: i32, age: u32) -> f32 {
    let mined = num_crystals - previous_crystals;
    mined as f32 * evaluate_player(player) * discount(age)
}

fn evaluate_win(player: usize, age: u32) -> f32 {
    WIN_PAYOFF * evaluate_player(player) * discount(age)
}

fn evaluate_player(player: usize) -> f32 {
    if player == ME {
        1.0
    } else {
        -1.0
    }
}