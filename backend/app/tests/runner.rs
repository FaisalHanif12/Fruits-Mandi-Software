//! The command runner: no failure closes the app, and every error number is in the log
//! (plan 03 Section 4.6, NFR-REL-6, NFR-OBS-3).
#![cfg(test)]

mod common;

use mandi_app_lib::error::{ErrorCode, UiError};
use mandi_app_lib::runner::{guard, run_blocking};
use mandi_services::db::DbError;

fn first_code(error: &UiError) -> ErrorCode {
    error.errors[0].code
}

#[test]
fn a_panic_becomes_err_internal_with_a_reference_in_the_log() {
    let (result, log) = common::capture_logs(|| {
        // A bug a dependency could have: it panics with a text.
        guard::<()>("bikri_save", || {
            std::panic::panic_any("attempt to divide by zero")
        })
    });
    let error = result.unwrap_err();
    assert_eq!(first_code(&error), ErrorCode::ErrInternal);
    // The reference on the screen is on a log line with the cause.
    let line = log
        .lines()
        .find(|l| l.contains(&error.reference))
        .unwrap_or_else(|| panic!("no log line for {}:\n{log}", error.reference));
    assert!(
        line.contains("command bikri_save panicked: attempt to divide by zero"),
        "{line}"
    );
    assert!(line.contains("ERROR"), "{line}");
    // The cause is not sent to the screen.
    assert!(!serde_json::to_string(&error).unwrap().contains("divide"));
}

#[test]
fn a_string_panic_is_caught_too() {
    let (result, log) =
        common::capture_logs(|| guard::<u8>("vouchers_save", || panic!("bad {} state", 7)));
    let error = result.unwrap_err();
    assert_eq!(first_code(&error), ErrorCode::ErrInternal);
    assert!(
        log.contains("bad 7 state") && log.contains(&error.reference),
        "{log}"
    );
}

#[test]
fn ok_and_err_pass_through_untouched() {
    assert_eq!(guard("x", || Ok::<_, UiError>(5)).unwrap(), 5);
    let failed = UiError::single(ErrorCode::ErrRequired, Some("buyer"));
    let reference = failed.reference.clone();
    let back = guard::<()>("x", move || Err(failed)).unwrap_err();
    assert_eq!(back.reference, reference);
    assert_eq!(first_code(&back), ErrorCode::ErrRequired);
}

#[test]
fn every_error_reference_is_written_to_the_log() {
    let (errors, log) = common::capture_logs(|| {
        vec![
            UiError::single(ErrorCode::ErrQtyTooLarge, Some("lots[0].qty")),
            UiError::from(DbError::DiskFull {
                available_bytes: 1,
                needed_bytes: 2,
            }),
            UiError::internal("the cause"),
        ]
    });
    for error in &errors {
        let line = log
            .lines()
            .find(|l| l.contains(&error.reference))
            .unwrap_or_else(|| panic!("{} missing:\n{log}", error.reference));
        assert!(
            line.contains(&format!("{:?}", error.errors[0].code)),
            "{line}"
        );
    }
    assert!(log.contains("lots[0].qty"), "the field path is logged");
    assert!(log.contains("the cause") && log.contains("not enough free disk space"));
}

#[test]
fn run_blocking_runs_on_a_worker_and_catches_a_panic() {
    let ok =
        tauri::async_runtime::block_on(run_blocking("a", || Ok::<_, UiError>(41 + 1))).unwrap();
    assert_eq!(ok, 42);
    let error =
        tauri::async_runtime::block_on(run_blocking::<(), _>("b", || panic!("worker failed")))
            .unwrap_err();
    assert_eq!(first_code(&error), ErrorCode::ErrInternal);
}
