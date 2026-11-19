//! Market trips (PRD 12.10, FR-12.29 to FR-12.33, plan 16b): create a trip with its partners and
//! shares, list and read trips, add trip costs and sales in other shops, and the figures of the
//! dashboard tile.
//!
//! Every user action is one function and one transaction (Code Standards B3). The preview and the
//! save of a cost or a sale share one checking and working out (B5). A payment is split by the
//! trip's shares with `split_by_bps` (C23); the parts paid from the shop's cash post on the
//! partners' 0604 accounts, the parts paid from pockets post nothing (FR-12.30).
//!
//! - `make`: create a trip and change its partners before the first entry.
//! - `read`: the form defaults, the list, one trip with its history, the tile.
//! - `cost`: a trip cost (C23, C24): preview, save, cancel.
//! - `sale`: a sale in another shop (C21, C25): preview, save, cancel.

mod cost;
pub(crate) mod cost_edit;
pub(crate) mod header_edit;
mod make;
mod read;
mod sale;
pub(crate) mod sale_edit;

pub(crate) use read::cards_of_partner;

use mandi_domain::core::trips::{
    OtherSaleInput, OtherSalePreview, OtherSaleSaved, TripCostInput, TripCostPreview,
    TripCostSaved, TripDefaults, TripDetail, TripEntryCancelInput, TripEntryCancelled,
    TripGetInput, TripInput, TripRow, TripSetPartnersInput, TripsList, TripsListInput,
    TripsSummary,
};

use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as venture_repo, EntryRec};

use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;

/// The entry of a TRIP with this id and kind (`cost` or `other_sale`), or `None`.
///
/// The kinds `cost` and `other_sale` are shared by trips and gardens, so a kind check alone lets a
/// trip command reach the entry of a garden, even one of a closed garden (PRD FR-12.38, Code
/// Standards V0). The entry's venture must be a trip too. A command maps `None` to `ERR_NOT_FOUND`
/// on `id`.
///
/// # Errors
/// Any SQLite error.
pub(crate) fn trip_entry(
    conn: &Connection,
    id: i64,
    kind: &str,
) -> mandi_db::rusqlite::Result<Option<EntryRec>> {
    let Some(entry) = venture_repo::get_entry(conn, id)?.filter(|e| e.kind == kind) else {
        return Ok(None);
    };
    Ok(venture_repo::get(conn, entry.venture_id)?
        .is_some_and(|v| v.kind == "trip")
        .then_some(entry))
}

/// The form of a new trip, filled from the last trip (PRD A56). Read only.
///
/// # Errors
/// `ServiceError::Db`.
pub fn defaults(ctx: &Context) -> Result<TripDefaults, ServiceError> {
    read::defaults(ctx)
}

/// Makes a trip with its partners, shares and default sources (PRD FR-12.29).
///
/// # Errors
/// The date, text and percent codes at their fields; `ERR_PARTNER_NOT_LISTED`; `ERR_SHARES_NOT_100`
/// on `partners` (value `total_bps`).
pub fn create(ctx: &Context, actor: &Actor, input: &TripInput) -> Result<TripRow, ServiceError> {
    make::create(ctx, actor, input)
}

/// Changes the partners and shares of a trip that has no entry yet (PRD FR-12.29).
///
/// # Errors
/// As [`create`] for the partners; `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED`;
/// `ERR_VENTURE_SHARES_LOCKED` on `partners`.
pub fn set_partners(
    ctx: &Context,
    actor: &Actor,
    input: &TripSetPartnersInput,
) -> Result<TripRow, ServiceError> {
    make::set_partners(ctx, actor, input)
}

/// One page of trips with the totals over the whole filter. Read only.
///
/// # Errors
/// The date codes on `from` and `to`.
pub fn list(ctx: &Context, input: &TripsListInput) -> Result<TripsList, ServiceError> {
    read::list(ctx, input)
}

/// One trip with its purchases, history and each partner's money. Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, input: &TripGetInput) -> Result<TripDetail, ServiceError> {
    read::get(ctx, input)
}

/// The data of the dashboard tile مارکیٹ سفر (PRD 9.1). Read only.
///
/// # Errors
/// `ServiceError::Db`.
pub fn summary(ctx: &Context) -> Result<TripsSummary, ServiceError> {
    read::summary(ctx)
}

/// Works out a typed trip cost for the live screen (PRD FR-12.31). Nothing is saved.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix; the user's mistakes
/// come back inside the preview.
pub fn cost_preview(ctx: &Context, input: &TripCostInput) -> Result<TripCostPreview, ServiceError> {
    cost::preview(ctx, input)
}

/// Saves a trip cost (PRD FR-12.31) in one transaction.
///
/// # Errors
/// See `cost::save`.
pub fn cost_save(
    ctx: &Context,
    actor: &Actor,
    input: &TripCostInput,
) -> Result<TripCostSaved, ServiceError> {
    cost::save(ctx, actor, input)
}

/// Cancels a trip cost by reversal (PRD FR-12.31).
///
/// # Errors
/// See `cost::cancel`.
pub fn cost_cancel(
    ctx: &Context,
    actor: &Actor,
    input: &TripEntryCancelInput,
) -> Result<TripEntryCancelled, ServiceError> {
    cost::cancel(ctx, actor, input)
}

/// Works out a typed sale in another shop for the live screen (PRD FR-12.32). Nothing is saved.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix; the user's mistakes
/// come back inside the preview.
pub fn other_sale_preview(
    ctx: &Context,
    input: &OtherSaleInput,
) -> Result<OtherSalePreview, ServiceError> {
    sale::preview(ctx, input)
}

/// Saves a sale in another shop (PRD FR-12.32) in one transaction.
///
/// # Errors
/// See `sale::save`.
pub fn other_sale_save(
    ctx: &Context,
    actor: &Actor,
    input: &OtherSaleInput,
) -> Result<OtherSaleSaved, ServiceError> {
    sale::save(ctx, actor, input)
}

/// Cancels a sale in another shop by reversal (PRD FR-12.32).
///
/// # Errors
/// See `sale::cancel`.
pub fn other_sale_cancel(
    ctx: &Context,
    actor: &Actor,
    input: &TripEntryCancelInput,
) -> Result<TripEntryCancelled, ServiceError> {
    sale::cancel(ctx, actor, input)
}
