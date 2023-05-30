use std::collections::VecDeque;

use super::inputs::*;
use super::view::*;
use super::movement::{self,Assignments};

pub struct Countermove {
    pub source: usize,
    pub target: usize,
    pub distance: i32,
}

struct ExtensionPath {
    pub source: usize,
    pub distance: i32,
}

pub fn enact_countermoves(player: usize, view: &View, state: &State) -> Assignments {
    if let Some(countermove) = predict_countermove(player, view, state) {
        let total_ants: i32 = state.num_ants[player].iter().cloned().sum();
        let num_existing_cells = state.num_ants[player].iter().filter(|c| **c > 0).count() as i32;
        if countermove.distance > total_ants {
            // Cannot execute this countermove - it is too far
            keep_existing_assignments(player, state)

        } else if num_existing_cells + countermove.distance > total_ants {
            // Cannot execute this countermove and keep ants where they are - move existing ants from base to this new target
            let base = view.layout.bases[player][0];
            
            let num_cells = view.layout.cells.len();
            let mut beacons = Vec::new();
            beacons.resize(num_cells, 0);

            for cell in view.paths.calculate_path(base, countermove.target, &view.layout) {
                beacons[cell] = 1;
            }

            movement::spread_ants_across_beacons(&mut beacons, player, state);
            beacons.into_boxed_slice()

        } else {
            // Add the countermove as an extension of existing ants
            let num_cells = view.layout.cells.len();
            let mut beacons = Vec::new();
            beacons.resize(num_cells, 0);

            // Keep ants at existing cells, but only if they are busy - otherwise they will be reassigned
            let busy = identify_busy_ants(player, view, state);
            let mut best: Option<ExtensionPath> = None;
            for cell in 0..num_cells {
                if busy[cell] {
                    beacons[cell] = 1;

                    let distance = view.paths.distance_between(cell, countermove.target);
                    if distance < best.as_ref().map(|b| b.distance).unwrap_or(std::i32::MAX) {
                        best = Some(ExtensionPath { source: cell, distance });
                    }
                }
            }

            // Extend to new cells
            if let Some(best) = best {
                // We have to find a new path to the countermove target (can't use countermove.source)
                // because some of those ants may be idle and we have now removed those
                for cell in view.paths.calculate_path(best.source, countermove.target, &view.layout) {
                    beacons[cell] = 1;
                }
            }

            movement::spread_ants_across_beacons(&mut beacons, player, state);
            beacons.into_boxed_slice()
        }

    } else {
        keep_existing_assignments(player, state)
    }
}

fn keep_existing_assignments(player: usize, state: &State) -> Assignments {
    state.num_ants[player].clone()
}

pub fn predict_countermove(player: usize, view: &View, state: &State) -> Option<Countermove> {
    let idle_ants = find_idle_frontier(player, view, state);
    if idle_ants.is_empty() { return None } // All ants are busy - no need to move any ants. No action will just leave the ants where they are.

    find_shortest_countermove(player, &idle_ants, view, state)
}

fn find_idle_frontier(player: usize, view: &View, state: &State) -> Vec<usize> {
    let num_cells = view.layout.cells.len();
    let base = view.layout.bases[player][0];
    
    let mut frontier = Vec::new();
    for cell in 0..num_cells {
        if state.num_ants[player][cell] <= 0 { continue } // we don't have any ants here
        if state.resources[cell] > 0 { continue } // these ants are harvesting - we have an explanation of what they are doing

        let my_distance = view.paths.distance_between(base, cell);

        let mut is_frontier = true;
        for &neighbor in view.layout.cells[cell].neighbors.iter() {
            if state.num_ants[player][neighbor] <= 0 { continue } // neighbor is empty

            let neighbor_distance = view.paths.distance_between(base, neighbor);
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

fn find_shortest_countermove(player: usize, sources: &[usize], view: &View, state: &State) -> Option<Countermove> {
    // Where at the idle ants going? Find cells unoccupied unharvested cells closest to the idle ants.
    let num_cells = view.layout.cells.len();
    let target =
        (0..num_cells)
        .filter(|target| state.resources[*target] > 0 && state.num_ants[player][*target] == 0)
        .filter_map(|target| {
            let countermove = sources.iter().map(|source| {
                let distance = view.paths.distance_between(*source, target);
                Countermove {
                    source: *source,
                    target,
                    distance,
                }
            }).min_by_key(|c| c.distance);
            countermove
        })
        .min_by_key(|c| c.distance);
    target
}

fn identify_busy_ants(player: usize, view: &View, state: &State) -> Box<[bool]> {
    let num_cells = view.layout.cells.len();

    let flow_distance_from_base = calculate_flow_distance_from_base(player, view, state);

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