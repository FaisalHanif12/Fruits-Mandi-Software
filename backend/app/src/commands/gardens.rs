//! Gardens (PRD 12.11, plan 17c): make a garden with its partners and shares, pay its installments
//! and running costs, sell its goods in another shop, read its running balance and close it so its
//! money goes to its partners (C26). Every command needs the signed-in Owner. The reads and the
//! previews only read (Code Standards B29); the others are one transaction each (Code Standards B3).

use mandi_services::core::arrivals::{
    ArrivalCancelInput, ArrivalCancelled, ArrivalEntryInput, ArrivalEntryPreview,
    ArrivalEntrySaved, ArrivalGetInput, ArrivalInput, ArrivalRow, ArrivalSaved,
    ArrivalWriteOffDone, ArrivalWriteOffInput, ArrivalWriteOffUndoInput, ArrivalWriteOffUndone,
    ArrivalsList, ArrivalsListInput, SellDateInput, SellDateSet,
};
use mandi_services::core::gardens::{
    GardenCloseInput, GardenClosePreview, GardenClosed, GardenCreatePreview, GardenCreated,
    GardenDefaults, GardenDetail, GardenEntryCancelInput, GardenEntryCancelled, GardenEntryInput,
    GardenEntryPreview, GardenEntrySaved, GardenGetInput, GardenInput, GardenReopenInput,
    GardenReopened, GardenRow, GardenSetPartnersInput, GardensList, GardensListInput,
    GardensSummary,
};
use mandi_services::core::split::{SplitInput, SplitSet};
use mandi_services::{auth, gardens};

use super::run;
use crate::error::UiError;

/// The form of a new garden: every active partner with the firm's share and the source shop (PRD
/// A56). Read only.
#[tauri::command]
pub async fn gardens_defaults<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<GardenDefaults, UiError> {
    run(app, "gardens_defaults", move |ctx| {
        auth::require(ctx)?;
        gardens::defaults(ctx)
    })
    .await
}

/// Works out the first installment of a new garden, nothing saved. Read only. The errors come back
/// inside the preview.
#[tauri::command]
pub async fn gardens_create_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenInput,
) -> Result<GardenCreatePreview, UiError> {
    run(app, "gardens_create_preview", move |ctx| {
        auth::require(ctx)?;
        gardens::create_preview(ctx, &input)
    })
    .await
}

/// Makes a garden (PRD FR-12.29, FR-12.34): shares first, then sources, then the details. It gets
/// its own account under control 0306 and, if `paid_now` is typed, its first installment. Errors:
/// the name codes on `name_ur` and `name_en`; `ERR_SPELLING_UNCONFIRMED`; the date codes on `date`;
/// `ERR_MUST_BE_MORE_THAN_ZERO` on `rent`; the percent codes on `partners[i].share`;
/// `ERR_NOT_FOUND` and `ERR_PARTNER_NOT_LISTED` on `partners[i].partner_id`; `ERR_SHARES_NOT_100` on
/// `partners` (value `total_bps`); `ERR_SERIES_FULL`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value
/// `cash_after_paisa`) when the shop parts of the first installment take cash below 0.
#[tauri::command]
pub async fn gardens_create<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenInput,
) -> Result<GardenCreated, UiError> {
    run(app, "gardens_create", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::create(ctx, &actor, &input)
    })
    .await
}

/// Changes the partners and shares of a garden that has no entry yet. Errors: as `gardens_create`;
/// `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`; `ERR_VENTURE_SHARES_LOCKED` on
/// `partners` after the first entry (PRD FR-12.29).
#[tauri::command]
pub async fn gardens_set_partners<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenSetPartnersInput,
) -> Result<GardenRow, UiError> {
    run(app, "gardens_set_partners", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::set_partners(ctx, &actor, &input)
    })
    .await
}

/// One page of gardens with the totals over the whole filter (PRD FR-19.11). Read only.
#[tauri::command]
pub async fn gardens_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardensListInput,
) -> Result<GardensList, UiError> {
    run(app, "gardens_list", move |ctx| {
        auth::require(ctx)?;
        gardens::list(ctx, &input)
    })
    .await
}

/// One garden with its figures, a page of its history with the running balance after each row, and
/// each partner's money in and back (PRD FR-12.37). Read only. Errors: `ERR_NOT_FOUND` on `id`.
#[tauri::command]
pub async fn gardens_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenGetInput,
) -> Result<GardenDetail, UiError> {
    run(app, "gardens_get", move |ctx| {
        auth::require(ctx)?;
        gardens::get(ctx, &input)
    })
    .await
}

