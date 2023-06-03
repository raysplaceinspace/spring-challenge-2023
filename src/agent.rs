use rand::prelude::*;
use std::fmt::Display;
use std::time::Instant;

use super::inputs::*;
use super::movement;
use super::view::*;
use super::evaluation::{self,Endgame};
use super::planning::{self,*};
use super::solving::{QuantileEstimator,PheromoneMatrix};

const ADVERSARIAL_MS: u128 = 30;
const SEARCH_MS: u128 = 60;
const CLOSE_ENOUGH: f32 = 0.01;

const WALK_MIN_POWER: f32 = 1.0;
const WALK_POWER_PER_ITERATION: f32 = 0.01;

pub struct Agent {
    pheromones: [PheromoneMatrix; NUM_PLAYERS],
    plan: [Vec<Milestone>; NUM_PLAYERS],
    rng: StdRng,
}
impl Agent {
    pub fn new(view: &View) -> Self {
        Self {
            pheromones: [
                PheromoneMatrix::new(ME, view),
                PheromoneMatrix::new(ENEMY, view),
            ],
            plan: [Vec::new(), Vec::new()],
            rng: StdRng::seed_from_u64(0x1234567890abcdef),
        }
    }

    pub fn act(&mut self, view: &View, state: &State) -> Vec<Action> {
        // Before using the previous plans - clean them up so we don't waste iterations on unnecessary milestones
        for plan in self.plan.iter_mut() {
            plan.retain(|m| !m.is_complete(state));
        }

        let (initial, best, stats) = self.generate_plan_for_player(ADVERSARIAL_MS, ENEMY, view, state);
        eprintln!("adversary {} -> {}: in {:.0} ms ({}/{} successful iterations)", -initial.score, -best.score, stats.elapsed_ms, stats.num_improvements, stats.num_evaluated);

        let (initial, best, stats) = self.generate_plan_for_player(SEARCH_MS, ME, view, state);
        eprintln!("best {} -> {}: in {:.0} ms ({}/{} successful iterations)", initial.score, best.score, stats.elapsed_ms, stats.num_improvements, stats.num_evaluated);

        let commands = planning::enact_plan(ME, &best.plan, view, state);

        let mut actions = movement::assignments_to_actions(&commands.assignments);
        actions.push(Action::Message { text: {
            if best.endgame.crystals[ME] >= best.endgame.crystals[ENEMY] {
                "Good game :)".to_string()
            } else {
                "Congratulations!".to_string()
            }
        }});

        eprintln!("best: {}", best);
        eprintln!(
            "Endgame: tick={}, crystals=[{} vs {}], ants=[{} vs {}]",
            best.endgame.tick,
            best.endgame.crystals[0], best.endgame.crystals[1],
            best.endgame.total_ants[0], best.endgame.total_ants[1],
        );
        eprintln!("Goals: {}", commands);

        actions
    }

    fn generate_plan_for_player(&mut self, max_ms: u128, player: usize, view: &View, state: &State) -> (Candidate,Candidate,GenerationStats) {
        let start = Instant::now();
        let enemy = (player + 1) % NUM_PLAYERS;

        let initial_plan = self.plan[player].clone();
        let enemy_plan = self.plan[enemy].clone();

        let initial = Candidate::evaluate(player, initial_plan, &enemy_plan, view, state);
        let mut best = initial.clone();

        let mut num_evaluated = 1;
        let mut num_improvements = 0;

        let mut scorer = QuantileEstimator::new();
        scorer.insert(best.score);

        while start.elapsed().as_millis() < max_ms {
            let walk_power = WALK_MIN_POWER + WALK_POWER_PER_ITERATION * num_evaluated as f32;

            let (plan, walks) = self.pheromones[player].generate(walk_power, &mut self.rng, |cell| {
                state.resources[cell] > 0
            });
            let candidate = Candidate::evaluate(player, plan, &enemy_plan, view, state);
            num_evaluated += 1;

            let quantile = scorer.quantile(candidate.score);
            scorer.insert(candidate.score);
            self.pheromones[player].learn(quantile, &walks);

            if candidate.is_improvement(&best) {
                best = candidate;
                num_improvements += 1;
            }
        }

        let stats = GenerationStats {
            num_evaluated,
            num_improvements,
            elapsed_ms: start.elapsed().as_millis() as f32,
        };

        self.plan[player] = best.plan.clone();
        (initial, best, stats)
    }
}

struct GenerationStats {
    pub num_evaluated: i32,
    pub num_improvements: i32,
    pub elapsed_ms: f32,
}

#[derive(Clone)]
struct Candidate {
    pub plan: Vec<Milestone>,
    pub score: f32,
    pub endgame: Endgame,
}
impl Candidate {
    pub(self) fn evaluate(player: usize, plan: Vec<Milestone>, countermoves: &Vec<Milestone>, view: &View, state: &State) -> Self {
        let plans = match player {
            ME => [&plan, countermoves],
            ENEMY => [countermoves, &plan],
            unknown => panic!("Unknown player: {}", unknown),
        };
        let (score, endgame) = evaluation::rollout(player, plans, view, state);
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