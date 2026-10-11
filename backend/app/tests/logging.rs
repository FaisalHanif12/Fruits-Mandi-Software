//! Log files: no secrets, 30 days, 50 MB (plan 03 Section 4.7, NFR-OBS-1, NFR-OBS-2).
#![cfg(test)]

mod common;

use std::fs;

use chrono::NaiveDate;
use mandi_app_lib::logging::{
    HIDDEN, LOG_FILE_PREFIX, LOG_KEEP_DAYS, LOG_MAX_TOTAL_BYTES, clean_up, is_secret_field,
    subscriber,
};
use tempfile::tempdir;

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// The check the plan asks for: log a password field and read the FILE.
#[test]
fn a_logged_password_never_reaches_the_file() {
    let dir = tempdir().unwrap();
    let appender = tracing_appender::rolling::daily(dir.path(), LOG_FILE_PREFIX);
    tracing::subscriber::with_default(subscriber(appender), || {
        tracing::info!(user = "owner", password = "hunter2-Xq9", "login attempt");
        tracing::info!(
            recovery_code = "ABCD-EFGH-1234",
            token = "tok_live_77",
            secret = "s3cr3t-value",
            audit_key = "key-bytes-aa",
            balance = 5_848_400,
            balance_paisa = 5_848_400,
            new_password = "pw-two-Zz",
            "sensitive fields"
        );
        tracing::info!(account = "03040001", rows = 3, "plain fields stay");
    });
    let files: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1, "one daily file");
    let name = files[0].file_name().unwrap().to_string_lossy().into_owned();
    assert!(name.starts_with("mandi.log."), "{name}");
    let text = fs::read_to_string(&files[0]).unwrap();
    for secret in [
        "hunter2-Xq9",
        "ABCD-EFGH-1234",
        "tok_live_77",
        "s3cr3t-value",
        "key-bytes-aa",
        "5848400",
        "pw-two-Zz",
    ] {
        assert!(
            !text.contains(secret),
            "{secret} leaked into the log:\n{text}"
        );
    }
    assert!(text.contains(&format!("password={HIDDEN}")), "{text}");
    assert!(text.contains(&format!("balance={HIDDEN}")), "{text}");
    // The names stay, so a log line still says which field was hidden; other fields stay.
    assert!(text.contains("user=\"owner\""), "{text}");
    assert!(text.contains("login attempt"), "{text}");
    assert!(
        text.contains("account=\"03040001\"") && text.contains("rows=3"),
        "{text}"
    );
}

#[test]
fn field_names_that_hide() {
    for name in [
        "password",
        "Password",
        "password_hash",
        "new_password",
        "key",
        "audit_key",
        "api_key",
        "secret",
        "client_secret",
        "token",
        "access_token",
        "recovery_code",
        "balance",
        "balance_paisa",
    ] {
        assert!(is_secret_field(name), "{name}");
    }
    for name in [
        "user",
        "account",
        "reference",
        "code",
        "detail",
        "rows",
        "entry_id",
        "message",
    ] {
        assert!(!is_secret_field(name), "{name}");
    }
}

fn touch(dir: &std::path::Path, name: &str, bytes: usize) {
    fs::write(dir.join(name), vec![b'x'; bytes]).unwrap();
}

fn names(dir: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn files_30_days_old_or_older_are_deleted() {
    let dir = tempdir().unwrap();
    let today = day(2026, 10, 11);
    for name in [
        "mandi.log.2026-08-01",
        "mandi.log.2026-09-11",
        "mandi.log.2026-09-12",
        "mandi.log.2026-10-10",
        "mandi.log.2026-10-11",
    ] {
        touch(dir.path(), name, 100);
    }
    touch(dir.path(), "notes.txt", 100);
    let report = clean_up(dir.path(), today, LOG_KEEP_DAYS, LOG_MAX_TOTAL_BYTES).unwrap();
    // 2026-09-11 is 30 days before 2026-10-11: gone. 2026-09-12 is 29 days: kept.
    assert_eq!(
        report.deleted,
        ["mandi.log.2026-08-01", "mandi.log.2026-09-11"]
    );
    assert_eq!(report.kept_bytes, 300);
    assert_eq!(
        names(dir.path()),
        [
            "mandi.log.2026-09-12",
            "mandi.log.2026-10-10",
            "mandi.log.2026-10-11",
            "notes.txt"
        ]
    );
}

#[test]
fn the_total_is_capped_oldest_first_and_today_stays() {
    let dir = tempdir().unwrap();
    let today = day(2026, 10, 11);
    for (i, name) in [
        "mandi.log.2026-10-05",
        "mandi.log.2026-10-06",
        "mandi.log.2026-10-07",
        "mandi.log.2026-10-08",
        "mandi.log.2026-10-09",
    ]
    .iter()
    .enumerate()
    {
        touch(dir.path(), name, 10 + i);
    }
    touch(dir.path(), "mandi.log.2026-10-11", 40);
    // 10+11+12+13+14 = 60 plus today's 40 = 100. Cap 70: delete oldest first until it fits.
    let report = clean_up(dir.path(), today, LOG_KEEP_DAYS, 70).unwrap();
    assert_eq!(
        report.deleted,
        [
            "mandi.log.2026-10-05",
            "mandi.log.2026-10-06",
            "mandi.log.2026-10-07"
        ]
    );
    assert_eq!(report.kept_bytes, 13 + 14 + 40);
    // A cap smaller than today's own file still keeps today's file (it is open).
    let report = clean_up(dir.path(), today, LOG_KEEP_DAYS, 1).unwrap();
    assert_eq!(names(dir.path()), ["mandi.log.2026-10-11"]);
    assert_eq!(report.kept_bytes, 40);
}

#[test]
fn the_limits_are_the_nfr_numbers() {
    assert_eq!(LOG_KEEP_DAYS, 30);
    assert_eq!(LOG_MAX_TOTAL_BYTES, 50 * 1024 * 1024);
}

#[test]
fn an_empty_or_missing_folder_is_fine() {
    let dir = tempdir().unwrap();
    let report = clean_up(
        dir.path(),
        day(2026, 10, 11),
        LOG_KEEP_DAYS,
        LOG_MAX_TOTAL_BYTES,
    )
    .unwrap();
    assert_eq!(report.deleted.len(), 0);
    assert!(
        clean_up(
            &dir.path().join("none"),
            day(2026, 10, 11),
            LOG_KEEP_DAYS,
            LOG_MAX_TOTAL_BYTES
        )
        .is_err()
    );
}
