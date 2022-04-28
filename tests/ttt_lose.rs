mod tic_tac_toe;

/// ```
/// O . O
/// X O X
/// . . X
/// ```
///
/// - `O` wins by playing at `a1` or `b3`.
///
#[test]
fn x_lose() {
    let mut board = tic_tac_toe::TicTacToe::empty();
    board.place(0, -1);
    board.place(2, -1);
    board.place(4, -1);
    board.place(3, 1);
    board.place(5, 1);
    board.place(8, 1);

    tic_tac_toe::assert_search(
        tic_tac_toe::TicTacToeProcess::new(),
        tic_tac_toe::TicTacToeState::new(board, 1),
        |mcts| {
            if let Some(step) = mcts.path().steps().first() {
                step.map(mcts, |_, per_child| {
                    per_child.value() <= 0.02
                })
            } else {
                return false
            }
        }
    );
}

/// ```
/// X O O
/// O . .
/// . X X
/// ```
///
/// - `X` wins by playing at `a1` or `b2`.
///
#[test]
fn o_lose() {
    let mut board = tic_tac_toe::TicTacToe::empty();
    board.place(1, -1);
    board.place(2, -1);
    board.place(3, -1);
    board.place(0, 1);
    board.place(7, 1);
    board.place(8, 1);

    tic_tac_toe::assert_search(
        tic_tac_toe::TicTacToeProcess::new(),
        tic_tac_toe::TicTacToeState::new(board, -1),
        |mcts| {
            if let Some(step) = mcts.path().steps().first() {
                step.map(mcts, |_, per_child| {
                    per_child.value() <= 0.02
                })
            } else {
                return false
            }
        }
    );
}
