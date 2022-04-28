use slab::Slab;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use smallvec::SmallVec;

use crate::{
    node::*,
    step::Step,
    Process, Trace, ProbeStatus
};

pub struct Mcts<P: Process> {
    pub(super) root: usize,
    pub(super) process: P,
    pub(super) slab: RwLock<Slab<Node<P>>>
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
        let root = slab.insert(Node::new(state));

        Self { root, process, slab: RwLock::new(slab) }
    }

    /// Returns the root node of this search tree.
    pub fn root(&self) -> MappedRwLockReadGuard<P::State> {
        RwLockReadGuard::map(self.slab.read(), |slab| slab[self.root].state())
    }

    /// Returns the _best_ sequence of nodes and edges through this search
    /// tree.
    pub fn path(&self) -> Trace {
        let slab = self.slab.read();
        let mut steps = SmallVec::new();
        let mut curr = self.root;
        let mut node = &slab[curr];

        while let Some((sparse_index, edge)) = node.best(&self.process) {
            steps.push(Step::new(curr, sparse_index));

            if !edge.is_valid() {
                break
            }

            curr = edge.ptr();
            node = &slab[curr];
        }

        Trace::new(steps, ProbeStatus::Success)
    }

    /// Returns a trace
    pub fn probe(&self) -> Result<Trace, ProbeStatus> {
        let slab = self.slab.read();
        let mut steps = SmallVec::new();
        let mut curr = self.root;

        loop {
            let node = &slab[curr];
            let next_index = match node.select(&self.process) {
                Some(next_index) => next_index,
                None => {
                    return Ok(Trace::new(steps, ProbeStatus::NoChildren))
                }
            };

            match node.try_set_expanding(next_index) {
                (sparse_index, ProbeStatus::AlreadyExpanded(next_index)) => {
                    steps.push(Step::new(curr, sparse_index));
                    curr = next_index;
                },
                (sparse_index, status) => {
                    steps.push(Step::new(curr, sparse_index));

                    return Ok(Trace::new(steps, status))
                },
            }
        }
    }

    fn insert(&self, trace: &Trace, new_state: P::State) -> RwLockReadGuard<Slab<Node<P>>> {
        let mut slab = self.slab.write();

        if let Some(last_step) = trace.steps().last() {
            let new_child = slab.insert(Node::new(new_state));
            let mut edge = slab[last_step.ptr].edge_mut(last_step.sparse_index);

            if !edge.try_insert(new_child) {
                drop(edge);
                slab.remove(new_child);
            }
        }

        RwLockWriteGuard::downgrade(slab)
    }

    pub fn update(&self, trace: Trace, state: Option<P::State>, up: P::Update) {
        let slab =
            if let Some(new_state) = state {
                self.insert(&trace, new_state)
            } else {
                self.slab.read()
            };

        for step in trace.steps() {
            let node = &slab[step.ptr];

            node.update(&self.process, step.sparse_index, &up);
        }
    }
}
