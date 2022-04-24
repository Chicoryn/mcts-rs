use smallvec::*;

use crate::{
    edge::Edge,
    probe_status::ProbeStatus,
    process::Process
};

pub(super) struct Node<P: Process> {
    state: P::State,
    edges: SmallVec<[Edge<P>; 8]>
}

impl<P: Process> Node<P> {
    pub(super) fn new(state: P::State) -> Self {
        let edges = smallvec! [];

        Self { state, edges }
    }

    pub(super) fn best(&self, process: &P) -> Option<&Edge<P>> {
        if let Some(per_child) = process.best(&self.state, self.edges.iter().map(|edge| *edge.per_child())) {
            self.edges.iter()
                .filter(|edge| edge.per_child().eq(&per_child))
                .next()
        } else {
            None
        }
    }

    pub(super) fn state(&self) -> &P::State {
        &self.state
    }

    pub(super) fn edge_mut(&mut self, sparse_index: usize) -> &mut Edge<P> {
        &mut self.edges[sparse_index]
    }

    fn try_insert(&mut self, per_child: P::PerChild) -> usize {
        self.edges.push(Edge::new(per_child));
        self.edges.len() - 1
    }

    pub(super) fn try_set_expanding(&mut self, per_child: P::PerChild) -> (usize, ProbeStatus) {
        for (i, edge) in self.edges.iter_mut().enumerate() {
            if edge.per_child().eq(&per_child) {
                if edge.is_valid() {
                    return (i, ProbeStatus::AlreadyExpanded(edge.ptr()))
                } else {
                    return (i, ProbeStatus::AlreadyExpanding)
                }
            }
        }

        (self.try_insert(per_child), ProbeStatus::Success)
    }

    pub(super) fn as_node_struct<'a>(&'a mut self, sparse_index: usize) -> (&'a mut P::State, &'a mut Edge<P>) {
        (&mut self.state, &mut self.edges[sparse_index])
    }

    pub(super) fn select(&self, process: &P) -> Option<P::PerChild> {
        process.select(&self.state, self.edges.iter().map(|edge| *edge.per_child()))
    }
}

#[cfg(test)]
mod tests {
    use crate::FakeProcess;
    use super::*;

    #[test]
    fn check_expanding() {
        let mut node = Node::<FakeProcess>::new(());

        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::Success));
        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::AlreadyExpanding));
    }

    #[test]
    fn check_double_expanding() {
        let mut node = Node::<FakeProcess>::new(());

        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::Success));
        assert_eq!(node.try_set_expanding(2), (1, ProbeStatus::Success));
    }

    #[test]
    fn check_already_expanded() {
        let mut node = Node::<FakeProcess>::new(());

        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::Success));
        let (_, edge) = node.as_node_struct(0);
        edge.insert(255);

        assert_eq!(node.try_set_expanding(1), (0, ProbeStatus::AlreadyExpanded(255)));
    }
}
