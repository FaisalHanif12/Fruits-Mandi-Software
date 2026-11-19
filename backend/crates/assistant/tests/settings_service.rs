//! The commands that need no model (plan 48 T3, PRD FR-7.81, FR-7.84, FR-7.85, NFR-SEC-15): status,
//! the setting, the key and the log. The key never leaves the key store: it is in no setting, no
//! audit row, no log row and no reply, only its last 4 characters in the status.
#![cfg(test)]

mod support;

use mandi_assistant::ask::{AskOutcome, AssistantNewConversationInput};
use mandi_assistant::limits::DAILY_LIMIT;
use mandi_assistant::log::{AssistantLogListInput, LogSource};
use mandi_assistant::service;
use mandi_assistant::settings::{
    AssistantKeyRemoveInput, AssistantKeySaveInput, AssistantSettingsSetInput, DEFAULT_MODEL,
};
use mandi_services::ServiceError;
use mandi_services::assistant::{self as store, QuestionSpec};
use mandi_services::core::auth::Language;
use mandi_services::core::error::ErrorCode;
use support::*;

/// A made-up key. It is not a real key; the tests look for it everywhere.
const KEY: &str = "test-key-not-real-wxyz1234";

fn save(password: &str) -> AssistantKeySaveInput {
    AssistantKeySaveInput {
        key: KEY.to_owned(),
        password: password.to_owned(),
    }
}

