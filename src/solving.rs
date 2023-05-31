use rand::prelude::*;
use std::fmt::Display;
use super::view::*;

const INITIAL_QUANTILE: f32 = 0.5;
const INITIAL_QUANTILE_DECAY_BASE: f32 = 0.75;
const QUANTILE_SAMPLE_LIMIT: usize = 32;

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
    /// cell to id
    id_lookup: Box<[Option<usize>]>,

    /// id to cell
    cell_lookup: Box<[usize]>,

    /// average quantile that solutions beginning with this cell have
    head_quantiles: Box<[f32]>,

    /// average quantile that solutions traversing this cell-to-cell link have
    link_quantiles: Box<[Box<[f32]>]>,
}
impl PheromoneMatrix {
    pub fn new(view: &View) -> Self {
        let num_cells = view.layout.cells.len();

        let mut id_lookup = Vec::new();
        id_lookup.resize(num_cells, None);

        let mut cell_lookup = Vec::new();
        for (index,cell) in view.layout.cells.iter().enumerate() {
            if cell.initial_resources > 0 {
                let id = cell_lookup.len();
                id_lookup[index] = Some(id);
                cell_lookup.push(index);
            }
        }

        let num_ids = cell_lookup.len();

        let mut head_quantiles = Vec::new();
        head_quantiles.resize(num_ids, INITIAL_QUANTILE);

        let mut link_quantiles = Vec::new();
        for &source in cell_lookup.iter() {
            // Give closer cells a higher initial quantile
            let mut targets = cell_lookup.clone();
            targets.sort_by_key(|&target| view.paths.distance_between(source, target));

            let mut quantiles = Vec::new();
            quantiles.resize(num_ids, INITIAL_QUANTILE);
            for (index, &target) in targets.iter().enumerate() {
                let id = id_lookup[target].expect("target missing id");
                quantiles[id] = INITIAL_QUANTILE_DECAY_BASE.powi(index as i32);
            }

            link_quantiles.push(quantiles.into_boxed_slice());
        }

        Self {
            id_lookup: id_lookup.into_boxed_slice(),
            cell_lookup: cell_lookup.into_boxed_slice(),
            head_quantiles: head_quantiles.into_boxed_slice(),
            link_quantiles: link_quantiles.into_boxed_slice(),
        }
    }

    pub fn walk<'a>(&'a self, power: f32, rng: &'a mut StdRng, is_allowed: impl Fn(usize) -> bool) -> impl Iterator<Item=usize> + 'a {
        let mut allowed: Vec<bool> = self.cell_lookup.iter().map(|&cell| is_allowed(cell)).collect();
        let mut num_remaining = allowed.iter().filter(|&&allowed| allowed).count() as i32;

        let mut previous: Option<usize> = None;
        std::iter::from_fn(move || {
            if num_remaining <= 0 { return None };

            let row =
                if let Some(previous) = previous {
                    &self.link_quantiles[previous]
                } else {
                    &self.head_quantiles
                };

            let mut total = 0.0;
            for id in 0..row.len() {
                if allowed[id] {
                    total += row[id].powf(power);
                }
            }

            let selector = total * rng.gen::<f32>();
            let mut cumulative = 0.0;
            for id in 0..row.len() {
                if allowed[id] {
                    cumulative += row[id].powf(power);
                    if selector <= cumulative {
                        allowed[id] = false;
                        num_remaining -= 1;
                        previous = Some(id);

                        return Some(self.cell_lookup[id]);
                    }
                }
            }

            panic!("Failed to select a cell: total={}, cumulative={}, selector={}", total, cumulative, selector)
        })
    }

    pub fn learn(&mut self, quantile: Quantile, learning_rate: f32, path: impl Iterator<Item=usize>) {
        let mut previous = None;
        for cell in path {
            if let Some(id) = self.id_lookup[cell] {
                let weight: &mut f32 =
                    if let Some(previous) = previous {
                        let row: &mut Box<[f32]> = &mut self.link_quantiles[previous];
                        &mut row[id]
                    } else {
                        &mut self.head_quantiles[id]
                    };
                *weight = (1.0 - learning_rate) * *weight + learning_rate * quantile.f32();

                previous = Some(id);
            }
        }
    }
}