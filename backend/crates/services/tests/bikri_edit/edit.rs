//! What an edit writes and what it refuses (PRD FR-12.18 to FR-12.21, FR-6.7, plan 16). The golden
//! numbers are in `golden_day.rs`; here are the rules around them.

use mandi_services::bikri;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::{DayCloseInput, DayReopenInput};
use mandi_services::day;

use crate::common::{assert_chain_ok, audit_count, raw};
use crate::helpers::*;
use crate::support::*;
use crate::tools::{self, refused};

#[test]
fn an_edit_never_changes_a_posted_row() {
    let w = world();
    give_advance(&w, w.rahim, "15,000");
    let saved = save_ok(&w, &event1(&w));
    let before = tools::history(&w);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "610".into();
    tools::edit(&w, &opened, &d);
    let after = tools::history(&w);
    for ((table, old), (_, new)) in before.iter().zip(&after) {
        assert!(new.len() > old.len(), "{table} got no new row");
        assert_eq!(&new[..old.len()], &old[..], "{table}: a posted row changed");
    }
    assert_eq!(tools::trial_difference(&w), 0);
    assert_chain_ok(&w.dir);
}

#[test]
fn a_closed_day_cannot_be_edited_or_cancelled_until_the_owner_reopens_it() {
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    let entries = scalar(&w, "SELECT COUNT(*) FROM journal_entries");
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    assert!(!opened.editable);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "610".into();
    let p = tools::preview(&w, &opened, &d);
    assert!(
        tools::flagged(&p.errors, ErrorCode::ErrDateClosed, "header.entry_date"),
        "{:?}",
        p.errors
    );
    let refused_edit = tools::try_edit(&w, &opened, &d, true).unwrap_err();
    assert!(refused(
        &refused_edit,
        ErrorCode::ErrDateClosed,
        "header.entry_date"
    ));
    // Cancel: the preview says so and the cancel is refused.
    let cp = bikri::cancel_preview(
        &w.env.ctx,
        &mandi_services::core::bikri::CancelPreviewInput { id: saved.bikri_id },
    )
    .unwrap();
    assert!(!cp.can_cancel);
    assert_eq!(cp.closed_dates, ["2026-10-10"]);
    let refused_cancel = tools::cancel(&w, saved.bikri_id, true).unwrap_err();
    assert_eq!(
        tools::refusals(&refused_cancel)[0].0,
        ErrorCode::ErrDateClosed
    );
    // Nothing was written by the refusals.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM journal_entries"), entries);
    // After the reopen the same edit works, and every step is in the audit log.
    day::reopen(
        &w.env.ctx,
        &w.owner,
        &DayReopenInput {
            date: "10-10-2026".into(),
            reason: "ریٹ کی درستی".into(),
        },
    )
    .unwrap();
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    assert!(opened.editable);
    tools::edit(&w, &opened, &d);
    assert_eq!(audit_count(&w.dir, "day_close"), 1);
    assert_eq!(audit_count(&w.dir, "day_reopen"), 1);
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'edit' AND entity = 'bikri'"
        ),
        1
    );
}

#[test]
fn a_stale_version_is_refused_and_nothing_is_written() {
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    let first = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = first.draft.clone();
    d.lots[0].sales[0].seller_rate = "610".into();
    tools::edit(&w, &first, &d);
    // The screen still holds the version it read before the edit above.
    let before = snapshot(&raw(&w.dir));
    let error = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::EditSaveInput {
            entry_id: first.entry_id,
            version: first.version,
            draft: d,
            reason: None,
            confirm_cash_below_zero: true,
        },
    );
    // The posting it points at has been replaced, so it is not found any more.
    let error = error.unwrap_err();
    assert!(
        refused(&error, ErrorCode::ErrNotFound, "entry_id")
            || refused(&error, ErrorCode::ErrVersionChanged, "version"),
        "{error:?}"
    );
    assert_eq!(snapshot(&raw(&w.dir)), before);
}

#[test]
fn the_version_of_the_bikri_is_checked_for_the_posting_in_force() {
    // A later sale changes the version; an edit of the first posting opened before it is stale.
    let w = world();
    let mut d = event1(&w);
    d.lots[0].sales.pop();
    let saved = save_ok(&w, &d);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let lot_id = opened.draft.lots[0].lot_id.unwrap();
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![sale("l-1", w.arshad, "30", "580")],
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let mut changed = opened.draft.clone();
    changed.lots[0].sales[0].seller_rate = "610".into();
    let error = tools::try_edit(&w, &opened, &changed, true).unwrap_err();
    assert!(
        refused(&error, ErrorCode::ErrVersionChanged, "version"),
        "{error:?}"
    );
}

