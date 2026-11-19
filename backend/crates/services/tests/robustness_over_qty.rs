//! Plan 57 T3, attack on the over-quantity rule (PRD FR-12.6 as changed in v1.28, plan 57 sections 2 and 7).
//! Owner: `robustness`.
//!
//! The rule: a sale line that sells more than the lot has left is refused with `ERR_QTY_EXCEEDS_LOT` at its
//! quantity cell (`available` = what is left), has no figures, takes no stock, and the lot, the whole بکری, the
//! journal and the split are built from the accepted lines only. The line that crosses is the one refused, in
//! line order. A save with ANY refused line writes nothing at all.
//!
//! 1. `every_order_of_lines_refuses_exactly_the_lines_that_cross`: every run of 1 to 3 lines over a lot of 50
//!    (quantities at and around the limits), every order of three multisets, and 150 pseudo-random runs of up
//!    to 8 lines. For each: the lines refused are exactly those a plain model refuses, `available` is right,
//!    a refused line has no figures, sold and balance count the accepted lines only, and the summary, the
//!    split and the lot totals EQUAL those of the same draft with the refused lines taken out. A preview writes
//!    nothing.
//! 2. `fifty_lines_on_one_lot`: 50 lines on lots of 40, 50 and 51 (the 51st quantity runs out at the right
//!    line), and the preview stays quick.
//! 3. `a_save_with_a_refused_line_writes_nothing_at_all`: the rows of eleven tables and the sums of stock,
//!    journal and split are the same before and after, the audit chain holds, and the same draft with the
//!    over quantity fixed then saves.
//! 4. `a_later_sale_over_the_stock_left_is_refused_and_writes_nothing`.
//! 5. `an_edit_that_takes_a_line_over_the_lot_is_refused_and_the_old_posting_stays`.
//! 6. `an_own_goods_lot_over_its_purchase_is_refused_and_the_purchase_keeps_its_stock`.
//! 7. `a_typed_sum_is_worked_out_before_the_lot_is_checked`: `20+40`, `20+30`, `20+31`, `60-20`, `20-30`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::time::Instant;

