use std::time::Instant;

use rand::prelude::*;

use super::inputs::*;
use super::movement;
use super::view::*;
use super::planning::{self,*};
use super::solving::{Candidate,Solver,SolverSession};
use super::valuation::SpawnEvaluator;

const ADVERSARY_MS: u128 = 10;
const SEARCH_MS: u128 = 80;

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
        let start = Instant::now();
        eprintln!("Crystals: me={}, enemy={}", state.crystals[0], state.crystals[1]);
        eprintln!("Ants: me={}, enemy={}", state.total_ants[0], state.total_ants[1]);

        for plan in self.plans.iter_mut() {
            Milestone::reap(plan, state);
        }

        let mut enemy_session = SolverSession::new(Candidate::evaluate(ENEMY, self.plans[ENEMY].clone(), &self.plans[ME], view, state));
        let initial_adversarial_score = -enemy_session.best.score;
        while start.elapsed().as_millis() < ADVERSARY_MS {
            self.solvers[ENEMY].step(&mut enemy_session, &self.plans[ME], view, state, &mut self.rng);
        }
        self.plans[ENEMY] = enemy_session.best.plan.clone();

        let mut my_session = SolverSession::new(Candidate::evaluate(ME, self.plans[ME].clone(), &self.plans[ENEMY], view, state));
        let initial_score = my_session.best.score;
        eprintln!("Initial: {}", my_session.best);
        while start.elapsed().as_millis() < SEARCH_MS + ADVERSARY_MS {
            self.solvers[ME].step(&mut my_session, &self.plans[ENEMY], view, state, &mut self.rng);
        }
        self.plans[ME] = my_session.best.plan.clone();

        let best = my_session.best;
        let adversary = enemy_session.best;
        let stats = [my_session.stats, enemy_session.stats];

        let num_evaluated = stats.iter().map(|s| s.num_evaluated()).sum::<i32>();
        eprintln!("{:.0} -> {:.0} -> {:.0} -> found best plan in {:.0} ms ({} iterations)",
            initial_adversarial_score, initial_score, best.score,
            start.elapsed().as_millis() as f32,
            num_evaluated);
        eprintln!("Successful: {}/{} generations, {}/{} mutations",
            stats.iter().map(|s| s.num_successful_generations()).sum::<i32>(),
            stats.iter().map(|s| s.num_generations()).sum::<i32>(),
            stats.iter().map(|s| s.num_successful_mutations()).sum::<i32>(),
            stats.iter().map(|s| s.num_mutations()).sum::<i32>());

        let harvests = [
            SpawnEvaluator::new(ME, view, state),
            SpawnEvaluator::new(ENEMY, view, state),
        ];

        let commands = planning::enact_plan(ME, &best.plan, view, state);
        let countermoves = planning::enact_plan(ENEMY, &adversary.plan, view, state);

        let mut actions = movement::assignments_to_actions(&commands.assignments);
        actions.push(Action::Message { text: format!("{}", num_evaluated) });

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
    }
}