use rand::{
    rngs::StdRng,
    SeedableRng
};
use mcts_rs::{Mcts, ProbeStatus};

use super::*;

pub fn assert_search(
    process: TicTacToeProcess,
    starting_point: TicTacToeState,
    mut until: impl FnMut(&Mcts<TicTacToeProcess>) -> bool
) -> Mcts<TicTacToeProcess>
{
    let mut prng = StdRng::seed_from_u64(0x12345678);
    let mut search_tree = Mcts::new(process, starting_point);

    while search_tree.root().visits() < 200 || !until(&search_tree) {
        match search_tree.probe().and_then(|trace| {
            if let Some(last_step) = trace.steps().last() {
                let (state, per_child) = last_step.as_state(&mut search_tree);

                Ok((trace, (state, per_child)))
            } else {
                Err(ProbeStatus::NoChildren)
            }
        }) {
            Ok((trace, (state, per_child))) if !state.is_terminal() && state.visits() >= 8 => {
                let state = {
                    let mut board = state.board().clone();
                    let current_turn = state.turn();
                    let vertex = per_child.vertex();

                    board.place(vertex, current_turn);
                    TicTacToeState::new(board, -current_turn)
                };

                search_tree.update(trace, Some(state), TicTacToeUpdate::new(
                    state.evaluate(&mut prng),
                    state.turn()
                ));
            },
            Ok((trace, (state, _))) => {
                let update = TicTacToeUpdate::new(
                    state.evaluate(&mut prng),
                    state.turn()
                );

                search_tree.update(trace, None, update)
            },
            Err(_) => { panic!() }
        }
    }

    search_tree
}
