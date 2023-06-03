use std::collections::BinaryHeap;

use super::fnv::FnvHashSet;
use std::fmt::Display;

use super::movement;
use super::valuation::{NumHarvests,HarvestEvaluator};
use super::pathing::NearbyPathMap;
use super::view::*;

#[derive(Clone)]
pub struct Milestone {
    pub cell: usize,
}
impl Milestone {
    pub fn new(cell: usize) -> Self {
        Self { cell }
    }

    pub fn is_complete(&self, state: &State) -> bool {
        state.resources[self.cell] <= 0
    }
}
impl Display for Milestone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cell.fmt(f)
    }
}

pub fn enact_plan(player: usize, plan: &[Milestone], view: &View, state: &State) -> Commands {
    let evaluator = HarvestEvaluator::new(player, state);
    let mut counts = NumHarvests::new();

    let mut targets = Vec::new();
    let mut beacons = FnvHashSet::default();
    for &base in view.layout.bases[player].iter() {
        beacons.insert(base);
    }

    let nearby = NearbyPathMap::near_my_ants(player, view, state);
    for milestone in plan.iter() {
        let target = milestone.cell;
        if state.resources[target] <= 0 { continue } // Nothing to harvest here

        let closest_beacon = beacons.iter().min_by_key(|&&beacon| {
            view.paths.distance_between(beacon, target)
        }).cloned().expect("no beacons");

        let path = find_closest_harvest_chain(closest_beacon, target, view, &nearby);

        let content = view.layout.cells[target].content;
        let new_counts = counts.clone().add(content);

        let initial_spread = beacons.len() as i32;
        let initial_collection_rate = evaluator.calculate_harvest_rate(&counts, initial_spread);

        let new_spread = initial_spread + path.len().saturating_sub(1) as i32; // -1 because the first cell is already a beacon
        let new_collection_rate = evaluator.calculate_harvest_rate(&new_counts, new_spread);
        if new_collection_rate > initial_collection_rate {
            for cell in path {
                beacons.insert(cell);
            }
            targets.push(target);

            counts = new_counts;

        } else {
            // Best harvest not worth it, so none others will be either
            break;
        }
    }

    Commands {
        assignments: movement::spread_ants_across_beacons(beacons.into_iter(), player, state),
        targets,
    }
}

pub struct Commands {
    pub assignments: Box<[i32]>,
    pub targets: Vec<usize>,
}
impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.targets.is_empty() {
            write!(f, "-")?;
        } else {
            let mut is_first = true;
            for &target in self.targets.iter() {
                if is_first {
                    is_first = false;
                } else {
                    write!(f, " ")?;
                }
                write!(f, "{}", target)?;
            }
        }
        Ok(())
    }
}

fn find_closest_harvest_chain(source: usize, target: usize, view: &View, nearby: &NearbyPathMap) -> Vec<usize> {
    if source == target { return vec![source] }

    let num_cells = view.layout.cells.len();

    let mut cost_map = Vec::new();
    cost_map.resize(num_cells, i32::MAX);

    let mut queue = BinaryHeap::new();
    queue.push((0, target));
    cost_map[target] = 0;

    while let Some((priority, cell)) = queue.pop() {
        if cell == source { break }

        let cost = -priority;
        for &n in view.layout.cells[cell].neighbors.iter() {
            let length_cost = 1; // prefer shorter paths so we don't spread our ants too thinly
            let step_cost = (nearby.distance_to(n) - 1).max(0); // prefer paths closer to our ants. -1 because our ants can move 1 cell per tick for free.
            let new_cost = cost + length_cost + step_cost;
            if new_cost < cost_map[n] {
                cost_map[n] = new_cost;
                queue.push((-new_cost, n));
            }
        }
    }

    let mut path = vec![source];
    loop {
        let end = path.last().expect("unexpected empty path").clone();
        if end == target { break }

        let next =
            view.layout.cells[end].neighbors.iter()
            .min_by_key(|&&n| cost_map[n])
            .expect("no neighbor found in cost map")
            .clone();
        path.push(next);
    }

    path
}