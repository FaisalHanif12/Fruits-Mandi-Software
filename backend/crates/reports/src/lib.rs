//! `mandi-reports`: report queries, print data and Excel export (PRD 19).
//! Depends on `mandi-db` and `mandi-core`.

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
