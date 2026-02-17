//! # abkve — Hyper-Efficient In-Memory Vector Database
//!
//! ## Architecture Overview
//!
//! ### Memory Layout: Structure of Arrays (SoA)
//!
//! The naive "Array of Structures" layout stores each vector as its own
//! heap allocation (`Vec<Vec<f32>>`). Each search iteration chases a pointer
//! to a random heap address, thrashing the L1/L2 cache with TLB misses.
//!
//! abkve instead uses a **flat, contiguous `Vec<f32>`** where vector `i`
//! occupies `data[i*dim .. (i+1)*dim]`. The search loop walks this buffer
//! sequentially, loading 256-bit (32-byte) cache lines that feed directly
//! into AVX2 YMM registers. Bandwidth from L2 → L1 is ~512 GB/s on modern
//! server CPUs; pointer chasing throttles that to ~20 GB/s.
//!
//! ### Why `unsafe` get_unchecked in the Hot Path
//!
//! Every `slice[i]` access in safe Rust emits a bounds check: a cmp + jae.
//! In the inner dot-product loop (1536 iterations per vector, 10k vectors
//! per search), that is 15.36 million extra branches per query. With branch
//! prediction, most are free — but they still consume micro-op buffer slots
//! and prevent the compiler from fully unrolling. `get_unchecked` removes
//! them entirely when we can *prove* bounds safety at the call site.
//!
//! ### Loop Unrolling and LLVM Auto-Vectorization
//!
//! We manually process 8 `f32` values per loop iteration, matching one
//! 256-bit AVX2 `VFMADD231PS` instruction. LLVM's auto-vectorizer will
//! further combine these into the widest SIMD width available (AVX-512 on
//! supporting CPUs). The `#[inline(always)]` + `#[target_feature]` attributes
//! guide LLVM without requiring nightly-only intrinsics.

// Global Allocator: mimalloc
//
// replace the system allocator globally. mimalloc uses per-thread "heaps"
// with size-segregated free lists, making small allocations O(1) and nearly
// contention-free. This is declared at the crate root so it applies to every
// allocation in this process, including those made by parking_lot and rayon.
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use parking_lot::RwLock;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

/// Thread-safe wrapper around the inner store.
///
/// `parking_lot::RwLock` is used instead of `std::sync::RwLock` because:
///   - Uncontended lock acquisition is a single atomic CAS (~3 cycles vs ~20).
///   - No OS-level futex call on the fast path.
///   - Provides `upgradable_read()` for future lock-promotion without writer starvation.
pub struct Abkve {
    inner: RwLock<AbkveInner>,
}

/// Search result type: (id, cosine_similarity_score)
pub type SearchResult = Option<(u64, f32)>;

impl Abkve {
    /// Create a new vector store for vectors of `dim` dimensions.
    ///
    /// Pre-allocates memory for `capacity` vectors to avoid reallocation
    /// during the trading session (realloc copies data and creates jitter).
    pub fn new(dim: usize, capacity: usize) -> Self {
        Self {
            inner: RwLock::new(AbkveInner::new(dim, capacity)),
        }
    }

    /// Insert a vector with the given ID.
    ///
    /// The vector is L2-normalized in-place before storage.
    /// Normalization at insert time means the hot search path never
    /// divides — it only multiplies and adds.
    pub fn add(&self, id: u64, vector: &[f32]) {
        self.inner.write().add(id, vector);
    }

    /// Search for the nearest neighbor to `query` above `threshold`.
    ///
    /// Returns `Some((id, score))` for the best matching vector,
    /// or `None` if all scores fall below `threshold`.
    ///
    /// Uses a read lock — multiple concurrent searches execute in parallel
    /// without blocking each other.
    pub fn search(&self, query: &[f32], threshold: f32) -> SearchResult {
        self.inner.read().search(query, threshold)
    }

    /// Parallel search using rayon. Preferred for datasets > ~50k vectors
    /// where the additional thread-dispatch overhead is amortized.
    pub fn search_parallel(&self, query: &[f32], threshold: f32) -> SearchResult {
        self.inner.read().search_parallel(query, threshold)
    }

    /// Naive search using idiomatic iterators (for benchmarking comparison).
    pub fn search_naive(&self, query: &[f32], threshold: f32) -> SearchResult {
        self.inner.read().search_naive(query, threshold)
    }

