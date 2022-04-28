use rand::{
    rngs::StdRng,
    SeedableRng
};
use mcts_rs::{Mcts, ProbeStatus, Trace};

use super::*;

fn is_expandable(trace: &Trace<'_, TicTacToeProcess>) -> bool {
    let last_step = trace.steps().last().unwrap();

    last_step.map(|state, _| !state.is_terminal() && state.visits() >= 8)
}

pub fn assert_search(
    process: TicTacToeProcess,
    starting_point: TicTacToeState,
    mut until: impl FnMut(&Mcts<TicTacToeProcess>) -> bool
) -> Mcts<TicTacToeProcess>
{
    let mut prng = StdRng::seed_from_u64(0xcafed00d);
    let search_tree = Mcts::new(process, starting_point);

    while search_tree.root().visits() < 200 || !until(&search_tree) {
        match search_tree.probe().and_then(|trace| {
            if !trace.is_empty() {
                Ok(trace)
            } else {
                Err(ProbeStatus::NoChildren)
            }
        }) {
            Ok(trace) if is_expandable(&trace) => {
                let last_step = trace.steps().last().unwrap();
                let (state, update) = last_step.map(|state, per_child| {
                    let state = {
                        let mut board = state.board().clone();
                        let current_turn = state.turn();
                        let vertex = per_child.vertex();

                        board.place(vertex, current_turn);
                        TicTacToeState::new(board, -current_turn)
                    };
                    let update = TicTacToeUpdate::new(
                        state.evaluate(&mut prng),
                        state.turn()
                    );

                    (state, update)
                });

                search_tree.update(trace, Some(state), update);
            },
            Ok(trace) => {
                let last_step = trace.steps().last().unwrap();
                let update = last_step.map(|state, _| {
                    TicTacToeUpdate::new(
                        state.evaluate(&mut prng),
                        state.turn()
                    )
                });

                search_tree.update(trace, None, update)
            },
            Err(_) => { panic!() }
        }
    }

    search_tree
}
