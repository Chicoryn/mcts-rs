use super::{TicTacToeState, TicTacToeUpdate};
use mcts_rs::{uct, PerChild};

#[derive(Clone)]
pub struct TicTacToePerChild {
    uct: uct::PerChild,
    vertex: u32
}

impl PerChild for TicTacToePerChild {
    type Key = u32;

    fn key(&self) -> Self::Key {
        self.vertex as u32
    }
}

impl TicTacToePerChild {
    pub fn new(vertex: usize) -> Self {
        Self {
            uct: uct::PerChild::new(),
            vertex: vertex as u32
        }
    }

    pub fn value(&self) -> f32 {
        self.uct.win_rate()
    }

    pub fn vertex(&self) -> usize {
        self.vertex as usize
    }

    pub fn update(&self, state: &TicTacToeState, update: &TicTacToeUpdate) {
        self.uct.update(&update.uct(state.turn()))
    }

    fn quantify(x: f32) -> u32 {
        (u16::MAX as f32 * x) as u32
    }

    #[inline(always)]
    pub fn uct(&self, total_visits: u32) -> u32 {
        Self::quantify(self.uct.uct(total_visits))
    }

    pub fn visits(&self) -> u32 {
        self.uct.visits()
    }
}
