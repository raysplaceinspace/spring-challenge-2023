use rand::prelude::*;

use super::inputs::*;
use super::movement;
use super::view::*;
use super::planning::{self,*};
use super::solving::Solver;
use super::valuation::SpawnEvaluator;

const ADVERSARIAL_MS: u128 = 30;
const SEARCH_MS: u128 = 60;

pub struct Agent {
    solvers: [Solver; NUM_PLAYERS],
    plans: [Vec<Milestone>; NUM_PLAYERS],
    rng: StdRng,
}
impl Agent {
    pub fn new(view: &View) -> Self {
        Self {
            solvers: [
                Solver::new(ME, view),
                Solver::new(ENEMY, view),
            ],
            plans: [Vec::new(), Vec::new()],
            rng: StdRng::seed_from_u64(0x1234567890abcdef),
        }
    }

    pub fn act(&mut self, view: &View, state: &State) -> Vec<Action> {
        let mut num_evaluated = 0;
        eprintln!("Crystals: me={}, enemy={}", state.crystals[0], state.crystals[1]);
        eprintln!("Ants: me={}, enemy={}", state.total_ants[0], state.total_ants[1]);

        for plan in self.plans.iter_mut() {
            Milestone::reap(plan, state);
        }

        let (initial, adversary, stats) =
            self.solvers[ENEMY].solve(ADVERSARIAL_MS, self.plans[ENEMY].clone(), &self.plans[ME], view, state, &mut self.rng);
        eprintln!("{:.0} -> {:.0} -> found adversarial plan in {:.0} ms ({} iterations)", -initial.score, -adversary.score, stats.elapsed_ms as f32, stats.num_evaluated);
        eprintln!("Successful: {}/{} generations, {}/{} mutations", stats.num_successful_generations, stats.num_generations, stats.num_successful_mutations, stats.num_mutations);
        self.plans[ENEMY] = adversary.plan.clone();
        num_evaluated += stats.num_evaluated;

        let (initial, best, stats) =
            self.solvers[ME].solve(SEARCH_MS, self.plans[ME].clone(), &self.plans[ENEMY], view, state, &mut self.rng);
        eprintln!("{:.0} -> {:.0} -> found best plan in {:.0} ms ({} iterations)", initial.score, best.score, stats.elapsed_ms as f32, stats.num_evaluated);
        eprintln!("Successful: {}/{} generations, {}/{} mutations", stats.num_successful_generations, stats.num_generations, stats.num_successful_mutations, stats.num_mutations);
        self.plans[ME] = best.plan.clone();
        num_evaluated += stats.num_evaluated;

        let harvests = [
            SpawnEvaluator::new(ME, view, state),
            SpawnEvaluator::new(ENEMY, view, state),
        ];

        let commands = planning::enact_plan(ME, &best.plan, view, state);
        let countermoves = planning::enact_plan(ENEMY, &adversary.plan, view, state);

        let mut actions = movement::assignments_to_actions(&commands.assignments);
        actions.push(Action::Message { text: format!("{}", num_evaluated) });

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

        eprintln!("{}", self.solvers[ME]);

        actions

        // Uncomment this to visualise the opponent's countermoves
        /*
        let moves = opponents::enact_countermoves(ME, view, state);
        movement::assignments_to_actions(&moves.assignments)
        */
    }
}