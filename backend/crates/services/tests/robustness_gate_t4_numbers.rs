//! Gate lane T4: bad numbers in a بکری draft, and the structure edge cases E1, E2 and E7 (plan 80;
//! Code Standards 4.2, 4.3, V0, M7, M8; PRD FR-7.48 sums, FR-12.44, E1, E2, E7, E24, E31, E42,
//! E99, E103; NFR-SCL-3). Owner: `robustness`.
//!
//! The pure parsers are attacked in `core/tests/robustness/`. Here the same values go through
//! `bikri::preview` and `bikri::save`, so the proof is about the whole path: a bad value is told at
//! its own cell with a code, the save of a draft with an error writes nothing, and a good value
//! gives the exact figure by plain integer maths.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::raw;
use mandi_services::ServiceError;
use mandi_services::bikri;
use mandi_services::core::bikri::{BikriDraft, BikriPreview, BikriSaveInput, FreightPaidBy};
use mandi_services::core::error::ErrorCode;
use support::*;

/// A draft of one lot of Rahim's banana, 50 received, and one line: Akram, 20 at 600. Nothing
/// else is typed on the lot, so the rules give the charges.
fn base(w: &World) -> BikriDraft {
    let lot = bare_lot(w, "lot-1", w.rahim, w.banana, "50");
    draft(vec![with_sales(
        lot,
        vec![sale("line-1", w.akram, "20", "600")],
    )])
}

/// How many rows of the tables a save writes to. A refused draft must leave them all alone.
fn counts(w: &World) -> Vec<i64> {
    let conn = raw(&w.dir);
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

fn try_save(
    w: &World,
    d: &BikriDraft,
) -> Result<mandi_services::core::bikri::BikriSaved, ServiceError> {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: d.clone(),
            confirm_cash_below_zero: true,
        },
    )
}

/// The refusal of a value at its cell: the preview names the field with one of the codes, the save
/// is refused and the file is the same afterwards.
fn refused_at(w: &World, d: &BikriDraft, field: &str, codes: &[ErrorCode]) {
    let before = counts(w);
    let p = bikri::preview(&w.env.ctx, d).unwrap();
    assert!(
        p.errors
            .iter()
            .any(|e| e.field == field && codes.contains(&e.code)),
        "wanted one of {codes:?} at {field}, got {:?}",
        p.errors
    );
    assert!(
        matches!(try_save(w, d), Err(ServiceError::Rejected(_))),
        "{field}: saved"
    );
    assert_eq!(
        counts(w),
        before,
        "{field}: a refused draft wrote something"
    );
}

fn gross(p: &BikriPreview) -> i64 {
    r(p.summary.gross)
}

#[test]
fn quantity_cells_at_every_edge() {
    let w = world();
    let qty = |text: &str| {
        let mut d = base(&w);
        d.lots[0].sales[0].qty = text.into();
        d
    };
    // E42: a sum is read, a result below 0 is not (FR-7.48).
    let p = bikri::preview(&w.env.ctx, &qty("20+25")).unwrap();
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    assert_eq!(gross(&p), 45 * 600, "20+25 is 45 at 600");
    let p = bikri::preview(&w.env.ctx, &qty(" 20 + 25 ")).unwrap();
    assert_eq!(gross(&p), 45 * 600, "spaces round the plus");
    let p = bikri::preview(&w.env.ctx, &qty("۲۰")).unwrap();
    assert_eq!(gross(&p), 20 * 600, "Urdu digits are read");
    let p = bikri::preview(&w.env.ctx, &qty("2\u{200b}0")).unwrap();
    assert_eq!(
        gross(&p),
        20 * 600,
        "a zero-width mark inside the number is ignored"
    );
    let field = "lots[0].sales[0].qty";
    use ErrorCode::*;
    for (text, codes) in [
        (
            "10-20",
            vec![ErrNegativeNotAllowed, ErrNotANumber, ErrMustBeMoreThanZero],
        ),
        ("20-20", vec![ErrMustBeMoreThanZero, ErrNotANumber]),
        ("5×3", vec![ErrNotANumber]),
        ("5÷3", vec![ErrNotANumber]),
        ("20+", vec![ErrNotANumber]),
        ("+20", vec![ErrNotANumber]),
        ("abc", vec![ErrNotANumber]),
        ("0", vec![ErrMustBeMoreThanZero]),
        (
            "-1",
            vec![ErrNegativeNotAllowed, ErrNotANumber, ErrMustBeMoreThanZero],
        ),
        (
            "1.5",
            vec![ErrDecimalsNotAllowed, ErrQtyNotWhole, ErrNotANumber],
        ),
        ("1e5", vec![ErrNotANumber]),
        ("1000001", vec![ErrQtyTooLarge, ErrQtyExceedsLot]),
        (
            "99999999999999999999",
            vec![ErrQtyTooLarge, ErrNotANumber, ErrAmountTooLarge],
        ),
        ("\u{202e}20", vec![ErrNotANumber, ErrControlChars]),
        ("2\u{0}0", vec![ErrNotANumber, ErrControlChars]),
    ] {
        refused_at(&w, &qty(text), field, &codes);
    }
    // 51 against 50 received (E103).
    refused_at(&w, &qty("51"), field, &[ErrQtyExceedsLot]);
}

