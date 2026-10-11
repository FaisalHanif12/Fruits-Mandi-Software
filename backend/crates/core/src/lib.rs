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
pub mod dates;
pub mod english_words;
pub mod error;
pub mod masters;
pub mod money;
pub mod search;
pub mod text;
pub mod translit;
pub mod urdu_words;
pub mod validate;

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
