use std::collections::HashSet;
use std::fmt::Display;

use super::inputs::*;
use super::view::*;

#[derive(Clone)]
pub struct Plan {
    pub priorities: Vec<usize>, // Cell indices
}
impl Plan {
    pub fn new() -> Self {
        Self {
            priorities: Vec::new(),
        }
    }
}

pub fn enact_plan(player: usize, plan: &Plan, view: &View, state: &State) -> Vec<Action> {
    let mut actions = Vec::new();

    let my_base = view.layout.bases[player][0];
    let total_ants: i32 = state.num_ants[player].iter().cloned().sum();

    let mut sources = HashSet::new();
    sources.insert(my_base);

    let mut num_harvests = 0;
    let mut total_distance = 0;
    let mut branches = Vec::new();

    let sequence = calculate_harvest_sequence(player, plan, view, state);
    for target in sequence {
        let initial_collection_rate = calculate_collection_rate(total_ants, total_distance, num_harvests);

        if let Some((distance, source)) =
            sources.iter()
            .map(|&source| (view.paths.distance_between(source, target),source))
            .min() {

            let new_collection_rate = calculate_collection_rate(total_ants, total_distance + distance, num_harvests + 1);
            eprintln!("considered harvesting <{}> (distance {}): {} -> {}", target, distance, initial_collection_rate, new_collection_rate);

            if new_collection_rate >= initial_collection_rate {
                sources.insert(target);

                total_distance += distance;
                num_harvests += 1;

                branches.push(HarvestBranch {
                    distance,
                    source,
                    target,
                });

            } else {
                // Best harvest not worth it, so none others will be either
                break;
            }

        } else {
            // No sources available - can't harvest
            break;
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

fn calculate_harvest_sequence(player: usize, plan: &Plan, view: &View, state: &State) -> Vec<usize> {
    let prioritized: HashSet<usize> = plan.priorities.iter().cloned().collect();

    let mut sequence: Vec<usize> = (0..view.layout.cells.len()).filter(|i| state.resources[*i] > 0 && !prioritized.contains(i)).collect();
    let base = view.layout.bases[player][0];
    sequence.sort_by_key(|&cell| view.paths.distance_between(base, cell));

    sequence.splice(0..0, plan.priorities.iter().cloned());

    sequence
}

fn calculate_collection_rate(total_ants: i32, total_distance: i32, num_harvests: i32) -> i32 {
    if total_distance <= 0 { return 0 }
    let per_cell = total_ants / total_distance; // intentional integer division since ants can't be split
    num_harvests * per_cell
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
}