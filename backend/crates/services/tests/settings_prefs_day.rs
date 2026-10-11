//! Settings, preferences and the day status (plan 04 Sections 3, 4 and 7).
//!
//! The Owner is the only user of the app, so there are no user-management tests.
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::auth::{Language, PrefKey};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::settings::{self, Firm};
use mandi_services::testing::TestEnv;
use mandi_services::{Secret, ServiceError, auth, day};
use tempfile::TempDir;

/// A set-up firm with the Owner signed in.
fn owner_ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

/// The (field, code) pairs of a rejection.
fn rejected(error: ServiceError) -> Vec<(String, ErrorCode)> {
    match error {
        ServiceError::Rejected(items) => items
            .into_iter()
            .map(|i| (i.field.unwrap_or_default(), i.code))
            .collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn no_password_or_hash_ever_reaches_the_audit_log() {
    let (dir, env, _owner) = owner_ready();
    wrong_try(&env, "owner"); // a failed sign-in is written down too
    auth::lock(&env.ctx).unwrap();
    auth::unlock(&env.ctx, &Secret::new(OWNER_PASSWORD)).unwrap();
    auth::logout(&env.ctx).unwrap();
    let everything: String = raw(&dir)
        .prepare("SELECT COALESCE(old_json, '') || COALESCE(new_json, '') || COALESCE(reason, '') || action || entity FROM audit_log")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    for secret in [OWNER_PASSWORD, "not-the-password", "$argon2", "recovery"] {
        assert!(
            !everything.contains(secret),
            "{secret} found in the audit log"
        );
    }
}

#[test]
fn settings_have_the_plan_defaults() {
    let (_dir, env, _owner) = owner_ready();
    let all = settings::get(&env.ctx).unwrap();
    assert_eq!(all.idle_lock_minutes, 15);
    assert_eq!(all.default_language, Language::Ur);
    assert_eq!(
        serde_json::to_string(&all.number_grouping).unwrap(),
        "\"international\""
    );
    assert_eq!(all.firm.mandi_en, "New Sabzi Mandi Kot Radha Kishan");
}

#[test]
fn the_firm_is_checked_script_by_script_and_audited() {
    let (dir, env, owner) = owner_ready();
    let mut firm = settings::get(&env.ctx).unwrap().firm;
    firm.name_en = "Maher Adnan & Sons".into();
    firm.phone = "".into();
    let saved = settings::set_firm(&env.ctx, &owner, &firm).unwrap();
    assert_eq!(
        (saved.name_en.as_str(), saved.phone.as_str()),
        ("Maher Adnan & Sons", "")
    );
    assert_eq!(settings::get(&env.ctx).unwrap().firm, saved);
    let (old, new): (String, String) = raw(&dir)
        .query_row("SELECT old_json, new_json FROM audit_log WHERE action = 'edit' AND entity = 'settings' ORDER BY id DESC LIMIT 1", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert!(old.contains("Maher Adnan and Brothers") && new.contains("Maher Adnan & Sons"));

    let bad = Firm {
        name_ur: "English".into(),
        name_en: "اردو".into(),
        mandi_ur: String::new(),
        mandi_en: "ok".into(),
        address: String::new(),
        phone: "0123".into(),
    };
    let got = rejected(settings::set_firm(&env.ctx, &owner, &bad).unwrap_err());
    for want in [
        ("name_ur", ErrorCode::ErrNameUrScript),
        ("name_en", ErrorCode::ErrNameEnScript),
        ("mandi_ur", ErrorCode::ErrRequired),
        ("phone", ErrorCode::ErrPhoneInvalid),
    ] {
        assert!(
            got.contains(&(want.0.to_owned(), want.1)),
            "{want:?} not in {got:?}"
        );
    }
    assert_eq!(
        settings::get(&env.ctx).unwrap().firm,
        saved,
        "nothing saved on error"
    );
}

#[test]
fn idle_lock_minutes_follow_the_range() {
    let (_dir, env, owner) = owner_ready();
    settings::set_idle_lock(&env.ctx, &owner, "30").unwrap();
    assert_eq!(settings::get(&env.ctx).unwrap().idle_lock_minutes, 30);
    for (text, code) in [
        ("0", ErrorCode::ErrOutOfRange),
        ("121", ErrorCode::ErrOutOfRange),
        ("abc", ErrorCode::ErrNotANumber),
        ("", ErrorCode::ErrRequired),
        ("-1", ErrorCode::ErrNotANumber),
    ] {
        let error = settings::set_idle_lock(&env.ctx, &owner, text).unwrap_err();
        assert_eq!(
            rejected(error),
            [("idle_lock_minutes".to_owned(), code)],
            "{text:?}"
        );
    }
    let range = settings::set_idle_lock(&env.ctx, &owner, "500").unwrap_err();
    let ServiceError::Rejected(items) = range else {
        panic!()
    };
    assert_eq!(items[0].values.get("max"), Some(&ErrorValue::Number(120)));
    assert_eq!(settings::get(&env.ctx).unwrap().idle_lock_minutes, 30);
    settings::set_idle_lock(&env.ctx, &owner, "1").unwrap();
    settings::set_idle_lock(&env.ctx, &owner, "120").unwrap();
}

#[test]
fn the_default_language_is_saved_and_used_until_the_owner_chooses_his_own() {
    let (_dir, env, owner) = owner_ready();
    settings::set_default_language(&env.ctx, &owner, Language::En).unwrap();
    assert_eq!(
        settings::get(&env.ctx).unwrap().default_language,
        Language::En
    );
    auth::logout(&env.ctx).unwrap();
    // The Owner never chose a language, so he starts in the default language.
    let info = login_owner(&env);
    assert_eq!((info.id, info.language), (owner.user_id, Language::En));
    assert_eq!(auth::status(&env.ctx).unwrap().login_language, Language::En);
}

#[test]
fn preferences_belong_to_the_user_and_set_the_sign_in_language() {
    let (dir, env, owner) = owner_ready();
    settings::set_pref(&env.ctx, &owner, PrefKey::Language, "en").unwrap();
    settings::set_pref(&env.ctx, &owner, PrefKey::TextSize, "120").unwrap();
    settings::set_pref(&env.ctx, &owner, PrefKey::LastScreen, "/bikri").unwrap();
    settings::set_pref(&env.ctx, &owner, PrefKey::TextSize, "110").unwrap(); // saving again replaces
    let rows: i64 = raw(&dir)
        .query_row(
            "SELECT COUNT(*) FROM user_prefs WHERE user_id = ?1",
            [owner.user_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(rows, 3);
    // The sign-in screen of this computer follows his language.
    auth::logout(&env.ctx).unwrap();
    assert_eq!(auth::status(&env.ctx).unwrap().login_language, Language::En);
    let info = login_owner(&env);
    assert_eq!((info.language, info.text_size), (Language::En, 110));
}

#[test]
fn preference_values_are_checked() {
    let (_dir, env, owner) = owner_ready();
    for (key, value, code) in [
        (PrefKey::Language, "fr", ErrorCode::ErrOutOfRange),
        (PrefKey::Language, "", ErrorCode::ErrOutOfRange),
        (PrefKey::TextSize, "79", ErrorCode::ErrOutOfRange),
        (PrefKey::TextSize, "155", ErrorCode::ErrOutOfRange),
        (PrefKey::TextSize, "125", ErrorCode::ErrOutOfRange),
        (PrefKey::TextSize, "big", ErrorCode::ErrNotANumber),
        (PrefKey::LastScreen, "bikri", ErrorCode::ErrOutOfRange),
        (PrefKey::LastScreen, "/a\nb", ErrorCode::ErrOutOfRange),
    ] {
        let error = settings::set_pref(&env.ctx, &owner, key, value).unwrap_err();
        assert_eq!(
            rejected(error),
            [("value".to_owned(), code)],
            "{key:?} {value:?}"
        );
    }
    for size in ["80", "90", "100", "110", "120", "130", "140", "150"] {
        settings::set_pref(&env.ctx, &owner, PrefKey::TextSize, size).unwrap();
    }
    // The last screen is only a route, never saved with a control character.
    settings::set_pref(&env.ctx, &owner, PrefKey::LastScreen, "/").unwrap();
}

#[test]
fn day_status_uses_the_local_date_and_the_closed_up_to_date() {
    let (dir, env, _owner) = owner_ready();
    // 09:00 UTC is 14:00 in Pakistan: still 10 October.
    let status = day::status(&env.ctx).unwrap();
    assert_eq!(
        (
            status.today.as_str(),
            status.closed_up_to.clone(),
            status.today_open
        ),
        ("2026-10-10", None, true)
    );
    // 21:00 UTC is 02:00 the next morning in Pakistan.
    env.clock.advance_seconds(12 * 3600);
    assert_eq!(day::status(&env.ctx).unwrap().today, "2026-10-11");
    // Days up to and including 09-10-2026 are closed (the PRD worked day): today is open.
    raw(&dir)
        .execute("UPDATE day_state SET closed_up_to_date = '2026-10-09'", [])
        .unwrap();
    let status = day::status(&env.ctx).unwrap();
    assert_eq!(
        (status.closed_up_to.as_deref(), status.today_open),
        (Some("2026-10-09"), true)
    );
    // Today closed: nothing can be posted.
    raw(&dir)
        .execute("UPDATE day_state SET closed_up_to_date = '2026-10-11'", [])
        .unwrap();
    assert!(!day::status(&env.ctx).unwrap().today_open);
}

#[test]
fn the_whole_audit_chain_holds_after_all_of_it() {
    let (dir, env, owner) = owner_ready();
    settings::set_idle_lock(&env.ctx, &owner, "20").unwrap();
    settings::set_default_language(&env.ctx, &owner, Language::En).unwrap();
    auth::lock(&env.ctx).unwrap();
    auth::unlock(&env.ctx, &Secret::new(OWNER_PASSWORD)).unwrap();
    auth::logout(&env.ctx).unwrap();
    wrong_try(&env, "owner");
    assert_chain_ok(&dir);
}
