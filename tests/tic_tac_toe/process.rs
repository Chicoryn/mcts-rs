use super::{TicTacToeState, TicTacToePerChild, TicTacToeUpdate};
use mcts_rs::{PerChild, Process, SelectResult};

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

    fn best<'a>(&self, _: &Self::State, edges: impl Iterator<Item=&'a Self::PerChild>) -> Option<<Self::PerChild as PerChild>::Key> where Self::PerChild: 'a {
        edges.max_by_key(|edge| edge.visits()).map(|edge| edge.key())
    }

    fn select<'a>(&self, state: &Self::State, edges: impl Iterator<Item=&'a Self::PerChild>) -> SelectResult<Self::PerChild> where Self::PerChild: 'a {
        let mut occupied = [false; 9];
        let total_visits = state.visits() as u32;
        let best_edge = edges.max_by_key(|edge| {
            occupied[edge.vertex()] = true;
            edge.uct(total_visits)
        });
        let zero_edge = (0..9)
            .filter(|&vertex| !occupied[vertex] && state.is_valid(vertex))
            .next()
            .map(|vertex| TicTacToePerChild::new(vertex));

        match (best_edge, zero_edge) {
            (None, None) => SelectResult::None,
            (None, Some(zero)) => SelectResult::Add(zero),
            (Some(best), None) => SelectResult::Existing(best.key()),
            (Some(best), Some(zero)) => {
                if zero.uct(total_visits) > best.uct(total_visits) {
                    SelectResult::Add(zero)
                } else {
                    SelectResult::Existing(best.key())
                }
            }
        }
    }

    fn update(&self, state: &Self::State, per_child: &Self::PerChild, update: &Self::Update, _: bool) {
        state.update(update);
        per_child.update(state, update);
    }
}
