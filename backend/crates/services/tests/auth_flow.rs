//! Sign-in, lockout, unlock, recovery and the sign-in check (plan 04 Sections 3 and 7,
//! D1-A11 to D1-A13). There are no roles: the Owner is the only user (PRD Section 2).
#![cfg(test)]

mod common;

use common::*;
use mandi_services::auth;
use mandi_services::core::error::ErrorCode;
use mandi_services::{Secret, ServiceError};

fn owner_secret() -> Secret {
    Secret::new(OWNER_PASSWORD)
}

fn failed_attempts(dir: &tempfile::TempDir) -> i64 {
    raw(dir)
        .query_row(
            "SELECT failed_attempts FROM users WHERE username = 'owner'",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn first_run_creates_the_owner_the_firm_and_the_recovery_code() {
    let (dir, env) = env();
    assert!(auth::status(&env.ctx).unwrap().first_run);
    let code = set_up(&env);
    // XXXX-XXXX-XXXX-XXXX from the unambiguous alphabet.
    assert_eq!(code.len(), 19);
    assert!(code.split('-').all(|g| g.len() == 4));
    assert!(
        code.chars()
            .all(|c| c == '-' || "ABCDEFGHJKMNPQRSTVWXYZ23456789".contains(c)),
        "{code}"
    );

    let status = auth::status(&env.ctx).unwrap();
    assert!(!status.first_run);
    assert!(status.session.is_none() && !status.locked);
    assert_eq!(status.firm.name_en, "Maher Adnan and Brothers");
    assert_eq!(status.firm.phone, "03001234567");

    let conn = raw(&dir);
    let (username, role, hash, active): (String, String, String, bool) = conn
        .query_row(
            "SELECT username, role, password_hash, is_active FROM users",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        (username.as_str(), role.as_str(), active),
        ("owner", "owner", true)
    );
    assert!(hash.starts_with("$argon2id$v=19$"), "{hash}");
    assert!(!hash.contains(OWNER_PASSWORD));
    // The recovery code is stored only as a hash.
    let stored: String = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'recovery_code_hash'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(stored.contains("$argon2id$") && !stored.contains(&code.replace('-', "")));
    // Two audit rows, a good chain.
    assert_eq!(audit_total(&dir), 2);
    assert_chain_ok(&dir);
}

#[test]
fn a_second_first_run_is_refused_even_with_bad_input() {
    let (_dir, env) = env();
    set_up(&env);
    let mut again = first_run_input();
    again.password = Secret::new("x");
    let error = auth::first_run(&env.ctx, &again).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrAlreadySetUp);
}

#[test]
fn first_run_returns_every_field_error_together() {
    let (_dir, env) = env();
    let mut bad = first_run_input();
    bad.firm.name_ur = "Latin letters".to_owned();
    bad.firm.name_en = "اردو".to_owned();
    bad.firm.mandi_ur = String::new();
    bad.firm.phone = "12345".to_owned();
    bad.firm.address = "x".repeat(501);
    bad.owner_username = "a b".to_owned();
    bad.owner_display_name = String::new();
    bad.password = Secret::new("abc");
    bad.password_repeat = Secret::new("abd");
    let ServiceError::Rejected(items) = auth::first_run(&env.ctx, &bad).unwrap_err() else {
        panic!("expected a rejection");
    };
    let got: Vec<(String, ErrorCode)> = items
        .iter()
        .map(|i| (i.field.clone().unwrap(), i.code))
        .collect();
    let expect = [
        ("firm_name_ur", ErrorCode::ErrNameUrScript),
        ("firm_name_en", ErrorCode::ErrNameEnScript),
        ("mandi_ur", ErrorCode::ErrRequired),
        ("address", ErrorCode::ErrTextTooLong),
        ("phone", ErrorCode::ErrPhoneInvalid),
        ("owner_username", ErrorCode::ErrUsernameInvalid),
        ("owner_display_name", ErrorCode::ErrRequired),
        ("password", ErrorCode::ErrPasswordTooShort),
    ];
    for (field, code) in expect {
        assert!(
            got.contains(&(field.to_owned(), code)),
            "{field} {code:?} missing in {got:?}"
        );
    }
    // Nothing was created.
    assert!(auth::status(&env.ctx).unwrap().first_run);
}

#[test]
fn a_mismatched_repeat_is_reported_on_the_repeat_field() {
    let (_dir, env) = env();
    let mut input = first_run_input();
    input.password_repeat = Secret::new("secret2");
    let ServiceError::Rejected(items) = auth::first_run(&env.ctx, &input).unwrap_err() else {
        panic!("expected a rejection");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrPasswordsDiffer, Some("password_repeat"))
    );
}

