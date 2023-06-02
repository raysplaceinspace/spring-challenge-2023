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
    ticks_to_harvest_remaining_crystals: i32,
    remaining_ticks: i32,
}
impl HarvestEvaluator {
    pub fn new(player: usize, view: &View, state: &State) -> Self {
        let total_ants: i32 = state.num_ants[player].iter().cloned().sum();
        let remaining_ticks = MAX_TICKS.saturating_sub(state.tick) as i32;

        let crystal_threshold = view.initial_crystals / 2;
        let mut remaining_crystals = (crystal_threshold - state.crystals[player]).max(0);
        if remaining_crystals <= 0 {
            return Self { total_ants, remaining_ticks, ticks_to_harvest_remaining_crystals: 0 };
        }

        let mut ticks_to_harvest_remaining_crystals = 0;
        for &cell in view.closest_crystals[player].iter() {
            let available = state.resources[cell];
            if available <= 0 { continue }

            let harvest = remaining_crystals.min(available);
            let base = view.closest_bases[player][cell];
            let distance = view.paths.distance_between(base, cell);

            let harvest_per_tick = total_ants / distance;
            if harvest_per_tick <= 0 {
                return Self { total_ants, remaining_ticks, ticks_to_harvest_remaining_crystals: i32::MAX };
            }

            let ticks_to_harvest = (harvest as f32 / harvest_per_tick as f32).ceil() as i32;

            ticks_to_harvest_remaining_crystals += ticks_to_harvest;
            remaining_crystals -= harvest;

            if remaining_crystals <= 0 { break }
        }

        Self {
            total_ants,
            ticks_to_harvest_remaining_crystals,
            remaining_ticks,
        }
    }

    pub fn is_worth_harvesting(&self, cell: usize, player: usize, view: &View, state: &State) -> bool {
        let available = state.resources[cell];
        if available <= 0 { return false }

        match view.layout.cells[cell].content {
            None => false,
            Some(Content::Crystals) => true,
            Some(Content::Eggs) => {
                let base = view.closest_bases[player][cell];
                let distance_from_base = view.paths.distance_between(base, cell);
                self.is_worth_harvesting_eggs(distance_from_base)
            },
        }
    }

    fn is_worth_harvesting_eggs(&self, distance_from_base: i32) -> bool {
        let harvest = self.total_ants / distance_from_base; // Don't consider number of available eggs - just assume we can reach multiple identical cells like this and harvest the maximum number of eggs per tick
        if harvest <= 0 { return false }

        if self.ticks_to_harvest_remaining_crystals == i32::MAX { return true } // Not enough eggs to win, so must harvest eggs

        // First tick of harvesting eggs is the most productive (highest increase in ratio of number of new ants to existing ants),
        // so if it doesn't break even, none of the remaining ticks will either.
        let new_ants = self.total_ants + harvest;
        let new_ticks = (1 + (self.ticks_to_harvest_remaining_crystals as f32 * (self.total_ants as f32 / new_ants as f32)).ceil() as i32).min(self.remaining_ticks); // +1 because we are delaying harvesting crystals by 1 tick to harvest eggs instead
        let old_ticks = self.ticks_to_harvest_remaining_crystals.min(self.remaining_ticks);

        new_ticks < old_ticks
    }

    pub fn calculate_harvest_rate(&self, counts: &NumHarvests, spread: i32) -> f32 {
        if spread <= 0 { return 0.0 }
        let harvest_per_cell = self.total_ants / spread; // intentional integer division since ants can't be split

        let num_crystals_harvested = harvest_per_cell * counts.num_crystal_harvests;
        let num_eggs_harvested = harvest_per_cell * counts.num_egg_harvests;

        num_crystals_harvested as f32 + num_eggs_harvested as f32
    }
}