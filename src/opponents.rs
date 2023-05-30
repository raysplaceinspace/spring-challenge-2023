use std::collections::HashSet;
use std::collections::VecDeque;

use super::inputs::*;
use super::view::*;
use super::movement::{self,Assignments};

pub struct Countermove {
    pub target: usize,
    pub distance: i32,
}

pub fn enact_countermoves(player: usize, view: &View, state: &State) -> Assignments {
    let countermove = predict_countermove(player, view, state);

    // Add the countermove as an extension of existing ants
    let num_cells = view.layout.cells.len();
    let mut beacons = HashSet::new();

    // Keep ants at existing cells, but only if they are busy - otherwise they will be reassigned
    let busy = identify_busy_ants(player, view, state);
    for cell in 0..num_cells {
        if busy[cell] {
            beacons.insert(cell);
        }
    }

    // Extend to the countermove target
    if let Some(countermove) = countermove {
        // Find closest beacon to extend from
        let source = beacons.iter().cloned().min_by_key(|&beacon| {
            view.paths.distance_between(beacon, countermove.target)
        }).unwrap_or_else(|| view.closest_bases[player][countermove.target]);

        for cell in view.paths.calculate_path(source, countermove.target, &view.layout) {
            beacons.insert(cell);
        }
    }

    let mut assignments = Vec::new();
    assignments.resize(num_cells, 0);
    movement::spread_ants_across_beacons(&mut assignments, player, state);
    assignments.into_boxed_slice()
}

pub fn predict_countermove(player: usize, view: &View, state: &State) -> Option<Countermove> {
    let idle_ants = find_idle_frontier(player, view, state);
    if idle_ants.is_empty() { return None } // All ants are busy - no need to move any ants. No action will just leave the ants where they are.

    let countermove = match find_shortest_countermove(player, &idle_ants, view, state) {
        Some(countermove) => countermove,
        None => return None,
    };

    let total_ants: i32 = state.num_ants[player].iter().cloned().sum();
    if countermove.distance > total_ants {
        // Cannot execute this countermove - it is too far
        return None;
    }

    Some(countermove)
}

fn find_idle_frontier(player: usize, view: &View, state: &State) -> Vec<usize> {
    let num_cells = view.layout.cells.len();
    
    let mut frontier = Vec::new();
    for cell in 0..num_cells {
        if state.num_ants[player][cell] <= 0 { continue } // we don't have any ants here
        if state.resources[cell] > 0 { continue } // these ants are harvesting - we have an explanation of what they are doing

        let base = view.closest_bases[player][cell];
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