#[test]
fn lots_and_lines_that_are_not_the_postings_are_refused_with_their_field() {
    let w = world();
    let mut two_lots = event1(&w);
    let mut second = lot(&w, "lot-2", w.karim, w.apple, "10");
    second.sales = vec![sale("s-3", w.akram, "10", "1200")];
    two_lots.lots.push(second);
    let saved = save_ok(&w, &two_lots);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let try_with = |change: &dyn Fn(&mut mandi_services::core::bikri::BikriDraft)| {
        let mut d = opened.draft.clone();
        change(&mut d);
        let p = tools::preview(&w, &opened, &d);
        let error = tools::try_edit(&w, &opened, &d, true).unwrap_err();
        // The preview and the save say the same.
        for (code, field) in tools::refusals(&error) {
            assert!(
                tools::flagged(&p.errors, code, &field),
                "{code:?} at {field}: {:?}",
                p.errors
            );
        }
        error
    };
    // A lot cannot be removed.
    let e = try_with(&|d| {
        d.lots.pop();
    });
    assert!(refused(&e, ErrorCode::ErrEditNotAllowed, "lots"));
    assert_eq!(tools::rule_at(&e, "lots").as_deref(), Some("lot_removed"));
    // Lots cannot be put in another order.
    let e = try_with(&|d| d.lots.reverse());
    assert!(refused(&e, ErrorCode::ErrEditNotAllowed, "lots[1].lot_id"));
    assert_eq!(
        tools::rule_at(&e, "lots[1].lot_id").as_deref(),
        Some("lot_order_changed")
    );
    // A lot that is not this بکری's.
    let e = try_with(&|d| d.lots[0].lot_id = Some(9_999));
    assert!(refused(&e, ErrorCode::ErrEditNotAllowed, "lots[0].lot_id"));
    assert_eq!(
        tools::rule_at(&e, "lots[0].lot_id").as_deref(),
        Some("lot_not_in_posting")
    );
    // A line that is another lot's.
    let e = try_with(&|d| {
        let foreign = d.lots[1].sales[0].sale_id;
        d.lots[0].sales[0].sale_id = foreign;
    });
    assert!(refused(
        &e,
        ErrorCode::ErrEditNotAllowed,
        "lots[0].sales[0].sale_id"
    ));
    assert_eq!(
        tools::rule_at(&e, "lots[0].sales[0].sale_id").as_deref(),
        Some("line_not_in_posting")
    );
    // The same line twice.
    let e = try_with(&|d| {
        let first = d.lots[0].sales[0].sale_id;
        d.lots[0].sales[1].sale_id = first;
    });
    assert!(refused(
        &e,
        ErrorCode::ErrEditNotAllowed,
        "lots[0].sales[1].sale_id"
    ));
    // The day of the posting cannot change.
    let e = try_with(&|d| d.header.entry_date = "09-10-2026".into());
    assert!(refused(
        &e,
        ErrorCode::ErrEditNotAllowed,
        "header.entry_date"
    ));
    assert_eq!(
        tools::rule_at(&e, "header.entry_date").as_deref(),
        Some("day_changed")
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM journal_entries WHERE posting_type <> 'opening'"
        ),
        1
    );
}

#[test]
fn an_edit_cannot_take_the_quantity_received_below_what_other_postings_took() {
    let w = world();
    let mut d = event1(&w);
    d.lots[0].sales.pop();
    let saved = save_ok(&w, &d);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let lot_id = opened.draft.lots[0].lot_id.unwrap();
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![sale("l-1", w.arshad, "30", "580")],
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // The first posting's lot is now sold 20 + 30. Receiving only 40 is below the 30 taken later.
    let reopened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut changed = reopened.draft.clone();
    changed.lots[0].qty_received = "20".into();
    let p = tools::preview(&w, &reopened, &changed);
    assert!(
        tools::flagged(
            &p.errors,
            ErrorCode::ErrQtyBelowSold,
            "lots[0].qty_received"
        ),
        "{:?}",
        p.errors
    );
    assert_eq!(
        tools::number(&p.errors, ErrorCode::ErrQtyBelowSold, "sold"),
        Some(30)
    );
    let error = tools::try_edit(&w, &reopened, &changed, true).unwrap_err();
    assert!(refused(
        &error,
        ErrorCode::ErrQtyBelowSold,
        "lots[0].qty_received"
    ));
}

