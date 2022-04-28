use parking_lot::{Mutex, MutexGuard, MappedMutexGuard};
use smallvec::*;

use crate::{
    edge::Edge,
    probe_status::ProbeStatus,
    process::Process
};

pub(super) struct Node<P: Process> {
    state: P::State,
    edges: Mutex<SmallVec<[Edge<P>; 8]>>
}

impl<P: Process> Node<P> {
    pub(super) fn new(state: P::State) -> Self {
        let edges = Mutex::new(smallvec! []);

        Self { state, edges }
    }

    pub(super) fn best(&self, process: &P) -> Option<(usize, MappedMutexGuard<Edge<P>>)> {
        let edges = self.edges.lock();

        if let Some(per_child) = process.best(&self.state, edges.iter().map(|edge| edge.per_child().clone())) {
            for (i, edge) in edges.iter().enumerate() {
                if edge.per_child().eq(&per_child) {
                    return Some((i, MutexGuard::map(edges, |edges| &mut edges[i])))
                }
            }

            None
        } else {
            None
        }
    }

    pub(super) fn state(&self) -> &P::State {
        &self.state
    }

    pub(super) fn edge_mut(&mut self, sparse_index: usize) -> MappedMutexGuard<Edge<P>> {
        MutexGuard::map(self.edges.lock(), |edges| &mut edges[sparse_index])
    }

    pub(super) fn try_set_expanding(&self, per_child: P::PerChild) -> (usize, ProbeStatus) {
        let mut edges = self.edges.lock();

        for (i, edge) in edges.iter_mut().enumerate() {
            if edge.per_child().eq(&per_child) {
                if edge.is_valid() {
                    return (i, ProbeStatus::AlreadyExpanded(edge.ptr()))
                } else {
                    return (i, ProbeStatus::AlreadyExpanding)
                }
            }
        }

        ({
            edges.push(Edge::new(per_child));
            edges.len() - 1
        }, ProbeStatus::Success)
    }

    pub(super) fn update(&self, process: &P, sparse_index: usize, up: &P::Update) {
        let mut edge = MutexGuard::map(self.edges.lock(), |edges| &mut edges[sparse_index]);
        let is_expanded = edge.is_valid();

        process.update(&self.state, edge.per_child_mut(), &up, is_expanded)
    }

    pub(super) fn select(&self, process: &P) -> Option<P::PerChild> {
        let edges = self.edges.lock();

        process.select(&self.state, edges.iter().map(|edge| edge.per_child().clone()))
    }

    pub(super) fn map<T>(&self, sparse_index: usize, f: impl FnOnce(&P::State, &Edge<P>) -> T) -> T {
        let edges = self.edges.lock();

        f(&self.state, &edges[sparse_index])
    }
}

#[cfg(test)]
mod tests {
    use crate::FakeProcess;
    use super::*;

    #[test]
    fn check_expanding() {
        let node = Node::<FakeProcess>::new(());

        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::Success));
        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::AlreadyExpanding));
    }

    #[test]
    fn check_double_expanding() {
        let node = Node::<FakeProcess>::new(());

        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::Success));
        assert_eq!(node.try_set_expanding(2), (1, ProbeStatus::Success));
    }

    #[test]
    fn check_already_expanded() {
        let mut node = Node::<FakeProcess>::new(());

        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::Success));
        assert_eq!(node.edge_mut(0).try_insert(255), true);

        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::AlreadyExpanded(255)));
    }
}
