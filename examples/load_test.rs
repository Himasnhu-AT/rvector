//! ```bash
//! RUSTFLAGS="-C target-cpu=native" cargo run --example load_test --release
//! ```

use abkve::Abkve;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    fs,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

/// OpenAI ada-002 / text-embedding-3-small output dimension
const DIM: usize = 1536;
/// Number of vectors to generate for the load test
const N_VECS: usize = 10_000;
/// Number of queries to run for verification
const N_QUERIES: usize = 100;
/// Similarity threshold for search
const THRESHOLD: f32 = 0.7;
/// Deterministic seed for reproducibility
const SEED: u64 = 42;

fn random_vector(rng: &mut StdRng, dim: usize) -> Vec<f32> {
    (0..dim).map(|_| rng.gen::<f32>() * 2.0 - 1.0).collect()
}

/// Pretty-print a byte count as a human-readable string.
fn fmt_bytes(n: u64) -> String {
    match n {
        b if b < 1024 => format!("{b} B"),
        b if b < 1024 * 1024 => format!("{:.2} KiB", b as f64 / 1024.0),
        b if b < 1024 * 1024 * 1024 => format!("{:.2} MiB", b as f64 / (1024.0 * 1024.0)),
        b => format!("{:.2} GiB", b as f64 / (1024.0 * 1024.0 * 1024.0)),
    }
}

fn divider() {
    println!("{}", "─".repeat(60));
}

