//! The types the backup commands send and receive (plan 18 contract). They are exported to
//! `frontend/src/bindings` with ts-rs. Money is `Paisa`; times are UTC text in the storage form
//! `2026-10-10T07:30:12.345Z`; the screen shows them in Pakistan time.

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{BackupCode, CloudReason};
use crate::model::{BackupKind, BackupTotals};
use crate::settings::BackupSettings;

/// Page size of `backup_list` when the screen sends none (Code Standards B31).
pub const DEFAULT_PAGE: i64 = 100;

/// One copy, named the way the list names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct BackupRef {
    /// Where it is kept.
    pub kind: BackupKind,
    /// The file name from `backup_list`, for example `mandi-20261010T160000000Z.mbk`.
    pub name: String,
}

/// How old the last good cloud copy is (NFR-BAK-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum CloudLevel {
    /// The cloud copy is off.
    Off,
    /// A good cloud copy is under 24 hours old.
    Green,
    /// The last good cloud copy is over 24 hours old, or there is none: red in the status bar.
    Red,
    /// Over 48 hours: red, and an alert on the dashboard.
    Alert,
}

/// A problem to show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatusError {
    /// Where the copy was meant to go.
    pub kind: BackupKind,
    /// The code for the message.
    pub code: BackupCode,
    /// When (UTC).
    pub at: String,
    /// True when the copy was made but failed its check right after (FR-20.7): a dashboard alert.
    pub check_failed: bool,
    /// The file that failed its check.
    pub file: Option<String>,
    /// For a failed cloud copy: why (offline, refused, not found), so the screen picks the
    /// words. Null for any other problem.
    pub reason: Option<CloudReason>,
}

/// The state of the backups, for the status bar, the dashboard and the backup screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BackupStatus {
    /// True when the backup key exists on this computer.
    pub key_set: bool,
    /// True when the Owner typed the printed key to prove he has it (NFR-BAK-7).
    pub key_confirmed: bool,
    /// True when the key was made over 7 days ago and is still not confirmed.
    pub key_confirm_due: bool,
    /// When the last good local copy was made (UTC), if ever.
    pub last_local_at: Option<String>,
    /// When the last good USB copy was made (UTC), if ever.
    pub last_usb_at: Option<String>,
    /// When the last good cloud copy was made (UTC), if ever.
    pub last_cloud_at: Option<String>,
    /// Whole hours since the last good cloud copy. Null when there is none.
    #[ts(type = "number | null")]
    pub cloud_age_hours: Option<i64>,
    /// The colour state of the cloud copy (NFR-BAK-10).
    pub cloud_level: CloudLevel,
    /// Cloud copies waiting for the internet.
    #[ts(type = "number")]
    pub queue_len: i64,
    /// True when the USB copy is switched on.
    pub usb_enabled: bool,
    /// True when the USB folder (or its drive) is there now.
    pub usb_present: bool,
    /// True when the cloud copy is switched on.
    pub cloud_enabled: bool,
    /// True when the cloud account details are all there (settings and keys).
    pub cloud_ready: bool,
    /// True while a backup or restore runs.
    pub running: bool,
    /// The latest problems, newest last.
    pub errors: Vec<StatusError>,
    /// When the last test restore ran (UTC).
    pub last_test_restore_at: Option<String>,
    /// True when a cloud copy exists and no good test restore ran in the last 30 days (NFR-BAK-11).
    pub test_restore_due: bool,
    /// True while the daily readable pack (روزنامچہ PDF, spreadsheets) is not built (FR-20.5):
    /// the PDF waits for the report engine, the spreadsheets for D5-Q1.
    pub pack_pending: bool,
}

/// Input of `backup_list`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
pub struct BackupListInput {
    /// Only this place. Null for all.
    pub kind: Option<BackupKind>,
    /// Rows to skip.
    #[ts(type = "number")]
    pub offset: i64,
    /// Rows to return. 0 or less means 100.
    #[ts(type = "number")]
    pub limit: i64,
}

