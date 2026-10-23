//! Plan 17 T2a: the rules of the خرچہ screen and ادھار رقم, one by one (PRD 16.2 to 16.5, FR-6.7,
//! FR-16.2, FR-16.5, FR-16.9, FR-7.62, FR-7.68, FR-15.3). The worked-day numbers are in
//! `golden_day`; this file checks each rule on its own.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::expenses::{
    ExpenseEntry, ExpensePreviewInput, PartnerTakesInput, PersonalExpenseInput,
    PersonalReturnInput, RepeatLastMonthInput, ShopExpenseInput,
};
use mandi_services::core::lent::{
    LentListInput, LentPersonCreateInput, LentReturnMode, MoneyLentInput, MoneyLentReturnInput,
};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput, StaffInput};
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{
    VoucherCancelInput, VoucherEditInput, VoucherEntry, VoucherType,
};
use mandi_services::{ServiceError, ledger, lent, masters, partners, search, vouchers};
use support::World;

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

/// Repeats a save that asks the user to confirm rule-made spellings, confirming what it asks.
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

fn head(w: &World, ur: &str, en: &str) -> i64 {
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
        .id
}

fn person(w: &World, ur: &str, en: &str, staff: Option<i64>) -> i64 {
    confirming(|words| {
        lent::person_create(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateInput {
                name_ur: ur.into(),
                name_en: en.into(),
                phone: None,
                staff_id: staff,
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .account
    .id
}

fn shop(head: i64, amount: &str, date: &str) -> ShopExpenseInput {
    ShopExpenseInput {
        date: date.into(),
        expense_head_id: Some(head),
        amount: amount.into(),
        paid_to: None,
        narration: None,
        confirm_cash_below_zero: false,
    }
}

fn personal(partner: i64, amount: &str, date: &str) -> PersonalExpenseInput {
    PersonalExpenseInput {
        date: date.into(),
        partner_id: Some(partner),
        amount: amount.into(),
        reason: "گھر کا خرچ".into(),
        return_by: None,
        taken_by: None,
        narration: None,
        confirm_cash_below_zero: false,
    }
}

fn lent_in(person: i64, amount: &str, date: &str) -> MoneyLentInput {
    MoneyLentInput {
        date: date.into(),
        person_account_id: Some(person),
        amount: amount.into(),
        reason: "ضرورت".into(),
        return_by: None,
        narration: None,
        confirm_cash_below_zero: false,
    }
}

fn lent_back(person: i64, amount: &str, date: &str, mode: LentReturnMode) -> MoneyLentReturnInput {
    MoneyLentReturnInput {
        date: date.into(),
        person_account_id: Some(person),
        amount: amount.into(),
        mode,
        salary_head_id: None,
        narration: None,
    }
}

/// The worked-day world with partner shares of 40 / 30 / 30 that started on 31-08-2026, so that
/// postings of earlier days can be split (a share is never back-dated, PRD FR-17.2).
fn world_with_old_shares() -> World {
    use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
    let w = support::world_with(50_000, false);
    set_today(&w.env, 2026, 8, 31);
    partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &PartnersSetSharesInput {
            effective_from: "31-08-2026".into(),
            shares: [
                (w.partners[0], "40"),
                (w.partners[1], "30"),
                (w.partners[2], "30"),
            ]
            .iter()
            .map(|&(partner_id, share)| ShareInput {
                partner_id,
                share: share.to_owned(),
            })
            .collect(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 10);
    w
}

fn cash(w: &World) -> i64 {
    ledger::account_balance(&w.env.ctx, support::id_of(&w.env, "01010001"))
        .unwrap()
        .amount_paisa
        .rupees()
}

fn count(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

fn live_shares(w: &World) -> Vec<i64> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows
             WHERE kind = 'expense' GROUP BY partner_id ORDER BY partner_id",
        )
        .unwrap();
    statement
        .query_map([], |r| Ok(r.get::<_, i64>(0)? / 100))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn every_cash_out_type_asks_before_cash_goes_below_zero_and_cash_in_never_asks() {
    let w = support::world_with(1_000, true);
    let electricity = head(&w, "بجلی", "Electricity");
    let aslam = person(&w, "اسلم", "Aslam", None);
    let a = w.partners[0];

    // The preview already shows the amber note data.
    let preview = vouchers::preview_expense(
        &w.env.ctx,
        &ExpensePreviewInput {
            entry: ExpenseEntry::ShopExpense(shop(electricity, "5,000", "10-10-2026")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(preview.cash.needs_confirm);
    assert_eq!(preview.cash.cash_after_paisa, rupees(-4_000));

    let before = count(&w, "SELECT COUNT(*) FROM journal_entries");
    let refusals = [
        vouchers::save_shop_expense(
            &w.env.ctx,
            &w.owner,
            &shop(electricity, "5,000", "10-10-2026"),
        )
        .unwrap_err(),
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "5,000", "10-10-2026"))
            .unwrap_err(),
        vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "5,000", "10-10-2026"))
            .unwrap_err(),
    ];
    for refusal in &refusals {
        assert_eq!(
            code_and_value(refusal, "cash_after_paisa"),
            (
                ErrorCode::ErrCashBelowZeroNotConfirmed,
                Some(rupees(-4_000).as_paisa())
            )
        );
    }
    // Nothing was written by any of them.
    assert_eq!(count(&w, "SELECT COUNT(*) FROM journal_entries"), before);
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 0);
    assert_eq!(cash(&w), 1_000);

    // Confirmed, each saves and cash goes down.
    let mut input = shop(electricity, "5,000", "10-10-2026");
    input.confirm_cash_below_zero = true;
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &input).unwrap();
    let mut input = personal(a, "2,000", "10-10-2026");
    input.confirm_cash_below_zero = true;
    vouchers::save_personal_expense(&w.env.ctx, &w.owner, &input).unwrap();
    let mut input = lent_in(aslam, "3,000", "10-10-2026");
    input.confirm_cash_below_zero = true;
    vouchers::save_money_lent(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 3);
    // Cash 1,000 - 5,000 - 2,000 - 3,000 = -9,000, shown as a debit balance with a minus.
    let balance = ledger::account_balance(&w.env.ctx, support::id_of(&w.env, "01010001")).unwrap();
    assert_eq!(
        (balance.side, balance.amount_paisa),
        (BalanceSide::Cr, rupees(9_000))
    );

    // Cash coming in never asks, although cash is below zero: a return and a cash return.
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
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(aslam, "500", "10-10-2026", LentReturnMode::Cash),
    )
    .unwrap();
}

#[test]
fn nothing_ever_posts_to_the_bank_and_an_old_mode_field_is_ignored() {
    let w = support::world();
    let electricity = head(&w, "بجلی", "Electricity");
    // A screen of an older build still sends mode and bank fields: they are not part of the input,
    // so they do nothing.
    let input: ShopExpenseInput = serde_json::from_value(serde_json::json!({
        "date": "10-10-2026", "expense_head_id": electricity, "amount": "3,000",
        "mode": "bank", "bank_account_id": 1, "cheque_no": "55"
    }))
    .unwrap();
    let saved = vouchers::save_shop_expense(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(saved.voucher_type, VoucherType::ShopExpense);
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
fn a_form_with_several_mistakes_returns_all_of_them_at_their_fields() {
    let w = support::world();
    let electricity = head(&w, "بجلی", "Electricity");
    let fields = |error: &ServiceError| -> Vec<(ErrorCode, String)> {
        match error {
            ServiceError::Rejected(items) => items
                .iter()
                .map(|i| (i.code, i.field.clone().unwrap_or_default()))
                .collect(),
            other => panic!("{other:?}"),
        }
    };
    // No head, a decimal amount, a closed-looking date in the future.
    let mut input = shop(electricity, "100.50", "11-10-2026");
    input.expense_head_id = None;
    let error = vouchers::save_shop_expense(&w.env.ctx, &w.owner, &input).unwrap_err();
    let got = fields(&error);
    assert!(
        got.contains(&(ErrorCode::ErrDecimalsNotAllowed, "amount".into())),
        "{got:?}"
    );
    assert!(
        got.contains(&(ErrorCode::ErrDateInFuture, "date".into())),
        "{got:?}"
    );
    assert!(
        got.contains(&(ErrorCode::ErrExpenseHeadRequired, "expense_head_id".into())),
        "{got:?}"
    );

    // An unknown head, a zero amount.
    let error = vouchers::save_shop_expense(&w.env.ctx, &w.owner, &shop(9_999, "0", "10-10-2026"))
        .unwrap_err();
    let got = fields(&error);
    assert!(
        got.contains(&(ErrorCode::ErrNotFound, "expense_head_id".into())),
        "{got:?}"
    );
    assert!(
        got.contains(&(ErrorCode::ErrMustBeMoreThanZero, "amount".into())),
        "{got:?}"
    );

    // A personal entry with no partner and no reason.
    let mut input = personal(w.partners[0], "100", "10-10-2026");
    input.partner_id = None;
    input.reason = "  ".into();
    let got = fields(&vouchers::save_personal_expense(&w.env.ctx, &w.owner, &input).unwrap_err());
    assert!(
        got.contains(&(ErrorCode::ErrPartnerRequired, "partner_id".into())),
        "{got:?}"
    );
    assert!(
        got.contains(&(ErrorCode::ErrReasonRequired, "reason".into())),
        "{got:?}"
    );

    // Money lent: no person, a buyer instead of a person, no reason.
    let mut input = lent_in(w.akram, "100", "10-10-2026");
    input.reason = String::new();
    let got = fields(&vouchers::save_money_lent(&w.env.ctx, &w.owner, &input).unwrap_err());
    assert!(
        got.contains(&(ErrorCode::ErrFormatInvalid, "person_account_id".into())),
        "{got:?}"
    );
    assert!(
        got.contains(&(ErrorCode::ErrReasonRequired, "reason".into())),
        "{got:?}"
    );
    let mut input = lent_in(w.akram, "100", "10-10-2026");
    input.person_account_id = None;
    let got = fields(&vouchers::save_money_lent(&w.env.ctx, &w.owner, &input).unwrap_err());
    assert!(
        got.contains(&(ErrorCode::ErrLentPersonRequired, "person_account_id".into())),
        "{got:?}"
    );

    // A salary deduction with no salary head in the books.
    let aslam = person(&w, "اسلم", "Aslam", None);
    vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "1,000", "10-10-2026"))
        .unwrap();
    let got = fields(
        &vouchers::save_money_lent_return(
            &w.env.ctx,
            &w.owner,
            &lent_back(aslam, "500", "10-10-2026", LentReturnMode::Salary),
        )
        .unwrap_err(),
    );
    assert!(
        got.contains(&(ErrorCode::ErrExpenseHeadRequired, "salary_head_id".into())),
        "{got:?}"
    );
    // A salary head sent with a cash return is a mistake.
    let mut input = lent_back(aslam, "500", "10-10-2026", LentReturnMode::Cash);
    input.salary_head_id = Some(1);
    let got = fields(&vouchers::save_money_lent_return(&w.env.ctx, &w.owner, &input).unwrap_err());
    assert!(
        got.contains(&(ErrorCode::ErrFormatInvalid, "salary_head_id".into())),
        "{got:?}"
    );
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 1);
}

