use smallvec::SmallVec;

use crate::{
    process::Process,
    step::Step,
};

pub struct Trace<'a, P: Process> {
    steps: SmallVec<[Step<'a, P>; 8]>
}

impl<'a, P: Process> Trace<'a, P> {
    /// Returns a new trace with the given `steps` and `status`.
    pub fn new(steps: SmallVec<[Step<'a, P>; 8]>) -> Self {
        Self { steps }
    }

    /// Returns if there are no steps in this trace.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Returns the steps in this trace.
    pub fn steps(&self) -> &[Step<'a, P>] {
        &self.steps
    }
}

#[cfg(test)]
mod tests {
    use smallvec::smallvec;
    use crate::FakeProcess;
    use super::*;

    #[test]
    fn check_empty() {
        assert!(Trace::<FakeProcess>::new(smallvec! []).is_empty());
    }
}
