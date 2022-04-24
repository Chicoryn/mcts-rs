use smallvec::SmallVec;

use crate::{
    probe_status::ProbeStatus,
    step::Step,
};

pub struct Trace {
    steps: SmallVec<[Step; 8]>,
    status: ProbeStatus
}

impl Trace {
    /// Returns a new trace with the given `steps` and `status`.
    pub fn new(steps: SmallVec<[Step; 8]>, status: ProbeStatus) -> Self {
        Self { steps, status }
    }

    /// Returns if there are no steps in this trace.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Returns the steps in this trace.
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Returns the status of the last step in this trace.
    pub fn status(&self) -> ProbeStatus {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use smallvec::smallvec;
    use super::*;

    #[test]
    fn check_empty() {
        assert!(Trace::new(smallvec! [], ProbeStatus::Success).is_empty());
    }

    #[test]
    fn check_non_empty() {
        assert!(!Trace::new(smallvec! [Step::new(0, 0)], ProbeStatus::Success).is_empty());
    }

    #[test]
    fn check_status() {
        assert_eq!(Trace::new(smallvec! [], ProbeStatus::AlreadyExpanding).status(), ProbeStatus::AlreadyExpanding);
    }
}
