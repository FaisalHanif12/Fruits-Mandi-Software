//! The timing harness of lane T2: runs one action many times, takes the 95th percentile by
//! nearest rank (the same rule as `scripts/perf/run.mjs`), prints one line and appends one JSON
//! line to the results file that `scripts/perf/gate-t2-run.mjs` turns into the report.
//!
//! "Cold" means the memory of kept sums (`mandi_db::memo`) is emptied before the call, as it is
//! after any save in the real app. That is the number held to the target. "Warm" calls the same
//! action again with the memory as the last call left it; it is printed for information only.

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// One measured action.
#[derive(Debug, Clone)]
pub struct Row {
    pub id: String,
    pub nfr: String,
    pub action: String,
    pub target_ms: f64,
    /// A second, stricter or older target for the same row (0 = none), for example NFR-SPD-12 next
    /// to the NFR-SPD-24 exception.
    pub also_ms: f64,
    pub runs: usize,
    pub cold_first_ms: f64,
    pub min_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
    pub warm_p95_ms: f64,
    /// Mean CPU time of one cold call, ms (what the call costs when the Mac is not busy).
    pub cpu_mean_ms: f64,
    /// Mean wall time of one cold call, ms.
    pub wall_mean_ms: f64,
}

impl Row {
    /// Over its target. A target of 0 means the row is information only.
    pub fn over(&self) -> bool {
        self.target_ms > 0.0 && self.p95_ms > self.target_ms
    }
}

/// Where the JSON lines go: `GATE_T2_OUT`, else next to the data folder.
fn out_path() -> PathBuf {
    if let Ok(path) = std::env::var("GATE_T2_OUT") {
        return PathBuf::from(path);
    }
    let base = std::env::var("MANDI_PERF_DIR").unwrap_or_else(|_| ".".to_owned());
    PathBuf::from(base).join("..").join("gate-t2-results.jsonl")
}

/// The one-minute load average, for the line that says how busy the Mac was.
fn load_average() -> f64 {
    std::process::Command::new("sysctl")
        .args(["-n", "vm.loadavg"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|text| {
            text.split_whitespace()
                .find_map(|w| w.trim_matches(|c| c == '{' || c == '}').parse::<f64>().ok())
        })
        .unwrap_or(-1.0)
}

/// CPU seconds this process has used so far, from `ps` (resolution 10 ms). `std` has no way to
/// read it and `unsafe` is forbidden, so the harness asks `ps`. -1 when it cannot.
fn cpu_seconds() -> f64 {
    let pid = std::process::id().to_string();
    let text = std::process::Command::new("ps")
        .args(["-o", "cputime=", "-p", &pid])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let mut seconds = 0.0_f64;
    for part in text.trim().split(':') {
        match part.trim().parse::<f64>() {
            Ok(n) => seconds = seconds * 60.0 + n,
            Err(_) => return -1.0,
        }
    }
    seconds
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000.0
}

/// Nearest-rank percentile of a sorted list.
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (p * sorted.len() as f64).ceil() as usize;
    sorted
        .get(rank.saturating_sub(1).min(sorted.len() - 1))
        .copied()
        .unwrap_or(0.0)
}

/// `GATE_T2_ROWS=dash.get,report.R2` measures only the rows whose id contains one of the texts (to
/// measure a few slow rows again on a quiet machine). Empty or unset measures everything.
fn wanted(id: &str) -> bool {
    match std::env::var("GATE_T2_ROWS") {
        Ok(list) if !list.trim().is_empty() => list.split(',').any(|part| id.contains(part.trim())),
        _ => true,
    }
}

fn skipped(id: &str, target_ms: f64) -> Row {
    Row {
        id: id.to_owned(),
        nfr: String::new(),
        action: String::new(),
        target_ms,
        also_ms: 0.0,
        runs: 0,
        cold_first_ms: 0.0,
        min_ms: 0.0,
        p50_ms: 0.0,
        p95_ms: 0.0,
        max_ms: 0.0,
        warm_p95_ms: 0.0,
        cpu_mean_ms: -1.0,
        wall_mean_ms: -1.0,
    }
}

