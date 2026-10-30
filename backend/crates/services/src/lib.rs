//! `mandi-services`: one function and one database transaction per user action
//! (Code Standards B3), plus the in-memory search index and read-only assist queries.
//! Depends on `mandi-domain` and `mandi-db`.

pub mod accounts;
pub mod assist;
mod audit;
pub mod auth;
pub mod bikri;
pub mod cash;
pub mod cashbook;
pub mod clean_data;
pub mod coa;
pub mod context;
pub mod day;
pub mod dead;
pub mod drafts;
pub mod error;
mod floor;
pub mod fruit;
pub mod gardens;
pub mod khata;
pub mod ledger;
pub mod lent;
pub mod masters;
pub mod matching;
mod page;
pub mod partners;
pub mod ports;
pub mod printing;
pub mod purchases;
pub mod rules;
pub mod search;
pub mod secret;
pub mod settings;
// Run B, plan 22 (append-only block).
pub mod health;
pub mod month;
pub mod screen_prefs;
pub mod settings_ext;
pub mod spelling;
mod store;
pub mod testing;
pub mod translit;
pub mod trips;
mod ventures;
pub mod vouchers;

pub use context::{Context, Parts};
pub use error::{Rejection, ServiceError};
pub use secret::Secret;

/// `mandi-core`, re-exported for the app (error codes, money types).
pub use mandi_domain::core;

/// `mandi-domain`, re-exported for the app.
pub use mandi_domain as domain;

/// `mandi-db`, re-exported for the app: the `Db` handle, `DbError` and the seed (the app does
/// not depend on the db crate itself, Code Standards B1).
pub use mandi_db as db;

/// The name of this crate. Used as context in log lines and errors (Code Standards B12).
pub const CRATE: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_matches_tech_stack() {
        assert_eq!(CRATE, "mandi-services");
    }
}
