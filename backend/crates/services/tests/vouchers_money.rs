//! Plan 15: the voucher screen's receipts and payments (PRD 15.1 to 15.4), edit and cancel by
//! reversal (PRD FR-15.3), the cash question (PRD FR-6.7), the payable rules of PRD v1.5
//! (FR-7.59), the collection round (PRD FR-7.61), make regular (PRD FR-13.8), the global search
//! (PRD FR-5.4) and the slip (PRD FR-15.2). The worked day itself is in `golden_day.rs`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::accounts::{AccountInput, AccountType, BalanceSide};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::masters::{MasterInput, StaffInput};
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{
    AdvanceInput, CollectionListInput, LineSource, MakeRegularInput, OpenBillsInput,
    OpenPayablesInput, PaymentInput, ReceiptInput, ReceiptsInput, VoucherCancelInput,
    VoucherEditInput, VoucherEntry, VoucherMode, VoucherPreviewInput, VoucherType,
};
use mandi_services::{ServiceError, bikri, ledger, vouchers};
use support::World;

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn receipt(account: i64, amount: &str) -> ReceiptInput {
    ReceiptInput {
        date: "10-10-2026".into(),
        account_id: account,
        person_key: None,
        amount: amount.into(),
        removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
        collector_staff_id: None,
        narration: String::new(),
    }
}

fn payment(account: i64, amount: &str, reason: &str) -> PaymentInput {
    PaymentInput {
        date: "10-10-2026".into(),
        account_id: account,
        person_key: None,
        amount: amount.into(),
        removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
        reason: reason.into(),
        narration: String::new(),
        confirm_cash_below_zero: false,
    }
}

fn advance(seller: i64, amount: &str) -> AdvanceInput {
    AdvanceInput {
        date: "10-10-2026".into(),
        seller_account_id: seller,
        amount: amount.into(),
        removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
        expected_goods: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: false,
    }
}

fn scalar(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

fn balance(w: &World, id: i64) -> (BalanceSide, i64) {
    let b = ledger::account_balance(&w.env.ctx, id).unwrap();
    (b.side, b.amount_paisa.rupees())
}

fn cash_id(w: &World) -> i64 {
    support::id_of(&w.env, "01010001")
}

fn event1_saved(w: &World) {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: support::event1(w),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

fn edit_receipt(
    w: &World,
    id: i64,
    input: ReceiptInput,
) -> Result<VoucherSavedTuple, ServiceError> {
    let row = vouchers::get(&w.env.ctx, id).unwrap();
    vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id,
            version: row.version,
            entry: VoucherEntry::Receipt(input),
            reason: "ٹھیک کیا".into(),
            confirm_cash_below_zero: false,
        },
    )
    .map(|s| (s.voucher_id, s.number, s.entry_id))
}

type VoucherSavedTuple = (i64, i64, i64);

/// The active bill rows as (bill line, credit line, paisa), oldest row first.
fn active_rows(w: &World) -> Vec<(i64, i64, i64)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT bill_line_id, payment_line_id, amount_paisa FROM bill_allocations
             WHERE is_active = 1 ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// Works the matching of a buyer out again and says whether anything changed. A canonical state
/// changes nothing (money agent 8.3).
fn rematch_changes(w: &World, account: i64, person_key: Option<&str>) -> (usize, usize) {
    let now = w.env.ctx.now_text();
    let done = w
        .env
        .ctx
        .db()
        .write(|tx| {
            mandi_services::matching::rematch(tx, w.owner.user_id, &now, account, person_key)
        })
        .unwrap();
    (done.switched_off, done.added)
}

fn advances_changes(w: &World, seller: i64) -> (usize, usize) {
    let now = w.env.ctx.now_text();
    let done = w
        .env
        .ctx
        .db()
        .write(|tx| mandi_services::matching::rematch_advances(tx, w.owner.user_id, &now, seller))
        .unwrap();
    (done.switched_off, done.added)
}

fn first_code(error: &ServiceError) -> ErrorCode {
    error.code()
}

#[test]
fn edit_of_a_buyer_receipt_reverses_it_and_works_the_matching_out_again() {
    let w = support::world();
    event1_saved(&w);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    let before = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    assert_eq!(
        before.version, 2,
        "insert 1, then the live posting is named"
    );
    assert!(before.editable);

    // PRD FR-15.3: edit 10,000 to 4,000. Same voucher, same number, a new posting.
    let (id, number, entry_id) =
        edit_receipt(&w, saved.voucher_id, receipt(w.akram, "4,000")).unwrap();
    assert_eq!((id, number), (saved.voucher_id, 1));
    assert_ne!(entry_id, saved.entry_id);
    let after = vouchers::get(&w.env.ctx, id).unwrap();
    assert_eq!(
        (after.amount_paisa, after.version, after.entry_id),
        (rupees(4_000), before.version + 1, Some(entry_id))
    );
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 8_880));
    // The old posting has its reversal on the same day; the old matching row is off with who and
    // when; one row of 4,000 is on.
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = {} AND entry_date = '2026-10-10'",
                saved.entry_id
            )
        ),
        1
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM bill_allocations WHERE is_active = 0 AND undone_by IS NOT NULL AND undone_at IS NOT NULL"
        ),
        1
    );
    let rows = active_rows(&w);
    assert_eq!(rows.iter().map(|r| r.2).sum::<i64>(), 400_000);
    // The books are as if only the new version had been saved.
    let bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: w.akram,
            person_key: None,
        },
    )
    .unwrap();
    assert_eq!(bills.open_paisa, rupees(8_880));
    assert_eq!(rematch_changes(&w, w.akram, None), (0, 0));
    // The audit row has the old and the new values and the reason.
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'edit' AND entity = 'voucher'"
        ),
        1
    );
    let (old, new, reason): (String, String, String) = raw(&w.dir)
        .query_row(
            "SELECT old_json, new_json, reason FROM audit_log WHERE action = 'edit' AND entity = 'voucher'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert!(old.contains("1000000"), "{old}");
    assert!(new.contains("400000"), "{new}");
    assert_eq!(reason, "ٹھیک کیا");
    // The next receipt is number 2: a number is never used again.
    let next = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "1,000")).unwrap();
    assert_eq!(next.number, 2);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

#[test]
fn cancel_gives_the_bill_back_keeps_the_number_and_refuses_a_second_change() {
    let w = support::world();
    event1_saved(&w);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();

    // A stale screen is refused; so is a cancel without a reason.
    let stale = vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: row.id,
            version: row.version + 5,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(stale.code(), ErrorCode::ErrVersionChanged);
    let no_reason = vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: row.id,
            version: row.version,
            reason: "  ".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(no_reason.code(), ErrorCode::ErrReasonRequired);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 2_880));

    let done = vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: row.id,
            version: row.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!((done.voucher_type, done.number), (VoucherType::Receipt, 1));
    assert_eq!(
        (done.new_balance.side, done.new_balance.amount_paisa),
        (BalanceSide::Dr, rupees(12_880))
    );
    // The receipt is منسوخ with its reason; the bill is open again; no row is active.
    let cancelled = vouchers::get(&w.env.ctx, row.id).unwrap();
    assert_eq!(
        (
            cancelled.status.as_str(),
            cancelled.cancel_reason.as_str(),
            cancelled.editable
        ),
        ("cancelled", "غلط اندراج", false)
    );
    assert!(active_rows(&w).is_empty());
    let bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: w.akram,
            person_key: None,
        },
    )
    .unwrap();
    assert_eq!(bills.open_paisa, rupees(12_880));
    assert_eq!(bills.bills[0].days, 0);
    assert_eq!(audit_count(&w.dir, "cancel"), 1);
    assert_eq!(rematch_changes(&w, w.akram, None), (0, 0));

    // It cannot be changed again, and its number is not reused.
    let again = vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: row.id,
            version: cancelled.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(again.code(), ErrorCode::ErrVoucherCancelled);
    let edit = edit_receipt(&w, row.id, receipt(w.akram, "1,000")).unwrap_err();
    assert_eq!(first_code(&edit), ErrorCode::ErrVoucherCancelled);
    let next = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "1,000")).unwrap();
    assert_eq!(next.number, 2);
    // The list shows both, the cancelled one marked.
    let list = vouchers::list(
        &w.env.ctx,
        &mandi_services::core::vouchers::VouchersListInput {
            sort: None,
            group_by_month: false,
            search: None,
            show_finished: true,
            voucher_type: Some(VoucherType::Receipt),
            from: String::new(),
            to: String::new(),
            account_id: Some(w.akram),
            person_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(
        list.rows
            .iter()
            .map(|r| (r.number, r.status.as_str()))
            .collect::<Vec<_>>(),
        [(1, "cancelled"), (2, "saved")]
    );
    // The total is of the saved one only (PRD FR-15.1).
    assert_eq!(list.totals.total_paisa, Some(rupees(1_000)));
}

#[test]
fn an_older_receipt_cancelled_hands_its_bill_to_the_next_receipt() {
    // Bills: بکری 1 12,880 and a loan of 3,000. Receipt A 12,880 settles بکری 1; receipt B 3,000
    // settles the loan. Cancel A: B takes the oldest bill (بکری 1) by the canonical working.
    let w = support::world();
    event1_saved(&w);
    vouchers::save_payment(
        &w.env.ctx,
        &w.owner,
        &payment(w.akram, "3,000", "نقد ادھار"),
    )
    .unwrap();
    let a = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "12,880")).unwrap();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "3,000")).unwrap();
    assert_eq!(balance(&w, w.akram).0, BalanceSide::Zero);
    let row = vouchers::get(&w.env.ctx, a.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: row.id,
            version: row.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: w.akram,
            person_key: None,
        },
    )
    .unwrap();
    assert_eq!(bills.bills.len(), 2);
    assert_eq!(
        (bills.bills[0].source, bills.bills[0].open_paisa.rupees()),
        (LineSource::Bikri, 9_880)
    );
    assert_eq!(
        (bills.bills[1].source, bills.bills[1].open_paisa.rupees()),
        (LineSource::CashLent, 3_000)
    );
    assert_eq!(bills.open_paisa.rupees(), 12_880);
    assert_eq!(rematch_changes(&w, w.akram, None), (0, 0));
}

