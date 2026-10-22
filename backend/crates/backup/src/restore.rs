//! Restore, preview and test restore (PRD FR-20.9, NFR-BAK-11, NFR-BAK-12, NFR-REL-4,
//! Architecture 14).
//!
//! The order of a restore is fixed so that a failure at any step leaves the data as it was:
//! fetch the copy, check its checksum, unlock it with the key, check the database inside, read
//! its books, make a safety copy of the live data, upgrade the copy if it is older than the app,
//! replace the data in one step, check the result. Nothing in the live data changes before the
//! safety copy is made and checked.

use std::fs;
use std::path::{Path, PathBuf};

use mandi_db::clock::format_utc;
use mandi_db::migrate;
use mandi_db::rusqlite::{Connection, OptionalExtension};
use serde_json::json;

use crate::api::{
    BackupRef, LostWork, RestoreDone, RestoreInput, RestorePreview, RestorePreviewInput, RowStatus,
    TestCheck, TestCheckName, TestRestoreInput, TestRestoreResult,
};
use crate::error::{BackupCode, BackupError};
use crate::io::{copy_atomic, hash_file, remove_quietly, require_room, write_atomic, write_error};
use crate::key::BackupKey;
use crate::model::{BackupKind, BackupTotals, Embedded, MANIFEST_FORMAT, Manifest};
use crate::names::{check_name, local_name, manifest_name, parse_stamp, stamp};
use crate::pack::open;
use crate::retention::newest_n_delete;
use crate::secrets::SecretName;
use crate::service::{BackupService, CLOUD_DIR, usb_present};
use crate::settings::BackupSettings;
use crate::snapshot::{Inspected, ROOM_MARGIN, inspect, read_totals, snapshot_copy};

/// Safety copies kept (the newest ones).
const KEEP_SAFETY: usize = 10;

fn corrupt(detail: impl std::fmt::Display) -> BackupError {
    BackupError::new(BackupCode::ErrBackupCorrupt, detail)
}

/// A copy brought into the scratch folder, not yet opened.
struct Fetched {
    path: PathBuf,
    encrypted: bool,
    manifest: Option<Manifest>,
}

/// A copy opened and checked: a plain SQLite file in the scratch folder.
struct Opened {
    plain: PathBuf,
    inspected: Inspected,
    created_at: String,
    size_bytes: u64,
    key_used: Option<BackupKey>,
}

impl BackupService {
    fn work_dir(&self, label: &str) -> Result<PathBuf, BackupError> {
        let dir = self
            .tmp_dir()
            .join(format!("{label}-{}", stamp(self.now())));
        fs::create_dir_all(&dir).map_err(|e| BackupError::io("scratch folder", &e))?;
        Ok(dir)
    }

    fn app_schema_version(&self) -> i64 {
        i64::try_from(self.db.migration().to).unwrap_or(i64::MAX)
    }

