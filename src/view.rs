use super::inputs::*;
use super::paths::*;

/// A Layout plus some pre-calculated values derived from the Layout
pub struct View {
    pub layout: Layout,
    pub initial_crystals: i32,
    pub paths: PathMap,
}
impl View {
    pub fn new(layout: Layout) -> Self {
        Self {
            paths: PathMap::generate(&layout),
            initial_crystals: layout.cells.iter().filter(|cell| cell.content == Some(Content::Crystals)).map(|cell| cell.initial_resources).sum(),
            layout,
        }
    }
}

pub type AntsPerCell = Box<[i32]>;
pub type AntsPerCellPerPlayer = [AntsPerCell; NUM_PLAYERS];
pub type ResourcesPerCell = Box<[i32]>;
pub type HarvestedPerPlayer = [i32; NUM_PLAYERS];

#[derive(Clone)]
pub struct State {
    pub num_ants: AntsPerCellPerPlayer,
    pub resources: ResourcesPerCell,
    pub crystals: HarvestedPerPlayer,
}
impl State {
    pub fn new(num_ants: AntsPerCellPerPlayer, resources: ResourcesPerCell, harvested: HarvestedPerPlayer) -> Self {
        Self {
            num_ants,
            resources,
            crystals: harvested,
        }
    }
}

pub fn remaining_crystals(cell: usize, resources: &ResourcesPerCell, view: &View) -> Option<i32> {
    if view.layout.cells[cell].content == Some(Content::Crystals) {
        Some(resources[cell])
    } else {
        None
    }
}

pub fn find_winner(crystals: &HarvestedPerPlayer, view: &View) -> Option<usize> {
    let threshold = view.initial_crystals / 2;
    for player in 0..NUM_PLAYERS {
        if crystals[player] > threshold {
            return Some(player);
        }
    }
    None
}