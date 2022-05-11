use crate::{node::Node, process::Process, safe_nonnull::SafeNonNull, step::Step};
use crossbeam_epoch as epoch;
use std::rc::Rc;

pub struct PathIter<'a, P: Process> {
    process: &'a P,
    pin: Rc<epoch::Guard>,
    current: Option<SafeNonNull<Node<P>>>
}

impl<'a, P: Process> PathIter<'a, P> {
    pub(super) fn new(process: &'a P, starting_point: SafeNonNull<Node<P>>) -> Self {
        let current = Some(starting_point);
        let pin = Rc::new(epoch::pin());

        Self { process, pin, current }
    }

    pub(super) fn pin(&self) -> &epoch::Guard {
        self.pin.as_ref()
    }
}

impl<'a, P: Process> Iterator for PathIter<'a, P> {
    type Item = Step<'a, P>;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.current;

        if let Some((key, edge)) = curr.and_then(|node| node.best(self.pin(), self.process)) {
            self.current = edge.ptr();

            Some(Step::new(self.process, self.pin.clone(), curr.unwrap(), key))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{FakePerChild, FakeProcess, FakeState};
    use super::*;

    #[test]
    fn next_gets_full_path() {
        let pin = unsafe { epoch::unprotected() };
        let root = SafeNonNull::new(Node::new(FakeState::new()));
        root.try_expand(&pin, FakePerChild::new(1));
        let process = FakeProcess::new(1, 0);
        let mut iter = PathIter::new(&process, root);

        assert_eq!(iter.next().map(|step| step.key()), Some(1));
        assert!(iter.next().is_none());
    }
}