use common::{assert_chain_ok, raw};
use mandi_services::bikri;
use mandi_services::core::bikri::{
    BikriDraft, BikriPreview, BikriSaveInput, EditOpenInput, EditPreviewInput, EditSaveInput,
    LaterSaleInput, LaterSaleSaveInput, LotInput, SaleInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{EntryPartInput, MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, purchases, trips};
use support::*;

// ---- helpers ------------------------------------------------------------------------------------------

/// The lines sold to buyers in turn, one per quantity, at 1,200 each.
fn lines(w: &World, qtys: &[&str]) -> Vec<SaleInput> {
    qtys.iter()
        .enumerate()
        .map(|(i, q)| {
            let buyer = if i % 2 == 0 { w.akram } else { w.arshad };
            sale(&format!("s-{i}"), buyer, q, "1200")
        })
        .collect()
}

fn one_lot(w: &World, received: i64, sales: Vec<SaleInput>) -> BikriDraft {
    draft(vec![with_sales(
        lot(w, "lot-1", w.rahim, w.banana, &received.to_string()),
        sales,
    )])
}

/// (field, available) of every `ERR_QTY_EXCEEDS_LOT`, in the order of the errors.
fn crossings(errors: &[FieldError]) -> Vec<(String, i64)> {
    errors
        .iter()
        .filter(|e| e.code == ErrorCode::ErrQtyExceedsLot)
        .map(|e| {
            let available = match e.values.get("available") {
                Some(ErrorValue::Number(n)) => *n,
                _ => -1,
            };
            (e.field.clone(), available)
        })
        .collect()
}

/// What a plain model says about a lot of `lot` and the quantities in order: for each line the quantity
/// left when it comes, or `None` when it fits. The accepted quantities leave the lot; a refused one does not.
fn model(lot: i64, qtys: &[i64]) -> Vec<Option<i64>> {
    let mut left = lot;
    qtys.iter()
        .map(|q| {
            if *q <= left {
                left -= q;
                None
            } else {
                Some(left)
            }
        })
        .collect()
}

/// The state of the books that a refused line must never touch: the rows of eleven tables and the sums of
/// stock, journal and the partner split.
fn snapshot(w: &World) -> Vec<(String, i64)> {
    let conn = raw(&w.dir);
    let mut out = Vec::new();
    for table in [
        "bikri_entries",
        "bikri_lots",
        "bikri_sales",
        "journal_entries",
        "journal_lines",
        "partner_share_rows",
        "stock_moves",
        "analytics_facts",
        "bill_allocations",
        "advance_allocations",
        "audit_log",
    ] {
        let n: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap_or_else(|e| panic!("count of {table}: {e}"));
        out.push((table.to_owned(), n));
    }
    for (name, sql) in [
        (
            "lots.qty_sold",
            "SELECT COALESCE(SUM(qty_sold), 0) FROM bikri_lots",
        ),
        (
            "lots.qty_received",
            "SELECT COALESCE(SUM(qty_received), 0) FROM bikri_lots",
        ),
        (
            "lots.qty_written_off",
            "SELECT COALESCE(SUM(qty_written_off), 0) FROM bikri_lots",
        ),
        (
            "lots.gross",
            "SELECT COALESCE(SUM(gross_paisa), 0) FROM bikri_lots",
        ),
        (
            "purchases.qty_sold",
            "SELECT COALESCE(SUM(qty_sold), 0) FROM purchases",
        ),
        ("purchases.rows", "SELECT COUNT(*) FROM purchases"),
        ("stock.qty", "SELECT COALESCE(SUM(qty), 0) FROM stock_moves"),
        (
            "journal.dr",
            "SELECT COALESCE(SUM(dr_paisa), 0) FROM journal_lines",
        ),
        (
            "journal.cr",
            "SELECT COALESCE(SUM(cr_paisa), 0) FROM journal_lines",
        ),
        (
            "split.sum",
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows",
        ),
    ] {
        let n: i64 = conn
            .query_row(sql, [], |r| r.get(0))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        out.push((name.to_owned(), n));
    }
    out
}

fn refused_fields(error: &ServiceError) -> Vec<String> {
    match error {
        ServiceError::Rejected(items) => items
            .iter()
            .filter(|i| i.code == ErrorCode::ErrQtyExceedsLot)
            .filter_map(|i| i.field.clone())
            .collect(),
        _ => Vec::new(),
    }
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

/// The checks of one preview against the model. `qtys` are the quantities of the lines in order.
fn check_preview(w: &World, lot_qty: i64, qtys: &[i64], what: &str) {
    let texts: Vec<String> = qtys.iter().map(i64::to_string).collect();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let d = one_lot(w, lot_qty, lines(w, &refs));
    let before = snapshot(w);
    let p = preview(w, &d);
    assert_eq!(snapshot(w), before, "{what}: a preview wrote something");
    let want = model(lot_qty, qtys);
    let wanted: Vec<(String, i64)> = want
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.map(|a| (format!("lots[0].sales[{i}].qty"), a)))
        .collect();
    let mut got = crossings(&p.errors);
    got.sort();
    let mut wanted_sorted = wanted.clone();
    wanted_sorted.sort();
    assert_eq!(
        got, wanted_sorted,
        "{what}: the refused lines or what is left differ ({qtys:?} on {lot_qty})"
    );
    let lot = &p.lots[0];
    for (i, a) in want.iter().enumerate() {
        let line = &lot.lines[i];
        assert_eq!(
            line.calc.is_none(),
            a.is_some(),
            "{what}: line {i} figures vs refusal ({qtys:?} on {lot_qty})"
        );
    }
    let accepted: Vec<i64> = qtys
        .iter()
        .zip(&want)
        .filter(|(_, a)| a.is_none())
        .map(|(q, _)| *q)
        .collect();
    let sold: i64 = accepted.iter().sum();
    assert_eq!(
        lot.sold_qty, sold,
        "{what}: sold qty counts accepted lines only"
    );
    assert_eq!(lot.balance_qty, Some(lot_qty - sold), "{what}: balance");
    assert!(sold <= lot_qty, "{what}: the lot sold more than it has");
    if accepted.is_empty() {
        assert!(
            lot.totals.is_none(),
            "{what}: a lot with no accepted line has no totals"
        );
        assert_eq!(r(p.summary.gross), 0, "{what}: gross");
        assert_eq!(r(p.summary.shared_income), 0, "{what}: shared income");
        assert!(
            p.split.iter().all(|s| r(s.income_paisa) == 0),
            "{what}: the split holds a refused line"
        );
        return;
    }
    // The same draft with the refused lines taken out gives the same summary, split and lot totals.
    let kept: Vec<String> = accepted.iter().map(i64::to_string).collect();
    let kept_refs: Vec<&str> = kept.iter().map(String::as_str).collect();
    let mut only = one_lot(w, lot_qty, lines(w, &kept_refs));
    // The buyers rotate by position, so the accepted lines keep their own buyers.
    let accepted_at: Vec<usize> = want
        .iter()
        .enumerate()
        .filter(|(_, a)| a.is_none())
        .map(|(i, _)| i)
        .collect();
    for (k, at) in accepted_at.iter().enumerate() {
        only.lots[0].sales[k] = d.lots[0].sales[*at].clone();
    }
    let q = preview(w, &only);
    assert_eq!(
        p.summary, q.summary,
        "{what}: the summary holds a refused line ({qtys:?} on {lot_qty})"
    );
    assert_eq!(p.split, q.split, "{what}: the split holds a refused line");
    assert_eq!(
        lot.totals, q.lots[0].totals,
        "{what}: the lot totals hold a refused line"
    );
}

/// A small deterministic generator (no random crate needed for this).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, below: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % below
    }
}

