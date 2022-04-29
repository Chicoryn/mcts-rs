use rand::{
    prelude::{SliceRandom, StdRng},
    Rng, SeedableRng, thread_rng
};
use std::{
    cell::RefCell,
    ops::{DerefMut, Deref},
    rc::Rc,
    sync::{Arc, Barrier},
    thread,
};
use smallvec::SmallVec;
use mcts_rs::{
    uct,
    PerChild, Process, Mcts, Trace, SelectResult
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

#[derive(Clone)]
pub struct SticksPerChild {
    uct: uct::PerChild,
    num_taken: usize
}

impl PerChild for SticksPerChild {
    type Key = usize;

    fn key(&self) -> Self::Key {
        self.num_taken
    }
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

pub struct SticksProcess;

impl SticksProcess {
    fn new() -> Self {
        Self { }
    }
}

impl Process for SticksProcess {
    type State = SticksState;
    type PerChild = SticksPerChild;
    type Update = SticksUpdate;

    fn best<'a>(&self, _: &Self::State, edges: impl Iterator<Item=&'a Self::PerChild>) -> Option<<Self::PerChild as PerChild>::Key> where Self::PerChild: 'a {
        edges.max_by_key(|per_child| per_child.uct.visits()).map(|per_child| per_child.key())
    }

    fn select<'a>(&self, state: &Self::State, edges: impl Iterator<Item=&'a Self::PerChild>) -> SelectResult<Self::PerChild> where Self::PerChild: 'a {
        let mut unexplored_moves = state.sticks.valid();
        let best_edge = edges.max_by_key(|per_child| {
            unexplored_moves.retain(|&mut n| n != per_child.num_taken);
            quantify(per_child.uct.uct(&state.uct))
        });

        if let Some(best_edge) = best_edge {
            if unexplored_moves.is_empty() || best_edge.uct.uct(&state.uct) > state.uct.baseline() {
                SelectResult::Existing(best_edge.key())
            } else {
                unexplored_moves.choose(&mut thread_rng()).map(|&n| Self::PerChild::new(n))
                    .map(|per_child| SelectResult::Add(per_child))
                    .unwrap_or(SelectResult::None)
            }
        } else {
            unexplored_moves.choose(&mut thread_rng()).map(|&n| Self::PerChild::new(n))
                .map(|per_child| SelectResult::Add(per_child))
                .unwrap_or(SelectResult::None)
        }
    }

    fn update(&self, state: &Self::State, per_child: &Self::PerChild, update: &Self::Update, _: bool) {
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

fn is_expandable(trace: &Trace<'_, SticksProcess>) -> bool {
    trace.steps().last().map(|s| s.map(|_, per_child| per_child.uct.visits() > 8)).unwrap_or(false)
}

fn inner_search(search_tree: &Mcts<SticksProcess>, limit: usize, _: usize) {
    let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(0xdeadbeef)));

    while search_tree.root().uct.visits() < limit {
        match search_tree.probe() {
            (trace, _) if !trace.is_empty() && is_expandable(&trace) => {
                let new_state = {
                    trace.steps().last().map(|s| s.map(|state, per_child| {
                        let new_sticks = state.sticks.play(per_child.num_taken);

                        SticksState::new(-state.side, new_sticks)
                    })).unwrap()
                };
                let update = new_state.evaluate(rng.borrow_mut().deref_mut());

                search_tree.update(trace, Some(new_state), update);
            },
            (trace, _) if !trace.is_empty() => {
                let update = trace.steps().last().map(|s| {
                    s.map(|state, per_child| {
                        per_child.evaluate(&state, rng.borrow_mut().deref_mut())
                    })
                }).unwrap();

                search_tree.update(trace, None, update);
            },
            _ => { panic!() }
        }
    }
}

pub fn search(num_threads: usize, limit: usize) -> Mcts<SticksProcess> {
    let barrier = Arc::new(Barrier::new(num_threads));
    let search_tree = Arc::new(Mcts::new(
        SticksProcess::new(),
        SticksState::new(1, Sticks::new())
    ));

    let handles = (0..num_threads - 1).map(|thread_id| {
        let barrier = barrier.clone();
        let search_tree = search_tree.clone();

        thread::spawn(move || {
            barrier.wait();
            inner_search(search_tree.deref(), limit, thread_id);
        })
    }).collect::<Vec<_>>();

    barrier.wait();
    inner_search(search_tree.deref(), limit, num_threads - 1);

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(search_tree).map_err(|_| ()).unwrap()
}
