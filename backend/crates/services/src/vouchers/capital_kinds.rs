//! سرمایہ: capital a partner puts in and takes back (PRD 17.4, FR-17.7, FR-17.8, plan 17).
//!
//! Each type keeps its own number series (س- and سو-), its own posting and its own checks (PRD
//! FR-7.67). Capital is never income or expense and is never split. Taking back more than his
//! capital balance warns in the preview and still saves (PRD FR-17.8). The cash question of PRD
//! FR-6.7 is asked for a take-back; money invested brings cash in, so it never asks.

use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::VoucherExtra;
use mandi_domain::capital::{capital_in_entry, capital_out_entry};
use mandi_domain::core::capital::CapitalInput;
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::validate::FieldErrors;
use mandi_domain::core::vouchers::{VoucherSaved, VoucherType};
use mandi_domain::ledger::BalancedEntry;

use super::cash_kinds::{
    PlannedCash, Window, common_fields, record_of_account, resolve_partner, save_with, template,
};
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::ledger;

/// Checks a سرمایہ جمع or سرمایہ واپسی: an open date, an amount above 0 and a partner. Money can
/// only be invested by an active partner; a partner who has left can still take his capital back.
///
/// # Errors
/// `ServiceError::Db` only; the user's mistakes are in the answer.
pub(super) fn plan_capital(
    conn: &Connection,
    window: Window,
    input: &CapitalInput,
    kind: VoucherType,
) -> Result<PlannedCash, ServiceError> {
    let mut errors = FieldErrors::default();
    let money = common_fields(
        conn,
        window,
        &mut errors,
        &input.date,
        &input.amount,
        input.narration.as_deref(),
    )?;
    let partner = resolve_partner(
        conn,
        &mut errors,
        input.partner_id,
        kind == VoucherType::CapitalIn,
    )?;
    let account = match &partner {
        Some(p) => record_of_account(conn, p.capital_account_id)?,
        None => None,
    };
    let errors = errors.finish().err().unwrap_or_default();
    let (mut entry, mut row) = (None, None);
    if let (true, Some(date), Some(amount), Some(cash), Some(p)) = (
        errors.is_empty(),
        money.date,
        money.amount,
        money.pay_account,
        partner.as_ref(),
    ) {
        let (capital, cash) = (AccountId::new(p.capital_account_id), AccountId::new(cash));
        let built: BalancedEntry = if kind == VoucherType::CapitalIn {
            capital_in_entry(cash, capital, amount)
        } else {
            capital_out_entry(capital, cash, amount)
        }
        .map_err(ledger::ledger_error)?;
        entry = Some(built);
        row = Some(template(
            kind,
            date,
            amount,
            p.capital_account_id,
            money.narration.clone(),
            None,
            VoucherExtra {
                partner_id: Some(p.id),
                ..VoucherExtra::default()
            },
        ));
    }
    Ok(PlannedCash {
        kind,
        errors,
        date: money.date,
        amount: money.amount,
        account,
        head: None,
        partner,
        paid_to: None,
        entry,
        template: row,
    })
}

/// Saves capital invested (PRD 17.4, FR-17.7): روکڑ بنام, the partner's capital account (0602) جمع.
/// Never split, never income. Example (PRD E2E-35): ب invests 50,000 on 11-10-26: cash 108,484.
///
/// # Errors
/// The date codes on `date`; the money codes on `amount`; `ERR_PARTNER_REQUIRED`, `ERR_NOT_FOUND`
/// and `ERR_ACCOUNT_INACTIVE` on `partner_id`; the text code on `narration`. Nothing is written
/// when any of them is returned.
pub fn save_capital_in(
    ctx: &Context,
    actor: &Actor,
    input: &CapitalInput,
) -> Result<VoucherSaved, ServiceError> {
    // Money coming in cannot take cash below 0, so the confirm of the input is not read.
    save_with(ctx, actor, false, |conn, window| {
        plan_capital(conn, window, input, VoucherType::CapitalIn)
    })
}

/// Saves capital taken back (PRD 17.4, FR-17.7): the capital account بنام, روکڑ جمع. More than his
/// balance saves; the preview warns with `ERR_CAPITAL_EXCEEDS_BALANCE` (PRD FR-17.8).
///
/// # Errors
/// As [`save_capital_in`], plus `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
pub fn save_capital_out(
    ctx: &Context,
    actor: &Actor,
    input: &CapitalInput,
) -> Result<VoucherSaved, ServiceError> {
    save_with(ctx, actor, input.confirm_cash_below_zero, |conn, window| {
        plan_capital(conn, window, input, VoucherType::CapitalOut)
    })
}
