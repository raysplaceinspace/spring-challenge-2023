use rand::prelude::*;
use std::fmt::Display;
use super::evaluation::{self,Endgame};
use super::inputs::{ME,ENEMY};
use super::planning::Milestone;
use super::view::*;

const SELECTION_POWER: i32 = 2;

const INITIAL_QUANTILE: f32 = 0.5;
const INITIAL_QUANTILE_DECAY_BASE: f32 = 0.5;
const QUANTILE_SAMPLE_LIMIT: usize = 32;

const LEARNING_RATE: f32 = 0.01;

#[derive(Copy,Clone,Debug)]
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

#[derive(Default)]
pub struct SolverStats {
    num_evaluated: i32,
    num_iterations: [i32; NUM_SOLVERS],
    num_successes: [i32; NUM_SOLVERS],
}
impl SolverStats {
    pub fn num_evaluated(&self) -> i32 { self.num_evaluated }

    pub fn num_mutations(&self) -> i32 { self.num_iterations[SolverType::Mutation as usize] }
    pub fn num_successful_mutations(&self) -> i32 { self.num_successes[SolverType::Mutation as usize] }

    pub fn num_generations(&self) -> i32 { self.num_iterations[SolverType::Generation as usize] }
    pub fn num_successful_generations(&self) -> i32 { self.num_successes[SolverType::Generation as usize] }
}

pub struct Solver {
    player: usize,
    solver_quantiles: [f32; NUM_SOLVERS],
    generator: PheromoneMatrix,
    mutator: Mutator,
}
impl Solver {
    pub fn new(player: usize, view: &View) -> Self {
        Self {
            player,
            solver_quantiles: [INITIAL_QUANTILE; NUM_SOLVERS],
            generator: PheromoneMatrix::new(player, view),
            mutator: Mutator::new(),
        }
    }

    pub fn step(&mut self, session: &mut SolverSession, countermoves: &Vec<Milestone>, view: &View, state: &State, rng: &mut StdRng) {
        // Generate solution
        let solver = SOLVERS[select_weighted(&self.solver_quantiles, rng)];
        let (plan, lesson) = match solver {
            SolverType::Generation => {
                let (plan, walks) = self.generator.generate(rng, |cell| {
                    state.resources[cell] > 0
                });
                (plan, Lesson::Generation(walks))
            },
            SolverType::Mutation => {
                let mut plan = session.best.plan.clone();
                let mutation = self.mutator.mutate(&mut plan, rng);
                (plan, Lesson::Mutation(mutation))
            },
        };
        session.stats.num_iterations[solver as usize] += 1;

        // Evaluate solution
        let candidate = Candidate::evaluate(self.player, plan, countermoves, view, state);
        session.stats.num_evaluated += 1;

        // Learn quantiles
        let quantile = session.scorer.quantile(candidate.score);
        session.scorer.insert(candidate.score);
        learn_quantile(&mut self.solver_quantiles[solver as usize], quantile);
        match lesson {
            Lesson::Generation(walks) => self.generator.learn(quantile, &walks),
            Lesson::Mutation(mutation) => self.mutator.learn(quantile, mutation),
        }

        // Update best
        if candidate.is_improvement(&session.best) {
            session.best = candidate;
            session.stats.num_successes[solver as usize] += 1;
        }
    }
}
impl Display for Solver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Solvers:")?;
        for (index,quantile) in self.solver_quantiles.iter().enumerate() {
            write!(f, " {:?}={:.2}", SOLVERS[index], quantile)?;
        }
        writeln!(f)?;

        writeln!(f, "Mutations: {}", self.mutator)?;

        Ok(())
    }
}

pub struct SolverSession {
    pub stats: SolverStats,
    pub best: Candidate,

    scorer: QuantileEstimator,
}
impl SolverSession {
    pub fn new(initial: Candidate) -> Self {
        Self {
            stats: SolverStats::default(),
            scorer: QuantileEstimator::new(),
            best: initial,
        }
    }
}

#[derive(Clone)]
pub struct Candidate {
    pub plan: Vec<Milestone>,
    pub score: f32,
    pub endgame: Endgame,
}
impl Candidate {
    pub fn evaluate(player: usize, plan: Vec<Milestone>, countermoves: &Vec<Milestone>, view: &View, state: &State) -> Self {
        let plans = match player {
            ME => [&plan, countermoves],
            ENEMY => [countermoves, &plan],
            unknown => panic!("Unknown player: {}", unknown),
        };
        let (payoff, endgame) = evaluation::rollout(plans, view, state);
        let score = match player {
            ME => payoff,
            ENEMY => -payoff,
            unknown => panic!("Unknown player: {}", unknown),
        };
        Self { plan, score, endgame }
    }