fn permutations(items: &[i64]) -> Vec<Vec<i64>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(i);
        for mut tail in permutations(&rest) {
            tail.insert(0, head);
            out.push(tail);
        }
    }
    out.sort();
    out.dedup();
    out
}

// ---- 1 ------------------------------------------------------------------------------------------------

#[test]
fn every_order_of_lines_refuses_exactly_the_lines_that_cross() {
    let w = world_with(50_000, true);
    let edge = [1_i64, 10, 20, 30, 49, 50, 51];
    let mut runs: Vec<Vec<i64>> = Vec::new();
    for a in edge {
        runs.push(vec![a]);
        for b in edge {
            runs.push(vec![a, b]);
            for c in edge {
                runs.push(vec![a, b, c]);
            }
        }
    }
    // Every order of three multisets that cross the lot in different places.
    for set in [
        vec![30, 30, 20, 10, 5],
        vec![25, 25, 25],
        vec![51, 1, 50, 49],
    ] {
        runs.extend(permutations(&set));
    }
    // 150 pseudo-random runs of up to 8 lines.
    let mut rng = Lcg(57);
    for _ in 0..150 {
        let n = 1 + rng.next(8) as usize;
        runs.push((0..n).map(|_| 1 + rng.next(60) as i64).collect());
    }
    let total = runs.len();
    for (n, run) in runs.iter().enumerate() {
        check_preview(&w, 50, run, &format!("run {n} of {total}"));
    }
    // Another lot size, so the number 50 is not the thing that is right.
    for run in [
        vec![1, 1, 1],
        vec![7, 7, 7, 7],
        vec![100, 1],
        vec![99, 1, 1],
    ] {
        check_preview(&w, 99, &run, "lot of 99");
    }
}

// ---- 2 ------------------------------------------------------------------------------------------------

