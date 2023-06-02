use super::inputs::*;
use super::pathing::*;

pub type AntsPerCell = Box<[i32]>;
pub type AntsPerCellPerPlayer = [AntsPerCell; NUM_PLAYERS];
pub type ResourcesPerCell = Box<[i32]>;
pub type ClosestBases = Box<[usize]>;
pub type ClosestBasesPerPlayer = [ClosestBases; NUM_PLAYERS];
pub type ClosestResources = Box<[usize]>;
pub type ClosestResourcesPerPlayer = [ClosestResources; NUM_PLAYERS];

/// A Layout plus some pre-calculated values derived from the Layout
pub struct View {
    pub layout: Layout,
    pub initial_crystals: i32,
    pub paths: PathMap,

    /// player -> cell -> closest base
    pub closest_bases: ClosestBasesPerPlayer,

    /// player -> cell containing resources, sorted nearest to farthest
    pub closest_crystals: ClosestResourcesPerPlayer,
    pub closest_eggs: ClosestResourcesPerPlayer,
}
impl View {
    pub fn new(layout: Layout) -> Self {
        let paths = PathMap::generate(&layout);

        Self {
            initial_crystals:
                layout.cells.iter()
                .filter(|cell| cell.content == Some(Content::Crystals))
                .map(|cell| cell.initial_resources)
                .sum(),

            closest_bases: [
                Self::find_closest_bases(ME, &layout, &paths),
                Self::find_closest_bases(ENEMY, &layout, &paths),
            ],

            closest_crystals: [
                Self::find_closest_resources(Content::Crystals, ME, &layout, &paths),
                Self::find_closest_resources(Content::Crystals, ENEMY, &layout, &paths),
            ],

            closest_eggs: [
                Self::find_closest_resources(Content::Eggs, ME, &layout, &paths),
                Self::find_closest_resources(Content::Eggs, ENEMY, &layout, &paths),
            ],

            paths,
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

    fn find_closest_resources(content: Content, player: usize, layout: &Layout, paths: &PathMap) -> ClosestResources {
        let mut closest_resources: Vec<usize> = (0..layout.cells.len()).filter(|&cell| layout.cells[cell].content == Some(content)).collect();
        closest_resources.sort_by_cached_key(|&resource| {
            layout.bases[player].iter().map(|&base| {
                paths.distance_between(base, resource)
            }).min().expect("bases missing")
        });
        closest_resources.into_boxed_slice()
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

pub fn find_winner(crystals: &CrystalsPerPlayer, view: &View) -> Option<usize> {
    let threshold = view.initial_crystals / 2;
    for player in 0..NUM_PLAYERS {
        if crystals[player] >= threshold {
            return Some(player);
        }
    }
    None
}