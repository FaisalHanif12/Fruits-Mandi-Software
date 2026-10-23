//! Making copies: the 15-minute local copy, the daily USB and cloud copy, the cloud queue
//! (PRD FR-20.2 to FR-20.7, NFR-BAK-1 to NFR-BAK-9, Architecture 13.4 and 14).
//!
//! One run takes one snapshot, checks it, then puts it where it was asked to go. The snapshot is
//! never changed after it is checked. Every copy is written under a `.partial` name and renamed
//! when whole, checked again where it lands, and only then remembered as good. A cloud copy goes
//! into the outbox first, so a crash, a power cut or a missing internet connection never loses it
//! (NFR-BAK-9).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::{DateTime, Duration, Utc};
use mandi_db::clock::format_utc;
use mandi_db::rusqlite::params;

use crate::api::{RunReport, TargetResult};
use crate::cloud::CloudError;
use crate::daily_pack::seal_file;
use crate::error::{BackupCode, BackupError, CloudReason};
use crate::io::{copy_atomic, hash_file, remove_quietly, require_room, write_atomic, write_error};
use crate::model::{BackupKind, Embedded, InnerManifest, MANIFEST_FORMAT, Manifest};
use crate::names::{bundle_name, is_copy_name, local_name, manifest_name, stamp};
use crate::pack::{Sealed, open, seal};
use crate::retention::{KEEP_LOCAL, KEEP_USB, cloud_delete, newest_n_delete};
use crate::service::{BackupService, CLOUD_DIR, lock, usb_present};
use crate::settings::BackupSettings;
use crate::snapshot::{Inspected, ROOM_MARGIN, inspect, snapshot_copy};
use crate::state::{Failure, Queued, Stamp};

/// Which places a run goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scope {
    /// The local folder.
    pub local: bool,
    /// The USB drive, when it is on and plugged in.
    pub usb: bool,
    /// The cloud, when it is on.
    pub cloud: bool,
}

impl Scope {
    /// Every place: "backup now", the daily run and the run at exit.
    pub const ALL: Scope = Scope {
        local: true,
        usb: true,
        cloud: true,
    };
    /// The 15-minute run.
    pub const LOCAL_ONLY: Scope = Scope {
        local: true,
        usb: false,
        cloud: false,
    };
    /// The run when a USB drive is plugged in.
    pub const USB_ONLY: Scope = Scope {
        local: false,
        usb: true,
        cloud: false,
    };
}

/// An encrypted bundle in the scratch folder, ready to go to the USB drive and the cloud.
struct Bundle {
    path: PathBuf,
    name: String,
    sealed: Sealed,
}

/// What goes into one row of `backup_log`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogRow<'a> {
    pub kind: BackupKind,
    pub path_or_key: &'a str,
    pub size_bytes: u64,
    pub checksum: Option<&'a str>,
    pub integrity_ok: bool,
    pub counts: Option<&'a BTreeMap<String, i64>>,
    pub error: Option<BackupCode>,
}

/// The day a time belongs to for the shop (Pakistan time, UTC+5, Code Standards G9).
pub fn shop_day(time: DateTime<Utc>) -> String {
    (time + Duration::hours(5)).format("%Y-%m-%d").to_string()
}

fn target(kind: BackupKind) -> TargetResult {
    TargetResult {
        kind,
        ok: false,
        file: None,
        size_bytes: 0,
        queued: false,
        code: None,
        reason: None,
    }
}

fn failed(kind: BackupKind, code: BackupCode) -> TargetResult {
    TargetResult {
        code: Some(code),
        ..target(kind)
    }
}

/// A failed result that also says why a cloud copy failed (offline, refused, not found).
fn failed_with(kind: BackupKind, error: &BackupError) -> TargetResult {
    TargetResult {
        reason: error.cloud_reason(),
        ..failed(kind, error.code)
    }
}

fn good(kind: BackupKind, file: &str, size_bytes: u64) -> TargetResult {
    TargetResult {
        ok: true,
        file: Some(file.to_owned()),
        size_bytes,
        ..target(kind)
    }
}

fn clean_dir(dir: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Err(error) = fs::remove_dir_all(&path) {
                    tracing::warn!(path = %path.display(), "cannot clean scratch folder: {error}");
                }
            } else {
                remove_quietly(&path);
            }
        }
    }
}

