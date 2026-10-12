//! Accounts and the chart of accounts (PRD 10.1, 10.2, plan 07): what the screens send and what
//! they read back. No database here.
//!
//! - An account is made under a control: a seller under 03011, a buyer under 0304, a person who
//!   borrows money under 0103, any other under the control chosen. The code is the control code
//!   plus a 4-digit serial (PRD FR-10.2, FR-10.3).
//! - An opening debit or credit posts as an opening entry against ابتدائی فرق (06019999)
//!   (PRD FR-10.9). It is never a column that only looks like a balance.
//! - Every balance is read from the posted journal and sent as a [`Balance`]: an amount that is
//!   never negative and the side it stands on, so a screen never works out a side.

mod assist;
mod prepare;
mod types;

pub use assist::*;
pub use prepare::{
    MAX_WEEKDAY, OpeningRequest, PreparedAccount, account_code_for, opening_request,
    prepare_account,
};
pub use types::*;
