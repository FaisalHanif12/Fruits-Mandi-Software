//! Plan 75 Section 7.8: the audit chain key is read from the key store once per app run, and a
//! refusal of the store (a dismissed or mistyped Mac Keychain box) is `ERR_KEYSTORE_DENIED`, not
//! `ERR_INTERNAL`. Sign-in writes an audit entry (PRD FR-8.6), which needs the key.
#![cfg(test)]

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use common::{OWNER_PASSWORD, first_run_input};
use mandi_services::auth;
use mandi_services::core::error::ErrorCode;
use mandi_services::db::DbError;
use mandi_services::db::audit::{AuditKey, KeyStore};
use mandi_services::testing::TestEnv;
use mandi_services::{Secret, ServiceError};

/// A key store that counts its reads and can refuse them, like a Keychain box that is dismissed.
struct CountingStore {
    reads: AtomicUsize,
    refuse: AtomicBool,
}

impl CountingStore {
    fn new(refuse: bool) -> Arc<Self> {
        Arc::new(Self {
            reads: AtomicUsize::new(0),
            refuse: AtomicBool::new(refuse),
        })
    }

    fn reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }
}

impl KeyStore for CountingStore {
    fn audit_key(&self) -> Result<AuditKey, DbError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if self.refuse.load(Ordering::SeqCst) {
            return Err(DbError::KeyStoreDenied(
                "Platform failure: The user name or passphrase you entered is not correct."
                    .to_owned(),
            ));
        }
        Ok(AuditKey::from_bytes([9; 32]))
    }
}

fn env_with(store: &Arc<CountingStore>) -> (tempfile::TempDir, TestEnv) {
    let dir = tempfile::tempdir().unwrap();
    let env = TestEnv::with_key_store(dir.path(), store.clone()).unwrap();
    (dir, env)
}

#[test]
fn a_refusing_key_store_gives_err_keystore_denied_and_saves_nothing() {
    let store = CountingStore::new(true);
    let (_dir, env) = env_with(&store);
    let error = auth::first_run(&env.ctx, &first_run_input()).unwrap_err();
    assert!(error.has_code(ErrorCode::ErrKeystoreDenied), "{error:?}");
    assert!(!error.has_code(ErrorCode::ErrInternal));
    // The screen gets the code only, with no field (plan 75 Section 7.8).
    assert!(matches!(
        error,
        ServiceError::Db(DbError::KeyStoreDenied(_))
    ));
    // Nothing was saved: after the user allows the app, the same set-up goes through.
    store.refuse.store(false, Ordering::SeqCst);
    auth::first_run(&env.ctx, &first_run_input()).unwrap();
}

#[test]
fn a_second_sign_in_in_one_run_does_not_read_the_key_store() {
    let store = CountingStore::new(false);
    let (_dir, env) = env_with(&store);
    auth::first_run(&env.ctx, &first_run_input()).unwrap();
    auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
    let after_first_sign_in = store.reads();
    assert_eq!(after_first_sign_in, 1, "one read covers the run");
    auth::logout(&env.ctx).unwrap();
    auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
    auth::logout(&env.ctx).unwrap();
    auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
    assert_eq!(
        store.reads(),
        after_first_sign_in,
        "later sign-ins use the key in memory"
    );
}

#[test]
fn a_refused_sign_in_is_err_keystore_denied_and_the_next_one_works_once_allowed() {
    // A key read at set-up is in memory, so to see a refusal at sign-in the key must not have
    // been read yet in this run: set up with one store, then open the same data with a refusing
    // one (a new run of the app, a new Keychain approval).
    let dir = tempfile::tempdir().unwrap();
    {
        let first = CountingStore::new(false);
        let env = TestEnv::with_key_store(dir.path(), first).unwrap();
        auth::first_run(&env.ctx, &first_run_input()).unwrap();
    }
    let store = CountingStore::new(true);
    let env = TestEnv::with_key_store(dir.path(), store.clone()).unwrap();
    let error = auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap_err();
    assert!(error.has_code(ErrorCode::ErrKeystoreDenied), "{error:?}");
    // The user allows the app in the Keychain box and signs in again.
    store.refuse.store(false, Ordering::SeqCst);
    auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
}
