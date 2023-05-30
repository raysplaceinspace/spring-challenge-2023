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
                if selector < 0.25 {
                    if rng.gen::<f32>() < 0.5 {
                        self.insert_milestone(&mut plan, rng)
                    } else {
                        self.remove_milestone(&mut plan, rng)
                    }
                } else if selector < 0.5 {
                    self.replace_milestone(&mut plan, rng)
                } else {
                    self.swap_milestone(&mut plan, rng)
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
        let milestone = Milestone::new(cell);

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

    fn replace_milestone(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> bool {
        if plan.is_empty() { return false }

        let index = rng.gen_range(0..self.unharvested.len());
        let cell = self.unharvested[index];
        
        let index = rng.gen_range(0..plan.len());
        let milestone = &mut plan[index];
        milestone.cell = cell;

        true
    }

    fn remove_milestone(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> bool {
        if plan.is_empty() { return false; }

        let index = rng.gen_range(0..plan.len());
        plan.remove(index);

        true
    }

    fn swap_milestone(&self, plan: &mut Vec<Milestone>, rng: &mut StdRng) -> bool {
        if plan.len() < 2 { return false; }

        let index = rng.gen_range(0 .. (plan.len() - 1));
        plan.swap(index, index + 1);

        true
    }
}