use super::interface::TurnInput;

use super::harvesting;
use super::model::*;
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

pub type AntsPerCell = [Box<[i32]>; NUM_PLAYERS];
pub type ResourcesPerCell = Box<[i32]>;
pub type HarvestedPerPlayer = [i32; NUM_PLAYERS];

#[derive(Clone)]
pub struct State {
    pub num_ants_per_cell: AntsPerCell,
    pub resources_per_cell: ResourcesPerCell,
    pub harvested: HarvestedPerPlayer,
}
impl State {
    pub fn new(input: TurnInput) -> Self {
        Self {
            num_ants_per_cell: input.num_ants_per_cell,
            resources_per_cell: input.resources_per_cell,
            harvested: [0; NUM_PLAYERS],
        }
    }

    pub fn forward(&self, input: TurnInput, view: &View) -> Self {
        let mut harvested = self.harvested.clone();
        let available_resources = &self.resources_per_cell; // Look at previous tick to determine available resources
        harvesting::harvest(view, &input.num_ants_per_cell, available_resources, &mut harvested);

        Self {
            num_ants_per_cell: input.num_ants_per_cell,
            resources_per_cell: input.resources_per_cell,
            harvested,
        }
    }
}