#[test]
fn a_closed_day_refuses_edit_and_cancel_and_nothing_changes() {
    let w = support::world();
    event1_saved(&w);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    mandi_services::day::close(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    let entries = scalar(&w, "SELECT COUNT(*) FROM journal_entries");
    let edit = edit_receipt(&w, row.id, receipt(w.akram, "4,000")).unwrap_err();
    assert_eq!(first_code(&edit), ErrorCode::ErrDateClosed);
    let cancel = vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: row.id,
            version: row.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(cancel.code(), ErrorCode::ErrDateClosed);
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM journal_entries"), entries);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 2_880));
    assert!(!vouchers::get(&w.env.ctx, row.id).unwrap().editable);
}

#[test]
fn edit_may_change_the_account_and_both_buyers_are_matched_again() {
    let w = support::world();
    event1_saved(&w);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    // It was اکرم's money; it was really ارشد's.
    edit_receipt(&w, saved.voucher_id, receipt(w.arshad, "10,000")).unwrap();
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 12_880));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 8_684));
    assert_eq!(rematch_changes(&w, w.akram, None), (0, 0));
    assert_eq!(rematch_changes(&w, w.arshad, None), (0, 0));
    // The voucher cannot change its type.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    let wrong = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: row.id,
            version: row.version,
            entry: VoucherEntry::Payment(payment(w.arshad, "1,000", "x")),
            reason: String::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(wrong.code(), ErrorCode::ErrFormatInvalid);
}

#[test]
fn cancel_of_an_advance_frees_the_credit_it_was_adjusted_by() {
    // رحیم: advance 15,000 (پ-1) adjusted by بکری 1's credit. Cancel the advance: the credit has
    // nothing to adjust, so the whole 24,316 - 5,000 = 19,316 is payable.
    let w = support::world();
    let given = vouchers::save_advance(&w.env.ctx, &w.owner, &advance(w.rahim, "15,000")).unwrap();
    event1_saved(&w);
    assert_eq!(
        vouchers::open_payables(
            &w.env.ctx,
            &OpenPayablesInput {
                seller_id: w.rahim,
                person_key: None,
                include_settled: false
            }
        )
        .unwrap()
        .open_paisa,
        rupees(4_316)
    );
    let row = vouchers::get(&w.env.ctx, given.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: row.id,
            version: row.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let p = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.rahim,
            person_key: None,
            include_settled: false,
        },
    )
    .unwrap();
    assert_eq!(
        (
            p.open_paisa.rupees(),
            p.advances_open_paisa.rupees(),
            p.postings[0].advance_adjusted_paisa.rupees()
        ),
        (19_316, 0, 0)
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM advance_allocations WHERE is_active = 1"
        ),
        0
    );
    assert_eq!(advances_changes(&w, w.rahim), (0, 0));
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 19_316));
}

#[test]
fn edit_of_an_advance_moves_the_adjustment() {
    // The advance is 15,000, then 30,000 (more than the credit can take): the credit 24,316
    // adjusts 24,316... no: the credit is 24,316 - 5,000 نقد is a separate debit, so it adjusts
    // up to 24,316.
    let w = support::world();
    // The advance is dated before the بکری, as in the worked day (01-09-26), so the credit comes
    // after it whatever its entry id.
    let early = |amount: &str| AdvanceInput {
        date: "01-09-2026".into(),
        ..advance(w.rahim, amount)
    };
    let given = vouchers::save_advance(&w.env.ctx, &w.owner, &early("15,000")).unwrap();
    event1_saved(&w);
    let row = vouchers::get(&w.env.ctx, given.voucher_id).unwrap();
    let edited = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: row.id,
            version: row.version,
            entry: VoucherEntry::Advance(AdvanceInput {
                confirm_cash_below_zero: true,
                ..early("30,000")
            }),
            reason: String::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert_eq!(edited.number, 1);
    // The credit adjusts 24,316 of the 30,000; 5,684 stays open.
    let info = mandi_services::assist::advances(&w.env.ctx, w.rahim).unwrap();
    assert_eq!(info.open_paisa, rupees(5_684));
    assert_eq!(advances_changes(&w, w.rahim), (0, 0));
}

/// The removed fields as an old client sends them.
fn old_fields(json: serde_json::Value) -> mandi_services::core::vouchers::RemovedPaymentFields {
    serde_json::from_value(json).unwrap()
}

#[test]
fn every_voucher_posts_cash_and_its_slip_has_no_mode_bank_or_cheque_line() {
    // PRD 15.1, FR-10.4, FR-15.2 (v1.6): all money is روکڑ. An old client that sends a cheque, a
    // bank account or a mode is refused at each name, and nothing is posted.
    let w = support::world();
    event1_saved(&w);
    let bank = id_of_code(&w, "01020001");
    let cash_before = balance(&w, cash_id(&w));
    assert_eq!(cash_before, (BalanceSide::Dr, 45_000));
    let sent = serde_json::json!({ "mode": "cheque", "bank_account_id": 2,
        "cheque_no": "000123", "cheque_date": "12-10-2026" });
    let mut cheque = receipt(w.akram, "5,000");
    cheque.removed = old_fields(sent.clone());
    let refused = vouchers::save_receipt(&w.env.ctx, &w.owner, &cheque).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrFieldNotAllowed);
    let mut by_bank = payment(w.rahim, "1,000", "");
    by_bank.removed = old_fields(sent);
    let refused = vouchers::save_payment(&w.env.ctx, &w.owner, &by_bank).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrFieldNotAllowed);
    assert_eq!(balance(&w, cash_id(&w)), cash_before, "nothing was posted");
    assert_eq!(balance(&w, bank), (BalanceSide::Zero, 0));

    // The same receipt without them posts to روکڑ and never to the bank.
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "5,000")).unwrap();
    assert_eq!(balance(&w, cash_id(&w)), (BalanceSide::Dr, 50_000));
    assert_eq!(balance(&w, bank), (BalanceSide::Zero, 0));
    // A row saved before v1.6 as a cheque shows as saved, and its slip is the same plain slip.
    // A bank row of the old days: the table stays, nothing writes to it any more.
    raw(&w.dir)
        .execute(
            "INSERT INTO bank_accounts (bank_name, title, number, account_id)
             SELECT 'HBL', 'Maher Adnan', '0001', id FROM accounts WHERE code = '01020001'",
            [],
        )
        .unwrap();
    raw(&w.dir)
        .execute(
            "UPDATE vouchers SET mode = 'cheque', cheque_no = '000123', cheque_date = '2026-10-12',
                    bank_account_id = (SELECT id FROM bank_accounts)
             WHERE id = ?1",
            [saved.voucher_id],
        )
        .unwrap();
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    assert_eq!(
        (row.mode, row.cheque_no.as_str(), row.cheque_date.as_deref()),
        (VoucherMode::Cheque, "000123", Some("2026-10-12"))
    );
    for language in [
        mandi_services::core::auth::Language::Ur,
        mandi_services::core::auth::Language::En,
    ] {
        let slip = mandi_reports::print::voucher_print_data(
            w.env.ctx.db(),
            &mandi_reports::print::VoucherPrintDataInput {
                voucher_id: saved.voucher_id,
                language,
            },
        )
        .unwrap();
        assert_eq!(
            slip.pages.first().map(|p| p.payment_id.as_deref()),
            Some(None)
        );
        assert!(
            slip.header
                .facts
                .iter()
                .all(|f| !f.label_id.contains("bank") && !f.label_id.contains("cheque")),
            "{:?}",
            slip.header.facts
        );
    }
    // Editing that old row posts the new version to روکڑ and writes it as a plain cash row.
    let before_edit = balance(&w, cash_id(&w));
    assert_eq!(before_edit, (BalanceSide::Dr, 50_000));
    let edited = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: saved.voucher_id,
            version: row.version,
            entry: VoucherEntry::Receipt(receipt(w.akram, "4,000")),
            reason: "رقم درست کی".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(balance(&w, cash_id(&w)), (BalanceSide::Dr, 49_000));
    assert_eq!(balance(&w, bank), (BalanceSide::Zero, 0));
    let now = vouchers::get(&w.env.ctx, edited.voucher_id).unwrap();
    assert_eq!(
        (
            now.mode,
            now.bank_account_id,
            now.cheque_no.as_str(),
            now.cheque_date
        ),
        (VoucherMode::Cash, None, "", None)
    );
}

