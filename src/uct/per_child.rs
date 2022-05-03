use std::sync::atomic::{Ordering, AtomicU32};
use super::update::Update;

pub struct PerChild {
    total_value: AtomicU32,
    visits: AtomicU32
}

impl Clone for PerChild {
    fn clone(&self) -> Self {
        Self {
            total_value: AtomicU32::new(self.total_value.load(Ordering::Relaxed)),
            visits: AtomicU32::new(self.visits())
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
            visits: AtomicU32::new(0)
        }
    }

    pub fn update(&self, up: &Update) {
        self.visits.fetch_add(1, Ordering::AcqRel);
        self.total_value.fetch_update(Ordering::AcqRel, Ordering::Acquire, |prev_value| {
            Some((f32::from_bits(prev_value) + up.value()).to_bits())
        }).unwrap();
    }

    #[inline(always)]
    fn total_value(&self) -> f32 {
        f32::from_bits(self.total_value.load(Ordering::Relaxed))
    }

    #[inline(always)]
    pub fn visits(&self) -> u32 {
        self.visits.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn win_rate(&self, visits: u32) -> f32 {
        if visits > 0 {
            self.total_value() / visits as f32
        } else {
            0.0f32
        }
    }

    #[inline(always)]
    pub fn uct(&self, total_visits: u32) -> f32 {
        let ln_n = (total_visits as f32).ln();
        let visits = self.visits();

        self.win_rate(visits) + (2.0f32 * ln_n / (visits + 1) as f32).sqrt()
    }
}
