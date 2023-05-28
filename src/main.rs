mod agent;
mod interface;
mod harvesting;
mod model;
mod paths;
mod view;

use model::*;
use view::{State,View};

fn main() {
    let layout = interface::read_initial();
    let view = View::new(layout);

    let mut previous_state: Option<State> = None;
    loop {
        let cells = interface::read_turn(&view.layout);

        let state = match previous_state {
            None => State::new(cells),
            Some(previous) => previous.forward(cells, &view),
        };
        eprintln!("Harvested: me={}, enemy={}", state.harvested[0], state.harvested[1]);
        let actions = agent::act(&view, &state);

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