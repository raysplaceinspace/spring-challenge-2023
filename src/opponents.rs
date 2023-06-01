use std::collections::VecDeque;
use std::fmt::Display;

use super::fnv::{FnvHashMap,FnvHashSet};
use super::inputs::*;
use super::view::*;
use super::movement::{self,Assignments};
use super::pathing::NearbyPathMap;
use super::valuation::{ValuationCalculator,NumHarvests};

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

#[derive(Clone,Copy)]
struct Link {
    pub source: usize,
    pub distance: i32,
}
pub fn enact_countermoves(player: usize, view: &View, state: &State) -> Countermoves {
    // Add the countermove as an extension of existing ants
    let num_cells = view.layout.cells.len();
    let total_ants: i32 = state.num_ants[player].iter().cloned().sum();
    let mut beacons = FnvHashSet::default();

    // Always begin the calculation with ants on the bases so we have somewhere to extend from
    for &base in view.layout.bases[player].iter() {
        beacons.insert(base);
    }

    // Keep ants at existing cells, but only if they are busy - otherwise they will be reassigned
    let flow_distance_from_base = calculate_flow_distance_from_base(player, view, state);
    let (mut counts, busy) = identify_busy_ants(view, state, &flow_distance_from_base);
    for cell in 0..num_cells {
        if busy[cell] {
            beacons.insert(cell);
        }
    }

    // Extend to collect nearby crystals
    let evaluator = ValuationCalculator::new(player, view, state);
    let mut countermoves: FnvHashMap<usize,Link> =
        (0..num_cells)
        .filter(|&cell| view.layout.cells[cell].content == Some(Content::Crystals) && state.resources[cell] > 0 && state.num_ants[player][cell] <= 0)
        .map(|target| {
            let closest = beacons.iter().map(|&source| {
                Link { source, distance: view.paths.distance_between(source, target) }
            }).min_by_key(|countermove| countermove.distance).expect("no beacons");
            (target,closest)
        }).collect();
    let mut targets = Vec::new();
    let mut nearby: Option<NearbyPathMap> = None;
    while !countermoves.is_empty() && (beacons.len() as i32) < total_ants {
        let (&target, &Link { source, distance }) =
            countermoves.iter()
            .min_by_key(|(_,x)| x.distance)
            .expect("no countermoves");

        let initial_distance = beacons.len() as i32;
        let new_distance = initial_distance + distance;
        if new_distance > total_ants { break } // Not enough ants to reach this target, or any others because this is the shortest one

        let initial_collection_rate = evaluator.calculate(&counts, initial_distance);
        let new_counts = counts.clone().add(view.layout.cells[target].content);
        let new_collection_rate = evaluator.calculate(&new_counts, new_distance);
        if new_collection_rate < initial_collection_rate { break } // This target is not worth the effort

        targets.push(target);
        countermoves.remove(&target);
        counts = new_counts;

        let nearby = nearby.get_or_insert_with(|| NearbyPathMap::generate(player, view, state));
        for beacon in nearby.calculate_path(source, target, &view.layout, &view.paths) {
            beacons.insert(beacon);

            // New beacons may reduce the distance to the countermoves
            for (&target, link) in countermoves.iter_mut() {
                let distance = view.paths.distance_between(beacon, target);
                if distance < link.distance {
                    *link = Link { source: beacon, distance };
                }
            }
        }
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
        if state.resources[cell] <= 0 { continue } // nothing to harvest here
        if flow_distance_from_base[cell] == i32::MAX { continue } // not harvesting this cell

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
        for &neighbor in view.layout.cells[source].neighbors.iter() {
            if state.num_ants[player][neighbor] <= 0 { continue }
            if flow_distance_from_base[neighbor] <= neighbor_distance { continue }

            flow_distance_from_base[neighbor] = neighbor_distance;
            queue.push_back(neighbor);
        }
    }

    flow_distance_from_base.into_boxed_slice()
}