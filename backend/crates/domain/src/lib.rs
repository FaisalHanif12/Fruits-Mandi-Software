//! `mandi-domain`: business rules as pure functions (Code Standards B4).
//! Commission rules, بکری maths, journal building, partner split, matching.
//! Never touches the database, files, clock or network (Code Standards B1).
//! Depends only on `mandi-core`.
//!
//! Money lives here, so every `+ - * /` on integers must be a checked call
//! (Code Standards M7, B11). Tests may use plain operators.
#![deny(clippy::arithmetic_side_effects)]
#![cfg_attr(test, allow(clippy::arithmetic_side_effects))]

pub mod auth;
pub mod bikri;
pub mod cash;
pub mod ledger;
pub mod matching;
pub mod partners;
pub mod payables;
pub mod rules;
pub mod ventures;

/// `mandi-core`, re-exported so the crates above reach it through the one allowed path
/// (app -> services -> domain -> core).
pub use mandi_core as core;

/// The name of this crate. Used as context in log lines and errors (Code Standards B12).
pub const CRATE: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_matches_tech_stack() {
        assert_eq!(CRATE, "mandi-domain");
    }
}
