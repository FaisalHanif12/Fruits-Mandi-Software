//! Where the OpenRouter key lives (PRD FR-7.85, NFR-SEC-7, NFR-SEC-15, Code Standards Q8).
//!
//! The key is kept only in Windows Credential Manager, under its own entry. It is read just
//! before each model call and dropped after it: never kept in the app's state, never written to a
//! log, a setting, an audit row, a backup or a reply to the screens. Everything that shows it
//! shows at most its last 4 characters ([`last4`]).
//!
//! [`KeyringStore`] is the real store. [`MemoryKeyStore`] keeps it in memory for tests, so no test
//! touches the real Credential Manager.

use std::sync::{Mutex, PoisonError};

use keyring::Entry;

/// The service name of every secret of the app (the same as the backup and audit keys use).
pub const KEYRING_SERVICE: &str = "pk.mandisoftware.app";
/// The entry of this key: its own, apart from the backup and audit keys.
pub const KEYRING_ENTRY: &str = "openrouter_api_key";
/// Longest key accepted.
pub const MAX_KEY_CHARS: usize = 200;

/// The store could not be reached. The text never holds the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyStoreError(pub String);

/// A place for the one key.
pub trait KeyStore: Send + Sync {
    /// The key, or `None` when none is saved.
    ///
    /// # Errors
    /// The store could not be reached.
    fn get(&self) -> Result<Option<String>, KeyStoreError>;

    /// Saves the key, replacing an older one.
    ///
    /// # Errors
    /// The store could not be reached.
    fn set(&self, key: &str) -> Result<(), KeyStoreError>;

    /// Removes the key. Removing one that is not there is not an error.
    ///
    /// # Errors
    /// The store could not be reached.
    fn delete(&self) -> Result<(), KeyStoreError>;
}

/// The Windows Credential Manager through `keyring` (the macOS keychain in development).
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyringStore;

fn entry() -> Result<Entry, KeyStoreError> {
    Entry::new(KEYRING_SERVICE, KEYRING_ENTRY).map_err(|e| KeyStoreError(format!("{e}")))
}

impl KeyStore for KeyringStore {
    fn get(&self) -> Result<Option<String>, KeyStoreError> {
        match entry()?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(KeyStoreError(format!("{error}"))),
        }
    }

    fn set(&self, key: &str) -> Result<(), KeyStoreError> {
        entry()?
            .set_password(key)
            .map_err(|e| KeyStoreError(format!("{e}")))
    }

    fn delete(&self) -> Result<(), KeyStoreError> {
        match entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(KeyStoreError(format!("{error}"))),
        }
    }
}

/// The key in memory. For tests.
#[derive(Default)]
pub struct MemoryKeyStore(Mutex<Option<String>>);

impl std::fmt::Debug for MemoryKeyStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never the key, not even in a test failure.
        f.write_str("MemoryKeyStore([hidden])")
    }
}

impl MemoryKeyStore {
    /// A store that already holds a key.
    pub fn with_key(key: &str) -> Self {
        Self(Mutex::new(Some(key.to_owned())))
    }
}

impl KeyStore for MemoryKeyStore {
    fn get(&self) -> Result<Option<String>, KeyStoreError> {
        Ok(self
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone())
    }

    fn set(&self, key: &str) -> Result<(), KeyStoreError> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(key.to_owned());
        Ok(())
    }

    fn delete(&self) -> Result<(), KeyStoreError> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = None;
        Ok(())
    }
}

/// The last 4 characters of a key, the only part of it that is ever shown ("••••" and these).
///
/// Example: a key that ends in `…ab12` shows `ab12`. A key of 4 characters or fewer shows none, so
/// a short key is never shown whole.
pub fn last4(key: &str) -> Option<String> {
    let chars: Vec<char> = key.chars().collect();
    let shown: Vec<char> = chars.iter().rev().take(4).rev().copied().collect();
    (chars.len() > shown.len()).then(|| shown.into_iter().collect())
}

/// True when the text is an acceptable key to save: not empty, no spaces or control characters,
/// at most [`MAX_KEY_CHARS`] characters. (The app does not know the form of an OpenRouter key and
/// does not guess; the connection test proves it.)
pub fn is_well_formed(key: &str) -> bool {
    !key.is_empty()
        && key.chars().count() <= MAX_KEY_CHARS
        && key.chars().all(|c| !c.is_whitespace() && !c.is_control())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_last_four_characters_are_ever_shown() {
        assert_eq!(last4("sk-test-1234abcd").as_deref(), Some("abcd"));
        assert_eq!(last4("abcd"), None);
        assert_eq!(last4("abc"), None);
        assert_eq!(last4("abcde").as_deref(), Some("bcde"));
        assert_eq!(last4(""), None);
    }

    #[test]
    fn a_key_with_spaces_or_no_text_is_not_well_formed() {
        assert!(is_well_formed("sk-or-v1-abc123"));
        assert!(!is_well_formed(""));
        assert!(!is_well_formed("has space"));
        assert!(!is_well_formed("two\nlines"));
        assert!(!is_well_formed(&"k".repeat(MAX_KEY_CHARS + 1)));
        assert!(is_well_formed(&"k".repeat(MAX_KEY_CHARS)));
    }

    #[test]
    fn the_memory_store_keeps_and_forgets_a_key_and_never_prints_it() {
        let store = MemoryKeyStore::default();
        assert_eq!(store.get(), Ok(None));
        store.set("sk-test-xyz").unwrap();
        assert_eq!(store.get().unwrap().as_deref(), Some("sk-test-xyz"));
        assert!(!format!("{store:?}").contains("sk-test"));
        store.delete().unwrap();
        store.delete().unwrap();
        assert_eq!(store.get(), Ok(None));
    }
}
