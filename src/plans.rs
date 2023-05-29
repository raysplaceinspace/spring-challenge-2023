use core::panic;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;

use super::paths::PathMap;
use super::inputs::*;
use super::view::*;

#[derive(Clone)]
pub struct Milestone {
    pub cells: Vec<usize>,
    pub num_cells_to_leave: i8,
}
impl Milestone {
    pub fn new(cells: Vec<usize>, num_cells_to_leave: i8) -> Self {
        Self { cells, num_cells_to_leave }
    }

    pub fn is_complete(&self, state: &State) -> bool {
        let num_cells_remaining = self.cells.iter().filter(|cell| state.resources[**cell] > 0).count();
        self.num_cells_to_leave >= num_cells_remaining as i8
    }

    pub fn is_smaller(me: &[Milestone], other: &[Milestone]) -> bool {
        if me.len() > other.len() { return false }
        if me.len() < other.len() { return true }

        let my_cells: usize = me.iter().map(|milestone| milestone.cells.len()).sum();
        let other_cells: usize = other.iter().map(|milestone| milestone.cells.len()).sum();
        my_cells < other_cells
    }
}
impl Display for Milestone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let mut is_first = true;
        for cell in self.cells.iter() {
            if is_first {
                is_first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{}", cell)?;
        }
        write!(f, "/{})", self.num_cells_to_leave)?;
        Ok(())
    }
}

struct HarvestMesh {
    unharvested: HashMap<usize,i32>, // unharvested cell -> distance to closest beacon
    beacons: HashSet<usize>,
}
impl HarvestMesh {
    pub fn new(cells: impl Iterator<Item=usize>, state: &State) -> Self {
        let unharvested: HashMap<usize,i32> =
            cells
            .filter(|i| state.resources[*i] > 0)
            .map(|cell| (cell, i32::MAX))
            .collect();

        Self {
            unharvested,
            beacons: HashSet::new(),
        }
    }

    pub fn generate(plan: &[Milestone], view: &View, state: &State) -> Self {
        if let Some(milestone) = plan.iter().find(|milestone| !milestone.is_complete(state)) {
            // Consider only the cells in the first incomplete milestone
            Self::new(milestone.cells.iter().cloned(), state)

        } else {
            // Consider all cells
            Self::new(0..view.layout.cells.len(), state)
        }
    }

    pub fn beacons(&self) -> impl Iterator<Item=usize> + '_ { self.beacons.iter().cloned() }
    pub fn num_beacons(&self) -> usize { self.beacons.len() }

    pub fn take_next(&mut self) -> Option<usize> {
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