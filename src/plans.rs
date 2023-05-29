use core::panic;
use std::collections::HashSet;
use std::fmt::Display;

use super::inputs::*;
use super::view::*;

#[derive(Clone)]
pub struct Milestone {
    pub cell: usize,
}
impl Display for Milestone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cell.fmt(f)
    }
}

pub fn enact_plan(player: usize, plan: &[Milestone], view: &View, state: &State) -> Vec<Action> {
    let mut actions = Vec::new();

    let my_base = view.layout.bases[player][0];
    let total_ants: i32 = state.num_ants[player].iter().cloned().sum();

    let mut beacons = HashSet::new();
    beacons.insert(my_base);

    let mut num_harvests = 0;

    let sequence = calculate_harvest_sequence(player, plan, view, state);
    for target in sequence {
        let initial_distance = beacons.len() as i32;
        let initial_collection_rate = calculate_collection_rate(total_ants, initial_distance, num_harvests);

        if let Some((distance, source)) =
            beacons.iter()
            .map(|&source| (view.paths.distance_between(source, target),source))
            .min() {

            let new_collection_rate = calculate_collection_rate(total_ants, initial_distance + distance, num_harvests + 1);
            // eprintln!("considered harvesting <{}> (distance {}): {} -> {}", target, distance, initial_collection_rate, new_collection_rate);

            if new_collection_rate > initial_collection_rate {
                for cell in view.paths.calculate_path(source, target, &view.layout) {
                    beacons.insert(cell);
                }
                num_harvests += 1;

            } else {
                // Best harvest not worth it, so none others will be either
                break;
            }

        } else {
            panic!("no sources available for harvest");
        }
    }

    for beacon in beacons {
        actions.push(Action::Beacon { index: beacon, strength: 1 });
    }

    actions
}

fn calculate_harvest_sequence(player: usize, plan: &[Milestone], view: &View, state: &State) -> Vec<usize> {
    let prioritized: HashSet<usize> = plan.iter().map(|s| s.cell).collect();

    let mut sequence: Vec<usize> = (0..view.layout.cells.len()).filter(|i| state.resources[*i] > 0 && !prioritized.contains(i)).collect();
    let base = view.layout.bases[player][0];
    sequence.sort_by_key(|&cell| view.paths.distance_between(base, cell));

    sequence.splice(0..0, plan.iter().filter_map(|s| {
        if state.resources[s.cell] > 0 {
            Some(s.cell)
        } else {
            None
        }
    }));

    sequence
}

fn calculate_collection_rate(total_ants: i32, total_distance: i32, num_harvests: i32) -> i32 {
    if total_distance <= 0 { return 0 }
    let per_cell = total_ants / total_distance; // intentional integer division since ants can't be split
    num_harvests * per_cell
}