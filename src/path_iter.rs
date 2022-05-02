use crate::{Mcts, process::Process, Step};

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
        let slab = self.search_tree.slab.read();
        let curr = self.current;

        if let Some((key, edge)) = slab.get(curr).and_then(|node| node.best(&self.search_tree.process)) {
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
