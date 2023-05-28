use std::collections::{HashMap, VecDeque};
use super::inputs::*;

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

    pub fn calculate_path<'a>(&'a self, source: usize, sink: usize, layout: &'a Layout) -> impl Iterator<Item=usize> + 'a {
        let mut next = Some(source);
        std::iter::from_fn(move || {
            let output = next;
            if let Some(current) = next {
                if current == sink {
                    next = None;
                } else {
                    next = self.step_towards(current, sink, layout);
                }
            }
            output
        })
    }
}

pub struct DistanceMap {
    distances: Box<[i32]>,
}
impl DistanceMap {
    pub fn generate(source: usize, layout: &Layout) -> Self {
        let mut lookup = HashMap::new();
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