//! Plan 61 T4 (D8 Bug 1; NFR-SCL-3 v1.17 of the NFR, PRD v1.32, NFR-SPD-7, PRD FR-7.5): one lot
//! with 1,000 sale lines.
//!
//! The live summary asks `bikri_preview` after every pause of typing, and the whole draft goes
//! with it. With 1,000 lines the call must stay quick: NFR-SPD-7 is 150 ms from the last key press,
//! of which the screen's own pause is 50 ms, so the backend has 100 ms.
//!
//! The test has two jobs:
//!
//! 1. The figures. The draft is 1,000 lines of 1 نگ each at rates of 500 to 695, ten lines for
//!    each of 100 buyers. The expected totals are worked out here with plain integer maths, apart
//!    from the code under test, so a speed change can never change a figure. The rates step by 5,
//!    so 6% of a line is often a half rupee (525 gives 31.5, rounded half up to 32): the sum of
//!    the rounded lines is not the same as 6% of the total, and a bug that rounds the total
//!    instead of each line would show (money review of plan 61).
//! 2. The time. The median of seven previews and one save are printed with `--nocapture`, and the
//!    preview is held to a bound: the NFR figure in a release build, a looser one in a debug build,
//!    which is about ten times slower. Run `cargo test --release -p mandi-services --test
//!    bikri_1000_lines -- --nocapture` for the real timing.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::time::{Duration, Instant};

use mandi_services::bikri;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::bikri::{BikriDraft, BikriPreview, BikriSaveInput};
use support::*;

/// Lines of the lot (NFR-SCL-3, the NFR at v1.17).
const LINES: usize = 1_000;
/// Buyers of the lines, in turn.
const BUYERS: usize = 100;
/// The backend's part of NFR-SPD-7: 150 ms less the 50 ms pause of the screen.
const PREVIEW_BOUND_RELEASE: Duration = Duration::from_millis(100);
/// A debug build is far slower; this only catches a step that grows with the square of the lines.
const PREVIEW_BOUND_DEBUG: Duration = Duration::from_millis(1_500);
/// The save of the 1,000 lines is one transaction with about 7,000 rows written. It has no NFR of
/// its own at this size (NFR-SPD-8 is 300 ms for 30 lines); the bound only catches a step that
/// goes wrong by an order of magnitude (a statement parsed for every line took 380 ms).
const SAVE_BOUND_RELEASE: Duration = Duration::from_millis(600);
const SAVE_BOUND_DEBUG: Duration = Duration::from_millis(4_000);
/// Runs of the preview that are timed.
const RUNS: usize = 7;

/// The rate of line `i` (0-based): 500, 505 ... 695, then again.
fn rate_of(i: usize) -> i64 {
    500 + (i64::try_from(i).unwrap() % 40) * 5
}

/// `lines` lines of 1 نگ. The lot has a freight of 10,000 and a labour of 1,000, like the screenshot.
fn big_draft(w: &World, buyers: &[i64], lines: usize) -> BikriDraft {
    let rows = (0..lines)
        .map(|i| {
            sale(
                &format!("line-{i}"),
                buyers[i % buyers.len()],
                "1",
                &rate_of(i).to_string(),
            )
        })
        .collect();
    let mut lot = bare_lot(w, "lot-1", w.rahim, w.banana, &lines.to_string());
    lot.freight = Some("10,000".into());
    lot.labour = manual("1,000");
    lot.sales = rows;
    // The screen keeps one empty row at the end (PRD FR-7.52). It is not a line of the limit.
    lot.sales.push(blank_sale("line-blank", ""));
    draft(vec![lot])
}

fn world_with_buyers() -> (World, Vec<i64>) {
    let w = world();
    let buyers = (0..BUYERS)
        .map(|n| {
            add_account(
                &w.env,
                &w.owner,
                AccountType::Buyer,
                &format!("خریدار {}", urdu_number(n)),
                &format!("Buyer {n}"),
                true,
                "",
            )
        })
        .collect();
    (w, buyers)
}

