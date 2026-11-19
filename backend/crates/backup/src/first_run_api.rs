//! The types of "restore on a new computer, from the first screen" (PRD FR-20.11, plan 56).
//!
//! The three commands run before sign-in, and only while no Owner exists. They keep nothing
//! between calls, so every call that needs a secret carries it. A secret never reaches a log:
//! the `Debug` text of every input hides the keys.

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::api::BackupRow;
use crate::model::BackupTotals;

/// The cloud account as the Owner types it on the first screen. The same fields as Settings.
#[derive(Clone, Deserialize, TS)]
pub struct CloudAccountInput {
    /// The S3 address, for example `https://<account>.r2.cloudflarestorage.com`. Must be https.
    pub endpoint: String,
    /// The bucket.
    pub bucket: String,
    /// The region. Cloudflare R2 uses `auto`.
    pub region: String,
    /// The folder inside the bucket, for example `mandi`.
    pub prefix: String,
    /// The access key id.
    pub key_id: String,
    /// The secret access key.
    pub secret: String,
}

impl fmt::Debug for CloudAccountInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CloudAccountInput")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("region", &self.region)
            .field("prefix", &self.prefix)
            .field("key_id", &"[hidden]")
            .field("secret", &"[hidden]")
            .finish()
    }
}

/// Where the copies are.
#[derive(Clone, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RestoreSource {
    /// The cloud account.
    Cloud(CloudAccountInput),
    /// A folder, for example a USB drive: its `mandi-*.mbk` files are the copies. The full path.
    Folder(String),
    /// One `.mbk` file. The full path.
    File(String),
}

impl fmt::Debug for RestoreSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestoreSource::Cloud(account) => f.debug_tuple("Cloud").field(account).finish(),
            RestoreSource::Folder(path) => f.debug_tuple("Folder").field(path).finish(),
            RestoreSource::File(path) => f.debug_tuple("File").field(path).finish(),
        }
    }
}

/// Input of `backup_first_run_list`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct FirstRunListInput {
    /// Where to look.
    pub source: RestoreSource,
}

/// The copies found, newest first. An empty list is not an error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FirstRunCopies {
    /// The copies. A cloud copy has the kind `cloud`; one from a folder or file has the kind `usb`.
    pub rows: Vec<BackupRow>,
    /// The same number as `rows.len()`.
    #[ts(type = "number")]
    pub total_count: i64,
}

/// Input of `backup_first_run_preview`.
#[derive(Clone, Deserialize, TS)]
pub struct FirstRunPreviewInput {
    /// Where the copies are.
    pub source: RestoreSource,
    /// The file name from the list. Null means the newest copy that is not marked bad. For a
    /// file source the file itself is the copy and this is ignored.
    pub copy: Option<String>,
    /// The backup key from the printed sheet, with or without dashes.
    pub key: String,
}

impl fmt::Debug for FirstRunPreviewInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FirstRunPreviewInput")
            .field("source", &self.source)
            .field("copy", &self.copy)
            .field("key", &"[hidden]")
            .finish()
    }
}

/// Input of `backup_first_run_restore`.
#[derive(Clone, Deserialize, TS)]
pub struct FirstRunRestoreInput {
    /// Where the copies are.
    pub source: RestoreSource,
    /// The file name from the preview. For a file source the file itself is the copy and this is
    /// ignored.
    pub copy: String,
    /// The backup key from the printed sheet.
    pub key: String,
}

impl fmt::Debug for FirstRunRestoreInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FirstRunRestoreInput")
            .field("source", &self.source)
            .field("copy", &self.copy)
            .field("key", &"[hidden]")
            .finish()
    }
}

/// One check of a copy before it is restored. All of them must pass (NFR-BAK-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FirstRunCheckName {
    /// The copy was fetched and opened with the key.
    Opens,
    /// The file's checksum is the one recorded when it was made.
    Checksum,
    /// The database inside passes the full integrity check.
    Integrity,
    /// The rows of every table are the numbers recorded when it was made.
    Counts,
    /// بنام and جمع of the whole book are equal.
    Balanced,
    /// The journal and audit hash chains verify (NFR-SEC-6).
    Chains,
}

/// A line of the checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct FirstRunCheck {
    /// Which check.
    pub name: FirstRunCheckName,
    /// True when it passed. A copy with a false check is refused, so a preview that comes back
    /// has all of them true.
    pub ok: bool,
}

/// What the Owner sees before he restores: the copy, its date and its books.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FirstRunPreview {
    /// The copy chosen. Send its `name` to the restore.
    pub copy: crate::api::BackupRef,
    /// When the copy was made (UTC).
    pub created_at: String,
    /// Size of the copy file.
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// The schema version of the copy.
    #[ts(type = "number")]
    pub schema_version: i64,
    /// The schema version of this app.
    #[ts(type = "number")]
    pub app_schema_version: i64,
    /// True when the copy is older than this app and is upgraded while it is put in place.
    pub needs_upgrade: bool,
    /// The books in the copy (after the upgrade, when there is one).
    pub totals: BackupTotals,
    /// The checks, in order.
    pub checks: Vec<FirstRunCheck>,
}

/// What a restore on a new computer did. The Owner is not signed in: he signs in next, with his
/// old user name and password.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FirstRunRestoreDone {
    /// When the restored copy was made (UTC).
    pub restored_from: String,
    /// The books after the restore, read from the data file now.
    pub totals: BackupTotals,
    /// The schema version the copy had, when it was upgraded.
    #[ts(type = "number | null")]
    pub upgraded_from: Option<i64>,
    /// True when the key of the tamper checks came from the copy and was saved on this computer.
    pub chain_key_restored: bool,
    /// True when the copy has the cloud copy switched on and this computer has no cloud keys:
    /// the Owner must enter them in Settings (a file or folder source brings none).
    pub cloud_needs_keys: bool,
    /// True when the assistant was switched on in the copy. Its key never travels: the screen
    /// asks for it again after sign-in.
    pub assistant_was_on: bool,
}

/// The step a restore is at. The screen shows a line for each (plan 56).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RestoreStage {
    /// The copy is coming in. `percent` is set.
    Downloading,
    /// The copy is being unlocked with the key.
    Decrypting,
    /// The database inside is being checked.
    Checking,
    /// The copy is older than the app and is being upgraded.
    Upgrading,
    /// The keys are being saved on this computer.
    Keys,
    /// The data is being put in place.
    PuttingInPlace,
    /// The search index is being built.
    SearchIndex,
    /// Finished.
    Done,
}

/// The payload of the event `backup-restore-progress`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct RestoreProgress {
    /// The step.
    pub stage: RestoreStage,
    /// 0 to 100 where the step can tell how far it is; null otherwise.
    #[ts(type = "number | null")]
    pub percent: Option<u8>,
}
