use std::ptr::NonNull;

use crate::{process::Process, PerChild};

/// Reserved value for the `ptr` field that indicates that this edge has not
/// yet been expanded.
pub const EXPANDING: usize = usize::MAX;

struct EdgeBox<P: Process> {
    ptr: usize,
    per_child: P::PerChild
}

pub struct Edge<P: Process> {
    ptr: NonNull<EdgeBox<P>>
}

unsafe impl<P: Process> Send for Edge<P> {}
unsafe impl<P: Process> Sync for Edge<P> {}

impl<P: Process> Clone for Edge<P> {
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(self.ptr.as_ptr()) }
        }
    }
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
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(EdgeBox {
                ptr: EXPANDING,
                per_child
            }))) }
        }
    }

    pub fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.ptr.as_ptr())) }
    }

    /// Returns the pointer to the destination node in the slab.
    pub fn ptr(&self) -> usize {
        unsafe { self.ptr.as_ref().ptr }
    }

    /// Returns a reference to the `per_child` of this edge.
    pub fn per_child(&self) -> &P::PerChild {
        unsafe { &self.ptr.as_ref().per_child }
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
    pub fn try_insert(&mut self, new_ptr: usize) -> bool {
        if self.is_valid() {
            return false;
        } else {
            unsafe { self.ptr.as_mut().ptr = new_ptr }
            return true;
        }
    }
}
