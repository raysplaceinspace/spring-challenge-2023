pub const NUM_PLAYERS: usize = 2;
pub const ME: usize = 0;
#[allow(dead_code)]
pub const ENEMY: usize = 1;

pub struct Layout {
    pub cells: Box<[CellLayout]>,
    pub bases: [Box<[usize]>; NUM_PLAYERS],
}

pub struct CellLayout {
    pub content: Option<Content>,
    pub neighbors: Box<[usize]>,
    pub initial_resources: i32,
}

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub enum Content {
    Eggs,
    Crystals,
}

#[allow(dead_code)]
pub enum Action {
    Beacon { index: usize, strength: i32 },
    Line { source: usize, target: usize, strength: i32 },
    Message { text: String },
    Wait,
}