impl BackupService {
    /// Makes copies to the places in `scope` (the ones that are switched on and, for USB,
    /// plugged in). A failure of one place does not stop the others. Every place tried has a
    /// result; nothing is skipped silently (NFR-BAK-9).
    ///
    /// # Errors
    /// `ERR_BACKUP_BUSY` when a backup or restore is already running.
    pub fn run(&self, scope: Scope) -> Result<RunReport, BackupError> {
        let _busy = self.try_busy()?;
        Ok(self.run_locked(scope))
    }

    /// The run at exit (FR-20.3). The same as "backup now"; the app waits for it a short time.
    ///
    /// # Errors
    /// `ERR_BACKUP_BUSY`.
    pub fn run_on_exit(&self) -> Result<RunReport, BackupError> {
        self.run(Scope::ALL)
    }

    fn record_failure(
        &self,
        kind: BackupKind,
        code: BackupCode,
        check_failed: bool,
        file: Option<String>,
    ) {
        self.record_failure_why(kind, code, check_failed, file, None);
    }

    /// As `record_failure`, with the reason of a failed cloud copy.
    fn record_failure_why(
        &self,
        kind: BackupKind,
        code: BackupCode,
        check_failed: bool,
        file: Option<String>,
        reason: Option<CloudReason>,
    ) {
        let at = self.now_text();
        self.update_state(|s| {
            s.add_failure(Failure {
                kind,
                code,
                at,
                check_failed,
                file,
                reason,
            });
        });
    }

    fn run_locked(&self, scope: Scope) -> RunReport {
        let started = self.now();
        let mut report = RunReport {
            started_at: format_utc(started),
            results: Vec::new(),
            snapshot_ms: 0,
            check_ms: 0,
        };
        let settings = match self.settings() {
            Ok(settings) => settings,
            Err(error) => {
                tracing::error!("backup cannot read its settings: {error}");
                report.results.push(failed(BackupKind::Local, error.code));
                return report;
            }
        };
        let mut wanted = Vec::new();
        if scope.local {
            wanted.push(BackupKind::Local);
        }
        let mut usb_missing = false;
        if scope.usb && settings.usb_enabled {
            if usb_present(&settings.usb_folder) {
                wanted.push(BackupKind::Usb);
            } else {
                usb_missing = true;
            }
        }
        if scope.cloud && settings.cloud_enabled {
            wanted.push(BackupKind::Cloud);
        }
        if usb_missing {
            report
                .results
                .push(failed(BackupKind::Usb, BackupCode::ErrBackupUsbNotPresent));
        }
        if wanted.is_empty() {
            return report;
        }
        if wanted.contains(&BackupKind::Local) {
            let at = report.started_at.clone();
            self.update_state(|s| s.last_local_try = Some(at));
        }

        let tmp = self.tmp_dir();
        clean_dir(&tmp);
        let name_stamp = stamp(started);
        let snap = tmp.join(format!("snapshot-{name_stamp}.db"));

        let timer = Instant::now();
        if let Err(error) = snapshot_copy(&self.db, &snap, self.parts.disk.as_ref()) {
            tracing::error!("backup snapshot failed: {error}");
            for kind in wanted {
                self.record_failure(kind, error.code, false, None);
                report.results.push(failed(kind, error.code));
            }
            return report;
        }
        report.snapshot_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX);

