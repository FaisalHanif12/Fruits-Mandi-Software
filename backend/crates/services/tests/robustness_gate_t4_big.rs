//! Gate lane T4: a بکری of 50 lots and 1,000 sale lines (NFR-SCL-3, NFR-SPD-7, NFR-SPD-8; PRD
//! FR-12.45 lot names, FR-12.8 one save; Code Standards B3, B5, M4, M9; E24, E103). Owner:
//! `robustness`.
//!
//! `bikri_1000_lines.rs` has ONE lot of 1,000 lines. NFR-SCL-3 says "up to 50 lots and 1,000 sale
//! lines", so this file builds the other shapes of the limit and proves each through the service:
//!
//! - 50 lots of 20 lines each, over 10 sellers and 2 items, so lot names (FR-12.45) climb with the
//!   same seller and item inside one draft;
//! - 49 lots of 1 line and one lot of 951 lines (a skewed draft);
//! - the limits themselves: 51 lots, 1,001 lines, and the 100 blank rows the screen may add.
//!
//! The expected figures are worked out here with plain integer maths, apart from the code under
//! test (6% of a line is often half a rupee, so rounding each line is not rounding the total). The
//! preview and the save must give the same numbers (B5), the journal must balance, the partner
//! parts must add back to the shared income, and a refused draft must write nothing.
//!
//! Times are printed with `--nocapture`. The bounds only catch a step that grows with the square of
//! the lines; the real NFR numbers are checked in release on the reference machine (LOCAL-ONLY).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::time::{Duration, Instant};

use common::raw;
use mandi_services::ServiceError;
use mandi_services::bikri;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::bikri::{BikriDraft, BikriPreview, BikriSaveInput, LotInput};
use mandi_services::core::error::ErrorCode;
use support::*;

/// The backend's part of NFR-SPD-7 (150 ms less the 50 ms pause of the screen), release build.
const PREVIEW_BOUND_RELEASE: Duration = Duration::from_millis(100);
/// Debug is far slower and the machine may be shared: only an order of magnitude is caught.
const PREVIEW_BOUND_DEBUG: Duration = Duration::from_secs(8);
const SAVE_BOUND_RELEASE: Duration = Duration::from_millis(600);
const SAVE_BOUND_DEBUG: Duration = Duration::from_secs(20);

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000.0
}

fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let value = f();
    (value, start.elapsed())
}

/// Six percent of `value` rupees, rounded half up, by integer maths.
fn percent(value: i64) -> i64 {
    (value * 6 + 50) / 100
}

fn urdu_number(n: usize) -> String {
    const LETTERS: [char; 10] = ['ا', 'ب', 'پ', 'ت', 'ٹ', 'ث', 'ج', 'چ', 'ح', 'خ'];
    n.to_string()
        .chars()
        .map(|c| LETTERS[usize::try_from(c.to_digit(10).unwrap()).unwrap()])
        .collect()
}

struct Big {
    w: World,
    sellers: Vec<i64>,
    buyers: Vec<i64>,
}

fn big_world() -> Big {
    let w = world();
    let sellers = (0..10)
        .map(|n| {
            add_account(
                &w.env,
                &w.owner,
                AccountType::Seller,
                &format!("بیوپاری {}", urdu_number(n)),
                &format!("Seller {n}"),
                true,
                "",
            )
        })
        .collect();
    let buyers = (0..100)
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
    Big { w, sellers, buyers }
}

/// The rate of line number `n` of the whole draft: 300, 305 ... 480, then again. Steps of 5 make
/// many lines whose 6% ends in half a rupee.
fn rate_of(n: usize) -> i64 {
    300 + (i64::try_from(n).unwrap() % 37) * 5
}

