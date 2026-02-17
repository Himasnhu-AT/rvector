//! Run with:
//! ```bash
//! RUSTFLAGS="-C target-cpu=native" cargo bench --bench benchmark
//! ```

use abkve::Abkve;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::{rngs::StdRng, Rng, SeedableRng};

/// OpenAI text-embedding-ada-002 / text-embedding-3-small output dimension.
const DIM: usize = 1536;
/// Number of vectors in the index for the main benchmarks.
const N_VECS: usize = 10_000;
/// Similarity threshold — only return matches above this score.
const THRESHOLD: f32 = 0.75;
/// Fixed RNG seed for reproducible benchmarks.
const SEED: u64 = 0xDEAD_BEEF_CAFE_BABE;

/// Generate `n` random f32 vectors of dimension `dim`.
/// Using a seeded RNG ensures the benchmark data is identical across runs,
/// making benchmark comparisons statistically valid.
fn generate_random_vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..n)
        .map(|_| (0..dim).map(|_| rng.gen::<f32>() * 2.0 - 1.0).collect())
        .collect()
}

/// Build an `Abkve` instance pre-loaded with `n_vecs` random vectors of `dim`.
fn build_db(n_vecs: usize, dim: usize) -> Abkve {
    let db = Abkve::new(dim, n_vecs);
    let vecs = generate_random_vectors(n_vecs, dim, SEED);
    for (i, v) in vecs.iter().enumerate() {
        db.add(i as u64, v);
    }
    db
}

/// Measures the core `search()` function: the hand-unrolled, `get_unchecked`
/// dot product over all 10,000 × 1536-dim vectors.
///
/// `black_box()` prevents the compiler from:
///   1. Hoisting the entire benchmark out of the loop (since it has no side effects).
///   2. Constant-folding the result away.
///   3. Reordering memory loads speculatively across benchmark iterations.
fn bench_search_optimized(c: &mut Criterion) {
    let db = build_db(N_VECS, DIM);

    let query = generate_random_vectors(1, DIM, SEED + 1).remove(0);

    let mut group = c.benchmark_group("search_optimized");
    group.throughput(Throughput::Elements(N_VECS as u64));

    group.bench_function(
        BenchmarkId::new("unrolled_8x_unsafe", format!("{N_VECS}vecs_dim{DIM}")),
        |b| b.iter(|| black_box(db.search(black_box(&query), black_box(THRESHOLD)))),
    );

    group.finish();
}

/// The idiomatic Rust iterator baseline.
/// This version relies entirely on LLVM's auto-vectorizer with bounds checks
/// intact. Compare against `bench_search_optimized` to quantify the speedup
/// from manual unrolling + unsafe access.
fn bench_search_naive(c: &mut Criterion) {
    let db = build_db(N_VECS, DIM);
    let query = generate_random_vectors(1, DIM, SEED + 1).remove(0);

    let mut group = c.benchmark_group("search_naive");
    group.throughput(Throughput::Elements(N_VECS as u64));

    group.bench_function(
        BenchmarkId::new("iterator_safe", format!("{N_VECS}vecs_dim{DIM}")),
        |b| b.iter(|| black_box(db.search_naive(black_box(&query), black_box(THRESHOLD)))),
    );

    group.finish();
}

/// Rayon-parallelized search — useful when N_VECS > ~50k or on machines with
/// many cores. Below that, thread-dispatch overhead dominates.
fn bench_search_parallel(c: &mut Criterion) {
    let db = build_db(N_VECS, DIM);
    let query = generate_random_vectors(1, DIM, SEED + 1).remove(0);

    let mut group = c.benchmark_group("search_parallel");
    group.throughput(Throughput::Elements(N_VECS as u64));

    group.bench_function(
        BenchmarkId::new("rayon_parallel", format!("{N_VECS}vecs_dim{DIM}")),
        |b| b.iter(|| black_box(db.search_parallel(black_box(&query), black_box(THRESHOLD)))),
    );

    group.finish();
}

/// Measures how latency scales from 100 to 10,000 vectors.
/// Expected: linear scaling — each added vector costs exactly one dot product.
/// Any super-linear behavior indicates cache pressure (working set exceeds L3).
fn bench_scaling(c: &mut Criterion) {
    let query = generate_random_vectors(1, DIM, SEED + 99).remove(0);

    let mut group = c.benchmark_group("scaling_by_n_vecs");
    for n in [100usize, 500, 1_000, 5_000, 10_000] {
        let db = build_db(n, DIM);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _n| {
            b.iter(|| black_box(db.search(black_box(&query), black_box(THRESHOLD))))
        });
    }
    group.finish();
}

/// Measures how fast vectors can be normalized and appended.
/// This bounds the index build rate — important for live-ingestion workloads.
fn bench_add(c: &mut Criterion) {
    let vecs = generate_random_vectors(N_VECS, DIM, SEED);

    let mut group = c.benchmark_group("add_throughput");
    group.throughput(Throughput::Elements(1));

    group.bench_function("add_single_vector", |b| {
        b.iter_batched(
            || Abkve::new(DIM, N_VECS + 1),
            |db| {
                db.add(0, black_box(&vecs[0]));
                db
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_search_optimized,
    bench_search_naive,
    bench_search_parallel,
    bench_scaling,
    bench_add,
);
criterion_main!(benches);
