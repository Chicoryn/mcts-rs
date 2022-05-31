use crate::{edge::Edge, process::{PerChild, SelectResult, Process}, safe_nonnull::SafeNonNull};
use crossbeam_epoch::{Atomic, Owned, Guard};
use smallvec::*;
use std::{collections::HashSet, ops::DerefMut, mem, sync::atomic::Ordering};

/// An interior node which represents a game state.
pub struct Node<P: Process> {
    state: P::State,
    edges: Atomic<SmallVec<[SafeNonNull<Edge<P, Node<P>>>; 8]>>
}

impl<P: Process> Drop for Node<P> {
    fn drop(&mut self) {
        unsafe {
            drop(mem::replace(&mut self.edges, Atomic::null()).into_owned());
        }
    }
}

impl<P: Process> Node<P> {
    pub(super) fn new(state: P::State) -> Self {
        let edges = Atomic::new(smallvec! []);

        Self { state, edges }
    }

    #[inline]
    pub(super) fn edges<'g>(&self, pin: &'g Guard) -> &'g [SafeNonNull<Edge<P, Node<P>>>] {
        unsafe { self.edges.load_consume(pin).deref() }
    }

    pub(super) fn recursive_drop(&mut self, pin: &Guard, already_dropped: &mut HashSet<*mut Node<P>>) {
        for edge in self.edges(pin) {
            if let Some(mut ptr) = edge.ptr() {
                if already_dropped.insert(ptr.as_ptr()) {
                    ptr.deref_mut().recursive_drop(pin, already_dropped);
                    ptr.drop();
                }
            }

            edge.drop();
        }
    }

    pub(super) fn best<'g>(&self, pin: &'g Guard, process: &P) -> Option<(<P::PerChild as PerChild>::Key, &'g Edge<P, Node<P>>)> {
        if let Some(key) = process.best(&self.state, self.edges(pin).iter().map(|edge| edge.per_child())) {
            self.edge(pin, key).map(|edge| (key, edge))
        } else {
            None
        }
    }

    pub(super) fn state(&self) -> &P::State {
        &self.state
    }

    #[cfg(test)]
    pub(super) fn len<'g>(&self, pin: &'g Guard) -> usize {
        self.edges(pin).len()
    }

    pub(super) fn edge<'g>(&self, pin: &'g Guard, key: <<P as Process>::PerChild as PerChild>::Key) -> Option<&'g Edge<P, Node<P>>> {
        let edges = self.edges(pin);

        edges.binary_search_by_key(&key, |edge| edge.key()).map(|i| {
            &*edges[i]
        }).ok()
    }

    pub(super) fn try_expand<'g>(&self, pin: &'g Guard, per_child: P::PerChild) {
        let edge = SafeNonNull::new(Edge::new(per_child));
        let mut current = self.edges.load_consume(pin);

        loop {
            let mut edges = unsafe { current.deref() }.clone();

            edges.push(edge.clone());
            edges.sort_unstable_by_key(|edge| edge.key());

            match self.edges.compare_exchange_weak(current, Owned::new(edges), Ordering::AcqRel, Ordering::Relaxed, pin) {
                Ok(_) => {
                    unsafe { pin.defer_destroy(current) };
                    break
                },
                Err(err) => {
                    current = err.current;
                }
            }
        }
    }

    pub(super) fn select<'g>(&self, pin: &'g Guard, process: &P) -> SelectResult<P::PerChild> {
        process.select(&self.state, self.edges(pin).iter().map(|edge| edge.per_child()))
    }

    pub(super) fn map<'g, T>(&self, pin: &'g Guard, key: <P::PerChild as PerChild>::Key, f: impl FnOnce(&P::State, &Edge<P, Node<P>>, &P::PerChild) -> T) -> T {
        let edge = self.edge(pin, key).unwrap();

        f(&self.state, edge, edge.per_child())
    }
}

#[cfg(test)]
mod tests {
    use crate::{FakePerChild, FakeProcess, FakeState};
    use crossbeam_epoch as epoch;
    use super::*;

    #[test]
    fn new_is_empty() {
        let pin = epoch::pin();
        let node: Node<FakeProcess> = Node::new(FakeState::new());
        assert_eq!(node.len(&pin), 0);
    }

    #[test]
    fn try_expand_add_one_edge() {
        let pin = epoch::pin();
        let node: Node<FakeProcess> = Node::new(FakeState::new());
        node.try_expand(&pin, FakePerChild::new(0));
        assert_eq!(node.len(&pin), 1);
    }

    #[test]
    fn map_gets_the_correct_edge() {
        let pin = epoch::pin();
        let node: Node<FakeProcess> = Node::new(FakeState::new());
        node.try_expand(&pin, FakePerChild::new(0));
        node.try_expand(&pin, FakePerChild::new(1));
        node.try_expand(&pin, FakePerChild::new(2));

        node.map(&pin, 0, |_, _, per_child| { assert_eq!(per_child.key(), 0); });
        node.map(&pin, 1, |_, _, per_child| { assert_eq!(per_child.key(), 1); });
        node.map(&pin, 2, |_, _, per_child| { assert_eq!(per_child.key(), 2); });
    }

    #[test]
    fn best_gets_none_when_empty() {
        let pin = epoch::pin();
        let process = FakeProcess::new(0, 0);
        let node: Node<FakeProcess> = Node::new(FakeState::new());
        assert!(node.best(&pin, &process).is_none());
    }

    #[test]
    fn best_gets_correct_edge() {
        let pin = epoch::pin();
        let process = FakeProcess::new(1, 0);
        let node: Node<FakeProcess> = Node::new(FakeState::new());
        node.try_expand(&pin, FakePerChild::new(0));
        node.try_expand(&pin, FakePerChild::new(1));
        node.try_expand(&pin, FakePerChild::new(2));

        assert_eq!(node.best(&pin, &process).map(|(key, _)| key), Some(1));
    }

    #[test]
    fn select_gets_correct_edge() {
        let pin = epoch::pin();
        let process = FakeProcess::new(0, 1);
        let node: Node<FakeProcess> = Node::new(FakeState::new());
        assert_eq!(node.select(&pin, &process), SelectResult::Add(FakePerChild::new(1)));
    }
}
