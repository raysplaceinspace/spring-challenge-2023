use rand::prelude::*;
use std::fmt::Display;
use std::time::Instant;

use super::inputs::*;
use super::movement;
use super::view::*;
use super::evaluation::{self,Endgame};
use super::opponents;
use super::planning::{self,*};
use super::solving::{QuantileEstimator,PheromoneMatrix,Mutator,Mutation,Walk};
use super::valuation::HarvestAndSpawnEvaluator;

const SEARCH_MS: u128 = 90;
const CLOSE_ENOUGH: f32 = 0.01;

#[derive(Copy,Clone)]
enum SolverType {
    Generation,
    Mutation,
}
const NUM_SOLVERS: usize = 2;
const SOLVERS: [SolverType; NUM_SOLVERS] = [
    SolverType::Generation,
    SolverType::Mutation,
];

enum Lesson {
    Generation(Box<[Walk]>),
    Mutation(Mutation),
}

pub struct Agent {
    generator: PheromoneMatrix,
    mutator: Mutator,
    previous_plan: Option<Vec<Milestone>>,
    rng: StdRng,
}
impl Agent {
    pub fn new(player: usize, view: &View) -> Self {
        Self {
            generator: PheromoneMatrix::new(player, view),
            mutator: Mutator::new(),
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
        eprintln!("initial: {}", best);

        let mut num_evaluated = 1;
        let mut num_successes = [0; NUM_SOLVERS];

        let mut scorer = QuantileEstimator::new();
        scorer.insert(best.score);

        while start.elapsed().as_millis() < SEARCH_MS {
            let solver = SOLVERS[num_evaluated % NUM_SOLVERS];
            let (plan, lesson) = match solver {
                SolverType::Generation => {
                    let (plan, walks) = self.generator.generate(&mut self.rng, |cell| {
                        state.resources[cell] > 0
                    });
                    (plan, Lesson::Generation(walks))
                },
                SolverType::Mutation => {
                    let mut plan = best.plan.clone();
                    let mutation = self.mutator.mutate(&mut plan, &mut self.rng);
                    (plan, Lesson::Mutation(mutation))
                },
            };

            let candidate = Candidate::evaluate(plan, view, state);
            num_evaluated += 1;

            let quantile = scorer.quantile(candidate.score);
            scorer.insert(candidate.score);

            match lesson {
                Lesson::Generation(walks) => self.generator.learn(quantile, &walks),
                Lesson::Mutation(mutation) => self.mutator.learn(quantile, mutation),
            }

            if candidate.is_improvement(&best) {
                best = candidate;
                num_successes[solver as usize] += 1;
            }
        }

        let harvests = [
            HarvestAndSpawnEvaluator::new(ME, view, state),
            HarvestAndSpawnEvaluator::new(ENEMY, view, state),
        ];

        let commands = planning::enact_plan(ME, &best.plan, view, state);
        let countermoves = opponents::enact_countermoves(ENEMY, view, state);

        let mut actions = movement::assignments_to_actions(&commands.assignments);
        actions.push(Action::Message { text: {
            if best.endgame.crystals[ME] >= best.endgame.crystals[ENEMY] {
                "Good game :)".to_string()
            } else {
                "Congratulations!".to_string()
            }
        }});

        eprintln!("{}: found best plan in {:.0} ms ({} iterations)", state.tick, start.elapsed().as_millis(), num_evaluated);
        eprintln!("Successful: {} generations, {} mutations", num_successes[SolverType::Generation as usize], num_successes[SolverType::Mutation as usize]);
        eprintln!("best: {}", best);
        eprintln!(
            "Endgame: tick={}, crystals=[{} vs {}], ants=[{} vs {}]",
            best.endgame.tick,
            best.endgame.crystals[0], best.endgame.crystals[1],
            best.endgame.total_ants[0], best.endgame.total_ants[1],
        );
        eprintln!("Goals: {} vs {}", commands, countermoves);
        eprintln!("Ticks to win: {:.0} vs {:.0}", harvests[0].ticks_to_harvest_remaining_crystals(), harvests[1].ticks_to_harvest_remaining_crystals());
        eprintln!("Ticks saved from 1 egg: {:.2} vs {:.2}", harvests[0].calculate_ticks_saved_harvesting_eggs(1), harvests[1].calculate_ticks_saved_harvesting_eggs(1));

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