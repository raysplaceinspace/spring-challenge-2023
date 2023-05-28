mod agent;
mod interface;
mod model;

use model::*;

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
    let mut layout = Layout::new();
    let mut state = State::new();
    interface::read_initial(&mut layout, &mut state);

    // game loop
    loop {
        interface::read_turn(&layout, &mut state);

        let actions = agent::act(&layout, &state);

        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");


        // WAIT | LINE <sourceIdx> <targetIdx> <strength> | BEACON <cellIdx> <strength> | MESSAGE <text>
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