/// A lot of `lines` lines of 1 نگ each, starting at line number `first` of the draft. Freight 200
/// and labour 50 on every lot.
fn lot_of(b: &Big, index: usize, seller: i64, item: i64, first: usize, lines: usize) -> LotInput {
    let mut lot = bare_lot(
        &b.w,
        &format!("lot-{index}"),
        seller,
        item,
        &lines.to_string(),
    );
    lot.freight = Some("200".into());
    lot.labour = manual("50");
    lot.sales = (0..lines)
        .map(|i| {
            let n = first + i;
            sale(
                &format!("line-{n}"),
                b.buyers[n % b.buyers.len()],
                "1",
                &rate_of(n).to_string(),
            )
        })
        .collect();
    lot
}

/// 50 lots of 20 lines: lot `i` has seller `i % 10` and item banana (first 10 of each 20) or apple.
fn fifty_by_twenty(b: &Big) -> BikriDraft {
    let lots = (0..50)
        .map(|i| {
            let item = if (i / 10) % 2 == 0 {
                b.w.banana
            } else {
                b.w.apple
            };
            lot_of(b, i, b.sellers[i % 10], item, i * 20, 20)
        })
        .collect();
    draft(lots)
}

/// 49 lots of 1 line and one lot of 951 lines.
fn skewed(b: &Big) -> BikriDraft {
    let mut lots: Vec<LotInput> = (0..49)
        .map(|i| lot_of(b, i, b.sellers[i % 10], b.w.banana, i, 1))
        .collect();
    lots.push(lot_of(b, 49, b.sellers[9], b.w.apple, 49, 951));
    draft(lots)
}

/// The figures of a whole draft of fruit lots by plain integer maths. `lots` is the number of
/// lots and `lines` the number of lines in all; every lot has freight 200 and labour 50.
struct Expected {
    gross: i64,
    seller_comm: i64,
    buyer_comm: i64,
    laga: i64,
    buyer_total: i64,
    freight: i64,
    labour: i64,
    deductions: i64,
    net: i64,
    shared: i64,
    split: [i64; 3],
}

fn expected(lots: usize, lines: usize) -> Expected {
    let (mut gross, mut seller_comm, mut buyer_comm, mut laga, mut buyer_total) = (0, 0, 0, 0, 0);
    for n in 0..lines {
        let value = rate_of(n);
        gross += value;
        seller_comm += percent(value);
        buyer_comm += percent(value);
        laga += 8;
        buyer_total += value + percent(value) + 8;
    }
    let lots = i64::try_from(lots).unwrap();
    let (freight, labour) = (200 * lots, 50 * lots);
    let deductions = freight + seller_comm + labour;
    let shared = seller_comm + buyer_comm + laga;
    let mut parts = [shared * 40 / 100, shared * 30 / 100, shared * 30 / 100];
    parts[0] += shared - parts.iter().sum::<i64>();
    Expected {
        gross,
        seller_comm,
        buyer_comm,
        laga,
        buyer_total,
        freight,
        labour,
        deductions,
        net: gross - deductions,
        shared,
        split: parts,
    }
}

fn check_figures(p: &BikriPreview, e: &Expected, lots: usize, lines: usize) {
    assert!(
        p.errors.is_empty(),
        "errors: {:?}",
        &p.errors[..p.errors.len().min(5)]
    );
    assert!(p.warnings.is_empty(), "warnings: {:?}", p.warnings);
    assert_eq!(p.lots.len(), lots);
    let standing: usize = p
        .lots
        .iter()
        .map(|l| l.lines.iter().filter(|line| !line.is_blank).count())
        .sum();
    assert_eq!(standing, lines);
    let s = &p.summary;
    assert_eq!(r(s.gross), e.gross, "gross");
    assert_eq!(r(s.seller_comm), e.seller_comm, "seller commission");
    assert_eq!(r(s.buyer_comm), e.buyer_comm, "buyer commission");
    assert_eq!(r(s.laga), e.laga, "laga");
    assert_eq!(r(s.buyer_total), e.buyer_total, "buyer total");
    assert_eq!(r(s.freight), e.freight, "freight");
    assert_eq!(r(s.labour), e.labour, "labour");
    assert_eq!(r(s.deductions), e.deductions, "deductions");
    assert_eq!(r(s.net), e.net, "net");
    assert_eq!(r(s.shared_income), e.shared, "shared income");
    let split: Vec<i64> = p.split.iter().map(|part| r(part.income_paisa)).collect();
    assert_eq!(split, e.split, "the partners' parts");
    assert_eq!(
        split.iter().sum::<i64>(),
        e.shared,
        "the split adds back to the income"
    );
    // The lots add up to the whole: no lot is lost or counted twice.
    let lots_gross: i64 = p
        .lots
        .iter()
        .map(|l| r(l.totals.as_ref().unwrap().gross))
        .sum();
    assert_eq!(lots_gross, e.gross, "the lots' gross adds up to the whole");
    let lots_net: i64 = p
        .lots
        .iter()
        .map(|l| r(l.totals.as_ref().unwrap().net))
        .sum();
    assert_eq!(lots_net, e.net, "the lots' net adds up to the whole");
}

