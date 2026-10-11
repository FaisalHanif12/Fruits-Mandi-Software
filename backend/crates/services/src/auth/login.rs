//! Sign-in and unlock with the lockout steps of PRD FR-8.2 (plan 04 Section 3).
//!
//! Every try, right or wrong, writes an audit row. A wrong user name behaves exactly like a
//! wrong password: same error, same counting, same time (a dummy hash is checked), so the
//! screen never tells which names exist.

use chrono::Duration as ChronoDuration;
use mandi_db::clock::format_utc;
use mandi_db::rusqlite::params;
use mandi_domain::auth::lockout::{WrongTry, after_wrong_try};
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::validate::FieldErrors;
use serde_json::json;

use super::SessionInfo;
use super::lockout;
use super::session::{self, session_info};
use crate::audit::Event;
use crate::context::Context;
use crate::error::ServiceError;
use crate::secret::Secret;
use crate::settings::KEY_LOGIN_LANGUAGE;
use crate::store::{self, UserRecord};

/// Which door the try came through. It changes the audit text, the error code and the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Via {
    Login,
    Unlock,
    Recover,
}

impl Via {
    fn as_str(self) -> &'static str {
        match self {
            Via::Login => "login",
            Via::Unlock => "unlock",
            Via::Recover => "recover",
        }
    }

    fn wrong_code(self) -> ErrorCode {
        match self {
            Via::Login | Via::Unlock => ErrorCode::ErrWrongPassword,
            Via::Recover => ErrorCode::ErrRecoveryCodeWrong,
        }
    }

    fn field(self) -> &'static str {
        match self {
            Via::Login | Via::Unlock => "password",
            Via::Recover => "recovery_code",
        }
    }
}

/// The most bytes a password can have: 128 characters of up to 4 bytes each.
const MAX_PASSWORD_BYTES: usize = 512;

/// The longest typed name written to the audit log.
const MAX_AUDIT_NAME_CHARS: usize = 64;

fn typed_for_audit(typed: &str) -> String {
    typed.trim().chars().take(MAX_AUDIT_NAME_CHARS).collect()
}

/// Checks one secret (password or recovery code) for one name, with the lockout around it.
///
/// 1. A name that is locked is refused without looking at the secret (and the try is logged).
/// 2. `check` says whether the secret is right. It always does the same work, known user or not.
/// 3. A wrong secret counts a try and may start a lock.
pub(super) fn attempt(
    ctx: &Context,
    key: &str,
    typed: &str,
    user: Option<&UserRecord>,
    via: Via,
    check: impl FnOnce() -> Result<bool, ServiceError>,
) -> Result<(), ServiceError> {
    if let Some(seconds) = lockout::remaining_seconds(ctx, key, user) {
        record_failure(
            ctx,
            user,
            typed,
            via,
            &json!({ "reason": "locked", "seconds": seconds }),
        )?;
        return Err(locked(seconds));
    }
    if check()? {
        return Ok(());
    }
    Err(register_wrong_try(ctx, key, typed, user, via))
}

fn locked(seconds: u64) -> ServiceError {
    ServiceError::reject_with(
        ErrorCode::ErrLockedWait,
        None,
        "seconds",
        i64::try_from(seconds).unwrap_or(i64::MAX),
    )
}

/// An audit row for a try that did not get in.
fn record_failure(
    ctx: &Context,
    user: Option<&UserRecord>,
    typed: &str,
    via: Via,
    detail: &serde_json::Value,
) -> Result<(), ServiceError> {
    ctx.db.write(|tx| {
        write_failure(ctx, tx, user, typed, via, detail)?;
        Ok(())
    })
}