#[test]
fn a_right_password_signs_in_and_is_written_down() {
    let (dir, env) = env();
    set_up(&env);
    let info = login_owner(&env);
    assert_eq!(
        (info.username.as_str(), info.display_name.as_str()),
        ("owner", "Maher Adnan")
    );
    assert_eq!(info.text_size, 100);
    assert_eq!(info.language.as_str(), "ur");
    let status = auth::status(&env.ctx).unwrap();
    assert_eq!(status.session.as_ref().map(|s| s.id), Some(info.id));
    assert!(!status.locked);
    let last: Option<String> = raw(&dir)
        .query_row("SELECT last_login_at FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(last.as_deref(), Some("2026-10-10T09:00:00.000Z"));
    assert_eq!(audit_count(&dir, "login"), 1);
    // User names are not case sensitive, and trimmed.
    auth::logout(&env.ctx).unwrap();
    assert!(auth::login(&env.ctx, "  OWNER ", &owner_secret()).is_ok());
}

#[test]
fn empty_fields_are_required_and_not_counted() {
    let (dir, env) = env();
    set_up(&env);
    let error = auth::login(&env.ctx, " ", &Secret::new("")).unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!()
    };
    let fields: Vec<_> = items.iter().map(|i| i.field.clone().unwrap()).collect();
    assert_eq!(fields, ["username", "password"]);
    assert_eq!(audit_count(&dir, "login_failed"), 0);
    assert_eq!(failed_attempts(&dir), 0);
}

/// The worked table of plan 04 Section 3, step by step, with the audit rows.
#[test]
fn the_lockout_table() {
    let (dir, env) = env();
    set_up(&env);
    let mut rows = 0;
    let expect_wrong = |env: &mandi_services::testing::TestEnv, left: i64| {
        let error = wrong_try(env, "owner");
        assert_eq!(
            code_and_value(&error, "tries_left"),
            (ErrorCode::ErrWrongPassword, Some(left))
        );
    };
    // Tries 1 to 4.
    for left in [4, 3, 2, 1] {
        expect_wrong(&env, left);
        rows += 1;
    }
    // Try 5: locked for 60 seconds.
    let error = wrong_try(&env, "owner");
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(60))
    );
    rows += 1;
    assert_eq!(failed_attempts(&dir), 5);
    // While locked, even the right password is refused, the wait runs down, and nothing is counted.
    env.mono.advance_seconds(20);
    let error = auth::login(&env.ctx, "owner", &owner_secret()).unwrap_err();
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(40))
    );
    rows += 1; // a blocked try is written down too
    assert_eq!(failed_attempts(&dir), 5);
    env.mono.advance_seconds(40);
    // Tries 6 to 9: wrong password again, 4, 3, 2, 1 tries left.
    for left in [4, 3, 2, 1] {
        expect_wrong(&env, left);
        rows += 1;
    }
    // Try 10: locked 300 seconds.
    let error = wrong_try(&env, "owner");
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(300))
    );
    rows += 1;
    env.mono.advance_seconds(299);
    assert_eq!(
        code_and_value(&wrong_try(&env, "owner"), "seconds"),
        (ErrorCode::ErrLockedWait, Some(1))
    );
    rows += 1;
    env.mono.advance_seconds(1);
    // Tries 11 to 14, then 15 is locked 900 seconds, and so are 20 and 25.
    for step in 0..3 {
        for left in [4, 3, 2, 1] {
            expect_wrong(&env, left);
            rows += 1;
        }
        let error = wrong_try(&env, "owner");
        assert_eq!(
            code_and_value(&error, "seconds"),
            (ErrorCode::ErrLockedWait, Some(900)),
            "step {step}"
        );
        rows += 1;
        env.mono.advance_seconds(900);
    }
    assert_eq!(failed_attempts(&dir), 25);
    // The right password after the wait signs in and sets the count back to 0.
    login_owner(&env);
    assert_eq!(failed_attempts(&dir), 0);
    let stored_lock: Option<String> = raw(&dir)
        .query_row("SELECT locked_until_at FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored_lock, None);
    auth::logout(&env.ctx).unwrap();
    expect_wrong(&env, 4);
    rows += 1;
    // Every try wrote one audit row.
    assert_eq!(audit_count(&dir, "login_failed"), rows);
    assert_chain_ok(&dir);
}

