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
    pub content: Option<Content>,
    pub neighbors: Vec<usize>,
    pub initial_resources: i32,
}

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub enum Content {
    Eggs,
    Crystals,
}

#[derive(Clone)]
pub struct CellState {
    pub resources: i32,
    pub num_my_ants: i32,
    pub num_enemy_ants: i32,
}

#[allow(dead_code)]
pub enum Action {
    Beacon { index: usize, strength: i32 },
    Line { source: usize, target: usize, strength: i32 },
    Message { text: String },
    Wait,
}