/// A number in Urdu letters is not needed for a name: the Urdu name only needs Urdu letters, so
/// the number is spelled with the letters of its digits, one letter for each digit.
fn urdu_number(n: usize) -> String {
    const LETTERS: [char; 10] = ['ا', 'ب', 'پ', 'ت', 'ٹ', 'ث', 'ج', 'چ', 'ح', 'خ'];
    n.to_string()
        .chars()
        .map(|c| LETTERS[usize::try_from(c.to_digit(10).unwrap()).unwrap()])
        .collect()
}

/// The figures of the whole draft by plain integer maths: rupees, half up for the percent charges.
struct Expected {
    gross: i64,
    seller_comm: i64,
    buyer_comm: i64,
    laga: i64,
    buyer_total: i64,
    /// The partners' parts of the shared income, 40 / 30 / 30, floor, leftover to the largest.
    split: [i64; 3],
}

fn expected() -> Expected {
    let percent = |value: i64| (value * 6 + 50) / 100;
    let mut e = Expected {
        gross: 0,
        seller_comm: 0,
        buyer_comm: 0,
        laga: 0,
        buyer_total: 0,
        split: [0; 3],
    };
    for i in 0..LINES {
        let value = rate_of(i);
        e.gross += value;
        e.seller_comm += percent(value);
        e.buyer_comm += percent(value);
        e.laga += 8;
        e.buyer_total += value + percent(value) + 8;
    }
    // Rounding each line is not rounding the total: the two differ for these rates.
    assert_ne!(e.seller_comm, percent(e.gross));
    let shared = e.seller_comm + e.buyer_comm + e.laga;
    let mut parts = [shared * 40 / 100, shared * 30 / 100, shared * 30 / 100];
    parts[0] += shared - parts.iter().sum::<i64>();
    e.split = parts;
    e
}

fn check_figures(p: &BikriPreview) {
    let e = expected();
    assert!(p.errors.is_empty(), "errors: {:?}", p.errors);
    assert!(p.warnings.is_empty(), "warnings: {:?}", p.warnings);
    // 1,000 lines and the empty row the screen keeps: not refused (the limit counts lines).
    assert_eq!(p.lots[0].lines.len(), LINES + 1);
    assert!(p.lots[0].lines[LINES].is_blank);
    assert!(p.lots[0].lines[..LINES].iter().all(|l| l.calc.is_some()));
    assert_eq!(p.lots[0].sold_qty, i64::try_from(LINES).unwrap());
    let s = &p.summary;
    assert_eq!(r(s.gross), e.gross);
    assert_eq!(r(s.seller_comm), e.seller_comm);
    assert_eq!(r(s.buyer_comm), e.buyer_comm);
    assert_eq!(r(s.laga), e.laga);
    assert_eq!(r(s.buyer_total), e.buyer_total);
    assert_eq!(r(s.freight), 10_000);
    assert_eq!(r(s.labour), 1_000);
    assert_eq!(r(s.deductions), 10_000 + e.seller_comm + 1_000);
    assert_eq!(r(s.net), e.gross - (10_000 + e.seller_comm + 1_000));
    assert_eq!(r(s.shared_income), e.seller_comm + e.buyer_comm + e.laga);
    let split: Vec<i64> = p.split.iter().map(|part| r(part.income_paisa)).collect();
    assert_eq!(split, e.split, "the partners' parts");
    assert_eq!(
        split.iter().sum::<i64>(),
        r(s.shared_income),
        "the split adds back to the income"
    );
}

fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let value = f();
    (value, start.elapsed())
}

