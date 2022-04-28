use crate::{
    process::Process,
    mcts::Mcts,
};

pub struct Step {
    pub(super) ptr: usize,
    pub(super) sparse_index: usize
}

impl Step {
    pub(super) fn new(ptr: usize, sparse_index: usize) -> Self {
        Self { ptr, sparse_index }
    }

    /// Returns the process `state` and `per_child` that was chosen for this step.
    ///
    /// # Arguments
    ///
    /// - `search_tree` -
    ///
    pub fn map<P: Process, T>(&self, search_tree: &Mcts<P>, f: impl FnOnce(&P::State, &P::PerChild) -> T) -> T {
        search_tree.slab.read()[self.ptr].map(self.sparse_index, |state, edge| f(state, edge.per_child()))
    }
}