fn write_failure(
    ctx: &Context,
    tx: &mandi_db::rusqlite::Connection,
    user: Option<&UserRecord>,
    typed: &str,
    via: Via,
    detail: &serde_json::Value,
) -> Result<(), ServiceError> {
    let mut body = detail.clone();
    if let Some(map) = body.as_object_mut() {
        map.insert("via".to_owned(), json!(via.as_str()));
        if user.is_none() {
            // An unknown name: keep what was typed, as the plan asks.
            map.insert("username".to_owned(), json!(typed_for_audit(typed)));
        }
    }
    let mut event = Event::new(user.map(|u| u.id), "login_failed", "user").new_value(body);
    if let Some(user) = user {
        event = event.entity_id(user.id);
    }
    ctx.db.auditor().write(tx, &event.into_entry())?;
    Ok(())
}

/// Counts a wrong try for this name, writes it down, starts a lock when the count reaches a
/// step, and returns the error the screen shows.
fn register_wrong_try(
    ctx: &Context,
    key: &str,
    typed: &str,
    user: Option<&UserRecord>,
    via: Via,
) -> ServiceError {
    let counted = match user {
        Some(user) => ctx.db.write(|tx| {
            let before: u32 = tx.query_row(
                "SELECT failed_attempts FROM users WHERE id = ?1",
                [user.id],
                |row| row.get(0),
            )?;
            let attempts = before.saturating_add(1);
            let outcome = after_wrong_try(attempts);
            let locked_until = match outcome {
                WrongTry::Locked { seconds } => Some(format_utc(
                    ctx.now_utc() + ChronoDuration::seconds(i64::try_from(seconds).unwrap_or(i64::MAX)),
                )),
                WrongTry::TriesLeft(_) => None,
            };
            tx.execute(
                "UPDATE users SET failed_attempts = ?1, locked_until_at = COALESCE(?2, locked_until_at),
                     updated_at = ?3, version = version + 1 WHERE id = ?4",
                params![attempts, locked_until, ctx.now_text(), user.id],
            )?;
            write_failure(
                ctx,
                tx,
                Some(user),
                typed,
                via,
                &json!({ "reason": "wrong_secret", "attempts": attempts }),
            )?;
            Ok::<_, ServiceError>(outcome)
        }),
        None => {
            let attempts = lockout::count_unknown(ctx, key);
            let outcome = after_wrong_try(attempts);
            ctx.db
                .write(|tx| {
                    write_failure(
                        ctx,
                        tx,
                        None,
                        typed,
                        via,
                        &json!({ "reason": "wrong_secret", "attempts": attempts }),
                    )?;
                    Ok::<_, ServiceError>(())
                })
                .map(|()| outcome)
        }
    };
    match counted {
        Err(error) => error,
        Ok(WrongTry::Locked { seconds }) => {
            lockout::start(ctx, key, seconds);
            locked(seconds)
        }
        Ok(WrongTry::TriesLeft(left)) => ServiceError::reject_with(
            via.wrong_code(),
            Some(via.field()),
            "tries_left",
            i64::from(left),
        ),
    }
}

/// Checks a password against a user's hash, or against a dummy hash when there is no such user.
pub(super) fn verify_password(
    ctx: &Context,
    user: Option<&UserRecord>,
    password: &Secret,
) -> Result<bool, ServiceError> {
    // No password can be longer than this (4 bytes per character at most). Hashing a huge text
    // would only waste time, so it is simply wrong.
    if password.expose().len() > MAX_PASSWORD_BYTES {
        return Ok(false);
    }
    match user {
        Some(user) => Ok(ctx.hasher.verify(password, &user.password_hash)),
        None => {
            ctx.hasher.verify(password, ctx.dummy_hash()?);
            Ok(false)
        }
    }
}

/// A right password for an inactive user is refused (and logged).
fn refuse_inactive(ctx: &Context, user: &UserRecord, via: Via) -> ServiceError {
    match record_failure(
        ctx,
        Some(user),
        &user.username,
        via,
        &json!({ "reason": "inactive" }),
    ) {
        Ok(()) => ServiceError::reject(ErrorCode::ErrUserInactive),
        Err(error) => error,
    }
}

