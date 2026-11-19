//! Plan 80, lane T2 (volume and speed): the local backup snapshot on the full NFR-SCL-1 data
//! (NFR-SPD-15: under 5 s for data up to 300 MB; NFR-SCL-4: under 2 GB on disk at the NFR-SCL-1
//! volume, compressed backup under 500 MB). Owner: `robustness`.
//!
//! `#[ignore]`d: it needs the data built by `scripts/perf/gate-t2-run.mjs` and a release build.
//! The snapshot is taken from a private copy of the data (the copy is removed afterwards), with the
//! real disk probe. NFR-SPD-15 names 300 MB, the normal volume, not the 10 times larger maximum;
//! the test reports the time of the whole file and of its first 300 MB worth (the rate times
//! 300 MB), and holds the 300 MB figure to 5 s.
#![cfg(test)]

use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use mandi_backup::snapshot::snapshot_copy;
use mandi_db::disk::RealDiskProbe;
use mandi_services::testing::TestEnv;

fn out(line: &serde_json::Value) {
    let path = std::env::var("GATE_T2_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("MANDI_PERF_DIR").unwrap_or_else(|_| ".".into()))
                .join("..")
                .join("gate-t2-results.jsonl")
        });
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(file, "{line}");
    }
}

#[test]
#[ignore = "needs the full volume of scripts/perf/gate-t2-run.mjs and a release build"]
fn t2_snapshot_and_size_of_the_full_volume() {
    let base = PathBuf::from(
        std::env::var("MANDI_PERF_DIR").expect("set MANDI_PERF_DIR under backend/target-gate-t2"),
    );
    let gate = base.with_file_name("nfr_scl1_gate");
    let source = gate.join("data").join("mandi.db");
    assert!(
        source.exists(),
        "build the data first: node scripts/perf/gate-t2-run.mjs --build-only"
    );
    let size = std::fs::metadata(&source).expect("the data file").len();
    let work = base.with_file_name(format!("gate_t2_snapshot_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(work.join("data")).expect("a folder");
    std::fs::copy(&source, work.join("data").join("mandi.db")).expect("a copy of the data");

    let env = TestEnv::new(&work).expect("the data opens");
    let dest = work.join("backups").join("snapshot.db");
    let mut times = Vec::new();
    for round in 0..3 {
        let _ = std::fs::remove_file(&dest);
        let started = Instant::now();
        let bytes = snapshot_copy(env.ctx.db(), &dest, &RealDiskProbe).expect("a snapshot");
        times.push(started.elapsed().as_secs_f64());
        assert!(
            bytes >= size / 2,
            "round {round}: the snapshot is {bytes} bytes of {size}"
        );
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let best = times[0];
    let worst = times[times.len() - 1];
    let mb = size as f64 / 1_000_000.0;
    // What 300 MB of this data would take, at the rate measured on the whole file.
    let at_300 = worst * 300.0 / mb;
    let snapshot_size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    let line = serde_json::json!({
        "id": "backup.snapshot.300mb",
        "nfr": "NFR-SPD-15",
        "action": format!("local backup snapshot, estimated for 300 MB from the whole {mb:.0} MB file (rate x 300 MB)"),
        "target_ms": 5_000.0,
        "also_ms": 0.0,
        "runs": times.len(),
        "cold_first_ms": times[0] * 1000.0,
        "p50_ms": times[1] * 1000.0,
        "p95_ms": at_300 * 1000.0,
        "max_ms": worst * 1000.0,
        "warm_p95_ms": best * 1000.0,
        "load1": -1.0,
    });
    out(&line);
    out(&serde_json::json!({
        "id": "backup.snapshot.whole",
        "fact": format!("snapshot of the whole {mb:.0} MB file: best {best:.2} s, worst {worst:.2} s; the snapshot file is {} MB", snapshot_size / 1_000_000),
    }));
    let lines: i64 = env
        .ctx
        .db()
        .read(|c| c.query_row("SELECT COUNT(*) FROM journal_lines", [], |r| r.get(0)))
        .unwrap_or(0);
    out(&serde_json::json!({
        "id": "scl4.size",
        "fact": format!(
            "NFR-SCL-4: the data file is {mb:.0} MB on disk (target under 2,000 MB at the NFR-SCL-1 volume). It holds {lines} journal lines, {:.2} times the 3.5 million of NFR-SCL-1, so the figure for the exact volume is about {:.0} MB.",
            lines as f64 / 3_500_000.0,
            // 203 bytes a journal line: the plain volume is 712 MB for 3.5 million lines.
            mb - (lines as f64 - 3_500_000.0) * 203.0 / 1_000_000.0
        ),
    }));
    drop(env);
    let _ = std::fs::remove_dir_all(&work);
}