#[test]
fn an_edit_that_takes_cash_below_zero_asks_and_one_that_gives_cash_back_does_not() {
    // PRD FR-6.7: cash 20,000, Event 1 gives 5,000 نقد. The edit makes the نقد 25,000.
    let w = world_with(20_000, true);
    let saved = save_ok(&w, &event1(&w));
    assert_eq!(tools::cash(&w), 15_000);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut more = opened.draft.clone();
    more.lots[0].cash_paid = Some("25,000".into());
    let p = tools::preview(&w, &opened, &more);
    assert_eq!(p.cash.cash_after_paisa.rupees(), -5_000);
    assert!(p.cash.needs_confirm);
    // The question is about this entry's change against the old posting: −20,000 more, not −25,000.
    assert_eq!(p.cash.change_paisa.rupees(), -20_000);
    let asked = tools::try_edit(&w, &opened, &more, false).unwrap_err();
    assert!(refused(&asked, ErrorCode::ErrCashBelowZeroNotConfirmed, ""));
    assert_eq!(
        tools::refusal_number(
            &asked,
            ErrorCode::ErrCashBelowZeroNotConfirmed,
            "cash_after_paisa"
        ),
        Some(-500_000)
    );
    assert_eq!(tools::cash(&w), 15_000);
    let done = tools::try_edit(&w, &opened, &more, true).unwrap();
    assert_eq!(tools::cash(&w), -5_000);
    // Back to 5,000: the edit gives 20,000 back, so it never asks.
    let reopened = tools::open(&w, saved.bikri_id, done.entry_id);
    let mut back = reopened.draft.clone();
    back.lots[0].cash_paid = Some("5,000".into());
    let p = tools::preview(&w, &reopened, &back);
    assert!(!p.cash.needs_confirm && !p.cash.below_zero);
    tools::try_edit(&w, &reopened, &back, false).unwrap();
    assert_eq!(tools::cash(&w), 15_000);
}

#[test]
fn the_credit_limit_of_an_edit_leaves_out_the_posting_it_replaces() {
    // His limit is 13,000. The line is 12,880 now; at 610 it is 13,092, so the edit warns. If the
    // old 12,880 were counted too, the warning would say 25,972.
    let w = world();
    let naeem = add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "نعیم",
        "Naeem",
        true,
        "13,000",
    );
    let d = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![sale("s-1", naeem, "20", "600")],
    )]);
    let saved = save_ok(&w, &d);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let unchanged = tools::preview(&w, &opened, &opened.draft);
    assert!(
        unchanged
            .warnings
            .iter()
            .all(|e| e.code != ErrorCode::ErrCreditLimitExceeded),
        "{:?}",
        unchanged.warnings
    );
    let mut changed = opened.draft.clone();
    changed.lots[0].sales[0].seller_rate = "610".into();
    let p = tools::preview(&w, &opened, &changed);
    assert_eq!(
        tools::number(
            &p.warnings,
            ErrorCode::ErrCreditLimitExceeded,
            "would_owe_paisa"
        ),
        Some(1_309_200)
    );
}

#[test]
fn the_rate_hint_of_an_edit_does_not_count_the_posting_it_replaces() {
    // Event 1 sold at 600 and 580. The edit sells the first line at 610 and adds a blank row: the
    // hint is the 610 of this draft, never a rate of the saved posting that the edit replaces.
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales.truncate(1);
    d.lots[0].sales[0].seller_rate = "610".into();
    d.lots[0].sales[0].qty = "10".into();
    d.lots[0].sales.push(blank_sale("s-new", "5"));
    let p = tools::preview(&w, &opened, &d);
    let hint = p.lots[0].lines[1].rate_hint.as_ref().expect("a hint");
    assert_eq!(hint.rate_paisa.rupees(), 610);
}

