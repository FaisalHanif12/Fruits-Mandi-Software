//! The partner code (PRD FR-17.11, NFR-SEC-14, plan 44): a 4-digit code that opens the partner
//! section.
//!
//! - **Stored as a hash.** The setting `partner_code` holds the Argon2id hash of the code, never the
//!   code. The setting `partner_code_tries` holds the count of wrong tries in a row and the end of
//!   the wait they started.
//! - **Open in memory only.** Like the sign-in, the unlock is kept in the running app. It is lost
//!   when the user leaves the section (`lock`), on the idle lock, on a logout, after a restore and
//!   when the app closes.
//! - **Waits as sign-in** (PRD FR-8.2): after 5 wrong codes 1 minute, after 10 five minutes, then 15
//!   minutes each time. The wait is kept in memory by a monotonic clock and on disk by the wall
//!   clock (NFR-REL-11). Every wrong try is in the audit log, without the code.
//! - **Setting needs the Owner's password**, changing needs the old code or the password, and a
//!   forgotten code is reset with the password or the recovery code (which is not spent).
//! - **No code set** means the section is open: the first visit asks the Owner to set one, and the
//!   first-run partner setup works because no code exists yet.
//!
//! The commands that show partner figures call [`require_unlocked`] (or [`is_locked`] to hide the
//! figures) after the sign-in check (Code Standards S8).
//!
//! Example (PRD E2E-46): the code is 4826. `unlock("1111")` gives `ERR_PARTNER_CODE_WRONG` with
//! `tries_left` 4; `unlock("4826")` opens the section; `lock()` closes it.

use chrono::Duration as ChronoDuration;
use mandi_db::clock::format_utc;
use mandi_db::rusqlite::Connection;
use mandi_domain::auth::lockout::{WrongTry, after_wrong_try};
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::partner_code::{
    PartnerCodeChangeInput, PartnerCodeResetInput, PartnerCodeSetInput, PartnerCodeStatus,
    PartnerCodeUnlockInput,
};
use mandi_domain::core::validate::FieldErrors;
use serde_json::{Value, json};

use crate::audit::Event;
use crate::auth::{Actor, lockout, prove_owner};
use crate::context::Context;
use crate::error::ServiceError;
use crate::secret::Secret;
use crate::store;

/// The setting that holds the hash of the code: `{ "hash": "$argon2id$..." }`.
const KEY_CODE: &str = "partner_code";
/// The setting that holds the wrong tries: `{ "count": 3, "until": "2026-10-10T..." | null }`.
const KEY_TRIES: &str = "partner_code_tries";
/// The key of the wait in the memory of the lockouts. It cannot be a user name.
const LOCK_KEY: &str = "\u{1}partner_code";
/// The digits of a code.
const CODE_DIGITS: usize = 4;

/// True if the text is exactly 4 ASCII digits.
fn is_code(text: &str) -> bool {
    text.len() == CODE_DIGITS && text.bytes().all(|b| b.is_ascii_digit())
}

fn stored_hash(conn: &Connection) -> Result<Option<String>, ServiceError> {
    Ok(store::setting(conn, KEY_CODE)?
        .and_then(|value| value.get("hash").and_then(Value::as_str).map(str::to_owned)))
}

/// The wrong tries in a row and the stored end of their wait.
fn stored_tries(conn: &Connection) -> Result<(u32, Option<String>), ServiceError> {
    let Some(value) = store::setting(conn, KEY_TRIES)? else {
        return Ok((0, None));
    };
    let count = value
        .get("count")
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(0);
    let until = value
        .get("until")
        .and_then(Value::as_str)
        .map(str::to_owned);
    Ok((count, until))
}

fn is_set(ctx: &Context) -> Result<bool, ServiceError> {
    Ok(ctx.db.read(stored_hash)?.is_some())
}

/// Seconds the code still waits, or `None` when it may be tried.
fn wait_seconds(ctx: &Context) -> Result<Option<u64>, ServiceError> {
    let (count, until) = ctx.db.read(stored_tries)?;
    Ok(lockout::remaining_from(
        ctx,
        LOCK_KEY,
        until.as_deref(),
        count,
    ))
}

fn locked_wait(seconds: u64, field: &str) -> ServiceError {
    ServiceError::reject_with(
        ErrorCode::ErrLockedWait,
        Some(field),
        "seconds",
        i64::try_from(seconds).unwrap_or(i64::MAX),
    )
}

/// The status as the screens read it.
fn current_status(ctx: &Context) -> Result<PartnerCodeStatus, ServiceError> {
    let set = is_set(ctx)?;
    let wait = wait_seconds(ctx)?.unwrap_or(0);
    Ok(PartnerCodeStatus {
        is_set: set,
        unlocked: set && ctx.partner_open(),
        wait_seconds: i64::try_from(wait).unwrap_or(i64::MAX),
    })
}

/// True while the partner section is locked: a code is set and was not given since the user
/// entered the section (PRD FR-17.11). With no code set the section is open.
///
/// # Errors
/// `ServiceError::Db`.
pub fn is_locked(ctx: &Context) -> Result<bool, ServiceError> {
    Ok(!ctx.partner_open() && is_set(ctx)?)
}

