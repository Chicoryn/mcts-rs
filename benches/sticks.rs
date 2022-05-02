use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mcts_rs::{uct, PerChild, Process, Mcts, SelectResult, State};
use ordered_float::OrderedFloat;
use rand::{prelude::SliceRandom, Rng, thread_rng};
use smallvec::SmallVec;
use std::{ops::Deref, sync::{Arc, Barrier}, thread, collections::hash_map::DefaultHasher, hash::Hasher};

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

    fn hash(&self) -> u64 {
        self.num_remaining as u64
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

impl State for SticksState {
    fn hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(self.sticks.hash());
        hasher.write_i8(self.side);
        Some(hasher.finish())
    }
}

impl SticksState {
    fn new(side: i8, sticks: Sticks) -> Self {
        Self {
            sticks,
            uct: uct::State::new(),
            side
        }
    }

    fn forward(&self, edge: &SticksPerChild) -> Self {
        let side = -self.side;
        let uct = uct::State::new();
        let sticks = self.sticks.play(edge.num_taken);

        Self { sticks, uct, side }
    }

    fn evaluate(&self) -> SticksUpdate {
        self.sticks.evaluate(self.side, &mut thread_rng())
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
        let total_visits = state.uct.visits();
        let best_edge = edges.max_by_key(|per_child| {
            unexplored_moves.retain(|&mut n| n != per_child.num_taken);
            OrderedFloat(per_child.uct.uct(total_visits))
        });

        if let Some(best_edge) = best_edge {
            if unexplored_moves.is_empty() || best_edge.uct.uct(total_visits) > state.uct.baseline() {
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

fn inner_search(search_tree: &Mcts<SticksProcess>, limit: u32, _: usize) {
    while search_tree.root().uct.visits() < limit {
        match search_tree.probe() {
            (trace, _) if trace.is_empty() => { panic!() },
            (trace, _) => {
                let last_step = trace.steps().last().unwrap();
                let (next_state, total_visits) = last_step.map(|state, per_child| {
                    let next_state = state.forward(per_child);

                    (next_state, state.uct.visits())
                });
                let update = next_state.evaluate();

                if total_visits > 8 {
                    search_tree.update(trace, Some(next_state), update);
                } else {
                    search_tree.update(trace, None, update);
                }
            }
        }
    }
}

pub fn search(num_threads: usize, limit: u32) -> Mcts<SticksProcess> {
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

fn sticks_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sticks");

    group.bench_function("1 100", |b| b.iter(|| search(1, black_box(100))));
    group.bench_function("1 1000", |b| b.iter(|| search(1, black_box(1000))));
    group.bench_function("1 10000", |b| b.iter(|| search(1, black_box(10000))));
    group.bench_function("2 10000", |b| b.iter(|| search(2, black_box(10000))));
    group.bench_function("4 10000", |b| b.iter(|| search(4, black_box(10000))));
}

criterion_group!(benches, sticks_benchmark);
criterion_main!(benches);
