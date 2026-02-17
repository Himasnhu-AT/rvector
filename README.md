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
