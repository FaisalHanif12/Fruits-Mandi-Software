//! Who is signed in, the sign-in check and the session commands (PRD FR-2.3, FR-8.4,
//! NFR-SEC-4).

use mandi_db::rusqlite::Connection;
use mandi_domain::auth::validate::DEFAULT_TEXT_SIZE;
use mandi_domain::core::auth::Language;
use mandi_domain::core::error::ErrorCode;
use serde_json::Value;

use super::{Actor, AuthStatusInfo, SessionInfo};
use crate::audit::Event;
use crate::context::{Context, Session};
use crate::error::ServiceError;
use crate::settings;
use crate::store::{self, UserRecord};

/// The user as the screens show him: his language and text size come from his preferences,
/// else from the defaults.
pub(super) fn session_info(
    conn: &Connection,
    user: &UserRecord,
) -> Result<SessionInfo, ServiceError> {
    let language = store::pref(conn, user.id, "language")?
        .as_ref()
        .and_then(Value::as_str)
        .and_then(Language::parse);
    let language = match language {
        Some(language) => language,
        None => settings::load_default_language(conn)?,
    };
    let text_size = store::pref(conn, user.id, "text_size")?
        .and_then(|v| v.as_u64())
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(DEFAULT_TEXT_SIZE);
    Ok(SessionInfo {
        id: user.id,
        username: user.username.clone(),
        display_name: user.display_name.clone(),
        language,
        text_size,
    })
}

/// Who is signed in, whether a first run is due, and what the sign-in screen shows. Needs no
/// sign-in: it is asked before sign-in.
///
/// # Errors
/// `ServiceError::Db`.
pub fn status(ctx: &Context) -> Result<AuthStatusInfo, ServiceError> {
    let session = ctx.session_snapshot();
    ctx.db.read(|conn| {
        let (info, locked) = match &session {
            Some(open) => match store::user_by_id(conn, open.user_id)? {
                Some(user) if user.is_active => (Some(session_info(conn, &user)?), open.locked),
                _ => (None, false),
            },
            None => (None, false),
        };
        Ok(AuthStatusInfo {
            first_run: store::count_users(conn)? == 0,
            session: info,
            locked,
            login_language: settings::load_login_language(conn)?,
            firm: settings::load_firm(conn)?,
        })
    })
}

/// The sign-in check every command starts with (NFR-SEC-4, Code Standards B2 and S1).
///
/// There is one user, the Owner, and no roles (PRD Section 2). The check is only that the
/// Owner is signed in and the app is not locked.
///
/// - No session: `ERR_NOT_LOGGED_IN`. A locked session: `ERR_SESSION_LOCKED`. Neither writes.
/// - The user is read from the database for this call, so an inactive flag works at once.
///
/// # Errors
/// As above, or `ServiceError::Db`.
pub fn require(ctx: &Context) -> Result<Actor, ServiceError> {
    let Some(open) = ctx.session_snapshot() else {
        return Err(ServiceError::reject(ErrorCode::ErrNotLoggedIn));
    };
    if open.locked {
        return Err(ServiceError::reject(ErrorCode::ErrSessionLocked));
    }
    let user = ctx.db.read(|conn| store::user_by_id(conn, open.user_id))?;
    let Some(user) = user.filter(|u| u.is_active) else {
        // The user was made inactive while he was signed in.
        *ctx.session_lock() = None;
        return Err(ServiceError::reject(ErrorCode::ErrNotLoggedIn));
    };
    Ok(Actor {
        user_id: user.id,
        username: user.username,
        display_name: user.display_name,
    })
}

/// Signs out. Also works while the session is locked.
///
/// # Errors
/// `ERR_NOT_LOGGED_IN` without a session.
pub fn logout(ctx: &Context) -> Result<(), ServiceError> {
    let Some(open) = ctx.session_snapshot() else {
        return Err(ServiceError::reject(ErrorCode::ErrNotLoggedIn));
    };
    ctx.db.write(|tx| {
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(open.user_id), "logout", "user")
                .entity_id(open.user_id)
                .into_entry(),
        )?;
        Ok::<(), ServiceError>(())
    })?;
    *ctx.session_lock() = None;
    Ok(())
}

/// Locks the session (idle lock, PRD FR-8.4). The screens keep their unsaved text. Locking a
/// locked session does nothing.
///
/// # Errors
/// `ERR_NOT_LOGGED_IN` without a session.
pub fn lock(ctx: &Context) -> Result<(), ServiceError> {
    let Some(open) = ctx.session_snapshot() else {
        return Err(ServiceError::reject(ErrorCode::ErrNotLoggedIn));
    };
    if open.locked {
        return Ok(());
    }
    ctx.db.write(|tx| {
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(open.user_id), "lock", "user")
                .entity_id(open.user_id)
                .into_entry(),
        )?;
        Ok::<(), ServiceError>(())
    })?;
    if let Some(session) = ctx.session_lock().as_mut() {
        session.locked = true;
    }
    Ok(())
}

/// Opens the session for a user who has just proved his password.
pub(super) fn open(ctx: &Context, user: &UserRecord) {
    *ctx.session_lock() = Some(Session {
        user_id: user.id,
        username: user.username.clone(),
        locked: false,
    });
}