/// Signs in (PRD FR-8.1 to FR-8.6).
///
/// # Errors
/// `ERR_REQUIRED` (`username`, `password`); `ERR_WRONG_PASSWORD` (`password`, `tries_left`);
/// `ERR_LOCKED_WAIT` (`seconds`); `ERR_USER_INACTIVE`; `ERR_ALREADY_SIGNED_IN` if someone is
/// already signed in (PRD FR-2.3).
pub fn login(
    ctx: &Context,
    username: &str,
    password: &Secret,
) -> Result<SessionInfo, ServiceError> {
    if ctx.session_snapshot().is_some() {
        return Err(ServiceError::reject(ErrorCode::ErrAlreadySignedIn));
    }
    let mut errors = FieldErrors::default();
    if username.trim().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrRequired, "username"));
    }
    if password.expose().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrRequired, "password"));
    }
    errors.finish()?;

    let key = lockout::key_for(username);
    let user = ctx.db.read(|conn| store::user_by_username(conn, &key))?;
    attempt(ctx, &key, username, user.as_ref(), Via::Login, || {
        verify_password(ctx, user.as_ref(), password)
    })?;
    // A right password means the user exists.
    let Some(user) = user else {
        return Err(ServiceError::Data(
            "sign-in passed for an unknown user".to_owned(),
        ));
    };
    if !user.is_active {
        return Err(refuse_inactive(ctx, &user, Via::Login));
    }

    ctx.db.write(|tx| {
        let now = ctx.now_text();
        tx.execute(
            "UPDATE users SET failed_attempts = 0, locked_until_at = NULL, last_login_at = ?1,
                 updated_at = ?1, updated_by = ?2, version = version + 1 WHERE id = ?2",
            params![now, user.id],
        )?;
        // The sign-in screen of this computer follows the last user's language (PRD FR-7.30).
        let language = session_info(tx, &user)?.language;
        store::set_setting(
            tx,
            KEY_LOGIN_LANGUAGE,
            &json!(language.as_str()),
            Some(user.id),
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user.id), "login", "user")
                .entity_id(user.id)
                .into_entry(),
        )?;
        Ok::<(), ServiceError>(())
    })?;
    lockout::clear(ctx, &key);
    session::open(ctx, &user);
    ctx.db.read(|conn| session_info(conn, &user))
}

/// Unlocks the locked session with the user's password. Uses the same wrong-try counter as
/// sign-in (plan 04 Section 4).
///
/// # Errors
/// As [`login`]; `ERR_NOT_LOGGED_IN` without a session; `ERR_NOT_LOCKED` if it is not locked.
pub fn unlock(ctx: &Context, password: &Secret) -> Result<SessionInfo, ServiceError> {
    let Some(open) = ctx.session_snapshot() else {
        return Err(ServiceError::reject(ErrorCode::ErrNotLoggedIn));
    };
    if !open.locked {
        return Err(ServiceError::reject(ErrorCode::ErrNotLocked));
    }
    if password.expose().is_empty() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrRequired,
            "password",
        ));
    }
    let user = ctx.db.read(|conn| store::user_by_id(conn, open.user_id))?;
    let Some(user) = user else {
        *ctx.session_lock() = None;
        return Err(ServiceError::reject(ErrorCode::ErrNotLoggedIn));
    };
    let key = lockout::key_for(&user.username);
    attempt(ctx, &key, &user.username, Some(&user), Via::Unlock, || {
        verify_password(ctx, Some(&user), password)
    })?;
    if !user.is_active {
        *ctx.session_lock() = None;
        return Err(refuse_inactive(ctx, &user, Via::Unlock));
    }
    ctx.db.write(|tx| {
        tx.execute(
            "UPDATE users SET failed_attempts = 0, locked_until_at = NULL, updated_at = ?1,
                 updated_by = ?2, version = version + 1 WHERE id = ?2",
            params![ctx.now_text(), user.id],
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user.id), "unlock", "user")
                .entity_id(user.id)
                .into_entry(),
        )?;
        Ok::<(), ServiceError>(())
    })?;
    lockout::clear(ctx, &key);
    if let Some(session) = ctx.session_lock().as_mut() {
        session.locked = false;
    }
    ctx.db.read(|conn| session_info(conn, &user))
}
