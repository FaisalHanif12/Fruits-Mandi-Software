//! Plan 92 T1 (PRD FR-15.3 v1.55, FR-7.71, FR-21.4): saving an edited وصولی, ادائیگی or پیشگی asks the
//! Owner's reason and sign on any day, today included. The sign check is the one of a day reopen
//! (`owner_sign::accepted`): "adnan" by default, case and spaces at the ends ignored, عدنان accepted.
//! A wrong sign saves nothing and writes one audit entry without the typed text. The right sign is
//! saved in the audit entry of the edit, beside the reason, and the edit stamp and history return it.
//! The other voucher types, and the review (preview), do not ask for it. A voucher of a closed day
//! edited in a reopen-and-fix does not ask for the sign twice.
//!
//! The data is the worked day (PRD 4.4): بکری 1 saved today, 10-10-2026, then payments to رحیم.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::{OWNER_PASSWORD, assert_chain_ok, raw, set_today};
use mandi_reports::audit::{EditHistoryInput, edit_history};
use mandi_services::core::edit::{DayFixCloseInput, DayFixOpenInput, EditEntity};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::{
    AdvanceInput, PaymentInput, ReceiptInput, RemovedPaymentFields, VoucherEditInput, VoucherEntry,
    VoucherSaved,
};
use mandi_services::secret::Secret;
use mandi_services::{ServiceError, bikri, day, edit, owner_sign, vouchers};
use support::World;

const REASON: &str = "رقم غلط تھی";

fn payment(w: &World, amount: &str, date: &str) -> PaymentInput {
    PaymentInput {
        date: date.into(),
        account_id: w.rahim,
        amount: amount.into(),
        removed: RemovedPaymentFields::default(),
        reason: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: false,
    }
}

fn receipt(w: &World, amount: &str) -> ReceiptInput {
    ReceiptInput {
        date: "10-10-2026".into(),
        account_id: w.akram,
        amount: amount.into(),
        removed: RemovedPaymentFields::default(),
        collector_staff_id: None,
        narration: String::new(),
    }
}

fn advance(w: &World, amount: &str) -> AdvanceInput {
    AdvanceInput {
        date: "10-10-2026".into(),
        seller_account_id: w.rahim,
        amount: amount.into(),
        removed: RemovedPaymentFields::default(),
        expected_goods: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: false,
        reason: None,
    }
}

/// The worked day with بکری 1 saved today (cash 15,000 after the نقد, اکرم owes 12,880).
fn day_with_event1() -> World {
    let w = support::world();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    w
}

/// An edit of voucher `id` to `entry`, with the reason and the sign typed.
fn edit_with(
    w: &World,
    id: i64,
    entry: VoucherEntry,
    reason: &str,
    sign: &str,
) -> Result<VoucherSaved, ServiceError> {
    let row = vouchers::get(&w.env.ctx, id).unwrap();
    vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id,
            version: row.version,
            entry,
            reason: reason.into(),
            sign: sign.into(),
            confirm_cash_below_zero: false,
        },
    )
}

/// `(code, field)` of each refusal of an error.
fn refusals(error: &ServiceError) -> Vec<(ErrorCode, Option<String>)> {
    match error {
        ServiceError::Rejected(items) => items.iter().map(|r| (r.code, r.field.clone())).collect(),
        other => panic!("not a refusal: {other:?}"),
    }
}

/// How many audit rows have this action.
fn count_of(w: &World, action: &str) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = ?1",
            [action],
            |r| r.get(0),
        )
        .unwrap()
}

