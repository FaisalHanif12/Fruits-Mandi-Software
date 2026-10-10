//! `mandi-db`: SQLite connection, migrations and repositories (Code Standards 5.3).
//! Depends only on `mandi-core`.

/// The name of this crate. Used as context in log lines and errors (Code Standards B12).
pub const CRATE: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_matches_tech_stack() {
        assert_eq!(CRATE, "mandi-db");
    }
}
