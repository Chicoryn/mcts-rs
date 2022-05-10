use crossbeam_epoch::Guard;
use std::rc::Rc;

use crate::{
    process::{PerChild, Process},
    step::Step, Mcts,
};

pub struct Trace<'a, P: Process> {
    steps: Vec<Step<'a, P>>,
}

impl<'a, P: Process> Trace<'a, P> {
    /// Returns a new trace with the given `steps` and `status`.
    pub fn new() -> Self {
        Self { steps: vec! [] }
    }

    pub(super) fn push(&mut self, search_tree: &'a Mcts<P>, pin: Rc<Guard>, ptr: usize, key: <P::PerChild as PerChild>::Key) {
        self.steps.push(Step::new(search_tree, pin, ptr, key));
    }

    /// Returns if there are no steps in this trace.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Returns the steps in this trace.
    pub fn steps(&self) -> &[Step<P>] {
        &self.steps
    }
}