/// What the save left in the file, by plain SQL: one balanced entry, the parts of the split, the
/// lines and the stock.
fn check_file(b: &Big, e: &Expected, lines: usize) {
    let conn = raw(&b.w.dir);
    let one = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
    assert_eq!(one("SELECT COUNT(*) FROM bikri_entries"), 1);
    assert_eq!(
        one("SELECT COUNT(*) FROM bikri_sales WHERE is_active = 1"),
        i64::try_from(lines).unwrap()
    );
    assert_eq!(
        one("SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'"),
        0
    );
    // Every posted entry balances, and the whole book does.
    assert_eq!(
        one(
            "SELECT COUNT(*) FROM (SELECT entry_id FROM journal_lines GROUP BY entry_id HAVING SUM(dr_paisa) <> SUM(cr_paisa))"
        ),
        0
    );
    assert_eq!(
        one("SELECT COALESCE(SUM(dr_paisa), 0) - COALESCE(SUM(cr_paisa), 0) FROM journal_lines"),
        0
    );
    // The partners' rows add up to the shared income of the posting (PRD FR-6.5, C9, C10).
    assert_eq!(
        one("SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows WHERE kind = 'income'"),
        e.shared * 100
    );
    // Every sale line is whole rupees (M1).
    assert_eq!(
        one("SELECT COUNT(*) FROM bikri_sales WHERE buyer_total_paisa % 100 <> 0"),
        0
    );
}

