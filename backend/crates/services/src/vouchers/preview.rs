//! The preview of a voucher (PRD FR-7.58, FR-7.59, FR-6.7, FR-7.68, FR-15.7, plan 15): what saving
//! the entry would do, shown while it is typed. Nothing is written.
//!
//! The preview calls the same checks as the save (`Planned::plan`, the seller limit) and the same
//! canonical workings as the matching after a save (`matching::key_view`, `matching::seller_view`),
//! so the screen and the books never differ (Code Standards B5). With `replacing_voucher_id` the
//! old posting is taken out of the workings first, which is what an edit does.

use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers as repo;
use mandi_domain::capital::capital_excess;
use mandi_domain::cash;
use mandi_domain::core::accounts::{Balance, BalanceSide};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::vouchers::{
    AdvanceEffect, BillEffect, OpenAdvance, OpenBill, VoucherPreview, VoucherPreviewInput,
    VoucherRow, VoucherType,
};

use super::common::{Role, balance_of_net, cash_change, stored_cash_change};
use super::kinds::Planned;
use super::reads::line_source;
use super::receipt::seller_receipt;
use crate::cash::cash_in_hand;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::floor::preview_refusal;
use crate::ledger;
use crate::matching::{self, KeyChange, WHAT_IF_LINE, WhatIfLine};

/// The voucher an edit would replace, with its posting.
struct Replacing {
    row: VoucherRow,
    entry: ledger_repo::StoredEntry,
}

/// What the old posting did to the signed balance of its party (debit positive): a receipt put a
/// credit on it, a payment and an advance a debit.
fn old_effect(old: &VoucherRow) -> Result<Paisa, ServiceError> {
    if credits_party(old.voucher_type) {
        old.amount_paisa.checked_neg().map_err(money_error)
    } else {
        Ok(old.amount_paisa)
    }
}

/// True for the kinds that put a credit on their account: a receipt, a return of money taken or
/// lent, capital invested and money received for a khata. The others put a debit on it.
fn credits_party(kind: VoucherType) -> bool {
    matches!(
        kind,
        VoucherType::Receipt
            | VoucherType::PersonalReturn
            | VoucherType::MoneyLentReturn
            | VoucherType::CapitalIn
            | VoucherType::KhataIn
    )
}

fn find_replacing(
    conn: &Connection,
    input: &VoucherPreviewInput,
    errors: &mut Vec<FieldError>,
) -> Result<Option<Replacing>, ServiceError> {
    let Some(id) = input.replacing_voucher_id else {
        return Ok(None);
    };
    let field = "replacing_voucher_id";
    let Some(row) = repo::get(conn, id)? else {
        errors.push(FieldError::new(ErrorCode::ErrNotFound, field));
        return Ok(None);
    };
    if row.status != "saved" {
        errors.push(FieldError::new(ErrorCode::ErrVoucherCancelled, field));
        return Ok(None);
    }
    if row.voucher_type != input.entry.voucher_type() {
        errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, "entry.type"));
        return Ok(None);
    }
    let entry = match row.entry_id {
        Some(entry_id) => ledger_repo::get_entry(conn, entry_id)?,
        None => None,
    };
    Ok(entry.map(|entry| Replacing { row, entry }))
}

/// The open bill detail of a line, with what is open on it now.
fn open_bill(
    row: &repo::BillRow,
    open: Paisa,
    today: BusinessDate,
) -> Result<OpenBill, ServiceError> {
    let day = BusinessDate::parse_iso(&row.entry_date)
        .ok_or_else(|| ServiceError::Data(format!("entry date {}", row.entry_date)))?;
    Ok(OpenBill {
        line_id: row.line_id,
        entry_date: row.entry_date.clone(),
        source: line_source(&row.posting_type, &row.source_type),
        number: row.number,
        amount_paisa: Paisa::try_from_paisa(row.amount_paisa).map_err(money_error)?,
        open_paisa: open,
        days: today.days_after(day),
    })
}