/// Time budget for the cold runs of one action, in milliseconds.
const BUDGET_MS: f64 = 8_000.0;
/// At most this many cold runs, at least [`MIN_RUNS`].
const MAX_RUNS: usize = 20;
const MIN_RUNS: usize = 3;

/// Times `call`: one cold first call, then as many cold runs as the budget allows (3 to 20),
/// then a few warm runs. Records the row.
pub fn measure<T>(
    id: &str,
    nfr: &str,
    action: &str,
    target_ms: f64,
    call: impl FnMut() -> T,
) -> Row {
    measure_with(id, nfr, action, target_ms, 0.0, call)
}

/// Like [`measure`], with a second target kept in the row.
pub fn measure_with<T>(
    id: &str,
    nfr: &str,
    action: &str,
    target_ms: f64,
    also_ms: f64,
    mut call: impl FnMut() -> T,
) -> Row {
    if !wanted(id) {
        return skipped(id, target_ms);
    }
    mandi_services::db::memo::clear();
    let started = Instant::now();
    drop(call());
    let first = ms(started.elapsed());
    let runs = ((BUDGET_MS / first.max(0.05)) as usize).clamp(MIN_RUNS, MAX_RUNS);
    let mut cold: Vec<f64> = Vec::with_capacity(runs);
    let cpu_before = cpu_seconds();
    let wall_before = Instant::now();
    for _ in 0..runs {
        mandi_services::db::memo::clear();
        let started = Instant::now();
        drop(call());
        cold.push(ms(started.elapsed()));
    }
    let cpu_after = cpu_seconds();
    let wall_total = ms(wall_before.elapsed());
    let cpu_mean = if cpu_before >= 0.0 && cpu_after >= 0.0 {
        (cpu_after - cpu_before) * 1_000.0 / runs as f64
    } else {
        -1.0
    };
    cold.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let warm_runs = runs.min(10);
    let mut warm: Vec<f64> = Vec::with_capacity(warm_runs);
    for _ in 0..warm_runs {
        let started = Instant::now();
        drop(call());
        warm.push(ms(started.elapsed()));
    }
    warm.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let row = Row {
        id: id.to_owned(),
        nfr: nfr.to_owned(),
        action: action.to_owned(),
        target_ms,
        also_ms,
        runs,
        cold_first_ms: first,
        min_ms: cold.first().copied().unwrap_or(0.0),
        p50_ms: percentile(&cold, 0.5),
        p95_ms: percentile(&cold, 0.95),
        max_ms: cold.last().copied().unwrap_or(0.0),
        warm_p95_ms: percentile(&warm, 0.95),
        cpu_mean_ms: cpu_mean,
        wall_mean_ms: wall_total / runs as f64,
    };
    record(&row);
    row
}

/// Prints the row and appends it to the results file.
pub fn record(row: &Row) {
    let load = load_average();
    let verdict = if row.over() { "OVER" } else { "ok  " };
    crate::support::note(&format!(
        "{verdict} p95 {:>9.2} ms (target {:>7.0}) first {:>9.2} p50 {:>9.2} max {:>9.2} warm p95 {:>9.2} cpu/call {:>8.1} wall/call {:>8.1} n={:<2} load {load:.1}  {} {}",
        row.p95_ms,
        row.target_ms,
        row.cold_first_ms,
        row.p50_ms,
        row.max_ms,
        row.warm_p95_ms,
        row.cpu_mean_ms,
        row.wall_mean_ms,
        row.runs,
        row.nfr,
        row.action
    ));
    let line = serde_json::json!({
        "id": row.id,
        "nfr": row.nfr,
        "action": row.action,
        "target_ms": row.target_ms,
        "also_ms": row.also_ms,
        "runs": row.runs,
        "cold_first_ms": row.cold_first_ms,
        "min_ms": row.min_ms,
        "p50_ms": row.p50_ms,
        "p95_ms": row.p95_ms,
        "max_ms": row.max_ms,
        "warm_p95_ms": row.warm_p95_ms,
        "cpu_mean_ms": row.cpu_mean_ms,
        "wall_mean_ms": row.wall_mean_ms,
        "load1": load,
    });
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out_path())
    {
        let _ = writeln!(file, "{line}");
    }
}

