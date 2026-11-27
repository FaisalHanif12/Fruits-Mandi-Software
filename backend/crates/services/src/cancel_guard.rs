//! The guard of every cancel (PRD FR-7.93 v1.61, plan 100 C) and of the 10-second undo (PRD
//! FR-7.47).
//!
//! A cancel reverses every figure an entry made. Before any reversal this checks, in one call:
//!
//! 1. a reason, the Owner's sign and the 4-digit partner code are all there (all three errors come
//!    back together);
//! 2. a partner code is set at all, and the code is not waiting after too many wrong tries;
//! 3. the sign is the Owner's (PRD FR-21.4) and the code is right, with the code's wait (PRD
//!    FR-17.11). Both are checked, so both wrong fields show at once.
//!
//! Only then does it call the reversal, which keeps all its own checks. A wrong value changes no
//! book row: only the wrong-try count and the audit log, never with the typed secret. Giving the
//! code here does not open the partner section.
//!
//! The undo of a save needs none of them, but it is only for the moment straight after the save, in
//! the same session (PRD FR-7.47): the app remembers what it just saved, for [`UNDO_WINDOW_SECONDS`]
//! counted from the save's commit (the 10 seconds on screen and a little slack). A cash question
//! before the save does not shorten it. The time is checked on the first undo request: when that
//! request came in time and stops for the cash question of the undo itself (PRD FR-6.7), the
//! confirmed retry is taken for [`UNDO_CONFIRM_SECONDS`] from that first request. Without this,
//! the undo would be a cancel of any old entry with no code and no sign.
//!
//! Example: to cancel وصولی 1 the Owner types "غلط رقم", "adnan" and 4826. With the code 1111 the
//! answer is `ERR_PARTNER_CODE_WRONG` on `partner_code` and the receipt stays saved.

use std::collections::BTreeSet;
use std::time::Duration;

use mandi_domain::core::arrivals::{ArrivalCancelled, ArrivalWriteOffUndone};
use mandi_domain::core::bikri::{
    BikriCancelled, MixedUndoInput, MixedUndoResult, UndoInput, UndoResult, WriteOffUndone,
};
use mandi_domain::core::cancel::{
    ArrivalCancelRequest, ArrivalWriteOffUndoRequest, BikriCancelRequest, BikriWriteOffUndoRequest,
    CancelProof, FruitCancelRequest, GardenEntryCancelRequest, GardenReopenRequest,
    PurchaseCancelRequest, PurchaseWriteOffUndoRequest, TripEntryCancelRequest,
    VoucherCancelRequest,
};
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::fruit::FruitCancelled;
use mandi_domain::core::gardens::{GardenEntryCancelled, GardenReopened};
use mandi_domain::core::purchases::{PurchaseCancelled, PurchaseWriteOffUndone};
use mandi_domain::core::trips::TripEntryCancelled;
use mandi_domain::core::validate::FieldErrors;
use mandi_domain::core::vouchers::VoucherCancelled;
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::{Rejection, ServiceError};
use crate::{bikri, fruit, gardens, owner_sign, partner_code, purchases, trips, vouchers};

/// The field of the code in every cancel request.
const CODE_FIELD: &str = "partner_code";
/// The field of the sign.
const SIGN_FIELD: &str = "sign";
/// The audit action of a wrong sign on a cancel. The entry holds nothing typed.
pub const SIGN_WRONG_ACTION: &str = "cancel_sign_wrong";

/// How long after a save its undo is taken: the 10 seconds on screen and some slack (PRD FR-7.47).
pub const UNDO_WINDOW_SECONDS: u64 = 15;
/// How long the confirmed retry of an undo is taken, from its first request.
pub const UNDO_CONFIRM_SECONDS: u64 = 120;
/// How many saves are remembered for an undo.
const MAX_TICKETS: usize = 20;

/// What a save made, to match its undo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Saved {
    /// One new بکری.
    Bikri(i64),
    /// A mixed save: the carried postings and the new بکری, if it made one.
    Mixed(BTreeSet<i64>, Option<i64>),
}

