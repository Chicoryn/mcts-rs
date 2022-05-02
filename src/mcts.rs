use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, RwLockUpgradableReadGuard, Mutex};
use slab::Slab;
use smallvec::SmallVec;
use std::collections::HashMap;

use crate::{
    node::*,
    step::Step,
    path_iter::PathIter,
    State, Process, Trace, ProbeStatus, SelectResult
};

pub struct Mcts<P: Process> {
    pub(super) root: usize,
    pub(super) process: P,
    pub(super) slab: RwLock<Slab<Node<P>>>,
    pub(super) transpositions: Mutex<HashMap<u64, usize>>
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
        let mut slab = Slab::new();
        let root_hash = state.hash();
        let root = slab.insert(Node::new(state));
        let mut transpositions = HashMap::with_capacity(32);

        if let Some(hash) = root_hash {
            transpositions.insert(hash, root);
        }

        Self { root, process, slab: RwLock::new(slab), transpositions: Mutex::new(transpositions) }
    }

    pub fn len(&self) -> usize {
        self.transpositions.lock().len()
    }

    /// Returns the root node of this search tree.
    pub fn root(&self) -> MappedRwLockReadGuard<P::State> {
        RwLockReadGuard::map(self.slab.read(), |slab| slab[self.root].state())
    }

    /// Returns the _best_ sequence of nodes and edges through this search
    /// tree.
    pub fn path<'a>(&'a self) -> PathIter<P> {
        PathIter::new(self)
    }

    /// Returns a trace
    pub fn probe<'a>(&'a self) -> (Trace<'a, P>, ProbeStatus) {
        let slab = self.slab.upgradable_read();
        let mut steps = SmallVec::new();
        let mut curr = self.root;

        loop {
            match slab[curr].select(&self.process) {
                SelectResult::Add(per_child) => {
                    let node_mut = &mut RwLockUpgradableReadGuard::upgrade(slab)[curr];
                    let (next_key, status) = node_mut.try_expand(per_child);
                    steps.push(Step::new(self, curr, next_key));

                    return (Trace::new(steps), status)
                },
                SelectResult::Existing(next_key) => {
                    let edge = slab[curr].edge(next_key);
                    steps.push(Step::new(self, curr, next_key));

                    if edge.is_valid() {
                        curr = edge.ptr();
                    } else {
                        return (Trace::new(steps), ProbeStatus::Busy);
                    }
                },
                SelectResult::None => { return (Trace::new(steps), ProbeStatus::Empty) }
            }
        }
    }

    fn insert(&self, trace: &Trace<'_, P>, new_state: P::State) -> RwLockReadGuard<Slab<Node<P>>> {
        let mut slab = self.slab.write();

        if let Some(last_step) = trace.steps().last() {
            let new_hash = new_state.hash();
            let mut transpositions = self.transpositions.lock();
            let transposed_child = new_hash.and_then(|hash| transpositions.get(&hash).cloned());

            if let Some(transposed_child) = transposed_child {
                let edge = slab[last_step.ptr].edge_mut(last_step.key);
                edge.try_insert(transposed_child);
            } else {
                let new_child = slab.insert(Node::new(new_state));
                if let Some(hash) = new_hash {
                    transpositions.insert(hash, new_child);
                }

                let edge = slab[last_step.ptr].edge_mut(last_step.key);
                if !edge.try_insert(new_child) {
                    drop(edge);
                    slab.remove(new_child);
                }
            }
        }

        RwLockWriteGuard::downgrade(slab)
    }

    pub fn update(&self, trace: Trace<'_, P>, state: Option<P::State>, up: P::Update) {
        let slab =
            if let Some(new_state) = state {
                self.insert(&trace, new_state)
            } else {
                self.slab.read()
            };

        for step in trace.steps() {
            let node = &slab[step.ptr];

            node.update(&self.process, step.key, &up);
        }
    }
}
