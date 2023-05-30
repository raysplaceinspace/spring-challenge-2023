use core::panic;
use std::collections::HashSet;
use std::fmt::Display;

use super::inputs::*;
use super::view::*;

#[derive(Clone)]
pub struct Milestone {
    pub cell: usize,
}
impl Milestone {
    pub fn new(cell: usize) -> Self {
        Self { cell }
    }

    pub fn is_complete(&self, state: &State) -> bool {
        state.resources[self.cell] <= 0
    }
}
impl Display for Milestone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cell.fmt(f)
    }
}

pub fn enact_plan(player: usize, plan: &[Milestone], view: &View, state: &State) -> (Vec<Action>,PlanDetail) {
    let mut actions = Vec::new();

    let total_ants: i32 = state.num_ants[player].iter().cloned().sum();

    let mut targets = Vec::new();
    let mut beacons = HashSet::new();
    for &base in view.layout.bases[player].iter() {
        beacons.insert(base);
    }

    for milestone in plan.iter().skip_while(|m| m.is_complete(&state)) {
        let initial_distance = beacons.len() as i32;
        let initial_harvests = targets.len() as i32;
        let initial_collection_rate = calculate_collection_rate(total_ants, initial_distance, initial_harvests);

        let target = milestone.cell;

        if let Some((distance, source)) = beacons.iter().map(|&source| (view.paths.distance_between(source, target),source)).min() {
            let new_collection_rate = calculate_collection_rate(total_ants, initial_distance + distance, initial_harvests + 1);
            // eprintln!("considered harvesting <{}> (distance {}): {} -> {}", target, distance, initial_collection_rate, new_collection_rate);

            if new_collection_rate > initial_collection_rate {
                for cell in view.paths.calculate_path(source, target, &view.layout) {
                    beacons.insert(cell);
                }
                targets.push(target);

            } else {
                // Best harvest not worth it, so none others will be either
                break;
            }

        } else {
            panic!("no sources available for harvest");
        }
    }

    for beacon in beacons {
        actions.push(Action::Beacon { index: beacon, strength: 1 });
    }

    actions.push(Action::Message { text: format_harvest_msg(targets.as_slice()) });

    let detail = PlanDetail { targets };
    (actions, detail)
}

pub struct PlanDetail {
    pub targets: Vec<usize>,
}
impl Display for PlanDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.targets.is_empty() {
            write!(f, "-")?;
        } else {
            let mut is_first = true;
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

fn format_harvest_msg(targets: &[usize]) -> String {
    use std::fmt::Write;

    let mut msg = String::new();
    for &target in targets {
        if !msg.is_empty() {
            msg.push_str(" ");
        }
        write!(&mut msg, "{}", target).ok();
    }

    if msg.is_empty() {
        msg.push_str("-");
    }

    msg
}

fn calculate_collection_rate(total_ants: i32, total_distance: i32, num_harvests: i32) -> i32 {
    if total_distance <= 0 { return 0 }
    let per_cell = total_ants / total_distance; // intentional integer division since ants can't be split
    num_harvests * per_cell
}