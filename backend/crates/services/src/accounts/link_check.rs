//! The rules for linking the two accounts of one person (PRD E10, FR-10.2; gate finding T4-2 and the
//! money review's note on link integrity, plan 80).
//!
//! One person can be a seller and a buyer. His two accounts are linked so the party card and the
//! ledger heading can show one combined balance. The combined balance adds exactly two ledgers, so a
//! link must be one clean pair:
//!
//! 1. It joins a seller and a buyer (`ERR_LINK_NOT_SELLER_BUYER`). Not two sellers, not a
//!    money-lent or other account, not چکتا کھاتہ or another system account.
//! 2. Neither account is linked to a third (`ERR_LINK_ALREADY_LINKED`): the other account must not
//!    be linked to anyone else, and nobody else may point at it; and when an existing account is
//!    edited, nobody else may point at it either. That also refuses a chain A→B→C. A reciprocal pair
//!    (A→B and B→A) is one pair and is allowed.
//!
//! Example: seller رحیم is linked to buyer اکرم. Linking seller کریم to اکرم is refused (اکرم is already
//! linked), and linking اکرم to a second seller is refused too.

use mandi_db::accounts as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{AccountType, PreparedAccount};
use mandi_domain::core::error::ErrorCode;

use crate::error::ServiceError;

/// The field every refusal points at.
const FIELD: &str = "linked_account_id";

fn refuse(code: ErrorCode) -> ServiceError {
    ServiceError::reject_field(code, FIELD)
}

/// Checks the link typed on an account form inside the save's transaction. Nothing is written. No
/// link (cleared or never set) is always fine.
///
/// # Errors
/// `ERR_LINK_NOT_SELLER_BUYER` and `ERR_LINK_ALREADY_LINKED` on `linked_account_id`;
/// `ERR_NOT_FOUND` on `linked_account_id` for an account that does not exist; `ServiceError::Db`.
pub(super) fn check(conn: &Connection, prepared: &PreparedAccount) -> Result<(), ServiceError> {
    let Some(target_id) = prepared
        .references
        .iter()
        .find(|reference| reference.field == FIELD)
        .map(|reference| reference.id)
    else {
        return Ok(());
    };
    let own_id = prepared.id;
    // A link to itself is refused by the form's own check (`prepare`).
    if Some(target_id) == own_id {
        return Err(refuse(ErrorCode::ErrNotFound));
    }
    let target = repo::get(conn, target_id)?.ok_or_else(|| refuse(ErrorCode::ErrNotFound))?;

    // 1. A seller and a buyer, both ordinary accounts.
    let pair = matches!(
        (prepared.account_type, target.account_type),
        (AccountType::Seller, AccountType::Buyer) | (AccountType::Buyer, AccountType::Seller)
    );
    if !pair || target.is_system || target.is_chukta_khata {
        return Err(refuse(ErrorCode::ErrLinkNotSellerBuyer));
    }

    // 2. One clean pair. The other account links to nobody but this one ...
    let target_links_elsewhere = target
        .linked_account
        .as_ref()
        .is_some_and(|linked| Some(linked.id) != own_id);
    // ... and nobody else points at it ...
    let pointed_at_elsewhere = !repo::accounts_linking_to(conn, target_id, own_id)?.is_empty();
    // ... and, for an account being edited, nobody else points at it either (it would be a chain).
    let own_pointed_at_elsewhere = match own_id {
        Some(own) => !repo::accounts_linking_to(conn, own, Some(target_id))?.is_empty(),
        None => false,
    };
    if target_links_elsewhere || pointed_at_elsewhere || own_pointed_at_elsewhere {
        return Err(refuse(ErrorCode::ErrLinkAlreadyLinked));
    }
    Ok(())
}
