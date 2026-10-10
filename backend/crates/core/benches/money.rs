//! Money speed budgets (plan 02 T6). Owner: `robustness`.
//!
//! The NFR has no target for one money function, so each budget is derived from the user
//! action that calls it (`scripts/perf/README.md` Section 6):
//!
//! - NFR-SPD-7: the live summary must show 150 ms after the last key; 50 ms of that is the
//!   wait for the user to stop typing, so the preview has 100 ms. A بکری can hold 500 sale
//!   lines (NFR-SCL-3). The preview also reads SQLite, crosses the Tauri bridge and builds
//!   the journal and the working records, so the pure money maths gets 10% of it:
//!   10 ms for 500 lines, 20 µs per line.
//! - One line parses about 3 texts (qty, seller rate, buyer rate) and makes about 3 percent
//!   charges (seller commission, buyer commission, لاگا). Budgets: parse 2 µs, apply_bps
//!   1 µs. That is 9 µs of the 20 µs, leaving room for line_value, per_unit and the sums.
//! - A partner split runs once per posting, not per line: 5 µs for 3 partners, 10 µs for
//!   10 partners (NFR-SCL-5).
//!
//! Each benchmark id here has one entry in `scripts/perf/thresholds.json`.

use std::hint::black_box;

use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use mandi_core::money::{
    Bps, Paisa, Qty, apply_bps, line_value, parse_rupees, parse_sum_qty, parse_sum_rupees,
    per_unit, split_by_bps,
};

/// Sale lines in the largest بکری (NFR-SCL-3).
const LINES: usize = 500;
/// Default commission, 6% (PRD 4.2).
const COMMISSION_BPS: i32 = 600;
/// لاگا per نگ on fruit, 8 rupees (PRD 4.2).
const LAGA_RUPEES: i64 = 8;

fn bps(v: i32) -> Option<Bps> {
    Bps::new(v).ok()
}

fn rupees(r: i64) -> Option<Paisa> {
    Paisa::from_rupees(r).ok()
}

/// Partners الف, ب, ج at 40 / 30 / 30 (PRD 4.2).
fn three_partners() -> Option<Vec<(u32, Bps)>> {
    Some(vec![(1, bps(4_000)?), (2, bps(3_000)?), (3, bps(3_000)?)])
}

/// Ten partners, the most NFR-SCL-5 allows, with uneven shares.
fn ten_partners() -> Option<Vec<(u32, Bps)>> {
    let shares = [1_900, 1_500, 1_200, 1_000, 1_000, 900, 800, 700, 600, 400];
    shares
        .iter()
        .zip(1_u32..)
        .map(|(&s, id)| Some((id, bps(s)?)))
        .collect()
}

/// The typed text of the 500 lines: qty, seller rate, buyer rate. Some use sums and
/// grouping commas, as a munshi types them.
fn line_texts() -> Vec<(String, String, String)> {
    (0..LINES)
        .map(|i| {
            let qty = if i % 5 == 0 {
                "20+25".to_owned()
            } else {
                (i % 90 + 1).to_string()
            };
            let rate = format!("1,{:03}", i % 1_000);
            (qty, rate.clone(), rate)
        })
        .collect()
}

/// The money maths of a 500-line preview: parse every line, work out value, both
/// commissions and لاگا, add the totals, split the income once. `None` on any error.
fn preview_maths(
    lines: &[(String, String, String)],
    partners: &[(u32, Bps)],
) -> Option<Vec<(u32, Paisa)>> {
    let pct = bps(COMMISSION_BPS)?;
    let laga = rupees(LAGA_RUPEES)?;
    let mut gross = Paisa::ZERO;
    let mut income = Paisa::ZERO;
    let mut buyer_total = Paisa::ZERO;
    for (qty_text, seller_text, buyer_text) in lines {
        let qty: Qty = parse_sum_qty(qty_text).ok()?;
        let seller_rate = parse_sum_rupees(seller_text).ok()?;
        let buyer_rate = parse_sum_rupees(buyer_text).ok()?;
        let seller_amount = line_value(qty, seller_rate).ok()?;
        let buyer_value = line_value(qty, buyer_rate).ok()?;
        let seller_comm = apply_bps(seller_amount, pct).ok()?;
        let buyer_comm = apply_bps(buyer_value, pct).ok()?;
        let laga_amount = per_unit(qty, laga).ok()?;
        gross = gross.checked_add(seller_amount).ok()?;
        income = income
            .checked_add(seller_comm)
            .and_then(|i| i.checked_add(buyer_comm))
            .and_then(|i| i.checked_add(laga_amount))
            .ok()?;
        buyer_total = buyer_total
            .checked_add(buyer_value)
            .and_then(|t| t.checked_add(buyer_comm))
            .and_then(|t| t.checked_add(laga_amount))
            .ok()?;
    }
    black_box((gross, buyer_total));
    split_by_bps(income, partners).ok()
}

fn money(c: &mut Criterion) {
    let base = rupees(5_630);
    let pct = bps(COMMISSION_BPS);
    let three = three_partners().unwrap_or_default();
    let ten = ten_partners().unwrap_or_default();
    let income = rupees(3_948);

    let mut group = c.benchmark_group("money");
    group.bench_function("apply_bps", |b| {
        b.iter(|| {
            base.zip(pct)
                .map(|(a, p)| apply_bps(black_box(a), black_box(p)))
        });
    });
    group.bench_function("parse_rupees", |b| {
        b.iter(|| parse_rupees(black_box("1,23,456")));
    });
    group.bench_function("split_by_bps_3", |b| {
        b.iter(|| income.map(|a| split_by_bps(black_box(a), black_box(&three))));
    });
    group.bench_function("split_by_bps_10", |b| {
        b.iter(|| income.map(|a| split_by_bps(black_box(a), black_box(&ten))));
    });
    group.finish();

    let lines = line_texts();
    let mut group = c.benchmark_group("preview");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(50);
    group.bench_function("maths_500_lines", |b| {
        b.iter(|| preview_maths(black_box(&lines), black_box(&three)));
    });
    group.finish();
}

criterion_group!(benches, money);
criterion_main!(benches);
