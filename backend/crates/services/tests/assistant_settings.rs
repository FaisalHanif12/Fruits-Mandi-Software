//! The database side of ask the app (plan 48 T3, PRD FR-7.81, FR-7.84, FR-7.85, NFR-SEC-15): the
//! two settings and their audit rows, the proof of the Owner's password, the Pakistan day of the
//! daily count, and the question log.
#![cfg(test)]

mod common;

use chrono::{TimeZone, Utc};
use common::*;
use mandi_services::assistant::{self, QuestionSpec};
use mandi_services::core::auth::Language;
use mandi_services::core::error::ErrorCode;
use mandi_services::{Secret, ServiceError, auth};

fn ready() -> (
    tempfile::TempDir,
    mandi_services::testing::TestEnv,
    auth::Actor,
) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn spec(source: &'static str, counted: bool) -> QuestionSpec {
    QuestionSpec {
        question: "ikram ke total bakaya kitne hai".to_owned(),
        language: Language::Ur,
        conversation_id: "c-1".to_owned(),
        parent_id: None,
        source,
        counted,
        sources_json: "[]".to_owned(),
        outcome: "answered",
        tool_calls_json: "[]".to_owned(),
        answer_json: Some("{}".to_owned()),
        model: Some("openai/gpt-6-luna".to_owned()),
        model_calls: Some(3),
        tokens_in: Some(100),
        tokens_cached: Some(60),
        tokens_out: Some(20),
        tokens_reasoning: Some(5),
        finish_reason: Some("stop".to_owned()),
    }
}

