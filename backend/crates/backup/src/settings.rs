//! The settings of the backup (PRD 21 "Backup": cloud account details, local backup folder, USB
//! on or off, closing time). The non-secret part is one row of `settings`, written with an audit
//! row in the same transaction (Code Standards S7). The cloud keys are secrets and live only in
//! the Credential Manager (`secrets`).

use mandi_core::error::ErrorValue;
use mandi_db::Db;
use mandi_db::audit::AuditEntry;
use mandi_db::rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::error::{BackupCode, BackupError};

/// The `settings.key` of this group.
pub const SETTINGS_KEY: &str = "backup";
/// When the cloud copy is made each day if the Owner set nothing (FR-20.3).
pub const DEFAULT_CLOSING_TIME: &str = "21:00";
/// The earliest closing time, sent as `min` with `ERR_OUT_OF_RANGE` on `closing_time`.
pub const CLOSING_TIME_MIN: &str = "00:00";
/// The latest closing time, sent as `max` with `ERR_OUT_OF_RANGE` on `closing_time`.
pub const CLOSING_TIME_MAX: &str = "23:59";

/// The backup settings that are not secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct BackupSettings {
    /// Folder of the 15-minute local copies. Empty means the `backups/local` folder of the app
    /// data. Must be a full path.
    pub local_dir: String,
    /// True when a copy goes to a USB drive (FR-20.4).
    pub usb_enabled: bool,
    /// The folder on the USB drive, for example `E:\MandiBackup`. The drive counts as plugged in
    /// when the folder, or the drive it is on, is there.
    pub usb_folder: String,
    /// True when a copy goes to the cloud (FR-20.3).
    pub cloud_enabled: bool,
    /// The S3 endpoint, for example `https://<account>.r2.cloudflarestorage.com`.
    pub cloud_endpoint: String,
    /// The bucket.
    pub cloud_bucket: String,
    /// The region. Cloudflare R2 uses `auto`.
    pub cloud_region: String,
    /// A folder inside the bucket for all objects of this shop, for example `mandi`.
    pub cloud_prefix: String,
    /// The time of day (Pakistan time, `HH:MM`) when the daily USB and cloud copy is made
    /// (FR-20.3). Shared with the Day settings group.
    pub closing_time: String,
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            local_dir: String::new(),
            usb_enabled: false,
            usb_folder: String::new(),
            cloud_enabled: false,
            cloud_endpoint: String::new(),
            cloud_bucket: String::new(),
            cloud_region: "auto".to_owned(),
            cloud_prefix: "mandi".to_owned(),
            closing_time: DEFAULT_CLOSING_TIME.to_owned(),
        }
    }
}

/// The closing time as minutes since midnight.
pub fn closing_minutes(text: &str) -> Option<u32> {
    let (hours, minutes) = text.split_once(':')?;
    if hours.len() != 2 || minutes.len() != 2 {
        return None;
    }
    let (hours, minutes): (u32, u32) = (hours.parse().ok()?, minutes.parse().ok()?);
    (hours < 24 && minutes < 60).then_some(hours * 60 + minutes)
}

/// The problem with a closing time, if any (`ERR_OUT_OF_RANGE` on `closing_time` with `min` and
/// `max`). Shared with the Day settings group.
pub(crate) fn closing_time_problem(text: &str) -> Option<BackupError> {
    if closing_minutes(text).is_some() {
        return None;
    }
    // The message says "between { $min } and { $max }", so both bounds travel with it.
    Some(
        out_of_range("closing_time", "closing time is not HH:MM")
            .with("min", ErrorValue::Text(CLOSING_TIME_MIN.to_owned()))
            .with("max", ErrorValue::Text(CLOSING_TIME_MAX.to_owned())),
    )
}

fn out_of_range(field: &str, detail: &str) -> BackupError {
    BackupError::new(BackupCode::ErrOutOfRange, detail).on_field(field)
}