#[test]
fn rate_and_paid_and_charge_cells_at_every_edge() {
    use ErrorCode::*;
    let w = world();
    let with = |f: &dyn Fn(&mut BikriDraft)| {
        let mut d = base(&w);
        f(&mut d);
        d
    };
    // Good values by other spellings.
    for (text, rupees) in [
        ("۶۰۰", 600),
        ("6\u{200b}00", 600),
        ("500+100", 600),
        ("1,200", 1_200),
    ] {
        let d = with(&|d| d.lots[0].sales[0].seller_rate = text.into());
        let p = bikri::preview(&w.env.ctx, &d).unwrap();
        assert!(p.errors.is_empty(), "{text}: {:?}", p.errors);
        assert_eq!(gross(&p), 20 * rupees, "{text}");
    }
    let rate = "lots[0].sales[0].seller_rate";
    for (text, codes) in [
        ("0", vec![ErrMustBeMoreThanZero]),
        ("-5", vec![ErrNegativeNotAllowed, ErrNotANumber]),
        ("850.50", vec![ErrDecimalsNotAllowed]),
        ("600.", vec![ErrDecimalsNotAllowed, ErrNotANumber]),
        ("1e5", vec![ErrNotANumber]),
        ("abc", vec![ErrNotANumber]),
        ("10,00,00,00,001", vec![ErrAmountTooLarge]),
        ("9223372036854775807", vec![ErrAmountTooLarge, ErrOverflow]),
        (
            "99999999999999999999999",
            vec![ErrAmountTooLarge, ErrNotANumber],
        ),
        (",100", vec![ErrNotANumber]),
        ("1,2,3", vec![ErrNotANumber]),
    ] {
        refused_at(
            &w,
            &with(&|d| d.lots[0].sales[0].seller_rate = text.into()),
            rate,
            &codes,
        );
    }
    // Paid on the spot (FR-12.44): the most is the line's buyer total, 12,000 + 720 + 160 = 12,880.
    let paid = "lots[0].sales[0].paid";
    let d = with(&|d| d.lots[0].sales[0].paid = Some("۵۰۰۰".into()));
    assert!(bikri::preview(&w.env.ctx, &d).unwrap().errors.is_empty());
    for (text, codes) in [
        ("850.50", vec![ErrDecimalsNotAllowed]),
        ("-1", vec![ErrNegativeNotAllowed, ErrNotANumber]),
        ("12,881", vec![ErrPaidExceedsTotal]),
        ("99999999", vec![ErrPaidExceedsTotal]),
        ("abc", vec![ErrNotANumber]),
    ] {
        refused_at(
            &w,
            &with(&|d| d.lots[0].sales[0].paid = Some(text.into())),
            paid,
            &codes,
        );
    }
    // The lot's own cells.
    for (field, text, codes) in [
        (
            "lots[0].freight",
            "10,00,00,00,001",
            vec![ErrAmountTooLarge],
        ),
        (
            "lots[0].freight",
            "-1",
            vec![ErrNegativeNotAllowed, ErrNotANumber],
        ),
        ("lots[0].freight", "1.5", vec![ErrDecimalsNotAllowed]),
        ("lots[0].cash_paid", "850.50", vec![ErrDecimalsNotAllowed]),
        ("lots[0].store_rent", "abc", vec![ErrNotANumber]),
        ("lots[0].qty_received", "0", vec![ErrMustBeMoreThanZero]),
        ("lots[0].qty_received", "1000001", vec![ErrQtyTooLarge]),
        (
            "lots[0].qty_received",
            "-5",
            vec![ErrNegativeNotAllowed, ErrNotANumber, ErrMustBeMoreThanZero],
        ),
    ] {
        let d = with(&|d| {
            let lot = &mut d.lots[0];
            match field {
                "lots[0].freight" => lot.freight = Some(text.into()),
                "lots[0].cash_paid" => lot.cash_paid = Some(text.into()),
                "lots[0].store_rent" => lot.store_rent = Some(text.into()),
                _ => lot.qty_received = text.into(),
            }
        });
        refused_at(&w, &d, field, &codes);
    }
    // A comma in the wrong place and a lakh grouping are both read the way the shop writes them.
    let d = with(&|d| d.lots[0].freight = Some("1,00,000".into()));
    let p = bikri::preview(&w.env.ctx, &d).unwrap();
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    assert_eq!(r(p.summary.freight), 100_000);
}

