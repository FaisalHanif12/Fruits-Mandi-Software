//! Sign-in under attack (D1 attack list items 2 and 4; PRD FR-8.1 to FR-8.6, NFR-SEC-1, NFR-SEC-2,
//! NFR-SEC-12, NFR-REL-11, NFR-OBS-2, Code Standards S2). Owner: `robustness`.
//!
//! - 1,000 wrong passwords in a row follow the lockout table and the right one still works.
//! - Hammering a locked name is cheap, does not escalate the lock and is bounded.
//! - Passwords: 10,000 characters, Urdu letters, spaces, the longest allowed, the shortest.
//! - A restart or a clock change never makes a lock longer than its step.
//! - No password, recovery code, hash or key reaches a log line, an error text, an audit row or a
//!   data file in plain text; hashes are Argon2id with a salt of their own.
#![cfg(test)]

mod common;

use std::fmt::Write as _;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use chrono::TimeZone;
use common::*;
use mandi_services::core::error::ErrorCode;
use mandi_services::ports::{Argon2Hasher, OsRandom, PasswordHasher};
use mandi_services::testing::TestEnv;
use mandi_services::{Secret, ServiceError, auth};
use tempfile::TempDir;
use tracing::field::{Field, Visit};

// ---------------------------------------------------------------------------------------------
// A log catcher: every tracing event of the whole test program, as text.
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<String>>);

struct Text<'a>(&'a mut String);

impl Visit for Text<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let _ = write!(self.0, "{}={:?} ", field.name(), value);
    }
}

impl tracing::Subscriber for Capture {
    fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        let mut line = format!("{} ", event.metadata().target());
        event.record(&mut Text(&mut line));
        if let Ok(mut all) = self.0.lock() {
            all.push_str(&line);
            all.push('\n');
        }
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}

fn logs() -> &'static Capture {
    static LOGS: OnceLock<Capture> = OnceLock::new();
    LOGS.get_or_init(|| {
        let capture = Capture::default();
        let _ = tracing::subscriber::set_global_default(capture.clone());
        capture
    })
}

