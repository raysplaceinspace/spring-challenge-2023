mod agent;
mod interface;
mod harvesting;
mod model;
mod paths;
mod view;

use model::*;
use view::{State,View,HarvestedPerPlayer};

use interface::TurnInput;

fn main() {
    let layout = interface::read_initial();
    let view = View::new(layout);

    let mut previous_state: Option<State> = None;
    loop {
        // Read input
        let TurnInput { num_ants_per_cell, resources_per_cell } = interface::read_turn(&view.layout);

        // Calculate new state
        let state = match previous_state {
            None => State::new(num_ants_per_cell, resources_per_cell, HarvestedPerPlayer::default()),
            Some(previous) => {
                let available_resources = &previous.resources_per_cell; // Look at previous tick to determine available resources
                let mut harvested = harvesting::harvest(&view, &num_ants_per_cell, available_resources);
                for (i,harvested) in harvested.iter_mut().enumerate() {
                    *harvested += previous.harvested[i];
                }
                State::new(num_ants_per_cell, resources_per_cell, harvested)
            },
        };
        eprintln!("Harvested: me={}, enemy={}", state.harvested[0], state.harvested[1]);

        // Calculate actions
        let actions = agent::act(&view, &state);

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

        previous_state = Some(state);
    }
}