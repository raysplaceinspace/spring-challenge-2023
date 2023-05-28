use super::inputs::*;
use super::view::*;
use super::evaluation;
use super::plans::{self,*};

pub fn act(view: &View, state: &State) -> Vec<Action> {
    let mut best = evaluate(Vec::new(), view, state);
    eprintln!("initial score: {}", best.score);

    for cell in 0..view.layout.cells.len() {
        if state.resources[cell] <= 0 { continue }
        let plan = vec![PlanStep { cell }];
        let candidate = evaluate(plan, view, state);
        eprintln!("candidate {} score: {}", cell, candidate.score);

        if candidate.score > best.score {
            best = candidate;
        }
    }

    let actions = plans::enact_plan(ME, &best.plan, view, state);

    actions
}

fn evaluate(plan: Vec<PlanStep>, view: &View, state: &State) -> Candidate {
    const NUM_TICKS: u32 = 100;
    let score = evaluation::rollout(&plan, NUM_TICKS, view, state);
    Candidate { plan, score }
}

struct Candidate {
    pub plan: Vec<PlanStep>,
    pub score: i32,
}