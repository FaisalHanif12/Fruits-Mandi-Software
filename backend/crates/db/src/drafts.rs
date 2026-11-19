//! The `drafts` table (Architecture 15.3, PRD FR-12.10, NFR-REL-2): the unsaved work of a screen,
//! one row for each user and screen. A draft is not a business record (Code Standards D3), so a
//! saved بکری deletes its own draft in its transaction.

use rusqlite::{Connection, OptionalExtension, params};

/// Keeps the draft of a screen for a user: updates the row if there is one, else adds it.
///
/// # Errors
/// Any SQLite error. A payload that is not valid JSON fails the CHECK of the table.
pub fn upsert(
    conn: &Connection,
    user_id: i64,
    screen: &str,
    payload_json: &str,
    now: &str,
) -> rusqlite::Result<()> {
    let changed = conn.execute(
        "UPDATE drafts SET payload_json = ?3, saved_at = ?4, updated_at = ?4, updated_by = ?1,
             version = version + 1
         WHERE user_id = ?1 AND screen = ?2",
        params![user_id, screen, payload_json, now],
    )?;
    if changed == 0 {
        conn.execute(
            "INSERT INTO drafts (created_at, created_by, updated_at, updated_by, screen, user_id, payload_json,
                 saved_at)
             VALUES (?1, ?2, ?1, ?2, ?3, ?2, ?4, ?1)",
            params![now, user_id, screen, payload_json],
        )?;
    }
    Ok(())
}

/// The draft of a screen: its payload and the time it was kept.
///
/// # Errors
/// Any SQLite error.
pub fn get(
    conn: &Connection,
    user_id: i64,
    screen: &str,
) -> rusqlite::Result<Option<(String, String)>> {
    conn.query_row(
        "SELECT payload_json, saved_at FROM drafts WHERE user_id = ?1 AND screen = ?2
         ORDER BY id DESC LIMIT 1",
        params![user_id, screen],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}

/// Throws the draft of a screen away. Returns how many rows went.
///
/// # Errors
/// Any SQLite error.
pub fn delete(conn: &Connection, user_id: i64, screen: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM drafts WHERE user_id = ?1 AND screen = ?2",
        params![user_id, screen],
    )
}
