//! A saved voucher as the typed entry of its own type (plan 44 T2, PRD FR-7.70), so the edit
//! screen of any of the 13 types can start from it, like `bikri_open_for_edit` does for a بکری.
//!
//! The entry is built from the stored columns and the stored posting only; nothing is worked out
//! again. Sending it back to `vouchers_edit_preview` is an edit that changes nothing.
//!
//! Example (PRD 4.8): the shop expense of 3,000 for electricity opens as
//! `ShopExpense { date: "10-10-2026", expense_head_id: <بجلی>, amount: "3000", paid_to: None, .. }`.

use mandi_db::ledger as ledger_repo;
use mandi_db::vouchers::{self as repo, VoucherExtra};
use mandi_domain::core::capital::CapitalInput;
use mandi_domain::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::expenses::{PersonalExpenseInput, PersonalReturnInput, ShopExpenseInput};
use mandi_domain::core::general::{GeneralInput, GeneralLineInput};
use mandi_domain::core::khata::{KhataInInput, KhataOutInput};
use mandi_domain::core::lent::{LentReturnMode, MoneyLentInput, MoneyLentReturnInput};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::vouchers::{
    AdvanceInput, PaymentInput, ReceiptInput, RemovedPaymentFields, VoucherEditable, VoucherEntry,
    VoucherGetInput, VoucherRow, VoucherType,
};

use crate::context::Context;
use crate::error::{ServiceError, money_error};

/// Whole rupees as the digits a person types.
fn rupees_text(amount: Paisa) -> String {
    amount.rupees().to_string()
}

/// A stored amount of a journal line as digits, or empty text for 0.
fn line_text(paisa: i64) -> Result<String, ServiceError> {
    if paisa == 0 {
        return Ok(String::new());
    }
    Paisa::try_from_paisa(paisa)
        .map(rupees_text)
        .map_err(money_error)
}

/// `YYYY-MM-DD` as `DD-MM-YYYY`.
fn day_text(iso: &str) -> Result<String, ServiceError> {
    BusinessDate::parse_iso(iso)
        .map(format_ddmmyyyy)
        .ok_or_else(|| ServiceError::Data(format!("a stored date is not a date: {iso}")))
}

/// Text that is empty means nothing was typed.
fn optional(text: &str) -> Option<String> {
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// One voucher as the typed entry of its type. Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ERR_VOUCHER_CANCELLED` on `id` for a cancelled voucher (it cannot be
/// edited).
pub fn open_for_edit(
    ctx: &Context,
    input: &VoucherGetInput,
) -> Result<VoucherEditable, ServiceError> {
    let row = super::get(ctx, input.id)?;
    if row.status != "saved" {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVoucherCancelled,
            "id",
        ));
    }
    let (extra, stored) = ctx.db.read(|conn| {
        let extra = repo::extras(conn, row.id)?;
        let stored = match row.entry_id {
            Some(entry_id) => ledger_repo::get_entry(conn, entry_id)?,
            None => None,
        };
        Ok::<_, ServiceError>((extra, stored))
    })?;
    let entry = entry_of(&row, &extra, stored.as_ref())?;
    Ok(VoucherEditable { row, entry })
}