#[test]
fn a_shop_expense_is_refused_when_the_shares_are_not_100() {
    let w = support::world_with(50_000, false);
    let electricity = head(&w, "بجلی", "Electricity");
    let error = vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &shop(electricity, "3,000", "10-10-2026"),
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrSharesNot100);
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 0);
    // The preview says the same, at no field, and shows no shares.
    let preview = vouchers::preview_expense(
        &w.env.ctx,
        &ExpensePreviewInput {
            entry: ExpenseEntry::ShopExpense(shop(electricity, "3,000", "10-10-2026")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(preview.shares.is_empty());
    assert!(
        preview
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrSharesNot100)
    );
    // A personal entry is never split, so it needs no shares.
    vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &personal(w.partners[0], "100", "10-10-2026"),
    )
    .unwrap();
}

#[test]
fn edit_and_cancel_go_through_the_generic_voucher_code() {
    let w = support::world();
    let electricity = head(&w, "بجلی", "Electricity");
    let a = w.partners[0];

    // Edit a shop expense 3,000 -> 3,384: the old shares are reversed, the new ones are E2E-20.
    let saved = vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &shop(electricity, "3,000", "10-10-2026"),
    )
    .unwrap();
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    let edited = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: saved.voucher_id,
            version: row.version,
            entry: VoucherEntry::ShopExpense(shop(electricity, "3,384", "10-10-2026")),
            reason: "typing mistake".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(
        (edited.number, edited.voucher_type),
        (1, VoucherType::ShopExpense)
    );
    // The old shares 1,200 / 900 / 900 and their reversal net to zero; the new ones are E2E-20.
    assert_eq!(live_shares(&w), vec![-1_354, -1_015, -1_015]);
    assert_eq!(cash(&w), 50_000 - 3_384);
    // Changing the type on an edit is refused.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    let refused = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: saved.voucher_id,
            version: row.version,
            entry: VoucherEntry::PersonalExpense(personal(a, "100", "10-10-2026")),
            reason: String::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrFormatInvalid);

    // Cancel it: cash comes back and the shares net to zero.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: saved.voucher_id,
            version: row.version,
            reason: "wrong head".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(cash(&w), 50_000);
    assert_eq!(live_shares(&w), vec![0, 0, 0]);
    assert_eq!(
        vouchers::get(&w.env.ctx, saved.voucher_id).unwrap().status,
        "cancelled"
    );
}