/// The row id of an account by its code.
fn id_of_code(w: &World, code: &str) -> i64 {
    w.env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, code))
        .unwrap()
        .unwrap_or_else(|| panic!("no account {code}"))
}

#[test]
fn the_cash_question_covers_payment_edit_and_cancel() {
    // PRD FR-6.7: cash 20,000, a payment of 25,000 leaves -5,000 and asks.
    let w = support::world_with(20_000, true);
    event1_saved(&w); // نقد 5,000 leaves cash 15,000 and rahim a credit
    let cash_now = balance(&w, cash_id(&w));
    assert_eq!(cash_now, (BalanceSide::Dr, 15_000));
    let mut input = payment(w.rahim, "20,000", "");
    let refused = vouchers::save_payment(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(
        code_and_value(&refused, "cash_after_paisa"),
        (ErrorCode::ErrCashBelowZeroNotConfirmed, Some(-500_000))
    );
    assert_eq!(balance(&w, cash_id(&w)), cash_now);
    // The preview says so too.
    let preview = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::Payment(input.clone()),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert_eq!(
        (preview.cash.cash_after_paisa, preview.cash.needs_confirm),
        (rupees(-5_000), true)
    );
    input.confirm_cash_below_zero = true;
    let saved = vouchers::save_payment(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(balance(&w, cash_id(&w)), (BalanceSide::Cr, 5_000));

    // Edit 20,000 down to 19,000: cash goes UP by 1,000, so it never asks, even at -4,000.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: row.id,
            version: row.version,
            entry: VoucherEntry::Payment(payment(w.rahim, "19,000", "")),
            reason: String::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(balance(&w, cash_id(&w)), (BalanceSide::Cr, 4_000));
    // Edit it up to 21,000: cash goes down by 2,000 and ends below 0, so it asks.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    let up = VoucherEditInput {
        id: row.id,
        version: row.version,
        entry: VoucherEntry::Payment(payment(w.rahim, "21,000", "")),
        reason: String::new(),
        confirm_cash_below_zero: false,
    };
    let refused = vouchers::edit(&w.env.ctx, &w.owner, &up).unwrap_err();
    assert_eq!(
        code_and_value(&refused, "cash_after_paisa"),
        (ErrorCode::ErrCashBelowZeroNotConfirmed, Some(-600_000))
    );
    assert_eq!(balance(&w, cash_id(&w)), (BalanceSide::Cr, 4_000));
    assert_eq!(
        vouchers::get(&w.env.ctx, row.id).unwrap().version,
        row.version
    );

    // Cancel a receipt that would take cash below 0 (PRD FR-6.7: "an edit, cancel or undo that
    // takes cash back"): cash is -4,000; a receipt of 10,000 brings it to 6,000; cancelling it
    // leaves -4,000 and asks.
    let r = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    assert_eq!(balance(&w, cash_id(&w)), (BalanceSide::Dr, 6_000));
    let row = vouchers::get(&w.env.ctx, r.voucher_id).unwrap();
    let cancel = VoucherCancelInput {
        id: row.id,
        version: row.version,
        reason: "x".into(),
        confirm_cash_below_zero: false,
    };
    assert_eq!(
        vouchers::cancel(&w.env.ctx, &w.owner, &cancel)
            .unwrap_err()
            .code(),
        ErrorCode::ErrCashBelowZeroNotConfirmed
    );
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            confirm_cash_below_zero: true,
            ..cancel
        },
    )
    .unwrap();
    assert_eq!(balance(&w, cash_id(&w)), (BalanceSide::Cr, 4_000));
    // An advance asks too (PRD FR-6.7).
    let big = vouchers::save_advance(&w.env.ctx, &w.owner, &advance(w.karim, "100")).unwrap_err();
    assert_eq!(big.code(), ErrorCode::ErrCashBelowZeroNotConfirmed);
}

#[test]
fn d4_a29_an_opening_credit_and_a_general_credit_are_payable() {
    use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;
    let w = support::world();
    // کریم opens with 10,160 جمع (from the old books), dated 10-10-26.
    let version = mandi_services::accounts::get(&w.env.ctx, w.karim)
        .unwrap()
        .version;
    mandi_services::accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: w.karim,
                version,
                opening_date: "10-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: "10,160".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    let payables = |seller: i64| {
        vouchers::open_payables(
            &w.env.ctx,
            &OpenPayablesInput {
                seller_id: seller,
                person_key: None,
                include_settled: true,
            },
        )
        .unwrap()
    };
    let p = payables(w.karim);
    assert_eq!(
        (
            p.postings[0].source,
            p.postings[0].still_open_paisa.rupees()
        ),
        (LineSource::Opening, 10_160)
    );
    // An ادائیگی of 4,000: paid 4,000, still open 6,160, balance 6,160 جمع, and no پیشگی note.
    let preview = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::Payment(payment(w.karim, "4,000", "")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert_eq!(preview.payable_open_paisa, Some(rupees(10_160)));
    assert!(!preview.payable_is_zero_for_seller);
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.karim, "4,000", "")).unwrap();
    let p = payables(w.karim);
    assert_eq!(
        (
            p.postings[0].paid_paisa.rupees(),
            p.postings[0].still_open_paisa.rupees(),
            p.open_paisa.rupees(),
            p.balance.side,
            p.balance.amount_paisa.rupees()
        ),
        (4_000, 6_160, 6_160, BalanceSide::Cr, 6_160)
    );

    // رحیم has 15,000 of advance open (the tick will not be used); a جنرل credit of 1,000 on him
    // is payable and never adjusts the advance (PRD 16.4).
    vouchers::save_advance(&w.env.ctx, &w.owner, &advance(w.rahim, "15,000")).unwrap();
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(cash_id(&w)), Side::Dr, rupees(1_000)),
        LedgerLine::new(AccountId::new(w.rahim), Side::Cr, rupees(1_000)),
    ])
    .unwrap();
    w.env
        .ctx
        .db()
        .write(|tx| {
            ledger::post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10)
                        .unwrap(),
                    posting_type: PostingType::General,
                    source_type: SourceType::Voucher,
                    source_id: 9_001,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
    assert_eq!(advances_changes(&w, w.rahim), (0, 0));
    let p = payables(w.rahim);
    assert_eq!(
        (
            p.open_paisa.rupees(),
            p.advances_open_paisa.rupees(),
            p.owes_firm_paisa.rupees()
        ),
        (1_000, 15_000, 0)
    );
    assert_eq!(p.postings[0].source, LineSource::General);
    // Σ still open − Σ open advances = his balance (credit less debit) = -14,000.
    assert_eq!(
        (p.balance.side, p.balance.amount_paisa.rupees()),
        (BalanceSide::Dr, 14_000)
    );
    assert_eq!(
        mandi_services::assist::advances(&w.env.ctx, w.rahim)
            .unwrap()
            .open_paisa,
        rupees(15_000)
    );
}

#[test]
fn a_sellers_opening_credit_is_his_oldest_payable_whatever_its_date() {
    // PRD FR-7.59 (v1.5): a seller's opening credit is his oldest posting. کریم has a جنرل credit
    // of 3,000 dated 05-10, and his opening credit of 10,160 is entered later, dated 10-10. By
    // date the opening is the newer one; as a payable it is the oldest, so a payment of 4,000
    // settles it first, in the full reading and in the short one alike.
    use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;
    let w = support::world();
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(cash_id(&w)), Side::Dr, rupees(3_000)),
        LedgerLine::new(AccountId::new(w.karim), Side::Cr, rupees(3_000)),
    ])
    .unwrap();
    w.env
        .ctx
        .db()
        .write(|tx| {
            ledger::post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 5).unwrap(),
                    posting_type: PostingType::General,
                    source_type: SourceType::Voucher,
                    source_id: 9_002,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
    let version = mandi_services::accounts::get(&w.env.ctx, w.karim)
        .unwrap()
        .version;
    mandi_services::accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: w.karim,
                version,
                opening_date: "10-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: "10,160".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    let payables = |include_settled: bool| {
        vouchers::open_payables(
            &w.env.ctx,
            &OpenPayablesInput {
                seller_id: w.karim,
                person_key: None,
                include_settled,
            },
        )
        .unwrap()
    };
    let shape = |p: &mandi_services::core::vouchers::OpenPayables| {
        p.postings
            .iter()
            .map(|x| {
                (
                    x.source,
                    x.entry_date.clone(),
                    x.paid_paisa.rupees(),
                    x.still_open_paisa.rupees(),
                )
            })
            .collect::<Vec<_>>()
    };
    let before = payables(true);
    assert_eq!(
        shape(&before),
        [
            (LineSource::Opening, "2026-10-10".to_owned(), 0, 10_160),
            (LineSource::General, "2026-10-05".to_owned(), 0, 3_000),
        ]
    );
    assert_eq!(shape(&payables(false)), shape(&before));
    // The save settles the opening first.
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.karim, "4,000", "")).unwrap();
    let after = payables(true);
    assert_eq!(
        shape(&after),
        [
            (LineSource::Opening, "2026-10-10".to_owned(), 4_000, 6_160),
            (LineSource::General, "2026-10-05".to_owned(), 0, 3_000),
        ]
    );
    assert_eq!(shape(&payables(false)), shape(&after));
    assert_eq!(after.open_paisa, rupees(9_160));
    assert_eq!(
        (after.balance.side, after.balance.amount_paisa.rupees()),
        (BalanceSide::Cr, 9_160)
    );
}

