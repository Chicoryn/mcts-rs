use crate::{process::{Process, PerChild}, safe_nonnull::SafeNonNull};
use std::{sync::atomic::{AtomicPtr, Ordering}, ptr::null_mut};

pub struct Edge<P: Process, Node> {
    ptr: AtomicPtr<Node>,
    per_child: P::PerChild
}

impl<P: Process, Node> Edge<P, Node> {
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
    pub(super) fn ptr(&self) -> Option<SafeNonNull<Node>> {
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
    pub(super) fn try_insert(&self, new_ptr: SafeNonNull<Node>) -> bool {
        self.ptr.compare_exchange_weak(null_mut(), new_ptr.as_ptr(), Ordering::AcqRel, Ordering::Relaxed).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::{FakeProcess, FakePerChild};
    use super::*;

    #[test]
    fn new_set_ptr_to_null() {
        let edge: Edge<FakeProcess, ()> = Edge::new(FakePerChild::new(1));
        assert!(edge.ptr().is_none());
    }

    #[test]
    fn key_gets_the_set_key() {
        for key in 0..10 {
            let edge: Edge<FakeProcess, ()> = Edge::new(FakePerChild::new(key));
            assert_eq!(edge.key(), key);
        }
    }

    #[test]
    fn try_insert_succeeds_when_ptr_is_null() {
        let edge: Edge<FakeProcess, ()> = Edge::new(FakePerChild::new(1));
        let ptr = SafeNonNull::new(());

        assert!(edge.try_insert(ptr));
        assert_eq!(edge.ptr(), Some(ptr));
    }

    #[test]
    fn try_insert_fails_when_ptr_is_not_null() {
        let edge: Edge<FakeProcess, ()> = Edge::new(FakePerChild::new(1));
        let ptr1 = SafeNonNull::new(());
        let ptr2 = SafeNonNull::new(());

        assert!(edge.try_insert(ptr1));
        assert!(!edge.try_insert(ptr2));
        assert_eq!(edge.ptr(), Some(ptr1));
    }
}
