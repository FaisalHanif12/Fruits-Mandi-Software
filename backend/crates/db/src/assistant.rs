//! The question log of the assistant (PRD FR-7.84, FR-7.81, Architecture 15.3, migration 0015).
//!
//! Append-only: a row is written once, after the answer, and never changed or deleted (triggers
//! refuse it). The daily count of 20 is the number of today's rows with `counted = 1`, where the
//! day is the Pakistan date the caller works out. No amount is stored: the tools and their inputs
//! are, so an answer can be read again on today's books.
//!
//! Every statement uses parameters (Code Standards B14).
//!
//! Example: after one model question today, `count_counted_on(conn, "2026-10-10")` is 1; a quick
//! question (a chip) adds a row but not to that count.

use rusqlite::{Connection, OptionalExtension, Row, params};

/// One question to write. Strings for the fixed lists (`source`, `outcome`, `language`): the
/// database checks them.
#[derive(Debug, Clone)]
pub struct NewQuestion<'a> {
    /// When it was asked, UTC, `YYYY-MM-DDTHH:MM:SS.mmmZ`.
    pub asked_at: &'a str,
    /// The Pakistan date it counts for, `YYYY-MM-DD`.
    pub asked_day: &'a str,
    /// The user who asked.
    pub asked_by: Option<i64>,
    /// The question as typed, or a chip's name.
    pub question: &'a str,
    /// `ur` or `en`.
    pub language: &'a str,
    /// The conversation of the panel.
    pub conversation_id: &'a str,
    /// The question a continuation belongs to.
    pub parent_id: Option<i64>,
    /// `chip`, `model`, `continue`, `rerun` or `test`.
    pub source: &'a str,
    /// True for a model question and a connection test.
    pub counted: bool,
    /// Each source read, as JSON.
    pub sources_json: &'a str,
    /// How it ended (PRD FR-7.84).
    pub outcome: &'a str,
    /// Each tool called with its inputs, as JSON.
    pub tool_calls_json: &'a str,
    /// The answer as the model wrote it, with references, as JSON. Null for a chip.
    pub answer_json: Option<&'a str>,
    /// The model. Null for a chip.
    pub model: Option<&'a str>,
    /// Model calls made.
    pub model_calls: Option<i64>,
    /// Input tokens.
    pub tokens_in: Option<i64>,
    /// Of those, cached.
    pub tokens_cached: Option<i64>,
    /// Output tokens.
    pub tokens_out: Option<i64>,
    /// Of those, reasoning.
    pub tokens_reasoning: Option<i64>,
    /// The last `finish_reason` of the model, or the limit that stopped the question.
    pub finish_reason: Option<&'a str>,
    /// Milliseconds this run took (plan 51 B2).
    pub total_millis: Option<i64>,
    /// Of those, waiting for the model.
    pub model_millis: Option<i64>,
    /// Of those, the app's own work in the tools.
    pub tool_millis: Option<i64>,
    /// Each model call of this run as JSON: times, tokens, tool names. Null for a chip.
    pub calls_json: Option<&'a str>,
}

/// One row of the log, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionRow {
    /// The row id.
    pub id: i64,
    /// When it was asked, UTC.
    pub asked_at: String,
    /// The Pakistan date it counts for.
    pub asked_day: String,
    /// The user who asked.
    pub asked_by: Option<i64>,
    /// The question as typed, or a chip's name.
    pub question: String,
    /// `ur` or `en`.
    pub language: String,
    /// The conversation of the panel.
    pub conversation_id: String,
    /// The question a continuation belongs to.
    pub parent_id: Option<i64>,
    /// `chip`, `model`, `continue`, `rerun` or `test`.
    pub source: String,
    /// True when it counts in the daily 20.
    pub counted: bool,
    /// Each source read, as JSON.
    pub sources_json: String,
    /// How it ended.
    pub outcome: String,
    /// Each tool called with its inputs, as JSON.
    pub tool_calls_json: String,
    /// The answer as the model wrote it, as JSON.
    pub answer_json: Option<String>,
    /// The model.
    pub model: Option<String>,
    /// Model calls made.
    pub model_calls: Option<i64>,
    /// Input tokens.
    pub tokens_in: Option<i64>,
    /// Of those, cached.
    pub tokens_cached: Option<i64>,
    /// Output tokens.
    pub tokens_out: Option<i64>,
    /// Of those, reasoning.
    pub tokens_reasoning: Option<i64>,
    /// The last `finish_reason` of the model, or the limit that stopped the question.
    pub finish_reason: Option<String>,
    /// Milliseconds this run took. Null for a row written before migration 17.
    pub total_millis: Option<i64>,
    /// Of those, waiting for the model.
    pub model_millis: Option<i64>,
    /// Of those, the app's own work in the tools.
    pub tool_millis: Option<i64>,
    /// Each model call of this run as JSON.
    pub calls_json: Option<String>,
}

