use std::time::Instant;
use super::inputs::*;
use super::view::*;
use super::evaluation;
use super::plans::{self,*};

pub fn act(view: &View, state: &State) -> Vec<Action> {
    let start = Instant::now();

    let mut best = evaluate(Vec::new(), view, state);
    eprintln!("initial score: {}", best.score);
    let mut num_evaluated = 1;

    for cell in 0..view.layout.cells.len() {
        if state.resources[cell] <= 0 { continue }
        let plan = vec![Milestone { cell }];
        let candidate = evaluate(plan, view, state);
        eprintln!("candidate {} score: {}", cell, candidate.score);

        num_evaluated += 1;

        if candidate.score > best.score {
            best = candidate;
        }
    }

    let actions = plans::enact_plan(ME, &best.plan, view, state);

    eprintln!("evaluated {} plans in {:.0} ms", num_evaluated, start.elapsed().as_millis());

    actions
}

fn evaluate(plan: Vec<Milestone>, view: &View, state: &State) -> Candidate {
    const NUM_TICKS: u32 = 100;
    let score = evaluation::rollout(&plan, NUM_TICKS, view, state);
    Candidate { plan, score }
}

struct Candidate {
    pub plan: Vec<Milestone>,
    pub score: i32,
}