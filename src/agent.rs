use rand::prelude::*;
use std::fmt::Display;
use std::time::Instant;

use super::inputs::*;
use super::movement;
use super::view::*;
use super::evaluation::{self,Endgame};
use super::opponents;
use super::planning::{self,*};
use super::solving::{QuantileEstimator,PheromoneMatrix};

const SEARCH_MS: u128 = 80;
const CLOSE_ENOUGH: f32 = 0.01;
const LEARNING_RATE: f32 = 0.01;

const WALK_MIN_POWER: f32 = 1.0;
const WALK_POWER_PER_ITERATION: f32 = 0.01;

pub struct Agent {
    pheromones: PheromoneMatrix,
    previous_plan: Option<Vec<Milestone>>,
    rng: StdRng,
}
impl Agent {
    pub fn new(view: &View) -> Self {
        Self {
            pheromones: PheromoneMatrix::new(view),
            previous_plan: None,
            rng: StdRng::seed_from_u64(0x1234567890abcdef),
        }
    }

    pub fn act(&mut self, view: &View, state: &State) -> Vec<Action> {
        let start = Instant::now();

        let initial_plan = match self.previous_plan.take() {
            Some(mut plan) => {
                plan.retain(|m| !m.is_complete(state));
                plan
            },
            None => Vec::new(),
        };
        let mut best = Candidate::evaluate(initial_plan, view, state);

        let mut num_evaluated = 1;
        let mut num_improvements = 0;

        let mut scorer = QuantileEstimator::new();

        while start.elapsed().as_millis() < SEARCH_MS {
            let walk_power = WALK_MIN_POWER + WALK_POWER_PER_ITERATION * num_evaluated as f32;

            let mut plan = Vec::new();
            for cell in self.pheromones.walk(walk_power, &mut self.rng, |cell| state.resources[cell] > 0) {
                plan.push(Milestone::new(cell));
            }
            let candidate = Candidate::evaluate(plan, view, state);
            num_evaluated += 1;

            let quantile = scorer.quantile(candidate.score);
            scorer.insert(candidate.score);
            self.pheromones.learn(quantile, LEARNING_RATE, candidate.plan.iter().map(|m| m.cell));

            if candidate.is_improvement(&best) {
                best = candidate;
                num_improvements += 1;
            }
        }

        eprintln!("{}: found best plan in {:.0} ms ({}/{} successful iterations)", state.tick, start.elapsed().as_millis(), num_improvements, num_evaluated);
        eprintln!("{}", best);

        eprintln!(
            "Endgame: tick={}, crystals=[{} vs {}], ants=[{} vs {}]",
            best.endgame.tick,
            best.endgame.crystals[0], best.endgame.crystals[1],
            best.endgame.num_ants[0], best.endgame.num_ants[1],
        );

        let commands = planning::enact_plan(ME, &best.plan, view, state);
        let countermoves = opponents::enact_countermoves(ENEMY, view, state);

        let mut actions = movement::assignments_to_actions(&commands.assignments);

        let summary = format!("{} vs {}", commands, countermoves);
        eprintln!("Next: {}", summary);
        actions.push(Action::Message { text: summary });

        self.previous_plan = Some(best.plan);
        actions
    }
}

#[derive(Clone)]
struct Candidate {
    pub plan: Vec<Milestone>,
    pub score: f32,
    pub endgame: Endgame,
}
impl Candidate {
    pub(self) fn evaluate(plan: Vec<Milestone>, view: &View, state: &State) -> Self {
        let (score, endgame) = evaluation::rollout(&plan, view, state);
        Self { plan, score, endgame }
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

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_simple_layout() {
    }
}