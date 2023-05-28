use std::io;
use super::model::*;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

pub fn read_initial(layout: &mut Layout, states: &mut Vec<CellState>) {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let number_of_cells = parse_input!(input_line, i32); // amount of hexagonal cells in this map
    for _ in 0..number_of_cells as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();

        let cell_type = match parse_input!(inputs[0], i32) {
            0 => CellType::Normal,
            1 => CellType::Egg,
            2 => CellType::Crystal,
            wrong => panic!("Invalid cell type: {}", wrong),
        }; // 0 for empty, 1 for eggs, 2 for crystal

        let initial_resources = parse_input!(inputs[1], i32); // the initial amount of eggs/crystals on this cell

        let mut neighbors = Vec::new();
        for _ in 0..6 {
            let neighbor = parse_input!(inputs[2], i32); // the index of the neighbouring cell for each direction
            if neighbor >= 0 {
                neighbors.push(neighbor as usize);
            }
        }

        layout.cells.push(CellLayout {
            cell_type,
            neighbors,
        });

        states.push(CellState {
            resources: initial_resources,
            num_my_ants: 0,
            num_enemy_ants: 0,
        });
    }
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    let _number_of_bases = parse_input!(input_line, i32);
    let mut inputs = String::new();
    io::stdin().read_line(&mut inputs).unwrap();
    for i in inputs.split_whitespace() {
        let my_base_index = parse_input!(i, usize);
        layout.my_bases.push(my_base_index);
    }
    let mut inputs = String::new();
    io::stdin().read_line(&mut inputs).unwrap();
    for i in inputs.split_whitespace() {
        let opp_base_index = parse_input!(i, usize);
        layout.enemy_bases.push(opp_base_index);
    }
}

pub fn read_turn(layout: &Layout, states: &mut Vec<CellState>) {
    for i in 0..layout.cells.len() {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();

        let cell = &mut states[i];
        cell.resources = parse_input!(inputs[0], i32); // the current amount of eggs/crystals on this cell
        cell.num_my_ants = parse_input!(inputs[1], i32); // the amount of your ants on this cell
        cell.num_enemy_ants = parse_input!(inputs[2], i32); // the amount of opponent ants on this cell
    }
}

pub fn format_action(action: &Action) -> String {
    match action {
        Action::Beacon { index, strength } => format!("BEACON {} {}", index, strength),
        Action::Line { source, target, strength } => format!("LINE {} {} {}", source, target, strength),
        Action::Message { text } => format!("MESSAGE {}", text),
        Action::Wait => format!("WAIT"),
    }
}