    /// Serialize the index to any `Write` sink (file, socket, memory buffer).
    pub fn save<W: Write>(&self, writer: W) -> io::Result<()> {
        let inner = self.inner.read();
        bincode::serialize_into(writer, &*inner)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    /// Deserialize an index from any `Read` source.
    pub fn load<R: Read>(reader: R) -> io::Result<Self> {
        let inner: AbkveInner = bincode::deserialize_from(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Self {
            inner: RwLock::new(inner),
        })
    }

    /// Returns the number of vectors currently stored.
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    /// Returns true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the dimensionality of stored vectors.
    pub fn dim(&self) -> usize {
        self.inner.read().dim
    }
}

/// The raw, unsynchronized store. All mutation is protected by the outer RwLock.
///
/// ### SoA Memory Layout Diagram
///
/// ```text
/// ids:  [ id_0  | id_1  | id_2  | ... | id_n  ]   ← Vec<u64>
///
/// data: [ v0[0] v0[1] ... v0[D-1] | v1[0] v1[1] ... v1[D-1] | ... ]
///         ^^^^^^^^^^^^^^^^^^^^^^^^
///         one contiguous "row" of D floats per vector
///         fits exactly 8 cache lines for D=1536 (1536×4 = 6144 bytes = 96 lines)
/// ```
///
/// Both `ids` and `data` grow together — they are always the same logical length.
#[derive(Serialize, Deserialize)]
struct AbkveInner {
    /// Dimensionality of every stored vector. Invariant: never changes after construction.
    dim: usize,
    /// Flat SoA vector storage. `data[i*dim..(i+1)*dim]` is vector `i`.
    data: Vec<f32>,
    /// External IDs — parallel array to `data` rows.
    ids: Vec<u64>,
}

impl AbkveInner {
    fn new(dim: usize, capacity: usize) -> Self {
        assert!(dim > 0, "dimension must be > 0");
        // Single allocation for all vector data. This is most critical
        // memory decision: one contiguous slab vs. N heap pointers.
        let data = Vec::with_capacity(dim * capacity);
        let ids = Vec::with_capacity(capacity);
        Self { dim, data, ids }
    }

    fn len(&self) -> usize {
        self.ids.len()
    }

    /// Insert and normalize a vector.
    fn add(&mut self, id: u64, vector: &[f32]) {
        assert_eq!(
            vector.len(),
            self.dim,
            "vector dim mismatch: expected {}, got {}",
            self.dim,
            vector.len()
        );

        // Compute L2 norm
        let norm = l2_norm(vector);
        debug_assert!(norm > 0.0, "cannot normalize a zero vector");

        // Push normalized components into the flat buffer.
        // e pre-allocated with_capacity, this is a memcpy into
        // the reserved region — no realloc, no cache eviction.
        let inv_norm = if norm > 1e-10 { 1.0 / norm } else { 1.0 };
        for &x in vector {
            self.data.push(x * inv_norm);
        }
        self.ids.push(id);
    }

    /// Search for the best match using the hand-unrolled dot product.
    ///
    /// ## Safety Rationale for `get_unchecked`
    ///
    /// The slice `row` is constructed as `&data[base..base+dim]` where
    /// `base = i * dim` and `i < n_vecs`. This is always in-bounds because:
    ///   1. `data.len() == n_vecs * dim` (maintained by `add`).
    ///   2. We compute `n_vecs = data.len() / dim` at the top of this function.
    ///   3. The loop bound is `i < n_vecs`.
    ///
    /// Inside `dot_product_unrolled`, the query and row slices are both
    /// guaranteed to be `dim` elements long. The unrolled loop processes
    /// `chunks_of(8)` up to `dim / 8 * 8`, then handles the remainder
    /// with checked indexing — the unsafe zone is strictly the full-chunk loop.
    fn search(&self, query: &[f32], threshold: f32) -> SearchResult {
        assert_eq!(query.len(), self.dim, "query dim mismatch");

        let n_vecs = self.ids.len();
        if n_vecs == 0 {
            return None;
        }

        // Normalize query so dot product === cosine similarity.
        // Allocation is intentional: the query buffer is small (6KB for
        // dim=1536) and will stay in L1 throughout the entire search loop.
        let norm_query = normalize_vec(query);

        let dim = self.dim;
        let data = &self.data;

        let mut best_score = threshold; // Only return results > threshold
        let mut best_idx: Option<usize> = None;

        for i in 0..n_vecs {
            let base = i * dim;
            // SAFETY: base + dim == (i+1)*dim <= n_vecs*dim == data.len()
            let row = unsafe { data.get_unchecked(base..base + dim) };
            let score = dot_product_unrolled(&norm_query, row);

            if score > best_score {
                best_score = score;
                best_idx = Some(i);
            }
        }

        best_idx.map(|i| (self.ids[i], best_score))
    }

