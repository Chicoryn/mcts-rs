use crate::{node::Node, path_iter::PathIter, probe_status::ProbeStatus, process::{State, Process, PerChild, SelectResult}, safe_nonnull::SafeNonNull, step::Step, trace::Trace};
use crossbeam_epoch as epoch;
use dashmap::DashMap;
use std::rc::Rc;

pub struct Mcts<P: Process> {
    root: SafeNonNull<Node<P>>,
    process: P,
    transpositions: DashMap<u64, SafeNonNull<Node<P>>>
}

impl<P: Process> Drop for Mcts<P> {
    fn drop(&mut self) {
        for mut entry in self.transpositions.iter_mut() {
            entry.value_mut().drop();
        }
    }
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
    pub fn path<'a>(&'a self) -> impl Iterator<Item=Step<'a, P, Node<P>>> {
        PathIter::new(&self.process, self.root)
    }

    /// Returns a trace
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
                let mut new_child = SafeNonNull::new(Node::new(new_state));

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
