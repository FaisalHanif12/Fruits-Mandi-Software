//! Change of password from the user menu (end-gate review 12, PRD FR-5.6, FR-8.2, NFR-SEC-1,
//! NFR-SEC-5) and a lock or sign-out that still works when the audit row cannot be written
//! (end-gate review 11, NFR-SEC-3).
#![cfg(test)]

mod common;

use common::*;
use mandi_services::auth::{self, ChangePasswordRaw};
use mandi_services::core::error::ErrorCode;
use mandi_services::testing::TestEnv;
use mandi_services::{Secret, ServiceError};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn change(current: &str, new: &str, repeat: &str) -> ChangePasswordRaw {
    ChangePasswordRaw {
        current: Secret::new(current),
        new: Secret::new(new),
        repeat: Secret::new(repeat),
    }
}

fn fields(error: ServiceError) -> Vec<(String, ErrorCode)> {
    match error {
        ServiceError::Rejected(items) => items
            .into_iter()
            .map(|i| (i.field.unwrap_or_default(), i.code))
            .collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn stored_hash(dir: &TempDir) -> String {
    raw(dir)
        .query_row("SELECT password_hash FROM users WHERE id = 1", [], |r| {
            r.get(0)
        })
        .unwrap()
}

fn failed_attempts(dir: &TempDir) -> i64 {
    raw(dir)
        .query_row("SELECT failed_attempts FROM users WHERE id = 1", [], |r| {
            r.get(0)
        })
        .unwrap()
}

#[test]
fn a_right_current_password_changes_the_password() {
    let (dir, env, owner) = ready();
    let old_hash = stored_hash(&dir);
    auth::change_password(
        &env.ctx,
        &owner,
        &change(OWNER_PASSWORD, "brand-new-9", "brand-new-9"),
    )
    .unwrap();
    assert_ne!(stored_hash(&dir), old_hash);
    // He stays signed in; the old password is dead and the new one works.
    auth::require(&env.ctx).unwrap();
    auth::logout(&env.ctx).unwrap();
    let error = auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrWrongPassword);
    auth::login(&env.ctx, "owner", &Secret::new("brand-new-9")).unwrap();
    assert_chain_ok(&dir);
}

#[test]
fn the_audit_row_says_it_changed_and_never_holds_a_password() {
    let (dir, env, owner) = ready();
    auth::change_password(
        &env.ctx,
        &owner,
        &change(OWNER_PASSWORD, "brand-new-9", "brand-new-9"),
    )
    .unwrap();
    assert_eq!(audit_count(&dir, "change_password"), 1);
    let new_hash = stored_hash(&dir);
    let rows: Vec<String> = {
        let conn = raw(&dir);
        let mut statement = conn
            .prepare(
                "SELECT COALESCE(old_json, '') || COALESCE(new_json, '') || COALESCE(reason, '')
                     || action || entity FROM audit_log",
            )
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let everything = rows.join("\n");
    for secret in [OWNER_PASSWORD, "brand-new-9", new_hash.as_str()] {
        assert!(!everything.contains(secret), "{secret} is in the audit log");
    }
    let (user, entity): (Option<i64>, String) = raw(&dir)
        .query_row(
            "SELECT user_id, entity FROM audit_log WHERE action = 'change_password'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((user, entity.as_str()), (Some(1), "user"));
}

#[test]
fn a_badly_filled_form_is_refused_with_its_fields_and_counts_no_try() {
    let (dir, env, owner) = ready();
    let hash = stored_hash(&dir);
    let audit_before = audit_total(&dir);
    for (input, want) in [
        (
            change("", "brand-new-9", "brand-new-9"),
            ("current", ErrorCode::ErrRequired),
        ),
        (
            change(OWNER_PASSWORD, "short", "short"),
            ("new", ErrorCode::ErrPasswordTooShort),
        ),
        (
            change(OWNER_PASSWORD, "brand-new-9", "brand-new-8"),
            ("repeat", ErrorCode::ErrPasswordsDiffer),
        ),
    ] {
        let got = fields(auth::change_password(&env.ctx, &owner, &input).unwrap_err());
        assert!(
            got.contains(&(want.0.to_owned(), want.1)),
            "{want:?} not in {got:?}"
        );
    }
    // A wrong current password together with a bad new one: the shape is refused first and the
    // wrong password is not counted.
    let got = fields(
        auth::change_password(&env.ctx, &owner, &change("wrong-one", "short", "short"))
            .unwrap_err(),
    );
    assert_eq!(got, [("new".to_owned(), ErrorCode::ErrPasswordTooShort)]);
    assert_eq!(failed_attempts(&dir), 0);
    assert_eq!(stored_hash(&dir), hash);
    assert_eq!(audit_total(&dir), audit_before, "nothing was written");
}

#[test]
fn a_wrong_current_password_counts_towards_the_sign_in_lockout() {
    let (dir, env, owner) = ready();
    let hash = stored_hash(&dir);
    let wrong = change("not-the-password", "brand-new-9", "brand-new-9");
    // Tries 1 to 4: wrong password on `current`, with the tries left.
    for left in [4, 3, 2, 1] {
        let error = auth::change_password(&env.ctx, &owner, &wrong).unwrap_err();
        assert_eq!(
            code_and_value(&error, "tries_left"),
            (ErrorCode::ErrWrongPassword, Some(left))
        );
        let ServiceError::Rejected(items) = error else {
            panic!("not a rejection")
        };
        assert_eq!(items[0].field.as_deref(), Some("current"));
    }
    // Try 5: locked for 60 seconds, the same step as sign-in (PRD FR-8.2).
    let error = auth::change_password(&env.ctx, &owner, &wrong).unwrap_err();
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(60))
    );
    assert_eq!(failed_attempts(&dir), 5);
    assert_eq!(stored_hash(&dir), hash, "the password did not change");
    // While locked, the right password changes nothing either.
    let right = change(OWNER_PASSWORD, "brand-new-9", "brand-new-9");
    let error = auth::change_password(&env.ctx, &owner, &right).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrLockedWait);
    assert_eq!(stored_hash(&dir), hash);
    // The wait runs out; a right password works and clears the count.
    env.mono.advance_seconds(61);
    auth::change_password(&env.ctx, &owner, &right).unwrap();
    assert_eq!(failed_attempts(&dir), 0);
}

