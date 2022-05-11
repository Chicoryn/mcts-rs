use crate::{node::Node, process::{PerChild, Process}, safe_nonnull::SafeNonNull};
use crossbeam_epoch::Guard;
use std::{rc::Rc, marker::PhantomData};

pub struct Step<'a, P: Process, Node> {
    pin: Rc<Guard>,
    ptr: SafeNonNull<Node>,
    key: <P::PerChild as PerChild>::Key,
    process: PhantomData<&'a P>
}

impl<'a, P: Process, Node> Step<'a, P, Node> {
    pub(super) fn new(_: &'a P, pin: Rc<Guard>, ptr: SafeNonNull<Node>, key: <P::PerChild as PerChild>::Key) -> Self {
        let process = PhantomData::default();

        Self { process, pin, ptr, key }
    }

    pub(super) fn pin(&self) -> &Guard {
        self.pin.as_ref()
    }

    pub(super) fn ptr(&self) -> &Node {
        &*self.ptr
    }

    /// Returns the key that is associated with this step.
    pub fn key(&self) -> <P::PerChild as PerChild>::Key {
        self.key
    }
}

impl<'a, P: Process> Step<'a, P, Node<P>> {
    /// Returns the result of calling the given `mapper` on this steps `state`
    /// and `per_child`.
    ///
    /// # Arguments
    ///
    /// * `f` - the mapper for this steps `state` and `per_child`
    ///
    pub fn map<T>(&self, f: impl FnOnce(&P::State, &P::PerChild) -> T) -> T {
        self.ptr.map(self.pin(), self.key, |state, _, per_child| f(state, per_child))
    }
}

#[cfg(test)]
mod tests {
    use crate::FakeProcess;
    use crossbeam_epoch as epoch;
    use super::*;

    #[test]
    fn new_sets_key_and_ptr() {
        let process = FakeProcess::new(0, 0);
        let pin = epoch::pin();
        let step = Step::new(&process, Rc::new(pin), SafeNonNull::new(()), 1);

        assert_eq!(step.ptr(), &());
        assert_eq!(step.key(), 1);
    }
}
