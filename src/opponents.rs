use std::collections::VecDeque;

use super::view::*;
use super::inputs::*;

pub fn enact_countermoves(player: usize, view: &View, state: &State) -> Vec<Action> {
    if let Some(countermove) = predict_countermove(player, view, state) {
        let total_ants: i32 = state.num_ants[player].iter().cloned().sum();
        let num_existing_cells = state.num_ants[player].iter().filter(|c| **c > 0).count() as i32;
        if countermove.distance > total_ants {
            // Cannot execute this countermove - it is too far
            Vec::new()

        } else if num_existing_cells + countermove.distance > total_ants {
            // Cannot execute this countermove and keep ants where they are - move existing ants from base to this new target
            let base = view.layout.bases[player][0];
            vec![
                Action::Line { source: base, target: countermove.target, strength: 1 },
            ]

        } else {
            // Add the countermove as an extension of existing ants
            let mut actions = Vec::new();

            let num_cells = view.layout.cells.len();

            // Keep ants at existing cells
            for cell in 0..num_cells {
                if state.num_ants[player][cell] > 0 {
                    actions.push(Action::Beacon { index: cell, strength: 1 });
                }
            }

            // Extend to new cells
            actions.push(Action::Line { source: countermove.source, target: countermove.target, strength: 1 });

            actions
        }

    } else {
        Vec::new()
    }
}
pub fn predict_countermove(player: usize, view: &View, state: &State) -> Option<Countermove> {
    let flow_distance_from_base = calculate_flow_distance_from_base(player, &state.num_ants[player], &view.layout);
    let busy_map = identify_busy_ants(&flow_distance_from_base, state, &view.layout);
    let idle_ants = identify_idle_ants(&state.num_ants[player], &busy_map);
    if idle_ants.is_empty() { return None } // All ants are busy - no need to move any ants. No action will just leave the ants where they are.

    find_shortest_countermove(player, &idle_ants, view, state)
}

fn calculate_flow_distance_from_base(player: usize, num_ants_per_cell: &AntsPerCell, layout: &Layout) -> Box<[i32]> {
    let num_cells = num_ants_per_cell.len();
    let mut flow_distance_from_base: Vec<i32> = Vec::new();
    flow_distance_from_base.resize(num_cells, i32::MAX);

    let mut queue = VecDeque::new();
    for &base in layout.bases[player].iter() {
        if num_ants_per_cell[base] <= 0 { continue }
        flow_distance_from_base[base] = 0;
        queue.push_back(base);
    }

    while let Some(source) = queue.pop_front() {
        let source_distance = flow_distance_from_base[source];

        let neighbor_distance = source_distance + 1;
        for &neighbor in layout.cells[source].neighbors.iter() {
            if num_ants_per_cell[neighbor] <= 0 { continue }
            if flow_distance_from_base[neighbor] <= neighbor_distance { continue }

            flow_distance_from_base[neighbor] = neighbor_distance;
            queue.push_back(neighbor);
        }
    }

    flow_distance_from_base.into_boxed_slice()
}

fn identify_busy_ants(flow_distance_from_base: &[i32], state: &State, layout: &Layout) -> Box<[bool]> {
    let num_cells = flow_distance_from_base.len();
    let mut busy = Vec::new();
    busy.resize(num_cells, false);

    for cell in 0..num_cells {
        if state.resources[cell] <= 0 { continue } // nothing to harvest here
        if flow_distance_from_base[cell] == i32::MAX { continue } // not harvesting this cell

        mark_return_path_as_busy(cell, &flow_distance_from_base, layout, &mut busy);
    }

    busy.into_boxed_slice()
}

fn mark_return_path_as_busy(cell: usize, flow_distance_from_base: &[i32], layout: &Layout, busy: &mut [bool]) {
    busy[cell] = true;

    let remaining_distance = flow_distance_from_base[cell];
    if remaining_distance <= 0 { return }
    else if remaining_distance == i32::MAX { return } // there won't be a path back to base from here

    // Flow back to base along the neighbors that are closer to the base
    for &neighbour in layout.cells[cell].neighbors.iter() {
        if flow_distance_from_base[neighbour] < remaining_distance {
            mark_return_path_as_busy(neighbour, flow_distance_from_base, layout, busy);
        }
    }
}

fn identify_idle_ants(num_ants_per_cell: &AntsPerCell, busy_map: &[bool]) -> Vec<usize> {
    let mut idle_ants = Vec::new();
    let num_cells = num_ants_per_cell.len();
    for cell in 0..num_cells {
        if num_ants_per_cell[cell] > 0 && !busy_map[cell] {
            idle_ants.push(cell);
        }
    }
    idle_ants
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

pub struct Countermove {
    pub source: usize,
    pub target: usize,
    pub distance: i32,
}