/// One save that may still be undone.
#[derive(Debug, Clone)]
pub(crate) struct UndoTicket {
    saved: Saved,
    /// Monotonic time until which an undo request is taken.
    until: Duration,
    /// Set when a request in time stopped for the cash question: until then the confirmed retry
    /// is taken too.
    confirm_until: Option<Duration>,
}

/// Remembers a save, so its undo is taken for a short time (PRD FR-7.47). Called after the save has
/// committed.
pub(crate) fn remember(ctx: &Context, saved: Saved) {
    let until = ctx
        .monotonic
        .elapsed()
        .saturating_add(Duration::from_secs(UNDO_WINDOW_SECONDS));
    let mut tickets = ctx.undo_lock();
    tickets.retain(|t| t.saved != saved);
    tickets.push(UndoTicket {
        saved,
        until,
        confirm_until: None,
    });
    let extra = tickets.len().saturating_sub(MAX_TICKETS);
    tickets.drain(..extra);
}

/// Refuses an undo that is not straight after its save in this session: in time, or the confirmed
/// retry of a request that was in time. Gives the time of this request.
fn require_fresh(
    ctx: &Context,
    saved: &Saved,
    confirmed: bool,
    field: &str,
) -> Result<Duration, ServiceError> {
    let now = ctx.monotonic.elapsed();
    let fresh = ctx.undo_lock().iter().any(|t| {
        t.saved == *saved
            && (now <= t.until || (confirmed && t.confirm_until.is_some_and(|c| now <= c)))
    });
    if fresh {
        Ok(now)
    } else {
        Err(ServiceError::reject_field(ErrorCode::ErrUndoExpired, field))
    }
}

/// After the undo asked at `asked_at`: forgets the ticket when it worked. When a request in time
/// stopped for the cash question, its confirmed retry gets [`UNDO_CONFIRM_SECONDS`] from it; a
/// later request never moves that time. Any other refusal keeps the time it had.
fn after_undo<T>(
    ctx: &Context,
    saved: &Saved,
    asked_at: Duration,
    result: Result<T, ServiceError>,
) -> Result<T, ServiceError> {
    match &result {
        Ok(_) => ctx.undo_lock().retain(|t| t.saved != *saved),
        Err(error) if error.has_code(ErrorCode::ErrCashBelowZeroNotConfirmed) => {
            let confirm_until = asked_at.saturating_add(Duration::from_secs(UNDO_CONFIRM_SECONDS));
            for ticket in ctx.undo_lock().iter_mut() {
                if ticket.saved == *saved
                    && asked_at <= ticket.until
                    && ticket.confirm_until.is_none()
                {
                    ticket.confirm_until = Some(confirm_until);
                }
            }
        }
        Err(_) => {}
    }
    result
}

/// Undoes a بکری straight after its save (PRD FR-7.47). No code, sign or reason.
///
/// # Errors
/// `ERR_UNDO_EXPIRED` on `id` when it is not straight after the save in this session; then as
/// [`bikri::undo`].
pub fn undo_bikri(
    ctx: &Context,
    actor: &Actor,
    input: &UndoInput,
) -> Result<UndoResult, ServiceError> {
    let saved = Saved::Bikri(input.id);
    let asked_at = require_fresh(ctx, &saved, input.confirm_cash_below_zero, "id")?;
    after_undo(ctx, &saved, asked_at, bikri::undo(ctx, actor, input))
}

/// Undoes a mixed save straight after it (PRD FR-7.47). No code, sign or reason.
///
/// # Errors
/// `ERR_UNDO_EXPIRED` on `new_bikri_id`, or on `later_entry_ids` when the save made no new بکری;
/// then as [`bikri::mixed_undo`].
pub fn undo_mixed(
    ctx: &Context,
    actor: &Actor,
    input: &MixedUndoInput,
) -> Result<MixedUndoResult, ServiceError> {
    let saved = Saved::Mixed(
        input.later_entry_ids.iter().copied().collect(),
        input.new_bikri_id,
    );
    let field = if input.new_bikri_id.is_some() {
        "new_bikri_id"
    } else {
        "later_entry_ids"
    };
    let asked_at = require_fresh(ctx, &saved, input.confirm_cash_below_zero, field)?;
    after_undo(ctx, &saved, asked_at, bikri::mixed_undo(ctx, actor, input))
}

