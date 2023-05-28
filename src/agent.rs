use std::collections::HashSet;
use std::fmt::Display;

use super::model::*;
use super::view::*;

pub fn act(view: &View, state: &State) -> Vec<Action> {
    let mut actions = Vec::new();

    let my_base = view.layout.bases[Player::Me as usize][0];
    let total_ants: i32 = state.num_ants_per_cell[Player::Me as usize].iter().cloned().sum();
    let initial_crystals = view.initial_crystals;
    let remaining_crystals: i32 =
        view.layout.cells.iter().enumerate()
        .filter(|(_,cell)| cell.content == Some(Content::Crystals))
        .map(|(i,_)| state.resources_per_cell[i])
        .sum();

    let mut sources = HashSet::new();
    sources.insert(my_base);

    let mut harvests = HarvestCounters::new();
    let mut total_distance = 0;
    let mut branches = Vec::new();

    let mut unharvested: HashSet<usize> = (0..view.layout.cells.len()).filter(|i| state.resources_per_cell[*i] > 0).collect();
    while !unharvested.is_empty() {
        let initial_collection_rate = calculate_collection_rate(total_ants, total_distance, initial_crystals, remaining_crystals, &harvests);

        if let Some(best) =
            unharvested.iter()
            .filter_map(|&target| {
                let content = view.layout.cells[target].content?;
                let (distance, source) = sources.iter().map(|&source| (view.paths.distance_between(source, target),source)).min()?;
                Some(HarvestBranch { distance, source, target, content })
            })
            .map(|branch| {
                let harvests = harvests.add(&branch.content);
                let new_collection_rate = calculate_collection_rate(total_ants, total_distance + branch.distance, initial_crystals, remaining_crystals, &harvests);
                HarvestCandidate { branch, rate: new_collection_rate }
            })
            .max_by_key(|candidate| (candidate.rate, candidate.branch.target)) {

            eprintln!("considered harvesting <{}> (distance {}): {} -> {}", best.branch.target, best.branch.distance, initial_collection_rate, best.rate);
            if best.rate >= initial_collection_rate {
                unharvested.remove(&best.branch.target);
                sources.insert(best.branch.target);

                total_distance += best.branch.distance;
                harvests = harvests.add(&best.branch.content);

                branches.push(best.branch);

            } else {
                // Best harvest not worth it, so none others will be either
                break;
            }

        } else {
            // No more harvests possible
            break
        }
    }

    for branch in branches {
        actions.push(Action::Line {
            source: branch.source,
            target: branch.target,
            strength: 1,
        });
    }

    actions
}

fn calculate_collection_rate(total_ants: i32, total_distance: i32, initial_crystals: i32, remaining_crystals: i32, harvests: &HarvestCounters) -> HarvestRate {
    const EGG_PAYOFF_FACTOR: f32 = 2.0;
    if total_distance <= 0 { return HarvestRate::new(0.0) }
    let per_cell = total_ants / total_distance; // intentional integer division since ants can't be split

    let crystal_harvest_rate = harvests.num_crystal_harvests * per_cell;
    let egg_harvest_rate = harvests.num_egg_harvests * per_cell;
    let egg_harvest_weighting = EGG_PAYOFF_FACTOR * (remaining_crystals as f32 / initial_crystals as f32);

    let rate = crystal_harvest_rate as f32 + (egg_harvest_rate as f32 * egg_harvest_weighting);
    HarvestRate::new(rate)
}

#[derive(Clone,Copy,PartialEq,PartialOrd)]
struct HarvestRate(f32);
impl HarvestRate {
    pub fn new(rate: f32) -> Self {
        Self(rate)
    }
}
impl Eq for HarvestRate {
}
impl Ord for HarvestRate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl Display for HarvestRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}

struct HarvestCandidate {
    pub branch: HarvestBranch,
    pub rate: HarvestRate,
}

struct HarvestBranch {
    pub distance: i32,
    pub source: usize,
    pub target: usize,
    pub content: Content,
}

#[derive(Clone,Copy)]
struct HarvestCounters {
    pub num_crystal_harvests: i32,
    pub num_egg_harvests: i32,
}
impl HarvestCounters {
    pub fn new() -> Self {
        Self { num_crystal_harvests: 0, num_egg_harvests: 0 }
    }

    pub fn add(&self, content: &Content) -> Self {
        match content {
            Content::Crystals => Self {
                num_crystal_harvests: self.num_crystal_harvests + 1,
                num_egg_harvests: self.num_egg_harvests,
            },
            Content::Eggs => Self {
                num_crystal_harvests: self.num_crystal_harvests,
                num_egg_harvests: self.num_egg_harvests + 1,
            },
        }
    }
}