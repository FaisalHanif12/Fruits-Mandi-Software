//! The audit viewer (plan 22 early, PRD FR-21.3, NFR-SEC-5, Code Standards B31): a read-only,
//! paged list of the audit log with filters for the user, the action, the record, the days and
//! words in the reason or in the old and new values.
//!
//! - **Read only.** Nothing here writes; the log cannot be changed or deleted from the app.
//! - **Paged** with the count of the whole filter from one query (B31). Newest first. The days
//!   are days in Pakistan time (UTC+5, Code Standards G9); the stored time is UTC.
//! - **Secrets hidden.** Password hashes, keys and recovery codes never reach the screen
//!   (Code Standards S2): see [`flatten`].
//! - **Indexed.** The filters by day, user and record use the indexes of `audit_log`; the
//!   filter by action and the words are applied to the rows those leave.
//!
//! Example: after the Owner changes the firm's English name, one entry reads `edit`, `settings`,
//! with the change `firm.name_en` from `Maher` to `Maher Adnan` and his user name.

mod flatten;
mod model;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use mandi_core::dates::{BusinessDate, parse_ddmmyyyy};
use mandi_core::error::{ErrorCode, FieldError};
use mandi_db::Db;
use mandi_db::rusqlite::types::Value as SqlValue;
use mandi_db::rusqlite::{Connection, OptionalExtension, params_from_iter};
use serde_json::Value;

use crate::error::ReportsError;
use crate::reports::common::{Conds, today};
use crate::reports::page_spec;

pub use model::{
    AuditChange, AuditFilters, AuditListInput, AuditOptions, AuditPage, AuditResolvedFilters,
    AuditRow, AuditUser, AuditValue,
};

/// Pakistan time is UTC+5 all year (Code Standards G9).
const PAKISTAN_OFFSET_HOURS: i64 = 5;
/// The longest action, entity or search text.
const MAX_TEXT: usize = 100;

fn user_of(conn: &Connection, id: i64) -> Result<Option<AuditUser>, ReportsError> {
    Ok(conn
        .query_row(
            "SELECT id, username, display_name FROM users WHERE id = ?1",
            [id],
            |r| {
                Ok(AuditUser {
                    id: r.get(0)?,
                    username: r.get(1)?,
                    display_name: r.get(2)?,
                })
            },
        )
        .optional()?)
}

/// A day typed by the user: `YYYY-MM-DD`, or `DD-MM-YYYY` with the shortcuts of PRD FR-7.19.
fn parse_day(text: &str, today: BusinessDate) -> Result<BusinessDate, ErrorCode> {
    let text = text.trim();
    BusinessDate::parse_iso(text).map_or_else(|| parse_ddmmyyyy(text, today), Ok)
}

/// The UTC time text at which a day starts in Pakistan time, `YYYY-MM-DDTHH:MM:SS`.
fn utc_start_of(day: &str) -> Result<String, ReportsError> {
    let date = NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .map_err(|_| ReportsError::Data(format!("a day is not a date: {day}")))?;
    let local_midnight = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| ReportsError::Data(format!("a day has no midnight: {day}")))?;
    let utc = local_midnight
        .checked_sub_signed(Duration::hours(PAKISTAN_OFFSET_HOURS))
        .ok_or_else(|| ReportsError::Data(format!("a day is out of range: {day}")))?;
    Ok(utc.format("%Y-%m-%dT%H:%M:%S").to_string())
}

/// A text filter box: trimmed, without control characters, not too long.
fn text_filter(value: &str, field: &str, problems: &mut Vec<FieldError>) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.chars().any(char::is_control) {
        problems.push(FieldError::new(ErrorCode::ErrControlChars, field));
        return None;
    }
    if value.chars().count() > MAX_TEXT {
        problems.push(FieldError::new(ErrorCode::ErrTextTooLong, field));
        return None;
    }
    Some(value.to_owned())
}

/// The filters read and checked, as the conditions of the query and the echo for the screen.
struct Typed {
    conds: Conds,
    resolved: AuditResolvedFilters,
}

