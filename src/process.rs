pub trait Process {
    type State;
    type PerChild: Clone + PartialEq;
    type Update;

    /// Returns the _best_ edge to *play* for a given `state` and set of
    /// evaluated `edges`.
    ///
    /// # Arguments
    ///
    /// * `state` -
    /// * `edges` - all explored edges for the given `state`.
    ///
    fn best(&self, state: &Self::State, edges: impl Iterator<Item=Self::PerChild>) -> Option<Self::PerChild>;

    /// Returns the edge to be explored during search for a given `state` and
    /// set of already evaluated `edges`. If `None` is returned then this is
    /// assumed to be terminal state.
    ///
    /// # Arguments
    ///
    /// * `state` -
    /// * `edges` -
    ///
    fn select(&self, state: &Self::State, edges: impl Iterator<Item=Self::PerChild>) -> Option<Self::PerChild>;

    /// Update the statistics for this `state` and `per_child` based on a
    /// user-provided evaluation `update`.
    ///
    /// # Arguments
    ///
    /// * `state` -
    /// * `per_child` -
    /// * `update` -
    /// * `is_expanded` -
    ///
    fn update(&self, state: &mut Self::State, per_child: &mut Self::PerChild, update: &Self::Update, is_expanded: bool);
}

#[cfg(test)]
pub struct FakeProcess;

#[cfg(test)]
impl Process for FakeProcess {
    type State = ();
    type PerChild = usize;
    type Update = ();

    fn best(&self, _: &Self::State, _: impl Iterator<Item=Self::PerChild>) -> Option<Self::PerChild> {
        Some(0)
    }

    fn select(&self, _: &Self::State, _: impl Iterator<Item=Self::PerChild>) -> Option<Self::PerChild> {
        Some(0)
    }

    fn update(&self, _: &mut Self::State, _: &mut Self::PerChild, _: &Self::Update, _: bool) {
        // pass
    }
}
