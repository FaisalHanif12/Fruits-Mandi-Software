//! Saving a بکری (PRD FR-12.8, FR-12.9, FR-13, FR-15.4, FR-15.8, plan 13): one test target, several
//! files. The golden numbers of Event 1 and Event 2 are in `golden_day.rs`.
//!
//! - `save`: what a save writes and refuses: numbering, errors together, charges, freight, the
//!   one-time person, advances, a lot with no sale.
//! - `all_or_nothing`: a failure forced after each step leaves no row (NFR-REL-1).
//! - `matching`: bill matching and advance matching on real data, and the canonical re-match.
//! - `reads`: a saved بکری, the list, find, first / previous / next / last, global search, the
//!   one-time people and the register.
//! - `drafts`: the unsaved work of a screen (NFR-REL-2).
//! - `properties`: random بکری through the real database.
//! - `fixtures`: the real answers of the commands as JSON files for the frontend; fails when a
//!   file is not the real answer any more.
//!
//! Run it with `cargo test -p mandi-services --test bikri_save`.
#![cfg(test)]

mod all_or_nothing;
#[path = "../common/mod.rs"]
mod common;
mod drafts;
mod fixtures;
mod helpers;
mod matching;
mod properties;
mod reads;
mod save;
#[allow(dead_code)]
#[path = "../bikri_preview/support.rs"]
mod support;
