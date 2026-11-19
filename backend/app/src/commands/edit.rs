//! Edit everywhere (PRD FR-7.70, FR-7.71, plan 44): the review and the save of the edits that are
//! new, and the review of the edits that exist already. Every command needs the signed-in Owner.
//! A review only reads (Code Standards B29); a save is one transaction (Code Standards B3).
//!
//! Every edit keeps the entry's date (`ERR_EDIT_DATE_FIXED` on the date field), asks for a reason
//! when the entry is not of today (`ERR_REASON_REQUIRED` on `reason`), and refuses a stale
//! `version` (`ERR_VERSION_CHANGED` on `version`). A trip cost and a garden payment have a reason of
//! their own (why the money was paid), so for those two the missing reason of the EDIT is named at
//! `edit_reason` (the input field is still `reason`). The edit of a trip or garden entry is a
//! reversal and a new posting on the same day; its rows are changed inside an edit window of that one
//! entry (migration 12).

use mandi_services::core::bikri::EditPreviewInput;
use mandi_services::core::edit::{
    EditPreview, EntryEdited, GardenArrivalEditInput, GardenEntryEditInput, GardenHeaderEditInput,
    PurchaseEditInput, TripCostEditInput, TripHeaderEditInput, TripOtherSaleEditInput,
};
use mandi_services::core::fruit::FruitEditInput;
use mandi_services::core::vouchers::VoucherEditInput;
use mandi_services::{auth, edit};

use super::run;
use crate::error::UiError;

/// Review of an edit of a بکری posting: what changes and what it does (PRD FR-7.70). Read only. The numbers of the live summary come from `bikri_edit_preview`; this answers the change list and the effect. Same input as `bikri_edit_preview`.
#[tauri::command]
pub async fn bikri_edit_review<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: EditPreviewInput,
) -> Result<EditPreview, UiError> {
    run(app, "bikri_edit_review", move |ctx| {
        auth::require(ctx)?;
        edit::bikri_review(ctx, &input)
    })
    .await
}

/// Review of an edit of a voucher of any type (وصولی, ادائیگی, پیشگی, خرچہ, ادھار رقم, الگ کھاتہ, سرمایہ, جنرل): what changes and what it does (PRD FR-7.70). Read only. Same input as `vouchers_edit`.
#[tauri::command]
pub async fn vouchers_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: VoucherEditInput,
) -> Result<EditPreview, UiError> {
    run(app, "vouchers_edit_preview", move |ctx| {
        auth::require(ctx)?;
        edit::voucher_review(ctx, &input)
    })
    .await
}

/// Review of an edit of fruit taken: what changes and what it does (PRD FR-7.70). Read only. Same input as `fruit_edit`.
#[tauri::command]
pub async fn fruit_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FruitEditInput,
) -> Result<EditPreview, UiError> {
    run(app, "fruit_edit_preview", move |ctx| {
        auth::require(ctx)?;
        edit::fruit_review(ctx, &input)
    })
    .await
}

/// The review of an edit of a trip purchase: every mistake of the edit, what changes and what it does
/// (PRD FR-7.70). Read only. Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on the date field; `ERR_REASON_REQUIRED` on `reason` for an entry not of
/// today; `ERR_EDIT_NOT_ALLOWED` with rule `goods_moved` once any of it was sold or written off, or with
/// rule `carries_cost` (field `qty`) for a new quantity when it carries a part of a trip cost.
#[tauri::command]
pub async fn purchases_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PurchaseEditInput,
) -> Result<EditPreview, UiError> {
    run(app, "purchases_edit_preview", move |ctx| {
        auth::require(ctx)?;
        edit::purchase_preview(ctx, &input)
    })
    .await
}

/// Saves the edit of a trip purchase: the old posting is reversed and the new one is posted on the same
/// day, in one transaction (PRD FR-12.19, FR-6.3). The errors of `purchases_edit_preview`; where the entry
/// moves cash, cash below 0 needs the `confirm_cash_below_zero` inside the entry (PRD FR-6.7).
#[tauri::command]
pub async fn purchases_edit_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PurchaseEditInput,
) -> Result<EntryEdited, UiError> {
    run(app, "purchases_edit_save", move |ctx| {
        let actor = auth::require(ctx)?;
        edit::purchase_save(ctx, &actor, &input)
    })
    .await
}

