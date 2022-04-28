use std::sync::atomic::{AtomicUsize, Ordering, AtomicU32};

use super::{
    state::State,
    update::Update
};

pub struct PerChild {
    total_value: AtomicU32,
    visits: AtomicUsize
}

impl Clone for PerChild {
    fn clone(&self) -> Self {
        Self {
            total_value: AtomicU32::new(self.total_value.load(Ordering::Relaxed)),
            visits: AtomicUsize::new(self.visits())
        }
    }
}

impl PartialEq for PerChild {
    fn eq(&self, rhs: &Self) -> bool {
        self.total_value() == rhs.total_value() &&
            self.visits() == rhs.visits()
    }
}

impl PerChild {
    pub fn new() -> Self {
        Self {
            total_value: AtomicU32::new(0),
            visits: AtomicUsize::new(0)
        }
    }

    pub fn update(&self, up: &Update) {
        self.total_value.fetch_update(Ordering::AcqRel, Ordering::Acquire, |prev_value| {
            Some((f32::from_bits(prev_value) + up.value()).to_bits())
        }).unwrap();
        self.visits.fetch_add(1, Ordering::Acquire);
    }

    fn total_value(&self) -> f32 {
        f32::from_bits(self.total_value.load(Ordering::Relaxed))
    }

    pub fn visits(&self) -> usize {
        self.visits.load(Ordering::Relaxed)
    }

    pub fn win_rate(&self) -> f32 {
        let visits = self.visits();

        if visits > 0 {
            self.total_value() / visits as f32
        } else {
            0.0f32
        }
    }

    pub fn uct(&self, state: &State) -> f32 {
        let ln_n = (state.visits() as f32).ln();

        self.win_rate() + (2.0f32 * ln_n / (self.visits() + 1) as f32).sqrt()
    }
}