/// The result of a copy, as the list knows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum RowStatus {
    /// Made and checked.
    Good,
    /// Made but failed its check.
    Bad,
    /// Found, but nothing says it was checked (no manifest next to it).
    Unchecked,
}

/// One copy in the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BackupRow {
    /// Where it is kept.
    pub kind: BackupKind,
    /// The file name. Send it back in a `BackupRef`.
    pub name: String,
    /// When it was made (UTC).
    pub created_at: String,
    /// Size in bytes.
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// True for a copy that needs the key (USB and cloud).
    pub encrypted: bool,
    /// The result of its check.
    pub status: RowStatus,
}

/// A page of the list, newest first, with the number of all rows (Code Standards B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BackupList {
    /// The rows of this page.
    pub rows: Vec<BackupRow>,
    /// Rows in all pages.
    #[ts(type = "number")]
    pub total_count: i64,
    /// False when the cloud could not be asked (no internet): its rows are missing.
    pub cloud_reachable: bool,
}

/// The result of a copy to one place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TargetResult {
    /// The place.
    pub kind: BackupKind,
    /// True when the copy is made and checked.
    pub ok: bool,
    /// The file name, when one was made.
    pub file: Option<String>,
    /// Its size.
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// True when the copy waits in the queue for the internet (cloud only).
    pub queued: bool,
    /// Why it failed.
    pub code: Option<BackupCode>,
    /// For a failed cloud copy: why (offline, refused, not found). Null for any other result.
    pub reason: Option<CloudReason>,
}

/// What a backup run did (FR-20.8 "backup now").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RunReport {
    /// When the run began (UTC).
    pub started_at: String,
    /// One result for each place tried.
    pub results: Vec<TargetResult>,
    /// Milliseconds the snapshot took (NFR-SPD-15).
    #[ts(type = "number")]
    pub snapshot_ms: u64,
    /// Milliseconds the check took. It runs after the snapshot is safe.
    #[ts(type = "number")]
    pub check_ms: u64,
}

/// Input of `backup_restore_preview`.
#[derive(Clone, Deserialize, TS)]
pub struct RestorePreviewInput {
    /// The copy.
    pub backup: BackupRef,
    /// The printed key. Needed for a USB or cloud copy when this computer has no key.
    pub key: Option<String>,
}

impl fmt::Debug for RestorePreviewInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RestorePreviewInput")
            .field("backup", &self.backup)
            .field("key", &"[hidden]")
            .finish()
    }
}

/// What a restore would lose: records the live data has and the copy does not (FR-20.9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LostWork {
    /// Posted journal entries newer than the copy.
    #[ts(type = "number")]
    pub journal_entries: i64,
    /// بکری newer than the copy.
    #[ts(type = "number")]
    pub bikri_entries: i64,
    /// Vouchers newer than the copy.
    #[ts(type = "number")]
    pub vouchers: i64,
}

/// What the Owner sees before he confirms a restore (FR-20.9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RestorePreview {
    /// The copy.
    pub backup: BackupRef,
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
    /// True when the copy is older than this app and is upgraded after the restore.
    pub needs_upgrade: bool,
    /// The books in the copy.
    pub totals: BackupTotals,
    /// The books now.
    pub current: BackupTotals,
    /// What the restore would lose.
    pub lost: LostWork,
}

/// Input of `backup_restore`.
#[derive(Clone, Deserialize, TS)]
pub struct RestoreInput {
    /// The copy.
    pub backup: BackupRef,
    /// The printed key. Needed for a USB or cloud copy when this computer has no key.
    pub key: Option<String>,
    /// Must be true: the Owner pressed the confirm button (FR-20.9).
    pub confirmed: bool,
}

impl fmt::Debug for RestoreInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RestoreInput")
            .field("backup", &self.backup)
            .field("key", &"[hidden]")
            .field("confirmed", &self.confirmed)
            .finish()
    }
}

