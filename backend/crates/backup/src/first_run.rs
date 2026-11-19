//! Restore on a new computer, from the first screen (PRD FR-20.11, NFR-INS-6, NFR-REL-4,
//! plan 56).
//!
//! Three steps, each its own call, none of them keeping anything between calls: list the
//! copies, preview one (its date, its books and its checks), restore it. All three are allowed
//! only while no Owner exists on this computer, as for `setup_first_run`.
//!
//! One restore is one safe operation. Everything is unpacked, upgraded and checked in a scratch
//! folder first, including both hash chains (NFR-SEC-6). Only then are the keys saved, the data
//! file replaced in one step and the settings adjusted. If any of that fails, the data file
//! goes back to the fresh one that was there, every key goes back to what it was, and the
//! scratch folder is deleted. The computer is then still a fresh install.
//!
//! No key is ever written to a log or a file: the inputs print `[hidden]`, and the errors carry
//! a fixed text, not what the cloud answered.

use std::fs;
use std::path::Path;

use mandi_core::error::ErrorValue;
use mandi_db::audit::AuditKey;

use crate::api::{BackupRef, BackupRow, RowStatus};
use crate::cloud::{CloudAccount, CloudStore};
use crate::error::{BackupCode, BackupError};
use crate::first_run_api::{
    FirstRunCheck, FirstRunCheckName, FirstRunCopies, FirstRunListInput, FirstRunPreview,
    FirstRunPreviewInput, FirstRunRestoreDone, FirstRunRestoreInput, RestoreProgress, RestoreStage,
};
use crate::first_run_source::{
    Checked, F_BUCKET, F_COPY, F_FOLDER, check_text, cloud_failure, file_row, named, not_found,
    not_there, on_copy, percent_of, validate,
};
use crate::io::{copy_atomic, require_room, write_error};
use crate::key::BackupKey;
use crate::model::{BackupKind, BackupTotals};
use crate::names::{check_name, manifest_name};
use crate::restore::{Fetched, Opened, copy_has_owner, finish_fetch, no_owner};
use crate::service::{BackupService, CLOUD_DIR, cloud_rows_from, scan_folder};
use crate::snapshot::ROOM_MARGIN;

/// Reports where a restore is. The command sends each report to the screen as an event.
pub type ProgressSink<'a> = &'a dyn Fn(RestoreProgress);

pub(crate) fn report(progress: ProgressSink<'_>, stage: RestoreStage, percent: Option<u8>) {
    progress(RestoreProgress { stage, percent });
}

/// A copy opened, upgraded and checked in the scratch folder, ready to preview or install.
pub(crate) struct Prepared {
    pub(crate) copy: BackupRef,
    pub(crate) opened: Opened,
    pub(crate) from_schema: i64,
    pub(crate) app_schema: i64,
    /// The books after the upgrade.
    pub(crate) totals: BackupTotals,
    pub(crate) chain_key: AuditKey,
}

