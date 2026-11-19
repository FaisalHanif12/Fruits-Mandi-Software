//! The shapes shared inside the crate and with the screens: the kind of a copy, the totals read
//! from a copy, and the manifests that describe a copy (Architecture 14, NFR-BAK-5).

use std::collections::BTreeMap;

use mandi_core::money::Paisa;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Where a copy is kept (Architecture 15.3 `backup_log.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum BackupKind {
    /// The local folder: a plain copy, every 15 minutes (FR-20.2).
    Local,
    /// A USB drive: an encrypted copy (FR-20.4).
    Usb,
    /// The cloud: an encrypted copy (FR-20.3).
    Cloud,
}

impl BackupKind {
    /// The word stored in `backup_log.kind`.
    pub fn as_str(self) -> &'static str {
        match self {
            BackupKind::Local => "local",
            BackupKind::Usb => "usb",
            BackupKind::Cloud => "cloud",
        }
    }
}

/// What a copy holds, read from the copy itself. The screens show it before a restore
/// (FR-20.9) and after a test restore (NFR-BAK-11). Money comes only from `SUM()` on the
/// `*_paisa` columns (CLAUDE.md "Gotchas").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct BackupTotals {
    /// Accounts.
    #[ts(type = "number")]
    pub accounts: i64,
    /// Saved and cancelled بکری.
    #[ts(type = "number")]
    pub bikri_entries: i64,
    /// Vouchers.
    #[ts(type = "number")]
    pub vouchers: i64,
    /// Posted journal entries.
    #[ts(type = "number")]
    pub journal_entries: i64,
    /// Rows of the audit log.
    #[ts(type = "number")]
    pub audit_rows: i64,
    /// The date of the newest posted entry, `YYYY-MM-DD`. Null when nothing is posted.
    pub last_entry_date: Option<String>,
    /// Σ بنام of all posted lines.
    #[ts(type = "Paisa")]
    pub debit_paisa: Paisa,
    /// Σ جمع of all posted lines. Equal to `debit_paisa` in a healthy book (PRD FR-6.2).
    #[ts(type = "Paisa")]
    pub credit_paisa: Paisa,
    /// Cash in hand (روکڑ, 01010001): its بنام less its جمع. Can be below zero (FR-6.7).
    #[ts(type = "Paisa")]
    pub cash_paisa: Paisa,
}

/// What a decrypted bundle or a plain copy says about itself (kept inside the encrypted bundle,
/// so a cloud sidecar never shows money).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerManifest {
    /// Checksum of the plain SQLite file.
    pub plain_blake3: String,
    /// Size of the plain SQLite file.
    pub plain_size: u64,
    /// Rows of every table, by table name.
    pub counts: BTreeMap<String, i64>,
    /// The totals.
    pub totals: BackupTotals,
}

/// The manifest inside an encrypted bundle, in front of the database bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Embedded {
    /// Format number of this manifest.
    pub format: u32,
    /// When the snapshot was taken (UTC, storage form).
    pub created_at: String,
    /// The app version that made it.
    pub app_version: String,
    /// The schema version (`PRAGMA user_version`) of the data.
    pub schema_version: i64,
    /// Checksums, counts and totals.
    pub inner: InnerManifest,
    /// The key of the journal and audit hash chains (NFR-SEC-6) as 64 hex characters. It lives
    /// ONLY inside the encrypted bundle, locked by the backup key, so a restore on a new
    /// computer can verify the chains (PRD FR-20.11). Never in the `.json` file next to a copy,
    /// never in a plain local copy, never in a log. Copies made before this field existed have
    /// none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_key: Option<SecretText>,
}

/// A secret kept as text inside the encrypted bundle. Its `Debug` text never shows it
/// (NFR-OBS-2), so a log line of a manifest cannot leak it.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretText(String);

impl SecretText {
    /// Wraps a secret.
    pub fn new(text: String) -> Self {
        Self(text)
    }

    /// The secret. Use it only to hand it to the key code, never to print it.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[hidden]")
    }
}

/// The `.json` file kept next to every copy. For an encrypted copy it holds no money and no
/// counts (NFR-BAK-6): only what the list and the integrity check need.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Format number of this manifest.
    pub format: u32,
    /// Where the copy is kept.
    pub kind: BackupKind,
    /// When the snapshot was taken (UTC, storage form).
    pub created_at: String,
    /// The app version that made it.
    pub app_version: String,
    /// The schema version of the data.
    pub schema_version: i64,
    /// True for a `.mbk` bundle, false for a plain `.db` copy.
    pub encrypted: bool,
    /// File name of the copy.
    pub file: String,
    /// Size of the copy.
    pub size_bytes: u64,
    /// Checksum of the copy (the bundle, or the plain file).
    pub file_blake3: String,
    /// True when the copy passed its check right after it was made (FR-20.7).
    pub integrity_ok: bool,
    /// Counts and totals. Only for a plain copy.
    pub inner: Option<InnerManifest>,
}

/// The format number written to every manifest.
pub const MANIFEST_FORMAT: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    fn embedded(chain_key: Option<&str>) -> Embedded {
        Embedded {
            format: MANIFEST_FORMAT,
            created_at: "2026-10-10T09:00:00.000Z".into(),
            app_version: "test".into(),
            schema_version: 18,
            inner: InnerManifest {
                plain_blake3: "ab".into(),
                plain_size: 1,
                counts: BTreeMap::new(),
                totals: BackupTotals {
                    accounts: 0,
                    bikri_entries: 0,
                    vouchers: 0,
                    journal_entries: 0,
                    audit_rows: 0,
                    last_entry_date: None,
                    debit_paisa: Paisa::ZERO,
                    credit_paisa: Paisa::ZERO,
                    cash_paisa: Paisa::ZERO,
                },
            },
            chain_key: chain_key.map(|k| SecretText::new(k.to_owned())),
        }
    }

    #[test]
    fn the_chain_key_is_written_only_when_there_is_one_and_old_bundles_still_read() {
        let with = serde_json::to_string(&embedded(Some("0a0b"))).unwrap();
        assert!(with.contains("\"chain_key\":\"0a0b\""));
        let without = serde_json::to_string(&embedded(None)).unwrap();
        assert!(!without.contains("chain_key"), "{without}");
        // A bundle made before the field existed has no such key and still reads.
        let old: Embedded = serde_json::from_str(&without).unwrap();
        assert_eq!(old.chain_key, None);
        let new: Embedded = serde_json::from_str(&with).unwrap();
        assert_eq!(new.chain_key.unwrap().expose(), "0a0b");
    }

    #[test]
    fn the_debug_text_of_a_manifest_never_shows_the_chain_key() {
        let shown = format!("{:?}", embedded(Some("deadbeefcafe")));
        assert!(!shown.contains("deadbeefcafe"), "{shown}");
        assert!(shown.contains("[hidden]"));
    }
}
