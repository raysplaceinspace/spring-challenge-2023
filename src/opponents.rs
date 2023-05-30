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
            if neighbor_distance < my_distance { continue } // neighbor is closer to the base than me - if either one of us are the frontier, it is me

            is_frontier = false;
            break;
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

pub struct Countermove {
    pub source: usize,
    pub target: usize,
    pub distance: i32,
}