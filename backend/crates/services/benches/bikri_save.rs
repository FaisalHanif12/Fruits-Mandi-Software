//! Saving a بکری on the NFR-SCL-1 volume (NFR-SPD-8, plan 13).
//!
//! `bikri::save` for a typed draft: parse, read the facts, `calc_posting`, the journal entry
//! and its balance check, the share rows, the بکری with its lots and lines, stock moves,
//! analytics facts, matching, the audit row, and one commit with `synchronous = FULL`.
//!
//! - `bikri/save_30_lines`: 3 lots of 10 lines, the NFR-SPD-8 case. 300 ms.
//!
//! Every iteration saves a new بکری on a private copy of the volume, so the buyers' lines grow
//! a little with each one. The volume has no matching rows for its old lines (it is made with
//! plain SQL), so the first save of a buyer also writes the matching of his history; the
//! warm-up runs before the measured samples.

mod bikri_draft;
mod support;

use std::hint::black_box;
use std::time::Duration;

use bikri_draft::{add_partners, draft};
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use mandi_services::bikri;
use mandi_services::core::bikri::BikriSaveInput;
use support::{copy_of_volume, ensure_volume, fail, open};

fn save_speed(c: &mut Criterion) {
    let volume = ensure_volume();
    support::describe(&volume);
    let copy = copy_of_volume(&volume);
    let (env, actor) = open(copy.path());
    add_partners(&env, &actor);
    let thirty = BikriSaveInput {
        draft: draft(&env, 3, 10),
        // The volume has little cash; the confirmation is part of the save, not of the speed.
        confirm_cash_below_zero: true,
    };
    // The draft must really be saved, not refused: a refusal is fast and would hide a slow
    // path.
    match bikri::save(&env.ctx, &actor, &thirty) {
        Ok(saved) => {
            let lines = saved
                .preview
                .lots
                .iter()
                .flat_map(|l| l.lines.iter())
                .filter(|l| l.calc.is_some())
                .count();
            if lines != 30 || saved.preview.split.len() != 3 {
                fail(&format!(
                    "the 30-line draft was not saved as 30 lines: {lines} lines, {} partners",
                    saved.preview.split.len()
                ));
            }
        }
        Err(error) => fail(&format!("save failed: {error}")),
    }

    let mut group = c.benchmark_group("bikri");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("save_30_lines", |b| {
        b.iter(
            || match bikri::save(black_box(&env.ctx), black_box(&actor), black_box(&thirty)) {
                Ok(saved) => black_box(saved.number),
                Err(error) => fail(&format!("save failed: {error}")),
            },
        );
    });
    group.finish();
}

criterion_group!(benches, save_speed);
criterion_main!(benches);
