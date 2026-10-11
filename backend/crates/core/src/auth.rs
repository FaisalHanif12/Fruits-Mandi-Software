//! Roles and languages: the two small closed lists that sign-in, settings and prints share
//! (PRD Section 2, FR-2.4, FR-7.30).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a user may do (PRD Section 2). The order matters: a higher role can do everything a
/// lower one can (FR-2.4), so `Role::Owner >= Role::Munshi >= Role::Partner`.
///
/// Stored in `users.role` as `owner`, `munshi` or `partner`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// شراکت دار: sees the dashboard, analytics and reports. No entries.
    Partner,
    /// منشی: all entries and reports. Not settings, rules, shares, reopen or restore.
    Munshi,
    /// مالک: everything.
    Owner,
}

impl Role {
    /// The storage form, as in the `users.role` column.
    pub const fn as_str(self) -> &'static str {
        match self {
            Role::Partner => "partner",
            Role::Munshi => "munshi",
            Role::Owner => "owner",
        }
    }

    /// Reads the storage form. `None` for anything else.
    pub fn parse(text: &str) -> Option<Role> {
        match text {
            "partner" => Some(Role::Partner),
            "munshi" => Some(Role::Munshi),
            "owner" => Some(Role::Owner),
            _ => None,
        }
    }

    /// True if this role can do what `needed` can (FR-2.4).
    ///
    /// ```
    /// use mandi_core::auth::Role;
    /// assert!(Role::Owner.is_at_least(Role::Munshi));
    /// assert!(!Role::Partner.is_at_least(Role::Munshi));
    /// ```
    pub fn is_at_least(self, needed: Role) -> bool {
        self >= needed
    }
}

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
    fn roles_are_ordered_and_round_trip() {
        assert!(Role::Owner > Role::Munshi && Role::Munshi > Role::Partner);
        for role in [Role::Partner, Role::Munshi, Role::Owner] {
            assert_eq!(Role::parse(role.as_str()), Some(role));
            assert!(role.is_at_least(role));
        }
        assert!(!Role::Munshi.is_at_least(Role::Owner));
        assert_eq!(Role::parse("admin"), None);
        assert_eq!(serde_json::to_string(&Role::Munshi).unwrap(), "\"munshi\"");
    }

    #[test]
    fn languages_round_trip() {
        for language in [Language::Ur, Language::En] {
            assert_eq!(Language::parse(language.as_str()), Some(language));
        }
        assert_eq!(Language::parse("fr"), None);
        assert_eq!(serde_json::to_string(&Language::Ur).unwrap(), "\"ur\"");
    }
}
