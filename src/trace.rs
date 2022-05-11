use crate::{node::Node, process::{PerChild, Process}, safe_nonnull::SafeNonNull, step::Step};
use crossbeam_epoch::Guard;
use std::rc::Rc;

pub struct Trace<'a, P: Process> {
    steps: Vec<Step<'a, P>>,
}

impl<'a, P: Process> Trace<'a, P> {
    /// Returns a new trace with the given `steps` and `status`.
    pub fn new() -> Self {
        Self { steps: vec! [] }
    }

    pub(super) fn push(&mut self, process: &'a P, pin: Rc<Guard>, ptr: SafeNonNull<Node<P>>, key: <P::PerChild as PerChild>::Key) {
        self.steps.push(Step::new(process, pin, ptr, key));
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
