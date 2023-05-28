use std::collections::VecDeque;

use super::model::*;
use super::view::*;

pub fn harvest(view: &View, num_ants: &AntsPerCell, available_resources: &ResourcesPerCell, harvested: &mut HarvestedPerPlayer) {
    for player in 0..NUM_PLAYERS {
        let harvest_map = HarvestMap::generate(player, view, num_ants);

        for cell in 0..view.layout.cells.len() {
            let harvest = harvest_map.calculate_harvest_at(cell, available_resources, view);
            if harvest > 0 {
                eprintln!(
                    "{} harvested {} crystals from {}",
                    if player == 0 { "We" } else { "Enemy" },
                    harvest, cell);

                harvested[player] += harvest;
            }
        }
    }
}

pub struct HarvestMap {
    max_flow: Box<[i32]>,
}
impl HarvestMap {
    pub fn generate(player: usize, view: &View, num_ants: &AntsPerCell) -> Self {
        Self {
            max_flow: calculate_max_flow_for_player(player, view, num_ants),
        }
    }

    pub fn calculate_harvest_at(&self, cell: usize, available_resources: &ResourcesPerCell, view: &View) -> i32 {
        if view.layout.cells[cell].content != Some(Content::Crystals) { return 0 }

        let available_resources = available_resources[cell];
        if available_resources <= 0 { return 0 }

        let demand = self.max_flow[cell];
        let harvest = demand.min(available_resources);
        harvest
    }
}

fn calculate_max_flow_for_player(player: usize, view: &View, num_ants: &AntsPerCell) -> Box<[i32]> {
    let mut max_flow = Vec::new();
    max_flow.resize(view.layout.cells.len(), 0);

    for &base in view.layout.bases[player].iter() {
        let flows_to_base = calculate_flows_to_base(base, player, view, num_ants);
        for i in 0..view.layout.cells.len() {
            max_flow[i] = max_flow[i].max(flows_to_base[i]);
        }
    }

    max_flow.into_boxed_slice()
}

fn calculate_flows_to_base(base: usize, player: usize, view: &View, num_ants: &AntsPerCell) -> Vec<i32> {
    let mut flows = Vec::new();
    flows.resize(view.layout.cells.len(), 0);

    let num_base_ants = num_ants[player][base];
    flows[base] = num_base_ants;

    let mut queue = VecDeque::new();
    queue.push_back(base);

    while let Some(source) = queue.pop_front() {
        let source_flow = flows[source];
        if source_flow < 0 { continue }

        for &neighbor in view.layout.cells[source].neighbors.iter() {
            let neighbor_ants = num_ants[player][neighbor];
            if neighbor_ants <= 0 { continue }

            let neighbor_flow = neighbor_ants.min(source_flow);

            if flows[neighbor] < neighbor_flow {
                flows[neighbor] = neighbor_flow;
                queue.push_back(neighbor);
            }
        }
    }

    flows
}