#[test]
fn an_unknown_name_behaves_exactly_like_a_wrong_password() {
    let (dir, env) = env();
    set_up(&env);
    for left in [4, 3, 2, 1] {
        let error = wrong_try(&env, "nobody");
        assert_eq!(
            code_and_value(&error, "tries_left"),
            (ErrorCode::ErrWrongPassword, Some(left))
        );
    }
    let error = wrong_try(&env, "nobody");
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(60))
    );
    // The typed name is in the audit rows, with no user.
    let conn = raw(&dir);
    let rows: Vec<(Option<i64>, String)> = conn
        .prepare(
            "SELECT user_id, new_json FROM audit_log WHERE action = 'login_failed' ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(rows.len(), 5);
    assert!(
        rows.iter()
            .all(|(user, json)| user.is_none() && json.contains("\"username\":\"nobody\""))
    );
    // The real Owner is not affected by the unknown name's lock.
    login_owner(&env);
}

#[test]
fn different_names_have_their_own_counters() {
    let (_dir, env) = env();
    set_up(&env);
    for _ in 0..4 {
        wrong_try(&env, "owner");
    }
    // Another name starts at 4 tries left.
    assert_eq!(
        code_and_value(&wrong_try(&env, "someone"), "tries_left"),
        (ErrorCode::ErrWrongPassword, Some(4))
    );
}

#[test]
fn moving_the_clock_during_a_lock_changes_nothing_while_the_app_runs() {
    let (_dir, env) = env();
    set_up(&env);
    for _ in 0..5 {
        wrong_try(&env, "owner");
    }
    // The wall clock goes back a day: the lock is not longer than its step.
    env.clock.advance_seconds(-86_400);
    env.mono.advance_seconds(10);
    let error = auth::login(&env.ctx, "owner", &owner_secret()).unwrap_err();
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(50))
    );
    // The wall clock goes a year forward: the lock is not over early.
    env.clock.advance_seconds(86_400 * 365);
    env.mono.advance_seconds(10);
    let error = auth::login(&env.ctx, "owner", &owner_secret()).unwrap_err();
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(40))
    );
    // Only the monotonic time ends it.
    env.mono.advance_seconds(40);
    assert!(auth::login(&env.ctx, "owner", &owner_secret()).is_ok());
}

#[test]
fn a_lock_that_ended_cannot_come_back_when_the_clock_moves_back() {
    let (_dir, env) = env();
    set_up(&env);
    for _ in 0..5 {
        wrong_try(&env, "owner");
    }
    env.mono.advance_seconds(61);
    // The lock is over. Moving the wall clock back makes the stored end time look new again.
    env.clock.advance_seconds(-3 * 3600);
    assert_eq!(
        code_and_value(&wrong_try(&env, "owner"), "tries_left"),
        (ErrorCode::ErrWrongPassword, Some(4))
    );
    assert!(auth::login(&env.ctx, "owner", &owner_secret()).is_ok());
}

#[test]
fn a_restart_keeps_the_lock_but_never_for_longer_than_its_step() {
    let (_dir, env) = env();
    set_up(&env);
    for _ in 0..5 {
        wrong_try(&env, "owner");
    }
    // A new run of the app: a new context on the same file, with no memory of the lock.
    let restarted = |env: &mandi_services::testing::TestEnv| {
        mandi_services::Context::with_parts(
            env.ctx.db().clone(),
            mandi_services::Parts {
                monotonic: std::sync::Arc::new(mandi_services::testing::ManualMonotonic::default()),
                hasher: mandi_services::testing::fast_hasher().unwrap(),
                random: std::sync::Arc::new(mandi_services::testing::CountingRandom::default()),
            },
        )
    };
    env.clock.advance_seconds(15);
    let fresh = restarted(&env);
    let error = auth::login(&fresh, "owner", &owner_secret()).unwrap_err();
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(45)),
        "from the stored end time"
    );
    // Restart again with the wall clock moved a day back: still at most the 60 second step.
    env.clock.advance_seconds(-86_400);
    let fresh = restarted(&env);
    let error = auth::login(&fresh, "owner", &owner_secret()).unwrap_err();
    let (code, seconds) = code_and_value(&error, "seconds");
    assert_eq!(code, ErrorCode::ErrLockedWait);
    assert_eq!(seconds, Some(60));
}

