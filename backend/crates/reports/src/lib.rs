//! `mandi-reports`: report queries, print data and Excel export (PRD 19).
//! Depends on `mandi-db` and `mandi-core`.
//!
//! - [`print`]: the print data of a saved بکری: the seller bill, the buyer slips and the
//!   پختہ slip (plan 14, PRD FR-12.12, FR-19.3 to FR-19.7). Read only; every number is a stored
//!   value, never worked out again (Code Standards B20).

pub mod error;
pub mod print;

pub use error::ReportsError;

/// The name of this crate. Used as context in log lines and errors (Code Standards B12).
pub const CRATE: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_matches_tech_stack() {
        assert_eq!(CRATE, "mandi-reports");
    }
}
