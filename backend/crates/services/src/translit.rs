//! Name suggestions and learned words (PRD FR-7.22, FR-7.24, FR-7.26, Tech Stack 4.4, plan 05).
//!
//! The engine in `mandi-core` is pure. This module loads the learned pairs from the
//! `translit_pairs` table, keeps them in memory, and passes them in. A learned pair never
//! changes a name that is already saved (Code Standards B28): it only changes what is offered.
//!
//! Suggesting reads and never writes. Learning and removing write one row and one audit row in
//! one transaction.

use std::sync::Arc;

use mandi_db::rusqlite::{Connection, OptionalExtension, params};
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::translit::{
    self, Direction, Kind, LearnedPair, MAX_INPUT_CHARS, Suggestion, WordList,
};
use mandi_domain::core::validate::{self, FieldErrors};
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;

/// The most characters of a text to suggest for. The engine reads no more than this.
pub const MAX_SUGGEST_CHARS: usize = MAX_INPUT_CHARS;
/// The most letters in a learned word, English or Urdu.
pub const MAX_WORD_LETTERS: usize = 40;

/// One learned pair, as the Owner sees it in Settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnedRow {
    /// Row id.
    pub id: i64,
    /// English spelling, lower case.
    pub roman: String,
    /// Urdu spelling.
    pub urdu: String,
    /// What kind of word.
    pub kind: Kind,
    /// How often it was chosen.
    pub use_count: u32,
    /// UTC time it was last chosen.
    pub last_used_at: Option<String>,
}

/// A pair to learn, as typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPair {
    /// English spelling, one word.
    pub roman: String,
    /// Urdu spelling, one word.
    pub urdu: String,
    /// Kind of word.
    pub kind: Kind,
}

fn row_from(row: &mandi_db::rusqlite::Row<'_>) -> Result<LearnedRow, mandi_db::rusqlite::Error> {
    let kind: String = row.get(3)?;
    let count: i64 = row.get(4)?;
    Ok(LearnedRow {
        id: row.get(0)?,
        roman: row.get(1)?,
        urdu: row.get(2)?,
        kind: Kind::parse(&kind).unwrap_or(Kind::Word),
        use_count: u32::try_from(count).unwrap_or(u32::MAX),
        last_used_at: row.get(5)?,
    })
}

const ROW_COLUMNS: &str = "id, roman, urdu, kind, use_count, last_used_at";

fn load_all(conn: &Connection) -> Result<Vec<LearnedRow>, ServiceError> {
    let mut statement = conn.prepare(&format!(
        "SELECT {ROW_COLUMNS} FROM translit_pairs
         ORDER BY use_count DESC, COALESCE(last_used_at, '') DESC, id"
    ))?;
    let rows = statement
        .query_map([], row_from)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// The learned pairs in memory, loaded from the database the first time.
pub(crate) fn learned_pairs(ctx: &Context) -> Result<Arc<Vec<LearnedPair>>, ServiceError> {
    let mut slot = ctx.learned_lock();
    if let Some(pairs) = slot.as_ref() {
        return Ok(Arc::clone(pairs));
    }
    let rows = ctx.db.read(load_all)?;
    let pairs: Arc<Vec<LearnedPair>> = Arc::new(
        rows.into_iter()
            .map(|r| LearnedPair {
                roman: r.roman,
                urdu: r.urdu,
                use_count: r.use_count,
            })
            .collect(),
    );
    *slot = Some(Arc::clone(&pairs));
    Ok(pairs)
}

/// Forgets what is in memory, so the next use reads the table again. The restore of a backup
/// calls this (the whole data is replaced).
pub fn reload(ctx: &Context) {
    *ctx.learned_lock() = None;
}

/// Gets the word list trained and the learned pairs loaded, so the first key press is not the
/// slow one (NFR-SPD-20). Failures are ignored: the first real use tries again.
pub fn warm_up(ctx: &Context) {
    let _ = learned_pairs(ctx);
    let _ = WordList::embedded().training_report();
}

/// Up to three suggestions for the typed word or phrase (PRD FR-7.22). Each says where it came
/// from, so the screen can mark a spelling that rests on the rules alone (PRD FR-7.66). Reads
/// only.
///
/// # Errors
/// `ERR_TEXT_TOO_LONG` on `text` (value `max`) for more than [`MAX_SUGGEST_CHARS`] characters.
pub fn suggest(
    ctx: &Context,
    text: &str,
    direction: Direction,
) -> Result<Vec<Suggestion>, ServiceError> {
    // Count with a cap, so a huge text is refused without reading all of it.
    if text
        .chars()
        .take(MAX_SUGGEST_CHARS.saturating_add(1))
        .count()
        > MAX_SUGGEST_CHARS
    {
        return Err(ServiceError::reject_with(
            ErrorCode::ErrTextTooLong,
            Some("text"),
            "max",
            i64::try_from(MAX_SUGGEST_CHARS).unwrap_or(i64::MAX),
        ));
    }
    let learned = learned_pairs(ctx)?;
    let list = WordList::embedded();
    Ok(translit::suggest(text, &learned, list, direction))
}

/// A learned English word: letters only, one word, lower case.
fn english_word(field: &str, input: &str) -> Result<String, FieldError> {
    let cleaned = validate::name_en(field, input)?;
    let word = cleaned.to_lowercase();
    if !word.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(FieldError::new(ErrorCode::ErrNameEnScript, field));
    }
    if word.chars().count() > MAX_WORD_LETTERS {
        return Err(too_long(field));
    }
    Ok(word)
}