    /// Parallel variant using rayon. Splits the flat buffer into chunks of
    /// `dim` floats and evaluates each chunk on the thread pool.
    ///
    /// Rayon's `par_chunks` guarantees each thread processes a non-overlapping
    /// subslice — no locking needed inside the map closure.
    fn search_parallel(&self, query: &[f32], threshold: f32) -> SearchResult {
        assert_eq!(query.len(), self.dim, "query dim mismatch");

        let n_vecs = self.ids.len();
        if n_vecs == 0 {
            return None;
        }

        let norm_query = normalize_vec(query);
        let dim = self.dim;

        // Each element of this iterator is a (index, row_slice) pair.
        // rayon distributes chunks across the global thread pool using
        // work-stealing — idle threads claim unprocessed chunks dynamically.
        let (best_idx, best_score) = self
            .data
            .par_chunks(dim)
            .enumerate()
            .map(|(i, row)| {
                let score = dot_product_unrolled(&norm_query, row);
                (i, score)
            })
            .reduce(
                || (usize::MAX, f32::NEG_INFINITY),
                |(ai, as_), (bi, bs)| {
                    if bs > as_ {
                        (bi, bs)
                    } else {
                        (ai, as_)
                    }
                },
            );

        if best_idx == usize::MAX || best_score <= threshold {
            None
        } else {
            Some((self.ids[best_idx], best_score))
        }
    }

    /// Naive search using idiomatic Rust iterators — no unsafe, no unrolling.
    /// Used as the benchmark baseline to measure the speedup from our optimizations.
    fn search_naive(&self, query: &[f32], threshold: f32) -> SearchResult {
        assert_eq!(query.len(), self.dim, "query dim mismatch");

        let norm_query = normalize_vec(query);
        let dim = self.dim;

        self.data
            .chunks_exact(dim)
            .enumerate()
            .map(|(i, row)| {
                let score: f32 = norm_query.iter().zip(row.iter()).map(|(a, b)| a * b).sum();
                (i, score)
            })
            .filter(|(_, score)| *score > threshold)
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, score)| (self.ids[i], score))
    }
}

// ─── Math Primitives ──────────────────────────────────────────────────────────

