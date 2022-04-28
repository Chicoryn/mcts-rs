use std::sync::atomic::{AtomicUsize, Ordering};

pub struct State {
    visits: AtomicUsize
}

impl State {
    pub fn new() -> Self {
        Self {
            visits: AtomicUsize::new(0)
        }
    }

    pub fn visits(&self) -> usize {
        self.visits.load(Ordering::Relaxed)
    }

    pub fn update(&self) {
        self.visits.fetch_add(1, Ordering::Acquire);
    }

    pub fn baseline(&self) -> f32 {
        (2.0 * (self.visits() as f32).ln()).sqrt()
    }
}