/// Records a plain fact (a count, a query plan, a size) as a line of the results file.
pub fn fact(id: &str, text: &str) {
    crate::support::note(&format!("fact {id}: {text}"));
    let line = serde_json::json!({ "id": id, "fact": text });
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out_path())
    {
        let _ = writeln!(file, "{line}");
    }
}

/// Ends a test: fails when a row is over its target, unless `MANDI_SPEED_REPORT_ONLY` is set.
pub fn finish(rows: &[Row]) {
    let over: Vec<String> = rows
        .iter()
        .filter(|r| r.over())
        .map(|r| format!("{} p95 {:.1} ms > {:.0} ms", r.id, r.p95_ms, r.target_ms))
        .collect();
    if std::env::var("MANDI_SPEED_REPORT_ONLY").is_err() {
        assert!(over.is_empty(), "over target: {over:?}");
    }
}

/// Times the next page of a list the way a user meets it: the first page has just been read (the
/// memory of kept sums and the kept results are as that read left them), then the next page is
/// asked. Each run empties the memory first, runs `prime` untimed, then times `call`. The row's CPU
/// figure is the CPU of the whole run (prime included) scaled to the timed call, so the adjusted
/// p95 of the report script stays honest.
pub fn measure_next<A, T>(
    id: &str,
    nfr: &str,
    action: &str,
    target_ms: f64,
    mut prime: impl FnMut() -> A,
    mut call: impl FnMut() -> T,
) -> Row {
    if !wanted(id) {
        return skipped(id, target_ms);
    }
    let one = |prime: &mut dyn FnMut() -> A, call: &mut dyn FnMut() -> T| -> (f64, f64) {
        mandi_services::db::memo::clear();
        let whole = Instant::now();
        drop(prime());
        let started = Instant::now();
        drop(call());
        (ms(started.elapsed()), ms(whole.elapsed()))
    };
    let (first_timed, first_whole) = one(&mut prime, &mut call);
    let runs = ((BUDGET_MS / first_whole.max(0.05)) as usize).clamp(MIN_RUNS, MAX_RUNS);
    let mut timed: Vec<f64> = Vec::with_capacity(runs);
    let (mut whole_total, mut timed_total) = (0.0_f64, 0.0_f64);
    let cpu_before = cpu_seconds();
    for _ in 0..runs {
        let (t, w) = one(&mut prime, &mut call);
        timed.push(t);
        timed_total += t;
        whole_total += w;
    }
    let cpu_after = cpu_seconds();
    timed.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let ratio = if cpu_before >= 0.0 && cpu_after >= 0.0 && whole_total > 0.0 {
        ((cpu_after - cpu_before) * 1_000.0 / whole_total).min(1.0)
    } else {
        -1.0
    };
    let mean_timed = timed_total / runs as f64;
    let row = Row {
        id: id.to_owned(),
        nfr: nfr.to_owned(),
        action: action.to_owned(),
        target_ms,
        also_ms: 0.0,
        runs,
        cold_first_ms: first_timed,
        min_ms: timed.first().copied().unwrap_or(0.0),
        p50_ms: percentile(&timed, 0.5),
        p95_ms: percentile(&timed, 0.95),
        max_ms: timed.last().copied().unwrap_or(0.0),
        warm_p95_ms: 0.0,
        cpu_mean_ms: if ratio >= 0.0 {
            ratio * mean_timed
        } else {
            -1.0
        },
        wall_mean_ms: mean_timed,
    };
    record(&row);
    row
}
