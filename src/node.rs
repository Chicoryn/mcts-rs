use parking_lot::{Mutex, MutexGuard, MappedMutexGuard};
use smallvec::*;
use std::ops::Deref;

use crate::{
    edge::Edge,
    probe_status::ProbeStatus,
    process::{PerChild, SelectResult, Process},
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

        if let Some(key) = process.best(&self.state, edges.iter().map(|edge| edge.per_child())) {
            edges.binary_search_by_key(&key, |edge| edge.per_child().key()).map(|i| {
                (key, MutexGuard::map(edges, |edges| &mut edges[i]))
            }).ok()
        } else {
            None
        }
    }

    pub(super) fn state(&self) -> &P::State {
        &self.state
    }

    pub(super) fn edge_mut(&self, key: usize) -> MappedMutexGuard<Edge<P>> {
        MutexGuard::map(self.edges.lock(), |edges| {
            edges.binary_search_by_key(&key, |edge| edge.per_child().key()).map(|i| {
                &mut edges[i]
            }).unwrap()
        })
    }

    pub(super) fn update(&self, process: &P, key: usize, up: &P::Update) {
        let mut edge = self.edge_mut(key);
        let is_expanded = edge.is_valid();

        process.update(&self.state, edge.per_child_mut(), &up, is_expanded)
    }

    pub(super) fn select(&self, process: &P) -> Option<(usize, ProbeStatus)> {
        let mut edges = self.edges.lock();
        let mut busy = ProbeStatus::Busy;
        let key = match process.select(&self.state, edges.iter().map(|edge| edge.per_child())) {
            SelectResult::Existing(key) => key,
            SelectResult::Add(per_child) => {
                let key = per_child.key();
                edges.push(Edge::new(per_child));
                edges.sort_unstable_by_key(|edge| edge.per_child().key());
                busy = ProbeStatus::Expanded;
                key
            },
            SelectResult::None => { return None }
        };

        edges.binary_search_by_key(&key, |edge| edge.per_child().key()).map(|i| {
            let edge = &edges[i];

            if edge.is_valid() {
                (key, ProbeStatus::Existing(edge.ptr()))
            } else {
                (key, busy)
            }
        }).ok()
    }

    pub(super) fn map<T>(&self, key: usize, f: impl FnOnce(&P::State, &Edge<P>) -> T) -> T {
        f(&self.state, self.edge_mut(key).deref())
    }
}