fn required(field: &str) -> BackupError {
    BackupError::new(BackupCode::ErrRequired, "required").on_field(field)
}

/// The most characters of a cloud address.
const ENDPOINT_MAX: usize = 300;
/// The most characters of a cloud region.
const REGION_MAX: usize = 64;

/// The value `reason` of `ERR_OUT_OF_RANGE` on the cloud address, or none when the address is
/// fine: `https` when it is not one word that starts with `https://` and has a host, `path` when
/// a path, a query or a fragment follows the host. A lone `/` after the host is allowed.
/// Example: `https://acct.r2.cloudflarestorage.com/mandi-backup` gives `path`: the bucket name
/// goes in the bucket field.
pub(crate) fn endpoint_problem(endpoint: &str) -> Option<&'static str> {
    const HTTPS: &str = "https";
    const PATH: &str = "path";
    // A host name is at most 253 characters; 300 leaves room for the scheme and a port.
    if endpoint.len() > ENDPOINT_MAX
        || endpoint
            .chars()
            .any(|c| c.is_whitespace() || c.is_control())
    {
        return Some(HTTPS);
    }
    let Some(rest) = endpoint.strip_prefix("https://") else {
        return Some(HTTPS);
    };
    // `?` and `#` end the host like `/` does.
    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (host, after) = rest.split_at(host_end);
    if host.is_empty() {
        return Some(HTTPS);
    }
    (!(after.is_empty() || after == "/")).then_some(PATH)
}

/// The one word a path names when it is only a bucket name: `/mandidemo` or `/mandidemo/` gives
/// `mandidemo`. Nothing for a longer path, a query or a word that is not a bucket name.
fn bucket_in_path(path: &str) -> Option<&str> {
    let word = path.strip_prefix('/')?.trim_end_matches('/');
    (is_bucket_name(word) && !word.contains('/')).then_some(word)
}

/// A bucket name: 3 to 63 lower-case letters, digits, dashes and dots (the rule of the bucket
/// field).
fn is_bucket_name(name: &str) -> bool {
    name.len() >= 3
        && name.len() <= 63
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
}

/// The error for a cloud address with a path: its code, its field, and what the screen needs to
/// offer "remove the path": the address cut off at the host, the part cut off, and the bucket
/// name when that part is just one. Call it only for an address `endpoint_problem` calls `path`.
/// Example: `https://acct.r2.cloudflarestorage.com/mandidemo` gives address
/// `https://acct.r2.cloudflarestorage.com`, path `/mandidemo`, bucket `mandidemo`.
fn path_error(endpoint: &str) -> BackupError {
    // `endpoint_problem` has checked the scheme and the host; `?` and `#` end the host like `/`.
    let rest = endpoint.strip_prefix("https://").unwrap_or(endpoint);
    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (host, path) = rest.split_at(host_end);
    let mut error = BackupError::new(BackupCode::ErrBackupCloudAddressPath, "endpoint has a path")
        .on_field("cloud_endpoint")
        .with("address", ErrorValue::Text(format!("https://{host}")))
        .with("path", ErrorValue::Text(path.to_owned()));
    if let Some(bucket) = bucket_in_path(path) {
        error = error.with("bucket", ErrorValue::Text(bucket.to_owned()));
    }
    error
}

fn is_full_path(text: &str) -> bool {
    std::path::Path::new(text).is_absolute()
}

