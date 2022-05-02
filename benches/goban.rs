use criterion::{black_box, criterion_group, criterion_main, Criterion};
use goban::{pieces::{stones::Stone, util::coord::Point}, rules::{game::Game, Move, Player, EndGame, CHINESE}};
use mcts_rs::{uct, PerChild, State, Process, SelectResult, Mcts};
use ordered_float::OrderedFloat;
use rand::{thread_rng, prelude::*};
use std::{sync::{Arc, Barrier}, thread};

const MAX_GAME_LENGTH: usize = 162;

struct GobanEdge {
    point: Point,
    uct: uct::PerChild,
}

impl PerChild for GobanEdge {
    type Key = Point;

    fn key(&self) -> Self::Key {
        self.point
    }
}

impl GobanEdge {
    fn new(point: Point) -> Self {
        Self { point, uct: uct::PerChild::new() }
    }

    fn visits(&self) -> u32 {
        self.uct.visits()
    }
}

struct GobanState {
    goban: Game,
    uct: uct::State
}

impl State for GobanState {
    fn hash(&self) -> Option<u64> {
        let hash = *self.goban.last_hash();

        if hash == 0 {
            None
        } else {
            Some(hash)
        }
    }
}

impl GobanState {
    fn new(goban: Game) -> Self {
        Self { goban, uct: uct::State::new() }
    }

    fn forward(&self, edge: &GobanEdge) -> Self {
        let mut goban = self.goban.clone();
        goban.play(Move::Play(edge.point.0, edge.point.1));

        Self { goban, uct: uct::State::new() }
    }

    fn total_visits(&self) -> u32 {
        self.uct.visits()
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

    fn evaluate(&self) -> f32 {
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
            Some(EndGame::WinnerByScore(winner, _)) => (winner == self.turn()) as i32 as f32,
            Some(EndGame::WinnerByResign(winner)) => (winner == self.turn()) as i32 as f32,
            _ => 0.5,
        }
    }
}

struct GobanUpdate {
    player: Player,
    uct: uct::Update
}

impl GobanUpdate {
    fn new(player: Player, value: f32) -> Self {
        Self { player, uct: uct::Update::new(value) }
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
        let total_visits = state.uct.visits();

        if let Some(best_edge) = edges.max_by_key(|edge| OrderedFloat(edge.uct.uct(total_visits))) {
            if best_edge.uct.uct(total_visits) < state.uct.baseline() {
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
        state.uct.update();
        per_child.uct.update(&uct::Update::new(
            if update.player == state.turn() { update.uct.value() } else { 1.0 - update.uct.value() }
        ));
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

fn goban_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("goban");

    group.significance_level(0.1).sample_size(10);
    group.bench_function("1 800", |b| b.iter(|| search_goban(1, black_box(800))));
    group.bench_function("2 800", |b| b.iter(|| search_goban(2, black_box(800))));
    group.bench_function("4 800", |b| b.iter(|| search_goban(4, black_box(800))));
    group.bench_function("8 800", |b| b.iter(|| search_goban(8, black_box(800))));
}

criterion_group!(benches, goban_benchmark);
criterion_main!(benches);
