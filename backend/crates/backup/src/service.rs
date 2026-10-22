//! The backup service: the one object the app and the tests talk to (plan 18).
//!
//! It holds the open data file, the folders, the secrets, the cloud and the remembered state.
//! This file has the parts that read: status, list, settings and the key. `run.rs` makes copies,
//! `restore.rs` restores them, `schedule.rs` decides what is due.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, TryLockError};

use chrono::{DateTime, Duration, Utc};
use mandi_db::Db;
use mandi_db::audit::AuditEntry;
use mandi_db::clock::format_utc;
use mandi_db::disk::{DiskProbe, RealDiskProbe};
use serde_json::json;

use crate::api::{
    BackupList, BackupListInput, BackupRow, BackupSettingsInput, BackupSettingsView, BackupStatus,
    CloudLevel, DEFAULT_PAGE, KeyCreated, RowStatus, StatusError,
};
use crate::cloud::{CloudFactory, S3Factory};
use crate::daily_pack::{PackSource, PendingPack};
use crate::error::{BackupCode, BackupError};
use crate::io::{IoHook, NoHook};
use crate::key::BackupKey;
use crate::model::{BackupKind, Manifest};
use crate::names::{is_copy_name, manifest_name, parse_stamp};
use crate::secrets::{KeyringSecretStore, SecretName, SecretStore};
use crate::settings::{self, BackupSettings};
use crate::state::{self, State};

/// The key is confirmed within this many days, or the screen reminds the Owner (NFR-BAK-7).
pub const KEY_CONFIRM_DAYS: i64 = 7;
/// A cloud copy older than this many hours turns the status bar red (NFR-BAK-10).
pub const CLOUD_RED_HOURS: i64 = 24;
/// A cloud copy older than this many hours raises a dashboard alert (NFR-BAK-10).
pub const CLOUD_ALERT_HOURS: i64 = 48;
/// A test restore is due again after this many days (NFR-BAK-11).
pub const TEST_RESTORE_DAYS: i64 = 30;
/// Folder of the cloud objects inside the bucket prefix.
pub const CLOUD_DIR: &str = "backups/";

/// Where the backup keeps its own files.
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// The app's backups folder (`<app data>/backups`): state, scratch files, the outbox of
    /// cloud copies waiting for the internet, the safety copies, and the default local folder.
    pub backups_dir: PathBuf,
    /// The app version, written into each manifest.
    pub app_version: String,
}

/// The parts that tests replace.
#[derive(Clone)]
pub struct BackupParts {
    /// Where the key and the cloud keys live.
    pub secrets: Arc<dyn SecretStore>,
    /// Makes the cloud store.
    pub cloud: Arc<dyn CloudFactory>,
    /// Free disk space.
    pub disk: Arc<dyn DiskProbe>,
    /// Breaks writes in fault tests.
    pub hook: Arc<dyn IoHook>,
    /// Makes the readable daily pack.
    pub pack: Arc<dyn PackSource>,
}

impl BackupParts {
    /// The real parts: the Credential Manager, S3, the real disk, no faults, and no pack yet.
    pub fn real() -> Self {
        Self {
            secrets: Arc::new(KeyringSecretStore),
            cloud: Arc::new(S3Factory),
            disk: Arc::new(RealDiskProbe),
            hook: Arc::new(NoHook),
            pack: Arc::new(PendingPack),
        }
    }
}

/// The backup service. Cheap to share: wrap it in an `Arc`.
pub struct BackupService {
    pub(crate) db: Arc<Db>,
    pub(crate) config: BackupConfig,
    pub(crate) parts: BackupParts,
    pub(crate) state: Mutex<State>,
    /// Held while a backup or restore runs, so two never run together.
    pub(crate) busy: Mutex<()>,
    /// The data version seen at the last copy, to tell that the data changed.
    pub(crate) data_seen: Mutex<Option<i64>>,
    /// Whether the USB folder was there at the last look.
    pub(crate) usb_was_present: Mutex<bool>,
}

