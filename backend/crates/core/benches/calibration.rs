//! Calibration benchmark for the speed harness (`pnpm perf`, plan 01 T5).
//!
//! A fixed, single-thread integer workload shaped like our money maths:
//! multiply in `i128`, divide by 10,000, round half up, add with checked
//! operations, then sort the results. It uses no database, no file and no
//! clock, so its time depends only on the CPU.
//!
//! Why it exists: `scripts/perf/README.md` sets the runner factor from public
//! single-thread CPU scores. This benchmark checks that estimate on every run.
//! `scripts/perf/run.mjs` compares its time with the time expected on the
//! reference CPU (NFR Section 0.2). If the runner is ever slower than that,
//! the harness warns that the factor must be measured again.
//!
//! The workload MUST NOT change. Its expected times in
//! `scripts/perf/thresholds.json` were measured on this exact code.
//! Real money benchmarks (plan 02 and later) go in their own files.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/// Number of amounts in one run of the workload.
const ITEMS: usize = 10_000;
/// Basis points in 100% (Code Standards M2).
const BPS_PER_WHOLE: i128 = 10_000;
/// Half of `BPS_PER_WHOLE`, for rounding half up.
const HALF: i128 = 5_000;
/// Seed of the fixed number sequence, so every run does the same work.
const SEED: u64 = 0x5EED_0000_0001;

/// A small linear congruential generator (Knuth MMIX constants).
/// Deterministic, so the workload is identical on every machine.
fn next(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

/// Builds the fixed input: (amount in paisa, rate in bps) pairs.
/// Amounts are whole rupees up to 10 crore; rates up to 100.00%.
fn inputs() -> Vec<(i64, i32)> {
    let mut state = SEED;
    let mut out = Vec::with_capacity(ITEMS);
    for _ in 0..ITEMS {
        state = next(state);
        let rupees = i64::try_from((state >> 16) % 1_000_000_000).unwrap_or(0);
        state = next(state);
        let bps = i32::try_from((state >> 16) % 10_001).unwrap_or(0);
        out.push((rupees.saturating_mul(100), bps));
    }
    out
}

/// The workload: percent of each amount, rounded half up to paisa, summed with
/// checked operations, then the parts sorted. Returns the total, or None on
/// overflow (which cannot happen with these inputs).
fn workload(items: &[(i64, i32)]) -> Option<i128> {
    let mut parts: Vec<i128> = Vec::with_capacity(items.len());
    let mut total: i128 = 0;
    for &(paisa, bps) in items {
        let product = i128::from(paisa).checked_mul(i128::from(bps))?;
        let rounded = product.checked_add(HALF)?.checked_div(BPS_PER_WHOLE)?;
        total = total.checked_add(rounded)?;
        parts.push(rounded);
    }
    parts.sort_unstable();
    let middle = parts.get(parts.len() / 2).copied().unwrap_or(0);
    total.checked_add(middle)
}

fn calibration(c: &mut Criterion) {
    let items = inputs();
    let mut group = c.benchmark_group("calibration");
    group.sample_size(100);
    group.bench_function("cpu_fixed_10k", |b| {
        b.iter(|| workload(black_box(&items)));
    });
    group.finish();
}

criterion_group!(benches, calibration);
criterion_main!(benches);
