pub trait State {
    fn hash(&self) -> Option<u64>;
}

pub trait PerChild {
    type Key: Copy + Ord;

    fn key(&self) -> Self::Key;
}

pub enum SelectResult<P: PerChild> {
    Add(P),
    Existing(P::Key),
    None
}

pub trait Process {
    type State: State;
    type PerChild: PerChild;
    type Update;

    /// Returns the _best_ edge to *play* for a given `state` and set of
    /// evaluated `edges`.
    ///
    /// # Arguments
    ///
    /// * `state` -
    /// * `edges` - all explored edges for the given `state`.
    ///
    fn best<'a>(&self, state: &Self::State, edges: impl Iterator<Item=&'a Self::PerChild>) -> Option<<Self::PerChild as PerChild>::Key> where Self::PerChild: 'a;

    /// Returns the edge to be explored during search for a given `state` and
    /// set of already evaluated `edges`. If `None` is returned then this is
    /// assumed to be terminal state.
    ///
    /// # Arguments
    ///
    /// * `state` -
    /// * `edges` -
    ///
    fn select<'a>(&self, state: &Self::State, edges: impl Iterator<Item=&'a Self::PerChild>) -> SelectResult<Self::PerChild> where Self::PerChild: 'a;

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
    fn update(&self, state: &Self::State, per_child: &Self::PerChild, update: &Self::Update, is_expanded: bool);
}

#[cfg(test)]
#[derive(Clone)]
pub struct FakeState;

#[cfg(test)]
impl State for FakeState {
    fn hash(&self) -> Option<u64> {
        None
    }
}

#[cfg(test)]
#[derive(Clone)]
pub struct FakePerChild {
    key: u32
}

#[cfg(test)]
impl FakePerChild {
    pub fn new(key: u32) -> Self {
        Self { key }
    }
}

#[cfg(test)]
impl PerChild for FakePerChild {
    type Key = u32;

    fn key(&self) -> Self::Key {
        self.key
    }
}

#[cfg(test)]
pub struct FakeProcess;

#[cfg(test)]
impl Process for FakeProcess {
    type State = FakeState;
    type PerChild = FakePerChild;
    type Update = ();

    fn best<'a>(&self, _: &Self::State, _: impl Iterator<Item=&'a Self::PerChild>) -> Option<<Self::PerChild as PerChild>::Key> where Self::PerChild: 'a {
        Some(0)
    }

    fn select<'a>(&self, _: &Self::State, _: impl Iterator<Item=&'a Self::PerChild>) -> SelectResult<Self::PerChild> where Self::PerChild: 'a {
        SelectResult::Add(FakePerChild::new(0))
    }

    fn update(&self, _: &Self::State, _: &Self::PerChild, _: &Self::Update, _: bool) {
        // pass
    }
}
