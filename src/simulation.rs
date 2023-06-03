use super::harvesting::HarvestMap;
use super::inputs::*;
use super::movement::{self,AssignmentsPerPlayer};
use super::view::*;

pub fn forward(assignments: &AssignmentsPerPlayer, view: &View, state: &mut State) {
    state.tick += 1;
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
    let harvest_map = HarvestMap::generate(view, &state.num_ants);
    for cell in 0..view.layout.cells.len() {
        let available = &mut state.resources[cell];
        if *available <= 0 { continue }

        let content = match view.layout.cells[cell].content {
            Some(content) => content,
            None => continue,
        };

        let mut reduction = 0;
        for player in 0..NUM_PLAYERS {
            let harvest = harvest_map.calculate_harvest_at(player, cell, *available);
            if harvest <= 0 { continue }

            reduction += harvest;

            match content {
                Content::Crystals => {
                    state.crystals[player] += harvest;
                },
                Content::Eggs => {
                    let num_bases = view.layout.bases[player].len();
                    let mut remaining = harvest;
                    for (index, &base) in view.layout.bases[player].iter().enumerate() {
                        let num_bases_remaining = num_bases - index;
                        let spawn_at_this_base = remaining / num_bases_remaining as i32;
                        state.num_ants[player][base] += spawn_at_this_base;
                        state.total_ants[player] += spawn_at_this_base;
                        remaining -= spawn_at_this_base;
                    }
                },
            }
        }
        if reduction <= 0 { continue }

        let next = *available - reduction;
        if next < 0 {
            *available = 0;

        } else {
            *available = next;
        }
    }
}