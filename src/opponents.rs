use std::fmt::Display;

use super::harvesting;
use super::inputs::{Content,NUM_PLAYERS};
use super::fnv::FnvHashSet;
use super::view::*;
use super::movement::{self,Assignments};
use super::pathing::NearbyPathMap;
use super::valuation::{HarvestEvaluator,SpawnEvaluator};

pub struct Countermoves {
    pub assignments: Assignments,
    pub harvests: Vec<usize>,
}
impl Display for Countermoves {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.harvests.is_empty() {
            write!(f, "-")?;
        } else {
            let mut is_first = true;
            for &target in self.harvests.iter() {
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

pub fn enact_countermoves(player: usize, view: &View, state: &State) -> Countermoves {
    let total_ants = state.total_ants[player];
    let enemy = (player + 1) % NUM_PLAYERS;
    let attacks = harvesting::calculate_max_flow_for_player(enemy, view, &state.num_ants);

    // Prepare candidates
    let nearby = NearbyPathMap::near_my_ants(player, view, state);
    let evaluator = HarvestEvaluator::new(player, state);
    let spawner = SpawnEvaluator::new(player, view, state);
    let mut candidates: FnvHashSet<usize> =
        view.closest_resources[player].iter().cloned()
        .filter(|&cell| spawner.is_worth_harvesting(cell, view, state, nearby.distance_to(cell)))
        .collect();

    // Extend to collect nearby crystals
    let mut harvests = Vec::new();
    let mut beacons: FnvHashSet<usize> = FnvHashSet::default();
    let mut harvest_mesh = NearbyPathMap::generate(&view.layout, view.layout.bases[player].iter().cloned());
    while !candidates.is_empty() && (beacons.len() as i32) < total_ants {
        let initial_harvests = harvests.len() as i32;
        let initial_spread = beacons.len() as i32;
        let initial_collection_rate = evaluator.calculate_harvest_rate(initial_harvests, initial_spread);

        // Find closest next target
        if let Some((_, extra_spread, target)) =
            candidates.iter()
            .filter_map(|&target| {
                let extra_spread = harvest_mesh.distance_to(target);
                let new_collection_rate = evaluator.calculate_harvest_rate(initial_harvests + 1, initial_spread + extra_spread);
                if new_collection_rate <= initial_collection_rate { return None } // This target is not worth the effort

                let mut ticks_lost = nearby.distance_to(target);
                if view.layout.cells[target].content == Some(Content::Eggs) {
                    // Treat eggs as closer than they are if harvesting them saves ticks rather than costs them
                    let harvest_per_tick = total_ants / (initial_spread + extra_spread);
                    let num_eggs = harvest_per_tick.min(state.resources[target]);
                    ticks_lost -= spawner.calculate_ticks_saved_harvesting_eggs(num_eggs).floor() as i32;
                }

                Some((ticks_lost, extra_spread, target))
            }).min() {

            harvests.push(target);
            candidates.remove(&target);

            let ants_per_cell = state.total_ants[player] / (initial_spread + extra_spread);
            let source = harvest_mesh.nearest(target, &view.layout);
            let path: Vec<usize> = nearby.calculate_path(source, target, &view.layout, &view.paths).collect();
            if !path.iter().any(|&cell| attacks[cell] > ants_per_cell) { // Only take paths that are not blocked by the enemy
                beacons.extend(path.iter().cloned());
                harvest_mesh.extend(path.iter().cloned(), &view.layout);
            }

        } else {
            break; // no valid countermoves
        }
    }

    Countermoves {
        assignments: movement::spread_ants_across_beacons(beacons.into_iter(), player, view, state),
        harvests,
    }
}