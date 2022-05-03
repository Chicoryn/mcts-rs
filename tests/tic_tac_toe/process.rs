use super::{TicTacToeState, TicTacToePerChild, TicTacToeUpdate};
use mcts_rs::{uct, PerChild, Process, SelectResult};
use ordered_float::OrderedFloat;

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
        let best_edge = edges.max_by_key(|&edge| {
            occupied[edge.vertex()] = true;
            OrderedFloat(edge.uct(total_visits))
        });

        if let Some(best_edge) = best_edge {
            if best_edge.uct(total_visits) > uct::State::baseline(total_visits) {
                SelectResult::Existing(best_edge.key())
            } else {
                (0..9).filter(|&i| !occupied[i] && state.is_valid(i)).next()
                    .map(|i| SelectResult::Add(Self::PerChild::new(i)))
                    .unwrap_or_else(|| SelectResult::Existing(best_edge.key()))
            }
        } else {
            (0..9).filter(|&i| !occupied[i] && state.is_valid(i)).next()
                .map(|i| SelectResult::Add(Self::PerChild::new(i)))
                .unwrap_or(SelectResult::None)
        }
    }

    fn update(&self, state: &Self::State, per_child: &Self::PerChild, update: &Self::Update, _: bool) {
        state.update(update);
        per_child.update(state, update);
    }
}
