//! Change of the Owner's password from the user menu (PRD FR-5.6, NFR-SEC-1, NFR-SEC-2,
//! plan 04 Section 9).
//!
//! The Owner proves the current password first. A wrong one counts as a wrong try of his name,
//! with the same lockout as sign-in (PRD FR-8.2), so the user menu is no way round the lock.
//! The audit row says that the password changed and never holds a password or a hash (S2).

use mandi_db::rusqlite::params;
use mandi_domain::auth::validate as rules;
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::validate::FieldErrors;

use super::login::{Via, attempt, verify_password};
use super::{Actor, ChangePasswordRaw, lockout};
use crate::audit::Event;
use crate::context::Context;
use crate::error::ServiceError;
use crate::store;

/// Changes the signed-in Owner's password.
///
/// `actor` comes from `auth::require`, which the command has already called, so the sign-in
/// check came first. The checks run in this order, and only the third counts as a try:
/// 1. the shape of the three fields (nothing is counted);
/// 2. the lock of his name (a locked name is refused without looking at the password);
/// 3. the current password, with Argon2id.
///
/// # Errors
/// `ERR_REQUIRED` (`current`); `ERR_PASSWORD_TOO_SHORT` (`new`, min 6); `ERR_PASSWORDS_DIFFER`
/// (`repeat`); `ERR_WRONG_PASSWORD` (`current`, value `tries_left`); `ERR_LOCKED_WAIT`
/// (value `seconds`; the 5th wrong try already returns it).
pub fn change_password(
    ctx: &Context,
    actor: &Actor,
    input: &ChangePasswordRaw,
) -> Result<(), ServiceError> {
    let mut errors = FieldErrors::default();
    if input.current.expose().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrRequired, "current"));
    }
    rules::password_pair(
        &mut errors,
        "new",
        "repeat",
        input.new.expose(),
        input.repeat.expose(),
    );
    errors.finish()?;

    let user = ctx
        .db
        .read(|conn| store::user_by_id(conn, actor.user_id))?
        .filter(|user| user.is_active)
        .ok_or_else(|| ServiceError::reject(ErrorCode::ErrNotLoggedIn))?;
    let key = lockout::key_for(&user.username);
    attempt(
        ctx,
        &key,
        &user.username,
        Some(&user),
        Via::ChangePassword,
        || verify_password(ctx, Some(&user), &input.current),
    )?;

    // The slow hashing happens before the write lock is taken.
    let new_hash = ctx.hasher.hash(&input.new)?;
    ctx.db.write(|tx| {
        // The password that was checked must still be the stored one: two changes at the same
        // moment must not both win.
        let stored: String = tx.query_row(
            "SELECT password_hash FROM users WHERE id = ?1",
            [user.id],
            |row| row.get(0),
        )?;
        if stored != user.password_hash {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrWrongPassword,
                "current",
            ));
        }
        tx.execute(
            "UPDATE users SET password_hash = ?1, failed_attempts = 0, locked_until_at = NULL,
                 updated_at = ?2, updated_by = ?3, version = version + 1 WHERE id = ?3",
            params![new_hash, ctx.now_text(), user.id],
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user.id), "change_password", "user")
                .entity_id(user.id)
                .into_entry(),
        )?;
        Ok::<(), ServiceError>(())
    })?;
    lockout::clear(ctx, &key);
    Ok(())
}
