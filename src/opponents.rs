use std::collections::VecDeque;
use std::fmt::Display;

use super::fnv::FnvHashSet;
use super::inputs::*;
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
        let mut is_first = true;
        if self.harvests.is_empty() {
            write!(f, "-")?;
        } else {
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
    let num_cells = view.layout.cells.len();
    let total_ants = state.total_ants[player];

    // Keep ants at existing cells, but only if they are busy - otherwise they will be reassigned
    let evaluator = HarvestEvaluator::new(player, state);
    let spawner = SpawnEvaluator::new(player, view, state);
    let flow_distance_from_base = calculate_flow_distance_from_base(player, view, state);
    let (mut harvests, busy) = identify_busy_ants(player, view, state, &flow_distance_from_base);
    let mut beacons: FnvHashSet<usize> = (0..num_cells).filter(|&cell| busy[cell]).collect();
    let mut beacon_mesh: Option<NearbyPathMap> = None;

    // Extend to collect nearby crystals
    let nearby = NearbyPathMap::near_my_ants(player, view, state);
    let mut countermoves: FnvHashSet<usize> =
        (0..num_cells)
        .filter(|&cell| !busy[cell] && spawner.is_worth_harvesting(cell, view, state, nearby.distance_to(cell)))
        .collect();
    while !countermoves.is_empty() && (beacons.len() as i32) < total_ants {
        let initial_harvests = harvests.len() as i32;
        let initial_spread = beacons.len() as i32;
        let beacon_mesh = beacon_mesh.get_or_insert_with(|| NearbyPathMap::generate(&view.layout, |cell| busy[cell]));

        // Find closest next target. If there is a tie, find the one which will increase our harvest rate the most.
        if let Some((_, extra_spread, target)) =
            countermoves.iter()
            .filter_map(|&target| {
                let extra_spread = beacon_mesh.distance_to(target);
                if initial_spread + extra_spread > total_ants { return None } // Not enough ants to reach this target

                let travel_ticks = nearby.distance_to(target);
                Some((travel_ticks, extra_spread, target))
            }).min() {

            let initial_collection_rate = evaluator.calculate_harvest_rate(initial_harvests, initial_spread);
            let new_collection_rate = evaluator.calculate_harvest_rate(initial_harvests + 1, initial_spread + extra_spread);
            if new_collection_rate <= initial_collection_rate { break } // This target is not worth the effort

            harvests.push(target);
            countermoves.remove(&target);

            let source = beacon_mesh.nearest(target, &view.layout);
            let path: Vec<usize> = nearby.calculate_path(source, target, &view.layout, &view.paths).collect();
            beacons.extend(path.iter().cloned());
            beacon_mesh.extend(path.into_iter(), &view.layout);

        } else {
            break; // no valid countermoves
        }
    }

    Countermoves {
        assignments: movement::spread_ants_across_beacons(beacons.into_iter(), player, view, state),
        harvests,
    }
}

fn identify_busy_ants(player: usize, view: &View, state: &State, flow_distance_from_base: &[i32]) -> (Vec<usize>,Box<[bool]>) {
    let num_cells = view.layout.cells.len();

    let mut busy = Vec::new();
    busy.resize(num_cells, false);

    let mut harvests = Vec::new();
    for cell in 0..num_cells {
        let distance_to_base = flow_distance_from_base[cell];
        if distance_to_base == 0 {
            // this is the base - always mark these as busy so we have starting points to extend from
            busy[cell] = true;
            continue;
        } else if distance_to_base == i32::MAX {
            // this cell is not connected to the base - don't harvest here
            continue;
        }
        if state.resources[cell] <= 0 { continue } // nothing to harvest here

        harvests.push(cell);
        mark_return_path_as_busy(cell, &flow_distance_from_base, &view.layout, &state.num_ants[player], &mut busy);
    }

    (harvests, busy.into_boxed_slice())
}

fn mark_return_path_as_busy(cell: usize, flow_distance_from_base: &[i32], layout: &Layout, num_ants: &AntsPerCell, busy: &mut [bool]) {
    if busy[cell] { return }
    busy[cell] = true;

    let remaining_distance = flow_distance_from_base[cell];
    if remaining_distance <= 0 { return }
    else if remaining_distance == i32::MAX { return } // there won't be a path back to base from here

    // Flow back to base along path with most ants
    let best =
        layout.cells[cell].neighbors.iter()
        .filter(|&&n| flow_distance_from_base[n] < remaining_distance)
        .max_by_key(|&&n| num_ants[n]).cloned();
    if let Some(best) = best {
        mark_return_path_as_busy(best, flow_distance_from_base, layout, num_ants, busy);
    }
}

fn calculate_flow_distance_from_base(player: usize, view: &View, state: &State) -> Box<[i32]> {
    let num_cells = view.layout.cells.len();
    let mut flow_distance_from_base: Vec<i32> = Vec::new();
    flow_distance_from_base.resize(num_cells, i32::MAX);

    let mut queue = VecDeque::new();
    for &base in view.layout.bases[player].iter() {
        flow_distance_from_base[base] = 0;
        queue.push_back(base);
    }

    while let Some(source) = queue.pop_front() {
        let source_distance = flow_distance_from_base[source];

        let neighbor_distance = source_distance + 1;
        for &n in view.layout.cells[source].neighbors.iter() {
            if state.num_ants[player][n] <= 0 { continue }

            if flow_distance_from_base[n] > neighbor_distance {
                flow_distance_from_base[n] = neighbor_distance;
                queue.push_back(n);
            }
        }
    }

    flow_distance_from_base.into_boxed_slice()
}