#[test]
fn open_bills_give_the_oldest_200_and_say_how_many_there_are() {
    // PRD FR-7.58, plan 15: a buyer can have tens of thousands of open bills; the read gives the
    // oldest 200, oldest first, and the count and the open total of all of them.
    use mandi_services::core::vouchers::MAX_OPEN_ROWS;
    let w = support::world();
    for _ in 0..205 {
        let mut loan = payment(w.akram, "1,000", "نقد ادھار");
        loan.confirm_cash_below_zero = true;
        vouchers::save_payment(&w.env.ctx, &w.owner, &loan).unwrap();
    }
    let bills = |w: &World| {
        vouchers::open_bills(
            &w.env.ctx,
            &OpenBillsInput {
                account_id: w.akram,
                person_key: None,
            },
        )
        .unwrap()
    };
    let all = bills(&w);
    assert_eq!(MAX_OPEN_ROWS, 200);
    assert_eq!(
        (all.bills.len(), all.total_count, all.truncated),
        (200, 205, true)
    );
    assert_eq!(all.open_paisa, rupees(205_000));
    // The oldest 200 are loans 1 to 200, in order.
    assert_eq!(all.bills.first().and_then(|b| b.number), Some(1));
    assert_eq!(all.bills.last().and_then(|b| b.number), Some(200));
    // A receipt of 1,500 settles bill 1 and 500 of bill 2. Now 204 are open, the first 200 shown
    // start at bill 2 with 500 left, and the open total is of all 204.
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "1,500")).unwrap();
    let after = bills(&w);
    assert_eq!(
        (after.bills.len(), after.total_count, after.truncated),
        (200, 204, true)
    );
    assert_eq!(after.open_paisa, rupees(203_500));
    assert_eq!(after.bills[0].number, Some(2));
    assert_eq!(after.bills[0].open_paisa, rupees(500));
    assert_eq!(after.bills[1].open_paisa, rupees(1_000));
    assert_eq!(after.bills.last().and_then(|b| b.number), Some(201));
    // "پورا بقایا" is the full open total, and a receipt of exactly that clears him.
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "203,500")).unwrap();
    assert_eq!(saved.new_balance.side, BalanceSide::Zero);
    let clear = bills(&w);
    assert_eq!(
        (clear.bills.len(), clear.total_count, clear.truncated),
        (0, 0, false)
    );
}

#[test]
fn open_payables_give_the_oldest_200_and_say_how_many_there_are() {
    // PRD FR-7.59, plan 15: the same cap for a seller's payable postings.
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;
    let w = support::world();
    for i in 0..205_i64 {
        let entry = build_entry(vec![
            LedgerLine::new(AccountId::new(cash_id(&w)), Side::Dr, rupees(1_000)),
            LedgerLine::new(AccountId::new(w.karim), Side::Cr, rupees(1_000)),
        ])
        .unwrap();
        w.env
            .ctx
            .db()
            .write(|tx| {
                ledger::post(
                    &w.env.ctx,
                    tx,
                    w.owner.user_id,
                    &PostingEvent {
                        date: mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10)
                            .unwrap(),
                        posting_type: PostingType::General,
                        source_type: SourceType::Voucher,
                        source_id: 9_100 + i,
                        narration: None,
                        reverses: None,
                        entry,
                    },
                )
            })
            .unwrap();
    }
    let payables = |include_settled: bool| {
        vouchers::open_payables(
            &w.env.ctx,
            &OpenPayablesInput {
                seller_id: w.karim,
                person_key: None,
                include_settled,
            },
        )
        .unwrap()
    };
    let first = payables(false);
    assert_eq!(
        (first.postings.len(), first.total_count, first.truncated),
        (200, 205, true)
    );
    assert_eq!(first.open_paisa, rupees(205_000));
    // A payment of 1,500 settles posting 1 and 500 of posting 2: 204 are open, 200 shown.
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.karim, "1,500", "")).unwrap();
    let after = payables(false);
    assert_eq!(
        (after.postings.len(), after.total_count, after.truncated),
        (200, 204, true)
    );
    assert_eq!(after.open_paisa, rupees(203_500));
    assert_eq!(
        (
            after.postings[0].paid_paisa,
            after.postings[0].still_open_paisa
        ),
        (rupees(500), rupees(500))
    );
    assert_eq!(after.postings[1].still_open_paisa, rupees(1_000));
    // The history is not capped and agrees with the short reading on the open ones.
    let history = payables(true);
    assert_eq!(
        (
            history.postings.len(),
            history.total_count,
            history.truncated
        ),
        (205, 205, false)
    );
    assert_eq!(history.open_paisa, after.open_paisa);
    let history_open: Vec<_> = history
        .postings
        .iter()
        .filter(|p| p.still_open_paisa > Paisa::ZERO)
        .take(200)
        .map(|p| (p.paid_paisa, p.still_open_paisa))
        .collect();
    let short: Vec<_> = after
        .postings
        .iter()
        .map(|p| (p.paid_paisa, p.still_open_paisa))
        .collect();
    assert_eq!(history_open, short);
}

#[test]
fn the_preview_of_an_entry_that_cannot_lower_cash_does_not_read_cash() {
    // PRD FR-6.7: such an entry never asks, so the balance is not read (a receipt in cash or by
    // bank, or an edit that does not lower cash). A payment, or an edit that lowers cash, reads it.
    let w = support::world();
    event1_saved(&w);
    let cash_now = rupees(balance(&w, cash_id(&w)).1);
    let preview = |entry: VoucherEntry, replacing: Option<i64>| {
        vouchers::preview(
            &w.env.ctx,
            &VoucherPreviewInput {
                entry,
                replacing_voucher_id: replacing,
            },
        )
        .unwrap()
    };
    let unread = preview(VoucherEntry::Receipt(receipt(w.akram, "1,000")), None).cash;
    assert_eq!(
        (
            unread.cash_before_paisa,
            unread.cash_after_paisa,
            unread.change_paisa
        ),
        (Paisa::ZERO, Paisa::ZERO, rupees(1_000))
    );
    assert!(!unread.lowers_cash && !unread.below_zero && !unread.needs_confirm);
    // A payment reads the balance and asks when it would go below zero.
    let read = preview(VoucherEntry::Payment(payment(w.rahim, "1,000", "")), None).cash;
    assert_eq!(read.cash_before_paisa, cash_now);
    assert_eq!(read.change_paisa, rupees(-1_000));
    assert!(read.lowers_cash);
    // An edit of a receipt to a smaller amount lowers cash: the balance is read.
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "5,000")).unwrap();
    let cash_after_receipt = rupees(balance(&w, cash_id(&w)).1);
    let smaller = preview(
        VoucherEntry::Receipt(receipt(w.akram, "2,000")),
        Some(saved.voucher_id),
    )
    .cash;
    assert_eq!(smaller.cash_before_paisa, cash_after_receipt);
    assert_eq!(smaller.change_paisa, rupees(-3_000));
    // An edit to a bigger amount raises cash: not read.
    let bigger = preview(
        VoucherEntry::Receipt(receipt(w.akram, "6,000")),
        Some(saved.voucher_id),
    )
    .cash;
    assert_eq!(
        (bigger.cash_before_paisa, bigger.change_paisa),
        (Paisa::ZERO, rupees(1_000))
    );
    assert!(!bigger.needs_confirm);
}

