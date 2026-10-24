//! Gardens (باغات, PRD 12.11, FR-12.29 to FR-12.38, plan 17c): make a garden with its partners and
//! shares, pay its installments and running costs, sell its goods in another shop, read its running
//! balance and close it so its money goes to its partners (C26).
//!
//! Every user action is one function and one transaction (Code Standards B3). The preview and the
//! save of an entry share one checking and working out (B5). A payment is split by the garden's
//! shares with `split_by_bps` (C23); the parts paid from the shop's cash post on the partners' 0604
//! accounts, the parts paid from pockets post nothing (FR-12.30). The money of the garden's goods
//! sold in our shop (the seller credit of a lot) and in other shops stays on the garden's own account
//! until the Owner closes the garden, when `split_by_bps` shares it out (C26).
//!
//! - `make`: the form defaults, make a garden (its account, partners, first installment), change its
//!   partners before the first entry.
//! - `read`: the list, one garden with its history and partners, the tile; the figures of a garden.
//! - `entry`: an installment, a running cost or a sale in another shop: preview, save, cancel.
//! - `close`: the closing and its undo.

mod close;
mod entry;
mod make;
mod read;

pub(crate) use read::cards_of_partner;

use mandi_domain::core::gardens::{
    GardenCloseInput, GardenClosePreview, GardenClosed, GardenCreatePreview, GardenCreated,
    GardenDefaults, GardenDetail, GardenEntryCancelInput, GardenEntryCancelled, GardenEntryInput,
    GardenEntryPreview, GardenEntrySaved, GardenGetInput, GardenInput, GardenReopenInput,
    GardenReopened, GardenRow, GardenSetPartnersInput, GardensList, GardensListInput,
    GardensSummary,
};

use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;

/// The form of a new garden: every active partner with the firm's share of today and the source
/// `shop` (PRD A56). Read only.
///
/// # Errors
/// `ServiceError::Db`.
pub fn defaults(ctx: &Context) -> Result<GardenDefaults, ServiceError> {
    make::defaults(ctx)
}

/// Works out a new garden's first installment, nothing saved. Read only.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix; the user's
/// mistakes come back inside the preview.
pub fn create_preview(
    ctx: &Context,
    input: &GardenInput,
) -> Result<GardenCreatePreview, ServiceError> {
    make::create_preview(ctx, input)
}

/// Makes a garden with its own account, its partners with their shares and sources, and its first
/// installment if one is typed (PRD FR-12.29, FR-12.34).
///
/// # Errors
/// Every error of the preview, each at its field and all together, with nothing written;
/// `ERR_SPELLING_UNCONFIRMED`, `ERR_SERIES_FULL`, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
pub fn create(
    ctx: &Context,
    actor: &Actor,
    input: &GardenInput,
) -> Result<GardenCreated, ServiceError> {
    make::create(ctx, actor, input)
}

/// Changes the partners and shares of a garden that has no entry yet (PRD FR-12.29). After its first
/// entry they are fixed.
///
/// # Errors
/// As [`create`] for the partners; `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_VENTURE_SHARES_LOCKED` on `partners` once the garden has an entry.
pub fn set_partners(
    ctx: &Context,
    actor: &Actor,
    input: &GardenSetPartnersInput,
) -> Result<GardenRow, ServiceError> {
    make::set_partners(ctx, actor, input)
}

/// One page of gardens with the totals over the whole filter (PRD FR-19.11). Read only.
///
/// # Errors
/// The date codes on `from` and `to`; `ServiceError::Db`.
pub fn list(ctx: &Context, input: &GardensListInput) -> Result<GardensList, ServiceError> {
    read::list(ctx, input)
}

/// One garden with its history, running balance and each partner's money (PRD FR-12.37). Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, input: &GardenGetInput) -> Result<GardenDetail, ServiceError> {
    read::get(ctx, input)
}

/// The open gardens with their running balances, for the dashboard tile (PRD 9.1). Read only.
///
/// # Errors
/// `ServiceError::Db`.
pub fn summary(ctx: &Context) -> Result<GardensSummary, ServiceError> {
    read::summary(ctx)
}

/// Works out an installment, a running cost or a sale in another shop, nothing saved. Read only.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix; the user's
/// mistakes come back inside the preview.
pub fn entry_preview(
    ctx: &Context,
    input: &GardenEntryInput,
) -> Result<GardenEntryPreview, ServiceError> {
    entry::preview(ctx, input)
}

/// Saves an installment, a running cost or a sale in another shop (PRD FR-12.35, FR-12.36).
///
/// # Errors
/// Every error of the preview, each at its field and all together, with nothing written;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`) when the shop parts take cash
/// below 0 and `confirm_cash_below_zero` is false.
pub fn entry_save(
    ctx: &Context,
    actor: &Actor,
    input: &GardenEntryInput,
) -> Result<GardenEntrySaved, ServiceError> {
    entry::save(ctx, actor, input)
}

/// Cancels an entry of a garden by reversal while its day is open.
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_GARDEN_CLOSED` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `id`;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
pub fn entry_cancel(
    ctx: &Context,
    actor: &Actor,
    input: &GardenEntryCancelInput,
) -> Result<GardenEntryCancelled, ServiceError> {
    entry::cancel(ctx, actor, input)
}

/// Works out the closing of a garden (C26), nothing saved. Read only.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix; the user's
/// mistakes come back inside the preview.
pub fn close_preview(
    ctx: &Context,
    input: &GardenCloseInput,
) -> Result<GardenClosePreview, ServiceError> {
    close::preview(ctx, input)
}

/// Closes a garden: its money goes to its partners by their % (PRD FR-12.38).
///
/// # Errors
/// Every error of the preview with nothing written; `ERR_GARDEN_CLOSED` on `id`.
pub fn close(
    ctx: &Context,
    actor: &Actor,
    input: &GardenCloseInput,
) -> Result<GardenClosed, ServiceError> {
    close::close(ctx, actor, input)
}

/// Undoes the closing of a garden by reversal while its day is open (PRD FR-12.38).
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_GARDEN_NOT_CLOSED` and `ERR_DATE_CLOSED` on `id`; `ERR_VERSION_CHANGED` on
/// `version`; `ERR_REASON_REQUIRED` on `reason`.
pub fn reopen(
    ctx: &Context,
    actor: &Actor,
    input: &GardenReopenInput,
) -> Result<GardenReopened, ServiceError> {
    close::reopen(ctx, actor, input)
}

/// A page size of 100 when the screen sends 0 (PRD FR-19.11).
const DEFAULT_PAGE: u32 = 100;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}
