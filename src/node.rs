use crossbeam_epoch::{self as epoch, Atomic, Owned, Guard};
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

impl<P: Process> Drop for Node<P> {
    fn drop(&mut self) {
        let pin = epoch::pin();
        let edges = unsafe { self.edges.load_consume(&pin).deref_mut() };

        for mut edge in edges.drain(..) {
            edge.drop();
        }
    }
}

impl<P: Process> Node<P> {
    pub(super) fn new(state: P::State) -> Self {
        let edges = Atomic::new(smallvec! []);

        Self { state, edges }
    }

    pub(super) fn best<'g>(&self, pin: &'g Guard, process: &P) -> Option<(<P::PerChild as PerChild>::Key, &'g Edge<P>)> {
        let edges = unsafe { self.edges.load_consume(pin).deref() };

        if let Some(key) = process.best(&self.state, edges.iter().map(|edge| edge.per_child())) {
            Some((key, self.edge(pin, key)))
        } else {
            None
        }
    }

    pub(super) fn state(&self) -> &P::State {
        &self.state
    }

    pub(super) fn edge<'g>(&self, pin: &'g Guard, key: <<P as Process>::PerChild as PerChild>::Key) -> &'g Edge<P> {
        let edges = unsafe { self.edges.load_consume(pin).deref() };

        edges.binary_search_by_key(&key, |edge| edge.key()).map(|i| {
            &edges[i]
        }).unwrap()
    }

    pub(super) fn try_expand<'g>(&self, pin: &'g Guard, per_child: P::PerChild) {
        let edge = Edge::new(per_child);

        loop {
            let current = self.edges.load_consume(pin);
            let mut edges = unsafe { current.deref() }.clone();

            edges.push(edge.clone());
            edges.sort_unstable_by_key(|edge| edge.key());

            if self.edges.compare_exchange_weak(current, Owned::new(edges), Ordering::AcqRel, Ordering::Relaxed, &pin).is_ok() {
                break
            }
        }
    }

    pub(super) fn select<'g>(&self, pin: &'g Guard, process: &P) -> SelectResult<P::PerChild> {
        let edges = unsafe { self.edges.load_consume(pin).deref() };

        process.select(&self.state, edges.iter().map(|edge| edge.per_child()))
    }

    pub(super) fn map<'g, T>(&self, pin: &'g Guard, key: <P::PerChild as PerChild>::Key, f: impl FnOnce(&P::State, &Edge<P>, &P::PerChild) -> T) -> T {
        let edges = unsafe { self.edges.load_consume(pin).deref() };
        let edge = edges.binary_search_by_key(&key, |edge| edge.key()).map(|i| {
            &edges[i]
        }).unwrap();

        f(&self.state, edge, edge.per_child())
    }
}
