//! `mandi-core`: pure helpers with no business rules and no database.
//! Money (`Paisa`, `Bps`, rounding, parsing), dates, typed IDs, search keys,
//! translit, amounts in words and input validation (Tech Stack 5.4).
//! Depends on nothing of ours.
//!
//! Money lives here, so every `+ - * /` on integers must be a checked call
//! (Code Standards M7, B11). Tests may use plain operators.
#![deny(clippy::arithmetic_side_effects)]
#![cfg_attr(test, allow(clippy::arithmetic_side_effects))]

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
