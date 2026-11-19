//! What the backup remembers about itself, in a small JSON file next to the backups (not in the
//! data file, so a restore of an older data file does not make the app forget what it has
//! backed up). Written through a `.partial` file and a rename.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{BackupCode, BackupError, CloudReason};
use crate::io::write_atomic;
use crate::model::BackupKind;

/// The file name inside the app's backups folder.
pub const STATE_FILE: &str = "backup_state.json";
/// The most failures remembered for the screen.
const MAX_FAILURES: usize = 20;

/// A copy that was made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stamp {
    /// When (UTC, storage form).
    pub at: String,
    /// File name of the copy.
    pub file: String,
    /// Size in bytes.
    pub size_bytes: u64,
}

/// A cloud copy that is waiting for the internet (NFR-BAK-9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Queued {
    /// File name of the bundle, in the outbox folder.
    pub file: String,
    /// When it was queued (UTC).
    pub queued_at: String,
    /// Tries so far.
    pub attempts: u32,
    /// The reason of the last failure: `offline` or `refused`.
    pub last_reason: String,
}

/// A problem the screen shows (FR-20.7, NFR-BAK-10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    /// Where the copy was meant to go.
    pub kind: BackupKind,
    /// The code for the message.
    pub code: BackupCode,
    /// When (UTC).
    pub at: String,
    /// True when the copy was made but failed its check (a dashboard alert).
    pub check_failed: bool,
    /// The file that failed its check, if any.
    pub file: Option<String>,
    /// For a failed cloud copy: why. A state file written before this field existed has none.
    #[serde(default)]
    pub reason: Option<CloudReason>,
}

/// Everything remembered.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    /// The last good local copy.
    pub last_local: Option<Stamp>,
    /// The last good USB copy.
    pub last_usb: Option<Stamp>,
    /// The last good cloud copy (uploaded and checked).
    pub last_cloud: Option<Stamp>,
    /// When the last local copy was tried, good or not (UTC).
    pub last_local_try: Option<String>,
    /// When the queue was last tried (UTC).
    pub last_queue_try: Option<String>,
    /// The local date (`YYYY-MM-DD`) of the last daily USB and cloud run.
    pub last_daily_day: Option<String>,
    /// When the daily run was last started (UTC), to space out tries after a failure.
    pub last_daily_try: Option<String>,
    /// When the key was created (UTC).
    pub key_created_at: Option<String>,
    /// When the Owner typed the printed key to confirm it (UTC).
    pub key_confirmed_at: Option<String>,
    /// When a test restore last ran (UTC).
    pub last_test_restore_at: Option<String>,
    /// Whether that test restore was good.
    pub last_test_restore_ok: Option<bool>,
    /// Cloud copies waiting for the internet, oldest first.
    pub queue: Vec<Queued>,
    /// The latest problems, newest last. A good copy of a kind clears that kind's problems.
    pub failures: Vec<Failure>,
    /// The date of the last readable pack that reached the cloud.
    pub last_pack_day: Option<String>,
}

impl State {
    /// Remembers a problem; the list stays short. A new problem replaces an older one of the same
    /// place and code, so a queue that fails every 10 minutes shows one line, not hundreds.
    pub fn add_failure(&mut self, failure: Failure) {
        self.failures.retain(|f| {
            !(f.kind == failure.kind
                && f.code == failure.code
                && f.check_failed == failure.check_failed)
        });
        self.failures.push(failure);
        if self.failures.len() > MAX_FAILURES {
            let extra = self.failures.len() - MAX_FAILURES;
            self.failures.drain(..extra);
        }
    }

    /// A good copy of `kind` clears the problems of that kind.
    pub fn clear_failures(&mut self, kind: BackupKind) {
        self.failures.retain(|f| f.kind != kind);
    }
}

/// The path of the state file in the backups folder.
pub fn state_path(backups_dir: &Path) -> PathBuf {
    backups_dir.join(STATE_FILE)
}

/// Reads the state. A missing file is a new state. A file that cannot be read is also a new
/// state (and a log line): the app must keep working.
pub fn load(backups_dir: &Path) -> State {
    let path = state_path(backups_dir);
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            tracing::warn!(path = %path.display(), "backup state is unreadable, starting new: {error}");
            State::default()
        }),
        Err(_) => State::default(),
    }
}

/// Saves the state.
///
/// # Errors
/// `ERR_INTERNAL` when the file cannot be written.
pub fn save(backups_dir: &Path, state: &State) -> Result<(), BackupError> {
    let bytes = serde_json::to_vec_pretty(state)?;
    write_atomic(&state_path(backups_dir), &bytes).map_err(|e| BackupError::io("state", &e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trips_and_a_missing_or_bad_file_is_new() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()), State::default());
        let mut state = State {
            last_cloud: Some(Stamp {
                at: "2026-10-10T16:00:00.000Z".into(),
                file: "mandi-x.mbk".into(),
                size_bytes: 5,
            }),
            ..State::default()
        };
        state.queue.push(Queued {
            file: "mandi-y.mbk".into(),
            queued_at: "2026-10-10T16:00:00.000Z".into(),
            attempts: 2,
            last_reason: "offline".into(),
        });
        save(dir.path(), &state).unwrap();
        assert_eq!(load(dir.path()), state);
        std::fs::write(state_path(dir.path()), b"{ not json").unwrap();
        assert_eq!(load(dir.path()), State::default());
    }

    #[test]
    fn failures_are_deduplicated_and_cleared_by_kind() {
        let mut state = State::default();
        for i in 0..30 {
            state.add_failure(Failure {
                kind: if i % 2 == 0 {
                    BackupKind::Usb
                } else {
                    BackupKind::Cloud
                },
                code: BackupCode::ErrBackupCloudFailed,
                at: format!("t{i}"),
                check_failed: false,
                file: None,
                reason: None,
            });
        }
        // Same place and code twice is one line.
        assert_eq!(state.failures.len(), 2);
        state.clear_failures(BackupKind::Usb);
        assert!(state.failures.iter().all(|f| f.kind == BackupKind::Cloud));
    }
}
