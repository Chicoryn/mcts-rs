use crate::{process::{PerChild, Process}, safe_nonnull::SafeNonNull, step::Step};
use crossbeam_epoch::Guard;
use std::rc::Rc;

pub struct Trace<'a, P: Process, Node> {
    steps: Vec<Step<'a, P, Node>>,
}

impl<'a, P: Process, Node> Trace<'a, P, Node> {
    /// Returns a new trace with the given `steps` and `status`.
    pub fn new() -> Self {
        Self { steps: vec! [] }
    }

    pub(super) fn push(&mut self, process: &'a P, pin: Rc<Guard>, ptr: SafeNonNull<Node>, key: <P::PerChild as PerChild>::Key) {
        self.steps.push(Step::new(process, pin, ptr, key));
    }

    /// Returns if there are no steps in this trace.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Returns the steps in this trace.
    pub fn steps(&self) -> &[Step<P, Node>] {
        &self.steps
    }
}

#[cfg(test)]
mod tests {
    use crate::FakeProcess;
    use crossbeam_epoch as epoch;
    use super::*;

    #[test]
    fn new_is_empty() {
        assert!(Trace::<FakeProcess, ()>::new().is_empty());
    }

    #[test]
    fn push_adds_one_step() {
        let process = FakeProcess::new(0, 0);
        let mut trace = Trace::<FakeProcess, ()>::new();
        trace.push(&process, Rc::new(epoch::pin()), SafeNonNull::new(()), 0);

        assert!(!trace.is_empty());
        assert_eq!(trace.steps().len(), 1);
    }
}