fn captured() -> String {
    logs().0.lock().map(|g| g.clone()).unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------

fn setup_with(password: &str) -> (TempDir, TestEnv, Result<String, ServiceError>) {
    let (dir, env) = env();
    let mut input = first_run_input();
    input.password = Secret::new(password);
    input.password_repeat = Secret::new(password);
    let result = auth::first_run(&env.ctx, &input);
    (dir, env, result)
}

fn locked_seconds(error: &ServiceError) -> Option<i64> {
    match error {
        ServiceError::Rejected(items) => items.iter().find_map(|i| {
            (i.code == ErrorCode::ErrLockedWait)
                .then(|| match i.values.get("seconds") {
                    Some(mandi_services::core::error::ErrorValue::Number(n)) => Some(*n),
                    _ => None,
                })
                .flatten()
        }),
        _ => None,
    }
}

#[test]
fn a_thousand_wrong_passwords_follow_the_lockout_table_and_the_right_one_still_works() {
    let (dir, env) = env();
    set_up(&env);
    let wrong = Secret::new("not-the-password");
    let mut locks: Vec<(i64, i64)> = Vec::new(); // (wrong tries so far, seconds) at each new lock
    let attempts = |dir: &TempDir| -> i64 {
        raw(dir)
            .query_row("SELECT failed_attempts FROM users", [], |r| r.get(0))
            .unwrap()
    };
    let mut guard = 0;
    while attempts(&dir) < 1_000 {
        guard += 1;
        assert!(guard < 5_000, "the loop never ended");
        let before = attempts(&dir);
        match auth::login(&env.ctx, "owner", &wrong) {
            Err(error) => {
                let after = attempts(&dir);
                if let Some(seconds) = locked_seconds(&error) {
                    if after > before {
                        // This very try was the one that started a lock.
                        locks.push((after, seconds));
                    }
                    // Wait it out, as the person at the counter would. A try during a lock is
                    // refused and is not counted as a wrong try.
                    assert!(after >= before && after - before <= 1);
                    env.mono
                        .advance_seconds(u64::try_from(seconds).unwrap() + 1);
                } else {
                    assert_eq!(after, before + 1, "a wrong try was not counted");
                    assert!(error.has_code(ErrorCode::ErrWrongPassword), "{error:?}");
                }
            }
            Ok(_) => panic!("a wrong password signed in"),
        }
    }
    // PRD FR-8.2: 5 tries then 1 minute; 5 more then 5 minutes; then 15 minutes each time.
    for (n, seconds) in &locks {
        let want = match n {
            5 => 60,
            10 => 300,
            _ => 900,
        };
        assert_eq!(*seconds, want, "after try {n}");
    }
    // A lock after every 5th wrong try: 200 of them in 1,000 tries.
    assert_eq!(locks.len(), 200, "{locks:?}");
    assert_eq!(
        locks.iter().map(|l| l.0).collect::<Vec<_>>(),
        (1..=200).map(|n| n * 5).collect::<Vec<_>>()
    );
    // Every wrong try was written down once; the refused tries during locks wrote their own rows.
    let wrong_rows = raw(&dir)
        .query_row("SELECT COUNT(*) FROM audit_log WHERE action = 'login_failed' AND new_json LIKE '%wrong_secret%'", [], |r| r.get::<_, i64>(0))
        .unwrap();
    assert_eq!(wrong_rows, 1_000);
    assert_chain_ok(&dir);
    // The right password, after the last lock ends, signs in and resets the count.
    env.mono.advance_seconds(1_000);
    auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
    let attempts: i64 = raw(&dir)
        .query_row("SELECT failed_attempts FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(attempts, 0);
}

#[test]
fn hammering_a_locked_name_is_cheap_does_not_escalate_and_is_logged_in_a_bounded_way() {
    let (dir, env) = env();
    set_up(&env);
    let wrong = Secret::new("not-the-password");
    for _ in 0..5 {
        let _ = auth::login(&env.ctx, "owner", &wrong);
    }
    let rows_before = audit_total(&dir);
    let started = Instant::now();
    let hammer = 3_000;
    for i in 0..hammer {
        let error = auth::login(&env.ctx, "owner", &wrong).unwrap_err();
        assert!(locked_seconds(&error).is_some(), "{error:?}");
        // A right password does not get through a lock either.
        if i % 500 == 0 {
            assert!(auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).is_err());
        }
    }
    let took = started.elapsed();
    let per_try = took / hammer;
    println!(
        "{hammer} tries during a lock: {took:?} ({per_try:?} each), audit rows added: {}",
        audit_total(&dir) - rows_before
    );
    assert!(
        per_try < Duration::from_millis(25),
        "a refused try during a lock took {per_try:?}: it must not hash"
    );
    let attempts: i64 = raw(&dir)
        .query_row("SELECT failed_attempts FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        attempts, 5,
        "tries during a lock must not count and escalate the lock"
    );
    // After the lock, one more wrong try is the 6th, not the 3000th.
    env.mono.advance_seconds(61);
    let error = auth::login(&env.ctx, "owner", &wrong).unwrap_err();
    assert!(error.has_code(ErrorCode::ErrWrongPassword), "{error:?}");
    // The data file must not have grown without bound: under 1 KB per refused try.
    let rows_added = audit_total(&dir) - rows_before;
    assert!(
        rows_added <= i64::from(hammer) + 10,
        "{rows_added} audit rows for {hammer} tries"
    );
    assert_chain_ok(&dir);
}

#[test]
fn passwords_of_every_kind_work_or_are_refused_clearly() {
    let longest = "x".repeat(128);
    let emoji_128 = "😀".repeat(128); // 512 bytes
    let urdu = "پاسورڈ٣٢١abc";
    for (label, password, accepted) in [
        ("shortest (6)", "abc123".to_owned(), true),
        ("5 characters", "abc12".to_owned(), false),
        ("empty", String::new(), false),
        ("longest (128)", longest.clone(), true),
        ("129 characters", "x".repeat(129), false),
        ("128 emoji, 512 bytes", emoji_128.clone(), true),
        ("129 emoji", "😀".repeat(129), false),
        ("10,000 characters", "y".repeat(10_000), false),
        ("a million characters", "z".repeat(1_000_000), false),
        ("Urdu letters and digits", urdu.to_owned(), true),
        ("spaces inside", "a b c d e f".to_owned(), true),
        ("spaces at the ends", "  secret  ".to_owned(), true),
        ("only spaces", "      ".to_owned(), true),
        ("a tab and a newline", "ab\tc\nde".to_owned(), true),
        ("a NUL byte", "ab\0cdef".to_owned(), true),
    ] {
        let started = Instant::now();
        let (dir, env, result) = setup_with(&password);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{label}: took {:?}",
            started.elapsed()
        );
        match (&result, accepted) {
            (Ok(_), true) => {
                // The password that was set is the one that signs in, exactly.
                let signed = auth::login(&env.ctx, "owner", &Secret::new(password.clone()));
                assert!(
                    signed.is_ok(),
                    "{label}: the password just set does not sign in: {signed:?}"
                );
                // Near misses do not.
                auth::logout(&env.ctx).unwrap();
                let mut near = password.clone();
                near.push('x');
                assert!(
                    auth::login(&env.ctx, "owner", &Secret::new(near)).is_err(),
                    "{label}: a longer text signed in"
                );
                if password.chars().count() > 1 {
                    let shorter: String = password
                        .chars()
                        .take(password.chars().count() - 1)
                        .collect();
                    assert!(
                        auth::login(&env.ctx, "owner", &Secret::new(shorter)).is_err(),
                        "{label}: a shorter text signed in"
                    );
                }
            }
            (Err(error), false) => {
                assert!(
                    matches!(error, ServiceError::Rejected(_)),
                    "{label}: {error:?}"
                );
                assert_eq!(
                    audit_total(&dir),
                    0,
                    "{label}: a refused wizard wrote audit rows"
                );
                assert_eq!(
                    raw(&dir)
                        .query_row("SELECT COUNT(*) FROM users", [], |r| r.get::<_, i64>(0))
                        .unwrap(),
                    0,
                    "{label}: a user was made"
                );
            }
            (Ok(_), false) => panic!("{label}: accepted but must be refused"),
            (Err(error), true) => panic!("{label}: refused but must be accepted: {error:?}"),
        }
    }
}