    pub fn is_improvement(&self, other: &Self) -> bool {
        self.score > other.score
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


#[derive(Clone,Copy,Debug,PartialEq,PartialOrd)]
pub struct Quantile(f32);
impl Quantile {
    pub fn new(quantile: f32) -> Self {
        Self(quantile)
    }
    pub fn f32(&self) -> f32 { self.0 }
}
impl Display for Quantile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone,Copy,Debug,PartialEq,PartialOrd)]
struct Sample(f32);
impl Sample {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}
impl Ord for Sample {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl Eq for Sample {}

pub struct QuantileEstimator {
    samples: Vec<Sample>,
    sample_limit: usize,
    reap_offset: usize,
}
impl QuantileEstimator {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            sample_limit: QUANTILE_SAMPLE_LIMIT,
            reap_offset: 0,
        }
    }

    pub fn insert(&mut self, score: f32) {
        let sample = Sample::new(score);
        let index = self.samples.binary_search(&sample).unwrap_or_else(|i| i);
        self.samples.insert(index, sample);
        self.reap();
    }

    pub fn quantile(&self, score: f32) -> Quantile {
        if self.samples.len() <= 1 { return Quantile::new(INITIAL_QUANTILE) }

        let sample = Sample::new(score);
        let sample_quantile = match self.samples.binary_search(&sample) {
            Ok(index) => index as f32 / self.samples.len() as f32,
            Err(index) => {
                if index <= 0 { 0.0 } // below the lowest value
                else if index >= self.samples.len() { 1.0 } // above the highest value
                else {
                    let below = index - 1;
                    let above = index; // index is the sorted insertion point so is above the value

                    let low = self.samples[below].0;
                    let high = self.samples[above].0;
                    if low == high { index as f32 / self.samples.len() as f32 }
                    else {
                        // Linearly interpolate where this score sits between its two bounds
                        let subindex = (score - low) / (high - low);
                        (below as f32 + subindex) / self.samples.len() as f32
                    }
                }
            },
        };

        // If there are only 2 samples, squash between 0.25-0.75. Only 3, squash between 0.16 - 0.86.
        // As the number of samples increases, the confidence factor approaches 1.0.
        let confidence = 1.0 - 1.0 / self.samples.len() as f32;
        let population_quantile = 0.5 + (sample_quantile - 0.5) * confidence;
        Quantile::new(population_quantile)
    }

    /// Remove items from the array if it is too large, but keep the distribution approximately the same
    fn reap(&mut self) {
        if self.samples.len() <= self.sample_limit { return }

        let divisor = 2;

        // We use an offset to ensure we reap fairly across the distribution
        // If we always removed offset values rounded down,
        // over time we will bias the sample towards removing lower values and keeping higher values
        let offset = self.take_reap_offset() % divisor;

        let mut index = 0;
        self.samples.retain(|_| {
            let keep = index % divisor == offset;
            index += 1;
            keep
        });
    }

    fn take_reap_offset(&mut self) -> usize {
        let offset = self.reap_offset;
        self.reap_offset += 1;
        offset
    }
}


pub struct PheromoneMatrix {
    /// cell to vein id
    id_lookup: Box<[Option<usize>]>,

    /// base id to cell
    bases: Box<[usize]>,

    /// vein id to cell
    veins: Box<[usize]>,

    /// base -> head -> quantile
    /// average quantile that solutions beginning with this cell have
    head_quantiles: Box<[Box<[f32]>]>,

