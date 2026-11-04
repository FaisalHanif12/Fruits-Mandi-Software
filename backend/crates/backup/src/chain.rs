//! The key of the tamper checks, on a restore (PRD FR-20.11, NFR-SEC-6, plan 56).
//!
//! The journal and audit hash chains are keyed hashes. The key lives in the Credential Manager
//! of the computer that wrote the rows, not in the data file. A copy on USB or in the cloud
//! carries it inside the encrypted bundle (locked by the backup key). A restore therefore does
//! three things with it:
//!
//! 1. It verifies BOTH chains of the unpacked copy before anything is installed. The key in the
//!    copy is tried first, then the key this computer already has (a plain copy, or a bundle made
//!    before the key was added, can only be checked with the latter).
//! 2. If neither key makes the chains verify, the copy is refused. A copy whose chains cannot be
//!    checked would show as tampered with from its first row, and rows already posted can never
//!    be signed again (Code Standards B17).
//! 3. If the key of the copy is not the key of this computer, it is saved here, in the key store
//!    and in the auditor, so the rows written after the restore continue the same chains.
//!
//! No key is ever logged or written to a file.

use std::path::Path;

use mandi_core::error::ErrorValue;
use mandi_db::audit::{AuditKey, VerifyError, verify_audit_chain};
use mandi_db::ledger::{JournalVerifyError, verify_journal_chain};
use mandi_db::rusqlite::{Connection, OpenFlags};

use crate::error::{BackupCode, BackupError};
use crate::service::BackupService;

/// The value `check` of `ERR_BACKUP_CORRUPT` when the chains do not verify.
pub const CHAINS_CHECK: &str = "chains";

/// True when both chains of the file verify with `key`.
///
/// # Errors
/// `ERR_BACKUP_CORRUPT` when the file cannot be read.
pub(crate) fn chains_verify(plain: &Path, key: &AuditKey) -> Result<bool, BackupError> {
    let unreadable = |detail: &dyn std::fmt::Display| {
        BackupError::new(BackupCode::ErrBackupCorrupt, format!("chains: {detail}"))
    };
    let conn = Connection::open_with_flags(
        plain,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| unreadable(&e))?;
    match verify_audit_chain(&conn, key) {
        Ok(()) => {}
        Err(VerifyError::Broken(_)) => return Ok(false),
        Err(VerifyError::Db(e)) => return Err(unreadable(&e)),
    }
    match verify_journal_chain(&conn, key) {
        Ok(()) => Ok(true),
        Err(JournalVerifyError::Broken(_)) => Ok(false),
        Err(JournalVerifyError::Db(e)) => Err(unreadable(&e)),
    }
}

impl BackupService {
    /// The key that makes the chains of the unpacked copy `plain` verify (see the module text).
    /// `copy_key` is the hex the bundle carried, if it carried one. `field` is the input field
    /// the error points at.
    ///
    /// # Errors
    /// `ERR_BACKUP_CORRUPT` (value `check` = `chains`) when the copy has a key and its chains do
    /// not verify with it or with this computer's key; `ERR_BACKUP_CHAIN_KEY_MISSING` when the
    /// copy has no key and this computer's key does not make them verify.
    pub(crate) fn pick_chain_key(
        &self,
        plain: &Path,
        copy_key: Option<&str>,
        field: &str,
    ) -> Result<AuditKey, BackupError> {
        let from_copy = match copy_key {
            Some(hex) => Some(AuditKey::from_hex(hex).map_err(|_| {
                BackupError::new(
                    BackupCode::ErrBackupCorrupt,
                    "the key in the copy is not valid",
                )
                .on_field(field)
                .with("check", ErrorValue::Text(CHAINS_CHECK.to_owned()))
            })?),
            None => None,
        };
        if let Some(key) = &from_copy
            && chains_verify(plain, key)?
        {
            return Ok(key.clone());
        }
        let local = self.db.auditor().key()?;
        let same = from_copy.as_ref().is_some_and(|k| k.same_as(&local));
        if !same && chains_verify(plain, &local)? {
            return Ok(local);
        }
        Err(if from_copy.is_some() {
            BackupError::new(
                BackupCode::ErrBackupCorrupt,
                "the journal or audit chain of the copy does not verify",
            )
            .on_field(field)
            .with("check", ErrorValue::Text(CHAINS_CHECK.to_owned()))
        } else {
            BackupError::new(
                BackupCode::ErrBackupChainKeyMissing,
                "the copy has no chain key and this computer's key does not fit",
            )
            .on_field(field)
        })
    }

    /// Makes `key` the key of this computer if it is not already. Returns the key that was
    /// there, to put back if the restore fails; null when nothing changed.
    ///
    /// # Errors
    /// `ERR_INTERNAL` when the key store cannot be written. Nothing changed then.
    pub(crate) fn install_chain_key(
        &self,
        key: &AuditKey,
    ) -> Result<Option<AuditKey>, BackupError> {
        let current = self.db.auditor().key()?;
        if current.same_as(key) {
            return Ok(None);
        }
        self.db.auditor().install_key(key.clone())?;
        Ok(Some(current))
    }

    /// Puts back the key that [`BackupService::install_chain_key`] replaced. A failure here is
    /// logged: the restore it belongs to has failed already.
    pub(crate) fn put_chain_key_back(&self, previous: Option<AuditKey>) {
        if let Some(old) = previous
            && let Err(error) = self.db.auditor().install_key(old)
        {
            tracing::error!("cannot put the old chain key back: {error}");
        }
    }
}
