//! The last edit of saved entries, read from the audit log (PRD FR-7.71, plan 44). Read only.
//!
//! One query for every caller: the edit stamp of a list row (`mandi-services`) and the
//! `audit_edit_stamp` command (`mandi-reports`) both read it here (Code Standards G6).

use std::collections::HashMap;

use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};

/// The ids looked up in one statement.
const CHUNK: usize = 200;

/// The last edit of one entry, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastEdit {
    /// How many edits the entry has (the last is this one).
    pub count: i64,
    /// The time of the last one, UTC, as stored.
    pub at: String,
    /// The reason typed with it. `None` when none was given.
    pub reason: Option<String>,
    /// The display name of the user who made it (the user name when there is no display name).
    pub by: Option<String>,
    /// The Owner's sign saved with it, when it was made in a reopened day (PRD FR-21.4, plan 73).
    pub sign: Option<String>,
}

/// The last edit of each entry in `ids` that was ever edited, by id. `entity` is the `entity` text
/// of the audit rows (`bikri`, `voucher`, `fruit_take` ...). An entry never edited is not in the
/// map.
///
/// # Errors
/// Any SQLite error.
pub fn last_edits(
    conn: &Connection,
    entity: &str,
    ids: &[i64],
) -> rusqlite::Result<HashMap<i64, LastEdit>> {
    let mut found = HashMap::new();
    for chunk in ids.chunks(CHUNK) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let sql = format!(
            "SELECT entity_id, COUNT(*), MAX(id) FROM audit_log
             WHERE entity = ? AND action = 'edit' AND entity_id IN ({marks}) GROUP BY entity_id"
        );
        let mut args: Vec<Value> = vec![Value::Text(entity.to_owned())];
        args.extend(chunk.iter().map(|id| Value::Integer(*id)));
        let mut statement = conn.prepare(&sql)?;
        let rows = statement
            .query_map(params_from_iter(args.iter()), |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for (id, count, last) in rows {
            let (at, reason, by, sign): (String, Option<String>, Option<String>, Option<String>) =
                conn.query_row(
                    "SELECT a.at, a.reason, COALESCE(NULLIF(u.display_name, ''), u.username),
                            json_extract(a.new_json, '$.sign')
                     FROM audit_log a LEFT JOIN users u ON u.id = a.user_id WHERE a.id = ?1",
                    [last],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )?;
            found.insert(
                id,
                LastEdit {
                    count,
                    at,
                    reason: reason.filter(|r| !r.is_empty()),
                    by,
                    sign: sign.filter(|s| !s.is_empty()),
                },
            );
        }
    }
    Ok(found)
}
