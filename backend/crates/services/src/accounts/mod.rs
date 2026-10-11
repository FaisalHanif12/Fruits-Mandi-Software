//! Accounts (PRD 10.2, plan 07). Contract stubs: T2 fills them in.

use mandi_domain::core::accounts::{
    AccountInput, AccountRow, AccountType, AccountsList, AccountsListInput, AccountsSimilarInput,
    OpeningsInput, OpeningsPreview, OpeningsSaved, SimilarAccount,
};
use mandi_domain::core::error::ErrorCode;

use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;

fn todo<T>() -> Result<T, ServiceError> {
    Err(ServiceError::reject(ErrorCode::ErrNotImplemented))
}

/// Adds or edits an account.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn save(
    _ctx: &Context,
    _actor: &Actor,
    _input: &AccountInput,
) -> Result<AccountRow, ServiceError> {
    todo()
}

/// One account.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn get(_ctx: &Context, _id: i64) -> Result<AccountRow, ServiceError> {
    todo()
}

/// The account list with its footer.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn list(_ctx: &Context, _input: &AccountsListInput) -> Result<AccountsList, ServiceError> {
    todo()
}

/// Makes an account active or inactive.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn set_active(
    _ctx: &Context,
    _actor: &Actor,
    _id: i64,
    _is_active: bool,
) -> Result<AccountRow, ServiceError> {
    todo()
}

/// The next free code.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn next_code(
    _ctx: &Context,
    _account_type: AccountType,
    _control_id: Option<i64>,
) -> Result<String, ServiceError> {
    todo()
}

/// Accounts that look like the one being typed.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn similar(
    _ctx: &Context,
    _input: &AccountsSimilarInput,
) -> Result<Vec<SimilarAccount>, ServiceError> {
    todo()
}

/// Checks the openings grid.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn openings_preview(
    _ctx: &Context,
    _input: &OpeningsInput,
) -> Result<OpeningsPreview, ServiceError> {
    todo()
}

/// Saves the openings grid.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn openings_save(
    _ctx: &Context,
    _actor: &Actor,
    _input: &OpeningsInput,
) -> Result<OpeningsSaved, ServiceError> {
    todo()
}