#[test]
fn a_seller_returns_what_he_owes_beyond_his_advances() {
    // Payable 4,316; a payment of 6,000 makes him owe 1,684; he returns 1,684 by a وصولی, and
    // 1,685 is refused (PRD FR-15.7, FR-15.12).
    let w = support::world();
    vouchers::save_advance(&w.env.ctx, &w.owner, &advance(w.rahim, "15,000")).unwrap();
    event1_saved(&w);
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "6,000", "")).unwrap();
    let p = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.rahim,
            person_key: None,
            include_settled: false,
        },
    )
    .unwrap();
    assert_eq!(
        (
            p.open_paisa.rupees(),
            p.owes_firm_paisa.rupees(),
            p.advances_open_paisa.rupees()
        ),
        (0, 1_684, 0)
    );
    assert_eq!(
        (p.balance.side, p.balance.amount_paisa.rupees()),
        (BalanceSide::Dr, 1_684)
    );
    let refused =
        vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "1,685")).unwrap_err();
    assert_eq!(
        code_and_value(&refused, "open_paisa"),
        (ErrorCode::ErrReturnExceedsOpen, Some(168_400))
    );
    let preview = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::Receipt(receipt(w.rahim, "1,684")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty());
    assert_eq!(
        (
            preview.owes_settled_paisa.rupees(),
            preview.advance_effects.len()
        ),
        (1_684, 0)
    );
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "1,684")).unwrap();
    assert_eq!(saved.new_balance.side, BalanceSide::Zero);
    // A seller we owe money to, with no advance, can return nothing.
    let refused =
        vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.karim, "100")).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
}

#[test]
fn a_back_dated_seller_return_is_refused_with_the_most_he_can_return_at_that_date() {
    // PRD FR-15.12. Advance 1 (15,000, 01-09) is fully used by the بکری credit of 10-10; advance 2
    // (5,000, 10-10, saved after the بکری) is open. Today he can return 5,000. A return of 5,000
    // dated 05-10 would take advance 1 and leave the بکری credit nothing to adjust, so the
    // payable would rise: nothing can be returned at that date, and the refusal says 0.
    let w = support::world();
    let mut first = advance(w.rahim, "15,000");
    first.date = "01-09-2026".into();
    first.confirm_cash_below_zero = true;
    vouchers::save_advance(&w.env.ctx, &w.owner, &first).unwrap();
    event1_saved(&w);
    let mut second = advance(w.rahim, "5,000");
    second.confirm_cash_below_zero = true;
    vouchers::save_advance(&w.env.ctx, &w.owner, &second).unwrap();
    let mut early = receipt(w.rahim, "5,000");
    early.date = "05-10-2026".into();
    let refused = vouchers::save_receipt(&w.env.ctx, &w.owner, &early).unwrap_err();
    assert_eq!(
        code_and_value(&refused, "open_paisa"),
        (ErrorCode::ErrReturnExceedsOpen, Some(0))
    );
    let preview = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::Receipt(early.clone()),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert_eq!(preview.return_limit_paisa, Some(Paisa::ZERO));
    assert_eq!(preview.errors.len(), 1);
    assert_eq!(preview.errors[0].code, ErrorCode::ErrReturnExceedsOpen);
    // The same 5,000 dated today takes advance 2 and passes; 5,001 does not.
    let today = receipt(w.rahim, "5,001");
    assert_eq!(
        code_and_value(
            &vouchers::save_receipt(&w.env.ctx, &w.owner, &today).unwrap_err(),
            "open_paisa"
        ),
        (ErrorCode::ErrReturnExceedsOpen, Some(500_000))
    );
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "5,000")).unwrap();
}

#[test]
fn the_fast_bill_read_falls_back_to_the_full_read_when_the_rows_are_not_canonical() {
    // Three loans of 5,000 and a receipt of 7,000 leave 8,000 open. With the matching rows put
    // out of use, open bills (15,000) and an unmatched credit (7,000) exist together, which the
    // app never writes. The walk would stop after the two newest bills; the guard sees that it
    // went over 8,000 and reads everything, as the full read does.
    let w = support::world();
    for _ in 0..3 {
        let mut loan = payment(w.akram, "5,000", "نقد ادھار");
        loan.confirm_cash_below_zero = true;
        vouchers::save_payment(&w.env.ctx, &w.owner, &loan).unwrap();
    }
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "7,000")).unwrap();
    let conn = raw(&w.dir);
    conn.execute(
        "UPDATE bill_allocations SET is_active = 0, undone_at = '2026-10-10T00:00:00Z', undone_by = 1",
        [],
    )
    .unwrap();
    let expected = mandi_db::matching::expected_open_bills(&conn, w.akram, None).unwrap();
    assert_eq!(expected, 800_000);
    let fast = mandi_db::matching::open_bills_upto(&conn, w.akram, None, expected).unwrap();
    let full = mandi_db::matching::open_bills(&conn, w.akram, None).unwrap();
    assert_eq!(fast.len(), 3);
    assert_eq!(fast, full);
}

#[test]
fn the_edit_preview_works_as_if_the_old_voucher_were_taken_back() {
    let w = support::world();
    event1_saved(&w);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    // Editing 10,000 to 12,880: the bill is shown as it was before the old receipt (12,880 open),
    // the new amount settles all of it.
    let preview = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::Receipt(receipt(w.akram, "12,880")),
            replacing_voucher_id: Some(saved.voucher_id),
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty());
    assert_eq!(
        (
            preview.current_balance.side,
            preview.current_balance.amount_paisa
        ),
        (BalanceSide::Dr, rupees(12_880))
    );
    assert_eq!(preview.new_balance.side, BalanceSide::Zero);
    assert_eq!(preview.bill_effects.len(), 1);
    assert_eq!(
        (
            preview.bill_effects[0].bill.open_paisa.rupees(),
            preview.bill_effects[0].settled_paisa.rupees(),
            preview.bill_effects[0].still_open_paisa.rupees()
        ),
        (12_880, 12_880, 0)
    );
    // Cash: the old receipt's 10,000 is given back first, then 12,880 comes in: +2,880.
    assert_eq!(preview.cash.change_paisa, rupees(2_880));
    // A replaced voucher that is cancelled or does not exist is a mistake of the input.
    let missing = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::Receipt(receipt(w.akram, "1")),
            replacing_voucher_id: Some(999),
        },
    )
    .unwrap();
    assert_eq!(missing.errors[0].code, ErrorCode::ErrNotFound);
    // And the edit does what the preview said.
    let (_, _, _) = edit_receipt(&w, saved.voucher_id, receipt(w.akram, "12,880")).unwrap();
    assert_eq!(balance(&w, w.akram).0, BalanceSide::Zero);
}

