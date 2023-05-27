use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

pub struct Layout {
    pub cells: Vec<CellLayout>,
    pub my_bases: Vec<usize>,
    pub enemy_bases: Vec<usize>,
}
impl Layout {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            my_bases: Vec::new(),
            enemy_bases: Vec::new(),
        }
    }
}

pub struct CellLayout {
    pub cell_type: CellType,
    pub neighbors: Vec<usize>,
}

#[derive(Clone)]
pub struct State {
    pub cells: Vec<CellState>,
}
impl State {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct CellState {
    pub resources: i32,
    pub num_my_ants: i32,
    pub num_enemy_ants: i32,
}

pub enum CellType {
    Normal,
    Egg,
    Crystal,
}

pub enum Action {
    Beacon { index: usize, strength: i32 },
    Line { source: usize, target: usize, strength: i32 },
    Message { text: String },
    Wait,
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
    let mut layout = Layout::new();
    let mut state = State::new();
    read_initial(&mut layout, &mut state);

    // game loop
    loop {
        read_turn(&layout, &mut state);

        let actions = act(&layout, &state);

        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");


        // WAIT | LINE <sourceIdx> <targetIdx> <strength> | BEACON <cellIdx> <strength> | MESSAGE <text>
        if actions.is_empty() {
            println!("{}", format_action(&Action::Wait));
        } else {
            for (i,action) in actions.into_iter().enumerate() {
                if i > 0 {
                    print!(";");
                }
                print!("{}", format_action(&action));
            }
            println!("");
        }
    }
}

fn read_initial(layout: &mut Layout, state: &mut State) {
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

        state.cells.push(CellState {
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

fn read_turn(layout: &Layout, state: &mut State) {
    for i in 0..layout.cells.len() {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();

        let cell = &mut state.cells[i];
        cell.resources = parse_input!(inputs[0], i32); // the current amount of eggs/crystals on this cell
        cell.num_my_ants = parse_input!(inputs[1], i32); // the amount of your ants on this cell
        cell.num_enemy_ants = parse_input!(inputs[2], i32); // the amount of opponent ants on this cell
    }
}

fn format_action(action: &Action) -> String {
    match action {
        Action::Beacon { index, strength } => format!("BEACON {} {}", index, strength),
        Action::Line { source, target, strength } => format!("LINE {} {} {}", source, target, strength),
        Action::Message { text } => format!("MESSAGE {}", text),
        Action::Wait => format!("WAIT"),
    }
}

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