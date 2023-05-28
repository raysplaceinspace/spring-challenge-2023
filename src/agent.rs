use super::inputs::*;
use super::view::*;
use super::evaluation;
use super::policies::{self,*};

pub fn act(view: &View, state: &State) -> Vec<Action> {
    let mut best = evaluate(Plan::new(), view, state);
    eprintln!("initial score: {}", best.score);

    for cell in 0..view.layout.cells.len() {
        if state.resources[cell] <= 0 { continue }
        let candidate = evaluate(Plan::singular(cell), view, state);
        eprintln!("candidate {} score: {}", cell, candidate.score);

        if candidate.score > best.score {
            best = candidate;
        }
    }

    let actions = policies::enact_plan(ME, &best.plan, view, state);

    actions
}

fn evaluate(plan: Plan, view: &View, state: &State) -> Candidate {
    const NUM_TICKS: u32 = 25;
    let score = evaluation::rollout(&plan, NUM_TICKS, view, state);
    Candidate { plan, score }
}

struct Candidate {
    pub plan: Plan,
    pub score: i32,
}