#[test]
fn the_collection_round_lists_the_planned_buyers_and_saves_the_receipts() {
    let w = support::world();
    // A collector (staff), and the buyers' collection weekday: 10-10-2026 is a Saturday (5).
    let mut collector_input = StaffInput {
        id: None,
        version: None,
        name_ur: "اسلم".into(),
        name_en: "Aslam".into(),
        code: String::new(),
        job: "collector".into(),
        phone: String::new(),
        confirmed_rules_words: Vec::new(),
    };
    let collector = loop {
        match mandi_services::masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::Staff(collector_input.clone()),
        ) {
            Ok(row) => break row.id(),
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(mandi_services::core::error::ErrorValue::Text(t)) =
                        item.values.get("words")
                    {
                        collector_input
                            .confirmed_rules_words
                            .extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    };
    for (id, collector_id) in [(w.akram, Some(collector)), (w.arshad, None)] {
        let mut input: AccountInput = account_form(&w, id);
        input.collection_weekday = Some(5);
        input.collector_staff_id = collector_id;
        let mut words: Vec<String> = Vec::new();
        loop {
            input.confirmed_rules_words = words.clone();
            match mandi_services::accounts::save(&w.env.ctx, &w.owner, &input) {
                Ok(_) => break,
                Err(ServiceError::Rejected(items))
                    if items
                        .iter()
                        .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
                {
                    for item in items {
                        if let Some(mandi_services::core::error::ErrorValue::Text(t)) =
                            item.values.get("words")
                        {
                            words.extend(t.split(' ').map(str::to_owned));
                        }
                    }
                }
                Err(other) => panic!("{other:?}"),
            }
        }
    }
    event1_saved(&w);
    let list = vouchers::collection_list(
        &w.env.ctx,
        &CollectionListInput {
            date: "10-10-2026".into(),
            collector_staff_id: None,
            city_id: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(
        (list.weekday, list.total_paisa.rupees()),
        (5, 12_880 + 18_684)
    );
    // Two groups: the buyers of the collector, then those with none (a missing collector sorts
    // first), each with its dues and the age of the oldest bill.
    assert_eq!(list.groups.len(), 2);
    let mut seen: Vec<(Option<String>, i64, i64, Option<i64>)> = Vec::new();
    for group in &list.groups {
        for buyer in &group.buyers {
            seen.push((
                group.collector.as_ref().map(|c| c.name_en.clone()),
                buyer.account.id,
                buyer.dues_paisa.rupees(),
                buyer.oldest_days,
            ));
        }
    }
    seen.sort();
    assert_eq!(
        seen,
        [
            (None, w.arshad, 18_684, Some(0)),
            (Some("Aslam".to_owned()), w.akram, 12_880, Some(0)),
        ]
    );
    // Plan 38 B1: the list pages by buyer down the groups; the count and the total are over every
    // buyer, and a group keeps its own total.
    assert_eq!(list.total_count, 2);
    let page = |offset: u32, limit: u32| {
        vouchers::collection_list(
            &w.env.ctx,
            &CollectionListInput {
                date: "10-10-2026".into(),
                collector_staff_id: None,
                city_id: None,
                offset,
                limit,
            },
        )
        .unwrap()
    };
    let mut paged = Vec::new();
    for offset in [0, 1, 2] {
        let p = page(offset, 1);
        assert_eq!(
            (p.total_count, p.total_paisa.rupees()),
            (2, 12_880 + 18_684)
        );
        assert_eq!(
            p.groups.iter().map(|g| g.buyers.len()).sum::<usize>(),
            usize::from(offset < 2)
        );
        paged.extend(
            p.groups
                .iter()
                .flat_map(|g| g.buyers.iter().map(|b| b.account.id)),
        );
        for group in &p.groups {
            // A group's total is over all its buyers, not the page.
            assert_eq!(
                group.total_paisa,
                list.groups
                    .iter()
                    .find(|x| x.collector == group.collector && x.city == group.city)
                    .unwrap()
                    .total_paisa
            );
        }
    }
    let whole: Vec<i64> = list
        .groups
        .iter()
        .flat_map(|g| g.buyers.iter().map(|b| b.account.id))
        .collect();
    assert_eq!(paged, whole);
    for limit in [25, 50, 100] {
        assert_eq!(page(0, limit).groups, list.groups);
    }
    // Every group of the whole filter with its count and total, on every page (plan 38 B2).
    assert_eq!(list.group_totals.len(), list.groups.len());
    assert_eq!(
        list.group_totals.iter().map(|g| g.count).sum::<u32>(),
        list.total_count
    );
    for (total, group) in list.group_totals.iter().zip(&list.groups) {
        assert_eq!(
            (&total.collector, &total.city, total.total_paisa),
            (&group.collector, &group.city, group.total_paisa)
        );
    }
    for offset in [0, 1, 2] {
        assert_eq!(page(offset, 1).group_totals, list.group_totals);
    }
    // Another day of the week lists nobody.
    let monday = vouchers::collection_list(
        &w.env.ctx,
        &CollectionListInput {
            date: "05-10-2026".into(),
            collector_staff_id: None,
            city_id: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert!(monday.groups.is_empty());
    // The round: اکرم pays 5,000; ارشد is left at 0 and skipped; a wrong amount fails alone.
    let mut bad = receipt(w.arshad, "abc");
    bad.narration = "x".into();
    let saved = vouchers::save_receipts_batch(
        &w.env.ctx,
        &w.owner,
        &ReceiptsInput {
            rows: vec![
                receipt(w.akram, "5,000"),
                receipt(w.arshad, "0"),
                bad,
                receipt(w.arshad, "1,000"),
            ],
        },
    )
    .unwrap();
    assert_eq!(saved.rows.len(), 4);
    assert!(saved.rows[0].saved.is_some() && !saved.rows[0].skipped);
    assert!(saved.rows[1].saved.is_none() && saved.rows[1].skipped);
    assert!(saved.rows[2].saved.is_none() && !saved.rows[2].skipped);
    assert_eq!(saved.rows[2].errors[0].field, "rows[2].amount");
    assert_eq!(saved.rows[2].errors[0].code, ErrorCode::ErrNotANumber);
    assert!(saved.rows[3].saved.is_some());
    assert_eq!(saved.saved_paisa.rupees(), 6_000);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 7_880));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 17_684));
}

/// The account form of an existing account, to change one field of it.
fn account_form(w: &World, id: i64) -> AccountInput {
    let row = mandi_services::accounts::get(&w.env.ctx, id).unwrap();
    AccountInput {
        id: Some(id),
        version: Some(row.version),
        account_type: AccountType::Buyer,
        control_id: None,
        code: row.code.clone(),
        name_ur: row.name_ur.clone(),
        name_en: row.name_en.clone(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: row.father_or_firm_ur.clone(),
        father_or_firm_en: row.father_or_firm_en.clone(),
        city_id: row.city.as_ref().map(|c| c.id),
        adda_id: None,
        phone1: row.phone1.clone(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: row.credit_allowed,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    }
}

/// Saves a بکری whose one buyer is the one-time person `name` on credit, at no commission.
fn one_time_buyer_owes(w: &World, name: (&str, &str), phone: &str, qty: &str, rate: &str) {
    use mandi_services::core::bikri::{LineChargeMethod, PaymentType, PukhtaInput};
    let mut line = support::sale("s-1", w.pukhta_buyer, qty, rate);
    line.pukhta = Some(PukhtaInput {
        name_ur: name.0.into(),
        name_en: name.1.into(),
        phone: phone.into(),
    });
    line.payment_type = Some(PaymentType::Credit);
    line.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
    line.seller_comm = support::typed(LineChargeMethod::Percent, "0");
    line.laga = support::typed(LineChargeMethod::PerUnit, "0");
    let mut lot = support::bare_lot(w, "lot-1", w.rahim, w.banana, qty);
    lot.sales = vec![line];
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: support::draft(vec![lot]),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

/// Makes the person a regular account, confirming the rule-made spellings the form asks about.
fn make_regular_confirmed(
    w: &World,
    key: &str,
    side: mandi_services::core::bikri::PukhtaSide,
    mut input: AccountInput,
) -> mandi_services::core::vouchers::MadeRegular {
    let mut words: Vec<String> = Vec::new();
    loop {
        input.confirmed_rules_words = words.clone();
        match vouchers::make_regular(
            &w.env.ctx,
            &w.owner,
            &MakeRegularInput {
                person_key: key.to_owned(),
                side,
                account_input: input.clone(),
            },
        ) {
            Ok(made) => return made,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(mandi_services::core::error::ErrorValue::Text(t)) =
                        item.values.get("words")
                    {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

/// The form of a new account for a person, with no id and no code.
fn new_account_form(w: &World, like: i64, ty: AccountType, names: (&str, &str)) -> AccountInput {
    let mut input = account_form(w, like);
    input.id = None;
    input.version = None;
    input.code = String::new();
    input.account_type = ty;
    input.name_ur = names.0.into();
    input.name_en = names.1.into();
    input.phone1 = "03000000000".into();
    input.city_id = None;
    input
}

#[test]
fn make_regular_moves_the_balance_of_a_one_time_buyer_to_a_new_account() {
    use mandi_services::core::bikri::PukhtaSide;
    let w = support::world();
    one_time_buyer_owes(&w, ("شاہد", "Shahid"), "0300-0000000", "2", "1000");
    let key = mandi_services::core::text::person_key("شاہد", "0300-0000000");
    let input = new_account_form(&w, w.akram, AccountType::Buyer, ("شاہد", "Shahid"));
    let made = make_regular_confirmed(&w, &key, PukhtaSide::Buyer, input.clone());
    // The new account is a buyer and holds his 2,000; the person has none left.
    assert_eq!(
        (made.moved.side, made.moved.amount_paisa),
        (BalanceSide::Dr, rupees(2_000))
    );
    assert!(made.entry_id.is_some());
    assert_eq!(made.account.code, "0022");
    assert_eq!(balance(&w, made.account.id), (BalanceSide::Dr, 2_000));
    assert_eq!(
        ledger::person_balance(&w.env.ctx, w.pukhta_buyer, &key)
            .unwrap()
            .side,
        BalanceSide::Zero
    );
    // His old bill is settled by the move (no open bill on the person), the new account has one.
    let person_bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: w.pukhta_buyer,
            person_key: Some(key.clone()),
        },
    )
    .unwrap();
    assert!(person_bills.bills.is_empty());
    assert_eq!(rematch_changes(&w, w.pukhta_buyer, Some(&key)), (0, 0));
    let new_bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: made.account.id,
            person_key: None,
        },
    )
    .unwrap();
    assert_eq!(new_bills.open_paisa.rupees(), 2_000);
    assert_eq!(audit_count(&w.dir, "make_regular"), 1);
    // The move touches no income or expense: no partner share rows for it.
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM partner_share_rows WHERE journal_entry_id = {}",
                made.entry_id.unwrap()
            )
        ),
        0
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    // A person not on the books, a side that does not fit the account and an account with an id
    // are refused.
    let unknown = vouchers::make_regular(
        &w.env.ctx,
        &w.owner,
        &MakeRegularInput {
            person_key: "نامعلوم|0".into(),
            side: PukhtaSide::Buyer,
            account_input: input.clone(),
        },
    )
    .unwrap_err();
    assert_eq!(unknown.code(), ErrorCode::ErrNotFound);
    let wrong_side = vouchers::make_regular(
        &w.env.ctx,
        &w.owner,
        &MakeRegularInput {
            person_key: key,
            side: PukhtaSide::Seller,
            account_input: input,
        },
    )
    .unwrap_err();
    assert_eq!(wrong_side.code(), ErrorCode::ErrFormatInvalid);
}

#[test]
fn make_regular_refuses_an_opening_on_the_form_so_the_balance_is_not_counted_twice() {
    use mandi_services::core::bikri::PukhtaSide;
    let w = support::world();
    one_time_buyer_owes(&w, ("شاہد", "Shahid"), "0300-0000000", "2", "1000");
    let key = mandi_services::core::text::person_key("شاہد", "0300-0000000");
    let base = new_account_form(&w, w.akram, AccountType::Buyer, ("شاہد", "Shahid"));
    let accounts_before = scalar(&w, "SELECT COUNT(*) FROM accounts");
    let entries_before = scalar(&w, "SELECT COUNT(*) FROM journal_entries");
    let mut dr = base.clone();
    dr.opening_dr = "2,000".into();
    let mut cr = base.clone();
    cr.opening_cr = "2,000".into();
    let mut advance = base;
    advance.opening_is_advance = true;
    for (form, field) in [
        (dr, "account_input.opening_dr"),
        (cr, "account_input.opening_cr"),
        (advance, "account_input.opening_is_advance"),
    ] {
        let error = vouchers::make_regular(
            &w.env.ctx,
            &w.owner,
            &MakeRegularInput {
                person_key: key.clone(),
                side: PukhtaSide::Buyer,
                account_input: form,
            },
        )
        .unwrap_err();
        match error {
            ServiceError::Rejected(items) => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].code, ErrorCode::ErrFormatInvalid);
                assert_eq!(items[0].field.as_deref(), Some(field));
            }
            other => panic!("{other:?}"),
        }
    }
    // Nothing was written: no account, no entry, and the person still owes his 2,000.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM accounts"), accounts_before);
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM journal_entries"),
        entries_before
    );
    assert_eq!(
        ledger::person_balance(&w.env.ctx, w.pukhta_buyer, &key)
            .unwrap()
            .amount_paisa,
        rupees(2_000)
    );
}