fn code_of(error: &ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        ServiceError::Rejected(items) => (items[0].code, items[0].field.clone()),
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn the_assistant_starts_off_and_can_be_switched_on_only_with_a_key() {
    let (dir, env, owner) = ready();
    let start = assistant::config(&env.ctx).unwrap();
    assert!(!start.enabled);
    assert_eq!(start.model, None);
    // On with no key is refused, on the field `enabled`.
    let error =
        assistant::set_config(&env.ctx, &owner, true, "openai/gpt-6-luna", false).unwrap_err();
    assert_eq!(
        code_of(&error),
        (
            ErrorCode::ErrAssistantKeyMissing,
            Some("enabled".to_owned())
        )
    );
    let before = audit_total(&dir);
    // With a key it is switched on, with its model, and one audit row says what changed.
    let on = assistant::set_config(&env.ctx, &owner, true, "openai/gpt-6-luna", true).unwrap();
    assert!(on.enabled);
    assert_eq!(on.model.as_deref(), Some("openai/gpt-6-luna"));
    assert_eq!(audit_total(&dir), before + 1);
    assert_eq!(assistant::config(&env.ctx).unwrap(), on);
    // The same again changes nothing and writes nothing.
    assistant::set_config(&env.ctx, &owner, true, "openai/gpt-6-luna", true).unwrap();
    assert_eq!(audit_total(&dir), before + 1);
    assert_chain_ok(&dir);
}

#[test]
fn the_model_name_is_checked() {
    let (_dir, env, owner) = ready();
    for bad in [
        "",
        "   ",
        "has space",
        "two\nlines",
        "کیلا",
        &"x".repeat(101),
        "a;b",
    ] {
        let error = assistant::set_config(&env.ctx, &owner, false, bad, true).unwrap_err();
        let (code, field) = code_of(&error);
        assert_eq!(field.as_deref(), Some("model"), "{bad:?}");
        assert!(
            matches!(code, ErrorCode::ErrRequired | ErrorCode::ErrFormatInvalid),
            "{bad:?}"
        );
    }
    for good in ["openai/gpt-6-luna", "anthropic/claude-x:beta", "a.b_c-d"] {
        assistant::set_config(&env.ctx, &owner, false, good, false).unwrap();
    }
}

#[test]
fn removing_the_key_switches_the_assistant_off_and_says_so_in_the_audit_log() {
    let (dir, env, owner) = ready();
    assistant::set_config(&env.ctx, &owner, true, "openai/gpt-6-luna", true).unwrap();
    let before = audit_total(&dir);
    assistant::switch_off(&env.ctx, &owner).unwrap();
    assert!(!assistant::config(&env.ctx).unwrap().enabled);
    assert_eq!(audit_total(&dir), before + 1);
    // Already off: nothing more is written.
    assistant::switch_off(&env.ctx, &owner).unwrap();
    assert_eq!(audit_total(&dir), before + 1);
    // The audit row of a key change holds no key and no part of one.
    assistant::audit_key_change(&env.ctx, &owner, "assistant_key_save").unwrap();
    assert_eq!(audit_count(&dir, "assistant_key_save"), 1);
    let everything: String = raw(&dir)
        .prepare("SELECT COALESCE(old_json, '') || COALESCE(new_json, '') || COALESCE(reason, '') || action || entity FROM audit_log")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert!(!everything.contains("sk-"), "{everything}");
    assert_chain_ok(&dir);
}

#[test]
fn the_owners_password_is_proved_before_the_key_changes() {
    let (_dir, env, owner) = ready();
    assistant::prove_password(&env.ctx, &owner, &Secret::new(OWNER_PASSWORD)).unwrap();
    let wrong = assistant::prove_password(&env.ctx, &owner, &Secret::new("not-it")).unwrap_err();
    assert_eq!(
        code_of(&wrong),
        (ErrorCode::ErrWrongPassword, Some("password".to_owned()))
    );
    let empty = assistant::prove_password(&env.ctx, &owner, &Secret::new("")).unwrap_err();
    assert_eq!(code_of(&empty).1.as_deref(), Some("password"));
}

#[test]
fn the_day_of_the_count_starts_at_midnight_pakistan_time() {
    let (_dir, env, _owner) = ready();
    // 09:00 UTC on 10-10-26 is 14:00 in Pakistan: the same day.
    set_today(&env, 2026, 10, 10);
    assert_eq!(assistant::pakistan_day(&env.ctx), "2026-10-10");
    // 19:30 UTC on 10-10-26 is 00:30 on 11-10-26 in Pakistan.
    env.clock.set(
        Utc.with_ymd_and_hms(2026, 10, 10, 19, 30, 0)
            .single()
            .unwrap(),
    );
    assert_eq!(assistant::pakistan_day(&env.ctx), "2026-10-11");
    // 18:59 UTC is 23:59 in Pakistan: still 10-10-26.
    env.clock.set(
        Utc.with_ymd_and_hms(2026, 10, 10, 18, 59, 0)
            .single()
            .unwrap(),
    );
    assert_eq!(assistant::pakistan_day(&env.ctx), "2026-10-10");
}

#[test]
fn the_log_counts_model_questions_and_tests_only_and_never_changes() {
    let (dir, env, owner) = ready();
    set_today(&env, 2026, 10, 10);
    let a = assistant::record_question(&env.ctx, &owner, &spec("model", true)).unwrap();
    let mut chip = spec("chip", false);
    chip.answer_json = None;
    chip.model = None;
    chip.model_calls = None;
    assistant::record_question(&env.ctx, &owner, &chip).unwrap();
    let mut next = spec("continue", false);
    next.parent_id = Some(a);
    assistant::record_question(&env.ctx, &owner, &next).unwrap();
    assistant::record_question(&env.ctx, &owner, &spec("test", true)).unwrap();
    assert_eq!(assistant::count_today(&env.ctx).unwrap(), 2);
    // Tomorrow's count starts from 0; yesterday's rows stay in the log.
    set_today(&env, 2026, 10, 11);
    assert_eq!(assistant::count_today(&env.ctx).unwrap(), 0);
    let (rows, total, limit) = assistant::list_questions(&env.ctx, 0, 0).unwrap();
    assert_eq!((rows.len(), total, limit), (4, 4, 100));
    assert_eq!(rows[0].source, "test", "newest first");
    assert_eq!(
        assistant::get_question(&env.ctx, a)
            .unwrap()
            .unwrap()
            .question,
        "ikram ke total bakaya kitne hai"
    );
    // Paging, and the limits of a page.
    let (page, total, _) = assistant::list_questions(&env.ctx, 3, 2).unwrap();
    assert_eq!((page.len(), total), (1, 4));
    for (offset, limit) in [(-1, 10), (0, -1), (0, 101)] {
        assert!(assistant::list_questions(&env.ctx, offset, limit).is_err());
    }
    // The table refuses a change from any code.
    let conn = raw(&dir);
    assert!(
        conn.execute("UPDATE assistant_questions SET outcome = 'error'", [])
            .is_err()
    );
    assert!(conn.execute("DELETE FROM assistant_questions", []).is_err());
}