const COLUMNS: &str = "id, asked_at, asked_day, asked_by, question, language, conversation_id, \
    parent_id, source, counted, sources_json, outcome, tool_calls_json, answer_json, model, \
    model_calls, tokens_in, tokens_cached, tokens_out, tokens_reasoning, finish_reason, \
    total_millis, model_millis, tool_millis, calls_json";

fn from_row(row: &Row<'_>) -> rusqlite::Result<QuestionRow> {
    Ok(QuestionRow {
        id: row.get(0)?,
        asked_at: row.get(1)?,
        asked_day: row.get(2)?,
        asked_by: row.get(3)?,
        question: row.get(4)?,
        language: row.get(5)?,
        conversation_id: row.get(6)?,
        parent_id: row.get(7)?,
        source: row.get(8)?,
        counted: row.get::<_, i64>(9)? != 0,
        sources_json: row.get(10)?,
        outcome: row.get(11)?,
        tool_calls_json: row.get(12)?,
        answer_json: row.get(13)?,
        model: row.get(14)?,
        model_calls: row.get(15)?,
        tokens_in: row.get(16)?,
        tokens_cached: row.get(17)?,
        tokens_out: row.get(18)?,
        tokens_reasoning: row.get(19)?,
        finish_reason: row.get(20)?,
        total_millis: row.get(21)?,
        model_millis: row.get(22)?,
        tool_millis: row.get(23)?,
        calls_json: row.get(24)?,
    })
}

/// Writes one row of the log and returns its id.
///
/// # Errors
/// A database error; a value outside the lists the table checks.
pub fn insert(conn: &Connection, q: &NewQuestion<'_>) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO assistant_questions (asked_at, asked_day, asked_by, question, language, \
         conversation_id, parent_id, source, counted, sources_json, outcome, tool_calls_json, \
         answer_json, model, model_calls, tokens_in, tokens_cached, tokens_out, \
         tokens_reasoning, finish_reason, total_millis, model_millis, tool_millis, calls_json, \
         created_by, updated_by) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, \
         ?19, ?20, ?21, ?22, ?23, ?24, ?3, ?3)",
        params![
            q.asked_at,
            q.asked_day,
            q.asked_by,
            q.question,
            q.language,
            q.conversation_id,
            q.parent_id,
            q.source,
            i64::from(q.counted),
            q.sources_json,
            q.outcome,
            q.tool_calls_json,
            q.answer_json,
            q.model,
            q.model_calls,
            q.tokens_in,
            q.tokens_cached,
            q.tokens_out,
            q.tokens_reasoning,
            q.finish_reason,
            q.total_millis,
            q.model_millis,
            q.tool_millis,
            q.calls_json,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// One row by id.
///
/// # Errors
/// A database error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<QuestionRow>> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM assistant_questions WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()
}

/// How many counted questions the day has (PRD FR-7.81): the number the panel shows as "آج 5 / 20".
///
/// # Errors
/// A database error.
pub fn count_counted_on(conn: &Connection, day: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM assistant_questions WHERE asked_day = ?1 AND counted = 1",
        [day],
        |row| row.get(0),
    )
}

/// One page of the log, newest first (Code Standards B31).
///
/// # Errors
/// A database error.
pub fn list(conn: &Connection, offset: i64, limit: i64) -> rusqlite::Result<Vec<QuestionRow>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM assistant_questions ORDER BY asked_at DESC, id DESC \
         LIMIT ?1 OFFSET ?2"
    ))?;
    let rows = stmt.query_map(params![limit, offset], from_row)?;
    rows.collect()
}

/// How many rows the whole log has.
///
/// # Errors
/// A database error.
pub fn total(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM assistant_questions", [], |row| {
        row.get(0)
    })
}

/// The continuations of a question, oldest first.
///
/// # Errors
/// A database error.
pub fn continuations(conn: &Connection, parent_id: i64) -> rusqlite::Result<Vec<QuestionRow>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM assistant_questions WHERE parent_id = ?1 ORDER BY id"
    ))?;
    let rows = stmt.query_map([parent_id], from_row)?;
    rows.collect()
}

/// The names (Urdu and English) of the accounts the app seeds and of the expense heads: words the
/// name check of the assistant lets through, because they are also ordinary words (PRD FR-7.76
/// point 1: روکڑ, کمیشن آمدن, بجلی). Read only.
///
/// # Errors
/// A database error.
pub fn exempt_names(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT name_ur, name_en FROM accounts WHERE is_system = 1 \
         UNION SELECT name_ur, name_en FROM expense_heads",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut out = Vec::new();
    for row in rows {
        let (ur, en) = row?;
        out.push(ur);
        out.push(en);
    }
    Ok(out)
}
