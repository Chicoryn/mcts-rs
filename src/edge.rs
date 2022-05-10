use std::{
    sync::atomic::{AtomicPtr, Ordering},
    ptr::null_mut
};

use crate::{process::Process, PerChild, node::Node, safe_nonnull::SafeNonNull};

pub struct Edge<P: Process> {
    ptr: AtomicPtr<Node<P>>,
    per_child: P::PerChild
}

impl<P: Process> Edge<P> {
    /// Returns an unexpanded edge with the given `per_child`.
    ///
    /// # Arguments
    ///
    /// * `per_child` -
    ///
    pub(super) fn new(per_child: P::PerChild) -> Self {
        Self {
            ptr: AtomicPtr::new(null_mut()),
            per_child
        }
    }

    /// Returns the pointer to the destination node in the slab.
    pub(super) fn ptr(&self) -> Option<SafeNonNull<Node<P>>> {
        let ptr = self.ptr.load(Ordering::Relaxed);

        if ptr.is_null() {
            None
        } else {
            Some(SafeNonNull::from_raw(ptr))
        }
    }

    /// Returns a reference to the `per_child` of this edge.
    pub(super) fn per_child(&self) -> &P::PerChild {
        &self.per_child
    }

    pub(super) fn key(&self) -> <<P as Process>::PerChild as PerChild>::Key {
        self.per_child().key()
    }

    /// Set the destination node of this edge to the given `new_ptr` if this
    /// edge does not have a destination. Return true iff this edge was updated.
    ///
    /// # Arguments
    ///
    /// * `new_ptr` -
    ///
    pub(super) fn try_insert(&self, new_ptr: SafeNonNull<Node<P>>) -> bool {
        self.ptr.compare_exchange_weak(null_mut(), new_ptr.into_raw(), Ordering::AcqRel, Ordering::Relaxed).is_ok()
    }
}