#[test]
fn the_largest_values_never_panic_and_never_overflow_the_books() {
    // The largest rate (10,00,00,00,000), the largest quantity (1,000,000) and 50 lots of it:
    // the figures are either exact or refused with a code, and a refusal writes nothing.
    let w = world();
    let before = counts(&w);
    let lots: Vec<_> = (0..50)
        .map(|i| {
            let mut lot = bare_lot(&w, &format!("lot-{i}"), w.rahim, w.banana, "1000000");
            lot.sales = vec![sale(
                &format!("line-{i}"),
                w.akram,
                "1000000",
                "10,00,00,00,000",
            )];
            lot
        })
        .collect();
    let d = draft(lots);
    let p = bikri::preview(&w.env.ctx, &d).unwrap();
    if p.errors.is_empty() {
        // Accepted: then the figure must be exact, and the journal must hold it.
        let want: i128 = 50 * 1_000_000_i128 * 1_000_000_000_i128;
        assert_eq!(i128::from(gross(&p)), want);
        let saved = try_save(&w, &d);
        println!(
            "the largest draft was ACCEPTED: gross {want} rupees, save {:?}",
            saved.as_ref().map(|s| s.number)
        );
        if saved.is_ok() {
            let conn = raw(&w.dir);
            let (dr, cr): (i64, i64) = conn
                .query_row(
                    "SELECT COALESCE(SUM(dr_paisa),0), COALESCE(SUM(cr_paisa),0) FROM journal_lines",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(dr, cr);
        }
    } else {
        assert!(
            p.errors.iter().all(|e| !e.field.is_empty()),
            "{:?}",
            &p.errors[..3.min(p.errors.len())]
        );
        assert!(matches!(try_save(&w, &d), Err(ServiceError::Rejected(_))));
        assert_eq!(counts(&w), before);
        println!(
            "the largest draft was refused with {:?}",
            p.errors
                .iter()
                .map(|e| e.code)
                .collect::<std::collections::BTreeSet<_>>()
        );
    }
}

/// E1 (one truck, several sellers) and E2 (one seller, several items): lots of one بکری keep their
/// own freight and "freight paid by", each seller is credited his own net, and the lot counters of
/// one seller go by item (FR-12.45).
#[test]
fn one_truck_with_several_sellers_and_one_seller_with_several_items() {
    let w = world();
    let mut a = bare_lot(&w, "a", w.rahim, w.banana, "10");
    a.freight = Some("3,000".into());
    let a = with_sales(a, vec![sale("a1", w.akram, "10", "600")]);
    // Karim's driver was paid by Karim himself: the freight is shown, not deducted or posted (E6).
    let mut b = bare_lot(&w, "b", w.karim, w.apple, "10");
    b.freight = Some("1,000".into());
    b.freight_paid_by = FreightPaidBy::Seller;
    let b = with_sales(b, vec![sale("b1", w.arshad, "10", "500")]);
    // Rahim again, a second item (E2), and a second lot of banana (counter 2).
    let mut c = bare_lot(&w, "c", w.rahim, w.apple, "5");
    c.freight = Some("200".into());
    let c = with_sales(c, vec![sale("c1", w.akram, "5", "400")]);
    let mut e = bare_lot(&w, "e", w.rahim, w.banana, "4");
    e.freight = Some("100".into());
    let e = with_sales(e, vec![sale("e1", w.arshad, "4", "300")]);
    let d = draft(vec![a, b, c, e]);

    let p = bikri::preview(&w.env.ctx, &d).unwrap();
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let counters: Vec<Option<i64>> = p.lots.iter().map(|l| l.lot_counter).collect();
    assert_eq!(
        counters,
        vec![Some(1), Some(1), Some(1), Some(2)],
        "Rahim: banana 1, apple 1, banana 2"
    );
    // Freight: only what the firm paid is deducted (3,000 + 200 + 100); Karim's 1,000 is not.
    assert_eq!(r(p.summary.freight), 3_300);
    let saved = try_save(&w, &d).unwrap();
    assert_eq!(saved.preview, p);
    let conn = raw(&w.dir);
    let one = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
    let freight_account = id_of(&w.env, "02010002");
    assert_eq!(
        one(&format!(
            "SELECT COALESCE(SUM(cr_paisa) - SUM(dr_paisa), 0) FROM journal_lines WHERE account_id = {freight_account}"
        )),
        3_300 * 100,
        "the freight account owes the transporters the firm's freight only"
    );
    assert_eq!(
        one(
            "SELECT COUNT(*) FROM (SELECT entry_id FROM journal_lines GROUP BY entry_id HAVING SUM(dr_paisa) <> SUM(cr_paisa))"
        ),
        0
    );
    // Each seller's credit is his own lots' net: Karim's lot has no freight deduction.
    let net_of = |seller: i64| -> i64 {
        one(&format!(
            "SELECT COALESCE(SUM(cr_paisa) - SUM(dr_paisa), 0) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id WHERE l.account_id = {seller} AND e.posting_type = 'bikri_first'"
        )) / 100
    };
    let pct = |v: i64| (v * 6 + 50) / 100;
    // Seller credit of a lot = gross - seller commission - freight paid by the firm (no other charge).
    let karim_gross = 10 * 500;
    assert_eq!(net_of(w.karim), karim_gross - pct(karim_gross));
    let rahim_gross = 10 * 600 + 5 * 400 + 4 * 300;
    let rahim_comm = pct(10 * 600) + pct(5 * 400) + pct(4 * 300);
    assert_eq!(
        net_of(w.rahim),
        rahim_gross - rahim_comm - 3_000 - 200 - 100
    );
}

/// E7: the same buyer on many lines, in one lot and across lots: each line is its own bill, his
/// journal debit is the sum of his lines, and the books balance.
#[test]
fn the_same_buyer_on_many_lines_owes_the_sum_of_his_lines() {
    let w = world();
    let lines = [("10", "600"), ("5", "610"), ("5", "620"), ("3", "555")];
    let mut a = bare_lot(&w, "a", w.rahim, w.banana, "23");
    a.sales = lines
        .iter()
        .enumerate()
        .map(|(i, (q, rate))| sale(&format!("a{i}"), w.akram, q, rate))
        .collect();
    let mut b = bare_lot(&w, "b", w.karim, w.apple, "7");
    b.sales = vec![sale("b0", w.akram, "7", "480")];
    let d = draft(vec![a, b]);
    let p = bikri::preview(&w.env.ctx, &d).unwrap();
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let saved = try_save(&w, &d).unwrap();
    assert_eq!(saved.preview, p);
    let pct = |v: i64| (v * 6 + 50) / 100;
    let bill = |q: i64, rate: i64| q * rate + pct(q * rate) + 8 * q;
    let owed: i64 = lines
        .iter()
        .map(|(q, rate)| bill(q.parse().unwrap(), rate.parse().unwrap()))
        .sum::<i64>()
        + bill(7, 480);
    let conn = raw(&w.dir);
    let balance: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(dr_paisa) - SUM(cr_paisa), 0) FROM journal_lines WHERE account_id = ?1",
            [w.akram],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        balance,
        owed * 100,
        "his balance is the sum of his five bills"
    );
    let sale_rows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM bikri_sales WHERE buyer_account_id = ?1 AND is_active = 1",
            [w.akram],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(sale_rows, 5);
}