/// Works out what a voucher would do, without saving it.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data`, `ERR_OVERFLOW`. A mistake of the user is in the
/// answer's `errors`, not an error of the call.
#[allow(clippy::too_many_lines)]
pub fn preview(ctx: &Context, input: &VoucherPreviewInput) -> Result<VoucherPreview, ServiceError> {
    let window = day::window(ctx)?;
    let today = window.1;
    ctx.db.read(|conn| {
        // The voucher an edit replaces is found first, so a return is judged on the books without
        // it, as the edit does (it posts the reversal before it plans; Code Standards B5).
        let mut replacing_errors: Vec<FieldError> = Vec::new();
        let replacing = find_replacing(conn, input, &mut replacing_errors)?;
        let planned = Planned::plan(
            conn,
            window,
            &input.entry,
            replacing.as_ref().map(|r| &r.entry),
        )?;
        let mut errors: Vec<FieldError> = planned.errors().to_vec();
        errors.extend(replacing_errors);
        let mut warnings: Vec<FieldError> = Vec::new();
        // A take or a loan that an edit would leave with returns above it says so now, like the
        // edit does; the old account is checked too (money agent, plan 17).
        if errors.is_empty()
            && let (Some(account), Some(date), Some(amount)) =
                (planned.account(), planned.date(), planned.amount())
        {
            let old = replacing
                .as_ref()
                .and_then(|r| r.row.account.as_ref().map(|a| (a.id, &r.entry)));
            if let Some(refusal) = preview_refusal(
                conn,
                planned.voucher_type(),
                old,
                Some((account.id, &date.to_iso(), amount)),
            )? {
                errors.push(refusal);
            }
        }
        let remove_entry = replacing.as_ref().map(|r| r.entry.id);
        let clean = errors.is_empty();

        let account = planned.account().cloned();
        let key = planned.person_key().map(str::to_owned);
        let amount = planned.amount().unwrap_or(Paisa::ZERO);
        let date = planned.date();

        // The balance of the account (the person, for a پختہ کھاتہ) without the voucher being
        // replaced, then with the new one.
        let (current_balance, new_balance) = match &account {
            Some(account) => {
                let now =
                    ledger::signed(ledger::balance_in(conn, account.id, key.as_deref(), None)?)?;
                let base = match &replacing {
                    Some(old)
                        if old.row.account.as_ref().map(|a| a.id) == Some(account.id)
                            && old.row.person.as_ref().map(|p| p.person_key.as_str())
                                == key.as_deref() =>
                    {
                        now.checked_sub(old_effect(&old.row)?)
                            .map_err(money_error)?
                    }
                    _ => now,
                };
                let after = if clean && amount != Paisa::ZERO {
                    if credits_party(planned.voucher_type()) {
                        base.checked_sub(amount).map_err(money_error)?
                    } else {
                        base.checked_add(amount).map_err(money_error)?
                    }
                } else {
                    base
                };
                (balance_of_net(base)?, balance_of_net(after)?)
            }
            None => (Balance::ZERO, Balance::ZERO),
        };

        let mut bill_effects = Vec::new();
        let mut extra_credit = Paisa::ZERO;
        let mut advance_effects = Vec::new();
        let mut owes_settled = Paisa::ZERO;
        let mut return_limit = None;
        let mut payable_open = None;
        let mut payable_is_zero = false;

        if let (true, Some(account), Some(date)) = (clean, account.as_ref(), date) {
            let day = date.to_iso();
            match (&planned, planned.role()) {
                // PRD FR-7.58: the bills the amount settles, from the canonical working.
                (Planned::Receipt(_), Some((Role::Buyer, _))) => {
                    let credit = [matching::NewLine {
                        line_id: matching::WHAT_IF_LINE,
                        amount,
                    }];
                    // The usual case reads only the open bills, like the save; a receipt dated
                    // before a later entry, and an edit, are worked from the whole key.
                    let fast = if replacing.is_none() {
                        matching::plan_new_credits(
                            conn,
                            account.id,
                            key.as_deref(),
                            &day,
                            &credit,
                            planned.hint(conn)?.open_bills,
                        )?
                    } else {
                        None
                    };
                    let changed: Vec<(i64, Paisa, Paisa)> = match fast {
                        Some(plan) => {
                            let mut settled = Paisa::ZERO;
                            let mut changed = Vec::new();
                            for bill in &plan.bills {
                                let mut taken = Paisa::ZERO;
                                for row in
                                    plan.rows.iter().filter(|r| r.bill_line_id == bill.line_id)
                                {
                                    taken = taken.checked_add(row.amount).map_err(money_error)?;
                                }
                                settled = settled.checked_add(taken).map_err(money_error)?;
                                if taken != Paisa::ZERO {
                                    changed.push((
                                        bill.line_id,
                                        bill.open,
                                        bill.open.checked_sub(taken).map_err(money_error)?,
                                    ));
                                }
                            }
                            extra_credit = amount.checked_sub(settled).map_err(money_error)?;
                            changed
                        }
                        None => {
                            let inputs = matching::load_key(conn, account.id, key.as_deref())?;
                            let before = matching::key_view(
                                &inputs,
                                &KeyChange {
                                    remove_entry,
                                    add: None,
                                },
                            )?;
                            let after = matching::key_view(
                                &inputs,
                                &KeyChange {
                                    remove_entry,
                                    add: Some(WhatIfLine {
                                        date: &day,
                                        dr: Paisa::ZERO,
                                        cr: amount,
                                        // An edit keeps the place of the voucher it replaces.
                                        order_key: remove_entry
                                            .map(|id| ledger::order_key_of(conn, id))
                                            .transpose()?,
                                    }),
                                },
                            )?;
                            extra_credit = after
                                .unmatched_credit
                                .checked_sub(before.unmatched_credit)
                                .map_err(money_error)?;
                            after
                                .bills
                                .iter()
                                .filter(|b| b.line_id != WHAT_IF_LINE)
                                .filter_map(|b| {
                                    let was =
                                        before.bills.iter().find(|x| x.line_id == b.line_id)?;
                                    (was.open != b.open).then_some((b.line_id, was.open, b.open))
                                })
                                .collect()
                        }
                    };
                    let ids: Vec<i64> = changed.iter().map(|c| c.0).collect();
                    let rows = repo::bill_rows_by_ids(conn, &ids)?;
                    for row in &rows {
                        let Some((_, was, now)) = changed.iter().find(|c| c.0 == row.line_id)
                        else {
                            continue;
                        };
                        bill_effects.push(BillEffect {
                            bill: open_bill(row, *was, today)?,
                            settled_paisa: was.checked_sub(*now).map_err(money_error)?,
                            still_open_paisa: *now,
                        });
                    }
                }
                // PRD FR-15.12: the advances before the credit, then what he owes.
                (Planned::Receipt(_), Some((Role::Seller, _))) => {
                    let check = seller_receipt(
                        conn,
                        &seller_party(&planned)?,
                        date,
                        amount,
                        remove_entry,
                        remove_entry,
                    )?;
                    return_limit = Some(check.limit);
                    owes_settled = amount
                        .checked_sub(check.after.receipt_adjusted)
                        .map_err(money_error)?;
                    let touched: Vec<&matching::AdvanceState> = check
                        .after
                        .advances
                        .iter()
                        .filter(|a| a.adjusted_by_new > Paisa::ZERO)
                        .collect();
                    let entry_ids: Vec<i64> = touched.iter().map(|a| a.entry_id).collect();
                    let numbers = repo::voucher_numbers_of_entries(conn, &entry_ids)?;
                    for after in touched {
                        let before = check
                            .before
                            .advances
                            .iter()
                            .find(|b| b.line_id == after.line_id);
                        let open_before = before.map_or(after.given, |b| b.open);
                        let number = numbers.get(&after.entry_id).copied();
                        advance_effects.push(AdvanceEffect {
                            advance: OpenAdvance {
                                source: (if number.is_some() {
                                    "voucher"
                                } else {
                                    "opening"
                                })
                                .to_owned(),
                                voucher_number: number,
                                entry_date: after.entry_date.clone(),
                                given_paisa: after.given,
                                adjusted_paisa: after
                                    .given
                                    .checked_sub(open_before)
                                    .map_err(money_error)?,
                                open_paisa: open_before,
                            },
                            adjusted_paisa: after.adjusted_by_new,
                            still_open_paisa: after.open,
                        });
                    }
                    if check.exceeds() {
                        errors.push(
                            FieldError::new(ErrorCode::ErrReturnExceedsOpen, "amount")
                                .with("open_paisa", ErrorValue::Number(check.limit.as_paisa())),
                        );
                    }
                }
                // PRD FR-7.59, FR-15.7, FR-7.68: what is payable, and the warning above it.
                (Planned::Payment(_), Some((Role::Seller, is_pukhta))) => {
                    // The totals come from sums, so a seller with years of postings costs a few
                    // milliseconds. An edit takes the old payment out first.
                    let minus = match &replacing {
                        Some(old)
                            if old.row.account.as_ref().map(|a| a.id) == Some(account.id)
                                && old.row.person.as_ref().map(|p| p.person_key.as_str())
                                    == key.as_deref() =>
                        {
                            old.row.amount_paisa
                        }
                        _ => Paisa::ZERO,
                    };
                    let totals = super::reads::seller_totals(
                        conn,
                        account.id,
                        if is_pukhta { key.as_deref() } else { None },
                        minus,
                    )?;
                    let open = totals.open;
                    payable_open = Some(open);
                    payable_is_zero = open == Paisa::ZERO;
                    if amount > open {
                        warnings.push(
                            FieldError::new(ErrorCode::ErrPaymentAbovePayable, "amount")
                                .with("payable_paisa", ErrorValue::Number(open.as_paisa()))
                                .with(
                                    "over_paisa",
                                    ErrorValue::Number(
                                        amount.checked_sub(open).map_err(money_error)?.as_paisa(),
                                    ),
                                ),
                        );
                    }
                }
                // PRD FR-17.8: taking back more than his capital balance warns and still saves.
                (Planned::CapitalOut(_), _) => {
                    let balance = capital_credit(current_balance)?;
                    if let Some(over) = capital_excess(balance, amount).map_err(money_error)? {
                        warnings.push(
                            FieldError::new(ErrorCode::ErrCapitalExceedsBalance, "amount")
                                .with("balance_paisa", ErrorValue::Number(balance.as_paisa()))
                                .with("over_paisa", ErrorValue::Number(over.as_paisa())),
                        );
                    }
                }
                _ => {}
            }
        }

        // PRD FR-6.7: cash after this voucher. An edit also gives back what the old one took.
        let new_cash = if clean {
            cash_change(conn, &planned.entry()?)?
        } else {
            Paisa::ZERO
        };
        let old_cash = match &replacing {
            Some(old) => stored_cash_change(conn, &old.entry)?,
            None => Paisa::ZERO,
        };
        let change = new_cash.checked_sub(old_cash).map_err(money_error)?;
        // An entry that cannot lower cash never asks, so its balance is not read (the sum over the
        // cash account is the slowest part of a preview). One that can lower it reads the balance.
        let cash = if change.is_negative() {
            cash::check(cash_in_hand(conn)?, change)
        } else {
            cash::check_unread(change)
        }
        .map_err(money_error)?;

        Ok(VoucherPreview {
            account,
            amount_paisa: amount,
            current_balance,
            new_balance,
            bill_effects,
            extra_credit_paisa: extra_credit,
            advance_effects,
            owes_settled_paisa: owes_settled,
            return_limit_paisa: return_limit,
            payable_open_paisa: payable_open,
            payable_is_zero_for_seller: payable_is_zero,
            cash,
            errors,
            warnings,
        })
    })
}

/// The credit balance of a capital account from its balance with a side: plus when it stands on
/// the credit side (the partner has capital in the shop), minus on the debit side.
fn capital_credit(balance: Balance) -> Result<Paisa, ServiceError> {
    match balance.side {
        BalanceSide::Cr => Ok(balance.amount_paisa),
        BalanceSide::Dr => balance.amount_paisa.checked_neg().map_err(money_error),
        BalanceSide::Zero => Ok(Paisa::ZERO),
    }
}

/// The party of a planned receipt.
fn seller_party(planned: &Planned) -> Result<super::common::Party, ServiceError> {
    match planned {
        Planned::Receipt(p) => p
            .party
            .clone()
            .ok_or_else(|| ServiceError::Data("a planned receipt has no party".to_owned())),
        _ => Err(ServiceError::Data("not a receipt".to_owned())),
    }
}