#[test]
fn the_tries_of_both_doors_are_one_count() {
    let (dir, env, owner) = ready();
    // Three wrong tries at the menu, then two at the sign-in door: the 5th locks.
    let wrong = change("not-the-password", "brand-new-9", "brand-new-9");
    for _ in 0..3 {
        auth::change_password(&env.ctx, &owner, &wrong).unwrap_err();
    }
    auth::logout(&env.ctx).unwrap();
    let error = wrong_try(&env, "owner");
    assert_eq!(
        code_and_value(&error, "tries_left"),
        (ErrorCode::ErrWrongPassword, Some(1))
    );
    let error = wrong_try(&env, "owner");
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(60))
    );
    assert_eq!(failed_attempts(&dir), 5);
}

#[test]
fn without_a_session_the_sign_in_check_refuses_first() {
    // The command calls `auth::require` before the service. The service takes the Actor it
    // returns, so there is no way to reach it without that check.
    let (_dir, env, owner) = ready();
    auth::lock(&env.ctx).unwrap();
    assert_eq!(
        auth::require(&env.ctx).unwrap_err().code(),
        ErrorCode::ErrSessionLocked
    );
    auth::logout(&env.ctx).unwrap();
    assert_eq!(
        auth::require(&env.ctx).unwrap_err().code(),
        ErrorCode::ErrNotLoggedIn
    );
    // An Actor kept from before is no way in either: the user is read fresh, and a wrong
    // password still counts.
    let error = auth::change_password(
        &env.ctx,
        &owner,
        &change("not-the-password", "brand-new-9", "brand-new-9"),
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrWrongPassword);
}

#[test]
fn lock_works_when_the_audit_row_cannot_be_written() {
    // NFR-SEC-3: a full disk, a failed data check or a busy file must never leave the session open.
    let (dir, env, _owner) = ready();
    env.disk.fill_up();
    let audit_before = audit_total(&dir);
    auth::lock(&env.ctx).unwrap();
    assert_eq!(
        auth::require(&env.ctx).unwrap_err().code(),
        ErrorCode::ErrSessionLocked,
        "locked although the audit row failed"
    );
    assert_eq!(audit_total(&dir), audit_before, "no row could be written");
    // With room again, the Owner unlocks normally.
    env.disk.make_room();
    auth::unlock(&env.ctx, &Secret::new(OWNER_PASSWORD)).unwrap();
    auth::require(&env.ctx).unwrap();
    // With room, the lock is written down as before.
    auth::lock(&env.ctx).unwrap();
    assert_eq!(audit_count(&dir, "lock"), 1);
    assert_chain_ok(&dir);
}

#[test]
fn logout_works_when_the_audit_row_cannot_be_written() {
    let (dir, env, _owner) = ready();
    env.disk.fill_up();
    let audit_before = audit_total(&dir);
    auth::logout(&env.ctx).unwrap();
    assert_eq!(
        auth::require(&env.ctx).unwrap_err().code(),
        ErrorCode::ErrNotLoggedIn,
        "still signed in although the audit row failed"
    );
    assert_eq!(audit_total(&dir), audit_before);
    // A second sign-out has no session to end.
    assert_eq!(
        auth::logout(&env.ctx).unwrap_err().code(),
        ErrorCode::ErrNotLoggedIn
    );
    env.disk.make_room();
    login_owner(&env);
    auth::logout(&env.ctx).unwrap();
    assert_eq!(audit_count(&dir, "logout"), 1);
    assert_chain_ok(&dir);
}