#[test]
fn an_inactive_user_with_the_right_password_is_refused() {
    let (dir, env) = env();
    set_up(&env);
    let info = login_owner(&env);
    auth::logout(&env.ctx).unwrap();
    // Made inactive outside the app (the app has no command for it).
    raw(&dir)
        .execute("UPDATE users SET is_active = 0 WHERE id = ?1", [info.id])
        .unwrap();
    let error = auth::login(&env.ctx, "owner", &owner_secret()).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrUserInactive);
    // A wrong password for him is still a wrong password, and counts.
    assert_eq!(
        code_and_value(&wrong_try(&env, "owner"), "tries_left"),
        (ErrorCode::ErrWrongPassword, Some(4))
    );
    assert!(audit_count(&dir, "login_failed") >= 2);
}

#[test]
fn only_one_user_is_signed_in_at_a_time() {
    let (_dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let error = auth::login(&env.ctx, "owner", &owner_secret()).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrAlreadySignedIn);
    auth::logout(&env.ctx).unwrap();
    assert!(auth::status(&env.ctx).unwrap().session.is_none());
    assert_eq!(
        auth::logout(&env.ctx).unwrap_err().code(),
        ErrorCode::ErrNotLoggedIn
    );
}

#[test]
fn logout_writes_an_audit_row() {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    auth::logout(&env.ctx).unwrap();
    assert_eq!(audit_count(&dir, "login"), 1);
    assert_eq!(audit_count(&dir, "logout"), 1);
    assert_chain_ok(&dir);
}

#[test]
fn idle_lock_keeps_the_session_and_unlock_needs_the_password() {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    auth::lock(&env.ctx).unwrap();
    auth::lock(&env.ctx).unwrap(); // locking twice does nothing more
    assert_eq!(audit_count(&dir, "lock"), 1);
    let status = auth::status(&env.ctx).unwrap();
    assert!(
        status.locked && status.session.is_some(),
        "the name is shown on the lock screen"
    );
    // Every command is refused while locked, and writes nothing.
    let before = audit_total(&dir);
    assert_eq!(
        auth::require(&env.ctx).unwrap_err().code(),
        ErrorCode::ErrSessionLocked
    );
    assert_eq!(audit_total(&dir), before);
    // A wrong password does not unlock; the right one does.
    let error = auth::unlock(&env.ctx, &Secret::new("wrong-wrong")).unwrap_err();
    assert_eq!(
        code_and_value(&error, "tries_left"),
        (ErrorCode::ErrWrongPassword, Some(4))
    );
    assert!(auth::status(&env.ctx).unwrap().locked);
    auth::unlock(&env.ctx, &owner_secret()).unwrap();
    assert!(!auth::status(&env.ctx).unwrap().locked);
    assert!(auth::require(&env.ctx).is_ok());
    assert_eq!(audit_count(&dir, "unlock"), 1);
    assert_eq!(
        auth::unlock(&env.ctx, &owner_secret()).unwrap_err().code(),
        ErrorCode::ErrNotLocked
    );
}

#[test]
fn unlock_uses_the_same_lockout_counter() {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    auth::lock(&env.ctx).unwrap();
    for _ in 0..4 {
        auth::unlock(&env.ctx, &Secret::new("nope-nope")).unwrap_err();
    }
    let error = auth::unlock(&env.ctx, &Secret::new("nope-nope")).unwrap_err();
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(60))
    );
    assert_eq!(failed_attempts(&dir), 5);
    // Even the right password waits.
    assert_eq!(
        auth::unlock(&env.ctx, &owner_secret()).unwrap_err().code(),
        ErrorCode::ErrLockedWait
    );
    // He can still sign out of the locked screen.
    auth::logout(&env.ctx).unwrap();
}