#[test]
fn a_take_or_loan_cannot_be_cancelled_or_made_smaller_than_its_returns() {
    let w = support::world();
    let a = w.partners[0];
    let take =
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "2,000", "10-10-2026"))
            .unwrap();
    vouchers::save_personal_return(
        &w.env.ctx,
        &w.owner,
        &PersonalReturnInput {
            date: "10-10-2026".into(),
            partner_id: Some(a),
            amount: "1,500".into(),
            narration: None,
        },
    )
    .unwrap();
    let row = vouchers::get(&w.env.ctx, take.voucher_id).unwrap();
    // Cancel: refused, nothing changes.
    let refused = vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: take.voucher_id,
            version: row.version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    // Edit to 1,000, below the 1,500 returned: refused. Edit to 1,500: allowed.
    let edit_to = |amount: &str| {
        let row = vouchers::get(&w.env.ctx, take.voucher_id).unwrap();
        vouchers::edit(
            &w.env.ctx,
            &w.owner,
            &VoucherEditInput {
                id: take.voucher_id,
                version: row.version,
                entry: VoucherEntry::PersonalExpense(personal(a, amount, "10-10-2026")),
                reason: String::new(),
                confirm_cash_below_zero: false,
            },
        )
    };
    assert_eq!(
        edit_to("1,000").unwrap_err().code(),
        ErrorCode::ErrReturnExceedsOpen
    );
    edit_to("1,500").unwrap();
    let takes = partners::takes(
        &w.env.ctx,
        &PartnerTakesInput {
            partner_id: a,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(
        (
            takes.rows[0].amount_paisa.rupees(),
            takes.rows[0].open_paisa.rupees()
        ),
        (1_500, 0)
    );
    // The chain of the journal is still whole after the refused changes.
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

/// A voucher edited to `entry`, with the version read now.
fn edit_voucher(
    w: &World,
    id: i64,
    entry: VoucherEntry,
) -> Result<mandi_services::core::vouchers::VoucherSaved, ServiceError> {
    let row = vouchers::get(&w.env.ctx, id).unwrap();
    vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id,
            version: row.version,
            entry,
            reason: String::new(),
            confirm_cash_below_zero: false,
        },
    )
}

/// The `open_paisa` value of a refusal.
fn open_value(error: &ServiceError) -> Option<i64> {
    match error {
        ServiceError::Rejected(items) => {
            items.iter().find_map(|i| match i.values.get("open_paisa") {
                Some(ErrorValue::Number(n)) => Some(*n),
                _ => None,
            })
        }
        _ => None,
    }
}

fn return_of(partner: i64, amount: &str, date: &str) -> PersonalReturnInput {
    PersonalReturnInput {
        date: date.into(),
        partner_id: Some(partner),
        amount: amount.into(),
        narration: None,
    }
}

// Money agent, plan 17, finding 1 (High): an edit checked only the NEW account.
#[test]
fn a_take_with_returns_cannot_be_edited_to_another_partner() {
    let w = support::world();
    let (a, b) = (w.partners[0], w.partners[1]);
    let take =
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "2,000", "10-10-2026"))
            .unwrap();
    vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "2,000", "10-10-2026"))
        .unwrap();
    let entries = count(&w, "SELECT COUNT(*) FROM journal_entries");
    let drawings = |partner: i64| {
        partners::takes(
            &w.env.ctx,
            &PartnerTakesInput {
                partner_id: partner,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
        .totals
    };

    // Moving the whole take to ب would leave الف's returns above what he took.
    let refused = edit_voucher(
        &w,
        take.voucher_id,
        VoucherEntry::PersonalExpense(personal(b, "2,000", "10-10-2026")),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    assert_eq!(open_value(&refused), Some(0));
    // Nothing was written: the same entries, الف's take and return, ب has nothing.
    assert_eq!(count(&w, "SELECT COUNT(*) FROM journal_entries"), entries);
    let (taken_a, returned_a, open_a) = {
        let t = drawings(a);
        (
            t.taken_paisa.rupees(),
            t.returned_paisa.rupees(),
            t.open_paisa.rupees(),
        )
    };
    assert_eq!((taken_a, returned_a, open_a), (2_000, 2_000, 0));
    assert_eq!(drawings(b).taken_paisa.rupees(), 0);
    assert_eq!(
        vouchers::get(&w.env.ctx, take.voucher_id).unwrap().status,
        "saved"
    );

    // The same edit is allowed when nothing was returned to الف: a take of ب's own.
    let free = vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &personal(w.partners[2], "700", "10-10-2026"),
    )
    .unwrap();
    edit_voucher(
        &w,
        free.voucher_id,
        VoucherEntry::PersonalExpense(personal(b, "700", "10-10-2026")),
    )
    .unwrap();
    assert_eq!(drawings(b).open_paisa.rupees(), 700);
    assert_eq!(drawings(w.partners[2]).open_paisa.rupees(), 0);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

#[test]
fn a_loan_with_returns_cannot_be_edited_to_another_person() {
    let w = support::world();
    let aslam = person(&w, "اسلم", "Aslam", None);
    let bashir = person(&w, "بشیر", "Bashir", None);
    let lent =
        vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "5,000", "10-10-2026"))
            .unwrap();
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(aslam, "5,000", "10-10-2026", LentReturnMode::Cash),
    )
    .unwrap();
    let entries = count(&w, "SELECT COUNT(*) FROM journal_entries");
    let refused = edit_voucher(
        &w,
        lent.voucher_id,
        VoucherEntry::MoneyLent(lent_in(bashir, "5,000", "10-10-2026")),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    assert_eq!(count(&w, "SELECT COUNT(*) FROM journal_entries"), entries);
    let open = |person_id: i64| {
        lent::list(
            &w.env.ctx,
            &LentListInput {
                person_id,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
        .totals
        .open_paisa
        .rupees()
    };
    assert_eq!((open(aslam), open(bashir)), (0, 0));
    // A loan nobody has returned can move to the other person.
    let other =
        vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "900", "10-10-2026"))
            .unwrap();
    // aslam's first loan is fully returned, so the 900 is open on him.
    assert_eq!(open(aslam), 900);
    edit_voucher(
        &w,
        other.voucher_id,
        VoucherEntry::MoneyLent(lent_in(bashir, "900", "10-10-2026")),
    )
    .unwrap();
    assert_eq!((open(aslam), open(bashir)), (0, 900));
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

// Money agent, plan 17, finding 2: the open amount of a return is judged on the return's own day.
#[test]
fn a_return_dated_before_its_take_cannot_be_saved() {
    let w = support::world();
    let a = w.partners[0];
    let aslam = person(&w, "اسلم", "Aslam", None);
    // The take and the loan are dated 11-10-26; 10-10-26 is the first open day.
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "2,000", "11-10-2026"))
        .unwrap();
    vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "5,000", "11-10-2026"))
        .unwrap();

    // The balance now is 2,000, but on 10-10-26 it was 0: nothing was open to return yet.
    let early =
        vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "500", "10-10-2026"))
            .unwrap_err();
    assert_eq!(early.code(), ErrorCode::ErrReturnExceedsOpen);
    assert_eq!(open_value(&early), Some(0));
    let early_loan = vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(aslam, "500", "10-10-2026", LentReturnMode::Cash),
    )
    .unwrap_err();
    assert_eq!(early_loan.code(), ErrorCode::ErrReturnExceedsOpen);
    // The preview says the same as the save (PRD FR-16.8, Code Standards B5).
    let preview = vouchers::preview_expense(
        &w.env.ctx,
        &ExpensePreviewInput {
            entry: ExpenseEntry::PersonalReturn(return_of(a, "500", "10-10-2026")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(
        preview
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrReturnExceedsOpen)
    );

    // On the day of the take, and after it, the same return saves; open stays the balance.
    vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "500", "11-10-2026"))
        .unwrap();
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(aslam, "500", "11-10-2026", LentReturnMode::Cash),
    )
    .unwrap();
    let takes = partners::takes(
        &w.env.ctx,
        &PartnerTakesInput {
            partner_id: a,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(takes.totals.open_paisa.rupees(), 1_500);
    // The most that can be returned is still the balance: 1,500 more saves, 1 more does not.
    let too_much =
        vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "1,501", "11-10-2026"))
            .unwrap_err();
    assert_eq!(open_value(&too_much), Some(1_500 * 100));
    vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "1,500", "11-10-2026"))
        .unwrap();
}

