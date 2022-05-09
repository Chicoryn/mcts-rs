use crossbeam::epoch::{self, Atomic, Owned};
use slab::Slab;
use smallvec::*;
use std::sync::atomic::Ordering;

use crate::{
    edge::Edge,
    process::{PerChild, SelectResult, Process},
};

pub(super) struct Node<P: Process> {
    state: P::State,
    edges: Atomic<SmallVec<[Edge<P>; 8]>>
}

impl<P: Process> Node<P> {
    pub(super) fn new(state: P::State) -> Self {
        let edges = Atomic::new(smallvec! []);

        Self { state, edges }
    }

    pub(super) fn best(&self, process: &P, per_childs: &Slab<P::PerChild>) -> Option<(<P::PerChild as PerChild>::Key, Edge<P>)> {
        let pin = epoch::pin();
        let edges = unsafe { self.edges.load_consume(&pin).deref() };

        if let Some(key) = process.best(&self.state, edges.iter().map(|edge| &per_childs[edge.per_child()])) {
            Some((key, self.edge(per_childs, key)))
        } else {
            None
        }
    }

    pub(super) fn state(&self) -> &P::State {
        &self.state
    }

    pub(super) fn edge(&self, per_childs: &Slab<P::PerChild>, key: <<P as Process>::PerChild as PerChild>::Key) -> Edge<P> {
        let pin = epoch::pin();
        let edges = unsafe { self.edges.load_consume(&pin).deref() };

        edges.binary_search_by_key(&key, |edge| per_childs[edge.per_child()].key()).map(|i| {
            edges[i].clone()
        }).unwrap()
    }

    pub(super) fn update<T>(&mut self, per_childs: &Slab<P::PerChild>, key: <<P as Process>::PerChild as PerChild>::Key, f: impl FnOnce(&mut Edge<P>) -> T) -> T {
        let pin = epoch::pin();
        let edges = unsafe { self.edges.load_consume(&pin).deref_mut() };
        let edge = edges.binary_search_by_key(&key, |edge| per_childs[edge.per_child()].key()).map(|i| {
            &mut edges[i]
        }).unwrap();

        f(edge)
    }

    pub(super) fn try_expand(&self, per_childs: &Slab<P::PerChild>, per_child: usize) {
        let pin = epoch::pin();
        let mut edges = unsafe { self.edges.load_consume(&pin).deref() }.clone();

        edges.push(Edge::new(per_child));
        edges.sort_unstable_by_key(|edge| per_childs[edge.per_child()].key());

        self.edges.swap(Owned::new(edges), Ordering::AcqRel, &pin);
    }

    pub(super) fn select(&self, process: &P, per_childs: &Slab<P::PerChild>) -> SelectResult<P::PerChild> {
        let pin = epoch::pin();
        let edges = unsafe { self.edges.load_consume(&pin).deref() };

        process.select(&self.state, edges.iter().map(|edge| &per_childs[edge.per_child()]))
    }

    pub(super) fn map<T>(&self, per_childs: &Slab<P::PerChild>, key: <P::PerChild as PerChild>::Key, f: impl FnOnce(&P::State, &Edge<P>, &P::PerChild) -> T) -> T {
        let pin = epoch::pin();
        let edges = unsafe { self.edges.load_consume(&pin).deref() };
        let edge = edges.binary_search_by_key(&key, |edge| per_childs[edge.per_child()].key()).map(|i| {
            &edges[i]
        }).unwrap();

        f(&self.state, edge, &per_childs[edge.per_child()])
    }
}
