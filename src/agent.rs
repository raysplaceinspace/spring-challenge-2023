use rand::prelude::*;

use super::inputs::*;
use super::movement;
use super::view::*;
use super::opponents;
use super::planning::{self,*};
use super::solving::Solver;
use super::valuation::HarvestAndSpawnEvaluator;

const SEARCH_MS: u128 = 90;

pub struct Agent {
    solver: Solver,
    previous_plan: Option<Vec<Milestone>>,
    rng: StdRng,
}
impl Agent {
    pub fn new(player: usize, view: &View) -> Self {
        Self {
            solver: Solver::new(player, view),
            previous_plan: None,
            rng: StdRng::seed_from_u64(0x1234567890abcdef),
        }
    }

    pub fn act(&mut self, view: &View, state: &State) -> Vec<Action> {
        let initial_plan = match self.previous_plan.take() {
            Some(plan) => Milestone::reap(plan, state),
            None => Vec::new(),
        };
        let (initial, best, stats) = self.solver.solve(SEARCH_MS, initial_plan, view, state, &mut self.rng);

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

        eprintln!("{}: found best plan in {:.0} ms ({} iterations)", state.tick, stats.elapsed_ms as f32, stats.num_evaluated);
        eprintln!("Successful: {}/{} generations, {}/{} mutations", stats.num_successful_generations, stats.num_generations, stats.num_successful_mutations, stats.num_mutations);
        eprintln!("Initial: {}", initial);
        eprintln!("Best: {}", best);
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