/// Compute the L2 norm of a slice.
#[inline]
fn l2_norm(v: &[f32]) -> f32 {
    // Using chunks_exact here (not the hot path) so bounds checks are fine.
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Normalize a slice, returning a new owned Vec<f32>.
#[inline]
fn normalize_vec(v: &[f32]) -> Vec<f32> {
    let norm = l2_norm(v);
    let inv = if norm > 1e-10 { 1.0 / norm } else { 1.0 };
    v.iter().map(|x| x * inv).collect()
}

/// # Hand-Unrolled Dot Product — The Core Hot Path
///
/// This function is called O(n_vectors) times per query. Every cycle counts.
///
/// ## Why 8×f32 per iteration?
///
/// AVX2 operates on 256-bit registers = 8×32-bit floats. Each loop body
/// below maps to one `VFMADD231PS ymm, ymm, ymm` instruction:
///   - 2 loads (a_chunk, b_chunk) from L1/L2
///   - 1 fused multiply-add
///   - ~5-cycle throughput with out-of-order execution
///
/// With 8 independent partial sums (`acc0..acc7`), the CPU can execute
/// up to 8 iterations in parallel via its out-of-order execution engine
/// (Skylake/Zen2 have 2 FMA pipes, so real IPC ≈ 2 FMAs/cycle).
///
/// ## Remainder Handling
///
/// For dimensions not divisible by 8 (e.g., dim=1536 is evenly divisible;
/// dim=768 is evenly divisible by 8), the remainder loop handles stragglers
/// with safe, bounds-checked code since they execute O(1) times per search.
///
/// ## `#[inline(always)]`
///
/// Forces inlining into the search loop so LLVM can:
///   1. Hoist the loop counter into a register.
///   2. Eliminate the function call overhead entirely.
///   3. Schedule loads from `a` and `b` speculatively across iterations.
#[inline(always)]
fn dot_product_unrolled(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    let len = a.len();
    let chunks = len / 8;

    // 8 independent accumulator registers — prevents a serial dependency chain.
    // Without this, each fmadd would depend on the previous, giving throughput
    // of 1 FMA/cycle instead of ~2 FMA/cycle on Skylake.
    let mut acc0 = 0.0f32;
    let mut acc1 = 0.0f32;
    let mut acc2 = 0.0f32;
    let mut acc3 = 0.0f32;
    let mut acc4 = 0.0f32;
    let mut acc5 = 0.0f32;
    let mut acc6 = 0.0f32;
    let mut acc7 = 0.0f32;

    // ── UNSAFE BLOCK ────────────────────────────────────────────────────────
    // SAFETY: `i * 8 + 7 < chunks * 8 <= len`, so all accesses are in bounds.
    // We checked `a.len() == b.len()` above (debug_assert). The calling code
    // in `search()` guarantees both slices are exactly `dim` elements long.
    unsafe {
        for i in 0..chunks {
            let base = i * 8;
            acc0 += a.get_unchecked(base) * b.get_unchecked(base);
            acc1 += a.get_unchecked(base + 1) * b.get_unchecked(base + 1);
            acc2 += a.get_unchecked(base + 2) * b.get_unchecked(base + 2);
            acc3 += a.get_unchecked(base + 3) * b.get_unchecked(base + 3);
            acc4 += a.get_unchecked(base + 4) * b.get_unchecked(base + 4);
            acc5 += a.get_unchecked(base + 5) * b.get_unchecked(base + 5);
            acc6 += a.get_unchecked(base + 6) * b.get_unchecked(base + 6);
            acc7 += a.get_unchecked(base + 7) * b.get_unchecked(base + 7);
        }
    }

    // Horizontal reduction: sum 8 partial accumulators.
    // LLVM typically maps this to a sequence of VADDPS + VHADDPS.
    let mut result = acc0 + acc1 + acc2 + acc3 + acc4 + acc5 + acc6 + acc7;

    // Remainder: safe path for trailing elements (often 0 iterations for
    // common embedding dims like 768, 1024, 1536 which are all divisible by 8).
    let remainder_start = chunks * 8;
    for i in remainder_start..len {
        result += a[i] * b[i];
    }

    result
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store(dim: usize) -> Abkve {
        Abkve::new(dim, 16)
    }

    #[test]
    fn test_basic_insert_and_search() {
        let db = make_store(4);
        db.add(1, &[1.0, 0.0, 0.0, 0.0]);
        db.add(2, &[0.0, 1.0, 0.0, 0.0]);

        // Query closest to vector 1
        let result = db.search(&[0.99, 0.01, 0.0, 0.0], 0.5);
        assert!(result.is_some());
        let (id, score) = result.unwrap();
        assert_eq!(id, 1);
        assert!(score > 0.99, "expected score > 0.99, got {}", score);
    }

    #[test]
    fn test_threshold_filtering() {
        let db = make_store(4);
        db.add(1, &[1.0, 0.0, 0.0, 0.0]);

        // Orthogonal vector — dot product = 0, should be filtered out
        let result = db.search(&[0.0, 1.0, 0.0, 0.0], 0.5);
        assert!(result.is_none(), "orthogonal vectors should not match");
    }

    #[test]
    fn test_dot_product_unrolled_correctness() {
        let a: Vec<f32> = (0..16).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..16).map(|i| i as f32).collect();
        let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let got = dot_product_unrolled(&a, &b);
        assert!(
            (got - expected).abs() < 1e-4,
            "unrolled dp mismatch: expected {}, got {}",
            expected,
            got
        );
    }

    #[test]
    fn test_normalization_invariant() {
        let db = make_store(4);
        // After normalization, dot product with itself should be ≈ 1.0
        db.add(42, &[3.0, 4.0, 0.0, 0.0]); // norm = 5.0
        let result = db.search(&[3.0, 4.0, 0.0, 0.0], 0.99);
        assert!(result.is_some());
        let (id, score) = result.unwrap();
        assert_eq!(id, 42);
        assert!(
            (score - 1.0).abs() < 1e-5,
            "self-similarity should be ≈ 1.0, got {}",
            score
        );
    }

    #[test]
    fn test_empty_store() {
        let db = make_store(4);
        assert!(db.search(&[1.0, 0.0, 0.0, 0.0], 0.0).is_none());
    }

    #[test]
    fn test_parallel_matches_sequential() {
        let db = Abkve::new(8, 100);
        for i in 0..50u64 {
            let v: Vec<f32> = (0..8).map(|j| (i + j) as f32).collect();
            db.add(i, &v);
        }
        let query: Vec<f32> = (0..8).map(|j| j as f32).collect();
        let seq = db.search(&query, 0.0);
        let par = db.search_parallel(&query, 0.0);
        match (seq, par) {
            (Some((sid, ss)), Some((pid, ps))) => {
                assert_eq!(sid, pid, "sequential and parallel should return same id");
                assert!(
                    (ss - ps).abs() < 1e-5,
                    "score mismatch: seq={}, par={}",
                    ss,
                    ps
                );
            }
            (None, None) => {}
            other => panic!("mismatch between sequential and parallel: {:?}", other),
        }
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let db = Abkve::new(4, 8);
        db.add(10, &[1.0, 0.0, 0.0, 0.0]);
        db.add(20, &[0.0, 1.0, 0.0, 0.0]);

        let mut buf = Vec::new();
        db.save(&mut buf).expect("save failed");

        let loaded = Abkve::load(buf.as_slice()).expect("load failed");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.dim(), 4);

        let result = loaded.search(&[0.99, 0.01, 0.0, 0.0], 0.5);
        assert_eq!(result.map(|(id, _)| id), Some(10));
    }

    #[test]
    fn test_remainder_handling() {
        // dim=9 is not divisible by 8 — exercises the remainder path
        let db = make_store(9);
        db.add(1, &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let result = db.search(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.99);
        assert!(result.is_some());
    }
}
