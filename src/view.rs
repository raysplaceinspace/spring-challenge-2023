use super::inputs::*;
use super::pathing::*;

pub type AntsPerCell = Box<[i32]>;
pub type AntsPerCellPerPlayer = [AntsPerCell; NUM_PLAYERS];
pub type ResourcesPerCell = Box<[i32]>;
pub type CrystalsPerPlayer = [i32; NUM_PLAYERS];
pub type ClosestBases = Box<[usize]>;
pub type ClosestBasesPerPlayer = [ClosestBases; NUM_PLAYERS];

/// A Layout plus some pre-calculated values derived from the Layout
pub struct View {
    pub layout: Layout,
    pub initial_crystals: i32,
    pub paths: PathMap,

    /// player -> cell -> closest base
    pub closest_bases: ClosestBasesPerPlayer,
}
impl View {
    pub fn new(layout: Layout) -> Self {
        let paths = PathMap::generate(&layout);
        let initial_crystals = layout.cells.iter().filter(|cell| cell.content == Some(Content::Crystals)).map(|cell| cell.initial_resources).sum();
        let closest_bases = [
            Self::find_closest_bases(ME, &layout, &paths),
            Self::find_closest_bases(ENEMY, &layout, &paths),
        ];
        Self {
            paths,
            initial_crystals,
            closest_bases,
            layout,
        }
    }

    fn find_closest_bases(player: usize, layout: &Layout, paths: &PathMap) -> ClosestBases {
        let closest_bases: Vec<usize> = (0..layout.cells.len()).map(|target| {
            layout.bases[player].iter().cloned().min_by_key(|&base| {
                paths.distance_between(base, target)
            }).expect("bases missing")
        }).collect();
        closest_bases.into_boxed_slice()
    }
}

#[derive(Clone)]
pub struct State {
    pub tick: u32,
    pub num_ants: AntsPerCellPerPlayer,
    pub resources: ResourcesPerCell,
    pub crystals: CrystalsPerPlayer,
}
impl State {
    pub fn new(tick: u32, num_ants: AntsPerCellPerPlayer, resources: ResourcesPerCell, harvested: CrystalsPerPlayer) -> Self {
        Self {
            tick,
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

pub fn find_winner(crystals: &CrystalsPerPlayer, view: &View) -> Option<usize> {
    let threshold = view.initial_crystals / 2;
    for player in 0..NUM_PLAYERS {
        if crystals[player] > threshold {
            return Some(player);
        }
    }
    None
}