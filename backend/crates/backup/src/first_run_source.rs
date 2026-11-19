//! The source of a restore on a new computer (PRD FR-20.11, plan 56): the checks of the fields
//! the Owner typed, the words for a cloud that said no, and the row of a file picked by hand.
//!
//! Every problem of the input is returned together, each on the field of the input it belongs
//! to (`source.cloud.endpoint`, `source.folder`, `key`, `copy` ...). The cloud's own words are
//! never kept in an error: they can hold a key id (NFR-OBS-2).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use mandi_core::error::ErrorValue;
use mandi_db::clock::format_utc;

use crate::api::{BackupRow, RowStatus};
use crate::cloud::{CloudAccount, CloudError};
use crate::error::{BackupCode, BackupError};
use crate::first_run_api::{CloudAccountInput, FirstRunCheckName, RestoreSource};
use crate::key::BackupKey;
use crate::model::{BackupKind, Manifest};
use crate::names::{manifest_name, parse_stamp};
use crate::settings::BackupSettings;

/// The longest path accepted from the screen.
const PATH_LIMIT: usize = 1024;
/// The longest cloud key accepted from the screen.
const CLOUD_KEY_LIMIT: usize = 200;

/// A source after the checks of its fields.
pub(crate) enum Checked {
    Cloud(CloudAccount),
    Folder(PathBuf),
    File(PathBuf),
}

impl Checked {
    /// The kind of the copies it holds: cloud copies are `cloud`, folder and file copies `usb`.
    pub(crate) fn kind(&self) -> BackupKind {
        match self {
            Checked::Cloud(_) => BackupKind::Cloud,
            Checked::Folder(_) | Checked::File(_) => BackupKind::Usb,
        }
    }
}

pub(crate) const F_ENDPOINT: &str = "source.cloud.endpoint";
pub(crate) const F_BUCKET: &str = "source.cloud.bucket";
pub(crate) const F_KEY_ID: &str = "source.cloud.key_id";
pub(crate) const F_SECRET: &str = "source.cloud.secret";
pub(crate) const F_FOLDER: &str = "source.folder";
pub(crate) const F_FILE: &str = "source.file";
pub(crate) const F_COPY: &str = "copy";

pub(crate) fn required(field: &str) -> BackupError {
    BackupError::new(BackupCode::ErrRequired, "required").on_field(field)
}

pub(crate) fn out_of_range(field: &str, detail: &str) -> BackupError {
    BackupError::new(BackupCode::ErrOutOfRange, detail).on_field(field)
}

pub(crate) fn not_there(field: &str) -> BackupError {
    BackupError::new(
        BackupCode::ErrBackupUsbNotPresent,
        "the drive, folder or file is not there",
    )
    .on_field(field)
}

pub(crate) fn not_found() -> BackupError {
    BackupError::new(BackupCode::ErrBackupNotFound, "no such backup").on_field(F_COPY)
}

/// A cloud failure for the first screen. The text the cloud answered is NOT kept: it can carry
/// the access key id (NFR-OBS-2). `missing` is the field a "not found" points at: the bucket
/// while connecting, the copy while fetching one.
pub(crate) fn cloud_failure(error: &CloudError, missing: &str) -> BackupError {
    let failed = |reason: CloudError, detail: &str, field: &str| {
        reason
            .into_backup_error()
            .with_detail(detail)
            .on_field(field)
    };
    match error {
        // The cloud cannot say which of the two keys is wrong, so both fields are marked.
        CloudError::Refused(_) => BackupError::from_many(vec![
            failed(
                CloudError::Refused(String::new()),
                "cloud refused",
                F_KEY_ID,
            ),
            failed(
                CloudError::Refused(String::new()),
                "cloud refused",
                F_SECRET,
            ),
        ])
        .unwrap_or_else(|| BackupError::internal("cloud refused")),
        CloudError::NotFound if missing == F_COPY => not_found(),
        CloudError::NotFound => failed(CloudError::NotFound, "cloud: no such bucket", missing),
        CloudError::Offline(_) => failed(
            CloudError::Offline(String::new()),
            "cloud: cannot be reached",
            F_ENDPOINT,
        ),
    }
}

/// Gives an error the field `copy` when it has none.
pub(crate) fn on_copy(mut error: BackupError) -> BackupError {
    if error.field.is_none() {
        error.field = Some(F_COPY.to_owned());
    }
    error
}

/// Names the check a corrupt-copy error belongs to, when it names none yet.
pub(crate) fn named(error: BackupError, check: FirstRunCheckName) -> BackupError {
    if error.code == BackupCode::ErrBackupCorrupt && !error.values.contains_key("check") {
        return error.with("check", ErrorValue::Text(check_text(check).to_owned()));
    }
    error
}

