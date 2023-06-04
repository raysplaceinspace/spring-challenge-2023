use super::inputs::Content;
use super::view::*;

pub struct HarvestEvaluator {
    total_ants: i32,
}
impl HarvestEvaluator {
    pub fn new(player: usize, state: &State) -> Self {
        Self {
            total_ants: state.total_ants[player],
        }
    }

    pub fn calculate_harvest_rate(&self, num_harvests: i32, spread: i32) -> i32 {
        if spread <= 0 { return 0 }
        let harvest_per_cell = self.total_ants / spread; // intentional integer division since ants can't be split
        harvest_per_cell * num_harvests
    }
}

pub struct SpawnEvaluator {
    #[allow(dead_code)]
    player: usize,
    total_ants: i32,
    ticks_to_harvest_remaining_crystals: i32,
}
impl SpawnEvaluator {
    pub fn new(player: usize, view: &View, state: &State) -> Self {
        let total_ants: i32 = state.total_ants[player];

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
            ticks_to_harvest_remaining_crystals,
        }
    }

    pub fn is_worth_harvesting(&self, cell: usize, view: &View, state: &State, travel_ticks: i32) -> bool {
        if state.resources[cell] <= 0 { return false }

        match view.layout.cells[cell].content {
            None => false,
            Some(Content::Crystals) => true,
            Some(Content::Eggs) => {
                let distance_to_base = view.distance_to_closest_base[self.player][cell];
                let harvest_per_tick = self.total_ants / distance_to_base;

                let num_eggs = state.resources[cell];
                let ticks_to_harvest_eggs = (num_eggs as f32 / harvest_per_tick as f32).ceil() as i32;

                // only harvest if we can save more ticks than we lose
                self.calculate_ticks_saved_harvesting_eggs(num_eggs) >= (travel_ticks + ticks_to_harvest_eggs) as f32
            },
        }
    }

    pub fn calculate_ticks_saved_harvesting_eggs(&self, num_eggs: i32) -> f32 {
        self.ticks_to_harvest_remaining_crystals as f32 * (num_eggs as f32 / (self.total_ants + num_eggs) as f32)
    }

    pub fn ticks_to_harvest_remaining_crystals(&self) -> i32 {
        self.ticks_to_harvest_remaining_crystals
    }
}