/// A learned Urdu word: Urdu letters (and marks) only, one word.
fn urdu_word(field: &str, input: &str) -> Result<String, FieldError> {
    let cleaned = validate::name_ur(field, input)?;
    if !cleaned.chars().all(|c| {
        mandi_domain::core::text::is_urdu_letter(c) || mandi_domain::core::text::is_urdu_mark(c)
    }) {
        return Err(FieldError::new(ErrorCode::ErrNameUrScript, field));
    }
    if cleaned.chars().count() > MAX_WORD_LETTERS {
        return Err(too_long(field));
    }
    Ok(cleaned)
}

fn too_long(field: &str) -> FieldError {
    FieldError::new(ErrorCode::ErrNameTooLong, field).with(
        "max",
        ErrorValue::Number(i64::try_from(MAX_WORD_LETTERS).unwrap_or(i64::MAX)),
    )
}

/// Remembers that this English word was written this way in Urdu (PRD FR-7.26). A pair that is
/// already known counts one more use. The row and its audit row are written together.
///
/// # Errors
/// `ERR_REQUIRED`, `ERR_NAME_EN_SCRIPT`, `ERR_NAME_TOO_LONG` on `roman`; `ERR_REQUIRED`,
/// `ERR_NAME_UR_SCRIPT`, `ERR_NAME_TOO_LONG` on `urdu`; both together when both are wrong.
pub fn learn(ctx: &Context, actor: &Actor, input: &NewPair) -> Result<LearnedRow, ServiceError> {
    let row = ctx.db.write(|tx| learn_in(ctx, actor, tx, input))?;
    remember(ctx, &row);
    Ok(row)
}

