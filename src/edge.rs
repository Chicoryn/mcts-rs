use crate::process::Process;
use std::marker::PhantomData;

/// Reserved value for the `ptr` field that indicates that this edge has not
/// yet been expanded.
pub const EXPANDING: usize = usize::MAX;

pub struct Edge<P: Process> {
    ptr: usize,
    per_child: usize,
    _phantom: PhantomData<P>
}

impl<P: Process> Clone for Edge<P> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            per_child: self.per_child,
            _phantom: PhantomData::default(),
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
    pub fn new(per_child: usize) -> Self {
        Self {
            ptr: EXPANDING,
            _phantom: PhantomData::default(),
            per_child
        }
    }

    /// Returns the pointer to the destination node in the slab.
    pub fn ptr(&self) -> usize {
        self.ptr
    }

    /// Returns is this edge has a destination node.
    pub fn is_valid(&self) -> bool {
        self.ptr != EXPANDING
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
            self.ptr = new_ptr;
            return true;
        }
    }

    /// Returns a reference to the `per_child` of this edge.
    pub fn per_child(&self) -> usize {
        self.per_child
    }
}

#[cfg(test)]
mod tests {
    use crate::FakeProcess;
    use super::*;

    #[test]
    fn check_valid() {
        let mut edge = Edge::<FakeProcess>::new(1);

        assert_eq!(edge.is_valid(), false);
        assert_eq!(edge.try_insert(1), true);
        assert_eq!(edge.ptr(), 1);
        assert_eq!(edge.is_valid(), true);
    }

    #[test]
    fn check_double_insert() {
        let mut edge = Edge::<FakeProcess>::new(1);

        assert_eq!(edge.try_insert(1), true);
        assert_eq!(edge.try_insert(1), false);
    }

    #[test]
    fn check_per_child() {
        let edge = Edge::<FakeProcess>::new(1);

        assert_eq!(edge.per_child(), 1);
    }
}
