use std::time::Instant;
use super::inputs::*;
use super::view::*;
use super::evaluation;
use super::plans::{self,*};

pub fn act(view: &View, state: &State) -> Vec<Action> {
    let start = Instant::now();

    let mut best = evaluate(Vec::new(), view, state);
    let mut num_evaluated = 1;

    for cell in 0..view.layout.cells.len() {
        if state.resources[cell] <= 0 { continue }
        let plan = vec![Milestone { cell }];
        let candidate = evaluate(plan, view, state);

        num_evaluated += 1;

        if candidate.score > best.score {
            eprintln!("candidate {}: {:.0} -> {:.0}", cell, best.score, candidate.score);
            best = candidate;
        }
    }

    eprintln!("{}: found best plan (score={}) in {:.0} ms ({} iterations)", state.tick, best.score, start.elapsed().as_millis(), num_evaluated);

    let actions = plans::enact_plan(ME, &best.plan, view, state);

    actions
}

fn evaluate(plan: Vec<Milestone>, view: &View, state: &State) -> Candidate {
    const NUM_TICKS: u32 = 100;
    let score = evaluation::rollout(&plan, NUM_TICKS, view, state);
    Candidate { plan, score }
}

struct Candidate {
    pub plan: Vec<Milestone>,
    pub score: f32,
}