use super::inputs::{Content,MAX_TICKS};
use super::view::*;

#[derive(Clone,Copy,Debug,PartialEq,PartialOrd)]
pub struct ValueOrd(pub f32);
impl ValueOrd {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}
impl Eq for ValueOrd {}
impl Ord for ValueOrd {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).expect("unexpected NaN for value")
    }
}

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

pub struct HarvestAndSpawnEvaluator {
    #[allow(dead_code)]
    player: usize,
    total_ants: i32,
    ticks_to_harvest_remaining_crystals: i32,
    remaining_ticks_proportion: f32,
}
impl HarvestAndSpawnEvaluator {
    pub fn new(player: usize, view: &View, state: &State) -> Self {
        let total_ants: i32 = state.total_ants[player];
        let remaining_ticks_proportion = MAX_TICKS.saturating_sub(state.tick) as f32 / MAX_TICKS as f32;

        let crystal_threshold = view.initial_crystals / 2;
        let mut remaining_crystals = (crystal_threshold - state.crystals[player]).max(0);
        let mut ticks_to_harvest_remaining_crystals = 0;
        for &cell in view.closest_crystals[player].iter() {
            if remaining_crystals <= 0 { break }

            let available = state.resources[cell];
            if available <= 0 { continue }

            let harvest = remaining_crystals.min(available);
            let distance = view.distance_to_closest_base[player][cell];

            let harvest_per_tick = (total_ants / distance).max(1); // If too far away to harvest, just pretend we can harvest it slowly - the math will work out about the same and we don't have to worry about infinities
            let ticks_to_harvest = (harvest as f32 / harvest_per_tick as f32).ceil() as i32;

            ticks_to_harvest_remaining_crystals += ticks_to_harvest;
            remaining_crystals -= harvest;
        }

        Self {
            player,
            total_ants,
            remaining_ticks_proportion,
            ticks_to_harvest_remaining_crystals,
        }
    }

    #[allow(dead_code)]
    pub fn is_worth_harvesting(&self, cell: usize, view: &View, state: &State) -> bool {
        if state.resources[cell] <= 0 { return false }

        match view.layout.cells[cell].content {
            None => false,
            Some(Content::Crystals) => true,
            Some(Content::Eggs) => {
                let distance = view.distance_to_closest_base[self.player][cell];
                let harvest_per_tick = self.total_ants / distance;
                self.calculate_ticks_saved_harvesting_eggs(harvest_per_tick) >= 1.0 // only harvest if we can save more ticks than we lose
            },
        }
    }

    pub fn calculate_harvest_rate(&self, counts: &NumHarvests, spread: i32) -> f32 {
        if spread <= 0 { return 0.0 }
        let harvest_per_cell = self.total_ants / spread; // intentional integer division since ants can't be split

        let num_crystals_harvested = harvest_per_cell * counts.num_crystal_harvests;
        let num_eggs_harvested = harvest_per_cell * counts.num_egg_harvests;

        let egg_harvesting_value = (self.calculate_ticks_saved_harvesting_eggs(num_eggs_harvested) / 1.0).min(1.0);
        let egg_time_value = self.remaining_ticks_proportion;
        let egg_value = egg_harvesting_value.min(egg_time_value);

        num_crystals_harvested as f32 + num_eggs_harvested as f32 * egg_value
    }

    pub fn calculate_ticks_saved_harvesting_eggs(&self, num_eggs: i32) -> f32 {
        self.ticks_to_harvest_remaining_crystals as f32 * (num_eggs as f32 / (self.total_ants + num_eggs) as f32)
    }

    pub fn ticks_to_harvest_remaining_crystals(&self) -> i32 {
        self.ticks_to_harvest_remaining_crystals
    }
}