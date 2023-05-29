use core::panic;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Display;

use super::paths::PathMap;
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

struct HarvestMesh {
    unharvested: HashMap<usize,i32>, // unharvested cell -> distance to closest beacon
    priorities: VecDeque<usize>, // cells, highest priority first
    beacons: HashSet<usize>,
}
impl HarvestMesh {
    pub fn generate(plan: &[Milestone], view: &View, state: &State) -> Self {
        let mut unharvested: HashMap<usize,i32> =
            (0..view.layout.cells.len())
            .filter(|i| state.resources[*i] > 0)
            .map(|cell| (cell, i32::MAX))
            .collect();
        let mut priorities = VecDeque::new();
        for milestone in plan.iter() {
            if unharvested.remove(&milestone.cell).is_some() {
                priorities.push_back(milestone.cell);
            }
        }

        Self {
            unharvested,
            priorities,
            beacons: HashSet::new(),
        }
    }

    pub fn beacons(&self) -> impl Iterator<Item=usize> + '_ { self.beacons.iter().cloned() }
    pub fn num_beacons(&self) -> usize { self.beacons.len() }

    pub fn take_next(&mut self) -> Option<usize> {
        if let Some(cell) = self.priorities.pop_front() {
            // Take next priority cell
            return Some(cell);
        }

        if let Some((cell, _)) = self.unharvested.iter().min_by_key(|(cell,distance)| (**distance,**cell)) {
            // Take closest unharvested cell
            let cell = *cell;
            self.unharvested.remove(&cell);
            return Some(cell);
        }

        None
    }

    pub fn add_beacon(&mut self, beacon: usize, paths: &PathMap) {
        if self.beacons.insert(beacon) {
            for (cell, distance) in self.unharvested.iter_mut() {
                let new_distance = paths.distance_between(*cell, beacon);
                if new_distance < *distance {
                    *distance = new_distance;
                }
            }
        }
    }
}

pub fn enact_plan(player: usize, plan: &[Milestone], view: &View, state: &State) -> Vec<Action> {
    let mut actions = Vec::new();

    let total_ants: i32 = state.num_ants[player].iter().cloned().sum();

    let mut targets = Vec::new();
    let mut mesh = HarvestMesh::generate(plan, view, state);
    for &base in view.layout.bases[player].iter() {
        mesh.add_beacon(base, &view.paths);
    }

    while let Some(target) = mesh.take_next() {
        let initial_distance = mesh.num_beacons() as i32;
        let initial_harvests = targets.len() as i32;
        let initial_collection_rate = calculate_collection_rate(total_ants, initial_distance, initial_harvests);

        if let Some((distance, source)) =
            mesh.beacons()
            .map(|source| (view.paths.distance_between(source, target),source))
            .min() {

            let new_collection_rate = calculate_collection_rate(total_ants, initial_distance + distance, initial_harvests + 1);
            // eprintln!("considered harvesting <{}> (distance {}): {} -> {}", target, distance, initial_collection_rate, new_collection_rate);

            if new_collection_rate > initial_collection_rate {
                for cell in view.paths.calculate_path(source, target, &view.layout) {
                    mesh.add_beacon(cell, &view.paths);
                }
                targets.push(target);

            } else {
                // Best harvest not worth it, so none others will be either
                break;
            }

        } else {
            panic!("no sources available for harvest");
        }
    }

    for beacon in mesh.beacons() {
        actions.push(Action::Beacon { index: beacon, strength: 1 });
    }

    actions.push(Action::Message { text: format_harvest_msg(targets.as_slice()) });

    actions
}

fn format_harvest_msg(targets: &[usize]) -> String {
    use std::fmt::Write;

    let mut msg = String::new();
    for &target in targets {
        if !msg.is_empty() {
            msg.push_str(" ");
        }
        write!(&mut msg, "{}", target).ok();
    }
    msg
}

fn calculate_collection_rate(total_ants: i32, total_distance: i32, num_harvests: i32) -> i32 {
    if total_distance <= 0 { return 0 }
    let per_cell = total_ants / total_distance; // intentional integer division since ants can't be split
    num_harvests * per_cell
}