impl BackupSettings {
    /// Checks every field and returns all problems together.
    ///
    /// # Errors
    /// `ERR_REQUIRED` and `ERR_OUT_OF_RANGE`, each on its field.
    pub fn validate(&self) -> Result<(), BackupError> {
        let mut problems = Vec::new();
        problems.extend(closing_time_problem(&self.closing_time));
        if self.local_dir.chars().any(char::is_control) || self.local_dir.len() > 260 {
            problems.push(out_of_range("local_dir", "local folder"));
        } else if !self.local_dir.is_empty() && !is_full_path(&self.local_dir) {
            problems.push(out_of_range(
                "local_dir",
                "local folder must be a full path",
            ));
        }
        if self.usb_enabled {
            if self.usb_folder.trim().is_empty() {
                problems.push(required("usb_folder"));
            } else if !is_full_path(&self.usb_folder)
                || self.usb_folder.chars().any(char::is_control)
            {
                problems.push(out_of_range("usb_folder", "usb folder must be a full path"));
            }
        }
        if self.cloud_enabled {
            let endpoint = self.cloud_endpoint.trim();
            if endpoint.is_empty() {
                problems.push(required("cloud_endpoint"));
            } else if let Some(reason) = endpoint_problem(endpoint) {
                if reason == "path" {
                    // Something follows the host, for example the bucket name, which belongs in
                    // its own box. Its own code, with the address to use (plan 53 section 10).
                    problems.push(path_error(endpoint));
                } else {
                    // `https`: the address must start with https:// and be one word.
                    problems.push(
                        out_of_range("cloud_endpoint", "endpoint form")
                            .with("reason", ErrorValue::Text(reason.to_owned())),
                    );
                }
            }
            let bucket = self.cloud_bucket.trim();
            if bucket.is_empty() {
                problems.push(required("cloud_bucket"));
            } else if !is_bucket_name(bucket) {
                problems.push(out_of_range("cloud_bucket", "bucket name"));
            }
            let region = self.cloud_region.trim();
            if region.is_empty() {
                problems.push(required("cloud_region"));
            } else if region.len() > REGION_MAX
                || !region
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            {
                problems.push(out_of_range("cloud_region", "region"));
            }
        }
        let prefix = &self.cloud_prefix;
        if prefix.len() > 64
            || prefix.contains("..")
            || !prefix
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
        {
            problems.push(out_of_range("cloud_prefix", "folder in the bucket"));
        }
        BackupError::from_many(problems).map_or(Ok(()), Err)
    }
}

/// Reads the settings. A missing row gives the defaults.
///
/// # Errors
/// `ERR_INTERNAL` if the row cannot be read or is not the expected JSON.
pub fn load(conn: &Connection) -> Result<BackupSettings, BackupError> {
    let text: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [SETTINGS_KEY],
            |row| row.get(0),
        )
        .optional()?;
    match text {
        None => Ok(BackupSettings::default()),
        Some(text) => Ok(serde_json::from_str(&text)?),
    }
}

/// Checks and saves the settings, with an audit row, in one transaction (Code Standards B3, S7).
///
/// # Errors
/// The errors of [`BackupSettings::validate`]; `ERR_DISK_FULL`, `ERR_DB_BUSY` and the other
/// database codes.
pub fn save(db: &Db, user_id: Option<i64>, new: &BackupSettings) -> Result<(), BackupError> {
    new.validate()?;
    db.write(|tx| save_in(db, tx, user_id, new))
}

