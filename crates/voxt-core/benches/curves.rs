use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use rand::RngExt;
use voxt_core::prelude::{FlatIdx, HilbertIdx, HilbertLinearIdx, MortonIdx};

const BATCH_SIZE: usize = 1000000;

fn generate_random_3xu5_batch() -> Vec<(u8, u8, u8)> {
    let mut rng = rand::rng();
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    for _ in 0..BATCH_SIZE {
        let x = rng.random_range(0..=31);
        let y = rng.random_range(0..=31);
        let z = rng.random_range(0..=31);

        batch.push((x, y, z));
    }
    batch
}

fn generate_random_3xu8_batch() -> Vec<(u8, u8, u8)> {
    let mut rng = rand::rng();
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    for _ in 0..BATCH_SIZE {
        let x = rng.random_range(0..=255);
        let y = rng.random_range(0..=255);
        let z = rng.random_range(0..=255);

        batch.push((x, y, z));
    }
    batch
}

fn bench_randomized_curves(c: &mut Criterion) {
    let mut group = c.benchmark_group("S-F Curves Batch Random (Round Trip)");

    let dataset_3xu5 = generate_random_3xu5_batch();
    let dataset_3xu8 = generate_random_3xu8_batch();

    group.bench_function("Flat-3xi5", |b| {
        b.iter(|| {
            for &(x, y, z) in dataset_3xu5.iter() {
                let f = FlatIdx::encode(x, y, z);
                let res = f.decode();
                black_box(res);
            }
        })
    });
    group.bench_function("Morton-3xi5", |b| {
        b.iter(|| {
            for &(x, y, z) in dataset_3xu5.iter() {
                let m = MortonIdx::encode(x, y, z);
                let res = m.decode();
                black_box(res);
            }
        })
    });
    group.bench_function("Hilbert-Linear-3xi5", |b| {
        b.iter(|| {
            for &(x, y, z) in dataset_3xu5.iter() {
                let h = HilbertLinearIdx::encode(x, y, z);
                let res = h.decode();
                black_box(res);
            }
        })
    });
    group.bench_function("Hilbert-2xi8", |b| {
        b.iter(|| {
            for &(x, _, z) in dataset_3xu8.iter() {
                let h = HilbertIdx::encode(x, z);
                let res = h.decode();
                black_box(res);
            }
        })
    });
    group.finish();
}

criterion_group!(benches, bench_randomized_curves);
criterion_main!(benches);