        let timer = Instant::now();
        let inspected = match inspect(&snap) {
            Ok(inspected) => inspected,
            Err(error) => {
                tracing::error!("the snapshot failed its check: {error}");
                for kind in wanted {
                    self.record_failure(kind, BackupCode::ErrBackupCheckFailed, true, None);
                    self.log_row(&LogRow {
                        kind,
                        path_or_key: &snap.display().to_string(),
                        size_bytes: 0,
                        checksum: None,
                        integrity_ok: false,
                        counts: None,
                        error: Some(BackupCode::ErrBackupCheckFailed),
                    });
                    report
                        .results
                        .push(failed(kind, BackupCode::ErrBackupCheckFailed));
                }
                remove_quietly(&snap);
                return report;
            }
        };
        report.check_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX);

        let created_at = report.started_at.clone();
        if wanted.contains(&BackupKind::Local) {
            report.results.push(self.copy_local(
                &settings,
                &snap,
                &inspected,
                &name_stamp,
                &created_at,
            ));
        }
        let wants_usb = wanted.contains(&BackupKind::Usb);
        let wants_cloud = wanted.contains(&BackupKind::Cloud);
        if wants_usb || wants_cloud {
            match self.make_bundle(&snap, &inspected, &name_stamp, &created_at) {
                Ok(bundle) => {
                    let manifest =
                        bundle_manifest(&bundle, &inspected, &created_at, &self.config.app_version);
                    if wants_usb {
                        report
                            .results
                            .push(self.copy_usb(&settings, &bundle, &manifest));
                    }
                    if wants_cloud {
                        report
                            .results
                            .push(self.send_cloud(&settings, bundle, &manifest));
                    } else {
                        remove_quietly(&bundle.path);
                    }
                }
                Err(error) => {
                    tracing::error!("cannot make the encrypted copy: {error}");
                    for kind in [BackupKind::Usb, BackupKind::Cloud] {
                        if wanted.contains(&kind) {
                            self.record_failure(
                                kind,
                                error.code,
                                error.code == BackupCode::ErrBackupCheckFailed,
                                None,
                            );
                            report.results.push(failed(kind, error.code));
                        }
                    }
                }
            }
        }
        remove_quietly(&snap);
        self.mark_data_seen();
        report
    }

    /// The data version now becomes "seen": the next 15-minute run happens only if it moves.
    /// Our own log rows move it too, so this is called after they are written.
    pub(crate) fn mark_data_seen(&self) {
        match self.db.data_version() {
            Ok(version) => *lock(&self.data_seen) = Some(version),
            Err(error) => tracing::warn!("cannot read the data version: {error}"),
        }
    }

    fn copy_local(
        &self,
        settings: &BackupSettings,
        snap: &Path,
        inspected: &Inspected,
        name_stamp: &str,
        created_at: &str,
    ) -> TargetResult {
        let kind = BackupKind::Local;
        let result = (|| -> Result<(String, u64), BackupError> {
            let dir = self.local_dir(settings);
            fs::create_dir_all(&dir).map_err(|e| write_error(kind, &e))?;
            require_room(
                self.parts.disk.as_ref(),
                &dir,
                inspected.size_bytes.saturating_add(ROOM_MARGIN),
            )?;
            let name = local_name(name_stamp);
            let dest = dir.join(&name);
            copy_atomic(snap, &dest, self.parts.hook.as_ref(), kind)
                .map_err(|e| write_error(kind, &e))?;
            // Read the copy back from the disk: it must be the checked bytes.
            let again = hash_file(&dest).map_err(|e| write_error(kind, &e))?;
            if again != inspected.blake3 {
                remove_quietly(&dest);
                return Err(BackupError::new(
                    BackupCode::ErrBackupCheckFailed,
                    "the local copy differs from its snapshot",
                ));
            }
            let manifest = Manifest {
                format: MANIFEST_FORMAT,
                kind,
                created_at: created_at.to_owned(),
                app_version: self.config.app_version.clone(),
                schema_version: inspected.schema_version,
                encrypted: false,
                file: name.clone(),
                size_bytes: inspected.size_bytes,
                file_blake3: inspected.blake3.clone(),
                integrity_ok: true,
                inner: Some(inner_of(inspected)),
            };
            write_atomic(
                &dir.join(manifest_name(&name)),
                &serde_json::to_vec_pretty(&manifest)?,
            )
            .map_err(|e| write_error(kind, &e))?;
            prune_folder(kind, &dir, KEEP_LOCAL);
            Ok((name, inspected.size_bytes))
        })();
        self.finish_target(kind, result, Some(inspected), created_at)
    }

    /// Records the end of one target: state, log row, result.
    fn finish_target(
        &self,
        kind: BackupKind,
        result: Result<(String, u64), BackupError>,
        inspected: Option<&Inspected>,
        created_at: &str,
    ) -> TargetResult {
        match result {
            Ok((name, size)) => {
                let stamp = Stamp {
                    at: created_at.to_owned(),
                    file: name.clone(),
                    size_bytes: size,
                };
                self.update_state(|s| {
                    match kind {
                        BackupKind::Local => s.last_local = Some(stamp),
                        BackupKind::Usb => s.last_usb = Some(stamp),
                        BackupKind::Cloud => s.last_cloud = Some(stamp),
                    }
                    s.clear_failures(kind);
                });
                self.log_row(&LogRow {
                    kind,
                    path_or_key: &name,
                    size_bytes: size,
                    checksum: inspected.map(|i| i.blake3.as_str()),
                    integrity_ok: true,
                    counts: inspected.map(|i| &i.counts),
                    error: None,
                });
                good(kind, &name, size)
            }
            Err(error) => {
                let check_failed = error.code == BackupCode::ErrBackupCheckFailed;
                tracing::error!(kind = kind.as_str(), "backup copy failed: {error}");
                self.record_failure(kind, error.code, check_failed, None);
                self.log_row(&LogRow {
                    kind,
                    path_or_key: "",
                    size_bytes: 0,
                    checksum: None,
                    integrity_ok: !check_failed,
                    counts: None,
                    error: Some(error.code),
                });
                failed(kind, error.code)
            }
        }
    }

    fn make_bundle(
        &self,
        snap: &Path,
        inspected: &Inspected,
        name_stamp: &str,
        created_at: &str,
    ) -> Result<Bundle, BackupError> {
        let key = self
            .key()?
            .ok_or_else(|| BackupError::new(BackupCode::ErrBackupKeyNotSet, "no backup key"))?;
        let tmp = self.tmp_dir();
        require_room(
            self.parts.disk.as_ref(),
            &tmp,
            inspected.size_bytes.saturating_add(ROOM_MARGIN),
        )?;
        let name = bundle_name(name_stamp);
        let path = tmp.join(&name);
        let embedded = Embedded {
            format: MANIFEST_FORMAT,
            created_at: created_at.to_owned(),
            app_version: self.config.app_version.clone(),
            schema_version: inspected.schema_version,
            inner: inner_of(inspected),
        };
        let sealed = seal(
            snap,
            &embedded,
            &key,
            &path,
            self.parts.hook.as_ref(),
            BackupKind::Local,
        )?;
        // Open it again and compare: the bundle must give back the checked database.
        let verify = tmp.join(format!("verify-{name_stamp}.db"));
        let opened = open(&path, &key, &verify);
        remove_quietly(&verify);
        match opened {
            Ok(back) if back == embedded => Ok(Bundle { path, name, sealed }),
            Ok(_) => {
                remove_quietly(&path);
                Err(BackupError::new(
                    BackupCode::ErrBackupCheckFailed,
                    "the bundle gave back another manifest",
                ))
            }
            Err(error) => {
                remove_quietly(&path);
                Err(BackupError::new(
                    BackupCode::ErrBackupCheckFailed,
                    format!("the bundle does not open: {error}"),
                ))
            }
        }
    }

    fn copy_usb(
        &self,
        settings: &BackupSettings,
        bundle: &Bundle,
        manifest: &Manifest,
    ) -> TargetResult {
        let kind = BackupKind::Usb;
        let result = (|| -> Result<(String, u64), BackupError> {
            let dir = PathBuf::from(&settings.usb_folder);
            if !usb_present(&settings.usb_folder) {
                return Err(BackupError::new(
                    BackupCode::ErrBackupUsbNotPresent,
                    "the drive is not there",
                ));
            }
            fs::create_dir_all(&dir).map_err(|e| write_error(kind, &e))?;
            require_room(
                self.parts.disk.as_ref(),
                &dir,
                bundle.sealed.bundle_size.saturating_add(ROOM_MARGIN),
            )?;
            let dest = dir.join(&bundle.name);
            copy_atomic(&bundle.path, &dest, self.parts.hook.as_ref(), kind)
                .map_err(|e| write_error(kind, &e))?;
            let again = hash_file(&dest).map_err(|e| write_error(kind, &e))?;
            if again != bundle.sealed.bundle_blake3 {
                remove_quietly(&dest);
                return Err(BackupError::new(
                    BackupCode::ErrBackupCheckFailed,
                    "the USB copy differs from its bundle",
                ));
            }
            let manifest = Manifest {
                kind,
                ..manifest.clone()
            };
            write_atomic(
                &dir.join(manifest_name(&bundle.name)),
                &serde_json::to_vec_pretty(&manifest)?,
            )
            .map_err(|e| write_error(kind, &e))?;
            prune_folder(kind, &dir, KEEP_USB);
            Ok((bundle.name.clone(), bundle.sealed.bundle_size))
        })();
        if matches!(&result, Err(e) if e.code == BackupCode::ErrBackupUsbNotPresent) {
            return failed(kind, BackupCode::ErrBackupUsbNotPresent);
        }
        self.finish_target(kind, result, None, &manifest.created_at)
    }

    /// Puts the bundle in the outbox, then tries to upload it (NFR-BAK-9).
    fn send_cloud(
        &self,
        settings: &BackupSettings,
        bundle: Bundle,
        manifest: &Manifest,
    ) -> TargetResult {
        let kind = BackupKind::Cloud;
        let queued = (|| -> Result<(), BackupError> {
            let outbox = self.outbox_dir();
            fs::create_dir_all(&outbox).map_err(|e| BackupError::io("outbox", &e))?;
            let dest = outbox.join(&bundle.name);
            if fs::rename(&bundle.path, &dest).is_err() {
                copy_atomic(&bundle.path, &dest, &crate::io::NoHook, BackupKind::Local)
                    .map_err(|e| write_error(BackupKind::Local, &e))?;
                remove_quietly(&bundle.path);
            }
            let manifest = Manifest {
                kind,
                ..manifest.clone()
            };
            write_atomic(
                &outbox.join(manifest_name(&bundle.name)),
                &serde_json::to_vec_pretty(&manifest)?,
            )
            .map_err(|e| BackupError::io("outbox", &e))?;
            let at = self.now_text();
            let file = bundle.name.clone();
            self.update_state(|s| {
                s.queue.push(Queued {
                    file,
                    queued_at: at,
                    attempts: 0,
                    last_reason: String::new(),
                });
            });
            Ok(())
        })();
        if let Err(error) = queued {
            self.record_failure(kind, error.code, false, None);
            return failed(kind, error.code);
        }
        match self.upload_one(settings, &bundle.name) {
            Ok(()) => {
                self.log_row(&LogRow {
                    kind,
                    path_or_key: &bundle.name,
                    size_bytes: bundle.sealed.bundle_size,
                    checksum: Some(&bundle.sealed.bundle_blake3),
                    integrity_ok: true,
                    counts: None,
                    error: None,
                });
                good(kind, &bundle.name, bundle.sealed.bundle_size)
            }
            Err(error) => TargetResult {
                file: Some(bundle.name.clone()),
                size_bytes: bundle.sealed.bundle_size,
                queued: true,
                ..failed_with(kind, &error)
            },
        }
    }

    /// Tries the cloud copies that are waiting, oldest first. Stops at the first failure: with
    /// no internet every try fails the same way.
    ///
    /// # Errors
    /// `ERR_BACKUP_BUSY`.
    pub fn retry_queue(&self) -> Result<Vec<TargetResult>, BackupError> {
        let _busy = self.try_busy()?;
        let at = self.now_text();
        self.update_state(|s| s.last_queue_try = Some(at));
        let settings = self.settings()?;
        let mut results = Vec::new();
        for item in self.state_copy().queue {
            match self.upload_one(&settings, &item.file) {
                Ok(()) => results.push(good(BackupKind::Cloud, &item.file, 0)),
                Err(error) => {
                    results.push(TargetResult {
                        file: Some(item.file),
                        queued: true,
                        ..failed_with(BackupKind::Cloud, &error)
                    });
                    break;
                }
            }
        }
        Ok(results)
    }

    /// Uploads one queued bundle: bundle first, then a check by downloading it again, then its
    /// manifest. The manifest in the cloud means "uploaded and checked". On success the outbox
    /// files and the queue entry go. On failure the entry stays with its reason.
    fn upload_one(&self, settings: &BackupSettings, name: &str) -> Result<(), BackupError> {
        let kind = BackupKind::Cloud;
        let outcome = (|| -> Result<(), BackupError> {
            let bundle_path = self.outbox_dir().join(name);
            let manifest_path = self.outbox_dir().join(manifest_name(name));
            let manifest: Manifest = serde_json::from_slice(
                &fs::read(&manifest_path).map_err(|e| BackupError::io("outbox manifest", &e))?,
            )?;
            if !bundle_path.exists() {
                return Err(BackupError::internal(format!(
                    "queued bundle {name} is missing"
                )));
            }
            let store = self
                .parts
                .cloud
                .open(settings, self.parts.secrets.as_ref())?;
            let object = format!("{CLOUD_DIR}{name}");
            let hook = self.parts.hook.as_ref();
            store
                .put_file(&object, &bundle_path, hook)
                .map_err(CloudError::into_backup_error)?;
            // "Upload confirmed by checksum" (NFR-BAK-5, Architecture 13.4).
            let verify = self.tmp_dir().join(format!("verify-{name}"));
            let downloaded = store.get_file(&object, &verify);
            let again = downloaded.map(|()| hash_file(&verify));
            remove_quietly(&verify);
            match again {
                Ok(Ok(sum)) if sum == manifest.file_blake3 => {}
                Ok(_) => {
                    let _ = store.delete(&object);
                    return Err(BackupError::new(
                        BackupCode::ErrBackupCheckFailed,
                        "the cloud copy differs from its bundle",
                    ));
                }
                Err(error) => return Err(error.into_backup_error()),
            }
            store
                .put_file(
                    &format!("{CLOUD_DIR}{}", manifest_name(name)),
                    &manifest_path,
                    hook,
                )
                .map_err(CloudError::into_backup_error)?;
            // Prune and pack are best effort: the copy itself is safe.
            if let Err(error) = self.prune_cloud(store.as_ref()) {
                tracing::warn!("cannot prune the cloud copies: {error}");
            }
            self.upload_pack(store.as_ref(), &manifest.created_at, &bundle_path);
            remove_quietly(&bundle_path);
            remove_quietly(&manifest_path);
            let stamp = Stamp {
                at: manifest.created_at.clone(),
                file: name.to_owned(),
                size_bytes: manifest.size_bytes,
            };
            self.update_state(|s| {
                s.queue.retain(|q| q.file != name);
                let newer = s.last_cloud.as_ref().is_none_or(|last| last.at < stamp.at);
                if newer {
                    s.last_cloud = Some(stamp);
                }
                s.clear_failures(kind);
            });
            Ok(())
        })();
        if let Err(error) = &outcome {
            tracing::warn!("cloud upload of {name} failed: {error}");
            let reason = match error.values.get("reason") {
                Some(mandi_core::error::ErrorValue::Text(text)) => text.clone(),
                _ => "failed".to_owned(),
            };
            let check_failed = error.code == BackupCode::ErrBackupCheckFailed;
            self.update_state(|s| {
                if let Some(item) = s.queue.iter_mut().find(|q| q.file == name) {
                    item.attempts = item.attempts.saturating_add(1);
                    item.last_reason = reason;
                }
            });
            self.record_failure_why(
                kind,
                error.code,
                check_failed,
                Some(name.to_owned()),
                error.cloud_reason(),
            );
        }
        outcome
    }

    /// Deletes the cloud copies that the retention rules (NFR-BAK-8) no longer keep.
    fn prune_cloud(&self, store: &dyn crate::cloud::CloudStore) -> Result<(), BackupError> {
        let objects = store
            .list(CLOUD_DIR)
            .map_err(CloudError::into_backup_error)?;
        let names: Vec<String> = objects
            .iter()
            .filter_map(|o| o.key.strip_prefix(CLOUD_DIR))
            .filter(|name| is_copy_name(BackupKind::Cloud, name))
            .map(str::to_owned)
            .collect();
        for name in cloud_delete(&names) {
            store
                .delete(&format!("{CLOUD_DIR}{name}"))
                .map_err(CloudError::into_backup_error)?;
            store
                .delete(&format!("{CLOUD_DIR}{}", manifest_name(&name)))
                .map_err(CloudError::into_backup_error)?;
        }
        Ok(())
    }

    /// Builds the readable pack once a day and uploads it encrypted (FR-20.5). The pack is built
    /// from the snapshot inside `bundle`, the very copy that was just uploaded, so it holds the
    /// numbers of that backup even if the live books changed meanwhile (a copy that waited in the
    /// queue, or a save during the upload). It never stops or fails the backup.
    fn upload_pack(&self, store: &dyn crate::cloud::CloudStore, created_at: &str, bundle: &Path) {
        let Some(day) = DateTime::parse_from_rfc3339(created_at)
            .ok()
            .map(|t| shop_day(t.with_timezone(&Utc)))
        else {
            return;
        };
        if self.state_copy().last_pack_day.as_deref() == Some(day.as_str()) {
            return;
        }
        let result = (|| -> Result<bool, BackupError> {
            let Some(key) = self.key()? else {
                return Ok(false);
            };
            let work = self.tmp_dir().join("pack");
            fs::create_dir_all(&work).map_err(|e| BackupError::io("pack", &e))?;
            // The snapshot is unpacked for the pack only and goes away with it, whatever happens.
            let snapshot = work.join("pack-snapshot.db");
            let built = open(bundle, &key, &snapshot)
                .and_then(|_| self.parts.pack.build(&day, &snapshot, &work));
            remove_quietly(&snapshot);
            // SQLite may have made its side files while the snapshot was open.
            for side in ["pack-snapshot.db-wal", "pack-snapshot.db-shm"] {
                remove_quietly(&work.join(side));
            }
            let files = built?;
            if files.is_empty() {
                return Ok(false);
            }
            for file in files {
                let sealed = work.join(format!("{}.age", file.name));
                seal_file(&file.path, &key, &sealed, self.parts.hook.as_ref())?;
                store
                    .put_file(
                        &format!("pack/{day}/{}.age", file.name),
                        &sealed,
                        self.parts.hook.as_ref(),
                    )
                    .map_err(CloudError::into_backup_error)?;
            }
            Ok(true)
        })();
        match result {
            Ok(true) => self.update_state(|s| s.last_pack_day = Some(day)),
            Ok(false) => {}
            Err(error) => tracing::warn!("cannot upload the readable pack: {error}"),
        }
    }

    /// One row of `backup_log` (Architecture 15.3). Best effort: the copy is already made.
    pub(crate) fn log_row(&self, row: &LogRow<'_>) {
        let LogRow {
            kind,
            path_or_key,
            size_bytes,
            checksum,
            integrity_ok,
            counts,
            error,
        } = *row;
        let counts_json = counts.and_then(|c| serde_json::to_string(c).ok());
        let error_text = error.and_then(|c| serde_json::to_string(&c).ok());
        let at = self.now_text();
        let size = i64::try_from(size_bytes).unwrap_or(i64::MAX);
        let result = self.db.write(|tx| {
            tx.execute(
                "INSERT INTO backup_log (at, kind, path_or_key, size_bytes, checksum, integrity_ok, row_counts_json, error) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![at, kind.as_str(), path_or_key, size, checksum, i64::from(integrity_ok), counts_json, error_text],
            )
            .map(|_| ())
            .map_err(BackupError::from)
        });
        if let Err(error) = result {
            tracing::warn!("cannot write the backup log row: {error}");
        }
    }
}

