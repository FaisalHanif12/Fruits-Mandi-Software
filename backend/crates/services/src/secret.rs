//! A text that must never be printed: a password or a recovery code (Code Standards S2,
//! NFR-OBS-2). Its `Debug` shows `[hidden]`, and it has no `Display`.

use std::fmt;

/// A secret text.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// Wraps a text.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The text. Use it only to hash or compare.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret([hidden])")
    }
}

#[cfg(test)]
mod tests {
    use super::Secret;

    #[test]
    fn debug_never_shows_the_text() {
        let secret = Secret::new("hunter2-Xq9");
        assert_eq!(format!("{secret:?}"), "Secret([hidden])");
        assert_eq!(format!("{:?}", Some(&secret)), "Some(Secret([hidden]))");
        assert_eq!(secret.expose(), "hunter2-Xq9");
    }
}
