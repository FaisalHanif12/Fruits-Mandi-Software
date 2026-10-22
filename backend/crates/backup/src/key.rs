//! The backup key (PRD FR-20.10, NFR-BAK-6, NFR-BAK-7).
//!
//! 24 letters and digits in six groups of four, for example `K7MQ-2XWD-...`. It has 120 bits of
//! chance in it, so a long key-stretching step is not needed. The letters I and O and the digits
//! 0 and 1 are left out, so a printed key is easy to read and type. The key is shown once at
//! setup. After that it lives only in the Windows Credential Manager, never in the data file.

use std::fmt;

use age::secrecy::SecretString;

use crate::error::{BackupCode, BackupError};

/// The 32 symbols a key is made of. 256 is a multiple of 32, so a random byte masked to 5 bits
/// picks every symbol with the same chance.
const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
/// Symbols in a key.
pub const KEY_LEN: usize = 24;
/// Symbols in one printed group.
const GROUP: usize = 4;

/// The backup key. Its `Debug` text never shows it (NFR-OBS-2).
#[derive(Clone)]
pub struct BackupKey(String);

impl BackupKey {
    /// A new random key from the operating system.
    ///
    /// # Errors
    /// `ERR_INTERNAL` if the system has no random source.
    pub fn generate() -> Result<Self, BackupError> {
        let mut bytes = [0_u8; KEY_LEN];
        getrandom::fill(&mut bytes)
            .map_err(|e| BackupError::internal(format!("no random source: {e}")))?;
        let text = bytes
            .iter()
            .map(|byte| {
                ALPHABET
                    .get(usize::from(byte & 31))
                    .copied()
                    .map_or('A', char::from)
            })
            .collect();
        Ok(Self(text))
    }

    /// Reads a key the Owner typed: any case, with or without dashes and spaces.
    ///
    /// # Errors
    /// `ERR_BACKUP_KEY_FORMAT` on field `key` when it is not 24 symbols of the alphabet.
    pub fn parse(text: &str) -> Result<Self, BackupError> {
        let clean: String = text
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .map(|c| c.to_ascii_uppercase())
            .collect();
        let valid = clean.len() == KEY_LEN && clean.bytes().all(|b| ALPHABET.contains(&b));
        if !valid {
            return Err(
                BackupError::new(BackupCode::ErrBackupKeyFormat, "key has the wrong form")
                    .on_field("key"),
            );
        }
        Ok(Self(clean))
    }

    /// The key as printed: `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`.
    pub fn display(&self) -> String {
        self.0
            .as_bytes()
            .chunks(GROUP)
            .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// The 24 symbols without dashes, for the Credential Manager.
    pub(crate) fn raw(&self) -> &str {
        &self.0
    }

    /// The passphrase that age uses.
    pub(crate) fn passphrase(&self) -> SecretString {
        SecretString::from(self.0.clone())
    }

    /// True when both keys are the same. Takes the same time wherever they differ.
    pub fn same_as(&self, other: &BackupKey) -> bool {
        let (a, b) = (self.0.as_bytes(), other.0.as_bytes());
        a.len() == b.len() && a.iter().zip(b).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0
    }
}

impl fmt::Debug for BackupKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BackupKey([hidden])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_has_six_groups_of_four_from_the_alphabet() {
        let key = BackupKey::generate().unwrap();
        let shown = key.display();
        assert_eq!(shown.len(), 24 + 5);
        assert_eq!(shown.matches('-').count(), 5);
        assert!(
            shown
                .chars()
                .all(|c| c == '-' || ALPHABET.contains(&(c as u8)))
        );
        assert_ne!(
            BackupKey::generate().unwrap().display(),
            shown,
            "two keys differ"
        );
    }

    #[test]
    fn a_typed_key_is_read_in_any_case_with_or_without_dashes() {
        let key = BackupKey::generate().unwrap();
        let shown = key.display();
        let typed = shown.to_lowercase().replace('-', " ");
        assert!(key.same_as(&BackupKey::parse(&typed).unwrap()));
        assert!(key.same_as(&BackupKey::parse(&shown.replace('-', "")).unwrap()));
    }

    #[test]
    fn a_key_in_the_wrong_form_points_at_the_key_field() {
        for bad in [
            "",
            "ABCD",
            "ABCD-EFGH-JKLM-NPQR-STUV-WXY1",
            "0000-0000-0000-0000-0000-0000",
        ] {
            let error = BackupKey::parse(bad).unwrap_err();
            assert_eq!(error.code, BackupCode::ErrBackupKeyFormat, "{bad}");
            assert_eq!(error.field.as_deref(), Some("key"));
        }
    }

    #[test]
    fn debug_text_never_shows_the_key() {
        let key = BackupKey::generate().unwrap();
        let text = format!("{key:?}");
        assert!(!text.contains(&key.display().replace('-', "")));
        assert!(text.contains("hidden"));
    }
}
