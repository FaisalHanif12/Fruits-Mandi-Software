//! The first-run wizard and the Owner recovery code (NFR-INS-2, NFR-SEC-12, PRD FR-8.5,
//! plan 04 Section 4).
//!
//! The recovery code is shown once. Only its Argon2id hash is stored, in the setting
//! `recovery_code_hash` next to the id of the Owner it belongs to. Using it spends it, and a
//! new code is made to print: no code works forever, because no master password exists.

use mandi_db::rusqlite::params;
use mandi_domain::auth::recovery;
use mandi_domain::auth::validate as rules;
use mandi_domain::core::auth::Language;
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::money::Grouping;
use mandi_domain::core::validate::FieldErrors;
use serde_json::{Value, json};

use super::lockout;
use super::login::{Via, attempt, verify_password};
use super::{Actor, FirstRunRaw, RecoverRaw};
use crate::audit::Event;
use crate::context::Context;
use crate::error::{Rejection, ServiceError};
use crate::secret::Secret;
use crate::settings::{
    FirmFields, KEY_DEFAULT_LANGUAGE, KEY_FIRM, KEY_IDLE_LOCK, KEY_LOGIN_LANGUAGE,
    KEY_NUMBER_GROUPING, spelling_checks,
};
use crate::store;

/// The setting that holds the hash of the recovery code.
const KEY_RECOVERY: &str = "recovery_code_hash";
/// Random bytes asked for one code. About 90 of them are used, so this is far more than needed.
const CODE_RANDOM_BYTES: usize = 96;
/// Tries to make a code before giving up (a fresh run of random bytes each time).
const CODE_ATTEMPTS: usize = 4;

/// A new recovery code: the 16 characters to hash, and the grouped text to show.
fn new_code(ctx: &Context) -> Result<(Secret, String), ServiceError> {
    for _ in 0..CODE_ATTEMPTS {
        let mut bytes = [0_u8; CODE_RANDOM_BYTES];
        ctx.random.fill(&mut bytes)?;
        if let Some(raw) = recovery::generate(&bytes) {
            let shown = recovery::format_code(&raw);
            return Ok((Secret::new(raw), shown));
        }
    }
    Err(ServiceError::Random(
        "could not make a recovery code".to_owned(),
    ))
}

/// The stored recovery record: the Owner it belongs to and the hash of his code.
fn stored_recovery(
    conn: &mandi_db::rusqlite::Connection,
) -> Result<Option<(i64, String)>, ServiceError> {
    Ok(store::setting(conn, KEY_RECOVERY)?.and_then(|value| {
        let user_id = value.get("user_id")?.as_i64()?;
        let hash = value.get("hash")?.as_str()?.to_owned();
        Some((user_id, hash))
    }))
}

/// The first-run wizard with no spelling confirmed: a firm or mandi name with a rules-only word
/// is refused. See [`first_run_confirmed`].
///
/// # Errors
/// As [`first_run_confirmed`].
pub fn first_run(ctx: &Context, input: &FirstRunRaw) -> Result<String, ServiceError> {
    first_run_confirmed(ctx, input, &[])
}

