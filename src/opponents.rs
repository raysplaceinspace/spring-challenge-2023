use std::collections::VecDeque;
use std::fmt::Display;

use super::fnv::FnvHashSet;
use super::inputs::*;
use super::view::*;
use super::movement::{self,Assignments};
use super::pathing::NearbyPathMap;
use super::valuation::{NumHarvests,HarvestEvaluator,ValueOrd};

pub struct Countermoves {
    pub assignments: Assignments,
    pub targets: Vec<usize>,
}
impl Display for Countermoves {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        if self.targets.is_empty() {
            write!(f, "-")?;
        } else {
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

pub fn enact_countermoves(player: usize, view: &View, state: &State) -> Countermoves {
    // Add the countermove as an extension of existing ants
    let num_cells = view.layout.cells.len();
    let total_ants: i32 = state.num_ants[player].iter().cloned().sum();

    // Keep ants at existing cells, but only if they are busy - otherwise they will be reassigned
    let flow_distance_from_base = calculate_flow_distance_from_base(player, view, state);
    let (mut counts, busy) = identify_busy_ants(view, state, &flow_distance_from_base);
    let mut beacons: FnvHashSet<usize> = (0..num_cells).filter(|&cell| busy[cell]).collect();
    let mut beacon_mesh: Option<NearbyPathMap> = None;

    // Extend to collect nearby crystals
    let evaluator = HarvestEvaluator::new(player, view, state);
    let mut countermoves: FnvHashSet<usize> =
        (0..num_cells)
        .filter(|&cell| !busy[cell] && state.resources[cell] > 0)
        .collect();
    let mut targets = Vec::new();
    let mut nearby: Option<NearbyPathMap> = None;
    while !countermoves.is_empty() && (beacons.len() as i32) < total_ants {
        let initial_spread = beacons.len() as i32;
        let beacon_mesh = beacon_mesh.get_or_insert_with(|| NearbyPathMap::generate(&view.layout, |cell| busy[cell]));
        let (target, distance, new_counts, new_collection_rate) =
            countermoves.iter()
            .map(|&target| {
                let distance = beacon_mesh.distance_to(target);
                let new_counts = counts.clone().add(view.layout.cells[target].content);
                let new_collection_rate = evaluator.calculate_harvest_rate_discounting_eggs(&new_counts, initial_spread + distance);
                (target, distance, new_counts, new_collection_rate)
            })
            .max_by_key(|(_,distance,_,new_collection_rate)| (ValueOrd::new(*new_collection_rate), -*distance))
            .expect("no countermoves");

        let new_spread = initial_spread + distance;
        if new_spread > total_ants { break } // Not enough ants to reach this target, or any others because this is the shortest one

        let initial_collection_rate = evaluator.calculate_harvest_rate_discounting_eggs(&counts, initial_spread);
        if new_collection_rate < initial_collection_rate { break } // This target is not worth the effort

        targets.push(target);
        countermoves.remove(&target);
        counts = new_counts;

        let source = beacon_mesh.nearest(target, &view.layout);
        let nearby = nearby.get_or_insert_with(|| NearbyPathMap::near_my_ants(player, view, state));
        let path: Vec<usize> = nearby.calculate_path(source, target, &view.layout, &view.paths).collect();
        beacons.extend(path.iter().cloned());
        beacon_mesh.extend(path.into_iter(), &view.layout);
    }

    Countermoves {
        assignments: movement::spread_ants_across_beacons(beacons.into_iter(), player, state),
        targets,
    }
}

fn identify_busy_ants(view: &View, state: &State, flow_distance_from_base: &[i32]) -> (NumHarvests,Box<[bool]>) {
    let num_cells = view.layout.cells.len();

    let mut busy = Vec::new();
    busy.resize(num_cells, false);

    let mut counts = NumHarvests::new();
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

        counts = counts.add(view.layout.cells[cell].content);
        mark_return_path_as_busy(cell, &flow_distance_from_base, &view.layout, &mut busy);
    }

    (counts, busy.into_boxed_slice())
}

fn mark_return_path_as_busy(cell: usize, flow_distance_from_base: &[i32], layout: &Layout, busy: &mut [bool]) {
    busy[cell] = true;

    let remaining_distance = flow_distance_from_base[cell];
    if remaining_distance <= 0 { return }
    else if remaining_distance == i32::MAX { return } // there won't be a path back to base from here

    // Flow back to base along the neighbors that are closer to the base
    for &neighbor in layout.cells[cell].neighbors.iter() {
        if !busy[neighbor] && flow_distance_from_base[neighbor] < remaining_distance {
            mark_return_path_as_busy(neighbor, flow_distance_from_base, layout, busy);
        }
    }
}

fn calculate_flow_distance_from_base(player: usize, view: &View, state: &State) -> Box<[i32]> {
    let num_cells = view.layout.cells.len();
    let mut flow_distance_from_base: Vec<i32> = Vec::new();
    flow_distance_from_base.resize(num_cells, i32::MAX);

    let mut queue = VecDeque::new();
    for &base in view.layout.bases[player].iter() {
        if state.num_ants[player][base] <= 0 { continue }
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