    /// Brings the copy into `work` and checks its checksum against the manifest next to it.
    fn fetch(
        &self,
        settings: &BackupSettings,
        backup: &BackupRef,
        work: &Path,
    ) -> Result<Fetched, BackupError> {
        check_name(backup.kind, &backup.name)?;
        let encrypted = backup.kind != BackupKind::Local;
        let path = work.join(if encrypted { "copy.mbk" } else { "copy.db" });
        let manifest_file = work.join("manifest.json");
        let not_found =
            || BackupError::new(BackupCode::ErrBackupNotFound, "no such backup").on_field("name");
        match backup.kind {
            BackupKind::Local => {
                let dir = self.local_dir(settings);
                let source = dir.join(&backup.name);
                if !source.exists() {
                    return Err(not_found());
                }
                copy_atomic(&source, &path, self.parts.hook.as_ref(), BackupKind::Local)
                    .map_err(|e| write_error(BackupKind::Local, &e))?;
                let _ = fs::copy(dir.join(manifest_name(&backup.name)), &manifest_file);
            }
            BackupKind::Usb => {
                if !usb_present(&settings.usb_folder) {
                    return Err(BackupError::new(
                        BackupCode::ErrBackupUsbNotPresent,
                        "the drive is not there",
                    ));
                }
                let dir = PathBuf::from(&settings.usb_folder);
                let source = dir.join(&backup.name);
                if !source.exists() {
                    return Err(not_found());
                }
                copy_atomic(&source, &path, self.parts.hook.as_ref(), BackupKind::Local)
                    .map_err(|e| write_error(BackupKind::Usb, &e))?;
                let _ = fs::copy(dir.join(manifest_name(&backup.name)), &manifest_file);
            }
            BackupKind::Cloud => {
                let store = self
                    .parts
                    .cloud
                    .open(settings, self.parts.secrets.as_ref())?;
                match store.get_file(&format!("{CLOUD_DIR}{}", backup.name), &path) {
                    Ok(()) => {}
                    Err(crate::cloud::CloudError::NotFound) => return Err(not_found()),
                    Err(error) => return Err(error.into_backup_error()),
                }
                // A missing manifest is not an error here: the encryption still protects the bundle.
                let _ = store.get_file(
                    &format!("{CLOUD_DIR}{}", manifest_name(&backup.name)),
                    &manifest_file,
                );
            }
        }
        let manifest: Option<Manifest> = fs::read(&manifest_file)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok());
        if let Some(m) = &manifest {
            let sum = hash_file(&path).map_err(|e| BackupError::io("copy", &e))?;
            if sum != m.file_blake3 {
                return Err(corrupt(
                    "the copy does not match the checksum recorded when it was made",
                ));
            }
        }
        Ok(Fetched {
            path,
            encrypted,
            manifest,
        })
    }

    fn key_from(&self, typed: Option<&str>) -> Result<BackupKey, BackupError> {
        match typed.map(str::trim).filter(|t| !t.is_empty()) {
            Some(text) => BackupKey::parse(text),
            None => self.key()?.ok_or_else(|| {
                BackupError::new(
                    BackupCode::ErrBackupKeyRequired,
                    "this computer has no backup key",
                )
                .on_field("key")
            }),
        }
    }

    /// Unlocks the copy and returns the plain database file and the manifest inside.
    fn unpack(
        &self,
        fetched: &Fetched,
        typed: Option<&str>,
        work: &Path,
    ) -> Result<(PathBuf, Option<Embedded>, Option<BackupKey>), BackupError> {
        if !fetched.encrypted {
            return Ok((fetched.path.clone(), None, None));
        }
        let key = self.key_from(typed)?;
        let plain = work.join("plain.db");
        let embedded = open(&fetched.path, &key, &plain)?;
        Ok((plain, Some(embedded), Some(key)))
    }

    /// Fetch, unlock and check. Used by the preview and the restore.
    fn open_backup(
        &self,
        settings: &BackupSettings,
        backup: &BackupRef,
        typed: Option<&str>,
        work: &Path,
    ) -> Result<Opened, BackupError> {
        let fetched = self.fetch(settings, backup, work)?;
        let (plain, embedded, key_used) = self.unpack(&fetched, typed, work)?;
        let inspected = inspect(&plain)?;
        let (created_at, expected) = match (&embedded, &fetched.manifest) {
            (Some(e), _) => (
                e.created_at.clone(),
                Some((e.schema_version, e.inner.counts.clone())),
            ),
            (None, Some(m)) => (
                m.created_at.clone(),
                m.inner
                    .as_ref()
                    .map(|i| (m.schema_version, i.counts.clone())),
            ),
            (None, None) => (
                parse_stamp(&backup.name)
                    .map(format_utc)
                    .unwrap_or_default(),
                None,
            ),
        };
        if let Some((schema, counts)) = expected
            && (schema != inspected.schema_version || counts != inspected.counts)
        {
            return Err(corrupt(
                "the database inside does not match what was recorded when it was made",
            ));
        }
        Ok(Opened {
            plain,
            size_bytes: fs::metadata(&fetched.path).map_or(inspected.size_bytes, |m| m.len()),
            inspected,
            created_at,
            key_used,
        })
    }

    fn live_totals(&self) -> Result<BackupTotals, BackupError> {
        self.db.read(read_totals)
    }

    /// What a restore would do, for the confirm box (FR-20.9). Changes nothing.
    ///
    /// # Errors
    /// `ERR_BACKUP_NOT_FOUND`, `ERR_BACKUP_KEY_REQUIRED`, `ERR_BACKUP_KEY_FORMAT`,
    /// `ERR_BACKUP_KEY_WRONG` (field `key`), `ERR_BACKUP_CORRUPT`, `ERR_BACKUP_NEWER_THAN_APP`,
    /// `ERR_BACKUP_BUSY`, `ERR_BACKUP_CLOUD_FAILED`, `ERR_BACKUP_USB_NOT_PRESENT`.
    pub fn restore_preview(
        &self,
        input: &RestorePreviewInput,
    ) -> Result<RestorePreview, BackupError> {
        let _busy = self.try_busy()?;
        let settings = self.settings()?;
        let work = self.work_dir("preview")?;
        let result = (|| {
            let opened = self.open_backup(&settings, &input.backup, input.key.as_deref(), &work)?;
            let app = self.app_schema_version();
            if opened.inspected.schema_version > app {
                return Err(BackupError::new(
                    BackupCode::ErrBackupNewerThanApp,
                    "made by a newer app",
                ));
            }
            let current = self.live_totals()?;
            let totals = opened.inspected.totals.clone();
            Ok(RestorePreview {
                backup: input.backup.clone(),
                created_at: opened.created_at.clone(),
                size_bytes: opened.size_bytes,
                schema_version: opened.inspected.schema_version,
                app_schema_version: app,
                needs_upgrade: opened.inspected.schema_version < app,
                lost: LostWork {
                    journal_entries: (current.journal_entries - totals.journal_entries).max(0),
                    bikri_entries: (current.bikri_entries - totals.bikri_entries).max(0),
                    vouchers: (current.vouchers - totals.vouchers).max(0),
                },
                totals,
                current,
            })
        })();
        let _ = fs::remove_dir_all(&work);
        result
    }

    /// A copy of the live data as it is now, in the safety folder, checked (FR-20.9).
    fn safety_copy(&self) -> Result<String, BackupError> {
        let dir = self.safety_dir();
        fs::create_dir_all(&dir).map_err(|e| write_error(BackupKind::Local, &e))?;
        let started = self.now();
        let name = local_name(&stamp(started));
        let dest = dir.join(&name);
        snapshot_copy(&self.db, &dest, self.parts.disk.as_ref())?;
        let inspected = match inspect(&dest) {
            Ok(inspected) => inspected,
            Err(error) => {
                remove_quietly(&dest);
                return Err(BackupError::new(
                    BackupCode::ErrBackupCheckFailed,
                    format!("safety copy: {error}"),
                ));
            }
        };
        let manifest = Manifest {
            format: MANIFEST_FORMAT,
            kind: BackupKind::Local,
            created_at: format_utc(started),
            app_version: self.config.app_version.clone(),
            schema_version: inspected.schema_version,
            encrypted: false,
            file: name.clone(),
            size_bytes: inspected.size_bytes,
            file_blake3: inspected.blake3.clone(),
            integrity_ok: true,
            inner: None,
        };
        write_atomic(
            &dir.join(manifest_name(&name)),
            &serde_json::to_vec_pretty(&manifest)?,
        )
        .map_err(|e| write_error(BackupKind::Local, &e))?;
        if let Ok(entries) = fs::read_dir(&dir) {
            let names: Vec<String> = entries
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| crate::names::is_copy_name(BackupKind::Local, n))
                .collect();
            for old in newest_n_delete(&names, KEEP_SAFETY) {
                remove_quietly(&dir.join(&old));
                remove_quietly(&dir.join(manifest_name(&old)));
            }
        }
        Ok(name)
    }

    /// Restores a copy over the live data (FR-20.9). The caller then throws away what the
    /// services keep in memory (`Context::invalidate_caches`), rebuilds the search index and
    /// signs the user out, because the users and sessions come from the restored data.
    ///
    /// # Errors
    /// Those of [`BackupService::restore_preview`], and `ERR_BACKUP_NOT_CONFIRMED` (field
    /// `confirmed`), `ERR_DISK_FULL`, `ERR_BACKUP_CHECK_FAILED` (the safety copy), and
    /// `ERR_BACKUP_RESTORE_FAILED`. If anything fails, the data is as it was.
    pub fn restore(
        &self,
        user_id: Option<i64>,
        input: &RestoreInput,
    ) -> Result<RestoreDone, BackupError> {
        if !input.confirmed {
            return Err(
                BackupError::new(BackupCode::ErrBackupNotConfirmed, "not confirmed")
                    .on_field("confirmed"),
            );
        }
        let _busy = self.try_busy()?;
        let settings = self.settings()?;
        let work = self.work_dir("restore")?;
        let result = self.restore_in(&work, &settings, user_id, input);
        let _ = fs::remove_dir_all(&work);
        result
    }

    fn restore_in(
        &self,
        work: &Path,
        settings: &BackupSettings,
        user_id: Option<i64>,
        input: &RestoreInput,
    ) -> Result<RestoreDone, BackupError> {
        let opened = self.open_backup(settings, &input.backup, input.key.as_deref(), work)?;
        let app = self.app_schema_version();
        let from = opened.inspected.schema_version;
        if from > app {
            return Err(BackupError::new(
                BackupCode::ErrBackupNewerThanApp,
                "made by a newer app",
            ));
        }
        require_room(
            self.parts.disk.as_ref(),
            work,
            opened
                .inspected
                .size_bytes
                .saturating_mul(2)
                .saturating_add(ROOM_MARGIN),
        )?;
        // The safety copy comes before anything is changed. If it fails, nothing was touched.
        let safety_name = self.safety_copy()?;

        let mut plain = opened.plain.clone();
        let mut totals = opened.inspected.totals.clone();
        if from < app {
            let mut conn = Connection::open(&plain)?;
            migrate::migrate(
                &mut conn,
                &work.join("pre-migration"),
                self.db.clock().as_ref(),
            )
            .map_err(|e| BackupError::new(BackupCode::ErrMigrationFailed, e))?;
            let _: String =
                conn.pragma_update_and_check(None, "journal_mode", "DELETE", |row| row.get(0))?;
            drop(conn);
            let upgraded = inspect(&plain).map_err(|e| {
                BackupError::new(
                    BackupCode::ErrMigrationFailed,
                    format!("after the upgrade: {e}"),
                )
            })?;
            totals = upgraded.totals;
            plain = opened.plain.clone();
        }

        self.db
            .replace_contents_from(&plain)
            .map_err(|e| BackupError::new(BackupCode::ErrBackupRestoreFailed, e))?;
        // The data file must now show the books of the copy. If not, the safety copy goes back.
        let after = self.live_totals();
        if !matches!(&after, Ok(now) if *now == totals) {
            let back = self.safety_dir().join(&safety_name);
            if let Err(error) = self.db.replace_contents_from(&back) {
                tracing::error!("cannot put the safety copy back: {error}");
            }
            return Err(BackupError::new(
                BackupCode::ErrBackupRestoreFailed,
                "the books after the restore differ from the copy",
            ));
        }

        // A new computer: keep the typed key, so the next copies are locked with it (NFR-REL-4).
        if let Some(key) = &opened.key_used
            && self.parts.secrets.get(SecretName::BackupKey)?.is_none()
        {
            self.parts.secrets.set(SecretName::BackupKey, key.raw())?;
            let at = self.now_text();
            self.update_state(|s| {
                s.key_created_at = Some(at.clone());
                s.key_confirmed_at = Some(at);
            });
        }
        self.mark_data_seen();
        let actor = self.db.read(|conn| -> Result<Option<i64>, BackupError> {
            Ok(user_id.and_then(|id| {
                conn.query_row("SELECT id FROM users WHERE id = ?1", [id], |row| row.get(0))
                    .optional()
                    .ok()
                    .flatten()
            }))
        })?;
        self.audit(
            actor,
            "restore",
            "backup",
            Some(json!({ "restored_from": opened.created_at, "kind": input.backup.kind.as_str() })),
        );
        Ok(RestoreDone {
            restored_from: opened.created_at,
            safety_copy: safety_name,
            totals: self.live_totals()?,
            upgraded_from: (from < app).then_some(from),
        })
    }

    /// Restores a copy into a separate scratch place and compares it with what was recorded
    /// when it was made (NFR-BAK-11). The live data is not touched.
    ///
    /// # Errors
    /// `ERR_BACKUP_NOT_FOUND` (no copy to test, or the name is wrong), `ERR_BACKUP_KEY_*`,
    /// `ERR_BACKUP_CLOUD_FAILED`, `ERR_BACKUP_BUSY`. A damaged copy is not an error: it is a
    /// result with `ok = false` and the failed check.
    pub fn test_restore(&self, input: &TestRestoreInput) -> Result<TestRestoreResult, BackupError> {
        let _busy = self.try_busy()?;
        let settings = self.settings()?;
        let backup = match &input.backup {
            Some(backup) => backup.clone(),
            None => self.latest_for_test()?,
        };
        check_name(backup.kind, &backup.name)?;
        let work = self.work_dir("test")?;
        let result = self.test_in(&work, &settings, &backup, input.key.as_deref());
        let _ = fs::remove_dir_all(&work);
        if let Ok(done) = &result {
            let at = self.now_text();
            let ok = done.ok;
            self.update_state(|s| {
                s.last_test_restore_at = Some(at);
                s.last_test_restore_ok = Some(ok);
            });
        }
        result
    }

    fn latest_for_test(&self) -> Result<BackupRef, BackupError> {
        let all = self.list(&crate::api::BackupListInput {
            kind: None,
            offset: 0,
            limit: 1000,
        })?;
        for kind in [BackupKind::Cloud, BackupKind::Usb, BackupKind::Local] {
            if let Some(row) = all
                .rows
                .iter()
                .find(|r| r.kind == kind && r.status != RowStatus::Bad)
            {
                return Ok(BackupRef {
                    kind,
                    name: row.name.clone(),
                });
            }
        }
        Err(BackupError::new(
            BackupCode::ErrBackupNotFound,
            "no backup to test",
        ))
    }

    fn test_in(
        &self,
        work: &Path,
        settings: &BackupSettings,
        backup: &BackupRef,
        typed: Option<&str>,
    ) -> Result<TestRestoreResult, BackupError> {
        let live = self.live_totals()?;
        let mut checks = Vec::new();
        let finish = |checks: &mut Vec<TestCheck>,
                      upto: usize,
                      created_at: String,
                      restored: Option<BackupTotals>| {
            let all = [
                TestCheckName::Opens,
                TestCheckName::Checksum,
                TestCheckName::Integrity,
                TestCheckName::Counts,
                TestCheckName::Balanced,
            ];
            for name in all.into_iter().skip(upto) {
                checks.push(TestCheck { name, ok: false });
            }
            let ok = checks.iter().all(|c| c.ok);
            TestRestoreResult {
                backup: backup.clone(),
                created_at,
                ok,
                checks: checks.clone(),
                restored: restored.unwrap_or_else(|| live.clone()),
                live: live.clone(),
            }
        };
        // Checksum first: it needs only the file and its manifest.
        let fetched = match self.fetch(settings, backup, work) {
            Ok(f) => f,
            Err(e) if e.code == BackupCode::ErrBackupCorrupt => {
                // The checksum failed; the copy cannot be opened with confidence.
                checks.push(TestCheck {
                    name: TestCheckName::Opens,
                    ok: false,
                });
                checks.push(TestCheck {
                    name: TestCheckName::Checksum,
                    ok: false,
                });
                return Ok(finish(&mut checks, 2, String::new(), None));
            }
            Err(e) => return Err(e),
        };
        let (plain, embedded, _) = match self.unpack(&fetched, typed, work) {
            Ok(v) => v,
            Err(e) if e.code == BackupCode::ErrBackupCorrupt => {
                checks.push(TestCheck {
                    name: TestCheckName::Opens,
                    ok: false,
                });
                return Ok(finish(&mut checks, 1, String::new(), None));
            }
            Err(e) => return Err(e),
        };
        checks.push(TestCheck {
            name: TestCheckName::Opens,
            ok: true,
        });
        checks.push(TestCheck {
            name: TestCheckName::Checksum,
            ok: true,
        });
        let created_at = embedded
            .as_ref()
            .map(|e| e.created_at.clone())
            .or_else(|| fetched.manifest.as_ref().map(|m| m.created_at.clone()))
            .unwrap_or_default();
        let inspected = match inspect(&plain) {
            Ok(i) => i,
            Err(_) => {
                checks.push(TestCheck {
                    name: TestCheckName::Integrity,
                    ok: false,
                });
                return Ok(finish(&mut checks, 3, created_at, None));
            }
        };
        checks.push(TestCheck {
            name: TestCheckName::Integrity,
            ok: true,
        });
        let expected = embedded.as_ref().map(|e| &e.inner.counts).or_else(|| {
            fetched
                .manifest
                .as_ref()
                .and_then(|m| m.inner.as_ref())
                .map(|i| &i.counts)
        });
        // A plain copy without a manifest has nothing to compare with; it counts as matching.
        let counts_ok = expected.is_none_or(|counts| *counts == inspected.counts);
        checks.push(TestCheck {
            name: TestCheckName::Counts,
            ok: counts_ok,
        });
        let balanced = inspected.totals.debit_paisa == inspected.totals.credit_paisa;
        checks.push(TestCheck {
            name: TestCheckName::Balanced,
            ok: balanced,
        });
        Ok(finish(&mut checks, 5, created_at, Some(inspected.totals)))
    }
}
