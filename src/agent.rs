use std::collections::HashSet;

use super::paths::PathMap;
use super::model::*;

pub struct Agent {
    layout: Layout,
    paths: PathMap,
}
impl Agent {
    pub fn new(layout: Layout) -> Self {
        Self {
            paths: PathMap::generate(&layout),
            layout,
        }
    }

    pub fn layout(&self) -> &Layout { &self.layout }

    pub fn act(&mut self, states: &Vec<CellState>) -> Vec<Action> {
        let mut actions = Vec::new();

        let my_base = self.layout.my_bases[0];
        let total_ants: i32 = states.iter().map(|state| state.num_my_ants).sum();

        let mut sources = HashSet::new();
        sources.insert(my_base);

        let mut num_harvests = 0;
        let mut total_distance = 0;
        let mut branches = Vec::new();

        let mut unharvested: HashSet<usize> = (0..self.layout.cells.len()).filter(|i| states[*i].resources > 0).collect();
        while !unharvested.is_empty() {
            let initial_collection_rate = calculate_collection_rate(total_ants, total_distance, num_harvests);

            if let Some(closest) =
                unharvested.iter()
                .filter_map(|&target| {
                    if let Some((distance, source)) = sources.iter().map(|&source| (self.paths.distance_between(source, target),source)).min() {
                        Some(HarvestBranch { distance, source, target })
                    } else {
                        None
                    }
                })
                .min_by_key(|branch| (branch.distance, branch.target)) {

                let new_collection_rate = calculate_collection_rate(total_ants, total_distance + closest.distance, num_harvests + 1);
                eprintln!("considered harvesting <{}> (distance {}): {:.1} -> {:.1}", closest.target, closest.distance, initial_collection_rate, new_collection_rate);
                if new_collection_rate >= initial_collection_rate {
                    unharvested.remove(&closest.target);
                    sources.insert(closest.target);

                    num_harvests += 1;
                    total_distance += closest.distance;

                    branches.push(closest);

                } else {
                    // Closest harvest not worth it, so none others will be either
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
}

fn calculate_collection_rate(total_ants: i32, total_distance: i32, num_harvests: i32) -> i32 {
    if total_distance <= 0 { return 0 }
    num_harvests * (total_ants / total_distance) // intentional integer division since ants can't be split
}

struct HarvestBranch {
    distance: i32,
    source: usize,
    target: usize,
}