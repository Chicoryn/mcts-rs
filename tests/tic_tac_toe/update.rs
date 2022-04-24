pub struct TicTacToeUpdate {
    value: f32,
    for_player: i8
}

impl TicTacToeUpdate {
    pub fn new(value: f32, for_player: i8) -> Self {
        Self { value, for_player }
    }

    pub fn value(&self, turn: i8) -> f32 {
        if self.for_player == turn {
            self.value
        } else {
            1.0 - self.value
        }
    }
}
