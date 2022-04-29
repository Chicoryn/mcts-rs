use smallvec::*;

use crate::{
    edge::Edge,
    probe_status::ProbeStatus,
    process::{PerChild, SelectResult, Process},
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

    pub(super) fn best(&self, process: &P) -> Option<(<P::PerChild as PerChild>::Key, &Edge<P>)> {
        if let Some(key) = process.best(&self.state, self.edges.iter().map(|edge| edge.per_child())) {
            Some((key, self.edge(key)))
        } else {
            None
        }
    }

    pub(super) fn state(&self) -> &P::State {
        &self.state
    }

    pub(super) fn edge(&self, key: <<P as Process>::PerChild as PerChild>::Key) -> &Edge<P> {
        self.edges.binary_search_by_key(&key, |edge| edge.per_child().key()).map(|i| {
            &self.edges[i]
        }).unwrap()
    }

    pub(super) fn edge_mut(&mut self, key: <<P as Process>::PerChild as PerChild>::Key) -> &mut Edge<P> {
        self.edges.binary_search_by_key(&key, |edge| edge.per_child().key()).map(|i| {
            &mut self.edges[i]
        }).unwrap()
    }

    pub(super) fn update(&self, process: &P, key: <P::PerChild as PerChild>::Key, up: &P::Update) {
        let edge = self.edge(key);
        let is_expanded = edge.is_valid();

        process.update(&self.state, edge.per_child(), &up, is_expanded)
    }

    pub(super) fn try_expand(&mut self, per_child: P::PerChild) -> (<P::PerChild as PerChild>::Key, ProbeStatus) {
        let key = per_child.key();

        self.edges.push(Edge::new(per_child));
        self.edges.sort_unstable_by_key(|edge| edge.per_child().key());

        (key, ProbeStatus::Expanded)
    }

    pub(super) fn select(&self, process: &P) -> SelectResult<P::PerChild> {
        process.select(&self.state, self.edges.iter().map(|edge| edge.per_child()))
    }

    pub(super) fn map<T>(&self, key: <P::PerChild as PerChild>::Key, f: impl FnOnce(&P::State, &Edge<P>) -> T) -> T {
        f(&self.state, self.edge(key))
    }
}
