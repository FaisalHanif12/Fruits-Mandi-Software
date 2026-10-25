//! Fruit taken (پھل, PRD 16.7, FR-16.10 to FR-16.12, plan 17d). The contract is built (plan 17d T1);
//! each function answers `ERR_NOT_IMPLEMENTED` until T2 writes the logic.
//!
//! Every user action will be one function and one transaction (Code Standards B3); the preview and
//! the save will work the entry out with the same function (B5).

use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::fruit::{
    FruitCancelInput, FruitCancelled, FruitEditInput, FruitEdited, FruitGetInput, FruitInput,
    FruitList, FruitListInput, FruitLots, FruitLotsInput, FruitPreview, FruitRow, FruitSaved,
};

use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;

fn not_yet<T>() -> Result<T, ServiceError> {
    Err(ServiceError::reject(ErrorCode::ErrNotImplemented))
}

/// Works out a fruit entry, nothing saved. Read only.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn preview(_ctx: &Context, _input: &FruitInput) -> Result<FruitPreview, ServiceError> {
    not_yet()
}

/// Saves a fruit entry (PRD FR-16.10, FR-16.11).
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn save(
    _ctx: &Context,
    _actor: &Actor,
    _input: &FruitInput,
) -> Result<FruitSaved, ServiceError> {
    not_yet()
}

/// Edits a fruit entry by reversal while its day is open (PRD FR-16.12).
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn edit(
    _ctx: &Context,
    _actor: &Actor,
    _input: &FruitEditInput,
) -> Result<FruitEdited, ServiceError> {
    not_yet()
}

/// Cancels a fruit entry by reversal while its day is open (PRD FR-16.12).
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn cancel(
    _ctx: &Context,
    _actor: &Actor,
    _input: &FruitCancelInput,
) -> Result<FruitCancelled, ServiceError> {
    not_yet()
}

/// One entry. Read only.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn get(_ctx: &Context, _input: &FruitGetInput) -> Result<FruitRow, ServiceError> {
    not_yet()
}

/// One page of entries with the totals over the whole filter (PRD FR-19.11). Read only.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn list(_ctx: &Context, _input: &FruitListInput) -> Result<FruitList, ServiceError> {
    not_yet()
}

/// The lots fruit can be taken from (PRD FR-16.11). Read only.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn lots(_ctx: &Context, _input: &FruitLotsInput) -> Result<FruitLots, ServiceError> {
    not_yet()
}
