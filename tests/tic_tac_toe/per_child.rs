use super::{TicTacToeState, TicTacToeUpdate};

#[derive(Clone, Debug)]
pub struct TicTacToePerChild {
    value: f32,
    visits: usize,
    vertex: usize,
}

impl PartialEq for TicTacToePerChild {
    fn eq(&self, other: &Self) -> bool {
        self.vertex == other.vertex
    }
}

impl TicTacToePerChild {
    pub fn new(value: f32, vertex: usize) -> Self {
        Self {
            value,
            visits: 0,
            vertex
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn vertex(&self) -> usize {
        self.vertex
    }

    pub fn update(&mut self, state: &TicTacToeState, update: &TicTacToeUpdate) {
        let value = update.value(state.turn());

        self.visits += 1;
        self.value += (value - self.value) / self.visits as f32;
    }

    fn quantify(x: f32) -> u32 {
        (u16::MAX as f32 * x) as u32
    }

    fn inner_uct(x: f32, visits: usize, total_visits: usize) -> f32 {
        const C: f32 = 1.41421356237; // 2.0f32.sqrt()

        x + C * ((total_visits as f32).ln() / (visits + 1) as f32).sqrt()
    }

    pub fn uct(&self, total_visits: usize) -> u32 {
        Self::quantify(Self::inner_uct(self.value, self.visits, total_visits))
    }

    pub fn visits(&self) -> usize {
        self.visits
    }
}
