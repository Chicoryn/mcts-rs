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
        self.uct.win_rate(self.uct.total_value(), self.uct.visits())
    }

    pub fn vertex(&self) -> usize {
        self.vertex as usize
    }

    pub fn update(&self, state: &TicTacToeState, update: &TicTacToeUpdate) {
        self.uct.update(&update.uct(state.turn()))
    }

    #[inline(always)]
    pub fn uct(&self, total_visits: u32) -> f32 {
        self.uct.uct(total_visits)
    }

    pub fn visits(&self) -> u32 {
        self.uct.visits()
    }
}