#[test]
fn changing_the_lots_seller_or_item_looks_the_rules_up_again() {
    // Money agent review of plan 16: nothing stored for the old seller or item stays on the lot.
    // رحیم has his own 5% on کیلا. Event 1 is saved with him: 600 + 870 = 1,470.
    use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput};
    let w = world();
    mandi_services::rules::save(
        &w.env.ctx,
        &w.owner,
        &RuleInput {
            id: None,
            version: None,
            side: ChargeSide::Seller,
            charge_type: ChargeType::Commission,
            account_id: Some(w.rahim),
            item_id: Some(w.banana),
            item_group: None,
            method: ChargeMethod::Percent,
            value: "5".into(),
            from_date: "01-01-2026".into(),
            to_date: String::new(),
            is_active: true,
            note: String::new(),
        },
    )
    .unwrap();
    let saved = save_ok(&w, &event1(&w));
    let comm = |w: &World| {
        bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0]
            .seller_comm
            .rupees()
    };
    assert_eq!(comm(&w), 1_470);
    // The munshi corrects the seller to کریم and touches no line: کریم has the 6% default, so
    // the commissions are 720 + 1,044.
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].seller_account_id = Some(w.karim);
    let edited = tools::edit(&w, &opened, &d);
    assert_eq!(comm(&w), 1_764);
    assert_eq!(edited.preview.summary.shared_income.rupees(), 3_948);
    assert_eq!(
        balance(&w, w.karim),
        (
            mandi_services::core::accounts::BalanceSide::Cr,
            24_316 - 5_000 + 5_000 - 5_000
        )
    );
    // Then the item becomes مولی, a vegetable: the fruit-only لاگا of 8 per نگ goes.
    let again = tools::open(&w, saved.bikri_id, edited.entry_id);
    let mut d = again.draft.clone();
    d.lots[0].item_id = Some(w.mooli);
    let done = tools::edit(&w, &again, &d);
    assert_eq!(done.preview.summary.laga.rupees(), 0);
    assert_eq!(
        done.preview.summary.shared_income.rupees(),
        1_764 + 1_764 + 20
    );
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn the_seller_and_item_of_a_lot_with_a_later_sale_are_locked_and_a_later_posting_keeps_its_lot() {
    let w = world();
    let mut d = event1(&w);
    d.lots[0].sales.pop();
    let saved = save_ok(&w, &d);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let lot_id = opened.draft.lots[0].lot_id.unwrap();
    let later = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![sale("l-1", w.arshad, "30", "580")],
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // First posting: the lot has a later sale, so its seller and item cannot change.
    let reopened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut changed = reopened.draft.clone();
    changed.lots[0].seller_account_id = Some(w.karim);
    changed.lots[0].item_id = Some(w.apple);
    let p = tools::preview(&w, &reopened, &changed);
    assert!(tools::flagged(
        &p.errors,
        ErrorCode::ErrEditNotAllowed,
        "lots[0].seller_account_id"
    ));
    assert!(tools::flagged(
        &p.errors,
        ErrorCode::ErrEditNotAllowed,
        "lots[0].item_id"
    ));
    let e = tools::try_edit(&w, &reopened, &changed, true).unwrap_err();
    assert_eq!(
        tools::rule_at(&e, "lots[0].seller_account_id").as_deref(),
        Some("lot_seller_locked")
    );
    assert_eq!(
        tools::rule_at(&e, "lots[0].item_id").as_deref(),
        Some("lot_item_locked")
    );
    // Later posting: only its own lot, only its own lines, and the day of the posting.
    let later_open = tools::open(&w, saved.bikri_id, later.entry_id);
    let mut two = later_open.draft.clone();
    two.lots.push(two.lots[0].clone());
    let e = tools::try_edit(&w, &later_open, &two, true).unwrap_err();
    assert_eq!(
        tools::rule_at(&e, "lots").as_deref(),
        Some("posting_lots_locked")
    );
    let mut foreign = later_open.draft.clone();
    foreign.lots[0].sales[0].sale_id = Some(1);
    let e = tools::try_edit(&w, &later_open, &foreign, true).unwrap_err();
    assert_eq!(
        tools::rule_at(&e, "lots[0].sales[0].sale_id").as_deref(),
        Some("line_not_in_posting")
    );
    let mut other_day = later_open.draft.clone();
    other_day.header.entry_date = "09-10-2026".into();
    let e = tools::try_edit(&w, &later_open, &other_day, true).unwrap_err();
    assert_eq!(
        tools::rule_at(&e, "header.entry_date").as_deref(),
        Some("day_changed")
    );
    // The unchanged drafts still save.
    tools::edit(&w, &later_open, &later_open.draft);
}
