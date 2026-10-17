//! The ادائیگی voucher (PRD 15.3, FR-15.7, FR-15.11, FR-14.2, FR-13.7, plan 15): money paid out.
//!
//! - **Seller**: the seller بنام, cash or bank جمع. A payment above his payable is allowed with a
//!   warning; the extra shows as what he owes the firm, and it is not an advance (PRD FR-15.7).
//! - **Buyer**: cash lent to him (PRD FR-15.11). A reason is required. It becomes one of his
//!   bills for matching, aging and اگراہی, and a credit he has waiting settles it at once.
//! - **A payable account** such as the transporters (کرایہ کھاتہ, PRD FR-14.2).
//!
//! A payment never adjusts an advance. It takes cash out, so the save asks the cash question of
//! PRD FR-6.7.

use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::{self as repo, NewVoucher};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::FieldErrors;
use mandi_domain::core::vouchers::{PaymentInput, VoucherSaved, VoucherType};
use mandi_domain::ledger::{BalancedEntry, LedgerLine, PostingType, Side, build_entry};

use super::common::{
    Accepts, Hint, Money, MoneyText, Party, PostingSpec, Role, cash_change, cash_check, hint_for,
    insert_and_post, optional_text, plan_money, resolve_party, stored_line, voucher_json,
};
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::require_confirmed;
use crate::context::Context;
use crate::day;
use crate::error::ServiceError;
use crate::ledger;
use crate::matching::{self, NewLine};

/// A payment that went through the checks of the form.
#[derive(Debug, Clone)]
pub(super) struct PlannedPayment {
    pub party: Option<Party>,
    pub money: Money,
    pub reason: Option<String>,
    /// What is wrong, at the fields as sent.
    pub errors: Vec<FieldError>,
}

impl PlannedPayment {
    /// The pieces a posting needs, when there is no mistake.
    pub(super) fn parts(&self) -> Result<(&Party, BusinessDate, Paisa, i64), ServiceError> {
        match (
            self.party.as_ref(),
            self.money.date,
            self.money.amount,
            self.money.pay_account,
        ) {
            (Some(party), Some(date), Some(amount), Some(pay)) if self.errors.is_empty() => {
                Ok((party, date, amount, pay))
            }
            _ => Err(ServiceError::Data(
                "a planned payment is not complete".to_owned(),
            )),
        }
    }
}

/// Checks a payment as typed: the form fields, the account and its person, and the reason that
/// cash lent to a buyer needs (PRD FR-15.11).
///
/// # Errors
/// `ServiceError::Db` only.
pub(super) fn plan_payment(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &PaymentInput,
) -> Result<PlannedPayment, ServiceError> {
    let mut errors = FieldErrors::default();
    let money = plan_money(
        conn,
        window,
        &mut errors,
        &MoneyText {
            date: &input.date,
            amount: &input.amount,
            mode: input.mode,
            bank_account_id: input.bank_account_id,
            cheque_no: &input.cheque_no,
            cheque_date: &input.cheque_date,
            narration: &input.narration,
        },
    )?;
    let reason = optional_text(&mut errors, "reason", &input.reason);
    let party = resolve_party(
        conn,
        &mut errors,
        input.account_id,
        input.person_key.as_deref(),
        Accepts {
            buyer: true,
            seller: true,
            payable: true,
            allow_inactive: false,
            refusal: ErrorCode::ErrFormatInvalid,
        },
    )?;
    // The trigger of the database refuses this too (with another code); the service checks first
    // so the screen gets the code with its field.
    if party.as_ref().is_some_and(|p| p.role == Role::Buyer) && reason.is_none() {
        errors.push(FieldError::new(
            ErrorCode::ErrPaymentToBuyerNeedsReason,
            "reason",
        ));
    }
    Ok(PlannedPayment {
        party,
        money,
        reason,
        errors: errors.finish().err().unwrap_or_default(),
    })
}

/// The journal entry of a payment (PRD 15.3): the party بنام, cash or the bank جمع.
///
/// # Errors
/// `ServiceError::Data` if the plan has a mistake; the ledger errors of the entry.
pub(super) fn payment_entry(planned: &PlannedPayment) -> Result<BalancedEntry, ServiceError> {
    let (party, _, amount, pay_account) = planned.parts()?;
    let mut debit = LedgerLine::new(AccountId::new(party.record.id), Side::Dr, amount);
    debit.person_key.clone_from(&party.person_key);
    build_entry(vec![
        debit,
        LedgerLine::new(AccountId::new(pay_account), Side::Cr, amount),
    ])
    .map_err(ledger::ledger_error)
}

