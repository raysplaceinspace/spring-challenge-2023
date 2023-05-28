mod agent;
mod interface;
mod model;

use model::*;

fn main() {
    let mut layout = Layout::new();
    let mut states = Vec::new();
    interface::read_initial(&mut layout, &mut states);

    // game loop
    loop {
        interface::read_turn(&layout, &mut states);

        let actions = agent::act(&layout, &states);

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