//! The بکری screen (PRD 12, plans 12 and 13). Every command needs the signed-in Owner. The
//! preview and the freight split only read (Code Standards B29). `bikri_save` is the one action
//! that writes: one transaction (Code Standards B3). The reads of saved بکری only read.

use mandi_services::core::bikri::{
    BikriCancelled, BikriDetail, BikriEdited, BikriFindInput, BikriGetInput, BikriList,
    BikriListInput, BikriListRow, BikriNavigateInput, BikriPreview, BikriPreviewInput,
    BikriSaveInput, BikriSaved, CancelInput, CancelPreview, CancelPreviewInput, EditOpenInput,
    EditPreviewInput, EditSaveInput, EditableBikri, FreightPart, LaterSaleInput,
    LaterSaleSaveInput, LaterSaleSaved, SplitFreightInput, StockList, StockListInput, UndoInput,
    UndoResult, WriteOffDone, WriteOffInput, WriteOffUndoInput, WriteOffUndone, split_freight,
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

/// One page of the list of بکری of a day or a range, with its filters and the totals over the
/// whole filter (PRD FR-12.7, FR-19.11, آج کی بکری). Read only.
/// Errors: the date codes on `date`, `from`, `to`.
#[tauri::command]
pub async fn bikri_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BikriListInput,
) -> Result<BikriList, UiError> {
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

// ---- Plan 16: goods left to sell, later sales, edit, cancel, undo, write-off ----
//
// Every command needs the signed-in Owner. The types and the codes are the contract
// (`mandi_core::bikri::edit`); each command is one service call.

/// The lots that still have goods left to sell, with the write-offs that can be undone (PRD
/// FR-12.14, FR-12.16). Read only.
#[tauri::command]
pub async fn bikri_stock_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: StockListInput,
) -> Result<StockList, UiError> {
    run(app, "bikri_stock_list", move |ctx| {
        auth::require(ctx)?;
        bikri::stock_list(ctx, &input)
    })
    .await
}

/// Works out a later sale of the goods left in one lot for the live summary (PRD FR-12.17). It
/// shares its working with `bikri_later_sale_save`. Nothing is saved.
#[tauri::command]
pub async fn bikri_later_sale_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: LaterSaleInput,
) -> Result<BikriPreview, UiError> {
    run(app, "bikri_later_sale_preview", move |ctx| {
        auth::require(ctx)?;
        bikri::later_sale_preview(ctx, &input)
    })
    .await
}

/// Saves a later sale as a new posting of the lot's بکری, on its own day (PRD FR-12.17): one
/// transaction, no lot charges again.
#[tauri::command]
pub async fn bikri_later_sale_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: LaterSaleSaveInput,
) -> Result<LaterSaleSaved, UiError> {
    run(app, "bikri_later_sale_save", move |ctx| {
        let actor = auth::require(ctx)?;
        bikri::later_sale_save(ctx, &actor, &input)
    })
    .await
}

/// Loads one posting of a saved بکری as a draft with its stored values, to edit (PRD FR-12.18).
/// Read only.
#[tauri::command]
pub async fn bikri_open_for_edit<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: EditOpenInput,
) -> Result<EditableBikri, UiError> {
    run(app, "bikri_open_for_edit", move |ctx| {
        auth::require(ctx)?;
        bikri::open_for_edit(ctx, &input)
    })
    .await
}

/// Works out an edited posting for the live summary (PRD FR-12.20). It shares its working with
/// `bikri_edit_save`. Nothing is saved.
#[tauri::command]
pub async fn bikri_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: EditPreviewInput,
) -> Result<BikriPreview, UiError> {
    run(app, "bikri_edit_preview", move |ctx| {
        auth::require(ctx)?;
        bikri::edit_preview(ctx, &input)
    })
    .await
}

/// Saves an edit (PRD FR-12.19): a reversal of the posting and a new posting on the same day, in
/// one transaction. Lines the user did not change keep their stored values exactly.
#[tauri::command]
pub async fn bikri_edit_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: EditSaveInput,
) -> Result<BikriEdited, UiError> {
    run(app, "bikri_edit_save", move |ctx| {
        let actor = auth::require(ctx)?;
        bikri::edit_save(ctx, &actor, &input)
    })
    .await
}

/// What a cancel would reverse and do to cash, for the confirm box (PRD FR-7.8). Read only.
#[tauri::command]
pub async fn bikri_cancel_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CancelPreviewInput,
) -> Result<CancelPreview, UiError> {
    run(app, "bikri_cancel_preview", move |ctx| {
        auth::require(ctx)?;
        bikri::cancel_preview(ctx, &input)
    })
    .await
}

/// Cancels a بکری with a reason (PRD FR-12.22): every posting is reversed in one transaction, the
/// number is kept, and matching is worked out again.
#[tauri::command]
pub async fn bikri_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CancelInput,
) -> Result<BikriCancelled, UiError> {
    run(app, "bikri_cancel", move |ctx| {
        let actor = auth::require(ctx)?;
        bikri::cancel(ctx, &actor, &input)
    })
    .await
}

/// Undoes a بکری straight after its save (PRD FR-7.47): a cancel with a fixed reason, giving back
/// a draft with the same values.
#[tauri::command]
pub async fn bikri_undo<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: UndoInput,
) -> Result<UndoResult, UiError> {
    run(app, "bikri_undo", move |ctx| {
        let actor = auth::require(ctx)?;
        bikri::undo(ctx, &actor, &input)
    })
    .await
}

/// Writes off goods left in a lot with a reason (PRD FR-12.16): a stock move, no journal entry.
#[tauri::command]
pub async fn bikri_write_off<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: WriteOffInput,
) -> Result<WriteOffDone, UiError> {
    run(app, "bikri_write_off", move |ctx| {
        let actor = auth::require(ctx)?;
        bikri::write_off(ctx, &actor, &input)
    })
    .await
}

/// Undoes a write-off with a reason (PRD FR-12.16): the opposite stock move is recorded.
#[tauri::command]
pub async fn bikri_write_off_undo<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: WriteOffUndoInput,
) -> Result<WriteOffUndone, UiError> {
    run(app, "bikri_write_off_undo", move |ctx| {
        let actor = auth::require(ctx)?;
        bikri::write_off_undo(ctx, &actor, &input)
    })
    .await
}