/// The check every partner command and report starts with, after the sign-in check (Code Standards
/// S8, NFR-SEC-14).
///
/// # Errors
/// `ERR_PARTNER_LOCKED` while the section is locked.
pub fn require_unlocked(ctx: &Context) -> Result<(), ServiceError> {
    if is_locked(ctx)? {
        return Err(ServiceError::reject(ErrorCode::ErrPartnerLocked));
    }
    Ok(())
}

/// Writes one audit row for the code. Never holds the code or the hash (Code Standards S2).
fn audit(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    action: &str,
    detail: &Value,
) -> Result<(), ServiceError> {
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(actor.user_id), action, "partner_code")
            .new_value(detail.clone())
            .into_entry(),
    )?;
    Ok(())
}

/// Counts a wrong try, starts a wait when the count reaches a step, writes it down and returns
/// the error to show.
fn register_wrong_try(ctx: &Context, actor: &Actor, field: &str) -> ServiceError {
    let counted = ctx.db.write(|tx| {
        let (before, _) = stored_tries(tx)?;
        let attempts = before.saturating_add(1);
        let outcome = after_wrong_try(attempts);
        let until = match outcome {
            WrongTry::Locked { seconds } => Some(format_utc(
                ctx.now_utc() + ChronoDuration::seconds(i64::try_from(seconds).unwrap_or(i64::MAX)),
            )),
            WrongTry::TriesLeft(_) => None,
        };
        store::set_setting(
            tx,
            KEY_TRIES,
            &json!({ "count": attempts, "until": until }),
            Some(actor.user_id),
        )?;
        audit(
            ctx,
            tx,
            actor,
            "partner_code_failed",
            &json!({ "reason": "wrong_code", "attempts": attempts }),
        )?;
        Ok::<_, ServiceError>(outcome)
    });
    match counted {
        Err(error) => error,
        Ok(WrongTry::Locked { seconds }) => {
            lockout::start(ctx, LOCK_KEY, seconds);
            locked_wait(seconds, field)
        }
        Ok(WrongTry::TriesLeft(left)) => ServiceError::reject_with(
            ErrorCode::ErrPartnerCodeWrong,
            Some(field),
            "tries_left",
            i64::from(left),
        ),
    }
}

/// Forgets the wrong tries after a right code or a reset.
fn clear_tries(ctx: &Context, tx: &Connection, actor: &Actor) -> Result<(), ServiceError> {
    store::set_setting(
        tx,
        KEY_TRIES,
        &json!({ "count": 0, "until": null }),
        Some(actor.user_id),
    )?;
    lockout::clear(ctx, LOCK_KEY);
    Ok(())
}

/// Checks a typed code against the stored one, with the wait around it.
fn check_code(
    ctx: &Context,
    actor: &Actor,
    typed: &str,
    field: &'static str,
) -> Result<(), ServiceError> {
    if !is_code(typed) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrPartnerCodeInvalid,
            field,
        ));
    }
    let Some(hash) = ctx.db.read(stored_hash)? else {
        return Err(ServiceError::reject(ErrorCode::ErrPartnerCodeNotSet));
    };
    if let Some(seconds) = wait_seconds(ctx)? {
        ctx.db.write(|tx| {
            audit(
                ctx,
                tx,
                actor,
                "partner_code_failed",
                &json!({ "reason": "locked", "seconds": seconds }),
            )
        })?;
        return Err(locked_wait(seconds, field));
    }
    if ctx.hasher.verify(&Secret::new(typed), &hash) {
        ctx.db.write(|tx| clear_tries(ctx, tx, actor))?;
        return Ok(());
    }
    Err(register_wrong_try(ctx, actor, field))
}

/// Checks the new code typed twice.
fn check_new_code(code: &str, code_again: &str) -> Result<(), ServiceError> {
    let mut errors = FieldErrors::default();
    if !is_code(code) {
        errors.push(FieldError::new(ErrorCode::ErrPartnerCodeInvalid, "code"));
    } else if code != code_again {
        errors.push(FieldError::new(
            ErrorCode::ErrPartnerCodesDiffer,
            "code_again",
        ));
    }
    errors.finish()?;
    Ok(())
}

/// Whether a code is set, whether the section is open, and how long to wait.
///
/// # Errors
/// `ServiceError::Db`.
pub fn status(ctx: &Context) -> Result<PartnerCodeStatus, ServiceError> {
    current_status(ctx)
}

/// Writes a new hash, forgets the wrong tries and opens the section.
fn store_code(
    ctx: &Context,
    actor: &Actor,
    code: &str,
    action: &str,
    expect_set: bool,
) -> Result<(), ServiceError> {
    // The slow hashing happens before the write lock is taken.
    let hash = ctx.hasher.hash(&Secret::new(code))?;
    ctx.db.write(|tx| {
        // Two requests at the same moment must not both win.
        if stored_hash(tx)?.is_some() != expect_set {
            return Err(ServiceError::reject(if expect_set {
                ErrorCode::ErrPartnerCodeNotSet
            } else {
                ErrorCode::ErrPartnerCodeAlreadySet
            }));
        }
        store::set_setting(tx, KEY_CODE, &json!({ "hash": hash }), Some(actor.user_id))?;
        clear_tries(ctx, tx, actor)?;
        audit(ctx, tx, actor, action, &json!({}))?;
        Ok::<(), ServiceError>(())
    })?;
    ctx.set_partner_open(true);
    Ok(())
}

