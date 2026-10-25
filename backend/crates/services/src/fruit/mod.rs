//! Fruit taken (پھل, PRD 16.7, FR-16.10 to FR-16.12, plan 17d): a partner takes fruit home, or fruit
//! is served to the shop's guests. The value is the partner's alone, or split equally between the
//! partners for the guests (C27), never by the firm's shares. The lot's seller or the other agent is
//! credited the value with no commission, and that credit is a payable posting that never adjusts an
//! advance (FR-7.59). No cash moves.
//!
//! Every user action is one function and one transaction (Code Standards B3). The preview, the save
//! and the edit work an entry out with the same function, `plan` (B5). A saved entry is edited and
//! cancelled by reversal while its day is open (FR-16.12).

mod act;
mod plan;
mod read;

use mandi_domain::core::fruit::{
    FruitCancelInput, FruitCancelled, FruitEditInput, FruitEdited, FruitGetInput, FruitInput,
    FruitList, FruitListInput, FruitLots, FruitLotsInput, FruitPreview, FruitRow, FruitSaved,
};

use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::ServiceError;

/// Works out a fruit entry, nothing saved. Read only. The mistakes come back inside the preview.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix.
pub fn preview(ctx: &Context, input: &FruitInput) -> Result<FruitPreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| act::preview_of(conn, window, input))
}

/// Saves a fruit entry (PRD FR-16.10, FR-16.11).
///
/// # Errors
/// Every error of the preview, each at its field and all together, with nothing written.
pub fn save(ctx: &Context, actor: &Actor, input: &FruitInput) -> Result<FruitSaved, ServiceError> {
    act::save(ctx, actor, input)
}

/// Edits a fruit entry by reversal while its day is open (PRD FR-16.12).
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_FRUIT_CANCELLED`, `ERR_DATE_CLOSED` and `ERR_GARDEN_CLOSED` on `id`;
/// `ERR_VERSION_CHANGED` on `version`; `ERR_REASON_REQUIRED` on `reason`; every error of the preview.
pub fn edit(
    ctx: &Context,
    actor: &Actor,
    input: &FruitEditInput,
) -> Result<FruitEdited, ServiceError> {
    act::edit(ctx, actor, input)
}

/// Cancels a fruit entry by reversal while its day is open (PRD FR-16.12).
///
/// # Errors
/// As [`edit`], without the preview errors.
pub fn cancel(
    ctx: &Context,
    actor: &Actor,
    input: &FruitCancelInput,
) -> Result<FruitCancelled, ServiceError> {
    act::cancel(ctx, actor, input)
}

/// One entry. Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, input: &FruitGetInput) -> Result<FruitRow, ServiceError> {
    read::get(ctx, input)
}

/// One page of entries with the totals over the whole filter (PRD FR-19.11). Read only.
///
/// # Errors
/// The date codes on `from` and `to`.
pub fn list(ctx: &Context, input: &FruitListInput) -> Result<FruitList, ServiceError> {
    read::list(ctx, input)
}

/// The lots fruit can be taken from (PRD FR-16.11). Read only.
///
/// # Errors
/// `ServiceError::Db`.
pub fn lots(ctx: &Context, input: &FruitLotsInput) -> Result<FruitLots, ServiceError> {
    read::lots(ctx, input)
}
