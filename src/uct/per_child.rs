use super::{
    state::State,
    update::Update
};

#[derive(Clone, PartialEq)]
pub struct PerChild {
    total_value: f32,
    visits: usize
}

impl PerChild {
    pub fn new() -> Self {
        Self {
            total_value: 0.0,
            visits: 0
        }
    }

    pub fn update(&mut self, up: &Update) {
        self.total_value += up.value();
        self.visits += 1;
    }

    pub fn visits(&self) -> usize {
        self.visits
    }

    pub fn win_rate(&self) -> f32 {
        if self.visits > 0 {
            self.total_value / self.visits as f32
        } else {
            0.0f32
        }
    }

    pub fn uct(&self, state: &State) -> f32 {
        let ln_n = (state.visits() as f32).ln();

        self.win_rate() + (2.0f32 * ln_n / (self.visits + 1) as f32).sqrt()
    }
}
