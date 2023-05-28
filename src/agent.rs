use super::inputs::*;
use super::view::*;
use super::policies::{self,*};

pub fn act(view: &View, state: &State) -> Vec<Action> {
    let plan = Plan::new();
    let actions = policies::enact_plan(ME, &plan, view, state);
    actions
}