/// The voucher row of a payment.
pub(super) fn payment_row(
    planned: &PlannedPayment,
    number: i64,
) -> Result<NewVoucher, ServiceError> {
    let (party, date, amount, _) = planned.parts()?;
    Ok(NewVoucher {
        voucher_type: VoucherType::Payment,
        number,
        entry_date: date.to_iso(),
        account_id: Some(party.record.id),
        amount_paisa: amount.as_paisa(),
        mode: planned.money.mode,
        bank_account_id: planned.money.bank_account_id,
        cheque_no: planned.money.cheque_no.clone(),
        cheque_date: planned.money.cheque_date.map(BusinessDate::to_iso),
        expected_goods: None,
        narration: planned.money.narration.clone(),
        person_key: party.person_key.clone(),
        collector_staff_id: None,
        reason: planned.reason.clone(),
    })
}

/// Matches a posted payment: cash lent is one of the buyer's bills, and a credit he has waiting
/// settles it (PRD FR-15.11). A payment to anyone else has no matching. Fast path: it assumes the
/// rows of the party are canonical.
///
/// # Errors
/// As [`matching::match_new_bills`].
pub(super) fn payment_match(
    tx: &Connection,
    user_id: i64,
    now: &str,
    planned: &PlannedPayment,
    entry_id: i64,
    hint: &Hint,
) -> Result<(), ServiceError> {
    let (party, date, amount, _) = planned.parts()?;
    if party.role == Role::Buyer {
        let debit_line = stored_line(tx, entry_id, party.record.id, false)?;
        matching::match_new_bills_hinted(
            tx,
            user_id,
            now,
            party.record.id,
            party.person_key.as_deref(),
            &date.to_iso(),
            &[NewLine {
                line_id: debit_line,
                amount,
            }],
            hint.unmatched_credit,
        )?;
    }
    Ok(())
}

/// Pays money out (PRD 15.3): see the module text. One transaction: the voucher, its journal
/// entry, the matching rows of a buyer's new bill and the audit row.
///
/// # Errors
/// The field errors of the date, amount, bank and cheque fields; `ERR_NOT_FOUND`,
/// `ERR_FORMAT_INVALID`, `ERR_ACCOUNT_INACTIVE` on `account_id`; `ERR_PERSON_REQUIRED` and
/// `ERR_NOT_FOUND` on `person_key`; `ERR_PAYMENT_TO_BUYER_NEEDS_REASON` on `reason`;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`); `ERR_DATE_CLOSED` and
/// `ERR_DATE_IN_FUTURE` on `date`. Nothing is written when any of them is returned.
pub fn save_payment(
    ctx: &Context,
    actor: &Actor,
    input: &PaymentInput,
) -> Result<VoucherSaved, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.write(|tx| {
        let planned = plan_payment(tx, window, input)?;
        if !planned.errors.is_empty() {
            return Err(planned.errors.into());
        }
        let (party, date, _, _) = planned.parts()?;
        let entry = payment_entry(&planned)?;
        // PRD FR-6.7: a payment takes cash out. If cash would end below 0 the user must confirm.
        require_confirmed(
            &cash_check(tx, cash_change(tx, &entry)?)?,
            input.confirm_cash_below_zero,
        )?;
        let hint = hint_for(tx, party.role, party.record.id, party.person_key.as_deref())?;
        let now = ctx.now_text();
        let number = repo::next_number(tx, VoucherType::Payment)?;
        let row = payment_row(&planned, number)?;
        let (voucher_id, posted) = insert_and_post(
            ctx,
            tx,
            actor.user_id,
            PostingSpec {
                posting_type: PostingType::Payment,
                date,
                narration: row.narration.clone(),
            },
            &row,
            entry,
        )?;
        payment_match(tx, actor.user_id, &now, &planned, posted.entry_id, &hint)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "create", "voucher")
                .entity_id(voucher_id)
                .new_value(voucher_json(number, "payment", &row))
                .into_entry(),
        )?;
        Ok(VoucherSaved {
            voucher_id,
            voucher_type: VoucherType::Payment,
            number,
            entry_id: posted.entry_id,
            new_balance: ledger::balance_in(
                tx,
                party.record.id,
                party.person_key.as_deref(),
                None,
            )?,
        })
    })
}
