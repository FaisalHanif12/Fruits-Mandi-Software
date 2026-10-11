//! Small SQL helpers shared by the services: users, settings and preferences.
//! Every statement uses parameters (Code Standards B14).

use mandi_db::rusqlite::{Connection, OptionalExtension, Row, params};
use mandi_domain::core::auth::Role;
use serde_json::Value;

use crate::context::role_from_column;
use crate::error::ServiceError;

/// One row of `users`, as the services read it.
#[derive(Debug, Clone)]
pub(crate) struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: Role,
    pub is_active: bool,
    pub failed_attempts: u32,
    pub locked_until_at: Option<String>,
    pub last_login_at: Option<String>,
}

const USER_COLUMNS: &str = "id, username, password_hash, display_name, role, is_active, failed_attempts, locked_until_at, last_login_at";

fn user_from_row(row: &Row<'_>) -> Result<(i64, String, String, String, String, bool, u32, Option<String>, Option<String>), mandi_db::rusqlite::Error> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
    ))
}

fn build(
    row: (i64, String, String, String, String, bool, u32, Option<String>, Option<String>),
) -> Result<UserRecord, ServiceError> {
    Ok(UserRecord {
        id: row.0,
        username: row.1,
        password_hash: row.2,
        display_name: row.3,
        role: role_from_column(&row.4)?,
        is_active: row.5,
        failed_attempts: row.6,
        locked_until_at: row.7,
        last_login_at: row.8,
    })
}

/// The user with this (lower case) sign-in name.
pub(crate) fn user_by_username(conn: &Connection, username: &str) -> Result<Option<UserRecord>, ServiceError> {
    let row = conn
        .query_row(
            &format!("SELECT {USER_COLUMNS} FROM users WHERE username = ?1"),
            [username],
            user_from_row,
        )
        .optional()?;
    row.map(build).transpose()
}

/// The user with this id.
pub(crate) fn user_by_id(conn: &Connection, id: i64) -> Result<Option<UserRecord>, ServiceError> {
    let row = conn
        .query_row(
            &format!("SELECT {USER_COLUMNS} FROM users WHERE id = ?1"),
            [id],
            user_from_row,
        )
        .optional()?;
    row.map(build).transpose()
}

/// How many users exist. 0 means the first-run wizard is due.
pub(crate) fn count_users(conn: &Connection) -> Result<i64, ServiceError> {
    Ok(conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?)
}

/// Active Owners other than `except_id`.
pub(crate) fn count_other_active_owners(conn: &Connection, except_id: i64) -> Result<i64, ServiceError> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM users WHERE role = 'owner' AND is_active = 1 AND id <> ?1",
        [except_id],
        |r| r.get(0),
    )?)
}

/// A setting by key, if it was ever saved.
pub(crate) fn setting(conn: &Connection, key: &str) -> Result<Option<Value>, ServiceError> {
    let text: Option<String> = conn
        .query_row("SELECT value_json FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()?;
    text.map(|t| serde_json::from_str(&t).map_err(|e| ServiceError::Data(format!("setting {key}: {e}"))))
        .transpose()
}

/// Saves a setting (insert or replace the value), keeping the version count.
pub(crate) fn set_setting(
    conn: &Connection,
    key: &str,
    value: &Value,
    by_user: Option<i64>,
) -> Result<(), ServiceError> {
    conn.execute(
        "INSERT INTO settings (key, value_json, created_by, updated_by) VALUES (?1, ?2, ?3, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_by = excluded.updated_by,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), version = version + 1",
        params![key, value.to_string(), by_user],
    )?;
    Ok(())
}

/// A preference of one user.
pub(crate) fn pref(conn: &Connection, user_id: i64, key: &str) -> Result<Option<Value>, ServiceError> {
    let text: Option<String> = conn
        .query_row(
            "SELECT value_json FROM user_prefs WHERE user_id = ?1 AND key = ?2",
            params![user_id, key],
            |r| r.get(0),
        )
        .optional()?;
    text.map(|t| serde_json::from_str(&t).map_err(|e| ServiceError::Data(format!("pref {key}: {e}"))))
        .transpose()
}

/// Saves a preference of one user.
pub(crate) fn set_pref(conn: &Connection, user_id: i64, key: &str, value: &Value) -> Result<(), ServiceError> {
    conn.execute(
        "INSERT INTO user_prefs (user_id, key, value_json, created_by, updated_by) VALUES (?1, ?2, ?3, ?1, ?1)
         ON CONFLICT(user_id, key) DO UPDATE SET value_json = excluded.value_json, updated_by = excluded.updated_by,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), version = version + 1",
        params![user_id, key, value.to_string()],
    )?;
    Ok(())
}
