mod tic_tac_toe;

#[test]
fn is_draw() {
    tic_tac_toe::assert_search(
        tic_tac_toe::TicTacToeProcess::new(),
        tic_tac_toe::TicTacToeState::starting_point(),
        |mcts| {
            if let Some((_, per_child)) = mcts.path().first() {
                0.48 <= per_child.value() && per_child.value() <= 0.52
            } else {
                return false
            }
        }
    );
}
