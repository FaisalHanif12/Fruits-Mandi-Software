//! Proof that the person at the computer is the Owner, for a step that needs more than the
//! sign-in: setting, changing or resetting the partner code (PRD FR-17.11, plan 44).
//!
//! A wrong try counts as a wrong try of his name, with the same lockout as sign-in (PRD FR-8.2), so
//! these steps are no way round the lock. Every wrong try is in the audit log as a failed sign-in
//! with the door `partner_code`.

use mandi_domain::auth::recovery;
use mandi_domain::core::error::{ErrorCode, FieldError};

use super::firstrun::stored_recovery;
use super::login::{Via, attempt, verify_password};
use super::{Actor, lockout};
use crate::context::Context;
use crate::error::ServiceError;
use crate::secret::Secret;
use crate::store;

/// Checks the Owner's password (and, when `allow_recovery`, his printed recovery code, which is
/// NOT spent here) typed in `field`.
///
/// # Errors
/// `ERR_REQUIRED` and `ERR_WRONG_PASSWORD` (value `tries_left`) at `field`; `ERR_LOCKED_WAIT`
/// (value `seconds`) while his name is locked; `ERR_NOT_LOGGED_IN` if he was made inactive.
pub(crate) fn prove_owner(
    ctx: &Context,
    actor: &Actor,
    typed: &Secret,
    field: &'static str,
    allow_recovery: bool,
) -> Result<(), ServiceError> {
    if typed.expose().trim().is_empty() {
        return Err(ServiceError::Rejected(vec![
            FieldError::new(ErrorCode::ErrRequired, field).into(),
        ]));
    }
    let (user, stored) = ctx.db.read(|conn| {
        Ok::<_, ServiceError>((
            store::user_by_id(conn, actor.user_id)?.filter(|user| user.is_active),
            stored_recovery(conn)?,
        ))
    })?;
    let user = user.ok_or_else(|| ServiceError::reject(ErrorCode::ErrNotLoggedIn))?;
    let key = lockout::key_for(&user.username);
    let result = attempt(
        ctx,
        &key,
        &user.username,
        Some(&user),
        Via::PartnerCode,
        || {
            if verify_password(ctx, Some(&user), typed)? {
                return Ok(true);
            }
            if allow_recovery
                && let (Some(code), Some((owner_id, hash))) =
                    (recovery::normalize(typed.expose()), stored.as_ref())
                && *owner_id == user.id
            {
                return Ok(ctx.hasher.verify(&Secret::new(code), hash));
            }
            Ok(false)
        },
    );
    match result {
        Ok(()) => {
            // A right proof ends the run of wrong tries, as a right sign-in does.
            ctx.db.write(|tx| {
                tx.execute(
                    "UPDATE users SET failed_attempts = 0, locked_until_at = NULL
                     WHERE id = ?1 AND (failed_attempts <> 0 OR locked_until_at IS NOT NULL)",
                    [user.id],
                )?;
                Ok::<(), ServiceError>(())
            })?;
            lockout::clear(ctx, &key);
            Ok(())
        }
        Err(ServiceError::Rejected(mut items)) => {
            // The wrong-password answer names the field the caller typed it in.
            for item in &mut items {
                if item.code == ErrorCode::ErrWrongPassword {
                    item.field = Some(field.to_owned());
                }
            }
            Err(ServiceError::Rejected(items))
        }
        Err(other) => Err(other),
    }
}
