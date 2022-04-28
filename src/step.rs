use crate::{
    process::Process,
    mcts::Mcts,
};

pub struct Step<'a, P: Process> {
    pub(super) search_tree: &'a Mcts<P>,
    pub(super) ptr: usize,
    pub(super) sparse_index: usize
}

impl<'a, P: Process> Step<'a, P> {
    pub(super) fn new(search_tree: &'a Mcts<P>, ptr: usize, sparse_index: usize) -> Self {
        Self { search_tree, ptr, sparse_index }
    }

    /// Returns the process `state` and `per_child` that was chosen for this step.
    ///
    /// # Arguments
    ///
    /// - `search_tree` -
    ///
    pub fn map<T>(&self, f: impl FnOnce(&P::State, &P::PerChild) -> T) -> T {
        self.search_tree.slab.read()[self.ptr].map(self.sparse_index, |state, edge| f(state, edge.per_child()))
    }
}
