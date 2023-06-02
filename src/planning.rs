use core::panic;

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
    let evaluator = HarvestEvaluator::new(player, view, state);
    let mut counts = NumHarvests::new();

    let mut targets = Vec::new();
    let mut beacons = FnvHashSet::default();
    for &base in view.layout.bases[player].iter() {
        beacons.insert(base);
    }

    let nearby = NearbyPathMap::near_my_ants(player, view, state);
    for milestone in plan.iter() {
        let target = milestone.cell;
        if !evaluator.is_worth_harvesting(target, player, view, state) { continue }

        if let Some((distance, source)) = beacons.iter().map(|&source| (view.paths.distance_between(source, target),source)).min() {
            let content = view.layout.cells[target].content;
            let new_counts = counts.clone().add(content);

            let initial_distance = beacons.len() as i32;
            let new_collection_rate = evaluator.calculate_harvest_rate(&new_counts, initial_distance + distance);
            // eprintln!("considered harvesting <{}> (distance {}): {} -> {}", target, distance, initial_collection_rate, new_collection_rate);

            let initial_collection_rate = evaluator.calculate_harvest_rate(&counts, initial_distance);
            if new_collection_rate > initial_collection_rate {
                for cell in nearby.calculate_path(source, target, &view.layout, &view.paths) {
                    beacons.insert(cell);
                }
                targets.push(target);

                counts = new_counts;

            } else {
                // Best harvest not worth it, so none others will be either
                break;
            }

        } else {
            panic!("no sources available for harvest");
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