use crate::{
    edge::Edge,
    process::Process,
    mcts::Mcts,
};

pub struct Step {
    ptr: usize,
    sparse_index: usize
}

impl Step {
    pub(super) fn new(ptr: usize, sparse_index: usize) -> Self {
        Self { ptr, sparse_index }
    }

    pub(super) fn as_edge<'a, P: Process>(&self, search_tree: &'a mut Mcts<P>) -> &'a mut Edge<P> {
        search_tree.slab[self.ptr].edge_mut(self.sparse_index)
    }

    pub(super) fn update<P: Process>(&self, search_tree: &mut Mcts<P>, up: &P::Update) {
        let node = &mut search_tree.slab[self.ptr];
        let (state, edge) = node.as_node_struct(self.sparse_index);
        let is_expanded = edge.is_valid();

        search_tree.process.update(state, edge.per_child_mut(), &up, is_expanded);
    }

    /// Returns the process `state` and `per_child` that was chosen for this step.
    ///
    /// # Arguments
    ///
    /// - `search_tree` -
    ///
    pub fn as_state<'a, P: Process>(&self, search_tree: &'a mut Mcts<P>) -> (&'a mut P::State, &'a mut P::PerChild) {
        let node = &mut search_tree.slab[self.ptr];
        let (state, edge) = node.as_node_struct(self.sparse_index);

        (state, edge.per_child_mut())
    }
}
