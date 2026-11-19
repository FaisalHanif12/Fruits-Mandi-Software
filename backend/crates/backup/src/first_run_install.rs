//! Putting a prepared copy in place on a new computer (PRD FR-20.11, plan 56).
//!
//! The order is fixed so that a failure at any step leaves this computer a fresh install: the
//! keys first (the credential store is the likeliest to refuse), then the data file in one
//! step, then the settings. Whatever was done is undone if a later step fails.

use std::path::Path;

use mandi_db::audit::AuditKey;
use mandi_db::rusqlite::OptionalExtension;
use serde_json::json;

use crate::error::{BackupCode, BackupError};
use crate::first_run::{Prepared, ProgressSink, report};
use crate::first_run_api::{FirstRunRestoreDone, RestoreStage};
use crate::first_run_source::Checked;
use crate::io::require_room;
use crate::key::BackupKey;
use crate::secrets::SecretName;
use crate::service::BackupService;
use crate::settings;
use crate::snapshot::{ROOM_MARGIN, read_totals, snapshot_copy};

/// The `settings.key` of the assistant switch (`services::assistant::KEY_ENABLED`). Read only.
const ASSISTANT_ENABLED_KEY: &str = "assistant_enabled";

/// Why the swap of the data file did not happen: the guard said no, or the database failed.
enum Swap {
    Refused(BackupError),
    Db(mandi_db::DbError),
}

impl From<mandi_db::DbError> for Swap {
    fn from(error: mandi_db::DbError) -> Self {
        Swap::Db(error)
    }
}

/// What a failed restore has to undo.
#[derive(Default)]
struct Undo {
    /// The secrets that were written, with what they held before (null: nothing).
    secrets: Vec<(SecretName, Option<String>)>,
    /// The chain key that was replaced.
    previous_chain: Option<AuditKey>,
    /// True once the data file holds the copy.
    data_replaced: bool,
}

impl BackupService {
    /// Puts a prepared copy in place. Everything that can fail runs before the data file is
    /// replaced, or is undone if it fails after.
    pub(crate) fn install(
        &self,
        checked: &Checked,
        prepared: &Prepared,
        key: &BackupKey,
        work: &Path,
        progress: ProgressSink<'_>,
    ) -> Result<FirstRunRestoreDone, BackupError> {
        let plain = &prepared.opened.plain;
        require_room(
            self.parts.disk.as_ref(),
            work,
            prepared
                .opened
                .inspected
                .size_bytes
                .saturating_mul(2)
                .saturating_add(ROOM_MARGIN),
        )?;
        // The fresh data file, to go back to if anything below fails.
        let fresh = work.join("fresh.db");
        snapshot_copy(&self.db, &fresh, self.parts.disk.as_ref())?;

        let mut undo = Undo::default();
        let done = self.install_steps(checked, prepared, key, plain, progress, &mut undo);
        if done.is_err() {
            self.undo(&undo, &fresh);
        }
        done
    }