#[test]
fn fifty_lines_on_one_lot() {
    let w = world_with(50_000, true);
    for (lot_qty, each, name) in [
        (40_i64, 1_i64, "40 with 50 ones"),
        (50, 1, "50 with 50 ones"),
        (51, 2, "51 with 50 twos"),
    ] {
        let qtys = vec![each; 50];
        let texts: Vec<String> = qtys.iter().map(i64::to_string).collect();
        let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
        let d = one_lot(&w, lot_qty, lines(&w, &refs));
        let started = Instant::now();
        let p = preview(&w, &d);
        let took = started.elapsed();
        assert!(
            took.as_millis() < 2_000,
            "{name}: the preview of 50 lines took {took:?}"
        );
        let want = model(lot_qty, &qtys);
        let refused = want.iter().filter(|a| a.is_some()).count();
        assert_eq!(crossings(&p.errors).len(), refused, "{name}: refused lines");
        let first_refused = want.iter().position(Option::is_some);
        if let Some(at) = first_refused {
            // The line that pushes the lot over is the first refused, in line order, and every later line is too
            // (a bigger quantity than what is left never fits; the ones that are 1 may, when each is 1).
            assert!(p.lots[0].lines[at].calc.is_none(), "{name}: line {at}");
            assert!(
                at == 0 || p.lots[0].lines[at - 1].calc.is_some(),
                "{name}: line {at} is not the first to cross"
            );
        }
        let sold = qtys
            .iter()
            .zip(&want)
            .filter(|(_, a)| a.is_none())
            .map(|(q, _)| *q)
            .sum::<i64>();
        assert_eq!(p.lots[0].sold_qty, sold, "{name}: sold");
    }
}

// ---- 3 ------------------------------------------------------------------------------------------------

#[test]
fn a_save_with_a_refused_line_writes_nothing_at_all() {
    let w = world_with(50_000, true);
    // One earlier بکری, so the books are not empty and a half-save would show.
    let first = try_save(&w, &event1(&w)).expect("the worked بکری saves");
    assert_chain_ok(&w.dir);
    let before = snapshot(&w);
    let mut bad: Vec<(&str, BikriDraft)> = vec![
        ("one line over", one_lot(&w, 50, lines(&w, &["70"]))),
        ("30 and 30 on 50", one_lot(&w, 50, lines(&w, &["30", "30"]))),
        (
            "a good line then a crossing one",
            one_lot(&w, 50, lines(&w, &["20", "20", "11"])),
        ),
        ("51 at once", one_lot(&w, 50, lines(&w, &["51"]))),
        (
            "a crossing line then a good one",
            one_lot(&w, 50, lines(&w, &["60", "10"])),
        ),
    ];
    // Two lots, the second over.
    let mut two = event1(&w);
    two.lots.push(with_sales(
        lot(&w, "lot-2", w.karim, w.apple, "10"),
        lines(&w, &["11"]),
    ));
    bad.push(("the second lot is over", two));
    // A cash line over (the over-quantity rule must hold for cash lines too).
    let mut cash = one_lot(&w, 50, lines(&w, &["30", "30"]));
    cash.lots[0].sales[1].buyer_account_id = Some(w.naqdi);
    bad.push(("a cash line over", cash));
    for (name, d) in &bad {
        let error = try_save(&w, d).expect_err(name);
        assert!(
            !refused_fields(&error).is_empty(),
            "{name}: the save was not refused for the quantity: {error:?}"
        );
        assert_eq!(
            snapshot(&w),
            before,
            "{name}: the refused save changed the books"
        );
    }
    assert_chain_ok(&w.dir);
    // The first بکری is untouched.
    let kept = bikri::get(&w.env.ctx, first.bikri_id).expect("the first بکری");
    assert_eq!(kept.number, first.number);
    // The over quantity fixed, the same draft saves.
    let good = one_lot(&w, 50, lines(&w, &["30", "20"]));
    try_save(&w, &good).expect("the fixed draft saves");
    assert_chain_ok(&w.dir);
}

// ---- 4 ------------------------------------------------------------------------------------------------

