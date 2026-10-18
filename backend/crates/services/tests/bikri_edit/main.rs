//! Edit, cancel, undo, later sale and write-off of a بکری (PRD 12.8, 12.9, FR-7.47, FR-11.9, plan
//! 16): one test target, several files. The golden numbers E2E-13 to E2E-16b are in
//! `golden_day.rs`.
//!
//! - `edit`: what an edit writes and refuses: no posted row changes, closed days, versions, lots
//!   and lines that are not the posting's, cash, a one-time seller.
//! - `per_lot`: the per-lot seller commission moves when its line is removed (PRD FR-11.9).
//! - `matching`: bills and advances after an edit, a cancel and a later sale.
//! - `cancel`: cancel, undo and the later sale of a cancelled or part-sold بکری.
//! - `stock`: the list of goods left, write-off and its undo.
//! - `same_day`: the stable order of two entries of one day (vouchers and بکری, migration 0005).
//! - `atomic`: a failure forced after each step of an edit and of a cancel leaves no row.
//! - `properties`: random sales, later sales, edits and cancels keep the books.
//! - `fixtures`: the real answers of the new commands as JSON files for the frontend.
//!
//! Run it with `cargo test -p mandi-services --test bikri_edit`.
#![cfg(test)]

mod atomic;
mod cancel;
#[path = "../common/mod.rs"]
mod common;
mod edit;
mod fixtures;
#[allow(dead_code)]
#[path = "../bikri_save/helpers.rs"]
mod helpers;
mod matching;
mod per_lot;
mod properties;
mod same_day;
mod stock;
#[allow(dead_code)]
#[path = "../bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
mod tools;
