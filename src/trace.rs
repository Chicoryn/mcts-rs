use smallvec::SmallVec;

use crate::{
    probe_status::ProbeStatus,
    process::Process,
    step::Step,
};

pub struct Trace<'a, P: Process> {
    steps: SmallVec<[Step<'a, P>; 8]>,
    status: ProbeStatus
}

impl<'a, P: Process> Trace<'a, P> {
    /// Returns a new trace with the given `steps` and `status`.
    pub fn new(steps: SmallVec<[Step<'a, P>; 8]>, status: ProbeStatus) -> Self {
        Self { steps, status }
    }

    /// Returns if there are no steps in this trace.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Returns the steps in this trace.
    pub fn steps(&self) -> &[Step<'a, P>] {
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
    use crate::FakeProcess;
    use super::*;

    #[test]
    fn check_empty() {
        assert!(Trace::<FakeProcess>::new(smallvec! [], ProbeStatus::Success).is_empty());
    }

    #[test]
    fn check_status() {
        assert_eq!(Trace::<FakeProcess>::new(smallvec! [], ProbeStatus::AlreadyExpanding).status(), ProbeStatus::AlreadyExpanding);
    }
}
