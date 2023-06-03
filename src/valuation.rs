use super::inputs::Content;
use super::view::*;

#[derive(Clone,Copy)]
pub struct NumHarvests {
    pub num_crystal_harvests: i32,
    pub num_egg_harvests: i32,
}
impl NumHarvests {
    pub fn new() -> Self {
        Self { num_crystal_harvests: 0, num_egg_harvests: 0 }
    }

    pub fn add(mut self, content: Option<Content>) -> Self {
        match content {
            Some(Content::Crystals) => self.num_crystal_harvests += 1,
            Some(Content::Eggs) => self.num_egg_harvests += 1,
            None => (),
        };
        self
    }
}

pub struct HarvestEvaluator {
    total_ants: i32,
}
impl HarvestEvaluator {
    pub fn new(player: usize, state: &State) -> Self {
        Self {
            total_ants: state.total_ants[player],
        }
    }

    pub fn calculate_harvest_rate(&self, counts: &NumHarvests, spread: i32) -> f32 {
        if spread <= 0 { return 0.0 }
        let harvest_per_cell = self.total_ants / spread; // intentional integer division since ants can't be split

        let num_crystals_harvested = harvest_per_cell * counts.num_crystal_harvests;
        let num_eggs_harvested = harvest_per_cell * counts.num_egg_harvests;

        num_crystals_harvested as f32 + num_eggs_harvested as f32
    }
}