/// The same as [`learn`], inside a transaction the caller already has. A save that learns
/// the pairs the user confirmed (see the `spelling` module) calls this, so the save and the
/// learning are one action. The copy in memory is updated by [`remember`] after the commit,
/// never before: a save that is rolled back must not leave a pair in memory.
///
/// # Errors
/// As [`learn`].
pub(crate) fn learn_in(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    input: &NewPair,
) -> Result<LearnedRow, ServiceError> {
    let mut errors = FieldErrors::default();
    let roman = errors.take(english_word("roman", &input.roman));
    let urdu = errors.take(urdu_word("urdu", &input.urdu));
    errors.finish()?;
    let (Some(roman), Some(urdu)) = (roman, urdu) else {
        return Err(ServiceError::Data(
            "learn: a checked field is missing".to_owned(),
        ));
    };
    let now = ctx.now_text();
    let old: Option<(i64, i64)> = tx
        .query_row(
            "SELECT id, use_count FROM translit_pairs WHERE roman = ?1 AND urdu = ?2",
            params![roman, urdu],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let id = match old {
        Some((id, _)) => {
            tx.execute(
                "UPDATE translit_pairs SET use_count = use_count + 1, last_used_at = ?1,
                     updated_at = ?1, updated_by = ?2, version = version + 1 WHERE id = ?3",
                params![now, actor.user_id, id],
            )?;
            id
        }
        None => {
            tx.execute(
                "INSERT INTO translit_pairs
                     (roman, urdu, kind, use_count, last_used_at, created_at, created_by,
                      updated_at, updated_by)
                 VALUES (?1, ?2, ?3, 1, ?4, ?4, ?5, ?4, ?5)",
                params![roman, urdu, input.kind.as_str(), now, actor.user_id],
            )?;
            tx.last_insert_rowid()
        }
    };
    let row = tx.query_row(
        &format!("SELECT {ROW_COLUMNS} FROM translit_pairs WHERE id = ?1"),
        [id],
        row_from,
    )?;
    let mut event = Event::new(Some(actor.user_id), "learn", "translit_pair")
        .entity_id(id)
        .new_value(json!({
            "roman": row.roman,
            "urdu": row.urdu,
            "kind": row.kind.as_str(),
            "use_count": row.use_count,
        }));
    if let Some((_, count)) = old {
        event = event.old(json!({ "use_count": count }));
    }
    ctx.db.auditor().write(tx, &event.into_entry())?;
    Ok(row)
}

/// Puts a pair that was written to the table into the copy in memory (only if the copy was ever
/// loaded). Call it after the transaction that wrote the row has committed.
pub(crate) fn remember(ctx: &Context, row: &LearnedRow) {
    if let Some(shared) = ctx.learned_lock().as_mut() {
        let pairs = Arc::make_mut(shared);
        match pairs
            .iter_mut()
            .find(|p| p.roman == row.roman && p.urdu == row.urdu)
        {
            Some(pair) => pair.use_count = row.use_count,
            None => pairs.push(LearnedPair {
                roman: row.roman.clone(),
                urdu: row.urdu.clone(),
                use_count: row.use_count,
            }),
        }
    }
}

/// All learned pairs, the most used first (Owner, Settings list).
///
/// # Errors
/// `ServiceError::Db`.
pub fn list(ctx: &Context) -> Result<Vec<LearnedRow>, ServiceError> {
    ctx.db.read(load_all)
}

/// One page of the learned pairs, the most used first, with how many there are and how many times
/// they were all used (PRD FR-19.11, B31).
///
/// # Errors
/// `ServiceError::Db`.
pub fn list_page(
    ctx: &Context,
    offset: u32,
    limit: u32,
) -> Result<(Vec<LearnedRow>, i64, i64), ServiceError> {
    let all = list(ctx)?;
    let total_count =
        i64::try_from(all.len()).map_err(|_| ServiceError::Data("too many rows".to_owned()))?;
    let mut uses = 0_i64;
    for row in &all {
        uses = uses
            .checked_add(i64::from(row.use_count))
            .ok_or_else(|| ServiceError::reject(ErrorCode::ErrOverflow))?;
    }
    Ok((crate::page::slice(all, offset, limit), total_count, uses))
}

/// Removes one learned pair (PRD FR-7.26, Owner). Names already saved are not touched.
///
/// # Errors
/// `ERR_NOT_FOUND` if there is no such pair.
pub fn remove(ctx: &Context, actor: &Actor, id: i64) -> Result<(), ServiceError> {
    let removed = ctx.db.write(|tx| {
        let row = tx
            .query_row(
                &format!("SELECT {ROW_COLUMNS} FROM translit_pairs WHERE id = ?1"),
                [id],
                row_from,
            )
            .optional()?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
        tx.execute("DELETE FROM translit_pairs WHERE id = ?1", [id])?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "remove", "translit_pair")
                .entity_id(id)
                .old(json!({
                    "roman": row.roman,
                    "urdu": row.urdu,
                    "kind": row.kind.as_str(),
                    "use_count": row.use_count,
                }))
                .into_entry(),
        )?;
        Ok::<_, ServiceError>(row)
    })?;
    if let Some(shared) = ctx.learned_lock().as_mut() {
        let pairs = Arc::make_mut(shared);
        pairs.retain(|p| !(p.roman == removed.roman && p.urdu == removed.urdu));
    }
    Ok(())
}