#[test]
fn make_regular_moves_a_credit_for_an_overpaid_buyer_and_for_a_one_time_seller() {
    use mandi_services::core::bikri::{LineChargeMethod, PukhtaInput, PukhtaSide};
    let w = support::world();
    // A one-time buyer owes 2,000, then pays 5,000: the firm holds 3,000 of his.
    one_time_buyer_owes(&w, ("شاہد", "Shahid"), "0300-0000000", "2", "1000");
    let buyer_key = mandi_services::core::text::person_key("شاہد", "0300-0000000");
    let mut pay = receipt(w.pukhta_buyer, "5,000");
    pay.person_key = Some(buyer_key.clone());
    vouchers::save_receipt(&w.env.ctx, &w.owner, &pay).unwrap();
    // A form that sends "0" for its empty openings is accepted like an empty one.
    let mut form = new_account_form(&w, w.akram, AccountType::Buyer, ("شاہد", "Shahid"));
    form.opening_dr = "0".into();
    form.opening_cr = "0".into();
    let made = make_regular_confirmed(&w, &buyer_key, PukhtaSide::Buyer, form);
    assert_eq!(
        (made.moved.side, made.moved.amount_paisa),
        (BalanceSide::Cr, rupees(3_000))
    );
    assert_eq!(balance(&w, made.account.id), (BalanceSide::Cr, 3_000));
    assert_eq!(
        ledger::person_balance(&w.env.ctx, w.pukhta_buyer, &buyer_key)
            .unwrap()
            .side,
        BalanceSide::Zero
    );
    // His 3,000 is a credit of the new account, to meet his next bill (FR-15.5): no open bill.
    let bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: made.account.id,
            person_key: None,
        },
    )
    .unwrap();
    assert!(bills.bills.is_empty());
    assert_eq!(rematch_changes(&w, made.account.id, None), (0, 0));
    assert_eq!(
        rematch_changes(&w, w.pukhta_buyer, Some(&buyer_key)),
        (0, 0)
    );

    // A one-time seller is owed 10,000 (his lot is left open: a typed 0 stops FR-13.10).
    let mut lot = support::bare_lot(&w, "lot-1", w.pukhta_seller, w.banana, "1");
    lot.pukhta = Some(PukhtaInput {
        name_ur: "ناصر".into(),
        name_en: "Nasir".into(),
        phone: "0300-1111111".into(),
    });
    lot.cash_paid = Some("0".into());
    let mut line = support::sale("s-1", w.akram, "1", "10000");
    line.seller_comm = support::typed(LineChargeMethod::Percent, "0");
    line.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
    line.laga = support::typed(LineChargeMethod::PerUnit, "0");
    lot.sales = vec![line];
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: support::draft(vec![lot]),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let seller_key = mandi_services::core::text::person_key("ناصر", "0300-1111111");
    let made = make_regular_confirmed(
        &w,
        &seller_key,
        PukhtaSide::Seller,
        new_account_form(&w, w.rahim, AccountType::Seller, ("ناصر", "Nasir")),
    );
    assert_eq!(
        (made.moved.side, made.moved.amount_paisa),
        (BalanceSide::Cr, rupees(10_000))
    );
    assert_eq!(balance(&w, made.account.id), (BalanceSide::Cr, 10_000));
    assert_eq!(
        ledger::person_balance(&w.env.ctx, w.pukhta_seller, &seller_key)
            .unwrap()
            .side,
        BalanceSide::Zero
    );
    // He is now a normal seller with 10,000 payable, and the person has nothing payable.
    let payable = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: made.account.id,
            person_key: None,
            include_settled: false,
        },
    )
    .unwrap();
    assert_eq!(payable.open_paisa, rupees(10_000));
    let old = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.pukhta_seller,
            person_key: Some(seller_key),
            include_settled: false,
        },
    )
    .unwrap();
    assert_eq!(old.open_paisa, Paisa::ZERO);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn the_global_search_finds_vouchers_by_number() {
    use mandi_services::core::accounts::GlobalHit;
    let w = support::world();
    event1_saved(&w);
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "1,000")).unwrap();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "1,000", "")).unwrap();
    let find = |text: &str| -> Vec<(VoucherType, i64)> {
        mandi_services::search::global(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::accounts::SearchGlobalInput {
                text: text.into(),
                limit: 10,
            },
        )
        .unwrap()
        .into_iter()
        .filter_map(|hit| match hit {
            GlobalHit::Voucher { hit } => Some((hit.voucher_type, hit.number)),
            _ => None,
        })
        .collect()
    };
    // The digits find the voucher with that number in every series.
    assert_eq!(
        find("1"),
        [(VoucherType::Receipt, 1), (VoucherType::Payment, 1)]
    );
    // A letter finds one series, in either language, with or without the dash.
    assert_eq!(find("R-1"), [(VoucherType::Receipt, 1)]);
    assert_eq!(find("p1"), [(VoucherType::Payment, 1)]);
    assert_eq!(find("ا-1"), [(VoucherType::Payment, 1)]);
    assert_eq!(find("و - 1"), [(VoucherType::Receipt, 1)]);
    assert!(find("A-1").is_empty());
    assert!(find("R-9").is_empty());
    assert!(find("x-1").is_empty());
}

#[test]
fn plan_17e_the_receipt_slip_has_the_shared_header_the_tag_and_the_short_code() {
    use mandi_services::core::auth::Language;
    use mandi_services::core::firm::FirmContact;
    use mandi_services::settings::{self, FirmPrintInfo};
    let w = support::world();
    event1_saved(&w);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    let slip = |language| {
        mandi_reports::print::voucher_print_data(
            w.env.ctx.db(),
            &mandi_reports::print::VoucherPrintDataInput {
                voucher_id: saved.voucher_id,
                language,
            },
        )
        .unwrap()
    };
    let before = slip(Language::En);
    assert!(before.header.contacts.is_empty());
    assert_eq!(before.header.trade_mark, None);
    assert_eq!(before.pages[0].party.short_code.as_deref(), Some("0"));
    assert_eq!(before.pages[0].party.tag, None);
    // The default Urdu slip has only plain digits (Umar, 26-10-2026).
    let plain = serde_json::to_string(&slip(Language::Ur)).unwrap();
    assert!(
        !plain
            .chars()
            .any(|c| ('\u{0660}'..='\u{0669}').contains(&c)
                || ('\u{06F0}'..='\u{06F9}').contains(&c)),
        "{plain}"
    );
    // The buyer's tag shows on the slip with the party.
    raw(&w.dir)
        .execute("UPDATE accounts SET tag = 'K' WHERE id = ?1", [w.akram])
        .unwrap();
    assert_eq!(slip(Language::En).pages[0].party.tag.as_deref(), Some("K"));
    let firm = settings::get(&w.env.ctx).unwrap().firm;
    let info = FirmPrintInfo {
        firm_contacts: vec![FirmContact {
            name_ur: "علی".into(),
            name_en: "Ali".into(),
            phone: "0300-4058860".into(),
        }],
        trade_mark: "RH".into(),
    };
    settings::set_firm_with_print_info(&w.env.ctx, &w.owner, &firm, &info, &[]).unwrap();
    for language in [Language::Ur, Language::En] {
        let after = slip(language);
        assert_eq!(after.header.contacts.len(), 1);
        assert_eq!(after.header.contacts[0].phone, "0300-4058860");
        assert_eq!(after.header.trade_mark.as_deref(), Some("RH"));
        // The amount and its words are what they were.
        assert_eq!(after.pages[0].amounts[0].amount, "10,000");
    }
    assert_eq!(
        slip(Language::Ur).header.contacts[0].name,
        "علی",
        "the contact follows the print language"
    );
}

