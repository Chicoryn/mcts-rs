use crate::{
    process::Process,
    mcts::Mcts,
};

pub struct Step<'a, P: Process> {
    pub(super) search_tree: &'a Mcts<P>,
    pub(super) ptr: usize,
    pub(super) key: usize
}

impl<'a, P: Process> Step<'a, P> {
    pub(super) fn new(search_tree: &'a Mcts<P>, ptr: usize, key: usize) -> Self {
        Self { search_tree, ptr, key }
    }

    /// Returns the result of calling the given `mapper` on this steps `state`
    /// and `per_child`.
    ///
    /// # Arguments
    ///
    /// * `f` - the mapper for this steps `state` and `per_child`
    ///
    pub fn map<T>(&self, f: impl FnOnce(&P::State, &P::PerChild) -> T) -> T {
        self.search_tree.slab.read()[self.ptr].map(self.key, |state, edge| f(state, edge.per_child()))
    }
}
