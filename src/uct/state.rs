use std::sync::atomic::{AtomicU32, Ordering};

pub struct State {
    visits: AtomicU32
}

impl State {
    pub fn new() -> Self {
        Self {
            visits: AtomicU32::new(0)
        }
    }

    pub fn visits(&self) -> u32 {
        self.visits.load(Ordering::Relaxed)
    }

    pub fn update(&self) {
        self.visits.fetch_add(1, Ordering::AcqRel);
    }

    pub fn baseline(total_visits: u32) -> f32 {
        (2.0 * (total_visits as f32).ln()).sqrt()
    }
}