    /// average quantile that solutions traversing this cell-to-cell link have
    link_quantiles: Box<[Box<[f32]>]>,
}
impl PheromoneMatrix {
    pub fn new(player: usize, view: &View) -> Self {
        let num_cells = view.layout.cells.len();

        let mut id_lookup = Vec::new();
        id_lookup.resize(num_cells, None);

        let mut veins = Vec::new();
        for (index,cell) in view.layout.cells.iter().enumerate() {
            if cell.initial_resources > 0 {
                let vein = veins.len();
                id_lookup[index] = Some(vein);
                veins.push(index);
            }
        }

        let mut head_quantiles = Vec::new();
        let bases = view.layout.bases[player].clone();
        for _ in 0..bases.len() { // Assume both players have the same number of bases
            // All cells have the same chance of being selected from the base
            let mut quantiles = Vec::new();
            quantiles.resize(veins.len(), INITIAL_QUANTILE);
            head_quantiles.push(quantiles.into_boxed_slice());
        }

        let mut link_quantiles = Vec::new();
        for &source in veins.iter() {
            // Give closer cells a higher initial quantile
            let mut targets = veins.clone();
            targets.sort_by_key(|&target| view.paths.distance_between(source, target));

            let mut quantiles = Vec::new();
            quantiles.resize(veins.len(), INITIAL_QUANTILE);
            for (index, &target) in targets.iter().enumerate() {
                let vein = id_lookup[target].expect("target missing id");
                quantiles[vein] = INITIAL_QUANTILE_DECAY_BASE.powi(index as i32);
            }

            link_quantiles.push(quantiles.into_boxed_slice());
        }

        Self {
            id_lookup: id_lookup.into_boxed_slice(),
            bases,
            veins: veins.into_boxed_slice(),
            head_quantiles: head_quantiles.into_boxed_slice(),
            link_quantiles: link_quantiles.into_boxed_slice(),
        }
    }

    pub fn generate(&self, rng: &mut StdRng, is_allowed: impl Fn(usize) -> bool) -> (Vec<Milestone>,Box<[Walk]>) {
        let mut allowed: Vec<bool> = self.veins.iter().map(|&cell| is_allowed(cell)).collect();
        let mut num_remaining = allowed.iter().filter(|&&allowed| allowed).count() as i32;

        let mut walks = Vec::with_capacity(self.bases.len());
        for base_id in 0..self.bases.len() {
            walks.push(Walk::new(base_id));
        }

        let mut priorities = Vec::new();
        while num_remaining > 0 {
            let base_id = (num_remaining as usize) % walks.len();
            let walk = &mut walks[base_id];

            let quantiles =
                if let Some(&previous) = walk.veins.last() {
                    &self.link_quantiles[previous]
                } else {
                    &self.head_quantiles[base_id]
                };

            let mut total = 0.0;
            for vein in 0..quantiles.len() {
                if allowed[vein] {
                    total += quantiles[vein].powi(SELECTION_POWER);
                }
            }

            let selector = total * rng.gen::<f32>();
            let mut cumulative = 0.0;
            
            let mut selected = None;
            for vein in 0..quantiles.len() {
                if allowed[vein] {
                    cumulative += quantiles[vein].powi(SELECTION_POWER);
                    if selector <= cumulative {
                        selected = Some(vein);
                        break;
                    }
                }
            }

            if let Some(vein) = selected {
                allowed[vein] = false;
                num_remaining -= 1;

                walk.veins.push(vein);
                priorities.push(Milestone::Harvest(self.veins[vein]));

            } else {
                panic!("Failed to select a cell: total={}, cumulative={}, selector={}", total, cumulative, selector)
            }
        }

        (priorities, walks.into_boxed_slice())
    }

    pub fn learn(&mut self, quantile: Quantile, walks: &[Walk]) {
        for walk in walks.iter() {
            let mut previous = None;
            for &cell in walk.veins.iter() {
                if let Some(vein) = self.id_lookup[cell] {
                    let quantiles =
                        if let Some(previous) = previous {
                            &mut self.link_quantiles[previous]
                        } else {
                            &mut self.head_quantiles[walk.base_id]
                        };
                    learn_quantile(&mut quantiles[vein], quantile);

                    previous = Some(vein);
                }
            }
        }
    }
}
impl Display for PheromoneMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for (base_id, &base) in self.bases.iter().enumerate() {
            if is_first {
                is_first = false;
            } else {
                writeln!(f)?;
            }

            write!(f, "Base {}:", base)?;
            for (vein,quantile) in self.head_quantiles[base_id].iter().enumerate() {
                write!(f, " {:?}={:.2}", self.veins[vein], quantile)?;
            }
        }
        Ok(())
    }
}

pub struct Walk {
    pub base_id: usize, // base id (not the cell id)
    pub veins: Vec<usize>, // vein ids
}
impl Walk {
    pub(self) fn new(base_id: usize) -> Self {
        Self {
            base_id,
            veins: Vec::new(),
        }
    }
}


#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum Mutation {
    Bubble,
    Move,
    Swap,
    Shift,
    Reverse,
    Barrier,
}

const NUM_MUTATIONS: usize = 6;
const MUTATIONS: [Mutation; NUM_MUTATIONS] = [
    Mutation::Bubble,
    Mutation::Move,
    Mutation::Swap,
    Mutation::Shift,
    Mutation::Reverse,
    Mutation::Barrier,
];

