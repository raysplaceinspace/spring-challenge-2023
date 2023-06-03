use std::collections::VecDeque;

use super::inputs::*;
use super::view::*;

pub struct HarvestMap {
    max_flow_per_player: [Box<[i32]>; NUM_PLAYERS],
}
impl HarvestMap {
    pub fn generate(view: &View, num_ants: &AntsPerCellPerPlayer) -> Self {
        let max_flow_per_player = [
            calculate_max_flow_for_player(ME, view, num_ants),
            calculate_max_flow_for_player(ENEMY, view, num_ants),
        ];

        let max_flow_per_player = match calculate_unhindered_ants(&max_flow_per_player, num_ants) {
            Some(num_unhindered_ants) => [
                calculate_max_flow_for_player(ME, view, &num_unhindered_ants),
                calculate_max_flow_for_player(ENEMY, view, &num_unhindered_ants),
            ],
            None => max_flow_per_player, // unchanged
        };

        Self { max_flow_per_player }
    }

    pub fn calculate_harvest_at(&self, player: usize, cell: usize, available: i32) -> i32 {
        if available <= 0 { return 0 }

        let demand = self.max_flow_per_player[player][cell];
        let harvest = demand.min(available);
        harvest
    }
}

pub fn calculate_max_flow_for_player(player: usize, view: &View, num_ants: &AntsPerCellPerPlayer) -> Box<[i32]> {
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

fn calculate_unhindered_ants(attack_chains_per_player: &[Box<[i32]>; NUM_PLAYERS], num_ants: &AntsPerCellPerPlayer) -> Option<AntsPerCellPerPlayer> {
    let mut unhindered = num_ants.clone();
    let num_cells = attack_chains_per_player[ME].len();

    let mut changed = false;
    for cell in 0..num_cells {
        let my_attack_chain = attack_chains_per_player[ME][cell];
        let enemy_attack_chain = attack_chains_per_player[ENEMY][cell];

        if my_attack_chain > enemy_attack_chain {
            unhindered[ENEMY][cell] = 0;
            changed = true;
        } else if enemy_attack_chain > my_attack_chain {
            unhindered[ME][cell] = 0;
            changed = true;
        }
    }

    if changed {
        Some(unhindered)
    } else {
        None
    }
}