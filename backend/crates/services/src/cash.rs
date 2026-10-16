//! Cash in hand after an entry (PRD FR-6.7, plan 12): the one place every cash-out action asks
//! "will cash go below zero, and must the user confirm?".
//!
//! The rule is `mandi_domain::cash::check`. This module reads the balance of روکڑ and turns an
//! unconfirmed entry into the error `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. The preview of a screen
//! calls [`cash_after_entry`] to show the amber note; the save (plans 13, 15, 16, 16b, 17) calls
//! [`cash_after_entry`] and [`require_confirmed`] inside its own transaction, with the change
//! worked out from the journal entry it is about to post (`mandi_domain::cash::net_change`).

use mandi_db::accounts as account_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::cash;
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use mandi_domain::core::money::Paisa;

use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger;

/// روکڑ, cash in hand (PRD FR-10.4).
pub const CASH_ACCOUNT_CODE: &str = "01010001";

/// Cash in hand now: the posted debits less the posted credits of روکڑ, debit positive.
///
/// # Errors
/// `ServiceError::Data` if the cash account is missing; `ServiceError::Db`.
pub fn cash_in_hand(conn: &Connection) -> Result<Paisa, ServiceError> {
    let id = account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ServiceError::Data("the cash account is missing".to_owned()))?;
    ledger::signed(ledger::balance_in(conn, id, None, None)?)
}

/// What an entry does to cash in hand and whether the user must confirm it. `change` is what the
/// entry adds to cash (negative when it takes cash out). Only reads.
///
/// # Errors
/// `ServiceError::Data`, `ServiceError::Db`, `ERR_OVERFLOW`.
pub fn cash_after_entry(conn: &Connection, change: Paisa) -> Result<CashCheck, ServiceError> {
    cash::check(cash_in_hand(conn)?, change).map_err(money_error)
}

/// Refuses an entry that needs the user's confirmation and did not get it (PRD FR-6.7). The
/// error carries `cash_after_paisa`, so the screen can ask "after this entry cash in hand will
/// be −5,000. Is this entry correct?".
///
/// # Errors
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
pub fn require_confirmed(check: &CashCheck, confirmed: bool) -> Result<(), ServiceError> {
    if check.needs_confirm && !confirmed {
        return Err(ServiceError::Rejected(vec![Rejection {
            code: ErrorCode::ErrCashBelowZeroNotConfirmed,
            field: None,
            values: std::collections::BTreeMap::from([(
                "cash_after_paisa".to_owned(),
                ErrorValue::Number(check.cash_after_paisa.as_paisa()),
            )]),
        }]));
    }
    Ok(())
}