/// The new value (JSON text) and the reason of every audit row of an action, oldest first.
fn rows_of(w: &World, action: &str) -> Vec<(Option<String>, Option<String>)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare("SELECT new_json, reason FROM audit_log WHERE action = ?1 ORDER BY id")
        .unwrap();
    statement
        .query_map([action], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// The new value and the reason of every audit row of an edit of a voucher, oldest first.
fn voucher_edits(w: &World) -> Vec<(Option<String>, Option<String>)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT new_json, reason FROM audit_log
             WHERE action = 'edit' AND entity = 'voucher' ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// The amount and the version a voucher has now (the edit changed neither if it was refused).
fn state(w: &World, id: i64) -> (i64, i64) {
    let row = vouchers::get(&w.env.ctx, id).unwrap();
    (row.amount_paisa.rupees(), row.version)
}

fn cash(w: &World) -> i64 {
    let balance =
        mandi_services::ledger::account_balance(&w.env.ctx, support::id_of(&w.env, "01010001"))
            .unwrap();
    balance.amount_paisa.rupees()
}

#[test]
fn an_edit_of_todays_payment_needs_the_reason_and_the_sign_and_a_wrong_sign_saves_nothing() {
    let w = day_with_event1();
    let saved =
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment(&w, "1,000", "10-10-2026")).unwrap();
    let id = saved.voucher_id;
    let before = (state(&w, id), cash(&w));
    let to = |amount: &str| VoucherEntry::Payment(payment(&w, amount, "10-10-2026"));

    // Neither: both refused together, each at its own field, with today's date too.
    let error = edit_with(&w, id, to("1,500"), "", "").unwrap_err();
    assert_eq!(
        refusals(&error),
        [
            (ErrorCode::ErrReasonRequired, Some("reason".into())),
            (ErrorCode::ErrSignRequired, Some("sign".into())),
        ]
    );
    // Only a reason, or only a sign.
    let error = edit_with(&w, id, to("1,500"), REASON, "  ").unwrap_err();
    assert_eq!(
        refusals(&error),
        [(ErrorCode::ErrSignRequired, Some("sign".into()))]
    );
    let error = edit_with(&w, id, to("1,500"), "  ", "adnan").unwrap_err();
    assert_eq!(
        refusals(&error),
        [(ErrorCode::ErrReasonRequired, Some("reason".into()))]
    );
    // Nothing was audited by those three: they are missing fields, not tries at the sign.
    assert_eq!(count_of(&w, "voucher_edit_sign_wrong"), 0);

    // A wrong sign: refused at `sign`, one audit entry without the typed text, nothing changed.
    let error = edit_with(&w, id, to("1,500"), REASON, "munshi").unwrap_err();
    assert_eq!(
        refusals(&error),
        [(ErrorCode::ErrSignWrong, Some("sign".into()))]
    );
    assert_eq!(count_of(&w, "voucher_edit_sign_wrong"), 1);
    let wrong = rows_of(&w, "voucher_edit_sign_wrong");
    let text = wrong[0].0.clone().unwrap_or_default();
    assert!(!text.to_lowercase().contains("munshi"), "{text}");
    assert_eq!((state(&w, id), cash(&w)), before);
    assert_eq!(voucher_edits(&w).len(), 0);

    // The right sign, in any case with spaces: the edit is saved and keeps its number.
    let edited = edit_with(&w, id, to("1,500"), REASON, " ADNAN ").unwrap();
    assert_eq!(edited.number, saved.number);
    assert_eq!(state(&w, id).0, 1_500);
    assert_eq!(cash(&w), before.1 - 500);
    // The audit entry has the reason and the Owner's own spelling of the sign.
    let edits = voucher_edits(&w);
    let last = edits.last().unwrap();
    assert_eq!(last.1.as_deref(), Some(REASON));
    assert!(
        last.0
            .as_deref()
            .is_some_and(|json| json.contains("\"sign\":\"adnan\"")),
        "{last:?}"
    );
    // The stamp on the row and the history return the sign, with the reason.
    let stamp = vouchers::get(&w.env.ctx, id).unwrap().flags.edited.unwrap();
    assert_eq!(stamp.sign.as_deref(), Some("adnan"));
    assert_eq!(stamp.reason.as_deref(), Some(REASON));
    let history = edit_history(
        w.env.ctx.db(),
        &EditHistoryInput {
            entity: EditEntity::Voucher,
            id,
        },
    )
    .unwrap();
    assert_eq!(history.versions.len(), 1);
    assert_eq!(history.versions[0].sign.as_deref(), Some("adnan"));
    // The Urdu spelling works too (the default sign).
    edit_with(&w, id, to("1,200"), REASON, "عدنان").unwrap();
    assert_eq!(state(&w, id).0, 1_200);
    assert_chain_ok(&w.dir);
}

#[test]
fn a_receipt_and_an_advance_ask_for_the_sign_too_and_the_other_types_do_not() {
    let w = day_with_event1();
    let received = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(&w, "5,000")).unwrap();
    let advanced = vouchers::save_advance(&w.env.ctx, &w.owner, &advance(&w, "2,000")).unwrap();
    for (id, entry) in [
        (
            received.voucher_id,
            VoucherEntry::Receipt(receipt(&w, "4,000")),
        ),
        (
            advanced.voucher_id,
            VoucherEntry::Advance(advance(&w, "2,500")),
        ),
    ] {
        let before = state(&w, id);
        let missing = edit_with(&w, id, entry.clone(), REASON, "").unwrap_err();
        assert_eq!(
            refusals(&missing),
            [(ErrorCode::ErrSignRequired, Some("sign".into()))]
        );
        let wrong = edit_with(&w, id, entry.clone(), REASON, "sana").unwrap_err();
        assert_eq!(
            refusals(&wrong),
            [(ErrorCode::ErrSignWrong, Some("sign".into()))]
        );
        assert_eq!(state(&w, id), before);
        edit_with(&w, id, entry, REASON, "adnan").unwrap();
        assert_ne!(state(&w, id), before);
    }
    assert_eq!(count_of(&w, "voucher_edit_sign_wrong"), 2);
    assert_chain_ok(&w.dir);
}

