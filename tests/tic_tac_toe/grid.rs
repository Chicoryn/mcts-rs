use std::{fmt::{self, Debug, Display, Formatter}, collections::hash_map::DefaultHasher, hash::Hasher};

#[derive(Clone, Copy, Debug)]
pub struct TicTacToe {
    board: [i8; 9]
}

impl Display for TicTacToe {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let f = |x: i8| if x == 0 { '.' } else if x > 0 { 'X' } else { 'O' };

        writeln!(fmt, "{} {} {}", f(self.board[0]), f(self.board[1]), f(self.board[2]))?;
        writeln!(fmt, "{} {} {}", f(self.board[3]), f(self.board[4]), f(self.board[5]))?;
        writeln!(fmt, "{} {} {}", f(self.board[6]), f(self.board[7]), f(self.board[8]))
    }
}

impl TicTacToe {
    pub fn empty() -> Self {
        Self {
            board: [0; 9]
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        for vertex in &self.board {
            hasher.write_i8(*vertex);
        }

        hasher.finish()
    }

    fn check_won(&self, vertices: [usize; 3], turn: i8) -> bool {
        vertices.iter().all(|&vertex| self.board[vertex] == turn)
    }

    pub fn won(&self, turn: i8) -> bool {
        self.check_won([0, 1, 2], turn)
        || self.check_won([3, 4, 5], turn)
        || self.check_won([6, 7, 8], turn)
        || self.check_won([0, 3, 6], turn)
        || self.check_won([1, 4, 7], turn)
        || self.check_won([2, 5, 8], turn)
        || self.check_won([0, 4, 8], turn)
        || self.check_won([2, 4, 6], turn)
    }

    pub fn is_over(&self) -> bool {
        self.won(1) || self.won(-1) || self.board.iter().all(|&x| x != 0)
    }

    pub fn is_valid(&self, index: usize) -> bool {
        self.board[index] == 0
    }

    pub fn place(&mut self, index: usize, player: i8) {
        debug_assert!(self.is_valid(index));
        self.board[index] = player;
    }
}
