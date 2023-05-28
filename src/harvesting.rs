use std::collections::VecDeque;
use super::view::*;

pub struct HarvestMap {
    max_flow: Box<[i32]>,
}
impl HarvestMap {
    pub fn generate(player: usize, view: &View, num_ants: &AntsPerCellPerPlayer) -> Self {
        Self {
            max_flow: calculate_max_flow_for_player(player, view, num_ants),
        }
    }

    pub fn calculate_harvest_at(&self, cell: usize, available: i32) -> i32 {
        if available <= 0 { return 0 }

        let demand = self.max_flow[cell];
        let harvest = demand.min(available);
        harvest
    }
}

fn calculate_max_flow_for_player(player: usize, view: &View, num_ants: &AntsPerCellPerPlayer) -> Box<[i32]> {
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

fn calculate_flows_to_base(base: usize, player: usize, view: &View, num_ants: &AntsPerCellPerPlayer) -> Vec<i32> {
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