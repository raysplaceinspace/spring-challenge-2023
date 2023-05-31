use rand::prelude::*;
use std::fmt::Display;
use super::planning::Milestone;
use super::view::*;

const INITIAL_QUANTILE: f32 = 0.5;
const INITIAL_QUANTILE_DECAY_BASE: f32 = 0.75;
const QUANTILE_SAMPLE_LIMIT: usize = 32;

const LEARNING_RATE: f32 = 0.01;

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

    pub fn generate(&self, power: f32, rng: &mut StdRng, is_allowed: impl Fn(usize) -> bool) -> (Vec<Milestone>,Box<[Walk]>) {
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
                    total += quantiles[vein].powf(power);
                }
            }

            let selector = total * rng.gen::<f32>();
            let mut cumulative = 0.0;
            
            let mut selected = None;
            for vein in 0..quantiles.len() {
                if allowed[vein] {
                    cumulative += quantiles[vein].powf(power);
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
                priorities.push(Milestone::new(self.veins[vein]));

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
                    let weight = &mut quantiles[vein];
                    *weight = (1.0 - LEARNING_RATE) * *weight + LEARNING_RATE * quantile.f32();

                    previous = Some(vein);
                }
            }
        }
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