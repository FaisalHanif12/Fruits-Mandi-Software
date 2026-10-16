//! The بکری entry screen (PRD 12.1 to 12.5, plan 12): what the screen sends, what the live
//! preview sends back, and the reading of the typed text.
//!
//! The screen sends everything as text (Tech Stack 1.1). [`prepare_draft`] reads the text into
//! numbers and collects every mistake with the field it belongs to (Code Standards 4.1 layer 2).
//! The rules and the maths are in `mandi_domain::bikri::calc_posting`; the database reads are in
//! `mandi_services::bikri`. Preview and save call the same domain function (Code Standards B5).
//!
//! Field paths in errors and warnings:
//!
//! | Path | Field |
//! |---|---|
//! | `header.entry_date` | a header field |
//! | `lots` | the whole list of lots |
//! | `lots[0].qty_received` | a lot field |
//! | `lots[0].sales[2].qty` | a sale line field |
//!
//! Example: the third line of the first lot has `"abc"` as qty, so the preview returns
//! `FieldError { code: ErrNotANumber, field: "lots[0].sales[2].qty", .. }`.

mod input;
mod prepare;
mod prepared;
mod preview;
mod saved;
#[cfg(test)]
mod tests;

pub use input::*;
pub use prepare::{prepare_draft, split_freight};
pub use prepared::*;
pub use preview::*;
pub use saved::*;

/// The most lots in one بکری (NFR-SCL-3).
pub const MAX_LOTS: usize = 50;
/// The most sale lines in one بکری, over all its lots (NFR-SCL-3).
pub const MAX_SALE_LINES: usize = 500;
