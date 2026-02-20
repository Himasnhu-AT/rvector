# abkve

**Hyper-efficient in-memory Vector Database for semantic LLM caching**

Target: sub-microsecond search latency for 10k × 1536-dim vectors on modern server CPUs.

> currently uses: Flat Scan O(N)
> future plans: HNSW Graph O(log N)

## Performance Tuning

### CPU-Level Flags

| Flag | Effect |
|------|--------|
| `-C target-cpu=native` | Enables AVX2, AVX-512, FMA, BMI2 for your specific CPU |
| `-C target-feature=+avx2,+fma` | Explicit AVX2+FMA without full native |

## bench

```bash
RUSTFLAGS="-C target-cpu=native" cargo bench --bench benchmark
```

## Concurrency Model

```md
                    ┌──────────────┐
  Thread A ──read──→│              │
  Thread B ──read──→│  RwLock      │──→  AbkveInner (SoA flat buffer)
  Thread C ──read──→│  (parking_lot)│
                    │              │
  Thread D ──write─→│              │  (exclusive, blocks readers)
                    └──────────────┘
```

Multiple concurrent searches proceed without blocking. A write (insertion) briefly blocks readers. `parking_lot::RwLock` uses a single atomic word for the uncontended fast path — no syscall, ~3 CPU cycles.

---

## File Format

Index files use `bincode` (little-endian binary encoding):

```
[u64 dim][u64 n_vecs][f32 × dim × n_vecs][u64 × n_vecs]
  8B       8B         6144 × n_vecs B      8 × n_vecs B
```

For 10k vectors at dim=1536: ~60 MiB on disk.

## Current Benchmarks (Baseline)

Based on recent load tests (10,000 vectors, 1536 dimensions):

- **Insert Throughput:** ~27,076 vectors/sec
- **Search Latency:** ~260 µs / query (average over 100 queries)
- **Serialization Speed:** ~47 MB/s
- **Deserialization Speed:** ~43 MB/s
- **Disk Footprint:** ~58.67 MiB for 10k vectors

Currently operates using O(N) Flat Scan over SIMD-accelerated flat buffers.

## TODOs & Future Migrations

To achieve our ultimate goal of **nano-second matching** capabilities for the inference layer with an extremely low memory/CPU footprint, the following migrations and improvements are planned:

### 1. Algorithmic Enhancements
- **HNSW (Hierarchical Navigable Small World) Graph:** Migrate from O(N) Flat Scan to O(log N) approximate nearest neighbor (ANN) search. This is critical to drop latency from ~260 µs down to the nano-second range for massive datasets.
- **Product Quantization (PQ) / Scalar Quantization (SQ):** Compress `f32` vectors to `int8` or binary vectors. This will drastically reduce memory footprint (e.g., from 60 MiB down to <15 MiB for 10k vectors) and improve CPU cache hit rates, directly lowering CPU usage.

### 2. Memory & Storage Architecture
- **Memory-Mapped Files (mmap):** Transition from loading full binary indexes into RAM (`bincode`) to utilizing zero-copy `mmap`. This will lower startup times and reduce active heap usage, keeping the database footprint minimal.
- **SIMD Refinements:** Hand-tuned AVX-512 / ARM Neon assembly for distance calculations (Cosine / L2) to maximize throughput per CPU cycle.

### 3. Competitor Benchmarking
To validate our efficiency, we will establish a comprehensive benchmark suite comparing `abkve` against established Vector DBs (e.g., **Qdrant, Milvus Lite, sqlite-vss, LanceDB**). 

**Target Benchmarking Metrics:**
- **P95 / P99 Latency:** Specifically targeting sub-microsecond/nano-second query times.
- **Memory Consumption (RSS):** Verifying our engine requires significantly less RAM compared to Qdrant/Milvus under identical loads.
- **CPU Utilization:** Measuring thread contention and idle CPU cycles per query. Prove that CPU usage remains minimal even under thousands of concurrent queries.
- **Throughput (QPS):** Maximizing concurrent read operations under high load in an inference layer context.

**Goal:** Prove that for embedded inference caching, `abkve` outperforms network-bound or generalized vector databases by eliminating network overhead, context switches, and bloated indexing times.
