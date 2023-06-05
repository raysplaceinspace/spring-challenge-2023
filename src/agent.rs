use rand::prelude::*;

use super::inputs::*;
use super::movement;
use super::view::*;
use super::opponents;
use super::planning::{self,*};
use super::solving::Solver;
use super::valuation::SpawnEvaluator;

const SEARCH_MS: u128 = 90;

pub struct Agent {
    solver: Solver,
    plan: Vec<Milestone>,
    rng: StdRng,
}
impl Agent {
    pub fn new(player: usize, view: &View) -> Self {
        Self {
            solver: Solver::new(player, view),
            plan: Vec::new(),
            rng: StdRng::seed_from_u64(0x1234567890abcdef),
        }
    }

    pub fn act(&mut self, view: &View, state: &State) -> Vec<Action> {
        let initial_plan = Milestone::reap(self.plan.clone(), state);
        let (initial, best, stats) = self.solver.solve(SEARCH_MS, initial_plan, view, state, &mut self.rng);

        let harvests = [
            SpawnEvaluator::new(ME, view, state),
            SpawnEvaluator::new(ENEMY, view, state),
        ];

        let commands = planning::enact_plan(ME, &best.plan, view, state);
        let countermoves = opponents::enact_countermoves(ENEMY, view, state);

        let mut actions = movement::assignments_to_actions(&commands.assignments);
        actions.push(Action::Message { text: format!("{}", stats.num_evaluated) });

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

        eprintln!("{}", self.solver);

        self.plan = best.plan;
        actions

        // Uncomment this to visualise the opponent's countermoves
        /*
        let moves = opponents::enact_countermoves(ME, view, state);
        movement::assignments_to_actions(&moves.assignments)
        */
    }
}