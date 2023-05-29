use std::collections::HashSet;

use super::plans::Milestone;
use super::view::*;
use rand::prelude::*;

pub struct Mutator {
    unharvested: Vec<usize>,
}
impl Mutator {
    pub fn new(view: &View, state: &State) -> Option<Self> {
        let unharvested: Vec<usize> = (0..view.layout.cells.len()).filter(|i| state.resources[*i] > 0).collect();
        if unharvested.is_empty() { return None }
        Some(Self {
            unharvested,
        })
    }

    pub fn mutate(&self, plan: &Vec<Milestone>, rng: &mut StdRng) -> Option<Vec<Milestone>> {
        let mut plan = plan.clone();

        let modified =
            if plan.is_empty() {
                self.insert_milestone(&mut plan, rng)
            } else {
                let selector: f32 = rng.gen();
                if selector < 0.5 {
                    if rng.gen::<f32>() < 0.75 {
                        self.insert_milestone(&mut plan, rng)
                    } else {
                        self.remove_milestone(&mut plan, rng)
                    }
                } else if selector < 0.75 {
                    if rng.gen::<f32>() < 0.75 {
                        self.extend_milestone(&mut plan, rng)
                    } else {
                        self.reduce_milestone(&mut plan, rng)
                    }
                } else {
                    if rng.gen::<f32>() < 0.75 {
                        self.increase_milestone(&mut plan, rng)
                    } else {
                        self.decrease_milestone(&mut plan, rng)
                    }
                }
            };

        if modified {
            Some(plan)
        } else {
            None
        }
    }

    fn insert_milestone(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> bool {
        if self.unharvested.is_empty() { return false; }

        let index = rng.gen_range(0..self.unharvested.len());
        let cell = self.unharvested[index];
        let milestone = Milestone::new(vec![cell], 0);

        if plan.is_empty() {
            plan.push(milestone);
        } else {
            let index = rng.gen_range(0..(plan.len()+1));
            if index < plan.len() {
                plan.insert(index, milestone);
            } else {
                plan.push(milestone);
            }
        }

        true
    }

    fn extend_milestone(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> bool {
        if plan.is_empty() { return false }

        let index = rng.gen_range(0..plan.len());
        let milestone = &mut plan[index];
        let mut remaining: HashSet<usize> = self.unharvested.iter().cloned().collect();
        for &cell in milestone.cells.iter() {
            remaining.remove(&cell);
        }

        if remaining.is_empty() { return false }

        let index = rng.gen_range(0..remaining.len());
        milestone.cells.push(remaining.into_iter().nth(index).expect("index out of bounds"));

        true
    }

    fn reduce_milestone(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> bool {
        if plan.is_empty() { return false }

        let index = rng.gen_range(0..plan.len());
        let milestone = &mut plan[index];
        if milestone.cells.is_empty() { return false }

        let index = rng.gen_range(0..milestone.cells.len());
        milestone.cells.swap_remove(index);

        true
    }

    fn increase_milestone(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> bool {
        if plan.is_empty() { return false }

        let index = rng.gen_range(0..plan.len());
        let milestone = &mut plan[index];
        let increase = milestone.num_cells_to_leave + 1;
        if increase as usize >= milestone.cells.len() { return false }
        milestone.num_cells_to_leave = increase;

        true
    }

    fn decrease_milestone(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> bool {
        if plan.is_empty() { return false }

        let index = rng.gen_range(0..plan.len());
        let milestone = &mut plan[index];
        let decreased = milestone.num_cells_to_leave - 1;
        if decreased < 0 { return false }
        milestone.num_cells_to_leave = decreased;

        true
    }

    fn remove_milestone(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> bool {
        if plan.is_empty() { return false; }

        let index = rng.gen_range(0..plan.len());
        plan.remove(index);

        true
    }
}