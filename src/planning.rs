use core::panic;
use super::fnv::FnvHashSet;
use std::fmt::Display;

use super::inputs::{Content,MAX_TICKS};
use super::movement;
use super::view::*;

#[derive(Clone)]
pub struct Milestone {
    pub cell: usize,
}
impl Milestone {
    pub fn new(cell: usize) -> Self {
        Self { cell }
    }

    pub fn is_complete(&self, state: &State) -> bool {
        state.resources[self.cell] <= 0
    }
}
impl Display for Milestone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cell.fmt(f)
    }
}

pub fn enact_plan(player: usize, plan: &[Milestone], view: &View, state: &State) -> Commands {
    let total_ants: i32 = state.num_ants[player].iter().cloned().sum();
    let value_per_egg = calculate_egg_value(view, state);

    let mut num_crystal_harvests = 0;
    let mut num_egg_harvests = 0;
    let mut targets = Vec::new();
    let mut beacons = FnvHashSet::default();
    for &base in view.layout.bases[player].iter() {
        beacons.insert(base);
    }

    for milestone in plan.iter().skip_while(|m| m.is_complete(&state)) {
        let initial_distance = beacons.len() as i32;
        let initial_collection_rate = calculate_collection_rate(total_ants, initial_distance, num_crystal_harvests, num_egg_harvests, value_per_egg);

        let target = milestone.cell;

        if let Some((distance, source)) = beacons.iter().map(|&source| (view.paths.distance_between(source, target),source)).min() {
            let content = view.layout.cells[target].content;
            let (new_crystal_harvests, new_egg_harvests) = match content {
                Some(Content::Crystals) => (num_crystal_harvests+1, num_egg_harvests),
                Some(Content::Eggs) => (num_crystal_harvests, num_egg_harvests+1),
                None => (num_crystal_harvests, num_egg_harvests),
            };

            let new_collection_rate = calculate_collection_rate(total_ants, initial_distance + distance, new_crystal_harvests, new_egg_harvests, value_per_egg);
            // eprintln!("considered harvesting <{}> (distance {}): {} -> {}", target, distance, initial_collection_rate, new_collection_rate);

            if new_collection_rate > initial_collection_rate {
                for cell in view.paths.calculate_path(source, target, &view.layout) {
                    beacons.insert(cell);
                }
                targets.push(target);

                num_crystal_harvests = new_crystal_harvests;
                num_egg_harvests = new_egg_harvests;

            } else {
                // Best harvest not worth it, so none others will be either
                break;
            }

        } else {
            panic!("no sources available for harvest");
        }
    }

    Commands {
        assignments: movement::spread_ants_across_beacons(beacons.into_iter(), player, state),
        targets,
    }
}

pub struct Commands {
    pub assignments: Box<[i32]>,
    pub targets: Vec<usize>,
}
impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.targets.is_empty() {
            write!(f, "-")?;
        } else {
            let mut is_first = true;
            for &target in self.targets.iter() {
                if is_first {
                    is_first = false;
                } else {
                    write!(f, " ")?;
                }
                write!(f, "{}", target)?;
            }
        }
        Ok(())
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