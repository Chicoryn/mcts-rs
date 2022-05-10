use crossbeam_epoch::Guard;
use std::{rc::Rc, marker::PhantomData};

use crate::{
    process::{PerChild, Process},
    safe_nonnull::SafeNonNull,
    node::Node,
    Mcts,
};

pub struct Step<'a, P: Process> {
    pub(super) pin: Rc<Guard>,
    pub(super) ptr: SafeNonNull<Node<P>>,
    pub(super) key: <P::PerChild as PerChild>::Key,
    search_tree: PhantomData<&'a Mcts<P>>
}

impl<'a, P: Process> Step<'a, P> {
    pub(super) fn new(_: &'a Mcts<P>, pin: Rc<Guard>, ptr: SafeNonNull<Node<P>>, key: <P::PerChild as PerChild>::Key) -> Self {
        let search_tree = PhantomData::default();

        Self { search_tree, pin, ptr, key }
    }

    pub(super) fn pin(&self) -> &Guard {
        self.pin.as_ref()
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
