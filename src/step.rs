use crossbeam_epoch::Guard;
use std::rc::Rc;

use crate::{
    process::{PerChild, Process},
    Mcts,
};

pub struct Step<'a, P: Process> {
    pub(super) search_tree: &'a Mcts<P>,
    pub(super) pin: Rc<Guard>,
    pub(super) ptr: usize,
    pub(super) key: <P::PerChild as PerChild>::Key
}

impl<'a, P: Process> Step<'a, P> {
    pub(super) fn new(search_tree: &'a Mcts<P>, pin: Rc<Guard>, ptr: usize, key: <P::PerChild as PerChild>::Key) -> Self {
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
        let search_tree = self.search_tree;
        let nodes = search_tree.nodes.read();

        nodes[self.ptr].map(self.pin(), self.key, |state, _, per_child| f(state, per_child))
    }
}
