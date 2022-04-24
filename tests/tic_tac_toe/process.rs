use super::{TicTacToeState, TicTacToePerChild, TicTacToeUpdate};
use mcts_rs::Process;

pub struct TicTacToeProcess;

impl TicTacToeProcess {
    pub fn new() -> Self {
        Self { }
    }
}

impl Process for TicTacToeProcess {
    type State = TicTacToeState;
    type PerChild = TicTacToePerChild;
    type Update = TicTacToeUpdate;

    fn best(&self, _: &Self::State, edges: impl Iterator<Item=Self::PerChild>) -> Option<Self::PerChild> {
        edges.max_by_key(|edge| edge.visits())
    }

    fn select(&self, state: &Self::State, edges: impl Iterator<Item=Self::PerChild>) -> Option<Self::PerChild> {
        let mut occupied = [false; 9];
        let best_edge = edges.max_by_key(|edge| {
            occupied[edge.vertex()] = true;
            edge.uct(state.visits())
        });
        let zero_edge = (0..9)
            .filter(|&vertex| !occupied[vertex] && state.is_valid(vertex))
            .next()
            .map(|vertex| TicTacToePerChild::new(0.0, vertex));

        match (best_edge, zero_edge) {
            (None, None) => None,
            (None, Some(zero)) => Some(zero),
            (Some(best), None) => Some(best),
            (Some(best), Some(zero)) => {
                if zero.uct(state.visits()) > best.uct(state.visits()) {
                    Some(zero)
                } else {
                    Some(best)
                }
            }
        }
    }

    fn update(&self, state: &mut Self::State, per_child: &mut Self::PerChild, update: &Self::Update, _: bool) {
        state.update(update);
        per_child.update(state, update);
    }
}

impl TicTacToeProcess {

}