#[test]
fn a_take_cannot_be_moved_to_a_day_after_its_return() {
    let w = support::world();
    let a = w.partners[0];
    let take =
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "2,000", "10-10-2026"))
            .unwrap();
    vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "1,000", "10-10-2026"))
        .unwrap();
    set_today(&w.env, 2026, 10, 11);
    // On 10-10-26 الف would have returned 1,000 before he took anything.
    let refused = edit_voucher(
        &w,
        take.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "2,000", "11-10-2026")),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    // The same day stays allowed.
    edit_voucher(
        &w,
        take.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "2,500", "10-10-2026")),
    )
    .unwrap();
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

/// PRD FR-7.67, FR-16.8, E2E-39, edge case E56: one amount typed under each type of the خرچہ screen shows
/// that type's effect, and saving it posts that type and nothing else. Only the shop expense is
/// split by the shares; a personal expense reaches the chosen partner alone.
#[test]
fn the_type_saved_is_the_type_previewed_and_only_the_shop_expense_is_split() {
    let w = support::world();
    let electricity = head(&w, "بجلی", "Electricity");
    let a = w.partners[0];
    let posting_of = |entry_id: i64| -> String {
        raw(&w.dir)
            .query_row(
                &format!("SELECT posting_type FROM journal_entries WHERE id = {entry_id}"),
                [],
                |r| r.get::<_, String>(0),
            )
            .unwrap()
    };
    let share_rows_of = |entry_id: i64| {
        count(
            &w,
            &format!("SELECT COUNT(*) FROM partner_share_rows WHERE journal_entry_id = {entry_id}"),
        )
    };
    let preview = |entry: ExpenseEntry| {
        vouchers::preview_expense(
            &w.env.ctx,
            &ExpensePreviewInput {
                entry,
                replacing_voucher_id: None,
            },
        )
        .unwrap()
    };

    // 2,000 as a shop expense: the split 800 / 600 / 600, no single partner.
    let as_shop = preview(ExpenseEntry::ShopExpense(shop(
        electricity,
        "2,000",
        "10-10-2026",
    )));
    assert_eq!(as_shop.voucher_type, VoucherType::ShopExpense);
    assert_eq!(
        as_shop
            .shares
            .iter()
            .map(|s| s.amount_paisa.rupees())
            .collect::<Vec<_>>(),
        vec![800, 600, 600]
    );
    assert!(as_shop.partner_effect.is_none() && !as_shop.others_not_affected);

    // The same 2,000 as الف's personal expense: his total only, no split.
    let as_personal = preview(ExpenseEntry::PersonalExpense(personal(
        a,
        "2,000",
        "10-10-2026",
    )));
    assert_eq!(as_personal.voucher_type, VoucherType::PersonalExpense);
    assert!(as_personal.shares.is_empty() && as_personal.others_not_affected);
    let effect = as_personal.partner_effect.as_ref().unwrap();
    assert_eq!(
        (
            effect.total_before_paisa.rupees(),
            effect.total_after_paisa.rupees()
        ),
        (0, -2_000)
    );

    // Saving gives what each preview showed, with its own voucher type and posting type.
    let shop_saved = vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &shop(electricity, "2,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!(shop_saved.voucher_type, VoucherType::ShopExpense);
    assert_eq!(posting_of(shop_saved.entry_id), "shop_expense");
    assert_eq!(share_rows_of(shop_saved.entry_id), 3);

    let personal_saved =
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "2,000", "10-10-2026"))
            .unwrap();
    assert_eq!(personal_saved.voucher_type, VoucherType::PersonalExpense);
    assert_eq!(posting_of(personal_saved.entry_id), "personal_expense");
    assert_eq!(share_rows_of(personal_saved.entry_id), 0);
    // Cash: both were paid out of the 50,000.
    assert_eq!(cash(&w), 46_000);
    // The edit of a saved voucher keeps its saved type: a shop expense cannot become a personal one.
    let wrong = edit_voucher(
        &w,
        shop_saved.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "2,000", "10-10-2026")),
    )
    .unwrap_err();
    assert_eq!(wrong.code(), ErrorCode::ErrFormatInvalid);
    assert_eq!(
        vouchers::get(&w.env.ctx, shop_saved.voucher_id)
            .unwrap()
            .voucher_type,
        VoucherType::ShopExpense
    );
}