    fn install_steps(
        &self,
        checked: &Checked,
        prepared: &Prepared,
        key: &BackupKey,
        plain: &Path,
        progress: ProgressSink<'_>,
        undo: &mut Undo,
    ) -> Result<FirstRunRestoreDone, BackupError> {
        // 1. The keys. The credential store is the likeliest thing to refuse (locked, denied),
        // so it comes before the data file is touched.
        report(progress, RestoreStage::Keys, None);
        self.keep_secret(undo, SecretName::BackupKey, key.raw())?;
        if let Checked::Cloud(account) = checked {
            self.keep_secret(undo, SecretName::CloudAccessKeyId, &account.key_id)?;
            self.keep_secret(undo, SecretName::CloudSecretAccessKey, &account.secret)?;
        }

        // 2. The data file: replaced in one step, under the write lock. In the same moment, under
        // the same lock: no Owner may have been set up meanwhile (the first-run wizard can run
        // while a restore downloads), and the chain key of the copy is saved. A wizard that
        // finishes first wins and the restore says no; one that comes later finds an Owner and
        // says no. The shop that was reported as set up is never erased without a word.
        report(progress, RestoreStage::PuttingInPlace, None);
        let mut previous_chain = None;
        let replaced = self.db.replace_contents_if(plain, |conn| {
            let users: i64 = conn
                .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
                .map_err(|e| Swap::Db(e.into()))?;
            if users > 0 {
                return Err(Swap::Refused(BackupError::new(
                    BackupCode::ErrAlreadySetUp,
                    "an Owner was set up while the restore ran",
                )));
            }
            previous_chain = self
                .install_chain_key(&prepared.chain_key)
                .map_err(Swap::Refused)?;
            Ok(())
        });
        undo.previous_chain = previous_chain;
        let chain_key_restored = undo.previous_chain.is_some();
        replaced.map_err(|e| match e {
            Swap::Refused(error) => error,
            Swap::Db(error) => BackupError::new(BackupCode::ErrBackupRestoreFailed, error),
        })?;
        undo.data_replaced = true;
        let now = self.db.read(read_totals)?;
        if now != prepared.totals {
            return Err(BackupError::new(
                BackupCode::ErrBackupRestoreFailed,
                "the books after the restore differ from the copy",
            ));
        }

        // 3. The settings of the old computer, made right for this one.
        self.adjust_settings(checked)?;

        // 4. Done. What follows cannot undo the restore: a failure here is only logged.
        let at = self.now_text();
        self.update_state(|s| {
            s.key_created_at = Some(at.clone());
            s.key_confirmed_at = Some(at);
        });
        self.mark_data_seen();
        self.audit(
            None,
            "restore",
            "backup",
            Some(json!({
                "restored_from": prepared.opened.created_at,
                "kind": prepared.copy.kind.as_str(),
                "new_computer": true,
            })),
        );
        Ok(FirstRunRestoreDone {
            restored_from: prepared.opened.created_at.clone(),
            totals: self.db.read(read_totals)?,
            upgraded_from: (prepared.from_schema < prepared.app_schema)
                .then_some(prepared.from_schema),
            chain_key_restored,
            cloud_needs_keys: self.cloud_needs_keys()?,
            assistant_was_on: self.assistant_was_on()?,
        })
    }

    /// Saves a secret and remembers what it held, so a failure can put it back.
    fn keep_secret(
        &self,
        undo: &mut Undo,
        name: SecretName,
        value: &str,
    ) -> Result<(), BackupError> {
        let old = self.parts.secrets.get(name)?;
        self.parts.secrets.set(name, value)?;
        undo.secrets.push((name, old));
        Ok(())
    }

    /// Puts this computer back as it was: the fresh data file, every secret as it was, the old
    /// chain key. Each step is tried even if an earlier one fails; a failure is logged.
    fn undo(&self, undo: &Undo, fresh: &Path) {
        if undo.data_replaced
            && let Err(error) = self.db.replace_contents_from(fresh)
        {
            tracing::error!("cannot put the fresh data file back: {error}");
        }
        for (name, old) in undo.secrets.iter().rev() {
            let put_back = match old {
                Some(value) => self.parts.secrets.set(*name, value),
                None => self.parts.secrets.delete(*name),
            };
            if let Err(error) = put_back {
                tracing::error!("cannot put a stored secret back: {error}");
            }
        }
        self.put_chain_key_back(undo.previous_chain.clone());
    }

    /// The backup settings of the old computer, made right for this one: a cloud source sets the
    /// cloud account the Owner typed, and a local folder that is not here goes back to the
    /// default. The USB folder and the cloud switch are kept (plan 56 decisions).
    fn adjust_settings(&self, checked: &Checked) -> Result<(), BackupError> {
        let before = self.settings()?;
        let mut now = before.clone();
        if let Checked::Cloud(account) = checked {
            now.cloud_enabled = true;
            now.cloud_endpoint.clone_from(&account.endpoint);
            now.cloud_bucket.clone_from(&account.bucket);
            now.cloud_region.clone_from(&account.region);
            now.cloud_prefix.clone_from(&account.prefix);
        }
        if !now.local_dir.trim().is_empty() && !Path::new(&now.local_dir).exists() {
            now.local_dir.clear();
        }
        if now != before {
            settings::save(&self.db, None, &now)?;
        }
        Ok(())
    }

    /// True when the cloud copy is on and this computer has no cloud keys.
    fn cloud_needs_keys(&self) -> Result<bool, BackupError> {
        let settings = self.settings()?;
        let has = |name| matches!(self.parts.secrets.get(name), Ok(Some(v)) if !v.is_empty());
        Ok(settings.cloud_enabled
            && !(has(SecretName::CloudAccessKeyId) && has(SecretName::CloudSecretAccessKey)))
    }

    /// True when the assistant was switched on in the restored settings. Read only: the
    /// assistant's own code is not touched.
    fn assistant_was_on(&self) -> Result<bool, BackupError> {
        let value: Option<String> = self.db.read(|conn| {
            conn.query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                [ASSISTANT_ENABLED_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(BackupError::from)
        })?;
        Ok(value.is_some_and(|text| text.trim() == "true"))
    }
}