impl BackupService {
    /// True while no Owner exists. The restore on a new computer is allowed only then.
    pub(crate) fn require_first_run(&self) -> Result<(), BackupError> {
        let users: i64 = self.db.read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
                .map_err(BackupError::from)
        })?;
        if users > 0 {
            return Err(BackupError::new(
                BackupCode::ErrAlreadySetUp,
                "an Owner exists on this computer",
            ));
        }
        Ok(())
    }

    fn store_for(
        &self,
        account: &CloudAccount,
    ) -> Result<std::sync::Arc<dyn CloudStore>, BackupError> {
        self.parts.cloud.open_account(account)
    }

    /// The copies in a source, newest first.
    fn first_run_rows(&self, checked: &Checked) -> Result<Vec<BackupRow>, BackupError> {
        let mut rows = match checked {
            Checked::Cloud(account) => {
                let store = self.store_for(account)?;
                cloud_rows_from(store.as_ref()).map_err(|e| cloud_failure(&e, F_BUCKET))?
            }
            Checked::Folder(dir) => {
                if !dir.is_dir() {
                    return Err(not_there(F_FOLDER));
                }
                scan_folder(BackupKind::Usb, dir)
            }
            Checked::File(path) => vec![file_row(path)?],
        };
        rows.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.name.cmp(&a.name))
        });
        Ok(rows)
    }

    /// Lists the copies in a source, newest first. An empty place gives an empty list.
    ///
    /// # Errors
    /// `ERR_ALREADY_SET_UP`; the field errors of the source; `ERR_BACKUP_CLOUD_FAILED` (value
    /// `reason`) on the cloud fields; `ERR_BACKUP_USB_NOT_PRESENT` on `source.folder` or
    /// `source.file`; `ERR_BACKUP_BUSY`.
    pub fn first_run_list(&self, input: &FirstRunListInput) -> Result<FirstRunCopies, BackupError> {
        self.require_first_run()?;
        let (checked, _) = validate(&input.source, None)?;
        let _busy = self.try_busy()?;
        let rows = self.first_run_rows(&checked)?;
        let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        Ok(FirstRunCopies { rows, total_count })
    }

    /// Names the copy to open. A name from the screen is checked (it can never point outside the
    /// folder); no name means the newest copy that is not marked bad.
    fn resolve_copy(
        &self,
        checked: &Checked,
        copy: Option<&str>,
    ) -> Result<BackupRef, BackupError> {
        let kind = checked.kind();
        if let Checked::File(path) = checked {
            // The file itself is the copy; a name sent with it is ignored.
            return Ok(BackupRef {
                kind,
                name: file_row(path)?.name,
            });
        }
        if let Some(name) = copy.map(str::trim).filter(|n| !n.is_empty()) {
            check_name(kind, name).map_err(|mut e| {
                e.field = Some(F_COPY.to_owned());
                e
            })?;
            return Ok(BackupRef {
                kind,
                name: name.to_owned(),
            });
        }
        self.first_run_rows(checked)?
            .into_iter()
            .find(|row| row.status != RowStatus::Bad)
            .map(|row| BackupRef {
                kind,
                name: row.name,
            })
            .ok_or_else(not_found)
    }

    /// Brings the copy into `work` and checks its checksum.
    fn fetch_new(
        &self,
        checked: &Checked,
        copy: &BackupRef,
        work: &Path,
        progress: ProgressSink<'_>,
    ) -> Result<Fetched, BackupError> {
        let path = work.join("copy.mbk");
        let manifest_file = work.join("manifest.json");
        report(progress, RestoreStage::Downloading, Some(0));
        match checked {
            Checked::Cloud(account) => {
                let store = self.store_for(account)?;
                // Room for the download, before the first byte comes (NFR-REL-10). The size is in
                // the listing; a failed listing is left to the download, which says why.
                let wanted = format!("{CLOUD_DIR}{}", copy.name);
                let size = store
                    .list(CLOUD_DIR)
                    .ok()
                    .and_then(|objects| objects.into_iter().find(|o| o.key == wanted))
                    .map(|o| o.size);
                if let Some(size) = size {
                    require_room(
                        self.parts.disk.as_ref(),
                        work,
                        size.saturating_add(ROOM_MARGIN),
                    )?;
                }
                let mut last = 0_u8;
                let got = store.get_file_progress(
                    &format!("{CLOUD_DIR}{}", copy.name),
                    &path,
                    &mut |done, total| {
                        if let Some(now) = percent_of(done, total)
                            && now != last
                        {
                            last = now;
                            report(progress, RestoreStage::Downloading, Some(now));
                        }
                    },
                );
                got.map_err(|e| cloud_failure(&e, F_COPY))?;
                // A missing manifest is not an error: the encryption still protects the bundle.
                let _ = store.get_file(
                    &format!("{CLOUD_DIR}{}", manifest_name(&copy.name)),
                    &manifest_file,
                );
            }
            Checked::Folder(dir) => {
                if !dir.is_dir() {
                    return Err(not_there(F_FOLDER));
                }
                self.copy_local_source(&dir.join(&copy.name), &path)?;
                let _ = fs::copy(dir.join(manifest_name(&copy.name)), &manifest_file);
            }
            Checked::File(file) => {
                self.copy_local_source(file, &path)?;
                if let Some(dir) = file.parent() {
                    let _ = fs::copy(dir.join(manifest_name(&copy.name)), &manifest_file);
                }
            }
        }
        report(progress, RestoreStage::Downloading, Some(100));
        finish_fetch(path, &manifest_file, true).map_err(on_copy)
    }

    fn copy_local_source(&self, source: &Path, dest: &Path) -> Result<(), BackupError> {
        if !source.is_file() {
            return Err(not_found());
        }
        // Room for the copy, before the first byte is written (NFR-REL-10).
        if let (Some(dir), Ok(meta)) = (dest.parent(), fs::metadata(source)) {
            require_room(
                self.parts.disk.as_ref(),
                dir,
                meta.len().saturating_add(ROOM_MARGIN),
            )?;
        }
        copy_atomic(source, dest, self.parts.hook.as_ref(), BackupKind::Local)
            .map(|_| ())
            .map_err(|e| write_error(BackupKind::Usb, &e))
    }

    /// Fetches, unlocks, checks and upgrades the copy in `work`, and verifies both chains. This
    /// is everything that can be learned without changing this computer.
    fn prepare(
        &self,
        checked: &Checked,
        copy: Option<&str>,
        key: &BackupKey,
        work: &Path,
        progress: ProgressSink<'_>,
    ) -> Result<Prepared, BackupError> {
        let copy = self.resolve_copy(checked, copy)?;
        let fetched = self.fetch_new(checked, &copy, work, progress)?;

        report(progress, RestoreStage::Decrypting, None);
        let (plain, embedded, key_used) = self
            .unpack(&fetched, Some(&key.display()), work)
            .map_err(|e| named(on_copy(e), FirstRunCheckName::Opens))?;

        report(progress, RestoreStage::Checking, None);
        let opened = Self::check_unpacked(&fetched, &copy.name, plain, embedded, key_used)
            .map_err(on_copy)?;
        let app_schema = self.app_schema_version();
        let from_schema = opened.inspected.schema_version;
        if from_schema > app_schema {
            return Err(
                BackupError::new(BackupCode::ErrBackupNewerThanApp, "made by a newer app")
                    .on_field(F_COPY),
            );
        }
        let totals = if from_schema < app_schema {
            report(progress, RestoreStage::Upgrading, None);
            require_room(
                self.parts.disk.as_ref(),
                work,
                opened
                    .inspected
                    .size_bytes
                    .saturating_mul(2)
                    .saturating_add(ROOM_MARGIN),
            )?;
            self.upgrade_copy(&opened.plain, work)?
        } else {
            opened.inspected.totals.clone()
        };
        if totals.debit_paisa != totals.credit_paisa {
            return Err(BackupError::new(
                BackupCode::ErrBackupCorrupt,
                "the books in the copy do not balance",
            )
            .on_field(F_COPY)
            .with(
                "check",
                ErrorValue::Text(check_text(FirstRunCheckName::Balanced).to_owned()),
            ));
        }
        if !copy_has_owner(&opened.plain)? {
            return Err(no_owner(F_COPY));
        }
        report(progress, RestoreStage::Checking, None);
        let chain_key = self.pick_chain_key(&opened.plain, opened.chain_key.as_deref(), F_COPY)?;
        Ok(Prepared {
            copy,
            opened,
            from_schema,
            app_schema,
            totals,
            chain_key,
        })
    }

    /// What a restore would do: the copy, its date, its books and its checks. Changes nothing on
    /// this computer; the scratch folder is deleted.
    ///
    /// # Errors
    /// `ERR_ALREADY_SET_UP`; the field errors of the source and the key; the cloud, file and
    /// copy errors of [`BackupService::first_run_list`]; `ERR_BACKUP_NOT_FOUND`,
    /// `ERR_BACKUP_KEY_WRONG` (field `key`), `ERR_BACKUP_CORRUPT` (value `check`),
    /// `ERR_BACKUP_NEWER_THAN_APP`, `ERR_BACKUP_CHAIN_KEY_MISSING` (all on `copy`),
    /// `ERR_DISK_FULL`, `ERR_MIGRATION_FAILED`, `ERR_BACKUP_BUSY`.
    pub fn first_run_preview(
        &self,
        input: &FirstRunPreviewInput,
        progress: ProgressSink<'_>,
    ) -> Result<FirstRunPreview, BackupError> {
        self.require_first_run()?;
        let (checked, key) = validate(&input.source, Some(&input.key))?;
        let key = key.ok_or_else(|| BackupError::internal("no key after validation"))?;
        let _busy = self.try_busy()?;
        let work = self.work_dir("firstrun-preview")?;
        let result = self
            .prepare(&checked, input.copy.as_deref(), &key, &work, progress)
            .map(|prepared| FirstRunPreview {
                copy: prepared.copy.clone(),
                created_at: prepared.opened.created_at.clone(),
                size_bytes: prepared.opened.size_bytes,
                schema_version: prepared.from_schema,
                app_schema_version: prepared.app_schema,
                needs_upgrade: prepared.from_schema < prepared.app_schema,
                totals: prepared.totals.clone(),
                checks: all_checks_passed(),
            });
        let _ = fs::remove_dir_all(&work);
        result
    }

    /// Restores a copy on this new computer. See the module text for the order and for what
    /// happens when a step fails.
    ///
    /// # Errors
    /// Those of [`BackupService::first_run_preview`], and `ERR_BACKUP_RESTORE_FAILED`. If
    /// anything fails, this computer is still a fresh install.
    pub fn first_run_restore(
        &self,
        input: &FirstRunRestoreInput,
        progress: ProgressSink<'_>,
    ) -> Result<FirstRunRestoreDone, BackupError> {
        self.require_first_run()?;
        let (checked, key) = validate(&input.source, Some(&input.key))?;
        let key = key.ok_or_else(|| BackupError::internal("no key after validation"))?;
        let _busy = self.try_busy()?;
        let work = self.work_dir("firstrun-restore")?;
        let result = self
            .prepare(&checked, Some(&input.copy), &key, &work, progress)
            .and_then(|prepared| self.install(&checked, &prepared, &key, &work, progress));
        let _ = fs::remove_dir_all(&work);
        result
    }
}

fn all_checks_passed() -> Vec<FirstRunCheck> {
    [
        FirstRunCheckName::Opens,
        FirstRunCheckName::Checksum,
        FirstRunCheckName::Integrity,
        FirstRunCheckName::Counts,
        FirstRunCheckName::Balanced,
        FirstRunCheckName::Chains,
    ]
    .into_iter()
    .map(|name| FirstRunCheck { name, ok: true })
    .collect()
}