fn resolve(
    conn: &Connection,
    now: BusinessDate,
    input: &AuditFilters,
) -> Result<Typed, ReportsError> {
    let mut problems: Vec<FieldError> = Vec::new();
    let mut resolved = AuditResolvedFilters::default();
    let mut conds = Conds::new();

    if let Some(id) = input.user_id {
        match user_of(conn, id)? {
            Some(user) => {
                conds.id("a.user_id = ?", Some(id));
                resolved.user = Some(user);
            }
            None => problems.push(FieldError::new(ErrorCode::ErrNotFound, "filters.user_id")),
        }
    }
    let action = text_filter(&input.action, "filters.action", &mut problems);
    let entity = text_filter(&input.entity, "filters.entity", &mut problems);
    let words = text_filter(&input.text, "filters.text", &mut problems);
    conds
        .text("a.action = ?", action.as_deref())
        .text("a.entity = ?", entity.as_deref());
    if entity.is_none() && input.entity_id.is_some() {
        // A record number means nothing without the kind of record.
        problems.push(FieldError::new(ErrorCode::ErrRequired, "filters.entity"));
    }
    conds.id("a.entity_id = ?", input.entity_id);
    resolved.action = action;
    resolved.entity = entity;
    resolved.entity_id = input.entity_id;

    let mut day = |text: &str, field: &str| -> Option<String> {
        if text.trim().is_empty() {
            return None;
        }
        match parse_day(text, now) {
            Ok(day) => Some(day.to_iso()),
            Err(code) => {
                problems.push(FieldError::new(code, field));
                None
            }
        }
    };
    let from = day(&input.from, "filters.from");
    let to = day(&input.to, "filters.to");
    if let (Some(from), Some(to)) = (&from, &to)
        && from > to
    {
        problems.push(FieldError::new(ErrorCode::ErrDateRange, "filters.to"));
    }
    if !problems.is_empty() {
        return Err(ReportsError::Rejected(problems));
    }

    // The stored time is UTC; a day in Pakistan time starts five hours earlier.
    if let Some(from) = &from {
        conds.text("a.at >= ?", Some(&utc_start_of(from)?));
    }
    if let Some(to) = &to {
        let next = BusinessDate::parse_iso(to)
            .and_then(BusinessDate::next_day)
            .ok_or_else(|| ReportsError::Data(format!("a day has no next day: {to}")))?;
        conds.text("a.at < ?", Some(&utc_start_of(&next.to_iso())?));
    }
    // `instr` finds the words with no wildcard characters to escape.
    conds.text(
        "instr(lower(COALESCE(a.reason, '') || ' ' || COALESCE(a.old_json, '') || ' ' \
         || COALESCE(a.new_json, '')), lower(?)) > 0",
        words.as_deref(),
    );
    resolved.from = from;
    resolved.to = to;
    resolved.text = words;
    Ok(Typed { conds, resolved })
}

/// The day and time in Pakistan time of a stored UTC time.
fn local_parts(at: &str) -> Result<(String, String), ReportsError> {
    let parsed: DateTime<Utc> = DateTime::parse_from_rfc3339(at)
        .map_err(|_| ReportsError::Data(format!("an audit time is not a time: {at}")))?
        .with_timezone(&Utc);
    let local = parsed + Duration::hours(PAKISTAN_OFFSET_HOURS);
    Ok((
        local.format("%Y-%m-%d").to_string(),
        local.format("%H:%M:%S").to_string(),
    ))
}

fn parse_json(text: Option<String>) -> Result<Option<Value>, ReportsError> {
    text.map(|t| {
        serde_json::from_str(&t)
            .map_err(|e| ReportsError::Data(format!("an audit value is not JSON: {e}")))
    })
    .transpose()
}

