mod agent;
mod interface;
mod model;
mod paths;

use agent::Agent;
use model::*;

fn main() {
    let mut layout = Layout::new();
    let mut states = Vec::new();
    interface::read_initial(&mut layout, &mut states);

    let mut agent = Agent::new(layout);
    loop {
        interface::read_turn(agent.layout(), &mut states);

        let actions = agent.act(&states);

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
    }
}