/// The first-run wizard: creates the firm settings, the Owner and his recovery code, in one
/// transaction. Only while no user exists. Returns the code to print, as `XXXX-XXXX-XXXX-XXXX`.
/// The firm and mandi names get the spelling check (PRD FR-7.66); the words in
/// `confirmed_rules_words` are accepted and learned in the same transaction (PRD FR-7.26).
///
/// # Errors
/// `ERR_ALREADY_SET_UP`; the field errors named in the plan (`firm_name_ur`, `firm_name_en`,
/// `mandi_ur`, `mandi_en`, `address`, `phone`, `owner_username`, `owner_display_name`,
/// `password`, `password_repeat`), all together; `ERR_SPELLING_UNCONFIRMED` (value `words`) on
/// a name field that has an open word, together with the other field errors.
pub fn first_run_confirmed(
    ctx: &Context,
    input: &FirstRunRaw,
    confirmed_rules_words: &[String],
) -> Result<String, ServiceError> {
    // Cheap check first, so a second setup is refused before any hashing.
    if ctx.db.read(store::count_users)? > 0 {
        return Err(ServiceError::reject(ErrorCode::ErrAlreadySetUp));
    }
    let mut errors = FieldErrors::default();
    let firm = match input.firm.validated(&FirmFields::FIRST_RUN) {
        Ok(firm) => Some(firm),
        Err(list) => {
            for error in list {
                errors.push(error);
            }
            None
        }
    };
    let username = errors.take(rules::username("owner_username", &input.owner_username));
    let display_name = errors.take(rules::display_name(
        "owner_display_name",
        &input.owner_display_name,
    ));
    rules::password_pair(
        &mut errors,
        "password",
        "password_repeat",
        input.password.expose(),
        input.password_repeat.expose(),
    );
    // The spelling check (PRD FR-7.66) needs names that passed the script check. Its answer
    // joins the field errors, so the screen sees everything at once.
    let spelling = match &firm {
        Some(firm) => spelling_checks(ctx, firm, &FirmFields::FIRST_RUN, confirmed_rules_words),
        None => Ok(Vec::new()),
    };
    let mut rejected: Vec<Rejection> = errors
        .finish()
        .err()
        .unwrap_or_default()
        .into_iter()
        .map(Rejection::from)
        .collect();
    let checks = match spelling {
        Ok(checks) => checks,
        Err(ServiceError::Rejected(items)) => {
            rejected.extend(items);
            Vec::new()
        }
        Err(other) => return Err(other),
    };
    if !rejected.is_empty() {
        return Err(ServiceError::Rejected(rejected));
    }
    let (Some(firm), Some(username), Some(display_name)) = (firm, username, display_name) else {
        return Err(ServiceError::Data(
            "first run: a checked field is missing".to_owned(),
        ));
    };

    // The slow work happens before the write lock is taken.
    let password_hash = ctx.hasher.hash(&input.password)?;
    let (code, shown) = new_code(ctx)?;
    let code_hash = ctx.hasher.hash(&code)?;

    let learned = ctx.db.write(|tx| {
        if store::count_users(tx)? > 0 {
            return Err(ServiceError::reject(ErrorCode::ErrAlreadySetUp));
        }
        tx.execute(
            "INSERT INTO users (username, password_hash, display_name, role) VALUES (?1, ?2, ?3, ?4)",
            params![username, password_hash, display_name, store::OWNER_ROLE_VALUE],
        )?;
        let owner_id = tx.last_insert_rowid();
        tx.execute(
            "UPDATE users SET created_by = ?1, updated_by = ?1 WHERE id = ?1",
            [owner_id],
        )?;
        let by = Some(owner_id);
        store::set_setting(tx, KEY_FIRM, &firm.to_json(), by)?;
        store::set_setting(tx, KEY_IDLE_LOCK, &json!(rules::DEFAULT_IDLE_LOCK_MINUTES), by)?;
        store::set_setting(tx, KEY_DEFAULT_LANGUAGE, &json!(Language::Ur.as_str()), by)?;
        store::set_setting(tx, KEY_LOGIN_LANGUAGE, &json!(Language::Ur.as_str()), by)?;
        store::set_setting(
            tx,
            KEY_NUMBER_GROUPING,
            &serde_json::to_value(Grouping::International).map_err(|e| ServiceError::Data(e.to_string()))?,
            by,
        )?;
        store::set_setting(tx, KEY_RECOVERY, &json!({ "user_id": owner_id, "hash": code_hash }), by)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(by, "create", "user")
                .entity_id(owner_id)
                .new_value(json!({
                    "username": username,
                    "display_name": display_name,
                    "first_run": true,
                }))
                .into_entry(),
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(by, "edit", "settings")
                .new_value(json!({ "firm": firm.to_json(), "first_run": true }))
                .into_entry(),
        )?;
        // The words the Owner confirmed are learned now, by the Owner who was just made.
        let actor = Actor {
            user_id: owner_id,
            username: username.clone(),
            display_name: display_name.clone(),
        };
        let mut learned = Vec::new();
        for check in &checks {
            learned.push(check.learn(ctx, &actor, tx)?);
        }
        Ok(learned)
    })?;
    for learned in learned {
        learned.after_commit(ctx);
    }
    Ok(shown)
}

