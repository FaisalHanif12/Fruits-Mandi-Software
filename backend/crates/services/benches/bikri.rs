//! The live summary of a بکری on the NFR-SCL-1 volume (NFR-SPD-7, NFR-SCL-3, plan 12).
//! Written by `backend` at the lead's request; `robustness` owns the folder and may change it.
//!
//! `bikri::preview` for a typed draft: parse, read the facts of the accounts and items, work out
//! every line (`calc_posting`), build the journal entry, split the income between the partners
//! and read cash in hand. All server time; the screen adds the 50 ms pause that waits for the
//! last key press (NFR-SPD-7 is 150 ms in all).
//!
//! - `bikri/preview_30_lines`: 3 lots of 10 lines, the NFR-SPD-7 case. 100 ms.
//! - `bikri/preview_500_lines`: 50 lots of 10 lines, the NFR-SCL-3 case. 150 ms.
//!
//! The draft is only worked out, never saved, so the volume is not changed; partners are added
//! on a private copy so that the split can be made.

mod bikri_draft;
mod support;

use std::hint::black_box;
use std::time::Duration;

use bikri_draft::{add_partners, draft};
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use mandi_services::bikri;
use support::{copy_of_volume, ensure_volume, fail, open};

fn bikri_speed(c: &mut Criterion) {
    let volume = ensure_volume();
    support::describe(&volume);
    let copy = copy_of_volume(&volume);
    let (env, actor) = open(copy.path());
    add_partners(&env, &actor);
    let thirty = draft(&env, 3, 10);
    let five_hundred = draft(&env, 50, 10);
    // The draft must really be worked out, not refused: a refusal is fast and would hide a
    // slow path.
    for (name, d, lines) in [("30", &thirty, 30_usize), ("500", &five_hundred, 500)] {
        match bikri::preview(&env.ctx, d) {
            Ok(p) => {
                let worked = p
                    .lots
                    .iter()
                    .flat_map(|l| l.lines.iter())
                    .filter(|l| l.calc.is_some())
                    .count();
                if worked != lines || p.split.len() != 3 {
                    fail(&format!(
                        "the {name}-line draft was not worked out: {worked} lines, {} partners, errors {:?}",
                        p.split.len(),
                        p.errors
                    ));
                }
            }
            Err(error) => fail(&format!("preview failed: {error}")),
        }
    }

    let mut group = c.benchmark_group("bikri");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(8));
    group.bench_function("preview_30_lines", |b| {
        b.iter(
            || match bikri::preview(black_box(&env.ctx), black_box(&thirty)) {
                Ok(p) => black_box(p.summary.net),
                Err(error) => fail(&format!("preview failed: {error}")),
            },
        );
    });
    group.bench_function("preview_500_lines", |b| {
        b.iter(
            || match bikri::preview(black_box(&env.ctx), black_box(&five_hundred)) {
                Ok(p) => black_box(p.summary.net),
                Err(error) => fail(&format!("preview failed: {error}")),
            },
        );
    });
    group.finish();
}

criterion_group!(benches, bikri_speed);
criterion_main!(benches);
