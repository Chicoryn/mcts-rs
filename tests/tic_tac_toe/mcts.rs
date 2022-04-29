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
    let mut prng = StdRng::seed_from_u64(0xcafed00d);
    let search_tree = Mcts::new(process, starting_point);

    while search_tree.root().visits() < 200 || !until(&search_tree) {
        match search_tree.probe() {
            (trace, ProbeStatus::Empty) if trace.is_empty() => { panic!() },
            (trace, _) => {
                let last_step = trace.steps().last().unwrap();
                let (new_state, is_expandable) = last_step.map(|state, per_child| {
                    let mut board = state.board().clone();
                    board.place(per_child.vertex(), state.turn());

                    (
                        TicTacToeState::new(board, -state.turn()),
                        !state.is_terminal() && state.visits() >= 8
                    )
                });
                let update = TicTacToeUpdate::new(
                    new_state.evaluate(&mut prng),
                    new_state.turn()
                );

                if is_expandable {
                    search_tree.update(trace, Some(new_state), update);
                } else {
                    search_tree.update(trace, None, update);
                }
            }
        }
    }

    search_tree
}
