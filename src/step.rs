use crossbeam::epoch;
use crate::{
    process::{PerChild, Process},
    mcts::Mcts,
};

pub struct Step<'a, P: Process> {
    pub(super) search_tree: &'a Mcts<P>,
    pub(super) ptr: usize,
    pub(super) key: <P::PerChild as PerChild>::Key
}

impl<'a, P: Process> Step<'a, P> {
    pub(super) fn new(search_tree: &'a Mcts<P>, ptr: usize, key: <P::PerChild as PerChild>::Key) -> Self {
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
        let nodes = self.search_tree.nodes.read();
        let per_childs = self.search_tree.per_childs.read();
        let pin = epoch::pin();

        nodes[self.ptr].map(&pin, &per_childs, self.key, |state, _, per_child| f(state, per_child))
    }
}
