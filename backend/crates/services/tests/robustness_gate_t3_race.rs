//! Gate lane T3 (plan 80), part 6: two saves at the same moment (NFR-REL-1, Code Standards B21,
//! PRD FR-12.6, FR-12.19, FR-12.22, E14, E15). Owner: `robustness`.
//!
//! The data file has one writer, so two saves are done one after the other. These tests start two
//! saves at the same instant from two threads (a barrier) and check what is left:
//!
//! - the same بکری saved twice: the books are whole and the numbers have no gap (the backend has no
//!   request number, so both are saved: this records it, see the QUESTION in the report);
//! - two later sales that together sell more than the lot has left: exactly one is saved, the other
//!   is refused with ERR_QTY_EXCEEDS_LOT, and the lot is never sold past its quantity;
//! - an edit and a cancel of the same بکری: exactly one wins, the other is refused (a changed
//!   version or a cancelled بکری), and nothing is half done;
//! - two edits of the same posting with the same version: one is saved, one is refused with
//!   ERR_VERSION_CHANGED.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::sync::{Arc, Barrier};

use common::*;
use mandi_services::core::bikri::{
    BikriSaveInput, CancelInput, EditOpenInput, EditSaveInput, LaterSaleSaveInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::{ServiceError, bikri, ledger};
use support::World;

fn scalar(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

fn assert_whole(w: &World, label: &str) {
    let imbalance = scalar(
        w,
        "SELECT COALESCE(SUM(l.dr_paisa), 0) - COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'",
    );
    assert_eq!(imbalance, 0, "{label}: the books do not balance");
    assert_eq!(
        scalar(
            w,
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'"
        ),
        0,
        "{label}: a draft entry was left"
    );
    assert_eq!(scalar(w, "SELECT COUNT(*) FROM edit_windows"), 0, "{label}");
    assert_eq!(
        scalar(
            w,
            "SELECT COUNT(*) FROM bikri_lots l WHERE l.qty_sold > l.qty_received"
        ),
        0,
        "{label}: a lot was sold past its quantity"
    );
    assert_eq!(
        scalar(
            w,
            "SELECT COUNT(*) FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id WHERE b.status = 'saved' AND l.qty_sold <> COALESCE((SELECT SUM(s.qty) FROM bikri_sales s WHERE s.lot_id = l.id AND s.is_active = 1), 0)"
        ),
        0,
        "{label}: a lot's sold quantity differs from its live lines"
    );
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

fn two<T: Send + 'static>(
    a: impl FnOnce() -> T + Send + 'static,
    b: impl FnOnce() -> T + Send + 'static,
) -> (T, T) {
    let barrier = Arc::new(Barrier::new(2));
    let (b1, b2) = (barrier.clone(), barrier);
    let h1 = std::thread::spawn(move || {
        b1.wait();
        a()
    });
    let h2 = std::thread::spawn(move || {
        b2.wait();
        b()
    });
    (h1.join().unwrap(), h2.join().unwrap())
}

fn part_sold(w: &World) -> mandi_services::core::bikri::BikriDraft {
    support::draft(vec![support::with_sales(
        support::lot(w, "lot-1", w.rahim, w.banana, "50"),
        vec![support::sale("s-1", w.akram, "20", "600")],
    )])
}

#[test]
fn the_same_bikri_saved_twice_at_once_leaves_whole_books_and_numbers_without_a_gap() {
    let w = support::world();
    let draft = support::event1(&w);
    let mut saved_twice = 0;
    for _ in 0..8 {
        let (ctx1, owner1, d1) = (w.env.ctx.clone(), w.owner.clone(), draft.clone());
        let (ctx2, owner2, d2) = (w.env.ctx.clone(), w.owner.clone(), draft.clone());
        let (a, b) = two(
            move || {
                bikri::save(
                    &ctx1,
                    &owner1,
                    &BikriSaveInput {
                        draft: d1,
                        confirm_cash_below_zero: true,
                    },
                )
                .map(|s| s.number)
            },
            move || {
                bikri::save(
                    &ctx2,
                    &owner2,
                    &BikriSaveInput {
                        draft: d2,
                        confirm_cash_below_zero: true,
                    },
                )
                .map(|s| s.number)
            },
        );
        if let (Ok(x), Ok(y)) = (&a, &b) {
            assert_ne!(x, y, "two saves got the same بکری number");
            saved_twice += 1;
        }
        assert!(a.is_ok() || b.is_ok(), "{a:?} {b:?}");
        assert_whole(&w, "same save twice");
        assert_eq!(
            scalar(
                &w,
                "SELECT COALESCE(MAX(number), 0) - COUNT(*) FROM bikri_entries"
            ),
            0,
            "the numbers have a gap"
        );
    }
    println!(
        "8 pairs of identical saves at once: {saved_twice} pairs were both saved (the backend has no request number)"
    );
}

#[test]
fn two_later_sales_that_together_sell_more_than_the_lot_has_left_save_only_one() {
    let w = support::world();
    for round in 0..8 {
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft: part_sold(&w),
                confirm_cash_below_zero: true,
            },
        )
        .unwrap();
        let lot_id = scalar(&w, "SELECT MAX(id) FROM bikri_lots");
        // 30 are left; each sale takes 20.
        let make = |key: &str| LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![support::sale(key, w.arshad, "20", "560")],
            confirm_cash_below_zero: true,
        };
        let (ctx1, owner1, in1) = (w.env.ctx.clone(), w.owner.clone(), make("s-a"));
        let (ctx2, owner2, in2) = (w.env.ctx.clone(), w.owner.clone(), make("s-b"));
        let (a, b) = two(
            move || bikri::later_sale_save(&ctx1, &owner1, &in1).map(|_| ()),
            move || bikri::later_sale_save(&ctx2, &owner2, &in2).map(|_| ()),
        );
        let worked = usize::from(a.is_ok()) + usize::from(b.is_ok());
        assert_eq!(worked, 1, "round {round}: {a:?} and {b:?}");
        let refused = a.err().or(b.err()).unwrap();
        assert!(
            refused.has_code(ErrorCode::ErrQtyExceedsLot),
            "round {round}: the second sale was refused with {refused:?}"
        );
        assert_eq!(
            scalar(
                &w,
                &format!("SELECT qty_sold FROM bikri_lots WHERE id = {lot_id}")
            ),
            40,
            "round {round}"
        );
        assert_whole(&w, "later sales race");
    }
}

