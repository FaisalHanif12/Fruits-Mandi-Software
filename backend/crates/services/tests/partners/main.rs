//! Partners and the live split (PRD 17.1 to 17.3, FR-6.5, FR-17.1 to FR-17.5, C9 to C12, E9,
//! E18, plan 11). One test target, several files:
//!
//! - `split_hook`: the share rows written by `ledger::post`, with the worked numbers of plan 11.
//! - `setup`: add, edit, list and read partners.
//! - `shares`: set the shares, the checks, the history, no back-dating.
//! - `overview`: the partner overview and its windows.
//! - `properties`: random postings and reversals.
//!
//! Run it with `cargo test -p mandi-services --test partners`.
#![cfg(test)]

#[path = "../common/mod.rs"]
mod common;
mod overview;
mod properties;
mod setup;
mod shares;
mod split_hook;
mod support;