#[test]
fn recovery_resets_the_owner_password_and_spends_the_code() {
    let (dir, env) = env();
    let code = set_up(&env);
    let recover = |code: &str, password: &str| {
        auth::recover(
            &env.ctx,
            &auth::RecoverRaw {
                username: "owner".into(),
                recovery_code: Secret::new(code),
                new_password: Secret::new(password),
                new_password_repeat: Secret::new(password),
            },
        )
    };
    // Locked out first: the recovery path is how he gets back in.
    for _ in 0..5 {
        wrong_try(&env, "owner");
    }
    env.mono.advance_seconds(61);
    // A lower case code without dashes works too.
    let new_code = recover(&code.replace('-', "").to_lowercase(), "brand-new-1").unwrap();
    assert_eq!(new_code.len(), 19);
    assert_ne!(new_code, code);
    assert_eq!(failed_attempts(&dir), 0, "recovery ends the lockout");
    assert_eq!(audit_count(&dir, "recover"), 1);
    // The new password works; the old one does not.
    assert!(auth::login(&env.ctx, "owner", &Secret::new("brand-new-1")).is_ok());
    auth::logout(&env.ctx).unwrap();
    assert_eq!(wrong_try(&env, "owner").code(), ErrorCode::ErrWrongPassword);
    // The used code is spent. The new code works once.
    let error = recover(&code, "another-1").unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrRecoveryCodeWrong);
    assert!(recover(&new_code, "third-pass1").is_ok());
    assert_eq!(
        recover(&new_code, "fourth-pass1").unwrap_err().code(),
        ErrorCode::ErrRecoveryCodeWrong
    );
    assert_chain_ok(&dir);
}

#[test]
fn wrong_recovery_codes_count_as_wrong_tries_and_lock() {
    let (dir, env) = env();
    let code = set_up(&env);
    let recover = |name: &str, code: &str| {
        auth::recover(
            &env.ctx,
            &auth::RecoverRaw {
                username: name.into(),
                recovery_code: Secret::new(code),
                new_password: Secret::new("new-pass-1"),
                new_password_repeat: Secret::new("new-pass-1"),
            },
        )
        .unwrap_err()
    };
    for left in [4, 3, 2, 1] {
        let error = recover("owner", "ABCD-EFGH-JKMN-PQRS");
        assert_eq!(
            code_and_value(&error, "tries_left"),
            (ErrorCode::ErrRecoveryCodeWrong, Some(left))
        );
    }
    let error = recover("owner", "not a code");
    assert_eq!(
        code_and_value(&error, "seconds"),
        (ErrorCode::ErrLockedWait, Some(60))
    );
    // The right code is refused while locked, and the password was not changed.
    assert_eq!(recover("owner", &code).code(), ErrorCode::ErrLockedWait);
    assert_eq!(failed_attempts(&dir), 5);
    env.mono.advance_seconds(60);
    assert!(auth::login(&env.ctx, "owner", &owner_secret()).is_ok());
    // An unknown name gets the same answer as a known one.
    assert_eq!(
        code_and_value(&recover("nobody", &code), "tries_left"),
        (ErrorCode::ErrRecoveryCodeWrong, Some(4))
    );
}

#[test]
fn recovery_is_for_the_owner_the_code_belongs_to_only() {
    let (_dir, env) = env();
    let code = set_up(&env);
    // The right code with a name that is not the Owner's is refused like a wrong code.
    let error = auth::recover(
        &env.ctx,
        &auth::RecoverRaw {
            username: "ali".into(),
            recovery_code: Secret::new(code),
            new_password: Secret::new("hacked-pass1"),
            new_password_repeat: Secret::new("hacked-pass1"),
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrRecoveryCodeWrong);
    assert!(auth::login(&env.ctx, "owner", &owner_secret()).is_ok());
}

#[test]
fn recovery_checks_the_new_password_first_without_counting() {
    let (dir, env) = env();
    let code = set_up(&env);
    let error = auth::recover(
        &env.ctx,
        &auth::RecoverRaw {
            username: "owner".into(),
            recovery_code: Secret::new(code),
            new_password: Secret::new("abc"),
            new_password_repeat: Secret::new("abd"),
        },
    )
    .unwrap_err();
    assert_eq!(
        code_and_value(&error, "min"),
        (ErrorCode::ErrPasswordTooShort, Some(6))
    );
    assert_eq!(failed_attempts(&dir), 0);
}

#[test]
fn no_session_means_not_logged_in_and_no_row() {
    let (dir, env) = env();
    set_up(&env);
    let before = audit_total(&dir);
    assert_eq!(
        auth::require(&env.ctx).unwrap_err().code(),
        ErrorCode::ErrNotLoggedIn
    );
    assert_eq!(audit_total(&dir), before);
}

#[test]
fn a_user_made_inactive_loses_the_session_at_once() {
    let (dir, env) = env();
    set_up(&env);
    let info = login_owner(&env);
    // Made inactive outside the app (the app has no command for it): his next call finds no session.
    raw(&dir)
        .execute("UPDATE users SET is_active = 0 WHERE id = ?1", [info.id])
        .unwrap();
    assert_eq!(
        auth::require(&env.ctx).unwrap_err().code(),
        ErrorCode::ErrNotLoggedIn
    );
    assert!(auth::status(&env.ctx).unwrap().session.is_none());
}

#[test]
fn two_wrong_tries_at_the_same_time_are_both_counted() {
    let (dir, env) = env();
    set_up(&env);
    let ctx = env.ctx.clone();
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                auth::login(&ctx, "owner", &Secret::new("wrong-pass"))
                    .unwrap_err()
                    .code()
            })
        })
        .collect();
    for t in threads {
        assert_eq!(t.join().unwrap(), ErrorCode::ErrWrongPassword);
    }
    assert_eq!(failed_attempts(&dir), 4);
}

