use criterion::{black_box, criterion_group, criterion_main, Criterion};
use goban::{pieces::{stones::Stone, util::coord::Point}, rules::{game::Game, Move, Player, EndGame, CHINESE}};
use mcts_rs::{PerChild, Process, SelectResult, Mcts};
use rand::{thread_rng, prelude::*};
use std::{sync::{atomic::{AtomicU64, AtomicU32, Ordering}, Arc, Barrier}, thread};

const MAX_GAME_LENGTH: usize = 162;

fn isqrt(s: u64) -> u64 {
    let mut x0 = s / 2;

    if x0 != 0 {
        let mut x1 = (x0 + s / x0) / 2;

        while x1 < x0 {
            x0 = x1;
            x1 = (x0 + s / x0) / 2;
        }

        x0
    } else {
        s
    }
}

struct GobanEdge {
    point: Point,
    total_value: AtomicU64,
    visits: AtomicU32,
}

impl PerChild for GobanEdge {
    type Key = Point;

    fn key(&self) -> Self::Key {
        self.point
    }
}

impl GobanEdge {
    fn new(point: Point) -> Self {
        let total_value = AtomicU64::new(0);
        let visits = AtomicU32::new(0);

        Self { point, total_value, visits }
    }

    fn visits(&self) -> u32 {
        self.visits.load(Ordering::Relaxed)
    }

    fn value(&self) -> u64 {
        let visits = self.visits() as u64;

        if visits == 0 {
            0
        } else {
            self.total_value.load(Ordering::Relaxed) / visits
        }
    }

    fn uct(&self, total_visits: u32) -> u64 {
        let ln_n = u32::MAX as u64 * (32 - total_visits.leading_zeros()) as u64;

        self.value() + isqrt(2 * ln_n / (self.visits() as u64 + 1))
    }
}

fn uct_baseline(total_visits: u32) -> u64 {
    let ln_n = u32::MAX as u64 * (32 - total_visits.leading_zeros()) as u64;

    isqrt(2 * ln_n)
}

struct GobanState {
    total_visits: AtomicU32,
    goban: Game
}

impl GobanState {
    fn new(goban: Game) -> Self {
        let total_visits = AtomicU32::new(0);

        Self { total_visits, goban }
    }

    fn forward(&self, edge: &GobanEdge) -> Self {
        let total_visits = AtomicU32::new(0);
        let mut goban = self.goban.clone();
        goban.play(Move::Play(edge.point.0, edge.point.1));

        Self { total_visits, goban }
    }

    fn total_visits(&self) -> u32 {
        self.total_visits.load(Ordering::Relaxed)
    }

    fn turn(&self) -> Player {
        self.goban.turn()
    }

    fn choose(&self) -> Option<Point> {
        let color = self.turn().stone_color();

        self.goban.legals()
            .filter(|&coordinates| !self.goban.check_eye(Stone { coordinates, color }))
            .choose(&mut thread_rng())
    }

    fn evaluate(&self) -> u16 {
        let mut count = 0;
        let mut goban = self.goban.clone();

        while count < MAX_GAME_LENGTH && !goban.is_over() {
            if let Some(next_move) = goban.legals().choose(&mut thread_rng()).map(|point| Move::Play(point.0, point.1)) {
                goban.play(next_move);
            } else {
                break
            }

            count += 1;
        }

        match goban.outcome() {
            Some(EndGame::WinnerByScore(winner, _)) => u16::MAX * (winner == self.turn()) as u16,
            Some(EndGame::WinnerByResign(winner)) => u16::MAX * (winner == self.turn()) as u16,
            _ => u16::MAX / 2,
        }
    }
}

struct GobanUpdate {
    player: Player,

    /// Quantized value where `u16::MAX` represents a win for `color`, and `0`
    /// represents a loss.
    value: u16,
}

impl GobanUpdate {
    fn new(player: Player, value: u16) -> Self {
        Self { player, value }
    }
}

struct GobanProcess;

impl GobanProcess {
    fn new() -> Self {
        Self {}
    }
}

impl Process for GobanProcess {
    type PerChild = GobanEdge;
    type State = GobanState;
    type Update = GobanUpdate;

    fn best<'a>(&self, _: &Self::State, edges: impl Iterator<Item=&'a Self::PerChild>) -> Option<<Self::PerChild as PerChild>::Key>
        where Self::PerChild: 'a
    {
        edges.max_by_key(|per_child| per_child.visits()).map(|per_child| per_child.point)
    }

    fn select<'a>(&self, state: &Self::State, edges: impl Iterator<Item=&'a Self::PerChild>) -> SelectResult<Self::PerChild>
        where Self::PerChild: 'a
    {
        let total_visits = state.total_visits();

        if let Some(best_edge) = edges.max_by_key(|edge| edge.uct(total_visits)) {
            if best_edge.uct(total_visits) < uct_baseline(total_visits) {
                match state.choose().map(|point| Self::PerChild::new(point)) {
                    Some(new_edge) => SelectResult::Add(new_edge),
                    None => SelectResult::Existing(best_edge.key())
                }
            } else {
                SelectResult::Existing(best_edge.key())
            }
        } else {
            match state.choose().map(|point| Self::PerChild::new(point)) {
                Some(new_edge) => SelectResult::Add(new_edge),
                None => SelectResult::None
            }
        }
    }

    fn update(&self, state: &Self::State, per_child: &Self::PerChild, update: &Self::Update, _: bool) {
        state.total_visits.fetch_add(1, Ordering::AcqRel);
        per_child.visits.fetch_add(1, Ordering::AcqRel);
        per_child.total_value.fetch_add(
            if update.player == state.turn() { update.value as u64 } else { (u16::MAX - update.value) as u64 },
            Ordering::AcqRel
        );
    }
}

fn inner_search_goban(search_tree: &Mcts<GobanProcess>, n: u32) {
    while search_tree.root().total_visits() < n {
        match search_tree.probe() {
            (trace, _) if trace.is_empty() => { panic!() },
            (trace, _) => {
                let last_step = trace.steps().last().unwrap();
                let (next_state, turn, total_visits) = last_step.map(|state, per_child| {
                    let next_state = state.forward(per_child);

                    (next_state, state.turn(), state.total_visits())
                });

                let update = GobanUpdate::new(turn, next_state.evaluate());

                if total_visits > 32 {
                    search_tree.update(trace, Some(next_state), update);
                } else {
                    search_tree.update(trace, None, update);
                }
            }
        }
    }
}

fn search_goban(num_threads: usize, limit: u32) -> Mcts<GobanProcess> {
    let starting_point = Game::builder()
        .size((9, 9))
        .rule(CHINESE)
        .build()
        .unwrap();

    let barrier = Arc::new(Barrier::new(num_threads));
    let search_tree = Arc::new(Mcts::new(
        GobanProcess::new(),
        GobanState::new(starting_point)
    ));

    let handles = (0..num_threads - 1).map(|_| {
        let barrier = barrier.clone();
        let search_tree = search_tree.clone();

        thread::spawn(move || {
            barrier.wait();
            inner_search_goban(&*search_tree, limit);
        })
    }).collect::<Vec<_>>();

    barrier.wait();
    inner_search_goban(&*search_tree, limit);

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(search_tree).map_err(|_| ()).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("goban 1 1600", |b| b.iter(|| search_goban(1, black_box(1600))));
    c.bench_function("goban 2 1600", |b| b.iter(|| search_goban(2, black_box(1600))));
    c.bench_function("goban 4 1600", |b| b.iter(|| search_goban(4, black_box(1600))));
    c.bench_function("goban 8 1600", |b| b.iter(|| search_goban(8, black_box(1600))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
