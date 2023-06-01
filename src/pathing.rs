use std::collections::VecDeque;
use super::fnv::FnvHashMap;

use super::inputs::*;
use super::view::*;

/// This creates an additional layer on top of PathMap where if there is a tie between two possible paths,
/// the path closer to the existing ants is chosen.
pub struct NearbyPathMap {
    distance_to_nearest: Box<[i32]>,
}
impl NearbyPathMap {
    pub fn generate(layout: &Layout, is_present: impl Fn(usize) -> bool) -> Self {
        let num_cells = layout.cells.len();
        let mut distance_to_nearest = Vec::new();
        distance_to_nearest.resize(num_cells, i32::MAX);

        let mut queue = VecDeque::new();
        for cell in 0..num_cells {
            if is_present(cell) {
                distance_to_nearest[cell] = 0;
                queue.push_back(cell);
            }
        }

        while let Some(current) = queue.pop_front() {
            let neighbor_distance = distance_to_nearest[current] + 1;
            for &n in layout.cells[current].neighbors.iter() {
                if neighbor_distance < distance_to_nearest[n] {
                    distance_to_nearest[n] = neighbor_distance;
                    queue.push_back(n);
                }
            }
        }

        Self {
            distance_to_nearest: distance_to_nearest.into_boxed_slice(),
        }

    }

    pub fn near_my_ants(player: usize, view: &View, state: &State) -> Self {
        Self::generate(&view.layout, |cell| state.num_ants[player][cell] > 0)
    }

    pub fn insert(&mut self, cell: usize, layout: &Layout) {
        let mut queue = VecDeque::new();
        self.distance_to_nearest[cell] = 0;
        queue.push_back(cell);

        while let Some(current) = queue.pop_front() {
            let neighbor_distance = self.distance_to_nearest[current] + 1;
            for &n in layout.cells[current].neighbors.iter() {
                if neighbor_distance < self.distance_to_nearest[n] {
                    self.distance_to_nearest[n] = neighbor_distance;
                    queue.push_back(n);
                }
            }
        }
    }

    pub fn nearest(&self, target: usize, layout: &Layout) -> usize {
        let mut current = target;
        loop {
            let distance = self.distance_to_nearest[current];
            if distance <= 0 { return current }
            current = layout.cells[current].neighbors.iter().min_by_key(|&&n| self.distance_to_nearest[n]).cloned().expect("missing neighbors");
        }
    }

    pub fn distance_to(&self, cell: usize) -> i32 {
        self.distance_to_nearest[cell]
    }

    pub fn step_towards(&self, source: usize, sink: usize, layout: &Layout, paths: &PathMap) -> Option<usize> {
        let distances_to_sink = &paths.sources[sink].distances; // The distance map is symmetrical, so can use the sink as a source
        let best = layout.cells[source].neighbors.iter().min_by_key(|&&n| {
            (distances_to_sink[n], self.distance_to_nearest[n])
        }).cloned();
        best
    }

    pub fn calculate_path<'a>(&'a self, source: usize, sink: usize, layout: &'a Layout, paths: &'a PathMap) -> impl Iterator<Item=usize> + 'a {
        let mut next = Some(source);
        std::iter::from_fn(move || {
            let output = next;
            if let Some(current) = next {
                if current == sink {
                    next = None;
                } else {
                    next = self.step_towards(current, sink, layout, paths);
                }
            }
            output
        })
    }
}

pub struct PathMap {
    sources: Box<[DistanceMap]>,
}
impl PathMap {
    pub fn generate(layout: &Layout) -> Self {
        let sources: Vec<DistanceMap> = (0..layout.cells.len()).map(|i| DistanceMap::generate(i, &layout)).collect();
        Self {
            sources: sources.into_boxed_slice(),
        }
    }

    pub fn distance_between(&self, source: usize, sink: usize) -> i32 {
        self.sources[source].distance_to(sink)
    }

    pub fn step_towards(&self, source: usize, sink: usize, layout: &Layout) -> Option<usize> {
        let distances_to_sink = &self.sources[sink].distances; // The distance map is symmetrical, so can use the sink as a source
        let best = layout.cells[source].neighbors.iter().min_by_key(|n| distances_to_sink[**n]).cloned();
        best
    }
}

pub struct DistanceMap {
    distances: Box<[i32]>,
}
impl DistanceMap {
    pub fn generate(source: usize, layout: &Layout) -> Self {
        let mut lookup = FnvHashMap::default();
        lookup.insert(source, 0);

        let mut queue = VecDeque::new();
        queue.push_back(source);

        while let Some(cell) = queue.pop_front() {
            let neighbor_distance = lookup[&cell] + 1;
            for &neighbor in layout.cells[cell].neighbors.iter() {
                if let Some(distance) = lookup.get_mut(&neighbor) {
                    if neighbor_distance < *distance {
                        *distance = neighbor_distance;
                        queue.push_back(neighbor);
                    }
                } else {
                    lookup.insert(neighbor, neighbor_distance);
                    queue.push_back(neighbor);
                }
            }
        }

        let mut distances = Vec::new();
        distances.resize(layout.cells.len(), i32::MAX);
        for (&index, &distance) in lookup.iter() {
            distances[index] = distance;
        }

        Self {
            distances: distances.into_boxed_slice(),
        }
    }

    pub fn distance_to(&self, index: usize) -> i32 { self.distances[index] }
}