/// The three secrets and the reason of one cancel, checked before any reversal. `entity` and
/// `entity_id` are written beside a wrong sign in the audit log.
///
/// # Errors
/// All of the missing ones together: `ERR_REASON_REQUIRED` (`reason`), `ERR_SIGN_REQUIRED`
/// (`sign`), `ERR_REQUIRED` or `ERR_PARTNER_CODE_INVALID` (`partner_code`). Then
/// `ERR_PARTNER_CODE_NOT_SET` (`partner_code`); `ERR_LOCKED_WAIT` (value `seconds`) while the code
/// waits. Then, together, `ERR_SIGN_WRONG` (`sign`) and
/// `ERR_PARTNER_CODE_WRONG` (`partner_code`, value `tries_left`) or `ERR_LOCKED_WAIT`
/// (`partner_code`, value `seconds`).
pub fn confirm(
    ctx: &Context,
    actor: &Actor,
    entity: &str,
    entity_id: i64,
    reason: &str,
    proof: &CancelProof,
) -> Result<(), ServiceError> {
    let code = proof.partner_code.trim();
    let mut errors = FieldErrors::default();
    if reason.trim().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrReasonRequired, "reason"));
    }
    if proof.sign.trim().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrSignRequired, SIGN_FIELD));
    }
    if code.is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrRequired, CODE_FIELD));
    } else if !partner_code::is_code(code) {
        errors.push(FieldError::new(
            ErrorCode::ErrPartnerCodeInvalid,
            CODE_FIELD,
        ));
    }
    errors.finish()?;
    if !partner_code::is_set(ctx)? {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrPartnerCodeNotSet,
            CODE_FIELD,
        ));
    }
    // While the code waits, nothing typed is looked at.
    partner_code::refuse_while_waiting(ctx, actor, CODE_FIELD)?;
    let mut refused: Vec<Rejection> = Vec::new();
    let configured = ctx.db.read(owner_sign::configured)?;
    if owner_sign::accepted(&proof.sign, configured.as_deref()).is_none() {
        note_wrong_sign(ctx, actor, entity, entity_id)?;
        refused.push(FieldError::new(ErrorCode::ErrSignWrong, SIGN_FIELD).into());
    }
    match partner_code::check_code(ctx, actor, code, CODE_FIELD) {
        Ok(()) => {}
        Err(ServiceError::Rejected(mut items)) => refused.append(&mut items),
        Err(other) => return Err(other),
    }
    if refused.is_empty() {
        Ok(())
    } else {
        Err(ServiceError::Rejected(refused))
    }
}

/// One audit entry for a wrong sign, in its own save so it stays when the cancel is refused. It
/// holds nothing typed.
fn note_wrong_sign(
    ctx: &Context,
    actor: &Actor,
    entity: &str,
    entity_id: i64,
) -> Result<(), ServiceError> {
    ctx.db.write(|tx| {
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), SIGN_WRONG_ACTION, entity)
                .entity_id(entity_id)
                .new_value(json!({ "refused": "sign" }))
                .into_entry(),
        )?;
        Ok::<(), ServiceError>(())
    })
}

/// `bikri_cancel` (PRD FR-12.22, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`bikri::cancel`].
pub fn cancel_bikri(
    ctx: &Context,
    actor: &Actor,
    request: &BikriCancelRequest,
) -> Result<BikriCancelled, ServiceError> {
    let input = &request.base;
    confirm(ctx, actor, "bikri", input.id, &input.reason, &request.proof)?;
    bikri::cancel(ctx, actor, input)
}

/// `bikri_write_off_undo` (PRD FR-12.16, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`bikri::write_off_undo`].
pub fn undo_bikri_write_off(
    ctx: &Context,
    actor: &Actor,
    request: &BikriWriteOffUndoRequest,
) -> Result<WriteOffUndone, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "stock_move",
        input.move_id,
        &input.reason,
        &request.proof,
    )?;
    bikri::write_off_undo(ctx, actor, input)
}

/// `vouchers_cancel`: every voucher type (PRD FR-15.3, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`vouchers::cancel`].
pub fn cancel_voucher(
    ctx: &Context,
    actor: &Actor,
    request: &VoucherCancelRequest,
) -> Result<VoucherCancelled, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "voucher",
        input.id,
        &input.reason,
        &request.proof,
    )?;
    vouchers::cancel(ctx, actor, input)
}

