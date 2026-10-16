//! `app_data_status` never says `failed` without a reference number (end-gate review 22,
//! NFR-REL-6, NFR-OBS-3).
#![cfg(test)]

use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::time::{Duration, Instant};

use mandi_app_lib::state::{AppState, StatusName};
use mandi_services::db::DataState;
use mandi_services::testing::TestEnv;

/// Waits until the start-up check has finished.
fn wait_for_ok(state: &AppState) {
    let started = Instant::now();
    while state.data_status().state != StatusName::Ok {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the start-up check did not finish"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_failed_check_always_comes_with_a_reference_even_before_the_check_thread_notes_it() {
    let dir = tempfile::tempdir().unwrap();
    let env = TestEnv::new(dir.path()).unwrap();
    let state = AppState::with_context(env.ctx.clone());
    wait_for_ok(&state);
    assert_eq!(state.data_status().reference, None);

    // Damage the file, then run the check the way the daily check will: the state turns to
    // `Failed` here, and nothing has noted a reference yet (this is the moment that used to
    // answer `failed` with no number).
    let path = dir.path().join("data").join("mandi.db");
    // Fold the log into the main file first, so the damage lands in the file the check reads.
    {
        let conn = mandi_services::db::rusqlite::Connection::open(&path).unwrap();
        let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
    }
    let mut file = OpenOptions::new().write(true).open(&path).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    file.write_all(&[0_u8; 100]).unwrap();
    file.sync_all().unwrap();
    drop(file);
    assert_eq!(env.ctx.db().run_quick_check(), DataState::Failed);

    let status = state.data_status();
    assert_eq!(status.state, StatusName::Failed);
    let reference = status.reference.expect("failed without a reference");
    assert!(reference.starts_with("E-"), "{reference}");
    // The same number every time it is asked.
    assert_eq!(
        state.data_status().reference.as_deref(),
        Some(reference.as_str())
    );
    assert_eq!(
        state.clone().data_status().reference.as_deref(),
        Some(reference.as_str())
    );
}
