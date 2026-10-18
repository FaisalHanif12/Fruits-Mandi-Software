//! Cancel, undo and the later sale (PRD FR-12.17, FR-12.22, FR-7.47, FR-6.7, plan 16).

use mandi_services::bikri;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{
    BikriStatus, LaterSaleInput, LaterSaleSaveInput, StockListInput, UNDO_REASON_ID, UndoInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::day;

use crate::common::{assert_chain_ok, audit_count};
use crate::helpers::*;
use crate::support::*;
use crate::tools::{self, matched, open_advance_of, refused};

fn part_sold(w: &World) -> mandi_services::core::bikri::BikriDraft {
    let mut d = event1(w);
    d.lots[0].sales[1].qty = "25".into();
    d
}

/// Σ debit − Σ credit of an account over the whole journal, in paisa.
fn net(w: &World, account: i64) -> i64 {
    scalar(
        w,
        &format!(
            "SELECT COALESCE(SUM(dr_paisa) - SUM(cr_paisa), 0) FROM journal_lines WHERE account_id = {account}"
        ),
    )
}

#[test]
fn a_cancel_of_a_bikri_with_a_later_sale_takes_back_every_posting() {
    let w = world();
    give_advance(&w, w.rahim, "25,000");
    let saved = save_ok(&w, &part_sold(&w));
    tools::receipt(&w, w.akram, "10,000");
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let lot_id = opened.draft.lots[0].lot_id.unwrap();
    let later = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![sale("l-1", w.arshad, "5", "560")],
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let cash_before = tools::cash(&w);
    let preview = bikri::cancel_preview(
        &w.env.ctx,
        &mandi_services::core::bikri::CancelPreviewInput { id: saved.bikri_id },
    )
    .unwrap();
    assert!(preview.can_cancel);
    assert_eq!(preview.postings.len(), 2);
    // The postings took 5,000 out of cash (the نقد); a cancel gives it back.
    assert_eq!(preview.cash.change_paisa.rupees(), 5_000);
    assert!(!preview.cash.needs_confirm);
    let version = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().version;
    // A cancel needs a version that is current.
    let stale = bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: saved.bikri_id,
            version: version - 1,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert!(refused(&stale, ErrorCode::ErrVersionChanged, "version"));
    let done = tools::cancel(&w, saved.bikri_id, false).unwrap();
    assert_eq!(done.reversal_entry_ids.len(), 2);
    assert_eq!(tools::cash(&w), cash_before + 5_000);
    // Every account is back to what it was before the بکری, except the receipt that stays.
    assert_eq!(net(&w, w.arshad), 0);
    assert_eq!(net(&w, w.akram), -1_000_000);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Cr, 10_000));
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 25_000));
    for code in [
        "04010001", "04010002", "04010003", "04010004", "02010001", "02010002", "02010004",
    ] {
        assert_eq!(net(&w, id_of(&w.env, code)), 0, "{code}");
    }
    // Stock, facts and shares net to zero; the lines are cancelled, not deleted.
    assert_eq!(
        scalar(&w, "SELECT COALESCE(SUM(qty), 0) FROM stock_moves"),
        0
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(sign * qty), 0) FROM analytics_facts"
        ),
        0
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(sign * seller_amount_paisa), 0) FROM analytics_facts"
        ),
        0
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows"
        ),
        0
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM bikri_sales WHERE ended = 'cancelled'"
        ),
        3
    );
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM bikri_sales WHERE is_active = 1"),
        0
    );
    // Matching: the advance is open again, the bill rows are switched off, the receipt is credit.
    assert_eq!(open_advance_of(&w, w.rahim), 25_000);
    assert_eq!(matched(&w, "advance_allocations"), 0);
    assert_eq!(matched(&w, "bill_allocations"), 0);
    assert_eq!(tools::open_bills(&w, w.akram), (0, 10_000));
    // The بکری keeps its number and its lot columns; it is no longer in باقی مال.
    let after = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(
        (after.number, after.status),
        (saved.number, BikriStatus::Cancelled)
    );
    assert_eq!(after.lots[0].qty_sold, 50);
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert!(stock.rows.is_empty() && stock.balance_total == 0);
    // After a cancel nothing of it can be edited, sold again or written off.
    let again = tools::cancel(&w, saved.bikri_id, true).unwrap_err();
    assert!(refused(&again, ErrorCode::ErrBikriCancelled, "id"));
    let e = bikri::open_for_edit(
        &w.env.ctx,
        &mandi_services::core::bikri::EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: later.entry_id,
        },
    )
    .unwrap_err();
    assert!(refused(&e, ErrorCode::ErrBikriCancelled, "entry_id"));
    let e = bikri::later_sale_preview(
        &w.env.ctx,
        &LaterSaleInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![sale("x", w.arshad, "1", "500")],
        },
    )
    .unwrap_err();
    assert!(refused(&e, ErrorCode::ErrBikriCancelled, "lot_id"));
    assert_eq!(tools::trial_difference(&w), 0);
    assert_eq!(audit_count(&w.dir, "cancel"), 1);
    assert_chain_ok(&w.dir);
    mandi_services::ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