/// The open gardens with their running balances, for the dashboard tile (PRD 9.1). Read only.
#[tauri::command]
pub async fn gardens_summary<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<GardensSummary, UiError> {
    run(app, "gardens_summary", move |ctx| {
        auth::require(ctx)?;
        gardens::summary(ctx)
    })
    .await
}

/// Works out an installment, a running cost or a sale in another shop for the live screen, nothing
/// saved (Code Standards B5: the save uses the same function). Read only; the errors come back
/// inside the preview.
#[tauri::command]
pub async fn gardens_entry_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenEntryInput,
) -> Result<GardenEntryPreview, UiError> {
    run(app, "gardens_entry_preview", move |ctx| {
        auth::require(ctx)?;
        gardens::entry_preview(ctx, &input)
    })
    .await
}

/// Saves an installment, a running cost or a sale in another shop (PRD FR-12.35, FR-12.36). Errors:
/// every error of the preview, each at its field and all together; `ERR_GARDEN_CLOSED` on
/// `garden_id`; `ERR_DATE_BEFORE_GARDEN`, `ERR_DATE_CLOSED` and `ERR_DATE_IN_FUTURE` on `date`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value
/// `cash_after_paisa`) when the shop parts take cash below 0 and `confirm_cash_below_zero` is false.
#[tauri::command]
pub async fn gardens_entry_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenEntryInput,
) -> Result<GardenEntrySaved, UiError> {
    run(app, "gardens_entry_save", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::entry_save(ctx, &actor, &input)
    })
    .await
}

/// Cancels an installment, a running cost or a sale in another shop by reversal while its day is
/// open. Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_GARDEN_CLOSED` on `id` while the garden is closed;
/// `ERR_DATE_CLOSED` on `id`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
#[tauri::command]
pub async fn gardens_entry_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenEntryCancelInput,
) -> Result<GardenEntryCancelled, UiError> {
    run(app, "gardens_entry_cancel", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::entry_cancel(ctx, &actor, &input)
    })
    .await
}

/// Works out the closing of a garden (C26), nothing saved. Read only; the errors come back inside
/// the preview.
#[tauri::command]
pub async fn gardens_close_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenCloseInput,
) -> Result<GardenClosePreview, UiError> {
    run(app, "gardens_close_preview", move |ctx| {
        auth::require(ctx)?;
        gardens::close_preview(ctx, &input)
    })
    .await
}

/// Closes a garden: its account goes to 0 and each partner is credited his part (PRD FR-12.38).
/// Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`; `ERR_GARDEN_CLOSED` on `id`;
/// the date codes on `date`; `ERR_CLOSE_BEFORE_LAST_ENTRY` on `date` (value `last_date`).
#[tauri::command]
pub async fn gardens_close<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenCloseInput,
) -> Result<GardenClosed, UiError> {
    run(app, "gardens_close", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::close(ctx, &actor, &input)
    })
    .await
}

/// Undoes the closing of a garden by reversal while the closing day is open (PRD FR-12.38). Errors:
/// `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`; `ERR_GARDEN_NOT_CLOSED` on `id`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `id`.
#[tauri::command]
pub async fn gardens_reopen<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenReopenInput,
) -> Result<GardenReopened, UiError> {
    run(app, "gardens_reopen", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::reopen(ctx, &actor, &input)
    })
    .await
}

// ---- Goods arrived from a garden (مال آیا, PRD FR-12.40, plan 33 v1.1) ----
// The types and the codes are the contract (`mandi_core::arrivals`); each command is one service call.

/// Saves a garden arrival: a stock record of the garden with its date to sell. Posts no money.
/// Errors: `ERR_NOT_FOUND`, `ERR_GARDEN_CLOSED` on `garden_id`; `ERR_DATE_BEFORE_GARDEN` and
/// `ERR_DATE_CLOSED` on `date`; `ERR_SELL_DATE_BEFORE_ENTRY` on `sell_date`; the item, packing, qty
/// and weight codes at their fields.
#[tauri::command]
pub async fn gardens_arrival_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ArrivalInput,
) -> Result<ArrivalSaved, UiError> {
    run(app, "gardens_arrival_save", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::arrivals::save(ctx, &actor, &input)
    })
    .await
}