#[test]
fn a_later_sale_over_the_stock_left_is_refused_and_writes_nothing() {
    let w = world_with(50_000, true);
    // 30 of 50 sold: 20 are left.
    let saved = try_save(&w, &one_lot(&w, 50, lines(&w, &["30"]))).expect("the first posting");
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let later = |qtys: &[&str]| LaterSaleInput {
        lot_id,
        date: "10-10-2026".into(),
        sales: lines(&w, qtys),
    };
    let before = snapshot(&w);
    // 25 is over: refused with 20 available, no figures, nothing written.
    let p = bikri::later_sale_preview(&w.env.ctx, &later(&["25"])).unwrap();
    assert_eq!(
        crossings(&p.errors),
        vec![("lots[0].sales[0].qty".to_owned(), 20)]
    );
    assert!(p.lots[0].lines[0].calc.is_none());
    assert_eq!(r(p.summary.gross), 0);
    assert_eq!(snapshot(&w), before, "the preview wrote something");
    // 12 and 12: the second crosses (8 left).
    let p = bikri::later_sale_preview(&w.env.ctx, &later(&["12", "12"])).unwrap();
    assert_eq!(
        crossings(&p.errors),
        vec![("lots[0].sales[1].qty".to_owned(), 8)]
    );
    assert!(p.lots[0].lines[0].calc.is_some());
    // The save with any refused line writes nothing.
    for qtys in [vec!["25"], vec!["12", "12"], vec!["10", "11"], vec!["21"]] {
        let error = bikri::later_sale_save(
            &w.env.ctx,
            &w.owner,
            &LaterSaleSaveInput {
                lot_id,
                date: "10-10-2026".into(),
                sales: lines(&w, &qtys),
                confirm_cash_below_zero: true,
            },
        )
        .expect_err("over the stock left");
        assert!(
            !refused_fields(&error).is_empty(),
            "{qtys:?}: not refused for the quantity: {error:?}"
        );
        assert_eq!(
            snapshot(&w),
            before,
            "{qtys:?}: the refused later sale changed the books"
        );
    }
    assert_chain_ok(&w.dir);
    // Exactly the stock left sells. Then nothing is left: the lot takes no later sale.
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: lines(&w, &["20"]),
            confirm_cash_below_zero: true,
        },
    )
    .expect("the stock left sells");
    let after = snapshot(&w);
    // With nothing left the lot takes no later sale at all: `ERR_NOTHING_TO_SELL` on `lot_id`, from the preview and
    // from the save, before any line is looked at.
    let nothing = |e: &ServiceError| {
        matches!(e, ServiceError::Rejected(items)
            if items.iter().any(|i| i.code == ErrorCode::ErrNothingToSell
                && i.field.as_deref() == Some("lot_id")))
    };
    let error = bikri::later_sale_preview(&w.env.ctx, &later(&["1"])).expect_err("nothing left");
    assert!(nothing(&error), "{error:?}");
    let error = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: lines(&w, &["1"]),
            confirm_cash_below_zero: true,
        },
    )
    .expect_err("nothing left");
    assert!(nothing(&error), "{error:?}");
    assert_eq!(snapshot(&w), after);
    assert_chain_ok(&w.dir);
}

// ---- 5 ------------------------------------------------------------------------------------------------