/// Writes the settings row and its audit row inside the caller's transaction. The caller has
/// checked the values.
pub(crate) fn save_in(
    db: &Db,
    tx: &Connection,
    user_id: Option<i64>,
    new: &BackupSettings,
) -> Result<(), BackupError> {
    let value = serde_json::to_string(new)?;
    let old = load(tx)?;
    tx.execute(
        "INSERT INTO settings (key, value_json, created_by, updated_by) VALUES (?1, ?2, ?3, ?3) \
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, \
         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), updated_by = excluded.updated_by, \
         version = version + 1",
        params![SETTINGS_KEY, value, user_id],
    )?;
    if old != *new {
        db.auditor().write(
            tx,
            &AuditEntry {
                user_id,
                action: "edit".to_owned(),
                entity: "backup_settings".to_owned(),
                entity_id: None,
                old_json: Some(json!(old)),
                new_json: Some(json!(new)),
                reason: None,
            },
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_time_is_hh_mm() {
        assert_eq!(closing_minutes("21:00"), Some(21 * 60));
        assert_eq!(closing_minutes("00:05"), Some(5));
        for bad in ["9:00", "24:00", "12:60", "12-00", "", "ab:cd"] {
            assert_eq!(closing_minutes(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_cloud_address_says_why_it_is_refused() {
        for fine in [
            "https://acct.r2.cloudflarestorage.com",
            "https://acct.r2.cloudflarestorage.com/",
            "https://localhost:9000",
            "https://s3.eu-west-1.amazonaws.com",
        ] {
            assert_eq!(endpoint_problem(fine), None, "{fine}");
        }
        for (bad, reason) in [
            ("http://acct.r2.cloudflarestorage.com", "https"),
            ("acct.r2.cloudflarestorage.com", "https"),
            ("ftp://acct.com", "https"),
            ("https://", "https"),
            ("https:///mandi", "https"),
            ("https://a b.com", "https"),
            ("https://acct.com\n", "https"),
            ("https://acct.r2.cloudflarestorage.com/mandi-backup", "path"),
            (
                "https://acct.r2.cloudflarestorage.com/mandi-backup/",
                "path",
            ),
            ("https://acct.r2.cloudflarestorage.com//", "path"),
            ("https://acct.com?x=1", "path"),
            ("https://acct.com#top", "path"),
        ] {
            assert_eq!(endpoint_problem(bad), Some(reason), "{bad}");
        }
        // A form problem travels with the error, on the field of the address.
        let settings = BackupSettings {
            cloud_enabled: true,
            cloud_endpoint: "http://acct.r2.cloudflarestorage.com".into(),
            cloud_bucket: "mandi-backup".into(),
            ..BackupSettings::default()
        };
        let items = settings.validate().unwrap_err().all();
        assert_eq!(items.len(), 1);
        let item = items.first().unwrap();
        assert_eq!(item.field.as_deref(), Some("cloud_endpoint"));
        assert_eq!(item.code, BackupCode::ErrOutOfRange);
        assert_eq!(
            item.values.get("reason"),
            Some(&ErrorValue::Text("https".into()))
        );
    }

    fn text(error: &BackupError, name: &str) -> Option<String> {
        match error.values.get(name) {
            Some(ErrorValue::Text(text)) => Some(text.clone()),
            _ => None,
        }
    }

    /// Plan 53 section 10: Umar typed the bucket at the end of the address. The save is refused
    /// with its own code on the address field, and the error carries the address to use.
    #[test]
    fn a_cloud_address_with_a_path_has_its_own_code_and_the_address_without_it() {
        let check = |typed: &str| {
            let settings = BackupSettings {
                cloud_enabled: true,
                cloud_endpoint: format!("  {typed} "),
                cloud_bucket: "mandidemo".into(),
                ..BackupSettings::default()
            };
            let items = settings.validate().unwrap_err().all();
            assert_eq!(items.len(), 1, "{typed}");
            items.into_iter().next().unwrap()
        };
        // Cloudflare's address with the bucket at its end: the case of the plan.
        let item = check("https://acct.r2.cloudflarestorage.com/mandidemo");
        assert_eq!(item.code, BackupCode::ErrBackupCloudAddressPath);
        assert_eq!(item.field.as_deref(), Some("cloud_endpoint"));
        assert_eq!(
            text(&item, "address").as_deref(),
            Some("https://acct.r2.cloudflarestorage.com")
        );
        assert_eq!(text(&item, "path").as_deref(), Some("/mandidemo"));
        assert_eq!(text(&item, "bucket").as_deref(), Some("mandidemo"));
        // The address to use is itself fine.
        assert_eq!(
            endpoint_problem("https://acct.r2.cloudflarestorage.com"),
            None
        );
        // A slash after the bucket, a port, a longer path, a query and a fragment.
        for (typed, address, path, bucket) in [
            (
                "https://acct.r2.cloudflarestorage.com/mandidemo/",
                "https://acct.r2.cloudflarestorage.com",
                "/mandidemo/",
                Some("mandidemo"),
            ),
            (
                "https://localhost:9000/mandi-backup",
                "https://localhost:9000",
                "/mandi-backup",
                Some("mandi-backup"),
            ),
            (
                "https://acct.com/mandidemo/mandi",
                "https://acct.com",
                "/mandidemo/mandi",
                None,
            ),
            ("https://acct.com//", "https://acct.com", "//", None),
            ("https://acct.com?x=1", "https://acct.com", "?x=1", None),
            ("https://acct.com#top", "https://acct.com", "#top", None),
            // Not a bucket name (capital letters, too short): the path is cut, no bucket offered.
            (
                "https://acct.com/MyBucket",
                "https://acct.com",
                "/MyBucket",
                None,
            ),
            ("https://acct.com/ab", "https://acct.com", "/ab", None),
        ] {
            let item = check(typed);
            assert_eq!(item.code, BackupCode::ErrBackupCloudAddressPath, "{typed}");
            assert_eq!(text(&item, "address").as_deref(), Some(address), "{typed}");
            assert_eq!(text(&item, "path").as_deref(), Some(path), "{typed}");
            assert_eq!(text(&item, "bucket").as_deref(), bucket, "{typed}");
            assert_eq!(endpoint_problem(address), None, "{address}");
        }
        // Without the cloud switched on the address is not looked at, as before.
        let off = BackupSettings {
            cloud_endpoint: "https://acct.com/mandidemo".into(),
            ..BackupSettings::default()
        };
        off.validate().unwrap();
        // The other problems of the same save still come with it.
        let both = BackupSettings {
            cloud_enabled: true,
            cloud_endpoint: "https://acct.com/mandidemo".into(),
            cloud_bucket: String::new(),
            ..BackupSettings::default()
        };
        let fields: std::collections::BTreeSet<(String, BackupCode)> = both
            .validate()
            .unwrap_err()
            .all()
            .into_iter()
            .map(|e| (e.field.unwrap_or_default(), e.code))
            .collect();
        assert_eq!(
            fields,
            [
                (
                    "cloud_endpoint".to_owned(),
                    BackupCode::ErrBackupCloudAddressPath
                ),
                ("cloud_bucket".to_owned(), BackupCode::ErrRequired),
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn defaults_are_valid() {
        BackupSettings::default().validate().unwrap();
    }

    #[test]
    fn all_problems_come_together_each_on_its_field() {
        let settings = BackupSettings {
            usb_enabled: true,
            usb_folder: String::new(),
            cloud_enabled: true,
            cloud_endpoint: "http://insecure".into(),
            cloud_bucket: String::new(),
            closing_time: "25:00".into(),
            ..BackupSettings::default()
        };
        let error = settings.validate().unwrap_err();
        let items = error.all();
        let time = items
            .iter()
            .find(|e| e.field.as_deref() == Some("closing_time"))
            .unwrap();
        assert_eq!(
            time.values.get("min"),
            Some(&ErrorValue::Text("00:00".into()))
        );
        assert_eq!(
            time.values.get("max"),
            Some(&ErrorValue::Text("23:59".into()))
        );
        let found: std::collections::BTreeSet<(String, BackupCode)> = items
            .into_iter()
            .map(|e| (e.field.unwrap_or_default(), e.code))
            .collect();
        let expected: std::collections::BTreeSet<(String, BackupCode)> = [
            ("cloud_bucket", BackupCode::ErrRequired),
            ("usb_folder", BackupCode::ErrRequired),
            ("closing_time", BackupCode::ErrOutOfRange),
            ("cloud_endpoint", BackupCode::ErrOutOfRange),
        ]
        .into_iter()
        .map(|(field, code)| (field.to_owned(), code))
        .collect();
        assert_eq!(found, expected);
    }
}