#[test]
fn a_cancel_that_takes_cash_out_below_zero_asks_and_one_that_gives_cash_back_does_not() {
    // A cash sale puts cash in; the cash is then lent to a seller as an advance; a cancel would
    // take the sale's cash out again, below zero, so it asks (PRD FR-6.7).
    let w = world_with(1_000, true);
    let mut d = draft(vec![with_sales(
        bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
        vec![sale("s-1", w.naqdi, "10", "1000")],
    )]);
    d.lots[0].cash_paid = None;
    let saved = save_ok(&w, &d);
    let in_cash = tools::cash(&w) - 1_000;
    assert!(in_cash > 0);
    give_advance(&w, w.rahim, &(in_cash + 1_000).to_string());
    assert_eq!(tools::cash(&w), 0);
    let preview = bikri::cancel_preview(
        &w.env.ctx,
        &mandi_services::core::bikri::CancelPreviewInput { id: saved.bikri_id },
    )
    .unwrap();
    assert!(preview.cash.needs_confirm);
    assert_eq!(preview.cash.cash_after_paisa.rupees(), -in_cash);
    let asked = tools::cancel(&w, saved.bikri_id, false).unwrap_err();
    assert!(refused(&asked, ErrorCode::ErrCashBelowZeroNotConfirmed, ""));
    assert_eq!(tools::cash(&w), 0);
    tools::cancel(&w, saved.bikri_id, true).unwrap();
    assert_eq!(tools::cash(&w), -in_cash);

    // A cancel that gives cash back never asks, even when cash is already below zero.
    let other = save_ok(&w, &event1(&w));
    let before = tools::cash(&w);
    tools::cancel(&w, other.bikri_id, false).unwrap();
    assert_eq!(tools::cash(&w), before + 5_000);
}

