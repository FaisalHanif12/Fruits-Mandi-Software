//! A second look at the backup key (PRD FR-20.10, v1.54, NFR-SEC-7).
//!
//! The key is shown once at setup and printed. If the paper is lost, the Owner can see the key again
//! after he proves two things: his password (with the same lockout as sign-in, PRD FR-8.2) and his
//! sign (PRD FR-21.4). [`prove`] checks both. The key itself is read and audited by
//! `mandi_backup::service::BackupService::key_reveal`, which the command calls only after [`prove`]
//! returned `Ok`.
//!
//! Every refused try (a wrong password, a locked name, a wrong sign) writes one audit row of its own,
//! `reveal_refused` on `backup_key`. No row, no log and no error ever holds the key, the password or
//! the typed sign (NFR-SEC-7, NFR-OBS-2).
//!
//! Example: the password is right and the sign is "adnan": `prove` returns `Ok(())`. A sign of
//! "munshi" returns `ERR_SIGN_WRONG` at `sign` and writes `{"refused":"sign"}`.

use mandi_domain::core::error::{ErrorCode, FieldError};
use serde_json::json;

use crate::audit::Event;
use crate::auth::{Actor, prove_owner};
use crate::context::Context;
use crate::error::ServiceError;
use crate::owner_sign;
use crate::secret::Secret;

/// The audit action of a refused try.
const REFUSED_ACTION: &str = "reveal_refused";
/// The audit entity: the backup key. Not the key itself.
const ENTITY: &str = "backup_key";

/// Writes the audit row of a refused try, and nothing typed.
fn audit_refused(ctx: &Context, actor: &Actor, why: &'static str) -> Result<(), ServiceError> {
    ctx.db.write(|tx| {
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), REFUSED_ACTION, ENTITY)
                .new_value(json!({ "refused": why }))
                .into_entry(),
        )?;
        Ok::<(), ServiceError>(())
    })
}

/// Proves the password and the sign of the Owner before the backup key is shown again.
///
/// The password goes first (a wrong one counts as a wrong try of his name, with the lockout of PRD
/// FR-8.2), then the sign. A try that gets a refusal writes one audit row without any typed text.
///
/// # Errors
/// `ERR_REQUIRED` on `password` and `ERR_SIGN_REQUIRED` on `sign` (both together, nothing audited);
/// `ERR_WRONG_PASSWORD` (value `tries_left`) on `password`; `ERR_LOCKED_WAIT` (value `seconds`);
/// `ERR_SIGN_WRONG` on `sign`; `ERR_NOT_LOGGED_IN` if he was made inactive.
pub fn prove(
    ctx: &Context,
    actor: &Actor,
    password: &Secret,
    sign: &str,
) -> Result<(), ServiceError> {
    let mut missing = Vec::new();
    if password.expose().trim().is_empty() {
        missing.push(FieldError::new(ErrorCode::ErrRequired, "password"));
    }
    if sign.trim().is_empty() {
        missing.push(FieldError::new(ErrorCode::ErrSignRequired, "sign"));
    }
    if !missing.is_empty() {
        return Err(missing.into());
    }
    match prove_owner(ctx, actor, password, "password", false) {
        Ok(()) => {}
        Err(error) => {
            let why = if error.has_code(ErrorCode::ErrLockedWait) {
                Some("locked")
            } else if error.has_code(ErrorCode::ErrWrongPassword) {
                Some("password")
            } else {
                None
            };
            if let Some(why) = why {
                audit_refused(ctx, actor, why)?;
            }
            return Err(error);
        }
    }
    let configured = ctx.db.read(owner_sign::configured)?;
    if owner_sign::accepted(sign, configured.as_deref()).is_none() {
        audit_refused(ctx, actor, "sign")?;
        return Err(ServiceError::reject_field(ErrorCode::ErrSignWrong, "sign"));
    }
    Ok(())
}