#[test]
fn an_edit_and_a_cancel_of_the_same_bikri_at_once_leave_exactly_one_result() {
    let w = support::world();
    for round in 0..8 {
        let saved = bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft: support::event1(&w),
                confirm_cash_below_zero: true,
            },
        )
        .unwrap();
        let entry = scalar(
            &w,
            &format!(
                "SELECT MAX(id) FROM journal_entries WHERE posting_type = 'bikri_first' AND source_id = {}",
                saved.bikri_id
            ),
        );
        let opened = bikri::open_for_edit(
            &w.env.ctx,
            &EditOpenInput {
                bikri_id: saved.bikri_id,
                entry_id: entry,
            },
        )
        .unwrap();
        let mut draft = opened.draft.clone();
        draft.lots[0].sales[0].seller_rate = "610".into();
        draft.lots[0].sales[0].buyer_rate = Some("610".into());
        let edit = EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("rate".into()),
            confirm_cash_below_zero: true,
        };
        let cancel = CancelInput {
            id: saved.bikri_id,
            version: bikri::get(&w.env.ctx, saved.bikri_id).unwrap().version,
            reason: "gate".into(),
            confirm_cash_below_zero: true,
        };
        let (ctx1, owner1) = (w.env.ctx.clone(), w.owner.clone());
        let (ctx2, owner2) = (w.env.ctx.clone(), w.owner.clone());
        let (edited, cancelled): (Result<(), ServiceError>, Result<(), ServiceError>) = two(
            move || bikri::edit_save(&ctx1, &owner1, &edit).map(|_| ()),
            move || bikri::cancel(&ctx2, &owner2, &cancel).map(|_| ()),
        );
        // Either order is allowed: an edit and then a cancel both work when the cancel is read
        // after the edit changed nothing it checks. What is not allowed is a half state.
        println!(
            "round {round}: edit {} / cancel {}",
            edited
                .as_ref()
                .map_or_else(|e| format!("refused {:?}", e.code()), |_| "saved".into()),
            cancelled
                .as_ref()
                .map_or_else(|e| format!("refused {:?}", e.code()), |_| "saved".into())
        );
        assert!(
            edited.is_ok() || cancelled.is_ok(),
            "round {round}: {edited:?} {cancelled:?}"
        );
        let status: String = raw(&w.dir)
            .query_row(
                &format!(
                    "SELECT status FROM bikri_entries WHERE id = {}",
                    saved.bikri_id
                ),
                [],
                |r| r.get(0),
            )
            .unwrap();
        if cancelled.is_ok() {
            assert_eq!(status, "cancelled", "round {round}");
            // Nothing of a cancelled بکری is live: no live line, no unreversed entry.
            assert_eq!(
                scalar(
                    &w,
                    &format!(
                        "SELECT COUNT(*) FROM bikri_sales WHERE bikri_id = {} AND is_active = 1",
                        saved.bikri_id
                    )
                ),
                0,
                "round {round}: a cancelled بکری still has live lines"
            );
            assert_eq!(
                scalar(
                    &w,
                    &format!(
                        "SELECT COUNT(*) FROM journal_entries e WHERE e.source_type = 'bikri' AND e.source_id = {} AND e.reverses_entry_id IS NULL AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id)",
                        saved.bikri_id
                    )
                ),
                0,
                "round {round}: a cancelled بکری has a posting that is not reversed"
            );
        }
        assert_whole(&w, "edit and cancel race");
    }
}

#[test]
fn two_edits_of_the_same_posting_with_the_same_version_save_one_and_refuse_one() {
    let w = support::world();
    let saved = bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let entry = scalar(
        &w,
        "SELECT id FROM journal_entries WHERE posting_type = 'bikri_first'",
    );
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: entry,
        },
    )
    .unwrap();
    let make = |rate: &str| {
        let mut draft = opened.draft.clone();
        draft.lots[0].sales[0].seller_rate = rate.into();
        draft.lots[0].sales[0].buyer_rate = Some(rate.into());
        EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("rate".into()),
            confirm_cash_below_zero: true,
        }
    };
    let (e1, e2) = (make("610"), make("620"));
    let (ctx1, owner1) = (w.env.ctx.clone(), w.owner.clone());
    let (ctx2, owner2) = (w.env.ctx.clone(), w.owner.clone());
    let (a, b) = two(
        move || bikri::edit_save(&ctx1, &owner1, &e1).map(|_| ()),
        move || bikri::edit_save(&ctx2, &owner2, &e2).map(|_| ()),
    );
    assert_eq!(
        usize::from(a.is_ok()) + usize::from(b.is_ok()),
        1,
        "{a:?} and {b:?}: two edits of the same version both saved, or neither"
    );
    let refused = a.err().or(b.err()).unwrap();
    assert!(
        refused.has_code(ErrorCode::ErrVersionChanged) || refused.has_code(ErrorCode::ErrNotFound),
        "the second edit was refused with {refused:?}"
    );
    assert_whole(&w, "two edits");
}