#[test]
fn an_edit_that_takes_a_line_over_the_lot_is_refused_and_the_old_posting_stays() {
    let w = world_with(50_000, true);
    let saved = try_save(&w, &one_lot(&w, 50, lines(&w, &["20", "30"]))).expect("a full lot");
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .expect("open for edit");
    assert!(opened.editable);
    let before = snapshot(&w);
    // Three edits that cross: line 1 grows to 21 (line 2 then crosses, 29 left); the lot shrinks to 40
    // (line 2 crosses, 20 left); line 2 grows to 31 (it crosses, 30 left).
    let mut grow = opened.draft.clone();
    grow.lots[0].sales[0].qty = "21".into();
    let mut shrink = opened.draft.clone();
    shrink.lots[0].qty_received = "40".into();
    let mut grow_two = opened.draft.clone();
    grow_two.lots[0].sales[1].qty = "31".into();
    for (name, d, field) in [
        ("line 1 grows", grow, "lots[0].sales[1].qty"),
        ("the lot shrinks", shrink, "lots[0].sales[1].qty"),
        ("line 2 grows", grow_two, "lots[0].sales[1].qty"),
    ] {
        let p = bikri::edit_preview(
            &w.env.ctx,
            &EditPreviewInput {
                entry_id: saved.entry_id,
                draft: d.clone(),
            },
        )
        .unwrap();
        let got = crossings(&p.errors);
        assert!(
            got.iter().any(|(f, _)| f == field),
            "{name}: the line that crosses is not refused: {got:?}"
        );
        let error = bikri::edit_save(
            &w.env.ctx,
            &w.owner,
            &EditSaveInput {
                entry_id: saved.entry_id,
                version: opened.version,
                draft: d,
                reason: Some("test".into()),
                confirm_cash_below_zero: true,
            },
        )
        .expect_err(name);
        assert!(
            !refused_fields(&error).is_empty(),
            "{name}: not refused for the quantity: {error:?}"
        );
        assert_eq!(
            snapshot(&w),
            before,
            "{name}: the refused edit changed the books"
        );
    }
    assert_chain_ok(&w.dir);
    // The old posting is still the one in force: its lines are intact and no reversal was written.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(
        detail.lots[0].sales.iter().filter(|s| s.is_active).count(),
        2,
        "the edit that was refused replaced a line"
    );
    // An edit that does not cross works, and it too stays inside the lot.
    let mut fine = opened.draft.clone();
    fine.lots[0].sales[0].qty = "19".into();
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: saved.entry_id,
            version: opened.version,
            draft: fine,
            reason: Some("test".into()),
            confirm_cash_below_zero: true,
        },
    )
    .expect("an edit inside the lot");
    assert_chain_ok(&w.dir);
}

// ---- 6 ------------------------------------------------------------------------------------------------

fn shop(partner: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id: partner,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

fn own_lot(w: &World, key: &str, purchase_id: i64, qty: &str) -> LotInput {
    LotInput {
        seller_account_id: Some(id_of(&w.env, "030119998")),
        purchase_id: Some(purchase_id),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..bare_lot(w, key, w.rahim, w.banana, qty)
    }
}

#[test]
fn an_own_goods_lot_over_its_purchase_is_refused_and_the_purchase_keeps_its_stock() {
    let w = world_with(50_000, true);
    let [a, b, c] = w.partners;
    let trip = trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: vec![shop(a, "40"), shop(b, "30"), shop(c, "30")],
        },
    )
    .expect("a trip")
    .id;
    let purchase = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            supplier: Some("Orange Traders".into()),
            item_id: Some(w.apple),
            packing_id: Some(w.box_),
            qty: "20".into(),
            weight: None,
            rate: "500".into(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::<EntryPartInput>::new(),
            confirm_cash_below_zero: false,
        },
    )
    .expect("a purchase of 20")
    .id;
    let before = snapshot(&w);
    let own = |received: &str, qtys: &[&str]| {
        draft(vec![with_sales(
            own_lot(&w, "own", purchase, received),
            lines(&w, qtys),
        )])
    };
    // Over the purchase: 25 received and sold, 12 and 12 on a lot of 20, a lot of 21.
    for (name, d) in [
        ("25 of 20", own("25", &["25"])),
        ("12 and 12 of 20", own("20", &["12", "12"])),
        ("21 of 20", own("21", &["21"])),
    ] {
        let p = preview(&w, &d);
        assert!(
            p.errors
                .iter()
                .any(|e| e.code == ErrorCode::ErrQtyExceedsLot),
            "{name}: the preview does not refuse the quantity: {:?}",
            p.errors
        );
        let error = try_save(&w, &d).expect_err(name);
        assert!(
            matches!(&error, ServiceError::Rejected(items)
                if items.iter().any(|i| i.code == ErrorCode::ErrQtyExceedsLot)),
            "{name}: not refused for the quantity: {error:?}"
        );
        assert_eq!(
            snapshot(&w),
            before,
            "{name}: the refused own lot changed the books"
        );
    }
    assert_chain_ok(&w.dir);
    // The whole purchase sells in one lot; then one more نگ of it is refused and the purchase stays at 0 left.
    try_save(&w, &own("20", &["12", "8"])).expect("the whole purchase sells");
    let after = snapshot(&w);
    let d = own("1", &["1"]);
    let error = try_save(&w, &d).expect_err("nothing of the purchase is left");
    assert!(
        matches!(&error, ServiceError::Rejected(items)
            if items.iter().any(|i| i.code == ErrorCode::ErrQtyExceedsLot)),
        "{error:?}"
    );
    assert_eq!(snapshot(&w), after, "the second own lot changed the books");
    assert_chain_ok(&w.dir);
}

