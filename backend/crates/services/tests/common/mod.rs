//! Helpers for the services integration tests.
#![allow(dead_code)]

use mandi_services::auth::{self, FirstRunRaw, SessionInfo};
use mandi_services::core::error::ErrorCode;
use mandi_services::db::audit::{AuditKey, verify_audit_chain};
use mandi_services::db::rusqlite::Connection;
use mandi_services::settings::Firm;
use mandi_services::testing::TestEnv;
use mandi_services::{Secret, ServiceError};
use tempfile::TempDir;

/// The password of the test Owner.
pub const OWNER_PASSWORD: &str = "secret1";

/// A test environment with a database in a temp folder.
pub fn env() -> (TempDir, TestEnv) {
    let dir = tempfile::tempdir().unwrap();
    let env = TestEnv::new(dir.path()).unwrap();
    (dir, env)
}

/// The first-run wizard filled in like the PRD client (names from the PRD header).
pub fn first_run_input() -> FirstRunRaw {
    FirstRunRaw {
        firm: Firm {
            name_ur: "ماہر عدنان اینڈ برادرز".to_owned(),
            name_en: "Maher Adnan and Brothers".to_owned(),
            mandi_ur: "نئی سبزی منڈی کوٹ رادھا کشن".to_owned(),
            mandi_en: "New Sabzi Mandi Kot Radha Kishan".to_owned(),
            address: "Kot Radha Kishan".to_owned(),
            phone: "0300-1234567".to_owned(),
        },
        owner_username: "Owner".to_owned(),
        owner_display_name: "Maher Adnan".to_owned(),
        password: Secret::new(OWNER_PASSWORD),
        password_repeat: Secret::new(OWNER_PASSWORD),
    }
}

/// Runs the wizard and returns the recovery code.
pub fn set_up(env: &TestEnv) -> String {
    auth::first_run(&env.ctx, &first_run_input()).unwrap()
}

/// Signs in the Owner.
pub fn login_owner(env: &TestEnv) -> SessionInfo {
    auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap()
}

/// A second raw connection to the data file, to look at what was written.
pub fn raw(dir: &TempDir) -> Connection {
    let conn = Connection::open(dir.path().join("data").join("mandi.db")).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn
}

/// The test audit key (the one `TestEnv` uses).
pub fn audit_key() -> AuditKey {
    AuditKey::from_bytes([7; 32])
}

/// Panics unless the whole audit chain is intact.
pub fn assert_chain_ok(dir: &TempDir) {
    verify_audit_chain(&raw(dir), &audit_key()).unwrap();
}

/// How many audit rows have this action.
pub fn audit_count(dir: &TempDir, action: &str) -> i64 {
    raw(dir)
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = ?1",
            [action],
            |r| r.get(0),
        )
        .unwrap()
}

/// How many audit rows there are.
pub fn audit_total(dir: &TempDir) -> i64 {
    raw(dir)
        .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
        .unwrap()
}

/// The code and the number value `key` of the first rejection of an error.
pub fn code_and_value(error: &ServiceError, key: &str) -> (ErrorCode, Option<i64>) {
    use mandi_services::core::error::ErrorValue;
    match error {
        ServiceError::Rejected(items) => {
            let item = items.first().unwrap();
            let value = match item.values.get(key) {
                Some(ErrorValue::Number(n)) => Some(*n),
                _ => None,
            };
            (item.code, value)
        }
        other => (other.code(), None),
    }
}

/// Signs in with a wrong password and returns the error.
pub fn wrong_try(env: &TestEnv, username: &str) -> ServiceError {
    auth::login(&env.ctx, username, &Secret::new("not-the-password")).unwrap_err()
}

/// Sets the test clock to 09:00 UTC (14:00 in Pakistan) of a day, so "today" is that day.
pub fn set_today(env: &TestEnv, year: i32, month: u32, day: u32) {
    use chrono::TimeZone;
    env.clock.set(
        chrono::Utc
            .with_ymd_and_hms(year, month, day, 9, 0, 0)
            .single()
            .unwrap(),
    );
}
