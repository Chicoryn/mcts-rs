use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, RwLockUpgradableReadGuard, Mutex};
use slab::Slab;
use std::collections::HashMap;

use crate::{
    node::*,
    step::Step,
    path_iter::PathIter,
    State, PerChild, Process, Trace, ProbeStatus, SelectResult
};

pub struct Mcts<P: Process> {
    pub(super) root: usize,
    pub(super) process: P,
    pub(super) nodes: RwLock<Slab<Node<P>>>,
    pub(super) per_childs: RwLock<Slab<P::PerChild>>,
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
        let mut nodes = Slab::new();
        let per_childs = RwLock::new(Slab::new());
        let root_hash = state.hash();
        let root = nodes.insert(Node::new(state));
        let mut transpositions = HashMap::with_capacity(32);

        if let Some(hash) = root_hash {
            transpositions.insert(hash, root);
        }

        Self { root, process, per_childs, nodes: RwLock::new(nodes), transpositions: Mutex::new(transpositions) }
    }

    pub fn len(&self) -> usize {
        self.transpositions.lock().len()
    }

    /// Returns the root node of this search tree.
    pub fn root(&self) -> MappedRwLockReadGuard<P::State> {
        RwLockReadGuard::map(self.nodes.read(), |slab| slab[self.root].state())
    }

    /// Returns the _best_ sequence of nodes and edges through this search
    /// tree.
    pub fn path<'a>(&'a self) -> impl Iterator<Item=Step<'a, P>> {
        PathIter::new(self)
    }

    /// Returns a trace
    pub fn probe<'a>(&'a self) -> (Trace<'a, P>, ProbeStatus) {
        let nodes = self.nodes.upgradable_read();
        let per_childs = self.per_childs.upgradable_read();
        let mut steps = Vec::new();
        let mut curr = self.root;

        loop {
            match nodes[curr].select(&self.process, &per_childs) {
                SelectResult::Add(per_child) => {
                    let next_key = per_child.key();
                    let node_mut = &mut RwLockUpgradableReadGuard::upgrade(nodes)[curr];
                    let mut per_childs_mut = RwLockUpgradableReadGuard::upgrade(per_childs);
                    let per_child_idx = per_childs_mut.insert(per_child);
                    node_mut.try_expand(&RwLockWriteGuard::downgrade(per_childs_mut), per_child_idx);
                    steps.push(Step::new(self, curr, next_key));

                    return (Trace::new(steps), ProbeStatus::Expanded)
                },
                SelectResult::Existing(next_key) => {
                    let edge = nodes[curr].edge(&per_childs, next_key);
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

    fn insert(&self, trace: &Trace<'_, P>, new_state: P::State) -> (RwLockReadGuard<Slab<Node<P>>>, RwLockReadGuard<Slab<P::PerChild>>) {
        let mut nodes = self.nodes.write();
        let per_childs = self.per_childs.read();

        if let Some(last_step) = trace.steps().last() {
            let new_hash = new_state.hash();
            let mut transpositions = self.transpositions.lock();
            let transposed_child = new_hash.and_then(|hash| transpositions.get(&hash).cloned());

            if let Some(transposed_child) = transposed_child {
                let edge = nodes[last_step.ptr].edge_mut(&per_childs, last_step.key);
                edge.try_insert(transposed_child);
            } else {
                let new_child = nodes.insert(Node::new(new_state));
                if let Some(hash) = new_hash {
                    transpositions.insert(hash, new_child);
                }

                let edge = nodes[last_step.ptr].edge_mut(&per_childs, last_step.key);
                if !edge.try_insert(new_child) {
                    drop(edge);
                    nodes.remove(new_child);
                }
            }
        }

        (RwLockWriteGuard::downgrade(nodes), per_childs)
    }

    pub fn update(&self, trace: Trace<'_, P>, state: Option<P::State>, up: P::Update) {
        let (nodes, per_childs) =
            if let Some(new_state) = state {
                self.insert(&trace, new_state)
            } else {
                (self.nodes.read(), self.per_childs.read())
            };

        for step in trace.steps() {
            let node = &nodes[step.ptr];
            let edge = node.edge(&per_childs, step.key);
            let per_child = &per_childs[edge.per_child()];

            self.process.update(node.state(), per_child, &up, edge.is_valid());
        }
    }
}