/// Cancels an arrival with a reason, only while nothing of it is sold or written off. Errors:
/// `ERR_NOT_FOUND`, `ERR_ARRIVAL_CANCELLED`, `ERR_ARRIVAL_HAS_SALES` on `id`; `ERR_VERSION_CHANGED`
/// on `version`; `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `id`.
#[tauri::command]
pub async fn gardens_arrival_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ArrivalCancelInput,
) -> Result<ArrivalCancelled, UiError> {
    run(app, "gardens_arrival_cancel", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::arrivals::cancel(ctx, &actor, &input)
    })
    .await
}

/// Changes the date to sell of an arrival with goods left. No money effect.
#[tauri::command]
pub async fn gardens_arrival_set_sell_date<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SellDateInput,
) -> Result<SellDateSet, UiError> {
    run(app, "gardens_arrival_set_sell_date", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::arrivals::set_sell_date(ctx, &actor, &input)
    })
    .await
}

/// One page of arrivals with the totals over the whole filter. Read only.
#[tauri::command]
pub async fn gardens_arrivals_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ArrivalsListInput,
) -> Result<ArrivalsList, UiError> {
    run(app, "gardens_arrivals_list", move |ctx| {
        auth::require(ctx)?;
        gardens::arrivals::list(ctx, &input)
    })
    .await
}

/// One arrival with its lots and its write-offs. Read only.
#[tauri::command]
pub async fn gardens_arrival_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ArrivalGetInput,
) -> Result<ArrivalRow, UiError> {
    run(app, "gardens_arrival_get", move |ctx| {
        auth::require(ctx)?;
        gardens::arrivals::get(ctx, &input)
    })
    .await
}

/// Writes off spoiled goods of an arrival. Nothing is posted. Errors: `ERR_NOT_FOUND`,
/// `ERR_ARRIVAL_CANCELLED` on `arrival_id`; the quantity codes on `qty`;
/// `ERR_WRITE_OFF_EXCEEDS_BALANCE` (value `available`) on `qty`; `ERR_REASON_REQUIRED` on `reason`;
/// `ERR_DATE_CLOSED` on `date`.
#[tauri::command]
pub async fn gardens_arrival_write_off<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ArrivalWriteOffInput,
) -> Result<ArrivalWriteOffDone, UiError> {
    run(app, "gardens_arrival_write_off", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::arrivals::write_off(ctx, &actor, &input)
    })
    .await
}

/// Undoes a write-off of an arrival, while its day is open. Errors: `ERR_NOT_FOUND` on `move_id`;
/// `ERR_ARRIVAL_CANCELLED` on `move_id`; `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on
/// `move_id`.
#[tauri::command]
pub async fn gardens_arrival_write_off_undo<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ArrivalWriteOffUndoInput,
) -> Result<ArrivalWriteOffUndone, UiError> {
    run(app, "gardens_arrival_write_off_undo", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::arrivals::write_off_undo(ctx, &actor, &input)
    })
    .await
}

/// Works out a goods-arrived entry with many item lines (PRD FR-12.41, R2): only the mistakes, each
/// at its field (`lines[i].qty` ...). Nothing is saved.
#[tauri::command]
pub async fn gardens_arrival_entry_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ArrivalEntryInput,
) -> Result<ArrivalEntryPreview, UiError> {
    run(app, "gardens_arrival_entry_preview", move |ctx| {
        auth::require(ctx)?;
        gardens::arrivals::entry_preview(ctx, &input)
    })
    .await
}

/// Saves a goods-arrived entry with many item lines in ONE transaction (PRD FR-12.41, R2). Each line
/// is its own arrival with its own number. A mistake in any line saves nothing.
#[tauri::command]
pub async fn gardens_arrival_entry_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ArrivalEntryInput,
) -> Result<ArrivalEntrySaved, UiError> {
    run(app, "gardens_arrival_entry_save", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::arrivals::entry_save(ctx, &actor, &input)
    })
    .await
}

/// Changes the split of an arrival between our shop and another shop, while goods are left (PRD
/// FR-12.41, R3). No money effect.
#[tauri::command]
pub async fn gardens_arrival_change_split<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SplitInput,
) -> Result<SplitSet, UiError> {
    run(app, "gardens_arrival_change_split", move |ctx| {
        let actor = auth::require(ctx)?;
        gardens::arrivals::change_split(ctx, &actor, &input)
    })
    .await
}
