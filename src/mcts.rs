use slab::Slab;
use smallvec::SmallVec;

use crate::{
    node::*,
    step::Step,
    Process, Trace, ProbeStatus
};

pub struct Mcts<P: Process> {
    pub(super) root: usize,
    pub(super) process: P,
    pub(super) slab: Slab<Node<P>>
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

        Self { root, process, slab }
    }

    /// Returns the root node of this search tree.
    pub fn root(&self) -> &P::State {
        &self.slab[self.root].state()
    }

    /// Returns the _best_ sequence of nodes and edges through this search
    /// tree.
    pub fn path(&self) -> Vec<(&P::State, &P::PerChild)> {
        let mut node = &self.slab[self.root];
        let mut out = Vec::with_capacity(16);

        while let Some(edge) = node.best(&self.process) {
            out.push((node.state(), edge.per_child()));
            if !edge.is_valid() {
                break
            }

            node = &self.slab[edge.ptr()];
        }

        out
    }

    /// Returns a trace
    pub fn probe(&mut self) -> Result<Trace, ProbeStatus> {
        let mut steps = SmallVec::new();
        let mut curr = self.root;

        loop {
            let node = &mut self.slab[curr];
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

    fn insert(&mut self, trace: &Trace, new_state: P::State) {
        if let Some(last_step) = trace.steps().last() {
            let new_child = self.slab.insert(Node::new(new_state));
            let edge = last_step.as_edge(self);

            if !edge.try_insert(new_child) {
                self.slab.remove(new_child);
            }
        }
    }

    pub fn update(&mut self, trace: Trace, state: Option<P::State>, up: P::Update) {
        if let Some(new_state) = state {
            self.insert(&trace, new_state);
        }

        for step in trace.steps() {
            step.update(self, &up);
        }
    }
}