// ---- 7 ------------------------------------------------------------------------------------------------

#[test]
fn a_typed_sum_is_worked_out_before_the_lot_is_checked() {
    // PRD FR-7.48: `20+40` is 60. On a lot of 50 it is over, and the error is the one of 60.
    let w = world_with(50_000, true);
    let amount = |p: &BikriPreview, line: usize| -> Option<i64> {
        p.lots[0].lines[line]
            .calc
            .as_ref()
            .map(|c| r(c.seller_amount))
    };
    let refused = |p: &BikriPreview| crossings(&p.errors);
    // 20+40: over, 50 available.
    let p = preview(&w, &one_lot(&w, 50, lines(&w, &["20+40"])));
    assert_eq!(refused(&p), vec![("lots[0].sales[0].qty".to_owned(), 50)]);
    assert_eq!(amount(&p, 0), None);
    // 20+30 is exactly 50: it fits, at 50 x 1,200.
    let p = preview(&w, &one_lot(&w, 50, lines(&w, &["20+30"])));
    assert!(refused(&p).is_empty(), "{:?}", p.errors);
    assert_eq!(amount(&p, 0), Some(60_000));
    // 20+31 is 51: over.
    let p = preview(&w, &one_lot(&w, 50, lines(&w, &["20+31"])));
    assert_eq!(refused(&p), vec![("lots[0].sales[0].qty".to_owned(), 50)]);
    // 60-20 is 40: it fits. The next line of 11 crosses (10 left).
    let p = preview(&w, &one_lot(&w, 50, lines(&w, &["60-20", "11"])));
    assert_eq!(amount(&p, 0), Some(48_000));
    assert_eq!(refused(&p), vec![("lots[0].sales[1].qty".to_owned(), 10)]);
    // A line after 20+20 is checked against what is left: 5+5 fits, 5+6 does not.
    let p = preview(&w, &one_lot(&w, 50, lines(&w, &["20+20", "5+5"])));
    assert!(refused(&p).is_empty(), "{:?}", p.errors);
    let p = preview(&w, &one_lot(&w, 50, lines(&w, &["20+20", "5+6"])));
    assert_eq!(refused(&p), vec![("lots[0].sales[1].qty".to_owned(), 10)]);
    // A sum below zero is not a quantity and is not the over-quantity error; no figures.
    let p = preview(&w, &one_lot(&w, 50, lines(&w, &["20-30"])));
    assert!(!p.errors.is_empty(), "a negative sum was accepted");
    assert!(
        refused(&p).is_empty(),
        "a negative sum is not an over quantity: {:?}",
        p.errors
    );
    assert_eq!(amount(&p, 0), None);
    // The save refuses the sums exactly as the preview does and writes nothing.
    let before = snapshot(&w);
    for qtys in [
        vec!["20+40"],
        vec!["20+31"],
        vec!["20-30"],
        vec!["20+20", "5+6"],
    ] {
        assert!(
            try_save(&w, &one_lot(&w, 50, lines(&w, &qtys))).is_err(),
            "{qtys:?} was saved"
        );
        assert_eq!(
            snapshot(&w),
            before,
            "{qtys:?}: the refused save changed the books"
        );
    }
    assert_chain_ok(&w.dir);
    // 20+30 saves, as 50 whole.
    let saved = try_save(&w, &one_lot(&w, 50, lines(&w, &["20+30"]))).expect("20+30 saves");
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(detail.lots[0].sales[0].qty, 50);
}