#[test]
fn a_date_that_changes_is_named_before_a_missing_reason_or_sign() {
    let w = day_with_event1();
    let saved =
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment(&w, "1,000", "10-10-2026")).unwrap();
    let error = edit_with(
        &w,
        saved.voucher_id,
        VoucherEntry::Payment(payment(&w, "1,000", "09-10-2026")),
        "",
        "",
    )
    .unwrap_err();
    assert_eq!(
        refusals(&error),
        [(ErrorCode::ErrEditDateFixed, Some("entry.date".into()))]
    );
    // A voucher that is not there is named first too.
    let error = edit_with(
        &w,
        saved.voucher_id,
        VoucherEntry::Payment(payment(&w, "1,000", "10-10-2026")),
        REASON,
        "adnan",
    );
    assert!(error.is_ok());
    let gone = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: 9_999,
            version: 1,
            entry: VoucherEntry::Payment(payment(&w, "1,000", "10-10-2026")),
            reason: String::new(),
            sign: String::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(
        refusals(&gone),
        [(ErrorCode::ErrNotFound, Some("id".into()))]
    );
}

#[test]
fn the_review_never_asks_for_the_sign() {
    let w = day_with_event1();
    let saved =
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment(&w, "1,000", "10-10-2026")).unwrap();
    let preview = edit::voucher_review(
        &w.env.ctx,
        &VoucherEditInput {
            id: saved.voucher_id,
            version: vouchers::get(&w.env.ctx, saved.voucher_id).unwrap().version,
            entry: VoucherEntry::Payment(payment(&w, "1,400", "10-10-2026")),
            reason: String::new(),
            sign: String::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(!preview.review.changes.is_empty());
    // The review saved nothing and audited nothing.
    assert_eq!(state(&w, saved.voucher_id).0, 1_000);
    assert_eq!(voucher_edits(&w).len(), 0);
    assert_eq!(count_of(&w, "voucher_edit_sign_wrong"), 0);
}

#[test]
fn a_payment_of_a_closed_day_edited_in_a_reopen_and_fix_takes_the_sign_of_the_reopen() {
    let w = day_with_event1();
    let saved =
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment(&w, "1,000", "10-10-2026")).unwrap();
    let id = saved.voucher_id;
    day::close(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    let to = |amount: &str| VoucherEntry::Payment(payment(&w, amount, "10-10-2026"));

    // The day is closed: the sign alone does not open it, the reopen does (FR-21.4).
    let opened = day::fix_open(
        &w.env.ctx,
        &w.owner,
        &DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: REASON.into(),
            sign: "adnan".into(),
        },
    )
    .unwrap();
    // Inside the fix the sign is not asked a second time; the reason is still sent.
    let error = edit_with(&w, id, to("1,500"), "", "").unwrap_err();
    assert_eq!(
        refusals(&error),
        [(ErrorCode::ErrReasonRequired, Some("reason".into()))]
    );
    // A sign that is typed anyway must still be the right one.
    let error = edit_with(&w, id, to("1,500"), REASON, "munshi").unwrap_err();
    assert_eq!(
        refusals(&error),
        [(ErrorCode::ErrSignWrong, Some("sign".into()))]
    );
    assert_eq!(state(&w, id).0, 1_000);
    edit_with(&w, id, to("1,500"), REASON, "").unwrap();
    assert_eq!(state(&w, id).0, 1_500);
    // The edit holds the sign the fix took, so the stamp shows it.
    let stamp = vouchers::get(&w.env.ctx, id).unwrap().flags.edited.unwrap();
    assert_eq!(stamp.sign.as_deref(), Some("adnan"));
    day::fix_close(
        &w.env.ctx,
        &w.owner,
        &DayFixCloseInput {
            token: opened.token,
        },
    )
    .unwrap();
    // The fix is over: the next edit of this voucher asks for the sign again.
    let error = edit_with(&w, id, to("1,200"), REASON, "").unwrap_err();
    assert_eq!(
        refusals(&error),
        [(ErrorCode::ErrSignRequired, Some("sign".into()))]
    );
    assert_chain_ok(&w.dir);
}

#[test]
fn the_sign_the_owner_chose_is_the_one_an_edit_asks_for() {
    let w = day_with_event1();
    let saved =
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment(&w, "1,000", "10-10-2026")).unwrap();
    let id = saved.voucher_id;
    owner_sign::set_owner_sign(
        &w.env.ctx,
        &w.owner,
        &Secret::new(OWNER_PASSWORD),
        "adnan",
        "Sana",
    )
    .unwrap();
    let to = VoucherEntry::Payment(payment(&w, "1,300", "10-10-2026"));
    // The old sign and its Urdu spelling are refused now.
    for old in ["adnan", "عدنان"] {
        let error = edit_with(&w, id, to.clone(), REASON, old).unwrap_err();
        assert_eq!(
            refusals(&error),
            [(ErrorCode::ErrSignWrong, Some("sign".into()))],
            "{old}"
        );
    }
    edit_with(&w, id, to, REASON, " SANA").unwrap();
    let stamp = vouchers::get(&w.env.ctx, id).unwrap().flags.edited.unwrap();
    assert_eq!(stamp.sign.as_deref(), Some("Sana"));
}
