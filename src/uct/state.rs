pub struct State {
    visits: usize
}

impl State {
    pub fn new() -> Self {
        Self {
            visits: 0
        }
    }

    pub fn visits(&self) -> usize {
        self.visits
    }

    pub fn update(&mut self) {
        self.visits += 1;
    }

    pub fn baseline(&self) -> f32 {
        (2.0 * (self.visits as f32).ln()).sqrt()
    }
}
