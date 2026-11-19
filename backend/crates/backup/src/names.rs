//! File names of copies, and the check of a name that comes from a screen.
//!
//! `mandi-local-20261010T090000123Z.db` is a plain local copy. `mandi-20261010T090000123Z.mbk`
//! is an encrypted bundle for USB and cloud. A `.json` file with the same stem sits next to it.

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::{BackupCode, BackupError};
use crate::model::BackupKind;

/// Extension of a plain copy.
pub const PLAIN_EXT: &str = "db";
/// Extension of an encrypted bundle.
pub const BUNDLE_EXT: &str = "mbk";
/// Extension of the manifest next to a copy.
pub const MANIFEST_EXT: &str = "json";

const STAMP_FORMAT: &str = "%Y%m%dT%H%M%S%3fZ";

/// The time part of a file name, for example `20261010T090000123Z`.
pub fn stamp(time: DateTime<Utc>) -> String {
    time.format(STAMP_FORMAT).to_string()
}

/// The file name of a plain local copy.
pub fn local_name(stamp: &str) -> String {
    format!("mandi-local-{stamp}.{PLAIN_EXT}")
}

/// The file name of an encrypted bundle.
pub fn bundle_name(stamp: &str) -> String {
    format!("mandi-{stamp}.{BUNDLE_EXT}")
}

/// The manifest name that belongs to a copy.
pub fn manifest_name(file: &str) -> String {
    let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
    format!("{stem}.{MANIFEST_EXT}")
}

/// The time in a copy's file name.
pub fn parse_stamp(file: &str) -> Option<DateTime<Utc>> {
    let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
    let text = stem
        .strip_prefix("mandi-local-")
        .or_else(|| stem.strip_prefix("mandi-"))?;
    NaiveDateTime::parse_from_str(text, STAMP_FORMAT)
        .ok()
        .map(|naive| naive.and_utc())
}

/// True for the name of a copy of the given kind (not a manifest, not a partial file).
pub fn is_copy_name(kind: BackupKind, file: &str) -> bool {
    match kind {
        BackupKind::Local => file.starts_with("mandi-local-") && file.ends_with(".db"),
        BackupKind::Usb | BackupKind::Cloud => {
            file.starts_with("mandi-")
                && !file.starts_with("mandi-local-")
                && file.ends_with(".mbk")
        }
    }
}

/// Checks a copy name from a screen: only letters, digits, `-`, `_` and one dot, so it can never
/// point outside the backup folder (Code Standards V0).
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on field `name`.
pub fn check_name(kind: BackupKind, name: &str) -> Result<(), BackupError> {
    let clean = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        && !name.contains("..")
        && name.len() <= 80;
    if clean && is_copy_name(kind, name) {
        Ok(())
    } else {
        Err(BackupError::new(BackupCode::ErrOutOfRange, "not a backup file name").on_field("name"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn names_round_trip_with_their_time() {
        let time = Utc.with_ymd_and_hms(2026, 10, 10, 9, 5, 7).unwrap();
        let s = stamp(time);
        assert_eq!(s, "20261010T090507000Z");
        assert_eq!(local_name(&s), "mandi-local-20261010T090507000Z.db");
        assert_eq!(bundle_name(&s), "mandi-20261010T090507000Z.mbk");
        assert_eq!(parse_stamp(&local_name(&s)), Some(time));
        assert_eq!(parse_stamp(&bundle_name(&s)), Some(time));
        assert_eq!(
            manifest_name(&bundle_name(&s)),
            "mandi-20261010T090507000Z.json"
        );
    }

    #[test]
    fn a_name_from_a_screen_cannot_leave_the_folder() {
        let good = "mandi-20261010T090507000Z.mbk";
        check_name(BackupKind::Cloud, good).unwrap();
        check_name(BackupKind::Usb, good).unwrap();
        assert!(check_name(BackupKind::Local, good).is_err(), "wrong kind");
        for bad in [
            "../mandi-x.mbk",
            "mandi-..\\x.mbk",
            "mandi-a/b.mbk",
            "",
            "mandi-x.json",
        ] {
            let error = check_name(BackupKind::Cloud, bad).unwrap_err();
            assert_eq!(error.code, BackupCode::ErrOutOfRange, "{bad}");
            assert_eq!(error.field.as_deref(), Some("name"));
        }
    }

    #[test]
    fn a_partial_file_is_not_a_copy() {
        assert!(!is_copy_name(
            BackupKind::Cloud,
            "mandi-20261010T090507000Z.mbk.partial"
        ));
        assert!(!is_copy_name(
            BackupKind::Local,
            "mandi-local-20261010T090507000Z.db.partial"
        ));
    }
}
