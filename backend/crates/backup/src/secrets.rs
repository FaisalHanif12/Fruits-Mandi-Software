//! Secrets of the backup: the backup key and the cloud account keys (NFR-SEC-7, Code Standards
//! S3). The real store is the Windows Credential Manager (the macOS keychain in development);
//! tests use memory. A secret is never written to the data file, a log or a settings row.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use crate::error::BackupError;

/// Credential Manager service name (Tech Stack 7: the bundle identifier). The same service as
/// the audit chain key; the entry names differ.
pub const KEYRING_SERVICE: &str = "pk.mandisoftware.app";

/// The secrets this crate keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretName {
    /// The key of USB and cloud copies (FR-20.10).
    BackupKey,
    /// The cloud account's access key id.
    CloudAccessKeyId,
    /// The cloud account's secret access key.
    CloudSecretAccessKey,
}

impl SecretName {
    /// The entry name in the Credential Manager.
    pub fn entry(self) -> &'static str {
        match self {
            SecretName::BackupKey => "backup-key",
            SecretName::CloudAccessKeyId => "cloud-access-key-id",
            SecretName::CloudSecretAccessKey => "cloud-secret-access-key",
        }
    }
}

/// Where the secrets live.
pub trait SecretStore: Send + Sync {
    /// The secret, or `None` if it was never stored.
    ///
    /// # Errors
    /// `ERR_INTERNAL` when the store cannot be reached.
    fn get(&self, name: SecretName) -> Result<Option<String>, BackupError>;

    /// Stores the secret, replacing an older one.
    ///
    /// # Errors
    /// `ERR_INTERNAL` when the store cannot be reached.
    fn set(&self, name: SecretName, value: &str) -> Result<(), BackupError>;

    /// Removes the secret. Removing one that is not there is not an error.
    ///
    /// # Errors
    /// `ERR_INTERNAL` when the store cannot be reached.
    fn delete(&self, name: SecretName) -> Result<(), BackupError>;
}

/// The Windows Credential Manager through `keyring`.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyringSecretStore;

fn keyring_error(error: &keyring::Error) -> BackupError {
    BackupError::internal(format!("credential store: {error}"))
}

fn entry(name: SecretName) -> Result<keyring::Entry, BackupError> {
    keyring::Entry::new(KEYRING_SERVICE, name.entry()).map_err(|e| keyring_error(&e))
}

impl SecretStore for KeyringSecretStore {
    fn get(&self, name: SecretName) -> Result<Option<String>, BackupError> {
        match entry(name)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(keyring_error(&error)),
        }
    }

    fn set(&self, name: SecretName, value: &str) -> Result<(), BackupError> {
        entry(name)?
            .set_password(value)
            .map_err(|e| keyring_error(&e))
    }

    fn delete(&self, name: SecretName) -> Result<(), BackupError> {
        match entry(name)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(keyring_error(&error)),
        }
    }
}

/// The secrets live in memory. For tests.
#[derive(Debug, Default)]
pub struct MemorySecretStore(Mutex<HashMap<&'static str, String>>);

impl SecretStore for MemorySecretStore {
    fn get(&self, name: SecretName) -> Result<Option<String>, BackupError> {
        let map = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(map.get(name.entry()).cloned())
    }

    fn set(&self, name: SecretName, value: &str) -> Result<(), BackupError> {
        let mut map = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        map.insert(name.entry(), value.to_owned());
        Ok(())
    }

    fn delete(&self, name: SecretName) -> Result<(), BackupError> {
        let mut map = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        map.remove(name.entry());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_sets_gets_and_deletes() {
        let store = MemorySecretStore::default();
        assert_eq!(store.get(SecretName::BackupKey).unwrap(), None);
        store.set(SecretName::BackupKey, "abc").unwrap();
        assert_eq!(
            store.get(SecretName::BackupKey).unwrap().as_deref(),
            Some("abc")
        );
        assert_eq!(store.get(SecretName::CloudAccessKeyId).unwrap(), None);
        store.delete(SecretName::BackupKey).unwrap();
        store.delete(SecretName::BackupKey).unwrap();
        assert_eq!(store.get(SecretName::BackupKey).unwrap(), None);
    }

    #[test]
    fn entry_names_are_distinct() {
        let names = [
            SecretName::BackupKey.entry(),
            SecretName::CloudAccessKeyId.entry(),
            SecretName::CloudSecretAccessKey.entry(),
        ];
        assert_eq!(
            names
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3
        );
    }
}
