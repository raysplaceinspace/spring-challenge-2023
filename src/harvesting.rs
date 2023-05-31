use std::collections::VecDeque;
use super::inputs::Layout;
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
    calculate_flows_to_base(view.layout.bases[player].iter().cloned(), &view.layout, &num_ants[player])
}

fn calculate_flows_to_base(bases: impl Iterator<Item=usize>, layout: &Layout, num_ants: &AntsPerCell) -> Box<[i32]> {
    let mut flows = Vec::new();
    flows.resize(layout.cells.len(), 0);

    let mut queue = VecDeque::new();
    for base in bases {
        flows[base] = num_ants[base];
        queue.push_back(base);
    }

    while let Some(source) = queue.pop_front() {
        let source_flow = flows[source];
        if source_flow < 0 { continue }

        for &neighbor in layout.cells[source].neighbors.iter() {
            let neighbor_ants = num_ants[neighbor];
            if neighbor_ants <= 0 { continue }

            let neighbor_flow = neighbor_ants.min(source_flow);

            if flows[neighbor] < neighbor_flow {
                flows[neighbor] = neighbor_flow;
                queue.push_back(neighbor);
            }
        }
    }

    flows.into_boxed_slice()
}