/// Sets the first code, proved by the Owner's password. The section is open after it.
///
/// # Errors
/// `ERR_PARTNER_CODE_ALREADY_SET`; `ERR_PARTNER_CODE_INVALID` (`code`); `ERR_PARTNER_CODES_DIFFER`
/// (`code_again`); `ERR_REQUIRED` and `ERR_WRONG_PASSWORD` (`password`, value `tries_left`);
/// `ERR_LOCKED_WAIT` (value `seconds`).
pub fn set(
    ctx: &Context,
    actor: &Actor,
    input: &PartnerCodeSetInput,
) -> Result<PartnerCodeStatus, ServiceError> {
    if is_set(ctx)? {
        return Err(ServiceError::reject(ErrorCode::ErrPartnerCodeAlreadySet));
    }
    check_new_code(&input.code, &input.code_again)?;
    let password = Secret::new(input.password.clone());
    prove_owner(ctx, actor, &password, "password", false)?;
    store_code(ctx, actor, &input.code, "partner_code_set", false)?;
    current_status(ctx)
}

/// Changes the code, proved by the old code or the Owner's password (a 4-digit text is the old
/// code; a password has at least 6 characters). The section is open after it.
///
/// # Errors
/// `ERR_PARTNER_CODE_NOT_SET`; `ERR_PARTNER_CODE_WRONG` (`old_code_or_password`, value
/// `tries_left`) or `ERR_WRONG_PASSWORD`; `ERR_LOCKED_WAIT`; `ERR_PARTNER_CODE_INVALID` (`code`);
/// `ERR_PARTNER_CODES_DIFFER` (`code_again`).
pub fn change(
    ctx: &Context,
    actor: &Actor,
    input: &PartnerCodeChangeInput,
) -> Result<PartnerCodeStatus, ServiceError> {
    if !is_set(ctx)? {
        return Err(ServiceError::reject(ErrorCode::ErrPartnerCodeNotSet));
    }
    check_new_code(&input.code, &input.code_again)?;
    let old = input.old_code_or_password.as_str();
    if is_code(old) {
        check_code(ctx, actor, old, "old_code_or_password")?;
    } else {
        prove_owner(
            ctx,
            actor,
            &Secret::new(old.to_owned()),
            "old_code_or_password",
            false,
        )?;
    }
    store_code(ctx, actor, &input.code, "partner_code_change", true)?;
    current_status(ctx)
}

/// Opens the partner section.
///
/// # Errors
/// `ERR_PARTNER_CODE_NOT_SET`; `ERR_PARTNER_CODE_INVALID` (`code`, not 4 digits);
/// `ERR_PARTNER_CODE_WRONG` (`code`, value `tries_left`); `ERR_LOCKED_WAIT` (value `seconds`) after
/// 5 wrong tries (as PRD FR-8.2).
pub fn unlock(
    ctx: &Context,
    actor: &Actor,
    input: &PartnerCodeUnlockInput,
) -> Result<PartnerCodeStatus, ServiceError> {
    check_code(ctx, actor, &input.code, "code")?;
    ctx.set_partner_open(true);
    ctx.db
        .write(|tx| audit(ctx, tx, actor, "partner_unlock", &json!({})))?;
    current_status(ctx)
}

/// Closes the partner section (leaving it). Safe when it is closed already.
///
/// # Errors
/// `ServiceError::Db`.
pub fn lock(ctx: &Context) -> Result<PartnerCodeStatus, ServiceError> {
    ctx.set_partner_open(false);
    current_status(ctx)
}

/// Sets a new code when the old one is forgotten, proved by the Owner's password or the recovery
/// code (which is not spent). It also ends the wait. The section is open after it.
///
/// # Errors
/// `ERR_PARTNER_CODE_NOT_SET`; `ERR_REQUIRED` and `ERR_WRONG_PASSWORD` (`password_or_recovery`);
/// `ERR_LOCKED_WAIT`; `ERR_PARTNER_CODE_INVALID` (`code`); `ERR_PARTNER_CODES_DIFFER`
/// (`code_again`).
pub fn reset(
    ctx: &Context,
    actor: &Actor,
    input: &PartnerCodeResetInput,
) -> Result<PartnerCodeStatus, ServiceError> {
    if !is_set(ctx)? {
        return Err(ServiceError::reject(ErrorCode::ErrPartnerCodeNotSet));
    }
    check_new_code(&input.code, &input.code_again)?;
    prove_owner(
        ctx,
        actor,
        &Secret::new(input.password_or_recovery.clone()),
        "password_or_recovery",
        true,
    )?;
    store_code(ctx, actor, &input.code, "partner_code_reset", true)?;
    current_status(ctx)
}
