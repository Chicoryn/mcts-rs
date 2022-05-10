use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{process::Process, PerChild};

/// Reserved value for the `ptr` field that indicates that this edge has not
/// yet been expanded.
pub const EXPANDING: usize = usize::MAX;

pub struct Edge<P: Process> {
    ptr: AtomicUsize,
    per_child: P::PerChild
}

impl<P: Process> Edge<P> {
    /// Returns an unexpanded edge with the given `per_child`.
    ///
    /// # Arguments
    ///
    /// * `per_child` -
    ///
    pub fn new(per_child: P::PerChild) -> Self {
        Self {
            ptr: AtomicUsize::new(EXPANDING),
            per_child
        }
    }

    /// Returns the pointer to the destination node in the slab.
    pub fn ptr(&self) -> usize {
        self.ptr.load(Ordering::Relaxed)
    }

    /// Returns a reference to the `per_child` of this edge.
    pub fn per_child(&self) -> &P::PerChild {
        &self.per_child
    }

    pub fn key(&self) -> <<P as Process>::PerChild as PerChild>::Key {
        self.per_child().key()
    }

    /// Returns is this edge has a destination node.
    pub fn is_valid(&self) -> bool {
        self.ptr() != EXPANDING
    }

    /// Set the destination node of this edge to the given `new_ptr` if this
    /// edge does not have a destination. Return true iff this edge was updated.
    ///
    /// # Arguments
    ///
    /// * `new_ptr` -
    ///
    pub fn try_insert(&self, new_ptr: usize) -> bool {
        self.ptr.compare_exchange_weak(EXPANDING, new_ptr, Ordering::AcqRel, Ordering::Relaxed) == Ok(EXPANDING)
    }
}