/// Resets the Owner's password with the printed recovery code, spends the code and makes a new
/// one. Wrong codes count as wrong tries of that name, with the same lockout as sign-in.
/// Returns the new code to print.
///
/// # Errors
/// The password errors on `new_password` and `new_password_repeat`; `ERR_REQUIRED`
/// (`username`, `recovery_code`); `ERR_RECOVERY_CODE_WRONG` (`tries_left`) for a wrong code, an
/// unknown name, or a user who is not the Owner the code belongs to; `ERR_LOCKED_WAIT`;
/// `ERR_USER_INACTIVE`.
pub fn recover(ctx: &Context, input: &RecoverRaw) -> Result<String, ServiceError> {
    let mut errors = FieldErrors::default();
    if input.username.trim().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrRequired, "username"));
    }
    if input.recovery_code.expose().trim().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrRequired, "recovery_code"));
    }
    rules::password_pair(
        &mut errors,
        "new_password",
        "new_password_repeat",
        input.new_password.expose(),
        input.new_password_repeat.expose(),
    );
    errors.finish()?;

    let key = lockout::key_for(&input.username);
    let (user, stored) = ctx.db.read(|conn| {
        Ok::<_, ServiceError>((store::user_by_username(conn, &key)?, stored_recovery(conn)?))
    })?;
    let normalized = recovery::normalize(input.recovery_code.expose());
    // The code can only be right for the one Owner it was made for.
    let target = match (&user, &stored) {
        (Some(user), Some((owner_id, hash))) if user.id == *owner_id => Some((user, hash.clone())),
        _ => None,
    };
    attempt(
        ctx,
        &key,
        &input.username,
        user.as_ref(),
        Via::Recover,
        || {
            match (&target, normalized) {
                (Some((_, hash)), Some(code)) => Ok(ctx.hasher.verify(&Secret::new(code), hash)),
                // Do the same work as for a right name, so the time tells nothing.
                _ => verify_password(ctx, None, &Secret::new("x")).map(|_| false),
            }
        },
    )?;
    let Some((user, old_hash)) = target else {
        return Err(ServiceError::Data(
            "recovery passed without an Owner".to_owned(),
        ));
    };
    if !user.is_active {
        return Err(ServiceError::reject(ErrorCode::ErrUserInactive));
    }

    let password_hash = ctx.hasher.hash(&input.new_password)?;
    let (new_code_secret, shown) = new_code(ctx)?;
    let new_code_hash = ctx.hasher.hash(&new_code_secret)?;
    ctx.db.write(|tx| {
        // The code is spent only once: a second request with the same code finds the hash changed.
        let still_valid =
            stored_recovery(tx)?.is_some_and(|(id, hash)| id == user.id && hash == old_hash);
        if !still_valid {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrRecoveryCodeWrong,
                "recovery_code",
            ));
        }
        tx.execute(
            "UPDATE users SET password_hash = ?1, failed_attempts = 0, locked_until_at = NULL,
                 updated_at = ?2, updated_by = ?3, version = version + 1 WHERE id = ?3",
            params![password_hash, ctx.now_text(), user.id],
        )?;
        let record: Value = json!({ "user_id": user.id, "hash": new_code_hash });
        store::set_setting(tx, KEY_RECOVERY, &record, Some(user.id))?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user.id), "recover", "user")
                .entity_id(user.id)
                .into_entry(),
        )?;
        Ok(())
    })?;
    lockout::clear(ctx, &key);
    Ok(shown)
}
