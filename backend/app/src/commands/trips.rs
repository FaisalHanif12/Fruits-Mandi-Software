//! Market trips (PRD 12.10, plan 16b): make a trip with its partners and shares, read trips, add
//! trip costs and sales in other shops. Every command needs the signed-in Owner. The reads and the
//! previews only read (Code Standards B29); the others are one transaction each (Code Standards B3).

use mandi_services::core::trips::{
    OtherSaleInput, OtherSalePreview, OtherSaleSaved, TripCostInput, TripCostPreview,
    TripCostSaved, TripDefaults, TripDetail, TripEntryCancelInput, TripEntryCancelled,
    TripGetInput, TripInput, TripRow, TripSetPartnersInput, TripsList, TripsListInput,
    TripsSummary,
};
use mandi_services::{auth, trips};

use super::run;
use crate::error::UiError;

/// The form of a new trip: the partners with the % and the sources of the last trip, else the
/// firm's shares and the source shop (PRD A56). Read only.
#[tauri::command]
pub async fn trips_defaults<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<TripDefaults, UiError> {
    run(app, "trips_defaults", move |ctx| {
        auth::require(ctx)?;
        trips::defaults(ctx)
    })
    .await
}

/// Makes a trip (PRD FR-12.29): shares first, then sources, then the details. Errors: the date
/// codes on `date`; the text codes on `market`, `sent_person` and `note`; the percent codes on
/// `partners[i].share`; `ERR_NOT_FOUND` and `ERR_PARTNER_NOT_LISTED` on `partners[i].partner_id`;
/// `ERR_SHARES_NOT_100` on `partners` (value `total_bps`).
#[tauri::command]
pub async fn trips_create<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripInput,
) -> Result<TripRow, UiError> {
    run(app, "trips_create", move |ctx| {
        let actor = auth::require(ctx)?;
        trips::create(ctx, &actor, &input)
    })
    .await
}

/// Changes the partners and shares of a trip that has no entry yet. Errors: as `trips_create`;
/// `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`; `ERR_VENTURE_SHARES_LOCKED` on
/// `partners` after the first entry (PRD FR-12.29).
#[tauri::command]
pub async fn trips_set_partners<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripSetPartnersInput,
) -> Result<TripRow, UiError> {
    run(app, "trips_set_partners", move |ctx| {
        let actor = auth::require(ctx)?;
        trips::set_partners(ctx, &actor, &input)
    })
    .await
}

/// One page of trips with the totals over the whole filter (PRD FR-19.11). Read only.
#[tauri::command]
pub async fn trips_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripsListInput,
) -> Result<TripsList, UiError> {
    run(app, "trips_list", move |ctx| {
        auth::require(ctx)?;
        trips::list(ctx, &input)
    })
    .await
}

/// One trip with its purchases, history and each partner's money in and money back. Read only.
#[tauri::command]
pub async fn trips_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripGetInput,
) -> Result<TripDetail, UiError> {
    run(app, "trips_get", move |ctx| {
        auth::require(ctx)?;
        trips::get(ctx, &input)
    })
    .await
}

/// The data of the dashboard tile مارکیٹ سفر: open trips, نگ left and cost left. Read only.
#[tauri::command]
pub async fn trips_summary<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<TripsSummary, UiError> {
    run(app, "trips_summary", move |ctx| {
        auth::require(ctx)?;
        trips::summary(ctx)
    })
    .await
}

/// Works out a typed trip cost for the live screen: each partner's part, where it comes from, and
/// how the cost is spread over the purchases with goods left (C24). Nothing is saved.
#[tauri::command]
pub async fn trips_cost_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripCostInput,
) -> Result<TripCostPreview, UiError> {
    run(app, "trips_cost_preview", move |ctx| {
        auth::require(ctx)?;
        trips::cost_preview(ctx, &input)
    })
    .await
}

/// Saves a trip cost (PRD FR-12.31) in one transaction. Errors: every error of the preview at its
/// field; `ERR_TRIP_CLOSED` and `ERR_TRIP_NO_STOCK` on `trip_id`; `ERR_DATE_CLOSED`;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
#[tauri::command]
pub async fn trips_cost_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripCostInput,
) -> Result<TripCostSaved, UiError> {
    run(app, "trips_cost_save", move |ctx| {
        let actor = auth::require(ctx)?;
        trips::cost_save(ctx, &actor, &input)
    })
    .await
}

/// Cancels a trip cost by reversal, only while nothing of the purchases it touched has been sold
/// or written off. Errors: `ERR_NOT_FOUND`, `ERR_TRIP_ENTRY_CANCELLED`, `ERR_TRIP_COST_HAS_SALES`
/// on `id`; `ERR_VERSION_CHANGED` on `version`; `ERR_REASON_REQUIRED` on `reason`;
/// `ERR_DATE_CLOSED` on `id`.
#[tauri::command]
pub async fn trips_cost_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripEntryCancelInput,
) -> Result<TripEntryCancelled, UiError> {
    run(app, "trips_cost_cancel", move |ctx| {
        let actor = auth::require(ctx)?;
        trips::cost_cancel(ctx, &actor, &input)
    })
    .await
}

/// Works out a typed sale in another shop for the live screen: the cost of the goods (C21), the
/// result, and each trip partner's part of the money (C25). Nothing is saved.
#[tauri::command]
pub async fn trips_other_sale_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: OtherSaleInput,
) -> Result<OtherSalePreview, UiError> {
    run(app, "trips_other_sale_preview", move |ctx| {
        auth::require(ctx)?;
        trips::other_sale_preview(ctx, &input)
    })
    .await
}

/// Saves a sale in another shop (PRD FR-12.32) in one transaction: روکڑ بنام, each trip partner's
/// 0604 account جمع his part. Errors: every error of the preview at its field; `ERR_QTY_EXCEEDS_LOT`
/// (value `available`) on `qty`; `ERR_DATE_CLOSED`.
#[tauri::command]
pub async fn trips_other_sale_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: OtherSaleInput,
) -> Result<OtherSaleSaved, UiError> {
    run(app, "trips_other_sale_save", move |ctx| {
        let actor = auth::require(ctx)?;
        trips::other_sale_save(ctx, &actor, &input)
    })
    .await
}

/// Cancels a sale in another shop by reversal; the qty and its cost go back to the purchase.
/// Errors: `ERR_NOT_FOUND`, `ERR_TRIP_ENTRY_CANCELLED` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `id`;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
#[tauri::command]
pub async fn trips_other_sale_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TripEntryCancelInput,
) -> Result<TripEntryCancelled, UiError> {
    run(app, "trips_other_sale_cancel", move |ctx| {
        let actor = auth::require(ctx)?;
        trips::other_sale_cancel(ctx, &actor, &input)
    })
    .await
}
