use crate::{
    process::Process
};

/// Reserved value for the `ptr` field that indicates that this edge has not
/// yet been expanded.
pub const EXPANDING: usize = usize::MAX;

pub struct Edge<P: Process> {
    ptr: usize,
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
            ptr: EXPANDING,
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

    /// Set the destination node of this edge to the given `new_ptr`.
    ///
    /// # Arguments
    ///
    /// * `new_ptr` -
    ///
    pub fn insert(&mut self, new_ptr: usize) {
        self.ptr = new_ptr;
    }

    /// Returns a reference to the `per_child` of this edge.
    pub fn per_child(&self) -> &P::PerChild {
        &self.per_child
    }

    /// Returns a mutable reference to the `per_child` of this edge.
    pub fn per_child_mut(&mut self) -> &mut P::PerChild {
        &mut self.per_child
    }
}

#[cfg(test)]
mod tests {
    use crate::FakeProcess;
    use super::*;

    #[test]
    fn check_valid() {
        let mut edge = Edge::<FakeProcess>::new(37);

        assert_eq!(edge.is_valid(), false);
        edge.insert(1);
        assert_eq!(edge.ptr(), 1);
        assert_eq!(edge.is_valid(), true);
    }

    #[test]
    fn check_per_child() {
        let mut edge = Edge::<FakeProcess>::new(1);

        assert_eq!(*edge.per_child(), 1);
        *edge.per_child_mut() = 2;
        assert_eq!(*edge.per_child(), 2);
    }
}