#[test]
fn cancelling_a_salary_return_gives_the_loan_back_and_its_shares_net_to_zero() {
    let w = support::world();
    let salary = head(&w, "تنخواہ", "Salaries");
    let aslam = person(&w, "اسلم", "Aslam", None);
    vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "2,000", "10-10-2026"))
        .unwrap();
    let back = vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(aslam, "2,000", "10-10-2026", LentReturnMode::Salary),
    )
    .unwrap();
    assert_eq!(live_shares(&w), vec![-800, -600, -600]);
    let row = vouchers::get(&w.env.ctx, back.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: back.voucher_id,
            version: row.version,
            reason: "mistake".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(live_shares(&w), vec![0, 0, 0]);
    let list = lent::list(
        &w.env.ctx,
        &LentListInput {
            person_id: aslam,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(list.totals.open_paisa, rupees(2_000));
    assert_eq!(list.rows.len(), 1);
    // The default head is the one called تنخواہ; a named head works too.
    let mut input = lent_back(aslam, "500", "10-10-2026", LentReturnMode::Salary);
    input.salary_head_id = Some(salary);
    vouchers::save_money_lent_return(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(
        lent::list(
            &w.env.ctx,
            &LentListInput {
                person_id: aslam,
                offset: 0,
                limit: 0
            }
        )
        .unwrap()
        .totals
        .open_paisa,
        rupees(1_500)
    );
}

#[test]
fn open_amounts_equal_the_account_balance_and_pages_add_up() {
    let w = support::world();
    let aslam = person(&w, "اسلم", "Aslam", None);
    for amount in ["1,000", "2,000", "3,000", "4,000", "500"] {
        vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, amount, "10-10-2026"))
            .unwrap();
    }
    for amount in ["1,500", "2,250", "700"] {
        vouchers::save_money_lent_return(
            &w.env.ctx,
            &w.owner,
            &lent_back(aslam, amount, "10-10-2026", LentReturnMode::Cash),
        )
        .unwrap();
        let list = lent::list(
            &w.env.ctx,
            &LentListInput {
                person_id: aslam,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap();
        // Σ open is the debit balance of his account, never below 0.
        assert_eq!(list.balance.side, BalanceSide::Dr);
        assert_eq!(list.totals.open_paisa, list.balance.amount_paisa);
        assert!(list.rows.iter().all(|r| !r.open_paisa.is_negative()));
    }
    // Oldest first: the first loans are settled before the later ones.
    let whole = lent::list(
        &w.env.ctx,
        &LentListInput {
            person_id: aslam,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let opens: Vec<i64> = whole.rows.iter().map(|r| r.open_paisa.rupees()).collect();
    // Returns 1,500 + 2,250 + 700 = 4,450 settle 1,000 and 2,000 and part of 3,000.
    assert_eq!(opens, vec![0, 0, 1_550, 4_000, 500]);
    // Pages of 2: the rows together are the whole list; every page has the same totals.
    let mut rows = Vec::new();
    let mut offset = 0;
    loop {
        let page = lent::list(
            &w.env.ctx,
            &LentListInput {
                person_id: aslam,
                offset,
                limit: 2,
            },
        )
        .unwrap();
        assert_eq!(page.totals, whole.totals);
        assert_eq!(page.total_count, 5);
        if page.rows.is_empty() {
            break;
        }
        rows.extend(page.rows);
        offset += 2;
    }
    assert_eq!(rows, whole.rows);
}

#[test]
fn a_staff_member_becomes_a_lent_person_once() {
    let w = support::world();
    let staff = confirming(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::Staff(StaffInput {
                id: None,
                version: None,
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                job: "munshi".into(),
                phone: String::new(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
    let _ = staff;
    let staff_id: i64 = count(&w, "SELECT id FROM staff LIMIT 1");
    let account = person(&w, "اسلم", "Aslam", Some(staff_id));
    assert_eq!(
        count(
            &w,
            &format!("SELECT account_id FROM staff WHERE id = {staff_id}")
        ),
        account
    );
    let list = lent::list(
        &w.env.ctx,
        &LentListInput {
            person_id: account,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(list.person.staff.map(|s| s.id), Some(staff_id));
    // The same staff member again, or a staff member that does not exist, is refused.
    let again = lent::person_create(
        &w.env.ctx,
        &w.owner,
        &LentPersonCreateInput {
            name_ur: "اسلم".into(),
            name_en: "Aslam".into(),
            phone: None,
            staff_id: Some(staff_id),
            confirmed_rules_words: vec!["aslam".into(), "اسلم".into()],
        },
    )
    .unwrap_err();
    assert_eq!(again.code(), ErrorCode::ErrFormatInvalid);
    let missing = lent::person_create(
        &w.env.ctx,
        &w.owner,
        &LentPersonCreateInput {
            name_ur: "ناصر".into(),
            name_en: "Nasir".into(),
            phone: None,
            staff_id: Some(9_999),
            confirmed_rules_words: Vec::new(),
        },
    )
    .unwrap_err();
    assert_eq!(missing.code(), ErrorCode::ErrNotFound);
    // A wrong phone is reported at `phone`.
    let bad_phone = lent::person_create(
        &w.env.ctx,
        &w.owner,
        &LentPersonCreateInput {
            name_ur: "ناصر".into(),
            name_en: "Nasir".into(),
            phone: Some("12".into()),
            staff_id: None,
            confirmed_rules_words: vec!["nasir".into(), "ناصر".into()],
        },
    )
    .unwrap_err();
    match bad_phone {
        ServiceError::Rejected(items) => {
            assert!(
                items.iter().any(|i| i.field.as_deref() == Some("phone")),
                "{items:?}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_expense_heads_come_most_used_first_and_last_month_is_proposed_for_checking() {
    let w = world_with_old_shares();
    let electricity = head(&w, "بجلی", "Electricity");
    let rent = head(&w, "کرایہ دکان", "Shop Rent");
    let tea = head(&w, "چائے", "Tea");
    // September: electricity twice, rent once (paid to the landlord), tea once.
    for (id, amount, date, paid_to) in [
        (electricity, "3,000", "05-09-2026", "WAPDA"),
        (electricity, "3,200", "25-09-2026", "WAPDA"),
        (rent, "10,000", "01-09-2026", "Landlord"),
        (tea, "400", "10-09-2026", ""),
    ] {
        let mut input = shop(id, amount, date);
        input.paid_to = (!paid_to.is_empty()).then(|| paid_to.to_owned());
        input.narration = Some("monthly".into());
        vouchers::save_shop_expense(&w.env.ctx, &w.owner, &input).unwrap();
    }
    // October: tea again.
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &shop(tea, "450", "03-10-2026")).unwrap();

    let heads = masters::expense_heads(&w.env.ctx).unwrap();
    let order: Vec<(&str, i64)> = heads
        .rows
        .iter()
        .map(|r| (r.head.name_en.as_str(), r.use_count))
        .collect();
    assert_eq!(
        order,
        vec![("Electricity", 2), ("Tea", 2), ("Shop Rent", 1)]
    );
    assert_eq!(heads.rows[0].last_used_date.as_deref(), Some("2026-09-25"));

    // October's proposal: the heads of September that October does not have yet, with the latest
    // amount of each. Tea is already entered in October.
    let repeat = vouchers::repeat_last_month_preview(
        &w.env.ctx,
        &RepeatLastMonthInput {
            month: "2026-10".into(),
        },
    )
    .unwrap();
    assert_eq!(
        (repeat.month.as_str(), repeat.last_month.as_str()),
        ("2026-10", "2026-09")
    );
    let rows: Vec<(&str, i64, &str, &str, &str)> = repeat
        .rows
        .iter()
        .map(|r| {
            (
                r.head.name_en.as_str(),
                r.amount_paisa.rupees(),
                r.paid_to.as_str(),
                r.narration.as_str(),
                r.last_date.as_str(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            ("Electricity", 3_200, "WAPDA", "monthly", "2026-09-25"),
            ("Shop Rent", 10_000, "Landlord", "monthly", "2026-09-01"),
        ]
    );
    assert_eq!(repeat.total_paisa, rupees(13_200));
    // Nothing was saved by the proposal.
    assert_eq!(count(&w, "SELECT COUNT(*) FROM vouchers"), 5);
    // Across a year: December looks at November (nothing); January looks at December.
    let none = vouchers::repeat_last_month_preview(
        &w.env.ctx,
        &RepeatLastMonthInput {
            month: "2026-12".into(),
        },
    )
    .unwrap();
    assert!(none.rows.is_empty() && none.total_paisa == Paisa::ZERO);
    let january = vouchers::repeat_last_month_preview(
        &w.env.ctx,
        &RepeatLastMonthInput {
            month: "2027-01".into(),
        },
    )
    .unwrap();
    assert_eq!(january.last_month, "2026-12");
    // A month that is not YYYY-MM is a format error at `month`.
    for bad in ["2026-13", "10-2026", "", "2026-1"] {
        let error = vouchers::repeat_last_month_preview(
            &w.env.ctx,
            &RepeatLastMonthInput { month: bad.into() },
        )
        .unwrap_err();
        assert_eq!(error.code(), ErrorCode::ErrFormatInvalid, "{bad}");
    }
}

#[test]
fn a_paid_to_that_is_a_partners_name_is_noted_in_either_script() {
    let w = support::world();
    let tea = head(&w, "چائے", "Tea");
    for (paid_to, found) in [
        ("partner a", true),
        ("  Partner   A ", true),
        ("الف", true),
        ("WAPDA", false),
        ("", false),
    ] {
        let mut input = shop(tea, "100", "10-10-2026");
        input.paid_to = Some(paid_to.to_owned());
        let preview = vouchers::preview_expense(
            &w.env.ctx,
            &ExpensePreviewInput {
                entry: ExpenseEntry::ShopExpense(input),
                replacing_voucher_id: None,
            },
        )
        .unwrap();
        assert_eq!(
            preview.paid_to_partner.map(|p| p.id),
            found.then_some(w.partners[0]),
            "{paid_to:?}"
        );
    }
}

#[test]
fn the_new_voucher_numbers_are_found_by_their_series_letters() {
    use mandi_services::core::accounts::{GlobalHit, SearchGlobalInput};
    let w = support::world();
    let tea = head(&w, "چائے", "Tea");
    let a = w.partners[0];
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &shop(tea, "100", "10-10-2026")).unwrap();
    vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "200", "10-10-2026"))
        .unwrap();
    vouchers::save_personal_return(
        &w.env.ctx,
        &w.owner,
        &PersonalReturnInput {
            date: "10-10-2026".into(),
            partner_id: Some(a),
            amount: "50".into(),
            narration: None,
        },
    )
    .unwrap();
    let found = |text: &str| -> Vec<VoucherType> {
        search::global(
            &w.env.ctx,
            &w.owner,
            &SearchGlobalInput {
                text: text.into(),
                limit: 20,
            },
        )
        .unwrap()
        .into_iter()
        .filter_map(|hit| match hit {
            GlobalHit::Voucher { hit } => Some(hit.voucher_type),
            _ => None,
        })
        .collect()
    };
    assert_eq!(found("E-1"), vec![VoucherType::ShopExpense]);
    assert_eq!(found("خ-1"), vec![VoucherType::ShopExpense]);
    assert_eq!(found("D-1"), vec![VoucherType::PersonalExpense]);
    assert_eq!(found("DR 1"), vec![VoucherType::PersonalReturn]);
    assert_eq!(found("ذو-1"), vec![VoucherType::PersonalReturn]);
    // The digits alone find the number in every series.
    let mut all = found("1");
    all.sort_by_key(|t| t.as_str());
    assert_eq!(
        all,
        vec![
            VoucherType::PersonalExpense,
            VoucherType::PersonalReturn,
            VoucherType::ShopExpense
        ]
    );
}

#[test]
fn the_cash_book_filters_by_day_and_gives_the_opening_before_it() {
    let w = world_with_old_shares();
    let tea = head(&w, "چائے", "Tea");
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &shop(tea, "1,000", "08-10-2026")).unwrap();
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &shop(tea, "2,000", "09-10-2026")).unwrap();
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &shop(tea, "4,000", "10-10-2026")).unwrap();
    let book = |from: Option<&str>, to: Option<&str>| {
        mandi_services::cashbook::get(
            &w.env.ctx,
            &mandi_services::core::cashbook::CashbookInput {
                from: from.map(str::to_owned),
                to: to.map(str::to_owned),
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
    };
    // The world's opening cash (50,000) is dated 10-10-2026 (support::set_cash).
    let day = book(Some("09-10-2026"), Some("09-10-2026"));
    assert_eq!(day.opening_paisa, rupees(-1_000));
    assert_eq!(day.closing_paisa, rupees(-3_000));
    assert_eq!(day.total_count, 1);
    assert_eq!(day.rows[0].running_paisa, rupees(-3_000));
    assert_eq!(day.totals.out_paisa, rupees(2_000));
    let last = book(Some("10-10-2026"), None);
    assert_eq!(last.opening_paisa, rupees(-3_000));
    assert_eq!(last.closing_paisa, rupees(50_000 - 7_000));
    // A bad date is refused at its field.
    let error = mandi_services::cashbook::get(
        &w.env.ctx,
        &mandi_services::core::cashbook::CashbookInput {
            from: Some("31-02-2026".into()),
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap_err();
    assert_eq!(first_field(&error).as_deref(), Some("from"));
}

fn first_field(error: &ServiceError) -> Option<String> {
    match error {
        ServiceError::Rejected(items) => items.first().and_then(|i| i.field.clone()),
        _ => None,
    }
}

#[test]
fn a_drawings_opening_debit_is_money_taken_dated_its_opening_day() {
    use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
    let w = support::world();
    let drawings = support::id_of(&w.env, "06030001");
    let version = mandi_services::accounts::get(&w.env.ctx, drawings)
        .unwrap()
        .version;
    mandi_services::accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: drawings,
                version,
                opening_date: "10-10-2026".into(),
                opening_dr: "2,000".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    let takes = partners::takes(
        &w.env.ctx,
        &PartnerTakesInput {
            partner_id: w.partners[0],
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(takes.rows.len(), 1);
    let row = &takes.rows[0];
    assert_eq!(
        row.source,
        mandi_services::core::expenses::TakeSource::Opening
    );
    assert_eq!(
        (
            row.amount_paisa.rupees(),
            row.open_paisa.rupees(),
            row.entry_date.as_str(),
            row.voucher_id
        ),
        (2_000, 2_000, "2026-10-10", None)
    );
    // He can return it: a return up to the opening debit is allowed, above it is not.
    let refused = vouchers::save_personal_return(
        &w.env.ctx,
        &w.owner,
        &PersonalReturnInput {
            date: "10-10-2026".into(),
            partner_id: Some(w.partners[0]),
            amount: "2,001".into(),
            narration: None,
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    vouchers::save_personal_return(
        &w.env.ctx,
        &w.owner,
        &PersonalReturnInput {
            date: "10-10-2026".into(),
            partner_id: Some(w.partners[0]),
            amount: "2,000".into(),
            narration: None,
        },
    )
    .unwrap();
}

#[test]
fn the_generic_voucher_preview_knows_the_credit_kinds() {
    use mandi_services::core::vouchers::VoucherPreviewInput;
    let w = support::world();
    let a = w.partners[0];
    vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "2,000", "10-10-2026"))
        .unwrap();
    let preview = |entry: VoucherEntry| {
        vouchers::preview(
            &w.env.ctx,
            &VoucherPreviewInput {
                entry,
                replacing_voucher_id: None,
            },
        )
        .unwrap()
    };
    // A return is a credit on his drawings account: 2,000 debit now, 1,500 debit after 500 back.
    let back = preview(VoucherEntry::PersonalReturn(PersonalReturnInput {
        date: "10-10-2026".into(),
        partner_id: Some(a),
        amount: "500".into(),
        narration: None,
    }));
    assert!(back.errors.is_empty(), "{:?}", back.errors);
    assert_eq!(
        (
            back.current_balance.side,
            back.current_balance.amount_paisa.rupees()
        ),
        (BalanceSide::Dr, 2_000)
    );
    assert_eq!(
        (
            back.new_balance.side,
            back.new_balance.amount_paisa.rupees()
        ),
        (BalanceSide::Dr, 1_500)
    );
    // Another take is a debit: 2,000 becomes 2,300.
    let take = preview(VoucherEntry::PersonalExpense(personal(
        a,
        "300",
        "10-10-2026",
    )));
    assert_eq!(take.new_balance.amount_paisa.rupees(), 2_300);
}

// ---- Money agent, plan 17, second pass ----

fn lent_back_id(
    w: &World,
    person: i64,
    amount: &str,
    date: &str,
) -> mandi_services::core::vouchers::VoucherSaved {
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(person, amount, date, LentReturnMode::Cash),
    )
    .unwrap()
}

// Finding 1: a return must not leave a day IN BETWEEN overdrawn.
#[test]
fn a_return_cannot_leave_a_day_between_two_takes_overdrawn() {
    let w = support::world();
    let a = w.partners[0];
    let aslam = person(&w, "اسلم", "Aslam", None);
    set_today(&w.env, 2026, 10, 13);
    // الف: take 1,000 on 10-10, return 1,000 on 12-10, take 1,000 on 13-10. Day ends 1,000, 0, 1,000.
    vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "1,000", "10-10-2026"))
        .unwrap();
    vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "1,000", "12-10-2026"))
        .unwrap();
    let last =
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "1,000", "13-10-2026"))
            .unwrap();
    // A second return of 1,000 dated 11-10 would leave 12-10 at -1,000: refused, open 0.
    let refused =
        vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "1,000", "11-10-2026"))
            .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    assert_eq!(open_value(&refused), Some(0));
    // The preview says the same.
    let preview = vouchers::preview_expense(
        &w.env.ctx,
        &ExpensePreviewInput {
            entry: ExpenseEntry::PersonalReturn(return_of(a, "1,000", "11-10-2026")),
            replacing_voucher_id: None,
        },
    )
    .unwrap();
    assert!(
        preview
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrReturnExceedsOpen)
    );
    // On 13-10, after the second take, 1,000 can be returned; so a take stays editable.
    vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "1,000", "13-10-2026"))
        .unwrap();
    edit_voucher(
        &w,
        last.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "1,000", "13-10-2026")),
    )
    .unwrap();

    // The same for a loan.
    vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "5,000", "10-10-2026"))
        .unwrap();
    lent_back_id(&w, aslam, "5,000", "12-10-2026");
    vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "5,000", "13-10-2026"))
        .unwrap();
    let refused = vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &lent_back(aslam, "5,000", "11-10-2026", LentReturnMode::Cash),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    assert_eq!(open_value(&refused), Some(0));
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