#[test]
fn undo_cancels_with_the_fixed_reason_gives_back_the_values_and_never_reuses_the_number() {
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    let undone = bikri::undo(
        &w.env.ctx,
        &w.owner,
        &UndoInput {
            id: saved.bikri_id,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(undone.number, saved.number);
    assert_eq!(undone.reversal_entry_ids.len(), 1);
    // The draft has the same values and no ids: it is a new unsaved entry.
    let d = &undone.draft;
    assert!(
        d.lots[0].lot_id.is_none()
            && d.lots
                .iter()
                .all(|l| l.sales.iter().all(|s| s.sale_id.is_none()))
    );
    assert_eq!(d.lots[0].sales[0].seller_rate, "600");
    assert_eq!(d.lots[0].sales[1].qty, "30");
    let after = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(after.status, BikriStatus::Cancelled);
    assert_eq!(after.cancel_reason.as_deref(), Some(UNDO_REASON_ID));
    // The corrected draft saves as the next number and the books balance.
    let again = save_ok(&w, d);
    assert_eq!(again.number, saved.number + 1);
    assert_eq!(tools::trial_difference(&w), 0);
    assert_eq!(audit_count(&w.dir, "cancel"), 1);
}

#[test]
fn undo_is_refused_when_the_day_is_closed() {
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    let e = bikri::undo(
        &w.env.ctx,
        &w.owner,
        &UndoInput {
            id: saved.bikri_id,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap_err();
    assert_eq!(tools::refusals(&e)[0].0, ErrorCode::ErrDateClosed);
    assert_eq!(
        bikri::get(&w.env.ctx, saved.bikri_id).unwrap().status,
        BikriStatus::Saved
    );
}

#[test]
fn a_later_sale_is_a_new_posting_and_leaves_the_first_one_alone() {
    let w = world();
    let saved = save_ok(&w, &part_sold(&w));
    let first_lines = entry_lines(&w, saved.entry_id);
    let before = tools::history(&w);
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    // The lot is listed with 5 left.
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(
        (stock.rows.len(), stock.rows[0].balance, stock.balance_total),
        (1, 5, 5)
    );
    // Preview, then save: the same numbers (Code Standards B5).
    let input = LaterSaleInput {
        lot_id,
        date: "10-10-2026".into(),
        sales: vec![sale("l-1", w.arshad, "5", "560")],
    };
    let preview = bikri::later_sale_preview(&w.env.ctx, &input).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    // Too many: only 5 are left.
    let too_many = bikri::later_sale_preview(
        &w.env.ctx,
        &LaterSaleInput {
            sales: vec![sale("l-1", w.arshad, "6", "560")],
            ..input.clone()
        },
    )
    .unwrap();
    assert!(tools::flagged(
        &too_many.errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[0].sales[0].qty"
    ));
    let saved_later = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: input.date.clone(),
            sales: input.sales.clone(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(saved_later.preview, preview);
    // 5 × 560 = 2,800; commissions 168 + 168; لاگا 40; اارشد pays 3,008; seller credit 2,632.
    let s = &preview.summary;
    assert_eq!(
        (
            s.gross.rupees(),
            s.seller_comm.rupees(),
            s.buyer_comm.rupees(),
            s.laga.rupees(),
            s.buyer_total.rupees(),
            s.seller_credit.rupees(),
            s.shared_income.rupees()
        ),
        (2_800, 168, 168, 40, 3_008, 2_632, 376)
    );
    // No lot charges again.
    assert_eq!(
        (
            s.freight.rupees(),
            s.labour.rupees(),
            s.munshiana.rupees(),
            s.phone.rupees(),
            s.cash_paid.rupees()
        ),
        (0, 0, 0, 0, 0)
    );
    // The first posting is not touched.
    assert_eq!(entry_lines(&w, saved.entry_id), first_lines);
    let after = tools::history(&w);
    for ((table, old), (_, new)) in before.iter().zip(&after) {
        assert_eq!(&new[..old.len()], &old[..], "{table}");
    }
    assert_eq!(tools::shares_of(&w, saved_later.entry_id), [152, 112, 112]);
    // The lot shows E2E-16b's totals and leaves باقی مال.
    let lot = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0];
    assert_eq!(
        (
            lot.gross.rupees(),
            lot.deductions.rupees(),
            lot.net.rupees()
        ),
        (29_300, 10_078, 19_222)
    );
    assert!(
        bikri::stock_list(&w.env.ctx, &StockListInput::default())
            .unwrap()
            .rows
            .is_empty()
    );
    // Nothing is left to sell.
    let e = bikri::later_sale_preview(&w.env.ctx, &input).unwrap_err();
    assert!(refused(&e, ErrorCode::ErrNothingToSell, "lot_id"));
}

#[test]
fn a_later_sale_before_the_goods_came_or_on_a_closed_day_is_refused() {
    let w = world();
    let saved = save_ok(&w, &part_sold(&w));
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let early = bikri::later_sale_preview(
        &w.env.ctx,
        &LaterSaleInput {
            lot_id,
            date: "09-10-2026".into(),
            sales: vec![sale("l", w.arshad, "5", "560")],
        },
    )
    .unwrap();
    assert!(
        tools::flagged(&early.errors, ErrorCode::ErrDateBeforeReceived, "date"),
        "{:?}",
        early.errors
    );
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    let closed = bikri::later_sale_preview(
        &w.env.ctx,
        &LaterSaleInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![sale("l", w.arshad, "5", "560")],
        },
    )
    .unwrap();
    assert!(
        tools::flagged(&closed.errors, ErrorCode::ErrDateClosed, "date"),
        "{:?}",
        closed.errors
    );
    // Nothing was written.
    let e = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![sale("l", w.arshad, "5", "560")],
            confirm_cash_below_zero: true,
        },
    )
    .unwrap_err();
    assert!(tools::refused(&e, ErrorCode::ErrDateClosed, "date"));
}

#[test]
fn a_later_sale_of_a_one_time_sellers_lot_pays_no_cash() {
    // PRD FR-12.17: the lot's نقد belongs to the first posting. A later sale never pays it again.
    let w = world();
    let mut first = lot(&w, "lot-1", w.pukhta_seller, w.banana, "10");
    first.pukhta = person("ناصر", "Nasir", "0301-1111111");
    first.cash_paid = Some("1,000".into());
    first.sales = vec![sale("s-1", w.akram, "5", "600")];
    let saved = save_ok(&w, &draft(vec![first]));
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let cash_before = tools::cash(&w);
    let later = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![sale("l", w.arshad, "5", "600")],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(tools::cash(&w), cash_before);
    let lines = entry_lines(&w, later.entry_id);
    assert!(
        lines.iter().all(|(code, _, _, _)| code != "01010001"),
        "{lines:?}"
    );
    // His person key is on the seller line.
    assert!(
        lines
            .iter()
            .any(|(code, _, _, key)| code == "030119999" && key.is_some())
    );
}