/// The typed entry of a stored voucher.
fn entry_of(
    row: &VoucherRow,
    extra: &VoucherExtra,
    stored: Option<&ledger_repo::StoredEntry>,
) -> Result<VoucherEntry, ServiceError> {
    let date = day_text(&row.entry_date)?;
    let amount = rupees_text(row.amount_paisa);
    let narration = optional(&row.narration);
    let account_id = row.account.as_ref().map(|a| a.id);
    let return_by = extra.return_by_date.as_deref().map(day_text).transpose()?;
    let need_account = || {
        account_id.ok_or_else(|| ServiceError::Data(format!("voucher {} has no account", row.id)))
    };
    Ok(match row.voucher_type {
        VoucherType::Receipt => VoucherEntry::Receipt(ReceiptInput {
            date,
            account_id: need_account()?,
            amount,
            removed: RemovedPaymentFields::default(),
            collector_staff_id: row.collector_staff_id,
            narration: row.narration.clone(),
        }),
        VoucherType::Payment => VoucherEntry::Payment(PaymentInput {
            date,
            account_id: need_account()?,
            amount,
            removed: RemovedPaymentFields::default(),
            reason: row.reason.clone(),
            narration: row.narration.clone(),
            confirm_cash_below_zero: false,
        }),
        VoucherType::Advance => VoucherEntry::Advance(AdvanceInput {
            date,
            seller_account_id: need_account()?,
            amount,
            removed: RemovedPaymentFields::default(),
            expected_goods: row.expected_goods.clone(),
            narration: row.narration.clone(),
            confirm_cash_below_zero: false,
        }),
        // A shop expense has no reason of its own: the column keeps "paid to" (PRD 16.2).
        VoucherType::ShopExpense => VoucherEntry::ShopExpense(ShopExpenseInput {
            date,
            expense_head_id: extra.expense_head_id,
            amount,
            paid_to: optional(&row.reason),
            narration,
            confirm_cash_below_zero: false,
        }),
        VoucherType::PersonalExpense => VoucherEntry::PersonalExpense(PersonalExpenseInput {
            date,
            partner_id: extra.partner_id,
            amount,
            reason: row.reason.clone(),
            return_by,
            taken_by: extra.taken_by.clone(),
            narration,
            confirm_cash_below_zero: false,
        }),
        VoucherType::PersonalReturn => VoucherEntry::PersonalReturn(PersonalReturnInput {
            date,
            partner_id: extra.partner_id,
            amount,
            narration,
        }),
        VoucherType::MoneyLent => VoucherEntry::MoneyLent(MoneyLentInput {
            date,
            person_account_id: account_id,
            amount,
            reason: row.reason.clone(),
            return_by,
            narration,
            confirm_cash_below_zero: false,
        }),
        VoucherType::MoneyLentReturn => VoucherEntry::MoneyLentReturn(MoneyLentReturnInput {
            date,
            person_account_id: account_id,
            amount,
            mode: if extra.salary {
                LentReturnMode::Salary
            } else {
                LentReturnMode::Cash
            },
            salary_head_id: extra.salary.then_some(extra.expense_head_id).flatten(),
            narration,
        }),
        VoucherType::CapitalIn | VoucherType::CapitalOut => {
            let input = CapitalInput {
                date,
                partner_id: extra.partner_id,
                amount,
                narration,
                confirm_cash_below_zero: false,
            };
            if row.voucher_type == VoucherType::CapitalIn {
                VoucherEntry::CapitalIn(input)
            } else {
                VoucherEntry::CapitalOut(input)
            }
        }
        VoucherType::KhataOut => VoucherEntry::KhataOut(KhataOutInput {
            date,
            khata_account_id: account_id,
            amount,
            reason: row.reason.clone(),
            narration,
            confirm_cash_below_zero: false,
        }),
        VoucherType::KhataIn => VoucherEntry::KhataIn(KhataInInput {
            date,
            khata_account_id: account_id,
            amount,
            reason: row.reason.clone(),
            narration,
            confirm_cash_below_zero: false,
        }),
        VoucherType::General => {
            let stored = stored.ok_or_else(|| {
                ServiceError::Data(format!("the posting of voucher {} is missing", row.id))
            })?;
            let lines = stored
                .lines
                .iter()
                .map(|l| {
                    Ok(GeneralLineInput {
                        account_id: Some(l.account_id),
                        dr: line_text(l.dr_paisa)?,
                        cr: line_text(l.cr_paisa)?,
                        narration: l.narration.clone(),
                    })
                })
                .collect::<Result<Vec<_>, ServiceError>>()?;
            VoucherEntry::General(GeneralInput {
                date,
                narration,
                lines,
                confirm_cash_below_zero: false,
            })
        }
    })
}