#[test]
fn fifty_lots_of_twenty_lines_are_worked_out_exactly_saved_once_and_named_in_order() {
    let b = big_world();
    let d = fifty_by_twenty(&b);
    let e = expected(50, 1_000);

    let first = bikri::preview(&b.w.env.ctx, &d).unwrap();
    check_figures(&first, &e, 50, 1_000);
    // Lot names (FR-12.45): the draft holds 20 pairs of seller and item, each used 2 or 3 times
    // in a row of lots, so the counters climb inside the draft: lots 0 to 19 are the first of
    // their pair (banana for 0..10, apple for 10..20), lots 20 to 39 the second, and so on.
    let counters: Vec<Option<i64>> = first.lots.iter().map(|l| l.lot_counter).collect();
    let want: Vec<Option<i64>> = (0..50_i64).map(|i| Some(i / 20 + 1)).collect();
    assert_eq!(counters, want, "the counter shown before the save");

    let mut times: Vec<Duration> = (0..5)
        .map(|_| {
            let (p, t) = timed(|| bikri::preview(&b.w.env.ctx, &d).unwrap());
            assert_eq!(p, first, "the same draft gives the same preview");
            t
        })
        .collect();
    times.sort();
    let median = times[2];
    let bound = if cfg!(debug_assertions) {
        PREVIEW_BOUND_DEBUG
    } else {
        PREVIEW_BOUND_RELEASE
    };
    eprintln!(
        "50 lots x 20 lines: preview median {:.1} ms (min {:.1}, max {:.1}); bound {:.0} ms",
        ms(median),
        ms(times[0]),
        ms(times[4]),
        ms(bound)
    );
    assert!(median <= bound, "the preview took {:.1} ms", ms(median));

    let (saved, save_time) = timed(|| {
        bikri::save(
            &b.w.env.ctx,
            &b.w.owner,
            &BikriSaveInput {
                draft: d.clone(),
                confirm_cash_below_zero: true,
            },
        )
        .unwrap()
    });
    let bound = if cfg!(debug_assertions) {
        SAVE_BOUND_DEBUG
    } else {
        SAVE_BOUND_RELEASE
    };
    eprintln!(
        "50 lots x 20 lines: save {:.1} ms; bound {:.0} ms",
        ms(save_time),
        ms(bound)
    );
    assert!(save_time <= bound, "the save took {:.1} ms", ms(save_time));
    assert_eq!(
        saved.preview, first,
        "the save writes the numbers the preview showed (B5)"
    );

    let detail = bikri::get(&b.w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(detail.lots.len(), 50);
    let sales: usize = detail.lots.iter().map(|l| l.sales.len()).sum();
    assert_eq!(sales, 1_000);
    let saved_counters: Vec<i64> = detail.lots.iter().map(|l| l.lot_counter).collect();
    let want: Vec<i64> = (0..50_i64).map(|i| i / 20 + 1).collect();
    assert_eq!(
        saved_counters, want,
        "the counter saved is the counter shown (FR-12.45)"
    );
    assert_eq!(
        detail.lots.iter().map(|l| l.gross.rupees()).sum::<i64>(),
        e.gross
    );
    check_file(&b, &e, 1_000);
    common::assert_chain_ok(&b.w.dir);
}

#[test]
fn a_skewed_draft_of_49_lots_of_one_line_and_one_lot_of_951_lines() {
    let b = big_world();
    let d = skewed(&b);
    let e = expected(50, 1_000);
    let p = bikri::preview(&b.w.env.ctx, &d).unwrap();
    check_figures(&p, &e, 50, 1_000);
    let (saved, t) = timed(|| {
        bikri::save(
            &b.w.env.ctx,
            &b.w.owner,
            &BikriSaveInput {
                draft: d,
                confirm_cash_below_zero: true,
            },
        )
        .unwrap()
    });
    eprintln!(
        "49 lots of 1 line + 1 lot of 951 lines: save {:.1} ms",
        ms(t)
    );
    assert_eq!(saved.preview, p);
    check_file(&b, &e, 1_000);
}

/// Everything a refused draft must leave alone.
fn counts(b: &Big) -> Vec<i64> {
    let conn = raw(&b.w.dir);
    [
        "bikri_entries",
        "bikri_lots",
        "bikri_sales",
        "journal_entries",
        "journal_lines",
        "partner_share_rows",
        "stock_moves",
        "analytics_facts",
        "lot_counters",
        "audit_log",
        "drafts",
    ]
    .iter()
    .map(|t| {
        conn.query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| r.get(0))
            .unwrap()
    })
    .collect()
}

