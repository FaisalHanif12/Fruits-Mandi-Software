//! Goods bought on market trips (PRD 12.10, plan 16b): purchases, their stock and result, and the
//! write-off of spoiled goods. Every command needs the signed-in Owner. The preview, the list and
//! the get only read (Code Standards B29); the others are one transaction each (Code Standards B3).

use mandi_services::core::purchases::{
    PurchaseCancelInput, PurchaseCancelled, PurchaseGetInput, PurchaseInput, PurchasePreview,
    PurchaseRow, PurchaseSaved, PurchaseWriteOffDone, PurchaseWriteOffInput,
    PurchaseWriteOffUndoInput, PurchaseWriteOffUndone, PurchasesList, PurchasesListInput,
};
use mandi_services::{auth, purchases};

use super::run;
use crate::error::UiError;

/// Works out a typed purchase for the live screen: the value, the paid-now amount, the cost for
/// each نگ, each partner's part (shop or pocket), each with its working, and the cash note. Nothing
/// is saved. Errors and warnings come back inside the preview, each at its field.
#[tauri::command]
pub async fn purchases_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PurchaseInput,
) -> Result<PurchasePreview, UiError> {
    run(app, "purchases_preview", move |ctx| {
        auth::require(ctx)?;
        purchases::preview(ctx, &input)
    })
    .await
}

/// Saves a purchase (PRD FR-12.24) in one transaction: the purchase, the partners' parts, the
/// journal entry for the parts paid from the shop's cash, the stock move and the audit row.
/// Errors: every error of the preview at its field; `ERR_TRIP_CLOSED` and `ERR_PURCHASE_BEFORE_TRIP`;
/// `ERR_DATE_CLOSED`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
#[tauri::command]
pub async fn purchases_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PurchaseInput,
) -> Result<PurchaseSaved, UiError> {
    run(app, "purchases_save", move |ctx| {
        let actor = auth::require(ctx)?;
        purchases::save(ctx, &actor, &input)
    })
    .await
}

/// Cancels a purchase by reversal, only while nothing of it was sold or written off.
/// Errors: `ERR_NOT_FOUND`, `ERR_PURCHASE_CANCELLED`, `ERR_PURCHASE_HAS_SALES`,
/// `ERR_PURCHASE_CARRIES_TRIP_COST` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `id`.
#[tauri::command]
pub async fn purchases_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PurchaseCancelInput,
) -> Result<PurchaseCancelled, UiError> {
    run(app, "purchases_cancel", move |ctx| {
        let actor = auth::require(ctx)?;
        purchases::cancel(ctx, &actor, &input)
    })
    .await
}

/// One page of purchases with the totals over the whole filter. Read only.
#[tauri::command]
pub async fn purchases_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PurchasesListInput,
) -> Result<PurchasesList, UiError> {
    run(app, "purchases_list", move |ctx| {
        auth::require(ctx)?;
        purchases::list(ctx, &input)
    })
    .await
}

/// One purchase with its parts, the own lots that sold it and its write-offs. Read only.
#[tauri::command]
pub async fn purchases_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PurchaseGetInput,
) -> Result<PurchaseRow, UiError> {
    run(app, "purchases_get", move |ctx| {
        auth::require(ctx)?;
        purchases::get(ctx, &input)
    })
    .await
}

/// Writes off spoiled trip goods (PRD FR-12.28): the qty and its cost (C21) come off the purchase.
/// Nothing is posted. Errors: `ERR_NOT_FOUND`, `ERR_PURCHASE_CANCELLED` on `purchase_id`; the
/// quantity codes on `qty`; `ERR_WRITE_OFF_EXCEEDS_BALANCE` (value `available`) on `qty`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `date`.
#[tauri::command]
pub async fn purchases_write_off<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PurchaseWriteOffInput,
) -> Result<PurchaseWriteOffDone, UiError> {
    run(app, "purchases_write_off", move |ctx| {
        let actor = auth::require(ctx)?;
        purchases::write_off(ctx, &actor, &input)
    })
    .await
}

/// Undoes a write-off of trip goods, while its day is open. Errors: `ERR_NOT_FOUND` on `move_id`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `move_id`.
#[tauri::command]
pub async fn purchases_write_off_undo<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PurchaseWriteOffUndoInput,
) -> Result<PurchaseWriteOffUndone, UiError> {
    run(app, "purchases_write_off_undo", move |ctx| {
        let actor = auth::require(ctx)?;
        purchases::write_off_undo(ctx, &actor, &input)
    })
    .await
}