// Finding 2a: the preview of an edit of a RETURN counts the books without the old return.
#[test]
fn the_edit_preview_of_a_return_works_on_the_books_without_the_old_return() {
    use mandi_services::core::vouchers::VoucherPreviewInput;
    let w = support::world();
    let aslam = person(&w, "اسلم", "Aslam", None);
    vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "5,000", "10-10-2026"))
        .unwrap();
    let back = lent_back_id(&w, aslam, "3,000", "10-10-2026");
    let preview = |amount: &str| {
        vouchers::preview(
            &w.env.ctx,
            &VoucherPreviewInput {
                entry: VoucherEntry::MoneyLentReturn(lent_back(
                    aslam,
                    amount,
                    "10-10-2026",
                    LentReturnMode::Cash,
                )),
                replacing_voucher_id: Some(back.voucher_id),
            },
        )
        .unwrap()
    };
    // Open is 2,000 now, 5,000 without the old return: the same 3,000 and up to 5,000 are fine.
    assert!(
        preview("3,000").errors.is_empty(),
        "{:?}",
        preview("3,000").errors
    );
    assert!(preview("5,000").errors.is_empty());
    let over = preview("5,001");
    assert!(
        over.errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrReturnExceedsOpen)
    );
    // And the edit itself agrees with both previews.
    edit_voucher(
        &w,
        back.voucher_id,
        VoucherEntry::MoneyLentReturn(lent_back(
            aslam,
            "5,000",
            "10-10-2026",
            LentReturnMode::Cash,
        )),
    )
    .unwrap();
}

