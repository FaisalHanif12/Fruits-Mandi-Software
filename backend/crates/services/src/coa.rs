//! The chart of accounts (PRD FR-10.1, plan 07). Contract stubs: T2 fills them in.

use mandi_domain::core::accounts::{CoaControl, CoaHead, CoaTree};
use mandi_domain::core::error::ErrorCode;

use crate::context::Context;
use crate::error::ServiceError;

/// Heads with controls with accounts and their balances.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn tree(_ctx: &Context) -> Result<CoaTree, ServiceError> {
    Err(ServiceError::reject(ErrorCode::ErrNotImplemented))
}

/// The heads.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn heads(_ctx: &Context) -> Result<Vec<CoaHead>, ServiceError> {
    Err(ServiceError::reject(ErrorCode::ErrNotImplemented))
}

/// The controls.
///
/// # Errors
/// `ERR_NOT_IMPLEMENTED` until T2.
pub fn controls(_ctx: &Context) -> Result<Vec<CoaControl>, ServiceError> {
    Err(ServiceError::reject(ErrorCode::ErrNotImplemented))
}
