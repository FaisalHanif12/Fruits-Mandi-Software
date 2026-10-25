//! Fruit taken (PRD 16.7, plan 17d): a partner takes fruit home, or fruit is served to the shop's
//! guests. Every command needs the signed-in Owner. The reads and the preview only read (Code
//! Standards B29); the others are one transaction each (Code Standards B3). Until plan 17d T2 each
//! command answers `ERR_NOT_IMPLEMENTED`.

use mandi_services::core::fruit::{
    FruitCancelInput, FruitCancelled, FruitEditInput, FruitEdited, FruitGetInput, FruitInput,
    FruitList, FruitListInput, FruitLots, FruitLotsInput, FruitPreview, FruitRow, FruitSaved,
};
use mandi_services::{auth, fruit};

use super::run;
use crate::error::UiError;

/// Works out a fruit entry before it is saved: the value, each partner's part, who is credited and
/// the lot's balance after. Read only. The mistakes come back inside the preview, each at its field.
#[tauri::command]
pub async fn fruit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FruitInput,
) -> Result<FruitPreview, UiError> {
    run(app, "fruit_preview", move |ctx| {
        auth::require(ctx)?;
        fruit::preview(ctx, &input)
    })
    .await
}

/// Saves a fruit entry (PRD FR-16.10, FR-16.11). Errors: `ERR_PARTNER_REQUIRED` and
/// `ERR_NOT_FOUND` on `partner_id`; the date codes on `date`; `ERR_REQUIRED` and
/// `ERR_LOT_NOT_FOUND` on `lot_id`; `ERR_GARDEN_CLOSED` on `lot_id`; `ERR_REQUIRED` on
/// `agent_account_id`; the quantity codes on `qty`; `ERR_QTY_EXCEEDS_LOT` on `qty` (value
/// `available`); `ERR_MUST_BE_MORE_THAN_ZERO` on `rate`.
#[tauri::command]
pub async fn fruit_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FruitInput,
) -> Result<FruitSaved, UiError> {
    run(app, "fruit_save", move |ctx| {
        let actor = auth::require(ctx)?;
        fruit::save(ctx, &actor, &input)
    })
    .await
}

/// Edits an entry by reversal while its day is open (PRD FR-16.12): the old posting is reversed and
/// the new one is posted. Errors as `fruit_save`, and `ERR_NOT_FOUND` on `id`,
/// `ERR_VERSION_CHANGED` on `version`, `ERR_REASON_REQUIRED` on `reason`, `ERR_FRUIT_CANCELLED` and
/// `ERR_DATE_CLOSED` on `id`.
#[tauri::command]
pub async fn fruit_edit<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FruitEditInput,
) -> Result<FruitEdited, UiError> {
    run(app, "fruit_edit", move |ctx| {
        let actor = auth::require(ctx)?;
        fruit::edit(ctx, &actor, &input)
    })
    .await
}

/// Cancels an entry by reversal while its day is open (PRD FR-16.12). Errors: `ERR_NOT_FOUND`,
/// `ERR_FRUIT_CANCELLED` and `ERR_DATE_CLOSED` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_REASON_REQUIRED` on `reason`.
#[tauri::command]
pub async fn fruit_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FruitCancelInput,
) -> Result<FruitCancelled, UiError> {
    run(app, "fruit_cancel", move |ctx| {
        let actor = auth::require(ctx)?;
        fruit::cancel(ctx, &actor, &input)
    })
    .await
}

/// One entry, with everything the edit form needs. Read only. `ERR_NOT_FOUND` on `id`.
#[tauri::command]
pub async fn fruit_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FruitGetInput,
) -> Result<FruitRow, UiError> {
    run(app, "fruit_get", move |ctx| {
        auth::require(ctx)?;
        fruit::get(ctx, &input)
    })
    .await
}

/// One page of entries with the totals over the whole filter (PRD FR-19.11). Read only. The filters
/// are partner or guests, item, source, dates and status.
#[tauri::command]
pub async fn fruit_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FruitListInput,
) -> Result<FruitList, UiError> {
    run(app, "fruit_list", move |ctx| {
        auth::require(ctx)?;
        fruit::list(ctx, &input)
    })
    .await
}

/// The lots fruit can be taken from: a balance above 0, not trip goods, not a lot of a closed
/// garden (PRD FR-16.11). Read only.
#[tauri::command]
pub async fn fruit_lots<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FruitLotsInput,
) -> Result<FruitLots, UiError> {
    run(app, "fruit_lots", move |ctx| {
        auth::require(ctx)?;
        fruit::lots(ctx, &input)
    })
    .await
}
