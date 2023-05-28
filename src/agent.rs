use super::model::*;

pub fn act(layout: &Layout, state: &State) -> Vec<Action> {
    let mut actions = Vec::new();

    let my_base = layout.my_bases[0];

    for (id,cell) in state.cells.iter().enumerate() {
        if cell.resources > 0 {
            actions.push(Action::Line {
                source: my_base,
                target: id,
                strength: 1,
            });
        }
    }

    actions
}