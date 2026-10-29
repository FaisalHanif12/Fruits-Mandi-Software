//! Plan 17 T2b: the rules of partner capital, الگ کھاتہ and جنرل, one by one (PRD 16.4, 16.6, 17.4,
//! FR-6.2, FR-6.5, FR-6.7, FR-15.4, FR-17.8, FR-17.9, FR-17.10). The worked-day numbers are in
//! `golden_day`; this file checks each rule on its own.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::accounts::{BalanceSide, OpeningLineInput, OpeningsInput};
use mandi_services::core::capital::{CapitalHistoryInput, CapitalInput, CapitalRowKind};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::expenses::{PersonalExpenseInput, PersonalReturnInput};
use mandi_services::core::general::{GeneralInput, GeneralLineInput, GeneralPreviewInput};
use mandi_services::core::khata::{
    KhataCreateInput, KhataInInput, KhataOutInput, KhataRowKind, KhataStatementInput,
};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{
    OpenBillsInput, VoucherCancelInput, VoucherEditInput, VoucherEntry, VoucherPreviewInput,
    VoucherType,
};
use mandi_services::{ServiceError, accounts, khata, ledger, masters, partners, vouchers};
use support::World;

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn count(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// Cash in hand in rupees, below zero as a minus.
fn cash(w: &World) -> i64 {
    let balance = ledger::account_balance(&w.env.ctx, support::id_of(&w.env, "01010001")).unwrap();
    match balance.side {
        BalanceSide::Cr => -balance.amount_paisa.rupees(),
        _ => balance.amount_paisa.rupees(),
    }
}

/// The fields and codes of a refusal, in order.
fn refusal(error: &ServiceError) -> Vec<(ErrorCode, String)> {
    match error {
        ServiceError::Rejected(items) => items
            .iter()
            .map(|i| (i.code, i.field.clone().unwrap_or_default()))
            .collect(),
        other => panic!("{other:?}"),
    }
}

fn confirming<T>(mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>) -> T {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match attempt(&words) {
            Ok(done) => return done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

fn make_khata(w: &World, ur: &str, en: &str) -> i64 {
    confirming(|words| {
        khata::create(
            &w.env.ctx,
            &w.owner,
            &KhataCreateInput {
                name_ur: ur.into(),
                name_en: en.into(),
                note: Some("family".into()),
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .account
    .id
}

/// An expense head, with the id of its account.
fn head_account(w: &World, ur: &str, en: &str) -> i64 {
    confirming(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::ExpenseHead(ExpenseHeadInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: ur.into(),
                name_en: en.into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
    masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .iter()
        .find(|r| r.head.name_en == en)
        .unwrap()
        .account_id
}

fn capital(partner: i64, amount: &str, date: &str) -> CapitalInput {
    CapitalInput {
        date: date.into(),
        partner_id: Some(partner),
        amount: amount.into(),
        narration: None,
        confirm_cash_below_zero: false,
    }
}

fn k_out(khata: i64, amount: &str) -> KhataOutInput {
    KhataOutInput {
        date: "10-10-2026".into(),
        khata_account_id: Some(khata),
        amount: amount.into(),
        reason: "خرچ".into(),
        narration: None,
        confirm_cash_below_zero: false,
    }
}

fn k_in(khata: i64, amount: &str) -> KhataInInput {
    KhataInInput {
        date: "10-10-2026".into(),
        khata_account_id: Some(khata),
        amount: amount.into(),
        reason: "واپس".into(),
        narration: None,
        confirm_cash_below_zero: false,
    }
}

fn gl(account: i64, dr: &str, cr: &str) -> GeneralLineInput {
    GeneralLineInput {
        account_id: Some(account),
        person_key: None,
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    }
}

fn general(lines: Vec<GeneralLineInput>) -> GeneralInput {
    GeneralInput {
        date: "10-10-2026".into(),
        narration: None,
        lines,
        confirm_cash_below_zero: false,
    }
}

fn overview(w: &World, partner: i64) -> mandi_services::core::partners::PartnerOverviewRow {
    partners::overview(&w.env.ctx)
        .unwrap()
        .into_iter()
        .find(|r| r.partner_id == partner)
        .unwrap()
}

// ---- Capital ----

#[test]
fn a_capital_entry_with_mistakes_returns_all_of_them_at_their_fields() {
    let w = support::world();
    let mut input = capital(w.partners[0], "100.50", "11-10-2026");
    input.partner_id = None;
    let error = vouchers::save_capital_in(&w.env.ctx, &w.owner, &input).unwrap_err();
    let fields = refusal(&error);
    assert!(fields.contains(&(ErrorCode::ErrPartnerRequired, "partner_id".into())));
    assert!(fields.contains(&(ErrorCode::ErrDecimalsNotAllowed, "amount".into())));
    assert!(fields.contains(&(ErrorCode::ErrDateInFuture, "date".into())));
    let unknown =
        vouchers::save_capital_out(&w.env.ctx, &w.owner, &capital(9_999, "100", "10-10-2026"))
            .unwrap_err();
    assert_eq!(
        refusal(&unknown),
        vec![(ErrorCode::ErrNotFound, "partner_id".into())]
    );
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 0);
}

#[test]
fn taking_back_more_than_the_balance_warns_in_the_preview_and_still_saves() {
    let w = support::world();
    let b = w.partners[1];
    vouchers::save_capital_in(&w.env.ctx, &w.owner, &capital(b, "50,000", "10-10-2026")).unwrap();
    let preview = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::CapitalOut(capital(b, "60,000", "10-10-2026")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert_eq!(preview.warnings.len(), 1);
    let warning = &preview.warnings[0];
    assert_eq!(
        (warning.code, warning.field.as_str()),
        (ErrorCode::ErrCapitalExceedsBalance, "amount")
    );
    assert_eq!(
        warning.values.get("balance_paisa"),
        Some(&ErrorValue::Number(rupees(50_000).as_paisa()))
    );
    assert_eq!(
        warning.values.get("over_paisa"),
        Some(&ErrorValue::Number(rupees(10_000).as_paisa()))
    );
    // An amount that fits has no warning.
    let fits = vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry: VoucherEntry::CapitalOut(capital(b, "10,000", "10-10-2026")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(fits.warnings.is_empty());
    // It saves: cash 50,000 + 50,000 - 60,000.
    vouchers::save_capital_out(&w.env.ctx, &w.owner, &capital(b, "60,000", "10-10-2026")).unwrap();
    assert_eq!(cash(&w), 40_000);
    let history = partners::capital_history(
        &w.env.ctx,
        &CapitalHistoryInput {
            group_by_month: false,
            sort: None,
            partner_id: b,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(history.balance_paisa.rupees(), -10_000);
    assert_eq!(
        history.rows.iter().map(|r| r.kind).collect::<Vec<_>>(),
        vec![CapitalRowKind::Invested, CapitalRowKind::TakenBack]
    );
    assert_eq!(
        (
            history.totals.invested_paisa.rupees(),
            history.totals.taken_back_paisa.rupees()
        ),
        (50_000, 60_000)
    );
}

#[test]
fn capital_taken_back_asks_before_cash_goes_below_zero_and_capital_in_never_asks() {
    let w = support::world_with(1_000, true);
    let a = w.partners[0];
    let refused =
        vouchers::save_capital_out(&w.env.ctx, &w.owner, &capital(a, "5,000", "10-10-2026"))
            .unwrap_err();
    assert_eq!(
        code_and_value(&refused, "cash_after_paisa"),
        (
            ErrorCode::ErrCashBelowZeroNotConfirmed,
            Some(rupees(-4_000).as_paisa())
        )
    );
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 0);
    let mut confirmed = capital(a, "5,000", "10-10-2026");
    confirmed.confirm_cash_below_zero = true;
    vouchers::save_capital_out(&w.env.ctx, &w.owner, &confirmed).unwrap();
    assert_eq!(cash(&w), -4_000);
    // Cash is below zero, but money coming in does not ask.
    vouchers::save_capital_in(&w.env.ctx, &w.owner, &capital(a, "100", "10-10-2026")).unwrap();
    // Capital is never split: no share row at all.
    assert_eq!(count(&w, "SELECT COUNT(*) FROM partner_share_rows"), 0);
}

#[test]
fn a_capital_entry_is_edited_and_cancelled_by_reversal() {
    let w = support::world();
    let b = w.partners[1];
    let saved =
        vouchers::save_capital_in(&w.env.ctx, &w.owner, &capital(b, "50,000", "10-10-2026"))
            .unwrap();
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    let edited = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: saved.voucher_id,
            version: row.version,
            entry: VoucherEntry::CapitalIn(capital(b, "30,000", "10-10-2026")),
            reason: "typing mistake".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(
        (edited.number, edited.voucher_type),
        (1, VoucherType::CapitalIn)
    );
    assert_eq!(cash(&w), 80_000);
    // The type cannot change on an edit.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    let refused = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: saved.voucher_id,
            version: row.version,
            entry: VoucherEntry::CapitalOut(capital(b, "30,000", "10-10-2026")),
            reason: String::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrFormatInvalid);
    // The history shows the live entry only.
    let history = partners::capital_history(
        &w.env.ctx,
        &CapitalHistoryInput {
            group_by_month: false,
            sort: None,
            partner_id: b,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(history.rows.len(), 1);
    assert_eq!(history.rows[0].amount_paisa.rupees(), 30_000);
    // Cancel: cash comes back to 50,000, the history is empty, and the number stays used.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: saved.voucher_id,
            version: row.version,
            reason: "wrong partner".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(cash(&w), 50_000);
    let history = partners::capital_history(
        &w.env.ctx,
        &CapitalHistoryInput {
            group_by_month: false,
            sort: None,
            partner_id: b,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!((history.rows.len(), history.balance_paisa.rupees()), (0, 0));
    let next = vouchers::save_capital_in(&w.env.ctx, &w.owner, &capital(b, "1,000", "10-10-2026"))
        .unwrap();
    assert_eq!(next.number, 2);
}

#[test]
fn the_capital_history_pages_and_the_totals_cover_every_row() {
    let w = support::world();
    let a = w.partners[0];
    for amount in ["1,000", "2,000", "3,000"] {
        vouchers::save_capital_in(&w.env.ctx, &w.owner, &capital(a, amount, "10-10-2026")).unwrap();
    }
    let page = |offset, limit| {
        partners::capital_history(
            &w.env.ctx,
            &CapitalHistoryInput {
                group_by_month: false,
                sort: None,
                partner_id: a,
                offset,
                limit,
            },
        )
        .unwrap()
    };
    let (first, second) = (page(0, 2), page(2, 2));
    assert_eq!(
        (first.rows.len(), second.rows.len(), first.total_count),
        (2, 1, 3)
    );
    assert_eq!(second.rows[0].running_paisa.rupees(), 6_000);
    assert_eq!(first.totals, second.totals);
    assert_eq!(first.totals.invested_paisa.rupees(), 6_000);
    assert_eq!(first.balance_paisa.rupees(), 6_000);
    let unknown = partners::capital_history(
        &w.env.ctx,
        &CapitalHistoryInput {
            group_by_month: false,
            sort: None,
            partner_id: 777,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap_err();
    assert_eq!(
        refusal(&unknown),
        vec![(ErrorCode::ErrNotFound, "partner_id".into())]
    );
}

// ---- Overview ----

#[test]
fn the_overview_has_returned_net_and_total_and_a_return_adds_back() {
    let w = support::world();
    let a = w.partners[0];
    let personal = PersonalExpenseInput {
        date: "10-10-2026".into(),
        partner_id: Some(a),
        amount: "2,000".into(),
        reason: "گھر".into(),
        return_by: None,
        taken_by: None,
        narration: None,
        confirm_cash_below_zero: false,
    };
    vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal).unwrap();
    vouchers::save_personal_return(
        &w.env.ctx,
        &w.owner,
        &PersonalReturnInput {
            date: "10-10-2026".into(),
            partner_id: Some(a),
            amount: "500".into(),
            narration: None,
        },
    )
    .unwrap();
    let row = overview(&w, a);
    // C20: خالص = 0 profit - (2,000 taken - 500 returned).
    assert_eq!(
        (
            row.month_taken_paisa.rupees(),
            row.month_returned_paisa.rupees(),
            row.month_net_paisa.rupees(),
            row.total_paisa.rupees()
        ),
        (2_000, 500, -1_500, -1_500)
    );
    // A cancelled return is taken off the returned column.
    let list = vouchers::list(
        &w.env.ctx,
        &mandi_services::core::vouchers::VouchersListInput {
            sort: None,
            group_by_month: false,
            search: None,
            show_finished: true,
            voucher_type: Some(VoucherType::PersonalReturn),
            from: String::new(),
            to: String::new(),
            account_id: None,
            person_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let voucher = &list.rows[0];
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: voucher.id,
            version: voucher.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let row = overview(&w, a);
    assert_eq!(
        (row.month_returned_paisa.rupees(), row.total_paisa.rupees()),
        (0, -2_000)
    );
}

// ---- الگ کھاتہ ----

#[test]
fn a_khata_entry_with_mistakes_is_refused_at_its_fields() {
    let w = support::world();
    let family = make_khata(&w, "خاندانی کھاتہ", "Family Khata");
    // No khata, no reason, a decimal amount.
    let mut input = k_out(family, "10.5");
    input.khata_account_id = None;
    input.reason = String::new();
    let fields = refusal(&vouchers::save_khata_out(&w.env.ctx, &w.owner, &input).unwrap_err());
    assert!(fields.contains(&(ErrorCode::ErrKhataRequired, "khata_account_id".into())));
    assert!(fields.contains(&(ErrorCode::ErrReasonRequired, "reason".into())));
    assert!(fields.contains(&(ErrorCode::ErrDecimalsNotAllowed, "amount".into())));
    // A buyer is not a khata.
    let error = vouchers::save_khata_in(&w.env.ctx, &w.owner, &k_in(w.akram, "10")).unwrap_err();
    assert_eq!(
        refusal(&error),
        vec![(ErrorCode::ErrFormatInvalid, "khata_account_id".into())]
    );
    let error = vouchers::save_khata_in(&w.env.ctx, &w.owner, &k_in(99_999, "10")).unwrap_err();
    assert_eq!(
        refusal(&error),
        vec![(ErrorCode::ErrNotFound, "khata_account_id".into())]
    );
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 0);
}

#[test]
fn money_paid_out_for_a_khata_asks_about_cash_and_money_received_does_not() {
    let w = support::world_with(1_000, true);
    let family = make_khata(&w, "خاندانی کھاتہ", "Family Khata");
    let refused =
        vouchers::save_khata_out(&w.env.ctx, &w.owner, &k_out(family, "5,000")).unwrap_err();
    assert_eq!(
        code_and_value(&refused, "cash_after_paisa"),
        (
            ErrorCode::ErrCashBelowZeroNotConfirmed,
            Some(rupees(-4_000).as_paisa())
        )
    );
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 0);
    let mut confirmed = k_out(family, "5,000");
    confirmed.confirm_cash_below_zero = true;
    vouchers::save_khata_out(&w.env.ctx, &w.owner, &confirmed).unwrap();
    vouchers::save_khata_in(&w.env.ctx, &w.owner, &k_in(family, "100")).unwrap();
    assert_eq!(cash(&w), -3_900);
    // A khata is never income or expense, and never split.
    assert_eq!(count(&w, "SELECT COUNT(*) FROM partner_share_rows"), 0);
    let balance = ledger::account_balance(&w.env.ctx, family).unwrap();
    assert_eq!(
        (balance.side, balance.amount_paisa),
        (BalanceSide::Dr, rupees(4_900))
    );
}

#[test]
fn a_khata_statement_has_a_range_an_opening_pages_and_edits_leave_no_trace() {
    let w = support::world();
    let family = make_khata(&w, "خاندانی کھاتہ", "Family Khata");
    let out = vouchers::save_khata_out(&w.env.ctx, &w.owner, &k_out(family, "10,000")).unwrap();
    set_today(&w.env, 2026, 10, 11);
    let mut later = k_in(family, "4,000");
    later.date = "11-10-2026".into();
    vouchers::save_khata_in(&w.env.ctx, &w.owner, &later).unwrap();
    // Edit the first entry to 8,000: the statement shows the live version only.
    let row = vouchers::get(&w.env.ctx, out.voucher_id).unwrap();
    vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: out.voucher_id,
            version: row.version,
            entry: VoucherEntry::KhataOut(k_out(family, "8,000")),
            reason: "typing".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let statement = |from: Option<&str>, to: Option<&str>, offset, limit| {
        khata::statement(
            &w.env.ctx,
            &KhataStatementInput {
                group_by_month: false,
                sort: None,
                khata_account_id: family,
                from: from.map(str::to_owned),
                to: to.map(str::to_owned),
                offset,
                limit,
            },
        )
        .unwrap()
    };
    let all = statement(None, None, 0, 0);
    assert_eq!(
        all.rows
            .iter()
            .map(|r| (r.kind, r.amount_paisa.rupees(), r.running_paisa.rupees()))
            .collect::<Vec<_>>(),
        vec![
            (KhataRowKind::PaidOut, -8_000, -8_000),
            (KhataRowKind::Received, 4_000, -4_000)
        ]
    );
    // From 11-10-26: it opens at -8,000 and holds the one row.
    let ranged = statement(Some("11-10-2026"), None, 0, 0);
    assert_eq!(
        (
            ranged.opening_paisa.rupees(),
            ranged.rows.len(),
            ranged.closing_paisa.rupees(),
            ranged.total_count
        ),
        (-8_000, 1, -4_000, 1)
    );
    // To 10-10-26: the second row is out.
    let to = statement(None, Some("10-10-2026"), 0, 0);
    assert_eq!((to.rows.len(), to.closing_paisa.rupees()), (1, -8_000));
    // Paging by one row: the totals stay those of the whole range.
    let (first, second) = (statement(None, None, 0, 1), statement(None, None, 1, 1));
    assert_eq!(
        (first.rows.len(), second.rows.len(), first.total_count),
        (1, 1, 2)
    );
    assert_eq!(first.totals, second.totals);
    assert_eq!(second.rows[0].running_paisa.rupees(), -4_000);
    // The sums of the statement are the sums of the posted lines.
    assert_eq!(
        (
            all.totals.paid_out_paisa.rupees(),
            all.totals.received_paisa.rupees()
        ),
        (8_000, 4_000)
    );
    // Cancel the received entry: the statement and the balance agree.
    let received = vouchers::list(
        &w.env.ctx,
        &mandi_services::core::vouchers::VouchersListInput {
            sort: None,
            group_by_month: false,
            search: None,
            show_finished: true,
            voucher_type: Some(VoucherType::KhataIn),
            from: String::new(),
            to: String::new(),
            account_id: None,
            person_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
    .rows
    .remove(0);
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: received.id,
            version: received.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let after = statement(None, None, 0, 0);
    assert_eq!(after.closing_paisa.rupees(), -8_000);
    assert_eq!(after.khata.balance_paisa.rupees(), -8_000);
    let unknown = khata::statement(
        &w.env.ctx,
        &KhataStatementInput {
            group_by_month: false,
            sort: None,
            khata_account_id: w.akram,
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap_err();
    assert_eq!(
        refusal(&unknown),
        vec![(ErrorCode::ErrNotFound, "khata_account_id".into())]
    );
}

#[test]
fn a_khata_needs_both_names_in_their_scripts_and_keeps_its_note() {
    let w = support::world();
    let error = khata::create(
        &w.env.ctx,
        &w.owner,
        &KhataCreateInput {
            name_ur: "Family".into(),
            name_en: "خاندان".into(),
            note: None,
            confirmed_rules_words: Vec::new(),
        },
    )
    .unwrap_err();
    let fields = refusal(&error);
    assert!(fields.contains(&(ErrorCode::ErrNameUrScript, "name_ur".into())));
    assert!(fields.contains(&(ErrorCode::ErrNameEnScript, "name_en".into())));
    let family = make_khata(&w, "خاندانی کھاتہ", "Family Khata");
    let statement = khata::statement(
        &w.env.ctx,
        &KhataStatementInput {
            group_by_month: false,
            sort: None,
            khata_account_id: family,
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(statement.khata.note, "family");
    assert_eq!(statement.khata.account.code, "03050001");
    assert_eq!(statement.khata.balance_paisa, Paisa::ZERO);
    let second = make_khata(&w, "دوسرا کھاتہ", "Second Khata");
    assert_ne!(second, family);
}

// ---- جنرل ----

#[test]
fn a_general_with_too_few_lines_or_unbalanced_lines_writes_nothing() {
    let w = support::world();
    let cash_id = support::id_of(&w.env, "01010001");
    let before = (
        count(&w, "SELECT COUNT(*) FROM journal_entries"),
        count(&w, "SELECT COUNT(*) FROM vouchers"),
    );
    let one = vouchers::general_save(&w.env.ctx, &w.owner, &general(vec![gl(cash_id, "100", "")]))
        .unwrap_err();
    assert_eq!(
        refusal(&one),
        vec![(ErrorCode::ErrGeneralTooFewLines, "lines".into())]
    );
    let unbalanced = vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &general(vec![gl(w.akram, "1,000", ""), gl(w.rahim, "", "900")]),
    )
    .unwrap_err();
    assert_eq!(
        code_and_value(&unbalanced, "dr_paisa"),
        (
            ErrorCode::ErrGeneralNotBalanced,
            Some(rupees(1_000).as_paisa())
        )
    );
    assert_eq!(
        code_and_value(&unbalanced, "cr_paisa").1,
        Some(rupees(900).as_paisa())
    );
    assert_eq!(
        refusal(&unbalanced),
        vec![(ErrorCode::ErrGeneralNotBalanced, "lines".into())]
    );
    assert_eq!(
        (
            count(&w, "SELECT COUNT(*) FROM journal_entries"),
            count(&w, "SELECT COUNT(*) FROM vouchers")
        ),
        before
    );
}

#[test]
fn a_general_line_with_mistakes_is_refused_at_its_own_path() {
    let w = support::world();
    let own = support::id_of(&w.env, "030119998");
    let naqdi = support::id_of(&w.env, "03040000");
    let pukhta = support::id_of(&w.env, "0030");
    let lines = vec![
        gl(own, "100", ""),
        gl(naqdi, "", "100"),
        GeneralLineInput {
            account_id: None,
            ..gl(w.akram, "100", "")
        },
        gl(w.akram, "100", "100"),
        gl(w.akram, "", ""),
        gl(pukhta, "100", ""),
        GeneralLineInput {
            person_key: Some("someone".into()),
            ..gl(w.rahim, "", "100")
        },
        gl(w.rahim, "10.5", ""),
    ];
    let error = vouchers::general_save(&w.env.ctx, &w.owner, &general(lines)).unwrap_err();
    let fields = refusal(&error);
    for expected in [
        (
            ErrorCode::ErrOwnGoodsAccountRestricted,
            "lines[0].account_id",
        ),
        (ErrorCode::ErrFormatInvalid, "lines[1].account_id"),
        (ErrorCode::ErrRequired, "lines[2].account_id"),
        (ErrorCode::ErrFormatInvalid, "lines[3].cr"),
        (ErrorCode::ErrRequired, "lines[4].dr"),
        (ErrorCode::ErrPersonRequired, "lines[5].person_key"),
        (ErrorCode::ErrFormatInvalid, "lines[6].person_key"),
        (ErrorCode::ErrDecimalsNotAllowed, "lines[7].dr"),
    ] {
        assert!(
            fields.contains(&(expected.0, expected.1.to_owned())),
            "{expected:?} in {fields:?}"
        );
    }
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 0);
}

#[test]
fn the_general_preview_gives_totals_balances_the_split_and_the_cash_note() {
    let w = support::world();
    let electricity = head_account(&w, "بجلی", "Electricity");
    let cash_id = support::id_of(&w.env, "01010001");
    // E2E-20 as a جنرل: a shop expense of 3,384 paid from cash.
    let preview = vouchers::general_preview(
        &w.env.ctx,
        &GeneralPreviewInput {
            date: Some("10-10-2026".into()),
            lines: vec![gl(electricity, "3,384", ""), gl(cash_id, "", "3,384")],
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty() && preview.can_save && preview.is_balanced);
    assert_eq!(
        (
            preview.total_dr_paisa.rupees(),
            preview.total_cr_paisa.rupees(),
            preview.difference_paisa.rupees()
        ),
        (3_384, 3_384, 0)
    );
    let split: Vec<i64> = preview
        .split
        .iter()
        .map(|s| s.amount_paisa.rupees())
        .collect();
    assert_eq!(split, vec![1_354, 1_015, 1_015]);
    assert_eq!(preview.cash.cash_after_paisa.rupees(), 46_616);
    assert!(!preview.cash.needs_confirm);
    // Balance after the entry: cash 50,000 - 3,384 on the credit line.
    let after = preview.lines[1].balance_after.unwrap();
    assert_eq!(
        (after.side, after.amount_paisa.rupees()),
        (BalanceSide::Dr, 46_616)
    );

    // Unbalanced: the difference shows and Save stays off.
    let unbalanced = vouchers::general_preview(
        &w.env.ctx,
        &GeneralPreviewInput {
            date: None,
            lines: vec![gl(w.akram, "1,000", ""), gl(w.rahim, "", "900")],
        },
    )
    .unwrap();
    assert_eq!(unbalanced.difference_paisa.rupees(), 100);
    assert!(!unbalanced.can_save && !unbalanced.is_balanced);
    assert_eq!(
        unbalanced.errors.iter().map(|e| e.code).collect::<Vec<_>>(),
        vec![ErrorCode::ErrGeneralNotBalanced]
    );
    assert!(unbalanced.split.is_empty());
    // A line with a mistake names it, and the entry error waits.
    let bad = vouchers::general_preview(
        &w.env.ctx,
        &GeneralPreviewInput {
            date: None,
            lines: vec![gl(w.akram, "1,000", ""), gl(w.rahim, "", "1,000.5")],
        },
    )
    .unwrap();
    assert!(!bad.can_save);
    assert_eq!(
        bad.lines[1]
            .errors
            .iter()
            .map(|e| (e.code, e.field.as_str()))
            .collect::<Vec<_>>(),
        vec![(ErrorCode::ErrDecimalsNotAllowed, "lines[1].cr")]
    );
    // The preview of a cash credit above cash asks for the confirm.
    let big = vouchers::general_preview(
        &w.env.ctx,
        &GeneralPreviewInput {
            date: Some("10-10-2026".into()),
            lines: vec![gl(w.akram, "60,000", ""), gl(cash_id, "", "60,000")],
        },
    )
    .unwrap();
    assert!(big.cash.needs_confirm);
    assert_eq!(big.cash.cash_after_paisa.rupees(), -10_000);
}

#[test]
fn a_general_on_a_shop_expense_is_split_and_an_income_line_is_shared() {
    let w = support::world();
    let electricity = head_account(&w, "بجلی", "Electricity");
    let cash_id = support::id_of(&w.env, "01010001");
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &general(vec![gl(electricity, "3,384", ""), gl(cash_id, "", "3,384")]),
    )
    .unwrap();
    let shares = |kind: &str| -> Vec<i64> {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare(
                "SELECT amount_paisa FROM partner_share_rows WHERE kind = ?1 ORDER BY partner_id",
            )
            .unwrap();
        statement
            .query_map([kind], |r| Ok(r.get::<_, i64>(0)? / 100))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(shares("expense"), vec![-1_354, -1_015, -1_015]);
    // Income: cash Dr 1,000, commission income Cr 1,000: shared 400 / 300 / 300.
    let income = support::id_of(&w.env, "04010001");
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &general(vec![gl(cash_id, "1,000", ""), gl(income, "", "1,000")]),
    )
    .unwrap();
    assert_eq!(shares("income"), vec![400, 300, 300]);
    // The numbers run in the series G-: 1, 2.
    let numbers: Vec<i64> = {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare("SELECT number FROM vouchers WHERE type = 'general' ORDER BY number")
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(numbers, vec![1, 2]);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

#[test]
fn a_general_on_a_buyer_is_a_bill_and_a_seller_advance_is_never_adjusted() {
    let w = support::world();
    // رحیم has an advance of 15,000 in the worked day; here give him one.
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::AdvanceInput {
            confirm_cash_below_zero: false,
            date: "10-10-2026".into(),
            seller_account_id: w.rahim,
            amount: "15,000".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
    // A correction: اکرم Dr 1,000 (a bill), رحیم Cr 1,000 (a payable posting).
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &general(vec![gl(w.akram, "1,000", ""), gl(w.rahim, "", "1,000")]),
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
    assert_eq!((bills.open_paisa.rupees(), bills.bills.len()), (1_000, 1));
    // PRD 16.4: the 1,000 credit does not adjust his advance, which stays open.
    let advances = mandi_services::assist::advances(&w.env.ctx, w.rahim).unwrap();
    assert_eq!(advances.open_paisa.rupees(), 15_000);
    assert_eq!(
        count(
            &w,
            "SELECT COUNT(*) FROM advance_allocations WHERE is_active = 1"
        ),
        0
    );
    // A credit to اکرم of 400 settles his oldest bill in part: matching ran for the buyer.
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &general(vec![gl(w.rahim, "400", ""), gl(w.akram, "", "400")]),
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
    assert_eq!(bills.open_paisa.rupees(), 600);
    assert_eq!(bills.bills[0].open_paisa.rupees(), 600);
}

#[test]
fn a_general_is_edited_and_cancelled_and_the_buyer_is_matched_again() {
    let w = support::world();
    let first = vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &general(vec![gl(w.akram, "1,000", ""), gl(w.rahim, "", "1,000")]),
    )
    .unwrap();
    assert_eq!(
        (first.voucher_type, first.number),
        (VoucherType::General, 1)
    );
    let credit = vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &general(vec![gl(w.rahim, "400", ""), gl(w.akram, "", "400")]),
    )
    .unwrap();
    let open = |w: &World| {
        vouchers::open_bills(
            &w.env.ctx,
            &OpenBillsInput {
                account_id: w.akram,
                person_key: None,
            },
        )
        .unwrap()
        .open_paisa
        .rupees()
    };
    assert_eq!(open(&w), 600);
    // Edit the bill to 1,500: the credit of 400 is matched again, 1,100 is open.
    let row = vouchers::get(&w.env.ctx, first.voucher_id).unwrap();
    let edited = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: first.voucher_id,
            version: row.version,
            entry: VoucherEntry::General(general(vec![
                gl(w.akram, "1,500", ""),
                gl(w.rahim, "", "1,500"),
            ])),
            reason: "typing".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(edited.number, 1);
    assert_eq!(open(&w), 1_100);
    // Cancel the credit: the whole bill is open again.
    let row = vouchers::get(&w.env.ctx, credit.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: credit.voucher_id,
            version: row.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(open(&w), 1_500);
    assert_eq!(
        vouchers::get(&w.env.ctx, credit.voucher_id).unwrap().status,
        "cancelled"
    );
    // The books balance and the chain holds.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

#[test]
fn a_general_cash_credit_uses_the_cash_confirm() {
    let w = support::world_with(1_000, true);
    let cash_id = support::id_of(&w.env, "01010001");
    let lines = vec![gl(w.akram, "5,000", ""), gl(cash_id, "", "5,000")];
    let refused =
        vouchers::general_save(&w.env.ctx, &w.owner, &general(lines.clone())).unwrap_err();
    assert_eq!(
        code_and_value(&refused, "cash_after_paisa"),
        (
            ErrorCode::ErrCashBelowZeroNotConfirmed,
            Some(rupees(-4_000).as_paisa())
        )
    );
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 0);
    let mut confirmed = general(lines);
    confirmed.confirm_cash_below_zero = true;
    vouchers::general_save(&w.env.ctx, &w.owner, &confirmed).unwrap();
    assert_eq!(cash(&w), -4_000);
}

#[test]
fn nothing_posts_to_the_bank_from_these_screens() {
    let w = support::world();
    let family = make_khata(&w, "خاندانی کھاتہ", "Family Khata");
    vouchers::save_capital_in(
        &w.env.ctx,
        &w.owner,
        &capital(w.partners[0], "100", "10-10-2026"),
    )
    .unwrap();
    vouchers::save_khata_out(&w.env.ctx, &w.owner, &k_out(family, "100")).unwrap();
    // An older screen still sends mode and bank fields: the input has no such field.
    let input: CapitalInput = serde_json::from_value(serde_json::json!({
        "date": "10-10-2026", "partner_id": w.partners[1], "amount": "100",
        "mode": "bank", "bank_account_id": 1
    }))
    .unwrap();
    vouchers::save_capital_in(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(
        count(
            &w,
            "SELECT COUNT(*) FROM journal_lines l JOIN accounts a ON a.id = l.account_id
             WHERE a.code = '01020001'"
        ),
        0
    );
    assert_eq!(
        count(
            &w,
            "SELECT COUNT(*) FROM vouchers WHERE mode <> 'cash' OR bank_account_id IS NOT NULL"
        ),
        0
    );
}

#[test]
fn an_opening_balance_is_refused_on_own_goods_and_on_a_partner_trip_account() {
    // PRD FR-10.9 (v1.6): اپنا مال never holds a balance, and the 0604 account of a partner holds
    // only the money of his gardens, trips and fruit. Nothing is written for a refused opening.
    let w = support::world();
    let own = support::id_of(&w.env, "030119998");
    let line = |id: i64| OpeningLineInput {
        account_id: id,
        version: accounts::get(&w.env.ctx, id).unwrap().version,
        opening_date: "09-10-2026".into(),
        opening_dr: "500".into(),
        opening_cr: String::new(),
        opening_is_advance: false,
    };
    let before = count(&w, "SELECT COUNT(*) FROM journal_entries");
    let preview = accounts::openings_preview(
        &w.env.ctx,
        &OpeningsInput {
            lines: vec![line(own)],
        },
    )
    .unwrap();
    let errors: Vec<(ErrorCode, String)> = preview
        .rows
        .iter()
        .flat_map(|r| r.errors.iter().map(|e| (e.code, e.field.clone())))
        .collect();
    assert!(errors.contains(&(
        ErrorCode::ErrOpeningNotAllowed,
        "lines[0].opening_dr".into()
    )));
    let saved = accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![line(own)],
        },
    );
    assert!(saved.is_err());
    assert_eq!(count(&w, "SELECT COUNT(*) FROM journal_entries"), before);
}