// Finding 2b: the edit preview of a TAKE or a LOAN says what the save will say.
#[test]
fn the_edit_preview_of_a_take_or_a_loan_refuses_what_the_edit_refuses() {
    use mandi_services::core::vouchers::VoucherPreviewInput;
    let w = support::world();
    let (a, b) = (w.partners[0], w.partners[1]);
    let aslam = person(&w, "اسلم", "Aslam", None);
    let bashir = person(&w, "بشیر", "Bashir", None);

    // A take of 2,000, returned in full.
    let take =
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "2,000", "10-10-2026"))
            .unwrap();
    vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "2,000", "10-10-2026"))
        .unwrap();
    let preview_take = |partner: i64, amount: &str| {
        vouchers::preview_expense(
            &w.env.ctx,
            &ExpensePreviewInput {
                entry: ExpenseEntry::PersonalExpense(personal(partner, amount, "10-10-2026")),
                replacing_voucher_id: Some(take.voucher_id),
            },
        )
        .unwrap()
    };
    let refuses = |errors: &[mandi_services::core::error::FieldError]| {
        errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrReturnExceedsOpen && e.field == "amount")
    };
    assert!(refuses(&preview_take(a, "1,000").errors), "smaller");
    // A refused preview carries no total, no share and no cash change (as the generic preview).
    let refused_preview = preview_take(a, "1,000");
    assert!(refused_preview.partner_effect.is_none() && !refused_preview.others_not_affected);
    assert!(refuses(&preview_take(b, "2,000").errors), "moved to ب");
    assert!(preview_take(a, "2,000").errors.is_empty(), "the same");
    assert!(preview_take(a, "2,500").errors.is_empty(), "bigger");
    // The saves agree.
    for (partner, amount) in [(a, "1,000"), (b, "2,000")] {
        let refused = edit_voucher(
            &w,
            take.voucher_id,
            VoucherEntry::PersonalExpense(personal(partner, amount, "10-10-2026")),
        )
        .unwrap_err();
        assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    }

    // A loan of 5,000, returned in full: the generic preview.
    let lent =
        vouchers::save_money_lent(&w.env.ctx, &w.owner, &lent_in(aslam, "5,000", "10-10-2026"))
            .unwrap();
    lent_back_id(&w, aslam, "5,000", "10-10-2026");
    let preview_lent = |person_id: i64, amount: &str| {
        vouchers::preview(
            &w.env.ctx,
            &VoucherPreviewInput {
                entry: VoucherEntry::MoneyLent(lent_in(person_id, amount, "10-10-2026")),
                replacing_voucher_id: Some(lent.voucher_id),
            },
        )
        .unwrap()
    };
    assert!(refuses(&preview_lent(aslam, "4,000").errors), "smaller");
    assert!(
        refuses(&preview_lent(bashir, "5,000").errors),
        "moved to بشیر"
    );
    assert!(preview_lent(aslam, "5,000").errors.is_empty());
    for (person_id, amount) in [(aslam, "4,000"), (bashir, "5,000")] {
        let refused = edit_voucher(
            &w,
            lent.voucher_id,
            VoucherEntry::MoneyLent(lent_in(person_id, amount, "10-10-2026")),
        )
        .unwrap_err();
        assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    }
}