/// The review of an edit of a trip cost: every mistake of the edit, what changes and what it does
/// (PRD FR-7.70). Read only. Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on the date field; `ERR_REASON_REQUIRED` on `reason` for an entry not of
/// today (at `edit_reason`, the cost has a reason of its own); `ERR_EDIT_NOT_ALLOWED` with rule
/// `goods_moved` once any purchase it touched was sold or written off.
#[tauri::command]
pub async fn trips_cost_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripCostEditInput,
) -> Result<EditPreview, UiError> {
    run(app, "trips_cost_edit_preview", move |ctx| {
        auth::require(ctx)?;
        edit::trip_cost_preview(ctx, &input)
    })
    .await
}

/// Saves the edit of a trip cost: the old posting is reversed and the new one is posted on the same
/// day, in one transaction (PRD FR-12.19, FR-6.3). The errors of `trips_cost_edit_preview`; where the entry
/// moves cash, cash below 0 needs the `confirm_cash_below_zero` inside the entry (PRD FR-6.7).
#[tauri::command]
pub async fn trips_cost_edit_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripCostEditInput,
) -> Result<EntryEdited, UiError> {
    run(app, "trips_cost_edit_save", move |ctx| {
        let actor = auth::require(ctx)?;
        edit::trip_cost_save(ctx, &actor, &input)
    })
    .await
}

/// The review of an edit of a sale in another shop of a trip: every mistake of the edit, what changes and what it does
/// (PRD FR-7.70). Read only. Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on the date field; `ERR_REASON_REQUIRED` on `reason` for an entry not of
/// today; `ERR_EDIT_NOT_ALLOWED` with rule `goods_moved` unless this sale is the only thing that moved
/// goods of its purchase. The input carries `confirm_cash_below_zero` for an edit that lowers the money.
#[tauri::command]
pub async fn trips_other_sale_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripOtherSaleEditInput,
) -> Result<EditPreview, UiError> {
    run(app, "trips_other_sale_edit_preview", move |ctx| {
        auth::require(ctx)?;
        edit::trip_other_sale_preview(ctx, &input)
    })
    .await
}

/// Saves the edit of a sale in another shop of a trip: the old posting is reversed and the new one is posted on the same
/// day, in one transaction (PRD FR-12.19, FR-6.3). The errors of `trips_other_sale_edit_preview`; where the entry
/// moves cash, cash below 0 needs the `confirm_cash_below_zero` inside the entry (PRD FR-6.7).
#[tauri::command]
pub async fn trips_other_sale_edit_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripOtherSaleEditInput,
) -> Result<EntryEdited, UiError> {
    run(app, "trips_other_sale_edit_save", move |ctx| {
        let actor = auth::require(ctx)?;
        edit::trip_other_sale_save(ctx, &actor, &input)
    })
    .await
}

/// The review of an edit of a garden installment, running cost or sale in another shop: every mistake of the edit, what changes and what it does
/// (PRD FR-7.70). Read only. Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on the date field; `ERR_REASON_REQUIRED` on `reason` for an entry not of
/// today (at `edit_reason` for a payment); `ERR_GARDEN_CLOSED` while the garden is closed;
/// `ERR_EDIT_NOT_ALLOWED` with rule `goods_moved` for a sale that is not the only thing that moved goods
/// of its arrival. The input carries `confirm_cash_below_zero` for a sale that lowers the money.
#[tauri::command]
pub async fn gardens_entry_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenEntryEditInput,
) -> Result<EditPreview, UiError> {
    run(app, "gardens_entry_edit_preview", move |ctx| {
        auth::require(ctx)?;
        edit::garden_entry_preview(ctx, &input)
    })
    .await
}

/// Saves the edit of a garden installment, running cost or sale in another shop: the old posting is reversed and the new one is posted on the same
/// day, in one transaction (PRD FR-12.19, FR-6.3). The errors of `gardens_entry_edit_preview`; where the entry
/// moves cash, cash below 0 needs the `confirm_cash_below_zero` inside the entry (PRD FR-6.7).
#[tauri::command]
pub async fn gardens_entry_edit_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenEntryEditInput,
) -> Result<EntryEdited, UiError> {
    run(app, "gardens_entry_edit_save", move |ctx| {
        let actor = auth::require(ctx)?;
        edit::garden_entry_save(ctx, &actor, &input)
    })
    .await
}

