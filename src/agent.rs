use super::model::*;

pub fn act(layout: &Layout, states: &Vec<CellState>) -> Vec<Action> {
    let mut actions = Vec::new();

    let my_base = layout.my_bases[0];

    for (id,cell) in states.iter().enumerate() {
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