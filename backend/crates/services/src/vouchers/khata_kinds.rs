//! الگ کھاتہ: money paid out for a separate khata and money received for it (PRD 16.6, FR-16.6,
//! E54, plan 17).
//!
//! Each type keeps its own number series (کد- and کا-), its own posting and its own checks (PRD
//! FR-7.67). A khata is never income or expense, never split and never in a partner's total, so
//! its postings touch only the khata's account (control 0305) and cash. A reason is required for
//! both types. The cash question of PRD FR-6.7 is asked for money paid out.

use mandi_db::accounts as account_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::VoucherExtra;
use mandi_domain::core::accounts::{KHATA_CONTROL, RecordRef};
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::khata::{KhataInInput, KhataOutInput};
use mandi_domain::core::validate::FieldErrors;
use mandi_domain::core::vouchers::{VoucherSaved, VoucherType};
use mandi_domain::khata::{khata_in_entry, khata_out_entry};

use super::cash_kinds::{
    PlannedCash, Window, common_fields, record_of_account, required_reason, save_with, template,
};
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::ledger;

/// The fields both types share, as typed.
struct Fields<'a> {
    date: &'a str,
    khata_account_id: Option<i64>,
    amount: &'a str,
    reason: &'a str,
    narration: Option<&'a str>,
}

/// The khata of an entry: it exists, it is under control 0305 and it is active.
fn resolve_khata(
    conn: &Connection,
    errors: &mut FieldErrors,
    id: Option<i64>,
) -> Result<Option<RecordRef>, ServiceError> {
    let field = "khata_account_id";
    let Some(id) = id else {
        errors.push(FieldError::new(ErrorCode::ErrKhataRequired, field));
        return Ok(None);
    };
    let Some(facts) = account_repo::facts(conn, id)? else {
        errors.push(FieldError::new(ErrorCode::ErrNotFound, field));
        return Ok(None);
    };
    if facts.control_code != KHATA_CONTROL {
        errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, field));
        return Ok(None);
    }
    if !facts.is_active {
        errors.push(FieldError::new(ErrorCode::ErrAccountInactive, field));
        return Ok(None);
    }
    record_of_account(conn, id)
}

fn plan_khata(
    conn: &Connection,
    window: Window,
    fields: &Fields<'_>,
    kind: VoucherType,
) -> Result<PlannedCash, ServiceError> {
    let mut errors = FieldErrors::default();
    let money = common_fields(
        conn,
        window,
        &mut errors,
        fields.date,
        fields.amount,
        fields.narration,
    )?;
    let account = resolve_khata(conn, &mut errors, fields.khata_account_id)?;
    let reason = required_reason(&mut errors, fields.reason);
    let errors = errors.finish().err().unwrap_or_default();
    let (mut entry, mut row) = (None, None);
    if let (true, Some(date), Some(amount), Some(cash), Some(khata)) = (
        errors.is_empty(),
        money.date,
        money.amount,
        money.pay_account,
        account.as_ref(),
    ) {
        let (khata_id, cash) = (AccountId::new(khata.id), AccountId::new(cash));
        entry = Some(
            if kind == VoucherType::KhataOut {
                khata_out_entry(khata_id, cash, amount)
            } else {
                khata_in_entry(cash, khata_id, amount)
            }
            .map_err(ledger::ledger_error)?,
        );
        row = Some(template(
            kind,
            date,
            amount,
            khata.id,
            money.narration.clone(),
            reason,
            VoucherExtra::default(),
        ));
    }
    Ok(PlannedCash {
        kind,
        errors,
        date: money.date,
        amount: money.amount,
        account,
        head: None,
        partner: None,
        paid_to: None,
        entry,
        template: row,
    })
}

/// Checks a رقم دی (money paid out for a khata).
///
/// # Errors
/// `ServiceError::Db` only; the user's mistakes are in the answer.
pub(super) fn plan_khata_out(
    conn: &Connection,
    window: Window,
    input: &KhataOutInput,
) -> Result<PlannedCash, ServiceError> {
    plan_khata(
        conn,
        window,
        &Fields {
            date: &input.date,
            khata_account_id: input.khata_account_id,
            amount: &input.amount,
            reason: &input.reason,
            narration: input.narration.as_deref(),
        },
        VoucherType::KhataOut,
    )
}

/// Checks a رقم آئی (money received for a khata).
///
/// # Errors
/// `ServiceError::Db` only; the user's mistakes are in the answer.
pub(super) fn plan_khata_in(
    conn: &Connection,
    window: Window,
    input: &KhataInInput,
) -> Result<PlannedCash, ServiceError> {
    plan_khata(
        conn,
        window,
        &Fields {
            date: &input.date,
            khata_account_id: input.khata_account_id,
            amount: &input.amount,
            reason: &input.reason,
            narration: input.narration.as_deref(),
        },
        VoucherType::KhataIn,
    )
}

/// Saves money paid out for a khata (PRD 16.6): the khata بنام (minus), روکڑ جمع. Example (PRD
/// E2E-38): the family khata pays out 10,000 on 10-10-26: it shows -10,000 and cash is 40,000.
///
/// # Errors
/// The date and money codes; `ERR_KHATA_REQUIRED`, `ERR_NOT_FOUND`, `ERR_FORMAT_INVALID` and
/// `ERR_ACCOUNT_INACTIVE` on `khata_account_id`; `ERR_REASON_REQUIRED` on `reason`; the text code
/// on `narration`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
pub fn save_khata_out(
    ctx: &Context,
    actor: &Actor,
    input: &KhataOutInput,
) -> Result<VoucherSaved, ServiceError> {
    save_with(ctx, actor, input.confirm_cash_below_zero, |conn, window| {
        plan_khata_out(conn, window, input)
    })
}

/// Saves money received for a khata (PRD 16.6): روکڑ بنام, the khata جمع (plus). Cash comes in, so
/// it never asks the cash question.
///
/// # Errors
/// As [`save_khata_out`] without the cash confirm.
pub fn save_khata_in(
    ctx: &Context,
    actor: &Actor,
    input: &KhataInInput,
) -> Result<VoucherSaved, ServiceError> {
    save_with(ctx, actor, false, |conn, window| {
        plan_khata_in(conn, window, input)
    })
}
