use crossbeam_epoch as epoch;
use std::rc::Rc;

use crate::{
    safe_nonnull::SafeNonNull,
    node::Node,
    process::Process,
    Mcts,
    Step
};

pub struct PathIter<'a, P: Process> {
    search_tree: &'a Mcts<P>,
    pin: Rc<epoch::Guard>,
    current: Option<SafeNonNull<Node<P>>>
}

impl<'a, P: Process> PathIter<'a, P> {
    pub(super) fn new(search_tree: &'a Mcts<P>) -> Self {
        let current = Some(search_tree.root);
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
        let curr = self.current;

        if let Some((key, edge)) = curr.and_then(|node| node.best(self.pin(), &search_tree.process)) {
            self.current = edge.ptr();

            Some(Step::new(search_tree, self.pin.clone(), curr.unwrap(), key))
        } else {
            None
        }
    }
}