fn inner_of(inspected: &Inspected) -> InnerManifest {
    InnerManifest {
        plain_blake3: inspected.blake3.clone(),
        plain_size: inspected.size_bytes,
        counts: inspected.counts.clone(),
        totals: inspected.totals.clone(),
    }
}

/// The manifest next to an encrypted bundle: no money and no counts (NFR-BAK-6).
fn bundle_manifest(
    bundle: &Bundle,
    inspected: &Inspected,
    created_at: &str,
    app_version: &str,
) -> Manifest {
    Manifest {
        format: MANIFEST_FORMAT,
        kind: BackupKind::Cloud,
        created_at: created_at.to_owned(),
        app_version: app_version.to_owned(),
        schema_version: inspected.schema_version,
        encrypted: true,
        file: bundle.name.clone(),
        size_bytes: bundle.sealed.bundle_size,
        file_blake3: bundle.sealed.bundle_blake3.clone(),
        integrity_ok: true,
        inner: None,
    }
}

/// Deletes the oldest copies in a folder beyond `keep`, with their manifests.
fn prune_folder(kind: BackupKind, dir: &Path, keep: usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let names: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| is_copy_name(kind, name))
        .collect();
    for name in newest_n_delete(&names, keep) {
        remove_quietly(&dir.join(&name));
        remove_quietly(&dir.join(manifest_name(&name)));
    }
}
