use crate::{Mcts, process::Process, Step};
use crossbeam::epoch;

pub struct PathIter<'a, P: Process> {
    search_tree: &'a Mcts<P>,
    current: usize
}

impl<'a, P: Process> PathIter<'a, P> {
    pub(super) fn new(search_tree: &'a Mcts<P>) -> Self {
        let current = search_tree.root;

        Self { search_tree, current }
    }
}

impl<'a, P: Process> Iterator for PathIter<'a, P> {
    type Item = Step<'a, P>;

    fn next(&mut self) -> Option<Self::Item> {
        let nodes = self.search_tree.nodes.read();
        let per_childs = self.search_tree.per_childs.read();
        let pin = epoch::pin();
        let curr = self.current;

        if let Some((key, edge)) = nodes.get(curr).and_then(|node| node.best(&pin, &self.search_tree.process, &per_childs)) {
            if edge.is_valid() {
                self.current = edge.ptr();
            } else {
                self.current = usize::MAX;
            }

            Some(Step::new(self.search_tree, curr, key))
        } else {
            None
        }
    }
}
