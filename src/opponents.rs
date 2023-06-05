use std::collections::VecDeque;
use std::fmt::Display;

use super::inputs::{Content,Layout};
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
    // Add the countermove as an extension of existing ants
    let total_ants = state.total_ants[player];

    // Keep ants at existing cells, but only if they are busy - otherwise they will be reassigned
    let (mut harvests, mut harvest_mesh) = identify_existing_harvests(player, view, state);

    // Extend to collect nearby crystals
    let evaluator = HarvestEvaluator::new(player, state);
    let spawner = SpawnEvaluator::new(player, view, state);
    let mut beacons: FnvHashSet<usize> = FnvHashSet::default();
    let nearby = NearbyPathMap::near_my_ants(player, view, state);
    let mut countermoves: FnvHashSet<usize> =
        view.closest_resources[player].iter().cloned()
        .filter(|&cell| spawner.is_worth_harvesting(cell, view, state, nearby.distance_to(cell)))
        .collect();
    while !countermoves.is_empty() && (beacons.len() as i32) < total_ants {
        let initial_harvests = harvests.len() as i32;
        let initial_spread = beacons.len() as i32;
        let initial_collection_rate = evaluator.calculate_harvest_rate(initial_harvests, initial_spread);

        // Find closest next target
        if let Some((_, _, target)) =
            countermoves.iter()
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
            countermoves.remove(&target);

            let source = harvest_mesh.nearest(target, &view.layout);
            let path: Vec<usize> = nearby.calculate_path(source, target, &view.layout, &view.paths).collect();
            beacons.extend(path.iter().cloned());
            harvest_mesh.extend(path.iter().cloned(), &view.layout);

        } else {
            break; // no valid countermoves
        }
    }

    Countermoves {
        assignments: movement::spread_ants_across_beacons(beacons.into_iter(), player, view, state),
        harvests,
    }
}

fn identify_existing_harvests(player: usize, view: &View, state: &State) -> (Vec<usize>, NearbyPathMap) {
    let num_cells = view.layout.cells.len();

    let mut busy = Vec::new();
    busy.resize(num_cells, false);
    for &base in view.layout.bases[player].iter() {
        busy[base] = true; // bases always begin as busy
    }

    let mut harvests = Vec::new();
    let harvest_distances = calculate_harvest_chain_lengths(player, view, state);
    for &candidate in view.closest_resources[player].iter() {
        if harvest_distances[candidate] == i32::MAX { continue } // Only consider cells that are connected to the base through a harvest chain

        harvests.push(candidate);
        mark_return_path_as_busy(candidate, &harvest_distances, &view.layout, &state.num_ants[player], &mut busy);
    }

    let harvest_mesh = NearbyPathMap::generate(
        &view.layout,
        (0..num_cells).filter(|&cell| busy[cell]),
    );

    (harvests, harvest_mesh)
}

fn calculate_harvest_chain_lengths(player: usize, view: &View, state: &State) -> Box<[i32]> {
    let num_cells = view.layout.cells.len();

    let mut distances = Vec::new();
    distances.resize(num_cells, i32::MAX);

    let mut queue = VecDeque::new();
    for &base in view.layout.bases[player].iter() {
        distances[base] = 0;
        queue.push_back(base);
    }

    while let Some(source) = queue.pop_front() {
        let source_distance = distances[source];
        let neighbor_distance = source_distance + 1;
        for &n in view.layout.cells[source].neighbors.iter() {
            if state.num_ants[player][n] <= 0 { continue; } // Chain only flows along cells which contain ants

            if distances[n] > neighbor_distance {
                distances[n] = neighbor_distance;
                queue.push_back(n);
            }
        }
    }

    distances.into_boxed_slice()
}

fn mark_return_path_as_busy(harvest: usize, harvest_distances: &[i32], layout: &Layout, num_ants: &AntsPerCell, busy: &mut [bool]) {
    let mut current = harvest;
    loop {
        if busy[current] { break } // Already processed the return path from this cell

        busy[current] = true;
        let distance = harvest_distances[current];
        if distance <= 0 { break }

        current =
            layout.cells[current].neighbors.iter().cloned()
            .filter(|&n| harvest_distances[n] < distance)
            .max_by_key(|&n| num_ants[n])
            .expect("missing return path");
    }
}