fn code(error: &ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        ServiceError::Rejected(items) => (items[0].code, items[0].field.clone()),
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn the_status_starts_off_with_no_key_and_the_default_model() {
    let (_dir, env, _owner, rt) = ready();
    let s = service::status(&env.ctx, &rt).unwrap();
    assert!(!s.enabled && !s.key_saved);
    assert_eq!(s.key_last4, None);
    assert_eq!(
        (s.model.as_str(), s.default_model.as_str()),
        (DEFAULT_MODEL, DEFAULT_MODEL)
    );
    assert_eq!((s.count_today, s.limit), (0, DAILY_LIMIT));
}

#[test]
fn saving_the_key_needs_the_password_and_shows_only_its_last_four_characters() {
    let (dir, env, owner, rt) = ready();
    // Wrong password: refused, nothing saved.
    let error = service::key_save(&env.ctx, &rt, &owner, &save("not-it")).unwrap_err();
    assert_eq!(
        code(&error),
        (ErrorCode::ErrWrongPassword, Some("password".to_owned()))
    );
    assert!(rt.keys().get().unwrap().is_none());
    // A bad key: empty and with spaces.
    for (key, want) in [
        ("  ", ErrorCode::ErrRequired),
        ("has space", ErrorCode::ErrFormatInvalid),
    ] {
        let input = AssistantKeySaveInput {
            key: key.to_owned(),
            password: OWNER_PASSWORD.to_owned(),
        };
        let error = service::key_save(&env.ctx, &rt, &owner, &input).unwrap_err();
        assert_eq!(code(&error), (want, Some("key".to_owned())));
    }
    // Right password: saved, and the reply holds only the last 4 characters.
    let s = service::key_save(&env.ctx, &rt, &owner, &save(OWNER_PASSWORD)).unwrap();
    assert!(s.key_saved);
    assert_eq!(s.key_last4.as_deref(), Some("1234"));
    assert!(!format!("{s:?}").contains(KEY));
    assert_eq!(rt.keys().get().unwrap().as_deref(), Some(KEY));
    assert_eq!(audit_count(&dir, "assistant_key_save"), 1);
    // Saving again is a replace, in its own audit row.
    service::key_save(&env.ctx, &rt, &owner, &save(OWNER_PASSWORD)).unwrap();
    assert_eq!(audit_count(&dir, "assistant_key_replace"), 1);
    assert_chain_ok(&dir);
}

#[test]
fn the_key_is_in_no_table_of_the_data_file() {
    let (dir, env, owner, rt) = ready();
    service::key_save(&env.ctx, &rt, &owner, &save(OWNER_PASSWORD)).unwrap();
    service::settings_set(
        &env.ctx,
        &rt,
        &owner,
        &AssistantSettingsSetInput {
            enabled: true,
            model: DEFAULT_MODEL.to_owned(),
        },
    )
    .unwrap();
    store::record_question(&env.ctx, &owner, &spec()).unwrap();
    // Every text column of every table, looked at for the key and for a piece of it.
    let conn = raw(&dir);
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    for table in tables {
        let columns: Vec<String> = conn
            .prepare(&format!(
                "SELECT name FROM pragma_table_info('{table}') WHERE type = 'TEXT'"
            ))
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for column in columns {
            for needle in [KEY, "not-real", "wxyz"] {
                let hits: i64 = conn
                    .query_row(
                        &format!("SELECT COUNT(*) FROM {table} WHERE instr({column}, ?1) > 0"),
                        [needle],
                        |r| r.get(0),
                    )
                    .unwrap();
                assert_eq!(hits, 0, "{needle} found in {table}.{column}");
            }
        }
    }
}

#[test]
fn it_can_be_switched_on_only_with_a_key_and_removing_the_key_switches_it_off() {
    let (dir, env, owner, rt) = ready();
    let on = AssistantSettingsSetInput {
        enabled: true,
        model: DEFAULT_MODEL.to_owned(),
    };
    let error = service::settings_set(&env.ctx, &rt, &owner, &on).unwrap_err();
    assert_eq!(
        code(&error),
        (
            ErrorCode::ErrAssistantKeyMissing,
            Some("enabled".to_owned())
        )
    );
    service::key_save(&env.ctx, &rt, &owner, &save(OWNER_PASSWORD)).unwrap();
    let s = service::settings_set(&env.ctx, &rt, &owner, &on).unwrap();
    assert!(s.enabled && s.key_saved);
    // A changed model is kept and audited.
    let other = AssistantSettingsSetInput {
        enabled: true,
        model: "openai/other-model".to_owned(),
    };
    assert_eq!(
        service::settings_set(&env.ctx, &rt, &owner, &other)
            .unwrap()
            .model,
        "openai/other-model"
    );
    // Removing needs the password too, and switches it off.
    let wrong = AssistantKeyRemoveInput {
        password: "nope".to_owned(),
    };
    assert!(service::key_remove(&env.ctx, &rt, &owner, &wrong).is_err());
    assert!(rt.keys().get().unwrap().is_some());
    let gone = service::key_remove(
        &env.ctx,
        &rt,
        &owner,
        &AssistantKeyRemoveInput {
            password: OWNER_PASSWORD.to_owned(),
        },
    )
    .unwrap();
    assert!(!gone.key_saved && !gone.enabled && gone.key_last4.is_none());
    assert!(rt.keys().get().unwrap().is_none());
    assert_eq!(audit_count(&dir, "assistant_key_remove"), 1);
    // Removing again is fine and writes no second audit row.
    service::key_remove(
        &env.ctx,
        &rt,
        &owner,
        &AssistantKeyRemoveInput {
            password: OWNER_PASSWORD.to_owned(),
        },
    )
    .unwrap();
    assert_eq!(audit_count(&dir, "assistant_key_remove"), 1);
    assert_chain_ok(&dir);
}

fn spec() -> QuestionSpec {
    QuestionSpec {
        question: "ikram ke total bakaya kitne hai".to_owned(),
        language: Language::Ur,
        conversation_id: "c-1".to_owned(),
        parent_id: None,
        source: "model",
        counted: true,
        sources_json: "[]".to_owned(),
        outcome: "answered",
        tool_calls_json: r#"[{"tool":"find","input":{"text":"ikram","role":"buyer"}},{"tool":"report","input":{"id":"R13"}},{"tool":"analytics","input":{"view":"A"}}]"#.to_owned(),
        answer_json: Some("{}".to_owned()),
        model: Some(DEFAULT_MODEL.to_owned()),
        model_calls: Some(3),
        tokens_in: Some(24_000),
        tokens_cached: Some(15_000),
        tokens_out: Some(1_200),
        tokens_reasoning: Some(400),
        finish_reason: Some("stop".to_owned()),
        total_millis: None,
        model_millis: None,
        tool_millis: None,
        calls_json: None,
    }
}

#[test]
fn the_log_page_names_the_tools_and_the_sources_each_row_read() {
    let (_dir, env, owner, _rt) = ready();
    set_today(&env, 2026, 10, 10);
    store::record_question(&env.ctx, &owner, &spec()).unwrap();
    let page = service::log_list(
        &env.ctx,
        &AssistantLogListInput {
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!((page.rows.len(), page.total_count, page.limit), (1, 1, 100));
    let row = &page.rows[0];
    assert_eq!(
        (row.source, row.outcome, row.counted),
        (LogSource::Model, AskOutcome::Answered, true)
    );
    let tools: Vec<(&str, Option<&str>)> = row
        .tools
        .iter()
        .map(|t| (t.tool.as_str(), t.source.as_deref()))
        .collect();
    assert_eq!(
        tools,
        vec![
            ("find", None),
            ("report", Some("R13")),
            ("analytics", Some("A"))
        ]
    );
    assert!(row.can_rerun);
    assert_eq!(
        (
            row.tokens_in,
            row.tokens_cached,
            row.tokens_out,
            row.tokens_reasoning
        ),
        (Some(24_000), Some(15_000), Some(1_200), Some(400))
    );
    assert_eq!(row.asked_day, "2026-10-10");
    // The page limits are refused with a code on their field.
    let bad = service::log_list(
        &env.ctx,
        &AssistantLogListInput {
            offset: 0,
            limit: 101,
        },
    )
    .unwrap_err();
    assert_eq!(
        code(&bad),
        (ErrorCode::ErrOutOfRange, Some("limit".to_owned()))
    );
}

#[test]
fn a_new_conversation_has_its_own_id_each_time() {
    let (_dir, env, _owner, rt) = ready();
    let none = AssistantNewConversationInput {
        conversation_id: None,
    };
    let a = service::new_conversation(&env.ctx, &rt, &none).conversation_id;
    let b = service::new_conversation(&env.ctx, &rt, &none).conversation_id;
    assert_ne!(a, b);
    assert!(a.starts_with("c-"));
}
