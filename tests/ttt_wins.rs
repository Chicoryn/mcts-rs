mod tic_tac_toe;

/// ```
/// . O .
/// X X O
/// . . .
/// ```
///
/// - `X` wins by playing at `a1` or `a3`.
///
#[test]
fn x_wins() {
    let mut board = tic_tac_toe::TicTacToe::empty();
    board.place(1, -1);
    board.place(5, -1);
    board.place(3, 1);
    board.place(4, 1);

    tic_tac_toe::assert_search(
        tic_tac_toe::TicTacToeProcess::new(),
        tic_tac_toe::TicTacToeState::new(board, 1),
        |mcts| {
            if let Some(step) = mcts.path().steps().first() {
                step.map(|_, per_child| {
                    per_child.value() >= 0.98 && (per_child.vertex() == 0 || per_child.vertex() == 6)
                })
            } else {
                return false
            }
        }
    );
}

/// ```
/// . O .
/// X X O
/// . . .
/// ```
///
/// - `O` wins by playing at `c3`.
///
#[test]
fn o_wins() {
    let mut board = tic_tac_toe::TicTacToe::empty();
    board.place(1, -1);
    board.place(5, -1);
    board.place(3, 1);
    board.place(4, 1);

    tic_tac_toe::assert_search(
        tic_tac_toe::TicTacToeProcess::new(),
        tic_tac_toe::TicTacToeState::new(board, -1),
        |mcts| {
            if let Some(step) = mcts.path().steps().first() {
                step.map(|_, per_child| {
                    per_child.value() >= 0.98 && per_child.vertex() == 2
                })
            } else {
                return false
            }
        }
    );
}