/// What a restore did. The session is closed: the Owner signs in again with the restored data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RestoreDone {
    /// When the restored copy was made (UTC).
    pub restored_from: String,
    /// The safety copy of the data as it was just before (kept in the safety folder).
    pub safety_copy: String,
    /// The books after the restore, read from the data file now.
    pub totals: BackupTotals,
    /// The schema version the copy had, when it was upgraded.
    #[ts(type = "number | null")]
    pub upgraded_from: Option<i64>,
}

/// Input of `backup_test_restore`.
#[derive(Clone, Deserialize, TS)]
pub struct TestRestoreInput {
    /// The copy to test. Null means the newest cloud copy, else the newest USB copy, else the
    /// newest local copy (NFR-BAK-11).
    pub backup: Option<BackupRef>,
    /// The printed key, when this computer has none.
    pub key: Option<String>,
}

impl fmt::Debug for TestRestoreInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestRestoreInput")
            .field("backup", &self.backup)
            .field("key", &"[hidden]")
            .finish()
    }
}

/// One check of a test restore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TestCheckName {
    /// The copy could be fetched and opened with the key.
    Opens,
    /// The file's checksum is the one recorded when it was made.
    Checksum,
    /// The database inside passes the full integrity check.
    Integrity,
    /// The rows of every table are the numbers recorded when it was made.
    Counts,
    /// بنام and جمع of the whole book are equal.
    Balanced,
}

/// A line of the result of a test restore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct TestCheck {
    /// Which check.
    pub name: TestCheckName,
    /// True when it passed.
    pub ok: bool,
}

/// The result of a test restore into a separate place (NFR-BAK-11). The live data is not touched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TestRestoreResult {
    /// The copy that was tested.
    pub backup: BackupRef,
    /// When it was made (UTC).
    pub created_at: String,
    /// True when every check passed.
    pub ok: bool,
    /// The checks, in order.
    pub checks: Vec<TestCheck>,
    /// The books in the restored test copy.
    pub restored: BackupTotals,
    /// The books now, to compare.
    pub live: BackupTotals,
}

/// The backup key, shown once (FR-20.10).
#[derive(Clone, PartialEq, Eq, Serialize, TS)]
pub struct KeyCreated {
    /// `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`. Never shown again.
    pub key: String,
    /// When it was made (UTC).
    pub created_at: String,
}

impl fmt::Debug for KeyCreated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyCreated")
            .field("key", &"[hidden]")
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// Input of `backup_key_confirm`.
#[derive(Clone, Deserialize, TS)]
pub struct KeyConfirmInput {
    /// The key as the Owner typed it from the printed sheet.
    pub key: String,
}

impl fmt::Debug for KeyConfirmInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyConfirmInput")
            .field("key", &"[hidden]")
            .finish()
    }
}

/// The settings as the screen shows them: the plain settings and whether the cloud keys exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BackupSettingsView {
    /// The settings that are not secret.
    pub settings: BackupSettings,
    /// True when the cloud access key id is stored.
    pub cloud_access_key_set: bool,
    /// True when the cloud secret access key is stored.
    pub cloud_secret_set: bool,
    /// The folder the local copies go to now (the default when `local_dir` is empty).
    pub local_dir_effective: String,
}

/// Input of `backup_settings_set`.
#[derive(Clone, Deserialize, TS)]
pub struct BackupSettingsInput {
    /// The settings that are not secret.
    pub settings: BackupSettings,
    /// A new cloud access key id. Null keeps the stored one. An empty text removes it.
    pub cloud_access_key_id: Option<String>,
    /// A new cloud secret access key. Null keeps the stored one. An empty text removes it.
    pub cloud_secret_access_key: Option<String>,
}

impl fmt::Debug for BackupSettingsInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BackupSettingsInput")
            .field("settings", &self.settings)
            .field("cloud_access_key_id", &"[hidden]")
            .field("cloud_secret_access_key", &"[hidden]")
            .finish()
    }
}

/// Input of `backup_now`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
pub struct BackupNowInput {
    /// Make the copy to the local folder.
    pub local: bool,
    /// Make the copy to the USB drive, if it is on and plugged in.
    pub usb: bool,
    /// Make the copy to the cloud, if it is on.
    pub cloud: bool,
}
