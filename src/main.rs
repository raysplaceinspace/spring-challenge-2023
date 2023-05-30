mod agent;
mod evaluation;
mod interface;
mod harvesting;
mod inputs;
mod movement;
mod opponents;
mod pathing;
mod planning;
mod simulation;
mod solving;
mod view;

use agent::Agent;
use harvesting::HarvestMap;
use inputs::*;
use view::*;

use interface::TurnInput;

fn main() {
    let layout = interface::read_initial();
    let view = View::new(layout);

    let mut agent = Agent::new(&view.layout);
    let mut previous_state: Option<State> = None;
    let mut tick = 0;
    loop {
        // Read input
        let TurnInput { num_ants_per_cell, resources_per_cell } = interface::read_turn(&view.layout);

        // Calculate new state
        let state = match previous_state {
            None => State::new(tick, num_ants_per_cell, resources_per_cell, CrystalsPerPlayer::default()),
            Some(previous) => {
                let available_resources = &previous.resources; // Look at previous tick to determine available resources
                let mut harvested = previous.crystals.clone();
                harvest(&view, &num_ants_per_cell, available_resources, &mut harvested);
                State::new(tick, num_ants_per_cell, resources_per_cell, harvested)
            },
        };
        eprintln!("Harvested: me={}, enemy={}", state.crystals[0], state.crystals[1]);

        // Calculate actions
        let actions = agent.act(&view, &state);

        // Emit actions
        if actions.is_empty() {
            println!("{}", interface::format_action(&Action::Wait));
        } else {
            for (i,action) in actions.into_iter().enumerate() {
                if i > 0 {
                    print!(";");
                }
                print!("{}", interface::format_action(&action));
            }
            println!("");
        }

        tick += 1;
        previous_state = Some(state);
    }
}

fn harvest(view: &View, num_ants: &AntsPerCellPerPlayer, available_resources: &ResourcesPerCell, harvested: &mut CrystalsPerPlayer) {
    for player in 0..NUM_PLAYERS {
        let harvest_map = HarvestMap::generate(player, view, num_ants);

        for cell in 0..view.layout.cells.len() {
            if let Some(remaining_crystals) = view::remaining_crystals(cell, available_resources, view) {
                let harvest = harvest_map.calculate_harvest_at(cell, remaining_crystals);
                if harvest > 0 {
                    /*
                    eprintln!(
                        "{} harvested {} crystals from {}",
                        if player == 0 { "We" } else { "Enemy" },
                        harvest, cell);
                    */

                    harvested[player] += harvest;
                }
            }
        }
    }
}