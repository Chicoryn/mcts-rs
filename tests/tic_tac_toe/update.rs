use mcts_rs::uct;

pub struct TicTacToeUpdate {
    uct: uct::Update,
    for_player: i8,
}

impl TicTacToeUpdate {
    pub fn new(value: f32, for_player: i8) -> Self {
        Self {
            uct: uct::Update::new(value),
            for_player
        }
    }

    pub fn uct(&self, turn: i8) -> uct::Update {
        if self.for_player == turn {
            self.uct.clone()
        } else {
            uct::Update::new(1.0 - self.uct.value())
        }
    }
}