fn main() -> anyhow::Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          abkve Load Test & Persistence Verifier          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // ── Phase 1: Data Generation ─────────────────────────────────────────────
    divider();
    println!("Phase 1 — Generating random vectors");
    println!("  Vectors:   {N_VECS}");
    println!("  Dimension: {DIM}");
    println!(
        "  Raw data:  {} (uncompressed, f32)",
        fmt_bytes((N_VECS * DIM * 4) as u64)
    );

    let t0 = Instant::now();
    let mut rng = StdRng::seed_from_u64(SEED);
    let vectors: Vec<Vec<f32>> = (0..N_VECS).map(|_| random_vector(&mut rng, DIM)).collect();
    println!("  Generated in {:?}", t0.elapsed());

    // ── Phase 2: Insertion ────────────────────────────────────────────────────
    divider();
    println!("Phase 2 — Inserting into Abkve (with L2 normalization)");

    let t0 = Instant::now();
    let db = Abkve::new(DIM, N_VECS);
    for (i, v) in vectors.iter().enumerate() {
        db.add(i as u64, v);
    }
    let insert_duration = t0.elapsed();
    println!("  Inserted {N_VECS} vectors in {insert_duration:?}");
    println!(
        "  Throughput: {:.0} vectors/sec",
        N_VECS as f64 / insert_duration.as_secs_f64()
    );

    // ── Phase 3: Baseline Searches (pre-persistence) ───────────────────────────
    divider();
    println!("Phase 3 — Running {N_QUERIES} baseline searches");

    let mut query_rng = StdRng::seed_from_u64(SEED + 1); // different seed from data
    let queries: Vec<Vec<f32>> = (0..N_QUERIES)
        .map(|_| random_vector(&mut query_rng, DIM))
        .collect();

    let t0 = Instant::now();
    let baseline_results: Vec<_> = queries.iter().map(|q| db.search(q, THRESHOLD)).collect();
    let search_duration = t0.elapsed();

    let hits = baseline_results.iter().filter(|r| r.is_some()).count();
    println!("  Completed in {:?}", search_duration);
    println!(
        "  Average per query: {:.2} µs",
        search_duration.as_micros() as f64 / N_QUERIES as f64
    );
    println!("  Cache hits (score > {THRESHOLD}): {hits}/{N_QUERIES}");

    // ── Phase 4: Serialization ────────────────────────────────────────────────
    divider();
    println!("Phase 4 — Saving index to disk (bincode)");

    let tmp_path = PathBuf::from("/tmp/abkve_load_test.bin");

    let t0 = Instant::now();
    {
        let file = fs::File::create(&tmp_path)?;
        let writer = BufWriter::new(file);
        db.save(writer)?;
    }
    let save_duration = t0.elapsed();
    let file_size = fs::metadata(&tmp_path)?.len();

    println!("  Saved to: {}", tmp_path.display());
    println!("  File size: {}", fmt_bytes(file_size));
    println!("  Saved in: {save_duration:?}");
    println!(
        "  Write throughput: {:.0} MB/s",
        file_size as f64 / save_duration.as_secs_f64() / 1_000_000.0
    );

    // ── Phase 5: Deserialization ──────────────────────────────────────────────
    divider();
    println!("Phase 5 — Loading index from disk");

    let t0 = Instant::now();
    let db_loaded = {
        let file = fs::File::open(&tmp_path)?;
        let reader = BufReader::new(file);
        Abkve::load(reader)?
    };
    let load_duration = t0.elapsed();

    println!("  Loaded in: {load_duration:?}");
    println!(
        "  Read throughput: {:.0} MB/s",
        file_size as f64 / load_duration.as_secs_f64() / 1_000_000.0
    );
    println!("  Vectors in loaded index: {}", db_loaded.len());
    println!("  Dimension in loaded index: {}", db_loaded.dim());

    // ── Phase 6: Integrity Verification ──────────────────────────────────────
    divider();
    println!("Phase 6 — Verifying round-trip integrity");
    println!("  Running {N_QUERIES} identical queries on the loaded index...");

    let t0 = Instant::now();
    let loaded_results: Vec<_> = queries
        .iter()
        .map(|q| db_loaded.search(q, THRESHOLD))
        .collect();
    let verify_duration = t0.elapsed();

    let mut mismatches = 0usize;
    let mut total_score_delta = 0.0f64;

    for (i, (orig, loaded)) in baseline_results
        .iter()
        .zip(loaded_results.iter())
        .enumerate()
    {
        match (orig, loaded) {
            (Some((oid, os)), Some((lid, ls))) => {
                if oid != lid {
                    eprintln!("  ✗ Query {i}: ID mismatch — original={oid}, loaded={lid}");
                    mismatches += 1;
                } else {
                    let delta = (os - ls).abs() as f64;
                    total_score_delta += delta;
                    if delta > 1e-4 {
                        eprintln!(
                            "  ✗ Query {i}: Score drift > 1e-4 — original={os:.6}, loaded={ls:.6}"
                        );
                        mismatches += 1;
                    }
                }
            }
            (None, None) => {}
            _ => {
                eprintln!(
                    "  ✗ Query {i}: Hit/miss mismatch — original={orig:?}, loaded={loaded:?}"
                );
                mismatches += 1;
            }
        }
    }

    let avg_score_delta = total_score_delta / N_QUERIES as f64;
    println!("  Verified in {:?}", verify_duration);
    println!("  Mismatches: {mismatches}/{N_QUERIES}");
    println!("  Avg score delta (floating-point drift): {avg_score_delta:.2e}");

    // ── Phase 7: Summary ─────────────────────────────────────────────────────
    divider();
    if mismatches == 0 {
        println!("✅ Round-trip integrity: PASSED");
    } else {
        println!("❌ Round-trip integrity: FAILED ({mismatches} mismatches)");
    }

    println!();
    println!("Performance Summary");
    println!("──────────────────────────────────────────────────");
    println!(
        "  Insert throughput:      {:.0} vec/s",
        N_VECS as f64 / insert_duration.as_secs_f64()
    );
    println!(
        "  Search latency (avg):   {:.2} µs/query ({N_VECS} vecs, dim={DIM})",
        search_duration.as_micros() as f64 / N_QUERIES as f64
    );
    println!(
        "  Serialization speed:    {:.0} MB/s",
        file_size as f64 / save_duration.as_secs_f64() / 1_000_000.0
    );
    println!(
        "  Deserialization speed:  {:.0} MB/s",
        file_size as f64 / load_duration.as_secs_f64() / 1_000_000.0
    );
    println!("──────────────────────────────────────────────────");

    // Clean up temp file
    let _ = fs::remove_file(&tmp_path);

    Ok(())
}
