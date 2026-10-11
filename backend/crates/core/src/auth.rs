//! Languages and preference keys: the small closed lists that sign-in, settings and prints
//! share (PRD FR-7.30). There are no roles: the Owner is the only user (PRD Section 2).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The app language (PRD Section 7.5): Urdu right-to-left or English left-to-right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// اردو, the default.
    Ur,
    /// English.
    En,
}

impl Language {
    /// The storage form: `ur` or `en`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Language::Ur => "ur",
            Language::En => "en",
        }
    }

    /// Reads the storage form. `None` for anything else.
    pub fn parse(text: &str) -> Option<Language> {
        match text {
            "ur" => Some(Language::Ur),
            "en" => Some(Language::En),
            _ => None,
        }
    }
}

/// The preferences one user keeps (Architecture 15.3 `user_prefs`, PRD FR-7.30, FR-7.45).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PrefKey {
    /// `ur` or `en`. Also becomes the language of the sign-in screen on this computer.
    Language,
    /// 80 to 150 in steps of 10 (NFR-USE-18).
    TextSize,
    /// The route of the screen open at sign-out, for the next sign-in (PRD FR-7.45).
    LastScreen,
}

impl PrefKey {
    /// The key in the `user_prefs` table.
    pub const fn as_str(self) -> &'static str {
        match self {
            PrefKey::Language => "language",
            PrefKey::TextSize => "text_size",
            PrefKey::LastScreen => "last_screen",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_round_trip() {
        for language in [Language::Ur, Language::En] {
            assert_eq!(Language::parse(language.as_str()), Some(language));
        }
        assert_eq!(Language::parse("fr"), None);
        assert_eq!(serde_json::to_string(&Language::Ur).unwrap(), "\"ur\"");
    }
}
