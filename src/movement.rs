use std::collections::HashSet;

use super::inputs::*;
use super::view::*;

pub type Assignments = Box<[i32]>;
pub type AssignmentsPerPlayer = [Assignments; NUM_PLAYERS];

struct Movement {
    pub source: usize,
    pub sink: usize,
    pub assigned: i32,
}

struct Candidate {
    pub source: usize,
    pub sink: usize,
    pub distance: i32,
}

pub fn spread_ants_across_beacons<'a>(beacons: &mut [i32], player: usize, state: &State) {
    let total_beacons: i32 = beacons.iter().cloned().sum();
    let total_ants: i32 = state.num_ants[player].iter().sum();
    if total_beacons > 0 {
        for beacon in beacons.iter_mut() {
            *beacon = *beacon * total_ants / total_beacons;
        }

    }
}

pub fn assignments_to_actions(assignments: &[i32]) -> Vec<Action> {
    let mut actions = Vec::new();
    for (cell, &num_ants) in assignments.iter().enumerate() {
        if num_ants > 0 {
            actions.push(Action::Beacon {
                index: cell,
                strength: num_ants,
            });
        }
    }
    actions
}

pub fn move_ants_for_player(assignments: &Assignments, view: &View, num_ants: &mut AntsPerCell) {
    let num_cells = view.layout.cells.len();

    // Calculate sources and sinks
    let mut excess: Vec<i32> = Vec::with_capacity(num_cells);
    let mut sources: HashSet<usize> = HashSet::new();
    let mut sinks: HashSet<usize> = HashSet::new();
    for cell in 0..num_cells {
        let cell_excess = num_ants[cell] - assignments[cell];
        if cell_excess > 0 {
            sources.insert(cell);
        } else if cell_excess < 0 {
            sinks.insert(cell);
        }
        excess.push(cell_excess);
    }

    // Assign which ants should move where
    let mut movements = Vec::new();
    while !sources.is_empty() && !sinks.is_empty() {
        if let Some(closest) =
            sinks.iter().filter_map(|sink| {
                let closest = sources.iter().map(|source| {
                    Candidate {
                        source: *source,
                        sink: *sink,
                        distance: view.paths.distance_between(*source, *sink),
                    }
                }).min_by_key(|candidate| candidate.distance)?;
                Some(closest)
            }).min_by_key(|candidate| candidate.distance) {

            let available = excess[closest.source];
            let required = -excess[closest.sink];
            let assigned = available.min(required);
            if assigned <= 0 { panic!("Nothing to assign from source to sink") }

            movements.push(Movement {
                source: closest.source,
                sink: closest.sink,
                assigned,
            });

            excess[closest.source] -= assigned;
            if excess[closest.source] <= 0 {
                sources.remove(&closest.source);
            }

            excess[closest.sink] += assigned;
            if excess[closest.sink] >= 0 {
                sinks.remove(&closest.sink);
            }

        } else {
            panic!("Unable to find a movement candidate")
        }
    }

    // Perform movement
    for movement in movements {
        if let Some(next) = view.paths.step_towards(movement.source, movement.sink, &view.layout) {
            let source_ants = &mut num_ants[movement.source];
            if movement.assigned > *source_ants { panic!("Not enough ants to move") }
            *source_ants -= movement.assigned;

            num_ants[next] += movement.assigned;

        } else {
            panic!("Unable to find a path from source to sink")
        }
    }
}