use crossbeam::epoch;
use std::rc::Rc;

use crate::{Mcts, process::Process, Step};

pub struct PathIter<'a, P: Process> {
    search_tree: &'a Mcts<P>,
    pin: Rc<epoch::Guard>,
    current: usize
}

impl<'a, P: Process> PathIter<'a, P> {
    pub(super) fn new(search_tree: &'a Mcts<P>) -> Self {
        let current = search_tree.root;
        let pin = Rc::new(epoch::pin());

        Self { search_tree, pin, current }
    }

    pub(super) fn pin(&self) -> &epoch::Guard {
        self.pin.as_ref()
    }
}

impl<'a, P: Process> Iterator for PathIter<'a, P> {
    type Item = Step<'a, P>;

    fn next(&mut self) -> Option<Self::Item> {
        let search_tree = self.search_tree;
        let nodes = search_tree.nodes.read();
        let per_childs = search_tree.per_childs.read();
        let curr = self.current;

        if let Some((key, edge)) = nodes.get(curr).and_then(|node| node.best(self.pin(), &search_tree.process, &per_childs)) {
            if edge.is_valid() {
                self.current = edge.ptr();
            } else {
                self.current = usize::MAX;
            }

            Some(Step::new(search_tree, self.pin.clone(), curr, key))
        } else {
            None
        }
    }
}
