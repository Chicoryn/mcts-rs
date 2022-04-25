use rand::{
    prelude::{SliceRandom, StdRng},
    Rng, SeedableRng
};
use std::{
    cell::RefCell,
    rc::Rc,
    ops::DerefMut
};
use smallvec::SmallVec;
use mcts_rs::{
    uct,
    Process, Mcts, Trace
};

#[derive(Clone)]
struct Sticks {
    num_remaining: usize
}

impl Sticks {
    fn new() -> Self {
        Self {
            num_remaining: 7
        }
    }

    fn is_over(&self) -> bool {
        self.num_remaining == 0
    }

    fn is_valid(&self, n: usize) -> bool {
        n <= self.num_remaining
    }

    fn valid(&self) -> SmallVec<[usize; 3]> {
        (1..=3).filter(|&n| self.is_valid(n)).collect::<_>()
    }

    fn play(&self, n: usize) -> Self {
        debug_assert!(n >= self.num_remaining);

        Self {
            num_remaining: self.num_remaining.saturating_sub(n)
        }
    }

    fn evaluate(&self, mut side: i8, rng: &mut impl Rng) -> SticksUpdate {
        let mut sticks = self.clone();

        while !sticks.is_over() {
            sticks = match sticks.valid().choose(rng) {
                None => break,
                Some(&n) => sticks.play(n)
            };
            side = -side;
        }

        SticksUpdate::new(side, 0.0)
    }
}

pub struct SticksState {
    sticks: Sticks,
    uct: uct::State,
    side: i8
}

impl SticksState {
    fn new(side: i8, sticks: Sticks) -> Self {
        Self {
            sticks,
            uct: uct::State::new(),
            side
        }
    }

    fn evaluate(&self, rng: &mut impl Rng) -> SticksUpdate {
        self.sticks.evaluate(self.side, rng)
    }
}

#[derive(Clone, PartialEq)]
pub struct SticksPerChild {
    uct: uct::PerChild,
    num_taken: usize
}

impl SticksPerChild {
    fn new(n: usize) -> Self {
        Self {
            uct: uct::PerChild::new(),
            num_taken: n
        }
    }

    fn evaluate(&self, state: &SticksState, rng: &mut impl Rng) -> SticksUpdate {
        state.sticks.play(self.num_taken).evaluate(state.side, rng)
    }
}

pub struct SticksUpdate {
    uct: uct::Update,
    side: i8,
}

impl SticksUpdate {
    fn new(side: i8, value: f32) -> Self {
        Self {
            uct: uct::Update::new(value),
            side
        }
    }
}

pub struct SticksProcess {
    rng: Rc<RefCell<StdRng>>
}

impl Process for SticksProcess {
    type State = SticksState;
    type PerChild = SticksPerChild;
    type Update = SticksUpdate;

    fn best(&self, _: &Self::State, edges: impl Iterator<Item=Self::PerChild>) -> Option<Self::PerChild> {
        edges.max_by_key(|per_child| per_child.uct.visits())
    }

    fn select(&self, state: &Self::State, edges: impl Iterator<Item=Self::PerChild>) -> Option<Self::PerChild> {
        let mut unexplored_moves = state.sticks.valid();
        let best_edge = edges.max_by_key(|per_child| {
            unexplored_moves.retain(|&mut n| n != per_child.num_taken);
            quantify(per_child.uct.uct(&state.uct))
        });

        if let Some(best_edge) = best_edge {
            if unexplored_moves.is_empty() || best_edge.uct.uct(&state.uct) > state.uct.baseline() {
                Some(best_edge)
            } else {
                unexplored_moves.choose(self.rng.borrow_mut().deref_mut()).map(|&n| Self::PerChild::new(n))
            }
        } else {
            unexplored_moves.choose(self.rng.borrow_mut().deref_mut()).map(|&n| Self::PerChild::new(n))
        }
    }

    fn update(&self, state: &mut Self::State, per_child: &mut Self::PerChild, update: &Self::Update, _: bool) {
        state.uct.update();
        per_child.uct.update(&if state.side == update.side {
            update.uct.clone()
        } else {
            uct::Update::new(1.0 - update.uct.value())
        });
    }
}

fn quantify(x: f32) -> u64 {
    (u16::MAX as f32 * x) as u64
}

fn is_expandable(search_tree: &mut Mcts<SticksProcess>, trace: &Trace) -> bool {
    trace.steps().last().map(|s| s.as_state(search_tree).0.uct.visits() > 8).unwrap_or(false)
}

pub fn search(n: usize) -> Mcts<SticksProcess> {
    let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(0xdeadbeef)));
    let mut search_tree = Mcts::new(
        SticksProcess { rng: rng.clone() },
        SticksState::new(1, Sticks::new())
    );

    while search_tree.root().uct.visits() < n {
        match search_tree.probe() {
            Ok(trace) if !trace.is_empty() && is_expandable(&mut search_tree, &trace) => {
                let new_state = {
                    let (state, per_child) = trace.steps().last().map(|s| s.as_state(&mut search_tree)).unwrap();
                    let new_sticks = state.sticks.play(per_child.num_taken);

                    SticksState::new(-state.side, new_sticks)
                };
                let update = new_state.evaluate(rng.borrow_mut().deref_mut());

                search_tree.update(trace, Some(new_state), update);
            },
            Ok(trace) if !trace.is_empty() => {
                let (state, per_child) = trace.steps().last().map(|s| s.as_state(&mut search_tree)).unwrap();
                let update = per_child.evaluate(&state, rng.borrow_mut().deref_mut());

                search_tree.update(trace, None, update);
            },

            Ok(_) => { panic!() }
            Err(_) => { panic!() }
        }
    }

    search_tree
}
