use rand::{
    prelude::SliceRandom,
    Rng
};
use smallvec::SmallVec;
use mcts_rs::{uct, State};

use super::{TicTacToe, TicTacToeUpdate};

pub struct TicTacToeState {
    board: TicTacToe,
    current_turn: i8,
    uct: uct::State
}

impl State for TicTacToeState {
    fn hash(&self) -> Option<u64> {
        Some(self.board.hash())
    }
}

impl TicTacToeState {
    #[allow(unused)]
    pub fn starting_point() -> Self {
        Self::new(TicTacToe::empty(), 1)
    }

    pub fn new(board: TicTacToe, current_turn: i8) -> Self {
        Self {
            board, current_turn,
            uct: uct::State::new()
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.board.is_over()
    }

    pub fn is_valid(&self, vertex: usize) -> bool {
        self.board.is_valid(vertex)
    }

    pub fn board(&self) -> &TicTacToe {
        &self.board
    }

    pub fn turn(&self) -> i8 {
        self.current_turn
    }

    pub fn visits(&self) -> usize {
        self.uct.visits()
    }

    pub fn uct(&self) -> &uct::State {
        &self.uct
    }

    pub fn update(&self, _: &TicTacToeUpdate) {
        self.uct.update()
    }

    pub fn evaluate(&self, prng: &mut impl Rng) -> f32 {
        let mut board = self.board.clone();
        let mut current_turn = self.current_turn;

        while !board.won(1) && !board.won(-1) {
            let valid_moves = (0..9)
                .filter(|&vertex| board.is_valid(vertex))
                .collect::<SmallVec<[_; 16]>>();

            if let Some(&vertex) = valid_moves.choose(prng) {
                board.place(vertex, current_turn);
            } else {
                return 0.5
            }

            current_turn = -current_turn;
        }

        if board.won(self.current_turn) {
            1.0
        } else {
            debug_assert!(board.won(-self.current_turn));
            0.0
        }
    }
}