/// `fruit_cancel` (PRD FR-16.12, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`fruit::cancel`].
pub fn cancel_fruit(
    ctx: &Context,
    actor: &Actor,
    request: &FruitCancelRequest,
) -> Result<FruitCancelled, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "fruit_take",
        input.id,
        &input.reason,
        &request.proof,
    )?;
    fruit::cancel(ctx, actor, input)
}

/// `purchases_cancel` (PRD FR-12.24, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`purchases::cancel`].
pub fn cancel_purchase(
    ctx: &Context,
    actor: &Actor,
    request: &PurchaseCancelRequest,
) -> Result<PurchaseCancelled, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "purchase",
        input.id,
        &input.reason,
        &request.proof,
    )?;
    purchases::cancel(ctx, actor, input)
}

/// `purchases_write_off_undo` (PRD FR-12.28, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`purchases::write_off_undo`].
pub fn undo_purchase_write_off(
    ctx: &Context,
    actor: &Actor,
    request: &PurchaseWriteOffUndoRequest,
) -> Result<PurchaseWriteOffUndone, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "stock_move",
        input.move_id,
        &input.reason,
        &request.proof,
    )?;
    purchases::write_off_undo(ctx, actor, input)
}

/// `trips_cost_cancel` (PRD FR-12.31, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`trips::cost_cancel`].
pub fn cancel_trip_cost(
    ctx: &Context,
    actor: &Actor,
    request: &TripEntryCancelRequest,
) -> Result<TripEntryCancelled, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "trip_cost",
        input.id,
        &input.reason,
        &request.proof,
    )?;
    trips::cost_cancel(ctx, actor, input)
}

/// `trips_other_sale_cancel` (PRD FR-12.32, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`trips::other_sale_cancel`].
pub fn cancel_trip_other_sale(
    ctx: &Context,
    actor: &Actor,
    request: &TripEntryCancelRequest,
) -> Result<TripEntryCancelled, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "trip_other_sale",
        input.id,
        &input.reason,
        &request.proof,
    )?;
    trips::other_sale_cancel(ctx, actor, input)
}

/// `gardens_entry_cancel` (PRD FR-12.35, FR-12.36, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`gardens::entry_cancel`].
pub fn cancel_garden_entry(
    ctx: &Context,
    actor: &Actor,
    request: &GardenEntryCancelRequest,
) -> Result<GardenEntryCancelled, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "garden_entry",
        input.id,
        &input.reason,
        &request.proof,
    )?;
    gardens::entry_cancel(ctx, actor, input)
}

/// `gardens_arrival_cancel` (PRD FR-12.40, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`gardens::arrivals::cancel`].
pub fn cancel_garden_arrival(
    ctx: &Context,
    actor: &Actor,
    request: &ArrivalCancelRequest,
) -> Result<ArrivalCancelled, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "garden_arrival",
        input.id,
        &input.reason,
        &request.proof,
    )?;
    gardens::arrivals::cancel(ctx, actor, input)
}

/// `gardens_arrival_write_off_undo` (PRD FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`gardens::arrivals::write_off_undo`].
pub fn undo_arrival_write_off(
    ctx: &Context,
    actor: &Actor,
    request: &ArrivalWriteOffUndoRequest,
) -> Result<ArrivalWriteOffUndone, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "stock_move",
        input.move_id,
        &input.reason,
        &request.proof,
    )?;
    gardens::arrivals::write_off_undo(ctx, actor, input)
}

/// `gardens_reopen`: undoes the closing of a garden (PRD FR-12.38, FR-7.93).
///
/// # Errors
/// As [`confirm`], then as [`gardens::reopen`].
pub fn undo_garden_close(
    ctx: &Context,
    actor: &Actor,
    request: &GardenReopenRequest,
) -> Result<GardenReopened, ServiceError> {
    let input = &request.base;
    confirm(
        ctx,
        actor,
        "garden",
        input.id,
        &input.reason,
        &request.proof,
    )?;
    gardens::reopen(ctx, actor, input)
}