#[test]
fn the_limits_are_refused_with_a_field_and_save_nothing() {
    let b = big_world();
    let before = counts(&b);
    let save = |d: BikriDraft| {
        bikri::save(
            &b.w.env.ctx,
            &b.w.owner,
            &BikriSaveInput {
                draft: d,
                confirm_cash_below_zero: true,
            },
        )
    };

    // 51 lots of 1 line: the lots are over the limit (NFR-SCL-3).
    let lots: Vec<LotInput> = (0..51)
        .map(|i| lot_of(&b, i, b.sellers[i % 10], b.w.banana, i, 1))
        .collect();
    let d = draft(lots);
    let p = bikri::preview(&b.w.env.ctx, &d).unwrap();
    assert!(
        has(&p.errors, ErrorCode::ErrOutOfRange, "lots"),
        "{:?}",
        p.errors
    );
    match save(d) {
        Err(ServiceError::Rejected(items)) => {
            assert!(
                items
                    .iter()
                    .any(|e| e.code == ErrorCode::ErrOutOfRange
                        && e.field.as_deref() == Some("lots"))
            );
        }
        other => panic!("51 lots were not refused: {other:?}"),
    }

    // 1,001 lines in one lot.
    let d = draft(vec![lot_of(&b, 0, b.sellers[0], b.w.banana, 0, 1_001)]);
    let p = bikri::preview(&b.w.env.ctx, &d).unwrap();
    assert!(
        has(&p.errors, ErrorCode::ErrOutOfRange, "lots[0].sales"),
        "{:?}",
        p.errors
    );
    assert!(matches!(save(d), Err(ServiceError::Rejected(_))));

    // 1,001 lines spread over 2 lots: the limit is for the whole بکری, not for each lot.
    let d = draft(vec![
        lot_of(&b, 0, b.sellers[0], b.w.banana, 0, 500),
        lot_of(&b, 1, b.sellers[1], b.w.banana, 500, 501),
    ]);
    let p = bikri::preview(&b.w.env.ctx, &d).unwrap();
    assert!(
        p.errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrOutOfRange && e.field.ends_with("sales")),
        "1,001 lines over two lots were not refused: {:?}",
        &p.errors[..p.errors.len().min(3)]
    );
    assert!(matches!(save(d), Err(ServiceError::Rejected(_))));

    // 1,000 lines and the 100 blank rows the screen may keep: not refused.
    let mut lots: Vec<LotInput> = (0..50)
        .map(|i| lot_of(&b, i, b.sellers[i % 10], b.w.banana, i * 20, 20))
        .collect();
    for lot in &mut lots {
        lot.sales.push(blank_sale("blank-a", ""));
        lot.sales.push(blank_sale("blank-b", ""));
    }
    let d = draft(lots);
    let p = bikri::preview(&b.w.env.ctx, &d).unwrap();
    assert!(
        p.errors.is_empty(),
        "{:?}",
        &p.errors[..p.errors.len().min(3)]
    );
    assert_eq!(p.lots.iter().map(|l| l.lines.len()).sum::<usize>(), 1_100);

    // 101 blank rows: refused with the blank-row limit, not the line limit.
    let mut lot = lot_of(&b, 0, b.sellers[0], b.w.banana, 0, 1);
    for i in 0..101 {
        lot.sales.push(blank_sale(&format!("blank-{i}"), ""));
    }
    let p = bikri::preview(&b.w.env.ctx, &draft(vec![lot])).unwrap();
    assert!(
        has(&p.errors, ErrorCode::ErrOutOfRange, "lots[0].sales"),
        "{:?}",
        p.errors
    );

    // Nothing the refused drafts did is in the file.
    assert_eq!(counts(&b), before, "a refused بکری wrote something");
}

#[test]
fn one_line_over_a_lot_in_a_big_draft_is_refused_at_its_own_cell_and_saves_nothing() {
    // E103: a quantity above what the lot has left, deep inside a draft of 1,000 lines.
    let b = big_world();
    let before = counts(&b);
    let mut d = fifty_by_twenty(&b);
    // Lot 37, its last line: sell 5 where the lot received 20 and 19 are already sold.
    d.lots[37].sales[19].qty = "5".into();
    let p = bikri::preview(&b.w.env.ctx, &d).unwrap();
    let over: Vec<_> = p
        .errors
        .iter()
        .filter(|e| e.code == ErrorCode::ErrQtyExceedsLot)
        .collect();
    assert_eq!(over.len(), 1, "{:?}", p.errors);
    assert_eq!(over[0].field, "lots[37].sales[19].qty");
    let saved = bikri::save(
        &b.w.env.ctx,
        &b.w.owner,
        &BikriSaveInput {
            draft: d,
            confirm_cash_below_zero: true,
        },
    );
    assert!(matches!(saved, Err(ServiceError::Rejected(_))), "{saved:?}");
    assert_eq!(counts(&b), before);
}
