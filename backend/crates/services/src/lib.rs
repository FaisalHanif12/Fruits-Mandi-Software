//! `mandi-services`: one function and one database transaction per user action
//! (Code Standards B3), plus the in-memory search index and read-only assist queries.
//! Depends on `mandi-domain` and `mandi-db`.

/// `mandi-core`, re-exported for the app (error codes, money types).
pub use mandi_domain::core;

/// The name of this crate. Used as context in log lines and errors (Code Standards B12).
pub const CRATE: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_matches_tech_stack() {
        assert_eq!(CRATE, "mandi-services");
    }
}
