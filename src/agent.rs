use rand::prelude::*;
use std::fmt::Display;
use std::time::Instant;
use super::inputs::*;
use super::view::*;
use super::evaluation;
use super::mutations::Mutator;
use super::opponents;
use super::plans::{self,*};

const SEARCH_MS: u128 = 80;
const CLOSE_ENOUGH: f32 = 0.01;

pub struct Agent {
    previous_plan: Option<Vec<Milestone>>,
    rng: StdRng,
}
impl Agent {
    pub fn new() -> Self {
        Self {
            previous_plan: None,
            rng: StdRng::seed_from_u64(0x1234567890abcdef),
        }
    }

    pub fn act(&mut self, view: &View, state: &State) -> Vec<Action> {
        let start = Instant::now();

        let mut best = Candidate::evaluate(self.previous_plan.take().unwrap_or_else(|| Vec::new()), view, state);

        let mut num_evaluated = 1;
        let mut num_improvements = 0;

        if let Some(mutator) = Mutator::new(view, state) {
            while start.elapsed().as_millis() < SEARCH_MS {
                let plan = match mutator.mutate(&best.plan, &mut self.rng) {
                    Some(plan) => plan,
                    None => continue,
                };
                let candidate = Candidate::evaluate(plan, view, state);
                num_evaluated += 1;

                if candidate.is_improvement(&best) {
                    best = candidate;
                    num_improvements += 1;
                }
            }
        }

        eprintln!("{}: found best plan in {:.0} ms ({}/{} successful iterations)", state.tick, start.elapsed().as_millis(), num_improvements, num_evaluated);
        eprintln!("{}", best);
        self.previous_plan = Some(best.plan.clone());

        if let Some(countermove) = opponents::predict_countermove(ENEMY, view, &state) {
            eprintln!("Predicted enemy countermove: {}", countermove.target);
        }

        let actions = plans::enact_plan(ME, &best.plan, view, state);

        actions
    }
}

#[derive(Clone)]
struct Candidate {
    pub plan: Vec<Milestone>,
    pub score: f32,
}
impl Candidate {
    pub(self) fn evaluate(plan: Vec<Milestone>, view: &View, state: &State) -> Self {
        let score = evaluation::rollout(&plan, view, state);
        Self { plan, score }
    }

    pub fn is_improvement(&self, other: &Self) -> bool {
        self.score > other.score
            || (self.score - other.score).abs() < CLOSE_ENOUGH && self.plan.len() < other.plan.len()
    }
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