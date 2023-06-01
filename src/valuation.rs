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

pub struct ValuationCalculator {
    total_ants: i32,
    value_per_egg: f32,
}
impl ValuationCalculator {
    pub fn new(player: usize, state: &State) -> Self {
        Self {
            total_ants: state.num_ants[player].iter().cloned().sum(),
            value_per_egg: 1.0,
        }
    }

    pub fn with_egg_decay(mut self, view: &View, state: &State) -> Self {
        self.value_per_egg = calculate_egg_value(view, state);
        self
    }

    pub fn calculate(&self, counts: &NumHarvests, distance: i32) -> f32 {
        calculate_collection_rate(self.total_ants, distance, counts.num_crystal_harvests, counts.num_egg_harvests, self.value_per_egg)
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

fn calculate_collection_rate(total_ants: i32, total_distance: i32, num_crystal_harvests: i32, num_egg_harvests: i32, value_per_egg: f32) -> f32 {
    if total_distance <= 0 { return 0.0 }
    let per_cell = total_ants / total_distance; // intentional integer division since ants can't be split
    num_crystal_harvests as f32 * per_cell as f32 + value_per_egg * num_egg_harvests as f32 * per_cell as f32
}

fn calculate_egg_value(view: &View, state: &State) -> f32 {
    let max_crystals = view.initial_crystals;
    let crystals_harvested: i32 = state.crystals.iter().cloned().sum();
    let crystals_remaining = (max_crystals - crystals_harvested).max(0);
    let crystals_remaining_proportion = (crystals_remaining as f32 / max_crystals as f32).max(0.0);

    let ticks_remaining = MAX_TICKS - state.tick;
    let ticks_remaining_proportion = (ticks_remaining as f32 / MAX_TICKS as f32).max(0.0);

    crystals_remaining_proportion.min(ticks_remaining_proportion)
}