pub(crate) fn check_text(check: FirstRunCheckName) -> &'static str {
    match check {
        FirstRunCheckName::Opens => "opens",
        FirstRunCheckName::Checksum => "checksum",
        FirstRunCheckName::Integrity => "integrity",
        FirstRunCheckName::Counts => "counts",
        FirstRunCheckName::Balanced => "balanced",
        FirstRunCheckName::Chains => "chains",
    }
}

pub(crate) fn validate_path(field: &str, text: &str) -> Result<PathBuf, BackupError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(required(field));
    }
    if text.chars().any(char::is_control) || text.len() > PATH_LIMIT {
        return Err(out_of_range(field, "path"));
    }
    let path = PathBuf::from(text);
    if !path.is_absolute() {
        return Err(out_of_range(field, "path must be a full path"));
    }
    Ok(path)
}

/// Checks the cloud fields the way Settings does, and the two keys. Every problem is returned.
pub(crate) fn validate_cloud(input: &CloudAccountInput) -> Result<CloudAccount, Vec<BackupError>> {
    let account = CloudAccount {
        endpoint: input.endpoint.trim().to_owned(),
        bucket: input.bucket.trim().to_owned(),
        region: input.region.trim().to_owned(),
        prefix: input.prefix.trim().to_owned(),
        key_id: input.key_id.trim().to_owned(),
        secret: input.secret.trim().to_owned(),
    };
    let as_settings = BackupSettings {
        cloud_enabled: true,
        cloud_endpoint: account.endpoint.clone(),
        cloud_bucket: account.bucket.clone(),
        cloud_region: account.region.clone(),
        cloud_prefix: account.prefix.clone(),
        ..BackupSettings::default()
    };
    let mut problems: Vec<BackupError> = match as_settings.validate() {
        Ok(()) => Vec::new(),
        Err(error) => error
            .all()
            .into_iter()
            .map(|mut item| {
                // `cloud_bucket` on the Settings screen is `source.cloud.bucket` here.
                if let Some(rest) = item.field.as_deref().and_then(|f| f.strip_prefix("cloud_")) {
                    item.field = Some(format!("source.cloud.{rest}"));
                }
                item
            })
            .collect(),
    };
    for (field, value) in [(F_KEY_ID, &account.key_id), (F_SECRET, &account.secret)] {
        if value.is_empty() {
            problems.push(required(field));
        } else if value.chars().any(char::is_control) || value.len() > CLOUD_KEY_LIMIT {
            problems.push(out_of_range(field, "cloud key"));
        }
    }
    if problems.is_empty() {
        Ok(account)
    } else {
        Err(problems)
    }
}

/// Checks the source, and the backup key when one is needed. All problems come together.
pub(crate) fn validate(
    source: &RestoreSource,
    key: Option<&str>,
) -> Result<(Checked, Option<BackupKey>), BackupError> {
    let mut problems = Vec::new();
    let checked = match source {
        RestoreSource::Cloud(input) => match validate_cloud(input) {
            Ok(account) => Some(Checked::Cloud(account)),
            Err(found) => {
                problems.extend(found);
                None
            }
        },
        RestoreSource::Folder(text) => match validate_path(F_FOLDER, text) {
            Ok(path) => Some(Checked::Folder(path)),
            Err(error) => {
                problems.push(error);
                None
            }
        },
        RestoreSource::File(text) => match validate_path(F_FILE, text) {
            Ok(path) if has_bundle_extension(&path) => Some(Checked::File(path)),
            Ok(_) => {
                problems.push(out_of_range(F_FILE, "a copy file ends with .mbk"));
                None
            }
            Err(error) => {
                problems.push(error);
                None
            }
        },
    };
    let key = match key {
        Some(text) => match BackupKey::parse(text.trim()) {
            Ok(key) => Some(key),
            Err(error) => {
                problems.push(error);
                None
            }
        },
        None => None,
    };
    match (BackupError::from_many(problems), checked) {
        (Some(error), _) => Err(error),
        (None, Some(checked)) => Ok((checked, key)),
        (None, None) => Err(BackupError::internal("source without a result")),
    }
}

pub(crate) fn has_bundle_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(crate::names::BUNDLE_EXT))
}

