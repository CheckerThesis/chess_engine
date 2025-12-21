use std::hint::black_box;

use chess_engine_2::{board::Board, fens::KIWIPETE, perft::perft};
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_perft(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft_benchmarks");
    
    group.sample_size(10); 

    group.bench_function("perft_depth_4", |b| {
        b.iter(|| {
            let mut position = Board::new(KIWIPETE);
            black_box(perft(&mut position, black_box(4)))
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_perft);
criterion_main!(benches);