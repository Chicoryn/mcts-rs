use crate::{node::Node, process::{PerChild, Process}, safe_nonnull::SafeNonNull};
use crossbeam_epoch::Guard;
use std::{rc::Rc, marker::PhantomData};

pub struct Step<'a, P: Process> {
    pin: Rc<Guard>,
    ptr: SafeNonNull<Node<P>>,
    key: <P::PerChild as PerChild>::Key,
    process: PhantomData<&'a P>
}

impl<'a, P: Process> Step<'a, P> {
    pub(super) fn new(_: &'a P, pin: Rc<Guard>, ptr: SafeNonNull<Node<P>>, key: <P::PerChild as PerChild>::Key) -> Self {
        let process = PhantomData::default();

        Self { process, pin, ptr, key }
    }

    pub(super) fn pin(&self) -> &Guard {
        self.pin.as_ref()
    }

    pub(super) fn ptr(&self) -> &Node<P> {
        &*self.ptr
    }

    /// Returns the key that is associated with this step.
    pub fn key(&self) -> <P::PerChild as PerChild>::Key {
        self.key
    }

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