/// Locks a mutex; a panic in another thread does not make the data unusable.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// True when the USB folder, or the drive it is on, is there (FR-20.4).
pub fn usb_present(folder: &str) -> bool {
    if folder.trim().is_empty() {
        return false;
    }
    let path = Path::new(folder);
    path.exists()
        || path
            .parent()
            .is_some_and(|parent| !parent.as_os_str().is_empty() && parent.exists())
}

impl BackupService {
    /// A service for this data file.
    pub fn new(db: Arc<Db>, config: BackupConfig, parts: BackupParts) -> Self {
        let state = state::load(&config.backups_dir);
        Self {
            db,
            config,
            parts,
            state: Mutex::new(state),
            busy: Mutex::new(()),
            data_seen: Mutex::new(None),
            usb_was_present: Mutex::new(false),
        }
    }

    /// The data file.
    pub fn db(&self) -> &Arc<Db> {
        &self.db
    }

    /// Takes the busy lock, or says that a backup or restore is running.
    pub(crate) fn try_busy(&self) -> Result<MutexGuard<'_, ()>, BackupError> {
        match self.busy.try_lock() {
            Ok(guard) => Ok(guard),
            Err(TryLockError::Poisoned(poisoned)) => Ok(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) => Err(BackupError::new(
                BackupCode::ErrBackupBusy,
                "a backup or restore is running",
            )),
        }
    }

    pub(crate) fn now(&self) -> DateTime<Utc> {
        self.db.clock().now_utc()
    }

    pub(crate) fn now_text(&self) -> String {
        format_utc(self.now())
    }

    /// Changes the remembered state and saves it. A save that fails is logged: the copy that
    /// was just made is still good.
    pub(crate) fn update_state<T>(&self, change: impl FnOnce(&mut State) -> T) -> T {
        let mut state = lock(&self.state);
        let value = change(&mut state);
        if let Err(error) = state::save(&self.config.backups_dir, &state) {
            tracing::warn!("cannot save the backup state: {error}");
        }
        value
    }

    pub(crate) fn state_copy(&self) -> State {
        lock(&self.state).clone()
    }

    /// The folder for scratch files. Emptied when a run starts.
    pub(crate) fn tmp_dir(&self) -> PathBuf {
        self.config.backups_dir.join("tmp")
    }

    /// Cloud copies that wait for the internet.
    pub(crate) fn outbox_dir(&self) -> PathBuf {
        self.config.backups_dir.join("outbox")
    }

    /// Copies of the data as it was just before a restore.
    pub fn safety_dir(&self) -> PathBuf {
        self.config.backups_dir.join("safety")
    }

    /// The folder of the 15-minute local copies.
    pub fn local_dir(&self, settings: &BackupSettings) -> PathBuf {
        if settings.local_dir.trim().is_empty() {
            self.config.backups_dir.join("local")
        } else {
            PathBuf::from(&settings.local_dir)
        }
    }

    /// The settings now.
    ///
    /// # Errors
    /// `ERR_INTERNAL` if the row cannot be read.
    pub fn settings(&self) -> Result<BackupSettings, BackupError> {
        self.db.read(settings::load)
    }

    /// The backup key, if this computer has one.
    pub(crate) fn key(&self) -> Result<Option<BackupKey>, BackupError> {
        match self.parts.secrets.get(SecretName::BackupKey)? {
            Some(text) => Ok(Some(BackupKey::parse(&text)?)),
            None => Ok(None),
        }
    }

    // ---------------------------------------------------------------------------------------
    // Key (FR-20.10, NFR-BAK-6, NFR-BAK-7)
    // ---------------------------------------------------------------------------------------

    /// Makes the backup key and returns it to be shown once and printed. It is stored in the
    /// Credential Manager only.
    ///
    /// A key that exists is kept. The one exception: a key that was never confirmed and has not
    /// locked any USB or cloud copy yet is replaced, so a lost first printout does not lock the
    /// Owner out (the wizard can be run again).
    ///
    /// # Errors
    /// `ERR_BACKUP_KEY_ALREADY_SET`; `ERR_INTERNAL` if the Credential Manager fails.
    pub fn key_setup(&self, user_id: Option<i64>) -> Result<KeyCreated, BackupError> {
        let _busy = self.try_busy()?;
        let state = self.state_copy();
        let has_key = self.parts.secrets.get(SecretName::BackupKey)?.is_some();
        let locked_something =
            state.last_usb.is_some() || state.last_cloud.is_some() || !state.queue.is_empty();
        if has_key && (state.key_confirmed_at.is_some() || locked_something) {
            return Err(BackupError::new(
                BackupCode::ErrBackupKeyAlreadySet,
                "the key exists and is in use",
            ));
        }
        let key = BackupKey::generate()?;
        self.parts.secrets.set(SecretName::BackupKey, key.raw())?;
        let created_at = self.now_text();
        self.update_state(|s| {
            s.key_created_at = Some(created_at.clone());
            s.key_confirmed_at = None;
        });
        self.audit(user_id, "create", "backup_key", None);
        Ok(KeyCreated {
            key: key.display(),
            created_at,
        })
    }

    /// Checks the key the Owner typed from the printed sheet against the stored one, and
    /// remembers that he has it (NFR-BAK-7).
    ///
    /// # Errors
    /// `ERR_BACKUP_KEY_FORMAT` and `ERR_BACKUP_KEY_WRONG` on field `key`;
    /// `ERR_BACKUP_KEY_NOT_SET` when there is no key yet.
    pub fn key_confirm(&self, user_id: Option<i64>, typed: &str) -> Result<(), BackupError> {
        let typed = BackupKey::parse(typed)?;
        let Some(stored) = self.key()? else {
            return Err(BackupError::new(
                BackupCode::ErrBackupKeyNotSet,
                "no key yet",
            ));
        };
        if !stored.same_as(&typed) {
            return Err(
                BackupError::new(BackupCode::ErrBackupKeyWrong, "typed key differs")
                    .on_field("key"),
            );
        }
        let at = self.now_text();
        self.update_state(|s| s.key_confirmed_at = Some(at));
        self.audit(user_id, "confirm", "backup_key", None);
        Ok(())
    }

    /// Writes an audit row. Best effort: the action it describes has happened.
    pub(crate) fn audit(
        &self,
        user_id: Option<i64>,
        action: &str,
        entity: &str,
        new: Option<serde_json::Value>,
    ) {
        let result = self.db.write(|tx| {
            self.db
                .auditor()
                .write(
                    tx,
                    &AuditEntry {
                        user_id,
                        action: action.to_owned(),
                        entity: entity.to_owned(),
                        entity_id: None,
                        old_json: None,
                        new_json: new,
                        reason: None,
                    },
                )
                .map(|_| ())
                .map_err(BackupError::from)
        });
        if let Err(error) = result {
            tracing::warn!("cannot write the audit row of {entity}: {error}");
        }
    }

    // ---------------------------------------------------------------------------------------
    // Settings (PRD 21 "Backup")
    // ---------------------------------------------------------------------------------------

    /// The settings and whether the cloud keys are stored. The keys themselves never leave the
    /// Credential Manager.
    ///
    /// # Errors
    /// `ERR_INTERNAL`.
    pub fn settings_view(&self) -> Result<BackupSettingsView, BackupError> {
        let settings = self.settings()?;
        let has = |name| matches!(self.parts.secrets.get(name), Ok(Some(v)) if !v.is_empty());
        Ok(BackupSettingsView {
            local_dir_effective: self.local_dir(&settings).display().to_string(),
            cloud_access_key_set: has(SecretName::CloudAccessKeyId),
            cloud_secret_set: has(SecretName::CloudSecretAccessKey),
            settings,
        })
    }

    /// Saves the settings (with an audit row) and the cloud keys (in the Credential Manager).
    ///
    /// # Errors
    /// The errors of [`BackupSettings::validate`], each on its field; `ERR_INTERNAL`.
    pub fn settings_set(
        &self,
        user_id: Option<i64>,
        input: &BackupSettingsInput,
    ) -> Result<BackupSettingsView, BackupError> {
        let _busy = self.try_busy()?;
        settings::save(&self.db, user_id, &input.settings)?;
        let secrets = [
            (SecretName::CloudAccessKeyId, &input.cloud_access_key_id),
            (
                SecretName::CloudSecretAccessKey,
                &input.cloud_secret_access_key,
            ),
        ];
        for (name, value) in secrets {
            match value {
                None => {}
                Some(text) if text.trim().is_empty() => self.parts.secrets.delete(name)?,
                Some(text) => self.parts.secrets.set(name, text.trim())?,
            }
        }
        if input.cloud_access_key_id.is_some() || input.cloud_secret_access_key.is_some() {
            // The keys are secret, so the row says only that they changed.
            self.audit(
                user_id,
                "edit",
                "backup_cloud_keys",
                Some(json!({ "changed": true })),
            );
        }
        self.settings_view()
    }

    // ---------------------------------------------------------------------------------------
    // Status (FR-20.7, NFR-BAK-10, NFR-BAK-11)
    // ---------------------------------------------------------------------------------------

    /// The state of the backups. Reads the settings and the remembered state only: no network.
    ///
    /// # Errors
    /// `ERR_INTERNAL`.
    pub fn status(&self) -> Result<BackupStatus, BackupError> {
        let settings = self.settings()?;
        let state = self.state_copy();
        let now = self.now();
        let age_of = |text: &Option<String>| -> Option<Duration> {
            let at = text
                .as_deref()
                .and_then(|t| DateTime::parse_from_rfc3339(t).ok())?;
            Some(now - at.with_timezone(&Utc))
        };
        let cloud_age = age_of(&state.last_cloud.as_ref().map(|s| s.at.clone()));
        let cloud_level = if !settings.cloud_enabled {
            CloudLevel::Off
        } else {
            match cloud_age.map(|age| age.num_hours()) {
                None => CloudLevel::Red,
                Some(hours) if hours > CLOUD_ALERT_HOURS => CloudLevel::Alert,
                Some(hours) if hours > CLOUD_RED_HOURS => CloudLevel::Red,
                Some(_) => CloudLevel::Green,
            }
        };
        let key_set = self.parts.secrets.get(SecretName::BackupKey)?.is_some();
        let key_confirm_due = state.key_confirmed_at.is_none()
            && age_of(&state.key_created_at)
                .is_some_and(|age| age > Duration::days(KEY_CONFIRM_DAYS));
        let test_age = age_of(&state.last_test_restore_at);
        let test_restore_due = state.last_cloud.is_some()
            && (state.last_test_restore_ok != Some(true)
                || test_age.is_none_or(|age| age > Duration::days(TEST_RESTORE_DAYS)));
        Ok(BackupStatus {
            key_set,
            key_confirmed: state.key_confirmed_at.is_some(),
            key_confirm_due,
            last_local_at: state.last_local.as_ref().map(|s| s.at.clone()),
            last_usb_at: state.last_usb.as_ref().map(|s| s.at.clone()),
            last_cloud_at: state.last_cloud.as_ref().map(|s| s.at.clone()),
            cloud_age_hours: cloud_age.map(|age| age.num_hours()),
            cloud_level,
            queue_len: i64::try_from(state.queue.len()).unwrap_or(i64::MAX),
            usb_enabled: settings.usb_enabled,
            usb_present: settings.usb_enabled && usb_present(&settings.usb_folder),
            cloud_enabled: settings.cloud_enabled,
            cloud_ready: self
                .parts
                .cloud
                .ready(&settings, self.parts.secrets.as_ref()),
            running: self.busy.try_lock().is_err(),
            errors: state
                .failures
                .iter()
                .map(|f| StatusError {
                    kind: f.kind,
                    code: f.code,
                    at: f.at.clone(),
                    check_failed: f.check_failed,
                    file: f.file.clone(),
                })
                .collect(),
            last_test_restore_at: state.last_test_restore_at.clone(),
            test_restore_due,
            pack_pending: self.parts.pack.pending(),
        })
    }

    // ---------------------------------------------------------------------------------------
    // List (FR-20.8)
    // ---------------------------------------------------------------------------------------

    /// The copies in the local folder, on the USB drive and in the cloud, newest first, one page.
    ///
    /// # Errors
    /// `ERR_INTERNAL`. An unreachable cloud is not an error: `cloud_reachable` is false.
    pub fn list(&self, input: &BackupListInput) -> Result<BackupList, BackupError> {
        let settings = self.settings()?;
        let wants = |kind: BackupKind| input.kind.is_none_or(|k| k == kind);
        let mut rows = Vec::new();
        if wants(BackupKind::Local) {
            rows.extend(scan_folder(BackupKind::Local, &self.local_dir(&settings)));
        }
        if wants(BackupKind::Usb) && settings.usb_enabled && usb_present(&settings.usb_folder) {
            rows.extend(scan_folder(
                BackupKind::Usb,
                Path::new(&settings.usb_folder),
            ));
        }
        let mut cloud_reachable = true;
        if wants(BackupKind::Cloud)
            && self
                .parts
                .cloud
                .ready(&settings, self.parts.secrets.as_ref())
        {
            match self.cloud_rows(&settings) {
                Ok(found) => rows.extend(found),
                Err(error) => {
                    tracing::warn!("cannot list the cloud copies: {error}");
                    cloud_reachable = false;
                }
            }
        }
        rows.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.name.cmp(&a.name))
        });
        let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        let limit = if input.limit > 0 {
            input.limit
        } else {
            DEFAULT_PAGE
        };
        let offset = usize::try_from(input.offset.max(0)).unwrap_or(0);
        let page = rows
            .into_iter()
            .skip(offset)
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .collect();
        Ok(BackupList {
            rows: page,
            total_count,
            cloud_reachable,
        })
    }

    fn cloud_rows(&self, settings: &BackupSettings) -> Result<Vec<BackupRow>, BackupError> {
        let store = self
            .parts
            .cloud
            .open(settings, self.parts.secrets.as_ref())?;
        let objects = store
            .list(CLOUD_DIR)
            .map_err(crate::cloud::CloudError::into_backup_error)?;
        let manifests: std::collections::BTreeSet<&str> = objects
            .iter()
            .filter_map(|o| o.key.strip_prefix(CLOUD_DIR))
            .filter(|name| name.ends_with(".json"))
            .collect();
        Ok(objects
            .iter()
            .filter_map(|object| {
                let name = object.key.strip_prefix(CLOUD_DIR)?;
                if !is_copy_name(BackupKind::Cloud, name) {
                    return None;
                }
                // The manifest is uploaded after the bundle and checked, so its presence says
                // the upload finished.
                let status = if manifests.contains(manifest_name(name).as_str()) {
                    RowStatus::Good
                } else {
                    RowStatus::Unchecked
                };
                Some(BackupRow {
                    kind: BackupKind::Cloud,
                    name: name.to_owned(),
                    created_at: parse_stamp(name)
                        .map_or_else(|| format_utc(object.modified), format_utc),
                    size_bytes: object.size,
                    encrypted: true,
                    status,
                })
            })
            .collect())
    }
}

/// The copies in one folder, with the result of their check from the manifest next to them.
fn scan_folder(kind: BackupKind, dir: &Path) -> Vec<BackupRow> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !is_copy_name(kind, &name) {
                return None;
            }
            let size_bytes = entry.metadata().ok()?.len();
            let manifest: Option<Manifest> = fs::read(dir.join(manifest_name(&name)))
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok());
            let created_at = manifest
                .as_ref()
                .map(|m| m.created_at.clone())
                .or_else(|| parse_stamp(&name).map(format_utc))?;
            let status = match &manifest {
                Some(m) if m.integrity_ok => RowStatus::Good,
                Some(_) => RowStatus::Bad,
                None => RowStatus::Unchecked,
            };
            Some(BackupRow {
                kind,
                name,
                created_at,
                size_bytes,
                encrypted: kind != BackupKind::Local,
                status,
            })
        })
        .collect()
}