#[test]
fn the_slip_has_the_party_the_amount_and_the_words_in_both_languages() {
    use mandi_services::core::auth::Language;
    let w = support::world();
    event1_saved(&w);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    let slip = |language| {
        mandi_reports::print::voucher_print_data(
            w.env.ctx.db(),
            &mandi_reports::print::VoucherPrintDataInput {
                voucher_id: saved.voucher_id,
                language,
            },
        )
        .unwrap()
    };
    let ur = slip(Language::Ur);
    let en = slip(Language::En);
    assert_eq!(ur.header.title_id, "print-receipt-slip-title");
    assert_eq!(
        (ur.dir, en.dir, ur.paper, ur.number.as_str()),
        (
            mandi_reports::print::TextDir::Rtl,
            mandi_reports::print::TextDir::Ltr,
            mandi_services::core::printing::Paper::A5,
            "1"
        )
    );
    let page = &en.pages[0];
    assert_eq!(
        (
            page.party.role_id.as_str(),
            page.party.name.as_str(),
            page.party.code.as_deref()
        ),
        ("print-party-buyer", "Akram Toll Tax", Some("0020"))
    );
    assert_eq!(page.amounts[0].amount, "10,000");
    assert_eq!(page.amounts[0].label_id, "print-slip-amount-received");
    assert_eq!(
        page.words.as_ref().map(|w| w.text.as_str()),
        Some("Ten thousand rupees only")
    );
    assert_eq!(
        ur.pages[0].words.as_ref().map(|w| w.text.as_str()),
        Some("دس ہزار روپے")
    );
    assert_eq!(ur.pages[0].party.name, "اکرم ٹول ٹیکس");
    // All money is روکڑ (PRD 15.1, v1.6): the slip has no line about how it was paid.
    assert_eq!(page.payment_id, None);
    assert!(!en.cancelled);
    // A payment and an advance have their own titles; a cancelled voucher is stamped.
    let paid =
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "1,000", "")).unwrap();
    let paid_slip = mandi_reports::print::voucher_print_data(
        w.env.ctx.db(),
        &mandi_reports::print::VoucherPrintDataInput {
            voucher_id: paid.voucher_id,
            language: Language::En,
        },
    )
    .unwrap();
    assert_eq!(paid_slip.header.title_id, "print-payment-slip-title");
    assert_eq!(
        paid_slip.pages[0].amounts[0].label_id,
        "print-slip-amount-paid"
    );
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: row.id,
            version: row.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert!(slip(Language::En).cancelled);
    let missing = mandi_reports::print::voucher_print_data(
        w.env.ctx.db(),
        &mandi_reports::print::VoucherPrintDataInput {
            voucher_id: 999,
            language: Language::En,
        },
    )
    .unwrap_err();
    assert!(matches!(missing, mandi_reports::ReportsError::NotFound));
}

#[test]
fn a_payment_to_an_account_the_picker_does_not_offer_is_refused() {
    let w = support::world();
    // Cash, the نقدی منڈی, an income account and the پختہ کھاتہ without a person.
    for (code, expect) in [
        ("01010001", ErrorCode::ErrFormatInvalid),
        ("03040000", ErrorCode::ErrFormatInvalid),
        ("04010001", ErrorCode::ErrFormatInvalid),
        ("0030", ErrorCode::ErrPersonRequired),
    ] {
        let id = support::id_of(&w.env, code);
        let refused =
            vouchers::save_payment(&w.env.ctx, &w.owner, &payment(id, "100", "x")).unwrap_err();
        assert_eq!(refused.code(), expect, "{code}");
    }
    // A receipt goes only to buyers and sellers.
    let refused = vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &receipt(support::id_of(&w.env, "01010001"), "100"),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrNotABuyer);
    // Freight payable (کرایہ کھاتہ) is paid by an ادائیگی (PRD FR-14.2).
    event1_saved(&w);
    let freight = support::id_of(&w.env, "02010002");
    assert_eq!(balance(&w, freight), (BalanceSide::Cr, 3_000));
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(freight, "3,000", "")).unwrap();
    assert_eq!(balance(&w, freight).0, BalanceSide::Zero);
    // An inactive account is refused.
    mandi_services::accounts::set_active(&w.env.ctx, &w.owner, w.arshad, false).unwrap();
    let refused =
        vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.arshad, "100")).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrAccountInactive);
}

#[test]
fn a_one_time_seller_is_paid_by_his_person_and_the_register_agrees_in_total() {
    use mandi_services::core::bikri::{LineChargeMethod, PukhtaInput};
    let w = support::world();
    let person = || PukhtaInput {
        name_ur: "ناصر".into(),
        name_en: "Nasir".into(),
        phone: "0300-1111111".into(),
    };
    // Two lots of the same one-time seller: 10,000 left open (a typed 0 stops the default of
    // PRD FR-13.10, which pays his whole net on the spot), then 8,000 paid on the spot. No
    // commission, so the nets are 10,000 and 8,000.
    let mut lots = Vec::new();
    for (key, rate, cash) in [
        ("lot-1", "10000", Some("0")),
        ("lot-2", "8000", Some("8,000")),
    ] {
        let mut lot = support::bare_lot(&w, key, w.pukhta_seller, w.banana, "1");
        lot.pukhta = Some(person());
        lot.cash_paid = cash.map(str::to_owned);
        let mut line = support::sale("s-1", w.akram, "1", rate);
        line.seller_comm = support::typed(LineChargeMethod::Percent, "0");
        line.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
        line.laga = support::typed(LineChargeMethod::PerUnit, "0");
        lot.sales = vec![line];
        lots.push(lot);
    }
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: support::draft(lots),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let key = mandi_services::core::text::person_key("ناصر", "0300-1111111");
    let payables = |include_settled| {
        vouchers::open_payables(
            &w.env.ctx,
            &OpenPayablesInput {
                seller_id: w.pukhta_seller,
                person_key: Some(key.clone()),
                include_settled,
            },
        )
        .unwrap()
    };
    // Payments settle the oldest credit first: lot 1 (10,000) is paid 8,000 by the second lot's
    // نقد, so 2,000 of lot 1 and all of lot 2's 8,000 are open.
    let p = payables(false);
    assert_eq!(p.open_paisa, rupees(10_000));
    assert_eq!(
        p.postings
            .iter()
            .map(|x| x.still_open_paisa.rupees())
            .collect::<Vec<_>>(),
        [2_000, 8_000]
    );
    // The register nets each lot inside itself: lot 1 open 10,000, lot 2 paid. The totals agree.
    let register = bikri::register(&w.env.ctx, &Default::default()).unwrap();
    let register_open: i64 = register
        .rows
        .iter()
        .filter(|row| row.person_key == key)
        .map(|row| row.open.rupees())
        .sum();
    assert_eq!(register_open, 10_000);
    // Paying him needs his person key; 10,000 clears him in both views.
    let mut pay = payment(w.pukhta_seller, "10,000", "");
    assert_eq!(
        vouchers::save_payment(&w.env.ctx, &w.owner, &pay)
            .unwrap_err()
            .code(),
        ErrorCode::ErrPersonRequired
    );
    pay.person_key = Some(key.clone());
    pay.confirm_cash_below_zero = true;
    let saved = vouchers::save_payment(&w.env.ctx, &w.owner, &pay).unwrap();
    assert_eq!(saved.new_balance.side, BalanceSide::Zero);
    assert_eq!(payables(false).open_paisa, Paisa::ZERO);
    let register = bikri::register(&w.env.ctx, &Default::default()).unwrap();
    assert_eq!(
        register
            .rows
            .iter()
            .filter(|row| row.person_key == key)
            .map(|row| row.open.rupees())
            .sum::<i64>(),
        0
    );
    // A return by the one-time seller is limited by what he owes: nothing.
    let mut back = receipt(w.pukhta_seller, "100");
    back.person_key = Some(key);
    assert_eq!(
        vouchers::save_receipt(&w.env.ctx, &w.owner, &back)
            .unwrap_err()
            .code(),
        ErrorCode::ErrReturnExceedsOpen
    );
}
