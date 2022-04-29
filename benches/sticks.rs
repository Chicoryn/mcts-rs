mod sticks_game;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sticks 1 100", |b| b.iter(|| sticks_game::search(1, black_box(100))));
    c.bench_function("sticks 1 1000", |b| b.iter(|| sticks_game::search(1, black_box(1000))));
    c.bench_function("sticks 1 10000", |b| b.iter(|| sticks_game::search(1, black_box(10000))));
    c.bench_function("sticks 1 100000", |b| b.iter(|| sticks_game::search(1, black_box(100000))));
    c.bench_function("sticks 2 100000", |b| b.iter(|| sticks_game::search(2, black_box(100000))));
    c.bench_function("sticks 4 100000", |b| b.iter(|| sticks_game::search(4, black_box(100000))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
