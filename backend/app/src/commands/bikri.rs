//! The بکری screen (PRD 12, plans 12 and 13). Every command needs the signed-in Owner. The
//! preview and the freight split only read (Code Standards B29). `bikri_save` is the one action
//! that writes: one transaction (Code Standards B3). The reads of saved بکری only read.

use mandi_services::core::bikri::{
    BikriDetail, BikriFindInput, BikriGetInput, BikriListInput, BikriListRow, BikriNavigateInput,
    BikriPreview, BikriPreviewInput, BikriSaveInput, BikriSaved, FreightPart, SplitFreightInput,
    split_freight,
};
use mandi_services::{auth, bikri};

use super::run;
use crate::error::UiError;

/// Works out a whole typed بکری for the live summary: every line, commission and لاگا with its
/// working, the seller summary, the buyer bills, the live partner split, the cash note and all
/// errors and warnings, each at its field. Nothing is saved. The draft is text; the answer is
/// money in paisa.
#[tauri::command]
pub async fn bikri_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BikriPreviewInput,
) -> Result<BikriPreview, UiError> {
    run(app, "bikri_preview", move |ctx| {
        auth::require(ctx)?;
        bikri::preview(ctx, &input.draft)
    })
    .await
}

/// Splits the header's total freight over the lots by نگ received (PRD C17). Read only.
/// Errors: the money and quantity codes on `total_freight` and `lots[i].qty_received`,
/// `ERR_NO_LOTS` (`lots`).
#[tauri::command]
pub async fn bikri_split_freight<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SplitFreightInput,
) -> Result<Vec<FreightPart>, UiError> {
    run(app, "bikri_split_freight", move |ctx| {
        auth::require(ctx)?;
        Ok(split_freight(&input)?)
    })
    .await
}

/// Saves a بکری (PRD FR-12.8) in one transaction: the بکری with its lots and lines and their
/// stored values, one balanced journal entry (sale part and cash part), the partner share rows,
/// stock moves, analytics facts, bill matching, advance matching, the پختہ کھاتہ person lines
/// and the audit row; the draft of the screen is removed. All or nothing. It works the draft out
/// with the same function as `bikri_preview`, so the saved numbers are the previewed ones.
/// Errors: every error and code of the preview, each at its field (`lots[i].sales[j].qty`, ...);
/// `ERR_NO_LOTS` and `ERR_NO_SALE_LINES` on `lots`; `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE` on
/// `header.entry_date`; `ERR_SHARES_NOT_100` on `split`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`
/// (value `cash_after_paisa`) when the entry takes cash below 0 and `confirm_cash_below_zero` is
/// false.
#[tauri::command]
pub async fn bikri_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BikriSaveInput,
) -> Result<BikriSaved, UiError> {
    run(app, "bikri_save", move |ctx| {
        let actor = auth::require(ctx)?;
        bikri::save(ctx, &actor, &input)
    })
    .await
}

/// A saved بکری with its stored values, its postings and its partner split (never worked out
/// again). Read only. Errors: `ERR_NOT_FOUND` on `id`.
#[tauri::command]
pub async fn bikri_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BikriGetInput,
) -> Result<BikriDetail, UiError> {
    run(app, "bikri_get", move |ctx| {
        auth::require(ctx)?;
        bikri::get(ctx, input.id)
    })
    .await
}

/// The list of بکری of a day or a range, with its filters (PRD FR-12.7, آج کی بکری). Read only.
/// Errors: the date codes on `date`, `from`, `to`.
#[tauri::command]
pub async fn bikri_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BikriListInput,
) -> Result<Vec<BikriListRow>, UiError> {
    run(app, "bikri_list", move |ctx| {
        auth::require(ctx)?;
        bikri::list(ctx, &input)
    })
    .await
}

/// Finds بکری by number, truck, seller, day or item (PRD FR-12.13, F3). Newest first, at most 50.
/// Read only. Errors: the date codes on `date`.
#[tauri::command]
pub async fn bikri_find<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BikriFindInput,
) -> Result<Vec<BikriListRow>, UiError> {
    run(app, "bikri_find", move |ctx| {
        auth::require(ctx)?;
        bikri::find(ctx, &input)
    })
    .await
}

/// The id of the first, previous, next or last بکری by number (PRD FR-12.13); null when there is
/// none in that direction. Read only. Errors: `ERR_NOT_FOUND` on `current_id`.
#[tauri::command]
pub async fn bikri_navigate<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BikriNavigateInput,
) -> Result<Option<i64>, UiError> {
    run(app, "bikri_navigate", move |ctx| {
        auth::require(ctx)?;
        bikri::navigate(ctx, &input)
    })
    .await
}