/// One page of the audit log, newest first, with the count of the whole filter.
///
/// # Errors
/// `ReportsError::Rejected` with every problem and its field (`filters.user_id`,
/// `filters.from`, `filters.to`, `filters.entity`, `offset`, `limit`); a database error.
pub fn list(db: &Db, input: &AuditListInput) -> Result<AuditPage, ReportsError> {
    let spec = page_spec(input.offset, input.limit)?;
    let now = today(db.clock().now_utc())?;
    db.read(|conn| {
        let typed = resolve(conn, now, &input.filters)?;
        let total_count: i64 = conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM audit_log a WHERE {}",
                typed.conds.sql()
            ),
            params_from_iter(typed.conds.args().iter()),
            |r| r.get(0),
        )?;
        let sql = format!(
            "SELECT a.id, a.at, a.user_id, u.username, u.display_name, a.action, a.entity,
                    a.entity_id, a.old_json, a.new_json, a.reason
             FROM audit_log a LEFT JOIN users u ON u.id = a.user_id
             WHERE {} ORDER BY a.at DESC, a.id DESC LIMIT ? OFFSET ?",
            typed.conds.sql()
        );
        let args: Vec<SqlValue> = typed.conds.page_args(spec.sql_limit(), spec.offset);
        let mut statement = conn.prepare(&sql)?;
        let raws = statement
            .query_map(params_from_iter(args.iter()), |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, Option<i64>>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, Option<String>>(9)?,
                    r.get::<_, Option<String>>(10)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut rows = Vec::with_capacity(raws.len());
        for (
            id,
            at,
            user_id,
            username,
            display_name,
            action,
            entity,
            entity_id,
            old,
            new,
            reason,
        ) in raws
        {
            let (date, time) = local_parts(&at)?;
            let old = parse_json(old)?;
            let new = parse_json(new)?;
            let (changes, changes_truncated) = flatten::changes(old.as_ref(), new.as_ref());
            rows.push(AuditRow {
                id,
                at,
                date,
                time,
                user: user_id.map(|id| AuditUser {
                    id,
                    username: username.clone().unwrap_or_default(),
                    display_name: display_name.clone().unwrap_or_default(),
                }),
                action,
                entity,
                entity_id,
                reason,
                old_json: old.as_ref().map(|v| flatten::redact(v).to_string()),
                new_json: new.as_ref().map(|v| flatten::redact(v).to_string()),
                changes,
                changes_truncated,
            });
        }
        Ok(AuditPage {
            filters: typed.resolved,
            rows,
            offset: spec.offset,
            limit: spec.reported_limit(),
            total_count,
        })
    })
}

/// The values the filter boxes offer: every user who did something, every action and every kind
/// of record in the log, in order.
///
/// # Errors
/// A database error.
pub fn options(db: &Db) -> Result<AuditOptions, ReportsError> {
    db.read(|conn| {
        let mut users = conn.prepare(
            "SELECT id, username, display_name FROM users
             WHERE id IN (SELECT DISTINCT user_id FROM audit_log WHERE user_id IS NOT NULL)
             ORDER BY username",
        )?;
        let users = users
            .query_map([], |r| {
                Ok(AuditUser {
                    id: r.get(0)?,
                    username: r.get(1)?,
                    display_name: r.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let distinct = |column: &str| -> Result<Vec<String>, ReportsError> {
            // The column name is one of two constants of this function, never user text.
            let mut statement = conn.prepare(&format!(
                "SELECT DISTINCT {column} FROM audit_log ORDER BY {column}"
            ))?;
            let found = statement
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(found)
        };
        Ok(AuditOptions {
            users,
            actions: distinct("action")?,
            entities: distinct("entity")?,
        })
    })
}

/// The SQL of the audit viewer for the query plan test (Code Standards B18): the page query for
/// a range of days and a user, with the values to bind.
#[doc(hidden)]
pub mod plans {
    use mandi_db::rusqlite::types::Value;

    use crate::reports::common::Conds;

    /// The page query for a UTC time range (`from` included, `to` excluded) and a user.
    pub fn page(from: Option<&str>, to: Option<&str>, user: Option<i64>) -> (String, Vec<Value>) {
        let mut c = Conds::new();
        c.text("a.at >= ?", from)
            .text("a.at < ?", to)
            .id("a.user_id = ?", user);
        let sql = format!(
            "SELECT a.id, a.at FROM audit_log a LEFT JOIN users u ON u.id = a.user_id
             WHERE {} ORDER BY a.at DESC, a.id DESC LIMIT ? OFFSET ?",
            c.sql()
        );
        (sql, c.page_args(100, 0))
    }
}
