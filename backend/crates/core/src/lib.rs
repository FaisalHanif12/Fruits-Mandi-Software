//! `mandi-core`: pure helpers with no business rules and no database.
//! Money (`Paisa`, `Bps`, rounding, parsing), dates, search keys, amounts in words and
//! input validation (Tech Stack 5.4). Depends on nothing of ours. Never reads the clock:
//! "today" is always passed in (Code Standards B1).
//!
//! Money lives here, so every `+ - * /` on integers must be a checked call
//! (Code Standards M7, B11). Tests may use plain operators.
#![deny(clippy::arithmetic_side_effects)]
#![cfg_attr(test, allow(clippy::arithmetic_side_effects))]

pub mod accounts;
pub mod auth;
pub mod bikri;
pub mod capital;
pub mod cash;
pub mod cashbook;
pub mod dates;
pub mod drafts;
pub mod english_words;
pub mod error;
pub mod expenses;
pub mod fruit;
pub mod gardens;
pub mod general;
pub mod ids;
pub mod khata;
pub mod lent;
pub mod masters;
pub mod money;
pub mod partners;
pub mod printing;
pub mod purchases;
pub mod rules;
pub mod search;
pub mod statement;
pub mod text;
pub mod translit;
pub mod trips;
pub mod urdu_words;
pub mod validate;
pub mod ventures;
pub mod vouchers;

/// The name of this crate. Used as context in log lines and errors (Code Standards B12).
pub const CRATE: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_matches_tech_stack() {
        assert_eq!(CRATE, "mandi-core");
    }
}