pub struct Mutator {
    mutation_quantiles: [f32; NUM_MUTATIONS],
}
impl Mutator {
    pub fn new() -> Self {
        Self {
            mutation_quantiles: [INITIAL_QUANTILE; NUM_MUTATIONS],
        }
    }

    pub fn mutate(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> Mutation {
        let mutation = MUTATIONS[select_weighted(&self.mutation_quantiles, rng)];
        match mutation {
            Mutation::Bubble => bubble_mutation(plan, rng),
            Mutation::Move => move_mutation(plan, rng),
            Mutation::Swap => swap_mutation(plan, rng),
            Mutation::Shift => shift_mutation(plan, rng),
            Mutation::Reverse => reverse_mutation(plan, rng),
            Mutation::Barrier => barrier_mutation(plan, rng),
        };
        mutation
    }

    pub fn learn(&mut self, quantile: Quantile, mutation: Mutation) {
        learn_quantile(&mut self.mutation_quantiles[mutation as usize], quantile);
    }
}
impl Display for Mutator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for (index,quantile) in self.mutation_quantiles.iter().enumerate() {
            if is_first {
                is_first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{:?}={:.2}", MUTATIONS[index], quantile)?;
        }
        Ok(())
    }
}

fn bubble_mutation(plan: &mut Vec<Milestone>, rng: &mut StdRng) {
    if plan.len() < 2 { return }
    let index = rng.gen_range(0 .. (plan.len()-1));
    plan.swap(index, index+1);
}

fn move_mutation(plan: &mut Vec<Milestone>, rng: &mut StdRng) {
    if plan.len() < 2 { return }
    let from = rng.gen_range(0 .. plan.len());
    let mut to = rng.gen_range(0 .. (plan.len()-1)); // -1 because insertion array is one shorter, but +1 because we want to be able to insert at the end as well, but then -1 again because we don't want to pick the same location twice
    if to >= from { to += 1 }

    let milestone = plan.remove(from);
    if to >= plan.len() {
        plan.push(milestone);
    } else {
        plan.insert(to, milestone);
    }
}

fn swap_mutation(plan: &mut Vec<Milestone>, rng: &mut StdRng) {
    if plan.len() < 2 { return }
    let from = rng.gen_range(0 .. plan.len());
    let mut to = rng.gen_range(0 .. (plan.len() - 1));
    if to >= from { to += 1 }

    plan.swap(from, to);
}

fn shift_mutation(plan: &mut Vec<Milestone>, rng: &mut StdRng) {
    if plan.len() < 2 { return }
    let length = rng.gen_range(1 .. plan.len());
    let from = rng.gen_range(0 .. (plan.len() - length + 1));
    let mut to = rng.gen_range(0 .. (plan.len() - length));
    if to >= from { to += 1 }

    let elems: Vec<Milestone> = plan.splice(from .. (from + length), std::iter::empty()).collect();
    plan.splice(to .. to, elems);
}

fn reverse_mutation(plan: &mut Vec<Milestone>, rng: &mut StdRng) {
    if plan.len() < 2 { return }
    let length = rng.gen_range(1 ..= plan.len());
    let from = rng.gen_range(0 ..= (plan.len() - length));

    let elems: Vec<Milestone> = plan.splice(from .. (from + length), std::iter::empty()).collect();
    plan.splice(from .. from, elems.into_iter().rev());
}

fn barrier_mutation(plan: &mut Vec<Milestone>, rng: &mut StdRng) {
    if plan.len() < 2 { return }
    let index = rng.gen_range(0 .. plan.len());

    match &plan[index] {
        Milestone::Harvest(_) => {
            // Insert a barrier after the harvest
            let barrier_index = index + 1;
            if barrier_index < plan.len() && plan[barrier_index] != Milestone::Barrier {
                plan.insert(barrier_index, Milestone::Barrier);
            }
        },
        Milestone::Barrier => {
            // Remove a barrier if we encounter one
            plan.remove(index);
        },
    }
}


fn learn_quantile(weight: &mut f32, quantile: Quantile) {
    *weight = (1.0 - LEARNING_RATE) * *weight + LEARNING_RATE * quantile.f32();
}

fn select_weighted(weights: &[f32], rng: &mut StdRng) -> usize {
    let total = weights.iter().map(|x| x.powi(SELECTION_POWER)).sum::<f32>();
    let selector = total * rng.gen::<f32>();

    let mut cumulative = 0.0;
    for (index, &quantile) in weights.iter().enumerate() {
        cumulative += quantile.powi(SELECTION_POWER);
        if selector <= cumulative {
            return index;
        }
    }
    panic!("Failed to select from weighted array");
}