// Finding 4: an account that was overdrawn on purpose (a جنرل credit above the open amount is
// allowed) does not block an edit that makes it no worse.
#[test]
fn an_overdrawn_history_blocks_only_the_changes_that_make_it_worse() {
    use mandi_services::core::general::{GeneralInput, GeneralLineInput};
    let w = support::world();
    let a = w.partners[0];
    let take =
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "1,000", "10-10-2026"))
            .unwrap();
    let drawings = support::id_of(&w.env, "06030001");
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        person_key: None,
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    // A جنرل puts 5,000 on the credit side of his drawings: the balance is -4,000.
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &GeneralInput {
            date: "10-10-2026".into(),
            narration: None,
            lines: vec![
                line(support::id_of(&w.env, "01010001"), "5,000", ""),
                line(drawings, "", "5,000"),
            ],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // A bigger take makes it better, the same take changes nothing: both allowed.
    edit_voucher(
        &w,
        take.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "2,000", "10-10-2026")),
    )
    .unwrap();
    edit_voucher(
        &w,
        take.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "2,000", "10-10-2026")),
    )
    .unwrap();
    // A smaller take makes it worse: refused.
    let refused = edit_voucher(
        &w,
        take.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "500", "10-10-2026")),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
}

// Finding 3: an opening on money taken (0603) or lent (0103) is a debit only, and a change of it
// may not leave returns above it.
#[test]
fn an_opening_on_money_taken_is_a_debit_and_cannot_be_cleared_under_its_returns() {
    use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
    let w = support::world();
    let a = w.partners[0];
    let drawings = support::id_of(&w.env, "06030001");
    let input = |dr: &str, cr: &str| {
        let version = mandi_services::accounts::get(&w.env.ctx, drawings)
            .unwrap()
            .version;
        OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: drawings,
                version,
                opening_date: "10-10-2026".into(),
                opening_dr: dr.into(),
                opening_cr: cr.into(),
                opening_is_advance: false,
            }],
        }
    };
    let field_of = |error: &ServiceError| match error {
        ServiceError::Rejected(items) => items
            .iter()
            .map(|i| (i.code, i.field.clone()))
            .collect::<Vec<_>>(),
        other => panic!("{other:?}"),
    };

    // A credit opening is refused, in the preview and in the save.
    let credit = input("", "1,000");
    let preview = mandi_services::accounts::openings_preview(&w.env.ctx, &credit).unwrap();
    assert!(
        preview.rows[0]
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrOpeningNotAllowed && e.field == "lines[0].opening_cr")
    );
    let refused =
        mandi_services::accounts::openings_save(&w.env.ctx, &w.owner, &credit).unwrap_err();
    assert!(
        field_of(&refused).contains(&(
            ErrorCode::ErrOpeningNotAllowed,
            Some("lines[0].opening_cr".into())
        )),
        "{:?}",
        field_of(&refused)
    );

    // A debit of 2,000 is money taken; he returns 2,000.
    mandi_services::accounts::openings_save(&w.env.ctx, &w.owner, &input("2,000", "")).unwrap();
    vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "2,000", "10-10-2026"))
        .unwrap();

    // Clearing or lowering the opening would leave the returns above it: refused, in the preview
    // and in the save, and nothing is written.
    let entries = count(&w, "SELECT COUNT(*) FROM journal_entries");
    for wanted in [input("", ""), input("1,500", "")] {
        let preview = mandi_services::accounts::openings_preview(&w.env.ctx, &wanted).unwrap();
        assert!(
            preview.rows[0]
                .errors
                .iter()
                .any(|e| e.code == ErrorCode::ErrReturnExceedsOpen
                    && e.field == "lines[0].opening_dr"),
            "{:?}",
            preview.rows[0].errors
        );
        let refused =
            mandi_services::accounts::openings_save(&w.env.ctx, &w.owner, &wanted).unwrap_err();
        assert!(
            field_of(&refused)
                .iter()
                .any(|(code, _)| *code == ErrorCode::ErrReturnExceedsOpen),
            "{:?}",
            field_of(&refused)
        );
    }
    assert_eq!(count(&w, "SELECT COUNT(*) FROM journal_entries"), entries);
    // Raising it is fine, and the takes list still shows open = balance.
    mandi_services::accounts::openings_save(&w.env.ctx, &w.owner, &input("3,000", "")).unwrap();
    let takes = partners::takes(
        &w.env.ctx,
        &PartnerTakesInput {
            partner_id: a,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(takes.totals.open_paisa.rupees(), 1_000);
}

// Third pass, finding 1: the books are compared day by day, so a deeper hole on an earlier day
// (a جنرل credit above the open amount is allowed) cannot hide a new one on a later day.
#[test]
fn a_deep_hole_on_an_earlier_day_does_not_hide_a_new_one_on_a_later_day() {
    use mandi_services::core::general::{GeneralInput, GeneralLineInput};
    let w = support::world();
    let a = w.partners[0];
    set_today(&w.env, 2026, 10, 12);
    let drawings = support::id_of(&w.env, "06030001");
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        person_key: None,
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    // 10-10: a take of 5,000, then a جنرل puts 6,000 on the credit side of his drawings: -1,000.
    vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "5,000", "10-10-2026"))
        .unwrap();
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &GeneralInput {
            date: "10-10-2026".into(),
            narration: None,
            lines: vec![
                line(support::id_of(&w.env, "01010001"), "6,000", ""),
                line(drawings, "", "6,000"),
            ],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // 11-10: a take of 3,000 (day end 2,000). 12-10: a return of 2,000 (day end 0).
    let second =
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal(a, "3,000", "11-10-2026"))
            .unwrap();
    vouchers::save_personal_return(&w.env.ctx, &w.owner, &return_of(a, "2,000", "12-10-2026"))
        .unwrap();
    // The 11-10 take cut to 2,500 would end 12-10 at -500, a new hole: refused, over by 500.
    let refused = edit_voucher(
        &w,
        second.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "2,500", "11-10-2026")),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    match &refused {
        ServiceError::Rejected(items) => assert_eq!(
            items[0].values.get("over_paisa"),
            Some(&ErrorValue::Number(500 * 100))
        ),
        other => panic!("{other:?}"),
    }
    // The preview says the same; a bigger take, and the same one, are fine.
    let preview = vouchers::preview_expense(
        &w.env.ctx,
        &ExpensePreviewInput {
            entry: ExpenseEntry::PersonalExpense(personal(a, "2,500", "11-10-2026")),
            replacing_voucher_id: Some(second.voucher_id),
        },
    )
    .unwrap();
    assert!(
        preview
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrReturnExceedsOpen)
    );
    edit_voucher(
        &w,
        second.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "3,000", "11-10-2026")),
    )
    .unwrap();
    edit_voucher(
        &w,
        second.voucher_id,
        VoucherEntry::PersonalExpense(personal(a, "3,500", "11-10-2026")),
    )
    .unwrap();
}
