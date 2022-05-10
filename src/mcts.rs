use crossbeam_epoch as epoch;
use dashmap::DashMap;
use std::rc::Rc;

use crate::{
    node::*,
    path_iter::PathIter,
    safe_nonnull::SafeNonNull,
    State, Process, Trace, ProbeStatus, SelectResult, Step, PerChild
};

pub struct Mcts<P: Process> {
    pub(super) root: SafeNonNull<Node<P>>,
    pub(super) process: P,
    pub(super) transpositions: DashMap<u64, SafeNonNull<Node<P>>>
}

impl<P: Process> Mcts<P> {
    /// Returns a new monte-carlo search tree for the given `process` and
    /// initial `state`.
    ///
    /// # Arguments
    ///
    /// * `process` -
    /// * `state` -
    ///
    pub fn new(process: P, state: P::State) -> Self {
        let root_hash = state.hash();
        let root = SafeNonNull::new(Node::new(state));
        let transpositions = DashMap::with_capacity(32);

        if let Some(hash) = root_hash {
            transpositions.insert(hash, root);
        }

        Self { root, process, transpositions }
    }

    pub fn len(&self) -> usize {
        self.transpositions.len()
    }

    /// Returns the root node of this search tree.
    pub fn root(&self) -> &P::State {
        self.root.state()
    }

    /// Returns the _best_ sequence of nodes and edges through this search
    /// tree.
    pub fn path<'a>(&'a self) -> impl Iterator<Item=Step<'a, P>> {
        PathIter::new(self)
    }

    /// Returns a trace
    pub fn probe<'a>(&'a self) -> (Trace<'a, P>, ProbeStatus) {
        let pin = Rc::new(epoch::pin());
        let mut trace = Trace::new();
        let mut curr = self.root;

        loop {
            match curr.select(&pin, &self.process) {
                SelectResult::Add(per_child) => {
                    let next_key = per_child.key();
                    curr.try_expand(&pin, per_child);
                    trace.push(self, pin.clone(), curr, next_key);

                    return (trace, ProbeStatus::Expanded)
                },
                SelectResult::Existing(next_key) => {
                    trace.push(self, pin.clone(), curr, next_key);

                    if let Some(next_curr) = curr.edge(&pin, next_key).ptr() {
                        curr = next_curr;
                    } else {
                        return (trace, ProbeStatus::Busy);
                    }
                },
                SelectResult::None => { return (trace, ProbeStatus::Empty) }
            }
        }
    }

    fn insert(&self, trace: &Trace<'_, P>, new_state: P::State) {
        if let Some(last_step) = trace.steps().last() {
            let new_hash = new_state.hash();
            let transposed_child = new_hash.and_then(|hash| self.transpositions.get(&hash)).map(|entry| entry.value().clone());

            if let Some(transposed_child) = transposed_child {
                let edge = last_step.ptr.edge(last_step.pin(), last_step.key);
                edge.try_insert(transposed_child);
            } else {
                let mut new_child = SafeNonNull::new(Node::new(new_state));

                if last_step.ptr.edge(last_step.pin(), last_step.key).try_insert(new_child.clone()) {
                    if let Some(hash) = new_hash {
                        self.transpositions.insert(hash, new_child);
                    }
                } else {
                    new_child.drop();
                }
            }
        }
    }

    pub fn update(&self, trace: Trace<'_, P>, state: Option<P::State>, up: P::Update) {
        if let Some(new_state) = state {
            self.insert(&trace, new_state)
        }

        for step in trace.steps() {
            let node = &step.ptr;
            let edge = node.edge(step.pin(), step.key);

            self.process.update(node.state(), edge.per_child(), &up, edge.ptr().is_some());
        }
    }
}
