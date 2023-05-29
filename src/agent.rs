use std::fmt::Display;
use std::time::Instant;
use super::inputs::*;
use super::view::*;
use super::evaluation;
use super::plans::{self,*};

pub fn act(view: &View, state: &State) -> Vec<Action> {
    let start = Instant::now();

    let mut best = evaluate(Vec::new(), view, state);
    let mut num_evaluated = 1;
    let mut num_improvements = 0;

    for cell in 0..view.layout.cells.len() {
        if state.resources[cell] <= 0 { continue }
        let plan = vec![Milestone { cell }];
        let candidate = evaluate(plan, view, state);

        num_evaluated += 1;

        if candidate.score > best.score {
            best = candidate;
            num_improvements += 1;
        }
    }

    eprintln!("{}: found best plan in {:.0} ms ({}/{} successful iterations)", state.tick, start.elapsed().as_millis(), num_improvements, num_evaluated);
    eprintln!("{}", best);

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
impl Display for Candidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "score={:.0}: ", self.score)?;

        let mut is_first = true;
        for milestone in self.plan.iter() {
            if is_first {
                is_first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{}", milestone)?;
        }
        Ok(())
    }
}