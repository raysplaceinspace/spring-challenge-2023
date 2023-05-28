use std::collections::VecDeque;

use super::model::*;
use super::view::*;

pub fn harvest(view: &View, num_ants: &AntsPerCell, available_resources: &ResourcesPerCell, harvested: &mut HarvestedPerPlayer) {
    let mut max_flow_per_player = Vec::new();
    for player in 0..NUM_PLAYERS {
        let mut max_flow = Vec::new();
        max_flow.resize(view.layout.cells.len(), 0);

        for &base in view.layout.bases[player].iter() {
            let flows_to_base = calculate_flows_to_base(base, player, view, num_ants);
            for i in 0..view.layout.cells.len() {
                max_flow[i] = max_flow[i].max(flows_to_base[i]);
            }
        }

        max_flow_per_player.push(max_flow);
    }

    for cell in 0..view.layout.cells.len() {
        if view.layout.cells[cell].content != Some(Content::Crystals) { continue }

        let available_resources = available_resources[cell];
        if available_resources <= 0 { continue }

        let flow = [
            max_flow_per_player[0][cell],
            max_flow_per_player[1][cell],
        ];
        let max_flow: i32 = flow[0] + flow[1];
        if max_flow <= 0 { continue }

        let harvestable = available_resources.min(max_flow);
        let ratio = if harvestable < max_flow { harvestable as f32 / max_flow as f32 } else { 1.0 };
        let harvested0 = (flow[0] as f32 * ratio).round() as i32;
        let harvested1 = harvestable - harvested0;

        harvested[0] += harvested0;
        harvested[1] += harvested1;
    }
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