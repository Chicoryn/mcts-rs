use crate::{node::Node, path_iter::PathIter, probe_status::ProbeStatus, process::{State, Process, PerChild, SelectResult}, safe_nonnull::SafeNonNull, step::Step, trace::Trace};
use crossbeam_epoch as epoch;
use dashmap::DashMap;
use std::{collections::HashSet, ops::DerefMut, rc::Rc};

pub struct Mcts<P: Process> {
    root: SafeNonNull<Node<P>>,
    process: P,
    transpositions: DashMap<u64, SafeNonNull<Node<P>>>
}

impl<P: Process> Drop for Mcts<P> {
    fn drop(&mut self) {
        let mut already_dropped = HashSet::with_capacity(self.len());
        let pin = unsafe { epoch::unprotected() };

        self.root.deref_mut().recursive_drop(pin, &mut already_dropped);
        if already_dropped.insert(self.root.as_ptr()) {
            self.root.drop();
        }
    }
}

impl<P: Process> Mcts<P> {
    /// Returns a new monte-carlo search tree for the given `process` and
    /// initial `state`.
    ///
    /// # Arguments
    ///
    /// * `process` - the monte carlo process to evaluate
    /// * `state` - the initial root state
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

    /// Returns the number of entries in the transposition table. This should
    /// correspond to the number of *unique* nodes in the search tree.
    pub fn len(&self) -> usize {
        self.transpositions.len()
    }

    /// Returns the process being evaluated by this search tree.
    pub fn process(&self) -> &P {
        &self.process
    }

    /// Returns the state of the root node in this search tree.
    pub fn root(&self) -> &P::State {
        self.root.state()
    }

    /// Returns the _best_ sequence of nodes and edges through this search
    /// tree.
    pub fn path<'a>(&'a self) -> impl Iterator<Item=Step<'a, P, Node<P>>> {
        PathIter::new(&self.process, self.root)
    }

    /// Returns a `trace` which represents the best path through this tree to
    /// explore at this moment according to the given monte-carlo process and
    /// selection criteria.
    ///
    /// Returns `ProbeStatus::Expanded` if the `trace` contains a previously
    /// unexplored edge as its final step; `ProbeStatus::Busy` if the final edge
    /// exist but has not yet been expanded yet; and `ProbeStatus::Empty` if
    /// the current selection criterias yielded a terminal node, which has no
    /// more edges to traverse.
    pub fn probe<'a>(&'a self) -> (Trace<'a, P, Node<P>>, ProbeStatus) {
        let pin = Rc::new(epoch::pin());
        let mut trace = Trace::new();
        let mut curr = self.root;

        loop {
            match curr.select(&pin, &self.process) {
                SelectResult::Add(per_child) => {
                    let next_key = per_child.key();
                    curr.try_expand(&pin, per_child);
                    trace.push(&self.process, pin.clone(), curr, next_key);

                    return (trace, ProbeStatus::Expanded)
                },
                SelectResult::Existing(next_key) => {
                    trace.push(&self.process, pin.clone(), curr, next_key);

                    if let Some(next_curr) = curr.edge(&pin, next_key).and_then(|edge| edge.ptr()) {
                        curr = next_curr;
                    } else {
                        return (trace, ProbeStatus::Busy);
                    }
                },
                SelectResult::None => { return (trace, ProbeStatus::Empty) }
            }
        }
    }

    fn insert(&self, trace: &Trace<'_, P, Node<P>>, new_state: P::State) {
        if let Some(last_step) = trace.steps().last() {
            let new_hash = new_state.hash();
            let transposed_child = new_hash.and_then(|hash| self.transpositions.get(&hash)).map(|entry| entry.value().clone());

            if let Some(transposed_child) = transposed_child {
                let edge = last_step.ptr().edge(last_step.pin(), last_step.key()).unwrap();
                edge.try_insert(transposed_child);
            } else {
                let new_child = SafeNonNull::new(Node::new(new_state));

                if last_step.ptr().edge(last_step.pin(), last_step.key()).map(|edge| edge.try_insert(new_child.clone())).unwrap_or(false) {
                    if let Some(hash) = new_hash {
                        self.transpositions.insert(hash, new_child);
                    }
                } else {
                    new_child.drop();
                }
            }
        }
    }

    pub fn update(&self, trace: Trace<'_, P, Node<P>>, state: Option<P::State>, up: P::Update) {
        if let Some(new_state) = state {
            self.insert(&trace, new_state)
        }

        for step in trace.steps() {
            let node = &step.ptr();
            let edge = node.edge(step.pin(), step.key()).unwrap();

            self.process.update(node.state(), edge.per_child(), &up, edge.ptr().is_some());
        }
    }
}
