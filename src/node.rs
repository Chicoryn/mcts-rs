use smallvec::*;
use slab::Slab;

use crate::{
    edge::Edge,
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

    pub(super) fn best(&self, process: &P, per_childs: &Slab<P::PerChild>) -> Option<(<P::PerChild as PerChild>::Key, &Edge<P>)> {
        let edges = self.edges.iter().map(|edge| &per_childs[edge.per_child()]);

        if let Some(key) = process.best(&self.state, edges) {
            Some((key, self.edge(per_childs, key)))
        } else {
            None
        }
    }

    pub(super) fn state(&self) -> &P::State {
        &self.state
    }

    pub(super) fn edge(&self, per_childs: &Slab<P::PerChild>, key: <<P as Process>::PerChild as PerChild>::Key) -> &Edge<P> {
        self.edges.binary_search_by_key(&key, |edge| per_childs[edge.per_child()].key()).map(|i| {
            &self.edges[i]
        }).unwrap()
    }

    pub(super) fn edge_mut(&mut self, per_childs: &Slab<P::PerChild>, key: <<P as Process>::PerChild as PerChild>::Key) -> &mut Edge<P> {
        self.edges.binary_search_by_key(&key, |edge| per_childs[edge.per_child()].key()).map(|i| {
            &mut self.edges[i]
        }).unwrap()
    }

    pub(super) fn try_expand(&mut self, per_childs: &Slab<P::PerChild>, per_child: usize) {
        self.edges.push(Edge::new(per_child));
        self.edges.sort_unstable_by_key(|edge| per_childs[edge.per_child()].key());
    }

    pub(super) fn select(&self, process: &P, per_childs: &Slab<P::PerChild>) -> SelectResult<P::PerChild> {
        let edges = self.edges.iter().map(|edge| &per_childs[edge.per_child()]);

        process.select(&self.state, edges)
    }

    pub(super) fn map<T>(&self, per_childs: &Slab<P::PerChild>, key: <P::PerChild as PerChild>::Key, f: impl FnOnce(&P::State, &Edge<P>, &P::PerChild) -> T) -> T {
        let edge = self.edge(per_childs, key);

        f(&self.state, edge, &per_childs[edge.per_child()])
    }
}
