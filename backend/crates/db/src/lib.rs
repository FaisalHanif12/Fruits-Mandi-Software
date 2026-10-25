//! `mandi-db`: SQLite connection, migrations and repositories (Code Standards 5.3).
//! Depends only on `mandi-core`.
//!
//! - [`Db`]: one write connection and one read connection; `Db::write` is the one
//!   transaction of a user action, with a free disk check (plan 03 Sections 4.2, 4.3).
//! - [`migrate`]: `0001_init.sql` (all of Architecture 15) with a full copy before an upgrade.
//! - [`seed`]: the system accounts of PRD FR-10.4.
//! - [`audit`]: audit rows with a keyed hash chain.
//! - [`DbError`]: the one error type, with a code for the screen.

pub mod accounts;
pub mod audit;
pub mod bikri;
pub mod cashbook;
pub mod clock;
mod db;
pub mod disk;
pub mod drafts;
mod error;
pub mod expense_heads;
pub mod fruit;
pub mod gardens;
pub mod ledger;
pub mod masters;
pub mod matching;
pub mod migrate;
pub mod open_lines;
pub mod partners;
pub mod purchases;
pub mod rules;
pub mod seed;
pub mod statement;
pub mod ventures;
pub mod vouchers;

pub use db::{BUSY_TIMEOUT, CHECK_WAIT, DataState, Db, DbOptions};
pub use error::DbError;
/// SQLite types, re-exported so services use the same version (`Transaction`, `params!`).
pub use rusqlite;

/// The name of this crate. Used as context in log lines and errors (Code Standards B12).
pub const CRATE: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_matches_tech_stack() {
        assert_eq!(CRATE, "mandi-db");
    }
}