#[test]
fn a_huge_password_is_a_wrong_password_and_is_not_hashed() {
    let (dir, env) = env();
    set_up(&env);
    let started = std::time::Instant::now();
    let huge = Secret::new("x".repeat(5_000_000));
    let error = auth::login(&env.ctx, "owner", &huge).unwrap_err();
    assert_eq!(
        code_and_value(&error, "tries_left"),
        (ErrorCode::ErrWrongPassword, Some(4))
    );
    assert!(
        started.elapsed().as_millis() < 1000,
        "took {:?}",
        started.elapsed()
    );
    assert_eq!(failed_attempts(&dir), 1);
    // The same for an unknown name, and for the recovery code.
    assert_eq!(
        auth::login(&env.ctx, "nobody", &huge).unwrap_err().code(),
        ErrorCode::ErrWrongPassword
    );
    let error = auth::recover(
        &env.ctx,
        &auth::RecoverRaw {
            username: "owner".into(),
            recovery_code: Secret::new("A".repeat(5_000_000)),
            new_password: Secret::new("new-pass-1"),
            new_password_repeat: Secret::new("new-pass-1"),
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrRecoveryCodeWrong);
    // A 128 character password is fine.
    let mut input = first_run_input();
    input.password = Secret::new("p".repeat(129));
    input.password_repeat = Secret::new("p".repeat(129));
    let (_dir2, env2) = common::env();
    assert!(
        auth::first_run(&env2.ctx, &input)
            .unwrap_err()
            .has_code(ErrorCode::ErrTextTooLong)
    );
}

#[test]
fn two_first_runs_at_the_same_time_make_one_owner() {
    let (dir, env) = env();
    let results: Vec<_> = (0..4)
        .map(|_| {
            let ctx = env.ctx.clone();
            std::thread::spawn(move || auth::first_run(&ctx, &first_run_input()))
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|t| t.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    for failed in results.iter().filter_map(|r| r.as_ref().err()) {
        assert_eq!(failed.code(), ErrorCode::ErrAlreadySetUp);
    }
    let users: i64 = raw(&dir)
        .query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(users, 1);
    assert_chain_ok(&dir);
}

#[test]
fn two_recoveries_with_the_same_code_at_the_same_time_have_one_winner() {
    let (dir, env) = env();
    let code = set_up(&env);
    let results: Vec<_> = (0..4)
        .map(|i| {
            let ctx = env.ctx.clone();
            let code = code.clone();
            std::thread::spawn(move || {
                auth::recover(
                    &ctx,
                    &auth::RecoverRaw {
                        username: "owner".into(),
                        recovery_code: Secret::new(code),
                        new_password: Secret::new(format!("new-pass-{i}")),
                        new_password_repeat: Secret::new(format!("new-pass-{i}")),
                    },
                )
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|t| t.join().unwrap())
        .collect();
    assert_eq!(
        results.iter().filter(|r| r.is_ok()).count(),
        1,
        "{results:?}"
    );
    assert_eq!(audit_count(&dir, "recover"), 1);
    assert_chain_ok(&dir);
}
