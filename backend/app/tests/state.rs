//! App state and `app_data_status` (plan 03 Sections 4.2b, 4.3, acceptance item 6).
//! A damaged file gives `ERR_DATA_CHECK_FAILED` with a reference, and never a crash.
#![cfg(test)]

mod common;

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use mandi_app_lib::error::ErrorCode;
use mandi_app_lib::state::{AppState, DataStatus, StatusName};
use mandi_services::db::{Db, DbOptions};
use serde_json::json;
use tempfile::tempdir;

/// One global capturing subscriber for this test file: the start-up check runs on its own
/// thread, so a per-thread subscriber would not see it.
fn log() -> &'static common::Capture {
    static LOG: OnceLock<common::Capture> = OnceLock::new();
    LOG.get_or_init(|| {
        let capture = common::Capture::default();
        tracing::subscriber::set_global_default(mandi_app_lib::logging::subscriber(
            capture.clone(),
        ))
        .unwrap();
        capture
    })
}

fn wait_for(state: &AppState, want: StatusName) -> DataStatus {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = state.data_status();
        if status.state == want {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "still {:?}, wanted {want:?}",
            status.state
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn is_reference(text: &str) -> bool {
    let b = text.as_bytes();
    text.len() == 15
        && text.starts_with("E-")
        && b[10] == b'-'
        && text[2..10].bytes().all(|c| c.is_ascii_digit())
        && text[11..].bytes().all(|c| c.is_ascii_digit())
}

#[test]
fn a_good_file_goes_from_checking_to_ok() {
    log();
    let dir = tempdir().unwrap();
    let state = AppState::start(dir.path());
    let status = wait_for(&state, StatusName::Ok);
    assert_eq!(status.reference, None);
    assert_eq!(
        serde_json::to_value(&status).unwrap(),
        json!({ "state": "ok", "reference": null })
    );
    assert!(state.db().is_ok());
    // The data file lives at data/mandi.db and the schema is there.
    assert!(dir.path().join("data").join("mandi.db").is_file());
}

#[test]
fn a_file_that_is_not_a_database_gives_failed_with_a_reference_that_is_in_the_log() {
    let log = log();
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("data")).unwrap();
    std::fs::write(
        dir.path().join("data").join("mandi.db"),
        vec![0xAB_u8; 8192],
    )
    .unwrap();
    let state = AppState::start(dir.path());
    let status = state.data_status();
    assert_eq!(status.state, StatusName::Failed);
    let reference = status.reference.clone().unwrap();
    assert!(is_reference(&reference), "{reference}");
    assert_eq!(
        serde_json::to_value(&status).unwrap(),
        json!({ "state": "failed", "reference": reference })
    );
    // The commands that need the data get the error, with the same reference every time.
    let error = state.db().unwrap_err();
    assert_eq!(error.errors[0].code, ErrorCode::ErrDataCheckFailed);
    assert_eq!(error.reference, reference);
    assert_eq!(state.db().unwrap_err().reference, reference);
    let text = log.text();
    let line = text
        .lines()
        .find(|l| l.contains(&reference))
        .unwrap_or_else(|| panic!("{reference} not in the log"));
    assert!(line.contains("ErrDataCheckFailed"), "{line}");
}

#[test]
fn damage_found_by_the_start_up_check_gives_failed_with_a_logged_reference() {
    let log = log();
    let dir = tempdir().unwrap();
    let path = dir.path().join("data").join("mandi.db");
    {
        let db = Db::open_checked(&path, DbOptions::new(dir.path().join("backups"))).unwrap();
        for i in 0..300 {
            db.write(|tx| -> Result<(), mandi_services::db::DbError> {
                tx.execute(
                    "INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic) VALUES (?1, 'ملتان', 'Multan', 'ملتان', 'multan', 'mltn')",
                    [i.to_string()],
                )?;
                Ok(())
            })
            .unwrap();
        }
    }
    let mut bytes = std::fs::read(&path).unwrap();
    let len = bytes.len();
    for byte in bytes.iter_mut().skip(len - 4096 * 3) {
        *byte = 0xFF;
    }
    std::fs::write(&path, &bytes).unwrap();
    for ext in ["db-wal", "db-shm"] {
        let _ = std::fs::remove_file(path.with_extension(ext));
    }
    let state = AppState::start(dir.path());
    let status = wait_for(&state, StatusName::Failed);
    // The reference is set before the state is visible as failed, or just after: wait for it.
    let deadline = Instant::now() + Duration::from_secs(5);
    let reference = loop {
        if let Some(r) = state.data_status().reference {
            break r;
        }
        assert!(
            Instant::now() < deadline,
            "no reference for a failed check: {status:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(is_reference(&reference), "{reference}");
    assert!(
        log.text().lines().any(|l| l.contains(&reference)),
        "{reference} not in the log"
    );
    // Saving is stopped: a write returns ERR_DATA_CHECK_FAILED.
    let db = state.db().unwrap();
    let write = db.write(|_tx| -> Result<(), mandi_services::db::DbError> { Ok(()) });
    assert_eq!(write.unwrap_err().code(), ErrorCode::ErrDataCheckFailed);
}