#[test]
fn a_lot_of_1000_lines_is_worked_out_exactly_and_quickly() {
    let (w, buyers) = world_with_buyers();
    let d = big_draft(&w, &buyers, LINES);

    // One warm-up (the first call also builds caches), then the timed runs.
    let first = bikri::preview(&w.env.ctx, &d).unwrap();
    check_figures(&first);
    let mut times: Vec<Duration> = (0..RUNS)
        .map(|_| {
            let (p, t) = timed(|| bikri::preview(&w.env.ctx, &d).unwrap());
            assert_eq!(p, first, "the same draft gives the same preview");
            t
        })
        .collect();
    times.sort();
    let median = times[RUNS / 2];
    eprintln!(
        "bikri_preview, one lot of {LINES} lines, {BUYERS} buyers: median {:.1} ms (min {:.1}, max {:.1})",
        ms(median),
        ms(times[0]),
        ms(times[RUNS - 1])
    );
    let bound = if cfg!(debug_assertions) {
        PREVIEW_BOUND_DEBUG
    } else {
        PREVIEW_BOUND_RELEASE
    };
    assert!(
        median <= bound,
        "the preview of {LINES} lines took {:.1} ms; the bound is {:.1} ms",
        ms(median),
        ms(bound)
    );

    // The save is one transaction and gives the same figures (Code Standards B5).
    let (saved, save_time) = timed(|| {
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft: d.clone(),
                confirm_cash_below_zero: true,
            },
        )
        .unwrap()
    });
    eprintln!(
        "bikri_save, one lot of {LINES} lines: {:.1} ms",
        ms(save_time)
    );
    let save_bound = if cfg!(debug_assertions) {
        SAVE_BOUND_DEBUG
    } else {
        SAVE_BOUND_RELEASE
    };
    assert!(
        save_time <= save_bound,
        "the save of {LINES} lines took {:.1} ms; the bound is {:.1} ms",
        ms(save_time),
        ms(save_bound)
    );
    assert_eq!(saved.preview, first);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(detail.lots[0].sales.len(), LINES);
    let e = expected();
    assert_eq!(detail.lots[0].gross.rupees(), e.gross);
    assert_eq!(detail.lots[0].seller_comm.rupees(), e.seller_comm);
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000.0
}

/// What the command layer adds around the service: the draft arrives as JSON and the preview
/// leaves as JSON (Tauri serialises both). Printed for 250, 500 and 1,000 lines so a step that
/// grows with the square of the lines shows as a time that more than doubles when the lines do.
#[test]
fn the_preview_and_the_save_grow_in_line_with_the_lines_and_the_json_is_measured() {
    for lines in [250, 500, 1_000] {
        let (w, buyers) = world_with_buyers();
        let d = big_draft(&w, &buyers, lines);
        let request = serde_json::to_string(&serde_json::json!({ "draft": &d })).unwrap();
        let (parsed, parse_time) = timed(|| {
            serde_json::from_str::<mandi_services::core::bikri::BikriPreviewInput>(&request)
                .unwrap()
        });
        assert_eq!(parsed.draft.lots[0].sales.len(), lines + 1);
        let _ = bikri::preview(&w.env.ctx, &d).unwrap();
        let mut times: Vec<Duration> = (0..RUNS)
            .map(|_| timed(|| bikri::preview(&w.env.ctx, &d).unwrap()).1)
            .collect();
        times.sort();
        let preview = bikri::preview(&w.env.ctx, &d).unwrap();
        let (json, json_time) = timed(|| serde_json::to_string(&preview).unwrap());
        let (_, save_time) = timed(|| {
            bikri::save(
                &w.env.ctx,
                &w.owner,
                &BikriSaveInput {
                    draft: d.clone(),
                    confirm_cash_below_zero: true,
                },
            )
            .unwrap()
        });
        eprintln!(
            "{lines:>5} lines: preview {:>6.1} ms | answer {:>5.0} KB, to JSON {:>5.1} ms | request {:>4.0} KB, from JSON {:>4.1} ms | save {:>6.1} ms",
            ms(times[RUNS / 2]),
            f64::from(u32::try_from(json.len()).unwrap()) / 1_024.0,
            ms(json_time),
            f64::from(u32::try_from(request.len()).unwrap()) / 1_024.0,
            ms(parse_time),
            ms(save_time),
        );
    }
}
