use super::harvesting::HarvestMap;
use super::inputs::*;
use super::movement::{self,Assignments,AssignmentsPerPlayer};
use super::view::{self,*};

pub fn forward(assignments: &AssignmentsPerPlayer, view: &View, state: &mut State) {
    apply_movement(assignments, view, state);
    apply_harvest(view, state);
}

fn apply_movement(assignments: &AssignmentsPerPlayer, view: &View, state: &mut State) {
    for player in 0..NUM_PLAYERS {
        let assignments = &assignments[player];
        let num_ants = &mut state.num_ants[player];
        movement::move_ants_for_player(assignments, view, num_ants);
    }
}

fn apply_harvest(view: &View, state: &mut State) {
    let harvesting = [
        HarvestMap::generate(ME, view, &state.num_ants),
        HarvestMap::generate(ENEMY, view, &state.num_ants),
    ];
    for cell in 0..view.layout.cells.len() {
        let available = &mut state.resources[cell];
        if *available <= 0 { continue }

        let content = match view.layout.cells[cell].content {
            Some(content) => content,
            None => continue,
        };

        let mut reduction = 0;
        for (player,harvest_map) in harvesting.iter().enumerate() {
            let harvest = harvest_map.calculate_harvest_at(cell, *available);
            reduction += harvest;

            match content {
                Content::Crystals => state.crystals[player] += harvest,
                Content::Eggs => {
                    let base = view.layout.bases[player][0];
                    state.num_ants[player][base] += harvest;
                },
            }
        }

        *available = (*available - reduction).max(0);
    }
}