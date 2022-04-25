mod sticks_game;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sticks 100", |b| b.iter(|| sticks_game::search(black_box(100))));
    c.bench_function("sticks 1000", |b| b.iter(|| sticks_game::search(black_box(1000))));
    c.bench_function("sticks 10000", |b| b.iter(|| sticks_game::search(black_box(10000))));
    c.bench_function("sticks 100000", |b| b.iter(|| sticks_game::search(black_box(100000))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
