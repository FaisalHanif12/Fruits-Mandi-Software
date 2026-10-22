//! `mandi-backup`: local, USB and cloud backup, restore and verification (PRD 20, NFR 4,
//! plan 18). Depends on `mandi-db` and `mandi-core`.

pub mod api;
pub mod cloud;
pub mod daily_pack;
pub mod error;
pub mod io;
pub mod key;
pub mod model;
pub mod names;
pub mod pack;
pub mod restore;
pub mod retention;
pub mod run;
pub mod schedule;
pub mod secrets;
pub mod service;
pub mod settings;
pub mod snapshot;
pub mod state;

pub use error::{BackupCode, BackupError, CloudReason};

/// The name of this crate. Used as context in log lines and errors (Code Standards B12).
pub const CRATE: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_matches_tech_stack() {
        assert_eq!(CRATE, "mandi-backup");
    }
}