/// The review of an edit of a garden arrival: every mistake of the edit, what changes and what it does
/// (PRD FR-7.70). Read only. Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on the date field; `ERR_REASON_REQUIRED` on `reason` for an entry not of
/// today; `ERR_EDIT_NOT_ALLOWED` with rule `goods_moved` once any of it was sold or written off.
#[tauri::command]
pub async fn gardens_arrival_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenArrivalEditInput,
) -> Result<EditPreview, UiError> {
    run(app, "gardens_arrival_edit_preview", move |ctx| {
        auth::require(ctx)?;
        edit::garden_arrival_preview(ctx, &input)
    })
    .await
}

/// Saves the edit of a garden arrival: the old posting is reversed and the new one is posted on the same
/// day, in one transaction (PRD FR-12.19, FR-6.3). The errors of `gardens_arrival_edit_preview`; where the entry
/// moves cash, cash below 0 needs the `confirm_cash_below_zero` inside the entry (PRD FR-6.7).
#[tauri::command]
pub async fn gardens_arrival_edit_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenArrivalEditInput,
) -> Result<EntryEdited, UiError> {
    run(app, "gardens_arrival_edit_save", move |ctx| {
        let actor = auth::require(ctx)?;
        edit::garden_arrival_save(ctx, &actor, &input)
    })
    .await
}

/// The review of an edit of the header of a trip (market, person sent, note): every mistake of the edit, what changes and what it does
/// (PRD FR-7.70). Read only. Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on the date field; `ERR_REASON_REQUIRED` on `reason` for an entry not of
/// today; `ERR_VENTURE_SHARES_LOCKED` on `trip.partners` if the shares differ from the trip's own.
#[tauri::command]
pub async fn trips_header_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripHeaderEditInput,
) -> Result<EditPreview, UiError> {
    run(app, "trips_header_edit_preview", move |ctx| {
        auth::require(ctx)?;
        edit::trip_header_preview(ctx, &input)
    })
    .await
}

/// Saves the edit of the header of a trip (market, person sent, note): the old posting is reversed and the new one is posted on the same
/// day, in one transaction (PRD FR-12.19, FR-6.3). The errors of `trips_header_edit_preview`; where the entry
/// moves cash, cash below 0 needs the `confirm_cash_below_zero` inside the entry (PRD FR-6.7).
#[tauri::command]
pub async fn trips_header_edit_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripHeaderEditInput,
) -> Result<EntryEdited, UiError> {
    run(app, "trips_header_edit_save", move |ctx| {
        let actor = auth::require(ctx)?;
        edit::trip_header_save(ctx, &actor, &input)
    })
    .await
}

/// The review of an edit of the header of a garden (names, item, rent, season end, note): every mistake of the edit, what changes and what it does
/// (PRD FR-7.70). Read only. Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on the date field; `ERR_REASON_REQUIRED` on `reason` for an entry not of
/// today; `ERR_VENTURE_SHARES_LOCKED` if the shares are touched.
#[tauri::command]
pub async fn gardens_header_edit_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenHeaderEditInput,
) -> Result<EditPreview, UiError> {
    run(app, "gardens_header_edit_preview", move |ctx| {
        auth::require(ctx)?;
        edit::garden_header_preview(ctx, &input)
    })
    .await
}

/// Saves the edit of the header of a garden (names, item, rent, season end, note): the old posting is reversed and the new one is posted on the same
/// day, in one transaction (PRD FR-12.19, FR-6.3). The errors of `gardens_header_edit_preview`; where the entry
/// moves cash, cash below 0 needs the `confirm_cash_below_zero` inside the entry (PRD FR-6.7).
#[tauri::command]
pub async fn gardens_header_edit_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GardenHeaderEditInput,
) -> Result<EntryEdited, UiError> {
    run(app, "gardens_header_edit_save", move |ctx| {
        let actor = auth::require(ctx)?;
        edit::garden_header_save(ctx, &actor, &input)
    })
    .await
}
