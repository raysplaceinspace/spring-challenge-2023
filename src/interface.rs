use std::io;
use super::inputs::*;

pub struct TurnInput {
    pub crystals_per_player: CrystalsPerPlayer,
    pub num_ants_per_cell: [Box<[i32]>; NUM_PLAYERS],
    pub resources_per_cell: Box<[i32]>,
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

pub fn read_initial() -> Layout {
    let mut cells = Vec::new();

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let number_of_cells = parse_input!(input_line, i32); // amount of hexagonal cells in this map
    for _ in 0..number_of_cells as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();

        let contents = match parse_input!(inputs[0], i32) {
            0 => None,
            1 => Some(Content::Eggs),
            2 => Some(Content::Crystals),
            wrong => panic!("Invalid cell type: {}", wrong),
        }; // 0 for empty, 1 for eggs, 2 for crystal

        let initial_resources = parse_input!(inputs[1], i32); // the initial amount of eggs/crystals on this cell

        let mut neighbors = Vec::new();
        for i in 0..6 {
            let neighbor = parse_input!(inputs[2+i], i32); // the index of the neighbouring cell for each direction
            if neighbor >= 0 {
                neighbors.push(neighbor as usize);
            }
        }

        cells.push(CellLayout {
            content: contents,
            neighbors: neighbors.into_boxed_slice(),
            initial_resources,
        });
    }
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    let _number_of_bases = parse_input!(input_line, i32);

    let mut my_bases = Vec::new();
    let mut inputs = String::new();
    io::stdin().read_line(&mut inputs).unwrap();
    for i in inputs.split_whitespace() {
        let my_base_index = parse_input!(i, usize);
        my_bases.push(my_base_index);
    }

    let mut enemy_bases = Vec::new();
    let mut inputs = String::new();
    io::stdin().read_line(&mut inputs).unwrap();
    for i in inputs.split_whitespace() {
        let opp_base_index = parse_input!(i, usize);
        enemy_bases.push(opp_base_index);
    }

    let layout = Layout {
        cells: cells.into_boxed_slice(),
        bases: [my_bases.into_boxed_slice(), enemy_bases.into_boxed_slice()],
    };
    layout
}

pub fn read_turn(layout: &Layout) -> TurnInput {
    let mut resources_per_cell = Vec::with_capacity(layout.cells.len());
    let mut num_my_ants_per_cell = Vec::with_capacity(layout.cells.len());
    let mut num_enemy_ants_per_cell = Vec::with_capacity(layout.cells.len());

    let crystals_per_player = {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        [
            parse_input!(inputs[0], i32),
            parse_input!(inputs[1], i32),
        ]
    };

    for _ in 0..layout.cells.len() {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();

        resources_per_cell.push(parse_input!(inputs[0], i32)); // the current amount of eggs/crystals on this cell
        num_my_ants_per_cell.push(parse_input!(inputs[1], i32)); // the amount of your ants on this cell
        num_enemy_ants_per_cell.push(parse_input!(inputs[2], i32)); // the amount of opponent ants on this cell
    }

    TurnInput {
        crystals_per_player,
        resources_per_cell: resources_per_cell.into_boxed_slice(),
        num_ants_per_cell: [
            num_my_ants_per_cell.into_boxed_slice(),
            num_enemy_ants_per_cell.into_boxed_slice(),
        ],
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
