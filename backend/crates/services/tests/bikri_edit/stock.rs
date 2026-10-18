//! The goods left to sell, write-off and its undo (PRD 12.8, FR-12.14 to FR-12.16, plan 16).

use mandi_services::bikri;
use mandi_services::core::bikri::{StockListInput, WriteOffInput, WriteOffUndoInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::day;

use crate::common::audit_count;
use crate::helpers::*;
use crate::support::*;
use crate::tools::{self, refused};

/// Event 1 with 45 of 50 sold.
fn part_sold(w: &World) -> mandi_services::core::bikri::BikriDraft {
    let mut d = event1(w);
    d.lots[0].sales[1].qty = "25".into();
    d
}

fn write_off(
    w: &World,
    lot_id: i64,
    qty: &str,
    reason: &str,
) -> Result<mandi_services::core::bikri::WriteOffDone, mandi_services::ServiceError> {
    bikri::write_off(
        &w.env.ctx,
        &w.owner,
        &WriteOffInput {
            lot_id,
            qty: qty.into(),
            reason: reason.into(),
        },
    )
}

#[test]
fn the_list_shows_lots_with_goods_left_and_filters_by_date_seller_and_item() {
    let w = world();
    let first = save_ok(&w, &part_sold(&w));
    // A second بکری of کریم's سیب with 10 left.
    let mut second = draft(vec![with_sales(
        lot(&w, "lot-1", w.karim, w.apple, "15"),
        vec![sale("s-1", w.akram, "5", "1200")],
    )]);
    second.lots[0].cash_paid = None;
    let second = save_ok(&w, &second);
    let all = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(all.rows.len(), 2);
    assert_eq!(all.balance_total, 15);
    let row = &all.rows[0];
    assert_eq!(
        (
            row.bikri_number,
            row.qty_received,
            row.qty_sold,
            row.qty_written_off,
            row.balance,
            row.days_waiting
        ),
        (first.number, 50, 45, 0, 5, 0)
    );
    assert_eq!(row.received_date, "2026-10-10");
    let by_seller = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            seller_id: Some(w.karim),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (by_seller.rows.len(), by_seller.rows[0].bikri_number),
        (1, second.number)
    );
    let by_item = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            item_id: Some(w.banana),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(by_item.rows.len(), 1);
    let by_date = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            date: Some("09-10-2026".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(by_date.rows.is_empty());
    // A bad date is a field error.
    let bad = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            date: Some("31-02-2026".into()),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(tools::refusals(&bad).iter().any(|(_, f)| f == "date"));
}

#[test]
fn a_cancelled_bikri_is_not_in_the_list() {
    let w = world();
    let saved = save_ok(&w, &part_sold(&w));
    tools::cancel(&w, saved.bikri_id, true).unwrap();
    assert!(
        bikri::stock_list(&w.env.ctx, &StockListInput::default())
            .unwrap()
            .rows
            .is_empty()
    );
}

#[test]
fn a_write_off_takes_goods_off_the_lot_and_its_undo_puts_them_back() {
    let w = world();
    let saved = save_ok(&w, &part_sold(&w));
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let journal_before = scalar(&w, "SELECT COUNT(*) FROM journal_entries");
    // A reason is required; a quantity above the balance is refused with the available quantity.
    let e = write_off(&w, lot_id, "2", "  ").unwrap_err();
    assert!(refused(&e, ErrorCode::ErrReasonRequired, "reason"));
    let e = write_off(&w, lot_id, "6", "خراب").unwrap_err();
    assert!(refused(&e, ErrorCode::ErrWriteOffExceedsBalance, "qty"));
    assert_eq!(
        tools::refusal_number(&e, ErrorCode::ErrWriteOffExceedsBalance, "available"),
        Some(5)
    );
    let e = write_off(&w, 9_999, "1", "خراب").unwrap_err();
    assert!(refused(&e, ErrorCode::ErrLotNotFound, "lot_id"));
    let done = write_off(&w, lot_id, "5", "خراب").unwrap();
    assert_eq!((done.qty, done.qty_written_off, done.balance), (5, 5, 0));
    // A stock move only: no journal entry for consigned goods (PRD FR-12.16).
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM journal_entries"),
        journal_before
    );
    assert_eq!(
        scalar(
            &w,
            &format!("SELECT qty FROM stock_moves WHERE id = {}", done.move_id)
        ),
        -5
    );
    // The lot is out of the list, but can be listed with its write-offs.
    assert!(
        bikri::stock_list(&w.env.ctx, &StockListInput::default())
            .unwrap()
            .rows
            .is_empty()
    );
    let with = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            include_written_off: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(with.rows[0].write_offs.len(), 1);
    assert_eq!(with.rows[0].write_offs[0].reason.as_deref(), Some("خراب"));
    // Nothing is left to sell on it.
    let e = bikri::later_sale_preview(
        &w.env.ctx,
        &mandi_services::core::bikri::LaterSaleInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![sale("l", w.arshad, "1", "500")],
        },
    )
    .unwrap_err();
    assert!(refused(&e, ErrorCode::ErrNothingToSell, "lot_id"));
    // Undo: the opposite move, the 5 are back.
    let undone = bikri::write_off_undo(
        &w.env.ctx,
        &w.owner,
        &WriteOffUndoInput {
            move_id: done.move_id,
            reason: "غلطی".into(),
        },
    )
    .unwrap();
    assert_eq!((undone.qty_written_off, undone.balance), (0, 5));
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE kind IN ('written_off', 'reversal') AND lot_id = 1"
        ),
        0
    );
    assert_eq!(
        bikri::stock_list(&w.env.ctx, &StockListInput::default())
            .unwrap()
            .rows
            .len(),
        1
    );
    // A write-off cannot be undone twice.
    let again = bikri::write_off_undo(
        &w.env.ctx,
        &w.owner,
        &WriteOffUndoInput {
            move_id: done.move_id,
            reason: "غلطی".into(),
        },
    )
    .unwrap_err();
    assert!(refused(&again, ErrorCode::ErrNotFound, "move_id"));
    assert_eq!(audit_count(&w.dir, "write_off"), 1);
    assert_eq!(audit_count(&w.dir, "write_off_undo"), 1);
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn a_write_off_then_a_cancel_leaves_all_the_stock_moves_at_zero() {
    let w = world();
    let saved = save_ok(&w, &part_sold(&w));
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    write_off(&w, lot_id, "3", "واپس").unwrap();
    tools::cancel(&w, saved.bikri_id, true).unwrap();
    // received +50, sold −45, written off −3 are all reversed.
    assert_eq!(
        scalar(&w, "SELECT COALESCE(SUM(qty), 0) FROM stock_moves"),
        0
    );
    // A write-off of a cancelled بکری, and the undo of its write-off, are refused.
    let e = write_off(&w, lot_id, "1", "خراب").unwrap_err();
    assert!(refused(&e, ErrorCode::ErrBikriCancelled, "lot_id"));
    let move_id = scalar(&w, "SELECT id FROM stock_moves WHERE kind = 'written_off'");
    let e = bikri::write_off_undo(
        &w.env.ctx,
        &w.owner,
        &WriteOffUndoInput {
            move_id,
            reason: "x".into(),
        },
    )
    .unwrap_err();
    // The cancel already took the write-off back, so there is nothing left to undo.
    assert!(refused(&e, ErrorCode::ErrNotFound, "move_id"));
}

