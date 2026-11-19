//! English letters to Urdu and back, and the learned words (PRD 7.4, plan 05 Section 4).
//!
//! Pickers and search never call these: only Urdu text fields convert (F41).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::run;
use crate::error::UiError;
use mandi_services::core::paging::SortInput;
use mandi_services::core::translit::{Direction, Kind, Suggestion};
use mandi_services::{auth, translit};

/// A word or a phrase to convert.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SuggestInput {
    /// The text typed: one word (the current word of the field) or a whole phrase. Up to 500
    /// characters. Empty gives no suggestions.
    pub text: String,
    /// `to_urdu` for English letters, `to_english` for Urdu letters.
    pub direction: Direction,
}

/// Up to three suggestions, the best first (PRD FR-7.22).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Suggestions {
    /// The suggested spellings, each with where it came from (`learned`, `word_list` or
    /// `rules`) and the same for each of its words. A phrase gives phrases. A word whose source
    /// is `rules` is marked "check the spelling" (PRD FR-7.66). Empty if there is nothing to
    /// suggest.
    pub items: Vec<Suggestion>,
}

/// A pair the user chose or corrected, to be offered first next time (PRD FR-7.26).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct LearnInput {
    /// The English spelling, one word, letters only.
    pub roman: String,
    /// The Urdu spelling, one word, Urdu letters only.
    pub urdu: String,
    /// What kind of word it is.
    pub kind: Kind,
}

/// One learned pair, for the Settings list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PairRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The English spelling, lower case.
    pub roman: String,
    /// The Urdu spelling.
    pub urdu: String,
    /// The kind of word.
    pub kind: Kind,
    /// How often it was chosen.
    pub use_count: u32,
    /// UTC time it was last chosen. Null if never.
    pub last_used_at: Option<String>,
}

/// The page of learned pairs to list.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PairsListInput {
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives a page of 100 (Code Standards B31).
    #[serde(default)]
    pub limit: u32,
    /// Only pairs whose English or Urdu spelling holds this text. The count and the uses are over
    /// these pairs.
    #[serde(default)]
    pub search: Option<String>,
    /// The column to sort the whole list by: `roman`, `urdu`, `kind`, `use_count` or
    /// `last_used_at`. Null is the most used first (plan 38 R4).
    #[serde(default)]
    pub sort: Option<SortInput>,
}

/// One page of learned pairs with the totals over every pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PairsList {
    /// This page, most used first.
    pub rows: Vec<PairRow>,
    /// How many pairs there are.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over every pair.
    pub totals: PairsTotals,
}

/// The totals of the learned pairs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PairsTotals {
    /// How many times all the pairs were chosen.
    #[ts(type = "number")]
    pub use_count: i64,
}

/// Which learned pair to remove.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct RemovePairInput {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
}

impl From<translit::LearnedRow> for PairRow {
    fn from(row: translit::LearnedRow) -> Self {
        Self {
            id: row.id,
            roman: row.roman,
            urdu: row.urdu,
            kind: row.kind,
            use_count: row.use_count,
            last_used_at: row.last_used_at,
        }
    }
}

/// Up to three suggestions for the typed word or phrase. Never writes.
/// Errors: `ERR_TEXT_TOO_LONG` (`text`, value `max`).
#[tauri::command]
pub async fn translit_suggest<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SuggestInput,
) -> Result<Suggestions, UiError> {
    run(app, "translit_suggest", move |ctx| {
        auth::require(ctx)?;
        translit::suggest(ctx, &input.text, input.direction).map(|items| Suggestions { items })
    })
    .await
}

/// Remembers that this English word was written this way in Urdu.
/// Errors: `ERR_REQUIRED`, `ERR_NAME_EN_SCRIPT`, `ERR_NAME_TOO_LONG` (`roman`);
/// `ERR_REQUIRED`, `ERR_NAME_UR_SCRIPT`, `ERR_NAME_TOO_LONG` (`urdu`).
#[tauri::command]
pub async fn translit_learn<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: LearnInput,
) -> Result<(), UiError> {
    run(app, "translit_learn", move |ctx| {
        let actor = auth::require(ctx)?;
        let pair = translit::NewPair {
            roman: input.roman,
            urdu: input.urdu,
            kind: input.kind,
        };
        translit::learn(ctx, &actor, &pair).map(|_| ())
    })
    .await
}

/// One page of the learned pairs, most used first, with how many there are and how many times
/// they were all used (PRD FR-19.11).
#[tauri::command]
pub async fn translit_pairs_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PairsListInput,
) -> Result<PairsList, UiError> {
    run(app, "translit_pairs_list", move |ctx| {
        auth::require(ctx)?;
        let (rows, total_count, use_count) = translit::list_page_with(
            ctx,
            input.offset,
            input.limit,
            input.search.as_deref(),
            input.sort.as_ref(),
        )?;
        Ok(PairsList {
            rows: rows.into_iter().map(PairRow::from).collect(),
            total_count,
            totals: PairsTotals { use_count },
        })
    })
    .await
}

/// Removes one learned pair. Errors: `ERR_NOT_FOUND`.
#[tauri::command]
pub async fn translit_pairs_remove<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: RemovePairInput,
) -> Result<(), UiError> {
    run(app, "translit_pairs_remove", move |ctx| {
        let actor = auth::require(ctx)?;
        translit::remove(ctx, &actor, input.id)
    })
    .await
}
