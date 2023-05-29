use std::fmt::Display;

use super::harvesting::HarvestMap;
use super::inputs::*;
use super::movement::{self,AssignmentsPerPlayer};
use super::view::*;

pub enum Event {
    HarvestCompleted { tick: u32, cell: usize },
}
impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::HarvestCompleted { tick, cell } => {
                write!(f, "{}: completed {}", tick, cell)
            },
        }
    }
}

pub fn forward(assignments: &AssignmentsPerPlayer, view: &View, state: &mut State, mut events: Option<&mut Vec<Event>>) {
    state.tick += 1;
    apply_movement(assignments, view, state);
    apply_harvest(view, state, &mut events);
}

fn apply_movement(assignments: &AssignmentsPerPlayer, view: &View, state: &mut State) {
    for player in 0..NUM_PLAYERS {
        let assignments = &assignments[player];
        let num_ants = &mut state.num_ants[player];

        movement::move_ants_for_player(assignments, view, num_ants);
    }
}

fn apply_harvest(view: &View, state: &mut State, events: &mut Option<&mut Vec<Event>>) {
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
            if harvest <= 0 { continue }

            reduction += harvest;

            match content {
                Content::Crystals => {
                    state.crystals[player] += harvest;
                },
                Content::Eggs => {
                    let base = view.layout.bases[player][0];
                    state.num_ants[player][base] += harvest;
                },
            }
        }
        if reduction <= 0 { continue }

        let next = *available - reduction;
        if next < 0 {
            *available = 0;
            if let Some(events) = events {
                events.push(Event::HarvestCompleted { cell, tick: state.tick });
            }

        } else {
            *available = next;
        }
    }
}