#[test]
fn a_write_off_needs_an_open_day_and_its_undo_needs_the_day_of_the_write_off_open() {
    let w = world();
    let saved = save_ok(&w, &part_sold(&w));
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let done = write_off(&w, lot_id, "2", "خراب").unwrap();
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    // Today is 10-10-2026 and it is closed: a new write-off is refused.
    let e = write_off(&w, lot_id, "1", "خراب").unwrap_err();
    assert!(refused(&e, ErrorCode::ErrDateClosed, "date"));
    // The undo of the earlier one is dated the same closed day: refused too.
    let e = bikri::write_off_undo(
        &w.env.ctx,
        &w.owner,
        &WriteOffUndoInput {
            move_id: done.move_id,
            reason: "غلطی".into(),
        },
    )
    .unwrap_err();
    assert!(refused(&e, ErrorCode::ErrDateClosed, "move_id"));
}

#[test]
fn an_edit_of_the_first_posting_keeps_the_write_off_and_the_balance() {
    // PRD FR-12.20: a lot that other things rest on keeps them. Received 50, sold 45, written off 3:
    // receiving only 47 would leave 44 to sell, so the 45 sold in this posting are too many (the
    // write-off of 3 is taken as "elsewhere"). The same edit with 50 saves and keeps every column.
    let w = world();
    let saved = save_ok(&w, &part_sold(&w));
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    write_off(&w, lot_id, "3", "خراب").unwrap();
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    // The edited posting sells 45 itself; the write-off of 3 is "elsewhere".
    let mut d = opened.draft.clone();
    d.lots[0].qty_received = "47".into();
    let p = tools::preview(&w, &opened, &d);
    assert!(
        tools::flagged(
            &p.errors,
            ErrorCode::ErrQtyExceedsLot,
            "lots[0].sales[1].qty"
        ),
        "{:?}",
        p.errors
    );
    // Receiving fewer than the 3 written off is below what others took.
    d.lots[0].qty_received = "2".into();
    let p = tools::preview(&w, &opened, &d);
    assert!(
        tools::flagged(
            &p.errors,
            ErrorCode::ErrQtyBelowSold,
            "lots[0].qty_received"
        ),
        "{:?}",
        p.errors
    );
    d.lots[0].qty_received = "50".into();
    tools::edit(&w, &opened, &d);
    let lot = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0];
    assert_eq!(
        (lot.qty_received, lot.qty_sold, lot.qty_written_off),
        (50, 45, 3)
    );
}

#[test]
fn a_lot_with_a_write_off_keeps_its_seller_and_item_in_an_edit() {
    // Money agent review of plan 16, finding 4: a write-off rests on the lot, so the seller and
    // the item of the lot cannot change in an edit of its first posting.
    let w = world();
    let saved = save_ok(&w, &part_sold(&w));
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    write_off(&w, lot_id, "2", "خراب").unwrap();
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut changed = opened.draft.clone();
    changed.lots[0].seller_account_id = Some(w.karim);
    changed.lots[0].item_id = Some(w.apple);
    let e = tools::try_edit(&w, &opened, &changed, true).unwrap_err();
    assert_eq!(
        tools::rule_at(&e, "lots[0].seller_account_id").as_deref(),
        Some("lot_seller_locked")
    );
    assert_eq!(
        tools::rule_at(&e, "lots[0].item_id").as_deref(),
        Some("lot_item_locked")
    );
    // Other changes of the posting are still allowed.
    let mut rate = opened.draft.clone();
    rate.lots[0].sales[0].seller_rate = "610".into();
    tools::edit(&w, &opened, &rate);
    assert_eq!(tools::trial_difference(&w), 0);
}
