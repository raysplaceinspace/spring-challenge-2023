use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Display;

use super::inputs::*;
use super::view::*;
use super::movement::{self,Assignments};

pub struct Countermoves {
    pub assignments: Assignments,
    pub target: Option<usize>,
}
impl Display for Countermoves {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(target) = self.target {
            target.fmt(f)
        } else {
            write!(f, "-")
        }
    }
}

pub fn enact_countermoves(player: usize, view: &View, state: &State) -> Countermoves {
    // Add the countermove as an extension of existing ants
    let num_cells = view.layout.cells.len();
    let total_ants: i32 = state.num_ants[player].iter().cloned().sum();
    let harvestable: Vec<usize> = (0..num_cells).filter(|&cell| state.resources[cell] > 0).collect();
    let mut beacons = HashSet::new();
    
    // Always begin the calculation with ants on the bases so we have somewhere to extend from
    for &base in view.layout.bases[player].iter() {
        beacons.insert(base);
    }

    // Keep ants at existing cells, but only if they are busy - otherwise they will be reassigned
    let flow_distance_from_base = calculate_flow_distance_from_base(player, view, state);
    let busy = identify_busy_ants(view, state, &flow_distance_from_base);
    for cell in 0..num_cells {
        if busy[cell] {
            beacons.insert(cell);
        }
    }

    // Extend to the countermove target
    let idle_ants = find_idle_frontier(player, view, state, &flow_distance_from_base, &busy);
    let mut countermove = None;
    if let Some(target) = find_shortest_countermove(player, &idle_ants, &harvestable, view, state) {
        // Find closest beacon to extend from
        let source = beacons.iter().cloned().min_by_key(|&beacon| {
            view.paths.distance_between(beacon, target)
        }).unwrap_or_else(|| view.closest_bases[player][target]);

        if beacons.len() as i32 + view.paths.distance_between(source, target) <= total_ants { // If we can reach this target
            for cell in view.paths.calculate_path(source, target, &view.layout) {
                beacons.insert(cell);
            }

            countermove = Some(target);
        }
    }

    Countermoves {
        assignments: movement::spread_ants_across_beacons(beacons.into_iter(), player, state),
        target: countermove,
    }
}

fn find_idle_frontier(player: usize, view: &View, state: &State, flow_distance_from_base: &[i32], busy: &[bool]) -> Vec<usize> {
    let num_cells = view.layout.cells.len();
    
    let mut frontier = Vec::new();
    for cell in 0..num_cells {
        if state.num_ants[player][cell] <= 0 { continue } // we don't have any ants here
        if busy[cell] { continue } // these ants are part of a harvesting chain - we have an explanation of what they are doing and don't need to find one

        let my_distance = flow_distance_from_base[cell];
        if my_distance == i32::MAX { continue } // disconnected from base - ignore these ants

        let mut is_frontier = true;
        for &neighbor in view.layout.cells[cell].neighbors.iter() {
            if state.num_ants[player][neighbor] <= 0 { continue } // neighbor is empty

            let neighbor_distance = flow_distance_from_base[cell];
            if neighbor_distance == i32::MAX { continue } // disconnected from base - ignore these ants

            if neighbor_distance > my_distance {
                // neighbor is further from the base than me - they are the frontier, if either of us are
                is_frontier = false;
                break;
            }
        }

        if is_frontier {
            frontier.push(cell);
        }
    }

    frontier
}

struct ExtensionPath {
    pub target: usize,
    pub distance: i32,
}
fn find_shortest_countermove(player: usize, sources: &[usize], targets: &[usize], view: &View, state: &State) -> Option<usize> {
    if sources.is_empty() || targets.is_empty() { return None }

    // Where are the idle ants going? Find cells unoccupied unharvested cells closest to the idle ants.
    let countermove =
        targets.iter()
        .filter(|&&target| state.num_ants[player][target] == 0)
        .filter_map(|&target| {
            sources.iter().map(|source| {
                ExtensionPath {
                    target,
                    distance: view.paths.distance_between(*source, target),
                }
            }).min_by_key(|c| c.distance)
        })
        .min_by_key(|c| c.distance);
    match countermove {
        Some(countermove) => Some(countermove.target),
        None => None,
    }
}

fn identify_busy_ants(view: &View, state: &State, flow_distance_from_base: &[i32]) -> Box<[bool]> {
    let num_cells = view.layout.cells.len();

    let mut busy = Vec::new();
    busy.resize(num_cells, false);

    for cell in 0..num_cells {
        if state.resources[cell] <= 0 { continue } // nothing to harvest here
        if flow_distance_from_base[cell] == i32::MAX { continue } // not harvesting this cell

        mark_return_path_as_busy(cell, &flow_distance_from_base, &view.layout, &mut busy);
    }

    busy.into_boxed_slice()
}

fn calculate_flow_distance_from_base(player: usize, view: &View, state: &State) -> Box<[i32]> {
    let num_cells = view.layout.cells.len();
    let mut flow_distance_from_base: Vec<i32> = Vec::new();
    flow_distance_from_base.resize(num_cells, i32::MAX);

    let mut queue = VecDeque::new();
    for &base in view.layout.bases[player].iter() {
        if state.num_ants[player][base] <= 0 { continue }
        flow_distance_from_base[base] = 0;
        queue.push_back(base);
    }

    while let Some(source) = queue.pop_front() {
        let source_distance = flow_distance_from_base[source];

        let neighbor_distance = source_distance + 1;
        for &neighbor in view.layout.cells[source].neighbors.iter() {
            if state.num_ants[player][neighbor] <= 0 { continue }
            if flow_distance_from_base[neighbor] <= neighbor_distance { continue }

            flow_distance_from_base[neighbor] = neighbor_distance;
            queue.push_back(neighbor);
        }
    }

    flow_distance_from_base.into_boxed_slice()
}

fn mark_return_path_as_busy(cell: usize, flow_distance_from_base: &[i32], layout: &Layout, busy: &mut [bool]) {
    busy[cell] = true;

    let remaining_distance = flow_distance_from_base[cell];
    if remaining_distance <= 0 { return }
    else if remaining_distance == i32::MAX { return } // there won't be a path back to base from here

    // Flow back to base along the neighbors that are closer to the base
    for &neighbor in layout.cells[cell].neighbors.iter() {
        if !busy[neighbor] && flow_distance_from_base[neighbor] < remaining_distance {
            mark_return_path_as_busy(neighbor, flow_distance_from_base, layout, busy);
        }
    }
}