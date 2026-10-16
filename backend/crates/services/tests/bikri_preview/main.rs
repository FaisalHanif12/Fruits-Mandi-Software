//! The بکری entry screen: the live preview, the freight split, the memories and the cash note
//! (PRD 12.1 to 12.5, FR-7.49 to FR-7.56, FR-6.7, plan 12). One test target, several files:
//!
//! - `preview`: the worked numbers of PRD 4.4 and 4.5 through the real services and database.
//! - `checks`: errors and warnings: shares, credit limit, rate check, cash note, dates.
//! - `memory`: truck, seller and last-rate memories, read from saved data.
//! - `properties`: random drafts through the real database.
//!
//! Run it with `cargo test -p mandi-services --test bikri_preview`.
#![cfg(test)]

mod checks;
#[path = "../common/mod.rs"]
mod common;
mod memory;
mod preview;
mod properties;
mod support;