#[test]
fn a_huge_text_typed_at_the_sign_in_is_a_wrong_password_and_is_quick() {
    let (_dir, env) = env();
    set_up(&env);
    for size in [513_usize, 10_000, 1_000_000, 20_000_000] {
        let started = Instant::now();
        let error = auth::login(&env.ctx, "owner", &Secret::new("p".repeat(size))).unwrap_err();
        assert!(
            error.has_code(ErrorCode::ErrWrongPassword) || locked_seconds(&error).is_some(),
            "{size}: {error:?}"
        );
        assert!(
            started.elapsed() < Duration::from_millis(500),
            "{size} characters took {:?}",
            started.elapsed()
        );
        env.mono.advance_seconds(1_000);
    }
    // A huge user name too: it is cut for the audit log and the counter.
    let started = Instant::now();
    let error = auth::login(&env.ctx, &"n".repeat(5_000_000), &Secret::new("secret1")).unwrap_err();
    assert!(
        error.has_code(ErrorCode::ErrWrongPassword) || matches!(error, ServiceError::Rejected(_)),
        "{error:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn the_same_password_typed_in_a_different_but_equal_unicode_form() {
    // Urdu "آ" can be one letter (U+0622) or alef plus madda (U+0627 U+0653). Two keyboards can
    // give the two forms. Record what the app does; the rule (PRD) is silent.
    let composed = "پاس\u{0622}ورڈ12";
    let decomposed = "پاس\u{0627}\u{0653}ورڈ12";
    assert_ne!(composed, decomposed);
    let (_dir, env, result) = setup_with(composed);
    result.unwrap();
    let same = auth::login(&env.ctx, "owner", &Secret::new(composed)).is_ok();
    assert!(same);
    auth::logout(&env.ctx).unwrap();
    let other = auth::login(&env.ctx, "owner", &Secret::new(decomposed));
    println!("equal Unicode form accepted at sign-in: {}", other.is_ok());
}

#[test]
fn a_restart_or_a_clock_change_never_makes_a_lock_longer_than_its_step() {
    // NFR-REL-11. Steps: 5 tries = 60 s, 10 tries = 300 s, 15 tries = 900 s.
    for (tries, step) in [(5_u32, 60_i64), (10, 300), (15, 900), (25, 900)] {
        for (label, new_clock) in [
            (
                "a year back",
                chrono::Utc
                    .with_ymd_and_hms(2025, 10, 10, 9, 0, 0)
                    .single()
                    .unwrap(),
            ),
            (
                "ten years back",
                chrono::Utc
                    .with_ymd_and_hms(2016, 1, 1, 0, 0, 0)
                    .single()
                    .unwrap(),
            ),
            (
                "a day back",
                chrono::Utc
                    .with_ymd_and_hms(2026, 10, 9, 9, 0, 0)
                    .single()
                    .unwrap(),
            ),
        ] {
            let (dir, env) = env();
            set_up(&env);
            let wrong = Secret::new("not-the-password");
            let mut made = 0;
            while made < tries {
                match auth::login(&env.ctx, "owner", &wrong) {
                    Err(error) if locked_seconds(&error).is_some() => {
                        env.mono.advance_seconds(
                            u64::try_from(locked_seconds(&error).unwrap()).unwrap() + 1,
                        );
                    }
                    Err(_) => made += 1,
                    Ok(_) => panic!("signed in with a wrong password"),
                }
            }
            drop(env);
            // The app is closed and opened again; the computer clock has been put back.
            let restarted = TestEnv::new(dir.path()).unwrap();
            restarted.clock.set(new_clock);
            let error =
                auth::login(&restarted.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap_err();
            let seconds = locked_seconds(&error)
                .unwrap_or_else(|| panic!("{tries} tries, {label}: not locked: {error:?}"));
            assert!(
                seconds <= step,
                "{tries} tries, clock {label}: locked for {seconds} s, more than its step of {step} s"
            );
            // The clock then goes a year forward: the lock is over.
            restarted.clock.set(
                chrono::Utc
                    .with_ymd_and_hms(2027, 10, 10, 9, 0, 0)
                    .single()
                    .unwrap(),
            );
            restarted.mono.advance_seconds(1);
        }
    }
    // A clock far in the future before a restart ends the stored lock at once (that is what a
    // stored end time means), and a right password then signs in.
    let (dir, env) = env();
    set_up(&env);
    for _ in 0..5 {
        let _ = auth::login(&env.ctx, "owner", &Secret::new("not-the-password"));
    }
    drop(env);
    let restarted = TestEnv::new(dir.path()).unwrap();
    restarted.clock.set(
        chrono::Utc
            .with_ymd_and_hms(2030, 1, 1, 0, 0, 0)
            .single()
            .unwrap(),
    );
    auth::login(&restarted.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
}

#[test]
fn hashes_are_argon2id_with_the_recommended_cost_and_a_salt_each() {
    let hasher = Argon2Hasher::recommended(OsRandom);
    let started = Instant::now();
    let one = hasher.hash(&Secret::new("same-password")).unwrap();
    let hash_time = started.elapsed();
    let two = hasher.hash(&Secret::new("same-password")).unwrap();
    assert!(one.starts_with("$argon2id$v=19$"), "{one}");
    assert_ne!(one, two, "two hashes of one password are equal: no salt");
    // m = 19 MiB (19456 KiB), t = 2, p = 1: the OWASP minimum for Argon2id.
    let params = one.split('$').nth(3).unwrap();
    let get = |k: &str| -> u32 {
        params
            .split(',')
            .find_map(|p| p.strip_prefix(&format!("{k}=")))
            .unwrap()
            .parse()
            .unwrap()
    };
    assert!(get("m") >= 19_456, "memory {} KiB", get("m"));
    assert!(get("t") >= 2, "passes {}", get("t"));
    assert!(get("p") >= 1);
    assert!(hasher.verify(&Secret::new("same-password"), &one));
    assert!(!hasher.verify(&Secret::new("same-passworD"), &one));
    assert!(!hasher.verify(&Secret::new("same-password"), "not a hash"));
    assert!(!hasher.verify(&Secret::new("same-password"), ""));
    assert!(!hasher.verify(&Secret::new(""), &one));
    println!("one Argon2id hash at the recommended cost: {hash_time:?} (debug build of the test)");
    assert!(hash_time < Duration::from_millis(1_500), "{hash_time:?}");
}

#[test]
fn no_password_code_or_key_reaches_a_log_an_error_an_audit_row_or_a_data_file() {
    logs();
    tracing::info!("robustness marker: the log catcher works");
    let (dir, env) = env();
    let p1 = "Zq7!hunter-2ÅÆ پاس";
    let p2 = "N3w-Zq88-پاسورڈ";
    let w1 = "Wr0ng-Zq77-secret";
    let w2 = "Wr0ng-Un10ck-Zq99";
    let mut input = first_run_input();
    input.password = Secret::new(p1);
    input.password_repeat = Secret::new(p1);
    let code = auth::first_run(&env.ctx, &input).unwrap();
    let mut texts: Vec<String> = Vec::new();
    let mut note = |what: &str, error: &ServiceError| {
        texts.push(format!("{what}: {error:?} / {error}"));
    };
    for _ in 0..3 {
        note(
            "wrong login",
            &auth::login(&env.ctx, "owner", &Secret::new(w1)).unwrap_err(),
        );
    }
    auth::login(&env.ctx, "owner", &Secret::new(p1)).unwrap();
    auth::lock(&env.ctx).unwrap();
    note(
        "wrong unlock",
        &auth::unlock(&env.ctx, &Secret::new(w2)).unwrap_err(),
    );
    auth::unlock(&env.ctx, &Secret::new(p1)).unwrap();
    auth::logout(&env.ctx).unwrap();
    let recover = |c: &str, np: &str| mandi_services::auth::RecoverRaw {
        username: "owner".into(),
        recovery_code: Secret::new(c),
        new_password: Secret::new(np),
        new_password_repeat: Secret::new(np),
    };
    note(
        "wrong code",
        &auth::recover(&env.ctx, &recover("AAAA-BBBB-CCCC-DDDD", p2)).unwrap_err(),
    );
    let new_code = auth::recover(&env.ctx, &recover(&code, p2)).unwrap();
    auth::login(&env.ctx, "owner", &Secret::new(p2)).unwrap();
    auth::logout(&env.ctx).unwrap();
    // Everything that could carry a secret.
    let needles: Vec<(String, String)> = vec![
        ("first password".into(), p1.into()),
        ("second password".into(), p2.into()),
        ("wrong password 1".into(), w1.into()),
        ("wrong password 2".into(), w2.into()),
        ("recovery code".into(), code.clone()),
        ("recovery code without dashes".into(), code.replace('-', "")),
        ("recovery code lower case".into(), code.to_lowercase()),
        ("new recovery code".into(), new_code.clone()),
        (
            "new recovery code without dashes".into(),
            new_code.replace('-', ""),
        ),
        ("audit key hex".into(), "07".repeat(32)),
    ];
    let log = captured();
    assert!(
        log.contains("robustness marker"),
        "the log catcher caught nothing, so the scan below would show nothing"
    );
    let audit: String = {
        let conn = raw(&dir);
        let mut st = conn.prepare("SELECT COALESCE(old_json,'') || COALESCE(new_json,'') || COALESCE(reason,'') || action || entity FROM audit_log").unwrap();
        st.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .join("\n")
    };
    let errors = texts.join("\n");
    for (name, needle) in &needles {
        assert!(!log.contains(needle.as_str()), "{name} is in a log line");
        assert!(
            !errors.contains(needle.as_str()),
            "{name} is in an error text"
        );
        assert!(
            !audit.contains(needle.as_str()),
            "{name} is in the audit log"
        );
    }
    for hashed in ["$argon2id$", "password_hash"] {
        assert!(!log.contains(hashed), "'{hashed}' is in a log line");
        assert!(!audit.contains(hashed), "'{hashed}' is in the audit log");
        assert!(!errors.contains(hashed), "'{hashed}' is in an error text");
    }
    // The data folder: the database, its log and the shared memory file.
    let mut files = 0;
    for entry in walk(dir.path()) {
        let bytes = std::fs::read(&entry).unwrap();
        files += 1;
        for (name, needle) in &needles {
            assert!(
                !contains(&bytes, needle.as_bytes()),
                "{name} is in the file {}",
                entry.display()
            );
        }
        assert!(
            !contains(&bytes, &[7_u8; 32]),
            "the audit key is in the file {}",
            entry.display()
        );
    }
    assert!(files >= 1);
    // What is stored for the password is a salted Argon2id hash (and only there).
    let stored: String = raw(&dir)
        .query_row("SELECT password_hash FROM users", [], |r| r.get(0))
        .unwrap();
    assert!(stored.starts_with("$argon2id$"), "{stored}");
    let recovery: String = raw(&dir)
        .query_row(
            "SELECT value_json FROM settings WHERE key LIKE '%recovery%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        recovery.contains("$argon2id$"),
        "the recovery code is not stored as an Argon2id hash: {recovery}"
    );
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walk(&path));
            } else {
                out.push(path);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Recovery code and the order of sign-in, lock and sign-out (PRD FR-8.4, FR-8.5, NFR-SEC-12).
// ---------------------------------------------------------------------------------------------

fn recover_input(code: &str, new_password: &str) -> auth::RecoverRaw {
    auth::RecoverRaw {
        username: "owner".into(),
        recovery_code: Secret::new(code),
        new_password: Secret::new(new_password),
        new_password_repeat: Secret::new(new_password),
    }
}

fn password_hash(dir: &TempDir) -> String {
    raw(dir)
        .query_row("SELECT password_hash FROM users", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn wrong_recovery_codes_follow_the_lockout_and_the_right_code_works_once_only() {
    let (dir, env) = env();
    let code = set_up(&env);
    let hash_before = password_hash(&dir);
    // Wrong codes of every shape: none may open the account, all count as wrong tries.
    let guesses = [
        "AAAA-BBBB-CCCC-DDDD".to_owned(),
        "0000-0000-0000-0000".to_owned(),
        " ".to_owned(),
        "x".repeat(10_000),
        "'; DROP TABLE users; --".to_owned(),
        "رحیم".to_owned(),
        "😀😀😀😀".to_owned(),
        // The real code with one letter changed.
        {
            let mut changed = code.clone();
            let last = changed.pop().unwrap_or('A');
            changed.push(if last == 'A' { 'B' } else { 'A' });
            changed
        },
    ];
    let mut locked = false;
    for guess in &guesses {
        match auth::recover(&env.ctx, &recover_input(guess, "newsecret1")) {
            Ok(_) => panic!("a wrong recovery code was accepted: {guess:.30}"),
            Err(error) => {
                if locked_seconds(&error).is_some() {
                    locked = true;
                } else {
                    // A blank code is a missing field; every other guess is a wrong code.
                    assert!(
                        error.has_code(ErrorCode::ErrRecoveryCodeWrong)
                            || error.has_code(ErrorCode::ErrRequired),
                        "{error:?}"
                    );
                }
            }
        }
    }
    assert!(
        locked,
        "{} wrong codes in a row never started a lock",
        guesses.len()
    );
    assert_eq!(
        password_hash(&dir),
        hash_before,
        "the password changed without the code"
    );
    // While the lock runs, even the right code is refused and nothing changes.
    let error = auth::recover(&env.ctx, &recover_input(&code, "newsecret1")).unwrap_err();
    assert!(locked_seconds(&error).is_some(), "{error:?}");
    assert_eq!(password_hash(&dir), hash_before);
    env.mono.advance_seconds(3_600);
    // The right code works, gives a new code, and is spent.
    let new_code = auth::recover(&env.ctx, &recover_input(&code, "newsecret1")).unwrap();
    assert_ne!(new_code, code, "the recovery code was not renewed");
    assert_ne!(password_hash(&dir), hash_before);
    let old_again = auth::recover(&env.ctx, &recover_input(&code, "otherpass9")).unwrap_err();
    assert!(
        old_again.has_code(ErrorCode::ErrRecoveryCodeWrong),
        "{old_again:?}"
    );
    auth::login(&env.ctx, "owner", &Secret::new("newsecret1")).unwrap();
    assert!(
        auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).is_err(),
        "the old password still works"
    );
    assert_chain_ok(&dir);
}

#[test]
fn two_recoveries_with_the_same_code_at_the_same_moment_let_one_through() {
    let (dir, env) = env();
    let code = set_up(&env);
    let ctx = env.ctx.clone();
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let results: Vec<Result<String, ServiceError>> = (0..2)
        .map(|n| {
            let ctx = ctx.clone();
            let barrier = barrier.clone();
            let code = code.clone();
            std::thread::spawn(move || {
                barrier.wait();
                auth::recover(&ctx, &recover_input(&code, &format!("password{n}")))
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    let won = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(won, 1, "{results:?}");
    let lost = results.iter().find_map(|r| r.as_ref().err()).unwrap();
    assert!(
        lost.has_code(ErrorCode::ErrRecoveryCodeWrong) || locked_seconds(lost).is_some(),
        "{lost:?}"
    );
    // Exactly one password is in force, and it is the winner's.
    let winner = results.iter().position(|r| r.is_ok()).unwrap();
    auth::login(&env.ctx, "owner", &Secret::new(format!("password{winner}"))).unwrap();
    assert_eq!(audit_count(&dir, "recover"), 1);
    assert_chain_ok(&dir);
}

#[test]
fn a_recovery_with_a_bad_new_password_changes_nothing_and_keeps_the_code() {
    let (dir, env) = env();
    let code = set_up(&env);
    let hash_before = password_hash(&dir);
    let cases: [(&str, String, String, ErrorCode); 3] = [
        (
            "5 characters",
            "abc12".into(),
            "abc12".into(),
            ErrorCode::ErrPasswordTooShort,
        ),
        (
            "the two do not match",
            "secret123".into(),
            "secret124".into(),
            ErrorCode::ErrPasswordsDiffer,
        ),
        (
            "empty",
            String::new(),
            String::new(),
            ErrorCode::ErrRequired,
        ),
    ];
    for (label, one, two, want) in cases {
        let input = auth::RecoverRaw {
            username: "owner".into(),
            recovery_code: Secret::new(&code),
            new_password: Secret::new(&one),
            new_password_repeat: Secret::new(&two),
        };
        let error = auth::recover(&env.ctx, &input).unwrap_err();
        assert!(error.has_code(want), "{label}: {error:?}");
        assert_eq!(password_hash(&dir), hash_before, "{label}");
    }
    // A refused request did not spend the code or count as a wrong try.
    let attempts: i64 = raw(&dir)
        .query_row("SELECT failed_attempts FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(attempts, 0);
    auth::recover(&env.ctx, &recover_input(&code, "goodsecret1")).unwrap();
    // Someone else's name with the Owner's code is a wrong code, not a way in.
    let (dir2, env2) = self::env();
    let code2 = set_up(&env2);
    let mut other = recover_input(&code2, "goodsecret1");
    other.username = "someone-else".into();
    assert!(
        auth::recover(&env2.ctx, &other)
            .unwrap_err()
            .has_code(ErrorCode::ErrRecoveryCodeWrong)
    );
    assert_chain_ok(&dir2);
}

#[test]
fn sign_in_lock_unlock_and_sign_out_refuse_every_wrong_order() {
    let (dir, env) = env();
    set_up(&env);
    let right = Secret::new(OWNER_PASSWORD);
    // Signed out: nothing to lock, unlock or sign out.
    for (label, error) in [
        ("lock", auth::lock(&env.ctx).unwrap_err()),
        ("logout", auth::logout(&env.ctx).unwrap_err()),
        ("unlock", auth::unlock(&env.ctx, &right).unwrap_err()),
    ] {
        assert!(
            error.has_code(ErrorCode::ErrNotLoggedIn),
            "{label} while signed out: {error:?}"
        );
    }
    login_owner(&env);
    // Signed in: a second sign-in and an unlock are refused.
    assert!(
        auth::login(&env.ctx, "owner", &right)
            .unwrap_err()
            .has_code(ErrorCode::ErrAlreadySignedIn)
    );
    assert!(
        auth::unlock(&env.ctx, &right)
            .unwrap_err()
            .has_code(ErrorCode::ErrNotLocked)
    );
    // Locked: every check says locked; a second lock changes nothing; a sign-in is refused.
    auth::lock(&env.ctx).unwrap();
    auth::lock(&env.ctx).unwrap();
    assert!(
        auth::require(&env.ctx)
            .unwrap_err()
            .has_code(ErrorCode::ErrSessionLocked)
    );
    assert_eq!(audit_count(&dir, "lock"), 1, "a second lock wrote a row");
    assert!(auth::login(&env.ctx, "owner", &right).is_err());
    // Wrong unlock tries follow the same table as sign-in: 5 wrong then a wait.
    let mut waited = false;
    for _ in 0..6 {
        let error = auth::unlock(&env.ctx, &Secret::new("not-the-password")).unwrap_err();
        waited |= locked_seconds(&error).is_some();
    }
    assert!(waited, "six wrong unlock tries never started a wait");
    assert!(
        auth::unlock(&env.ctx, &right).is_err(),
        "the right password got through a wait"
    );
    assert!(
        auth::require(&env.ctx)
            .unwrap_err()
            .has_code(ErrorCode::ErrSessionLocked)
    );
    env.mono.advance_seconds(120);
    auth::unlock(&env.ctx, &right).unwrap();
    assert!(auth::require(&env.ctx).is_ok());
    // A locked session can still be signed out, and then every check says signed out.
    auth::lock(&env.ctx).unwrap();
    auth::logout(&env.ctx).unwrap();
    assert!(
        auth::require(&env.ctx)
            .unwrap_err()
            .has_code(ErrorCode::ErrNotLoggedIn)
    );
    assert!(
        auth::logout(&env.ctx)
            .unwrap_err()
            .has_code(ErrorCode::ErrNotLoggedIn)
    );
    assert_chain_ok(&dir);
}

// ---------------------------------------------------------------------------------------------
// Change of password from the user menu (PRD FR-5.6, NFR-SEC-1, NFR-SEC-2).
// ---------------------------------------------------------------------------------------------

fn change_input(current: &str, new: &str, repeat: &str) -> auth::ChangePasswordRaw {
    auth::ChangePasswordRaw {
        current: Secret::new(current),
        new: Secret::new(new),
        repeat: Secret::new(repeat),
    }
}

#[test]
fn a_password_change_needs_the_current_password_and_wrong_ones_count_towards_the_lockout() {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let hash_before = password_hash(&dir);
    // The shape is checked first and counts nothing.
    for (label, input, want) in [
        (
            "no current password",
            change_input("", "newsecret1", "newsecret1"),
            ErrorCode::ErrRequired,
        ),
        (
            "5 characters",
            change_input(OWNER_PASSWORD, "abc12", "abc12"),
            ErrorCode::ErrPasswordTooShort,
        ),
        (
            "the repeat differs",
            change_input(OWNER_PASSWORD, "newsecret1", "newsecret2"),
            ErrorCode::ErrPasswordsDiffer,
        ),
    ] {
        let error = auth::change_password(&env.ctx, &owner, &input).unwrap_err();
        assert!(error.has_code(want), "{label}: {error:?}");
    }
    let attempts = |dir: &TempDir| -> i64 {
        raw(dir)
            .query_row("SELECT failed_attempts FROM users", [], |r| r.get(0))
            .unwrap()
    };
    assert_eq!(attempts(&dir), 0, "a refused shape counted as a wrong try");
    assert_eq!(password_hash(&dir), hash_before);
    // A wrong current password is a wrong try, with the same table as sign-in: the 5th locks.
    let mut waited = None;
    for n in 1..=5 {
        let error = auth::change_password(
            &env.ctx,
            &owner,
            &change_input("wrong-one", "newsecret1", "newsecret1"),
        )
        .unwrap_err();
        if n < 5 {
            assert!(
                error.has_code(ErrorCode::ErrWrongPassword),
                "try {n}: {error:?}"
            );
        } else {
            waited = locked_seconds(&error);
        }
    }
    assert_eq!(
        waited,
        Some(60),
        "the 5th wrong try must start the 1 minute wait"
    );
    // During the wait even the right current password is refused and nothing changes.
    let error = auth::change_password(
        &env.ctx,
        &owner,
        &change_input(OWNER_PASSWORD, "newsecret1", "newsecret1"),
    )
    .unwrap_err();
    assert!(locked_seconds(&error).is_some(), "{error:?}");
    assert_eq!(password_hash(&dir), hash_before);
    // The same wrong tries also lock the sign-in of that name (one counter, FR-8.2).
    auth::logout(&env.ctx).unwrap();
    let error = auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap_err();
    assert!(locked_seconds(&error).is_some(), "{error:?}");
    env.mono.advance_seconds(61);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    // Now the change works, and only the new password works after it.
    auth::change_password(
        &env.ctx,
        &owner,
        &change_input(OWNER_PASSWORD, "newsecret1", "newsecret1"),
    )
    .unwrap();
    assert_ne!(password_hash(&dir), hash_before);
    assert_eq!(audit_count(&dir, "change_password"), 1);
    auth::logout(&env.ctx).unwrap();
    assert!(auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).is_err());
    auth::login(&env.ctx, "owner", &Secret::new("newsecret1")).unwrap();
    // The new password is not in any audit row, log line or file in plain text.
    let audit: String = raw(&dir)
        .query_row(
            "SELECT group_concat(COALESCE(old_json, '') || COALESCE(new_json, ''), ' ') FROM audit_log",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!audit.contains("newsecret1") && !audit.contains(OWNER_PASSWORD));
    assert!(!captured().contains("newsecret1"));
    assert_chain_ok(&dir);
}

#[test]
fn two_password_changes_at_the_same_moment_let_one_win() {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let ctx = env.ctx.clone();
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let results: Vec<Result<(), ServiceError>> = (0..2)
        .map(|n| {
            let ctx = ctx.clone();
            let owner = owner.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                let new = format!("changed-by-{n}");
                auth::change_password(&ctx, &owner, &change_input(OWNER_PASSWORD, &new, &new))
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    let won: Vec<usize> = results
        .iter()
        .enumerate()
        .filter_map(|(n, r)| r.is_ok().then_some(n))
        .collect();
    assert_eq!(won.len(), 1, "{results:?}");
    auth::logout(&env.ctx).unwrap();
    auth::login(
        &env.ctx,
        "owner",
        &Secret::new(format!("changed-by-{}", won[0])),
    )
    .unwrap();
    assert_eq!(audit_count(&dir, "change_password"), 1);
    assert_chain_ok(&dir);
}