/// The row of one `.mbk` file picked by hand.
pub(crate) fn file_row(path: &Path) -> Result<BackupRow, BackupError> {
    let meta = fs::metadata(path).map_err(|_| not_there(F_FILE))?;
    if !meta.is_file() {
        return Err(not_there(F_FILE));
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| out_of_range(F_FILE, "file name"))?
        .to_owned();
    let manifest: Option<Manifest> = path
        .parent()
        .and_then(|dir| fs::read(dir.join(manifest_name(&name))).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let created_at = manifest
        .as_ref()
        .map(|m| m.created_at.clone())
        .or_else(|| parse_stamp(&name).map(format_utc))
        .or_else(|| {
            meta.modified()
                .ok()
                .map(|t| format_utc(DateTime::<Utc>::from(t)))
        })
        .unwrap_or_default();
    let status = match &manifest {
        Some(m) if m.integrity_ok => RowStatus::Good,
        Some(_) => RowStatus::Bad,
        None => RowStatus::Unchecked,
    };
    Ok(BackupRow {
        kind: BackupKind::Usb,
        name,
        created_at,
        size_bytes: meta.len(),
        encrypted: true,
        status,
    })
}

pub(crate) fn percent_of(done: u64, total: u64) -> Option<u8> {
    if total == 0 {
        return None;
    }
    let hundred = u128::from(done).saturating_mul(100) / u128::from(total);
    Some(u8::try_from(hundred.min(100)).unwrap_or(100))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_a_whole_percent_never_above_100() {
        assert_eq!(percent_of(0, 0), None);
        assert_eq!(percent_of(0, 10), Some(0));
        assert_eq!(percent_of(1, 3), Some(33));
        assert_eq!(percent_of(50, 100), Some(50));
        assert_eq!(percent_of(200, 100), Some(100));
        assert_eq!(percent_of(u64::MAX, u64::MAX), Some(100));
    }

    #[test]
    fn only_a_full_clean_path_is_accepted() {
        let field = "source.folder";
        assert_eq!(
            validate_path(field, "").unwrap_err().code,
            BackupCode::ErrRequired
        );
        for bad in [
            "relative/dir",
            "..",
            "/a\u{7}b",
            &format!("/{}", "x".repeat(1100)),
        ] {
            let error = validate_path(field, bad).unwrap_err();
            assert_eq!(error.code, BackupCode::ErrOutOfRange, "{bad}");
            assert_eq!(error.field.as_deref(), Some(field));
        }
        assert!(validate_path(field, "  /mnt/usb/MandiBackup  ").is_ok());
    }

    #[test]
    fn a_copy_file_ends_with_mbk_in_any_case() {
        assert!(has_bundle_extension(Path::new("/x/mandi-1.mbk")));
        assert!(has_bundle_extension(Path::new("/x/MANDI-1.MBK")));
        assert!(!has_bundle_extension(Path::new("/x/mandi-1.db")));
        assert!(!has_bundle_extension(Path::new("/x/mbk")));
    }

    #[test]
    fn a_cloud_failure_names_fields_and_keeps_the_cloud_words_out_of_the_log() {
        let refused = cloud_failure(&CloudError::Refused("AKIASECRETID".into()), F_BUCKET);
        let fields: Vec<_> = refused.all().into_iter().map(|e| e.field).collect();
        assert_eq!(
            fields,
            [Some(F_KEY_ID.to_owned()), Some(F_SECRET.to_owned())]
        );
        let offline = cloud_failure(&CloudError::Offline("AKIASECRETID".into()), F_BUCKET);
        assert_eq!(offline.field.as_deref(), Some(F_ENDPOINT));
        assert!(!offline.detail.contains("AKIA"));
        let missing = cloud_failure(&CloudError::NotFound, F_COPY);
        assert_eq!(missing.code, BackupCode::ErrBackupNotFound);
        assert_eq!(missing.field.as_deref(), Some(F_COPY));
        let bucket = cloud_failure(&CloudError::NotFound, F_BUCKET);
        assert_eq!(bucket.code, BackupCode::ErrBackupCloudFailed);
        assert_eq!(bucket.field.as_deref(), Some(F_BUCKET));
    }

    #[test]
    fn the_check_names_are_the_words_of_the_screen() {
        assert_eq!(check_text(FirstRunCheckName::Chains), "chains");
        let error = BackupError::new(BackupCode::ErrBackupCorrupt, "x");
        let named_error = named(error, FirstRunCheckName::Counts);
        assert_eq!(
            named_error.values.get("check"),
            Some(&ErrorValue::Text("counts".to_owned()))
        );
        // A name already there is kept, and other codes get none.
        let again = named(named_error, FirstRunCheckName::Opens);
        assert_eq!(
            again.values.get("check"),
            Some(&ErrorValue::Text("counts".to_owned()))
        );
        let other = named(
            BackupError::new(BackupCode::ErrBackupKeyWrong, "x"),
            FirstRunCheckName::Opens,
        );
        assert!(other.values.is_empty());
    }
}
