mod agent;
mod evaluation;
mod fnv;
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
use inputs::*;
use view::*;

use interface::TurnInput;

fn main() {
    let layout = interface::read_initial();
    let view = View::new(layout);

    let mut agent = Agent::new(ME, &view);
    let mut tick = 0;
    loop {
        // Read input
        let TurnInput { crystals_per_player, num_ants_per_cell, resources_per_cell } = interface::read_turn(&view.layout);

        // Calculate new state
        let state = State::new(tick, num_ants_per_cell, resources_per_cell, crystals_per_player);
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
    }
}