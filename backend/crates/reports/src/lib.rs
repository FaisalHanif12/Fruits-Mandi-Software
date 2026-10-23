//! `mandi-reports`: report queries, print data and Excel export (PRD 19).
//! Depends on `mandi-db` and `mandi-core`.
//!
//! - [`reports`]: the report engine (catalogue, filters, pages with totals, print data) and the
//!   Accounts group R11, R13, R14, R16 to R19 (plan 20, PRD 19).
//! - [`analytics`]: the analytics views A to E and H over the sale facts (plan 21, PRD 18).
//! - [`audit`]: the audit viewer, a read-only paged list of the audit log (plan 22, PRD FR-21.3).
//! - [`people`]: the one-time people of the global search (PRD FR-5.4).
//! - [`print`]: the print data of a saved بکری: the seller bill, the buyer slips and the
//!   پختہ slip (plan 14, PRD FR-12.12, FR-19.3 to FR-19.7). Read only; every number is a stored
//!   value, never worked out again (Code Standards B20).

pub mod analytics;
pub mod audit;
pub mod error;
pub mod people;
pub mod print;
pub mod reports;

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
