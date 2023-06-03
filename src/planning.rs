use super::fnv::FnvHashSet;
use std::fmt::Display;

use super::inputs::NUM_PLAYERS;
use super::harvesting;
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
    let enemy = (player + 1) % NUM_PLAYERS;
    let attack = harvesting::calculate_max_flow_for_player(enemy, view, &state.num_ants);
    let evaluator = HarvestEvaluator::new(player, state);
    let mut counts = NumHarvests::new();

    let mut targets = Vec::new();
    let mut unused_bases: FnvHashSet<_> = view.layout.bases[player].iter().copied().collect();
    let mut beacons = FnvHashSet::default();

    let nearby = NearbyPathMap::near_my_ants(player, view, state);
    for milestone in plan.iter() {
        let target = milestone.cell;
        if state.resources[target] <= 0 { continue } // Nothing to harvest here

        let (distance, source) = beacons.iter().chain(unused_bases.iter()).map(|&beacon| {
            let distance = view.paths.distance_between(beacon, target);
            (distance, beacon)
        }).min().expect("no beacons");

        let content = view.layout.cells[target].content;
        let new_counts = counts.clone().add(content);

        let initial_spread = beacons.len() as i32;
        let initial_collection_rate = evaluator.calculate_harvest_rate(&counts, initial_spread);

        let new_spread = initial_spread + distance;
        let new_collection_rate = evaluator.calculate_harvest_rate(&new_counts, new_spread);
        if new_collection_rate > initial_collection_rate {
            let ants_per_cell = state.total_ants[player] / new_spread;
            for cell in nearby.calculate_path(source, target, &view.layout, &view.paths) {
                if attack[cell] > ants_per_cell { break } // Stop if we cannot gain anything from harvesting this cell

                beacons.insert(cell);
                unused_bases.remove(&cell);
            }
            targets.push(target);

            counts = new_counts;

        } else {
            // Best harvest not worth it, so none others will be either
            break;
        }
    }

    Commands {
        assignments: movement::spread_ants_across_beacons(beacons.into_iter(), player, view, state),
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
