//! Plan 82: the conversation id a command is given is checked at the input (Code Standards 4.1
//! layer 2, Q1, V0): a NUL, a control character or a too long id is refused with a field error before
//! anything is read, paid for or written. An empty or blank id starts a new conversation.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/harness.rs"]
mod harness;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use harness::Harness;
use mandi_assistant::ask::{AssistantAskInput, AssistantQuickInput, QuickChip};
use mandi_assistant::client::RecordedClient;
use mandi_assistant::conversation::checked_conversation_id;
use mandi_assistant::quick::quick;
use mandi_assistant::run::ask;
use mandi_services::ServiceError;
use mandi_services::core::error::ErrorCode;

fn bad_ids() -> Vec<String> {
    vec![
        "\u{0}".to_owned(),
        "a\u{0}b".to_owned(),
        "a\nb".to_owned(),
        "\u{7f}".to_owned(),
        "a".repeat(65),
        "ا".repeat(10_000),
    ]
}

fn is_field_error(result: &Result<impl std::fmt::Debug, ServiceError>) -> bool {
    match result {
        Err(ServiceError::Rejected(items)) => items.first().is_some_and(|i| {
            i.code == ErrorCode::ErrFormatInvalid && i.field.as_deref() == Some("conversation_id")
        }),
        _ => false,
    }
}

#[test]
fn the_helper_keeps_a_good_id_and_drops_a_blank_one() {
    assert_eq!(checked_conversation_id(Some("c-1")).unwrap(), Some("c-1"));
    assert_eq!(
        checked_conversation_id(Some("  c-1 ")).unwrap(),
        Some("c-1")
    );
    assert_eq!(checked_conversation_id(Some("   ")).unwrap(), None);
    assert_eq!(checked_conversation_id(Some("")).unwrap(), None);
    assert_eq!(checked_conversation_id(None).unwrap(), None);
    let longest = "a".repeat(64);
    assert_eq!(
        checked_conversation_id(Some(&longest)).unwrap(),
        Some(longest.as_str())
    );
    for id in bad_ids() {
        assert!(
            is_field_error(&checked_conversation_id(Some(&id))),
            "{:.12?}",
            id
        );
    }
}

#[test]
fn ask_and_quick_refuse_a_bad_id_before_anything_is_read_or_written() {
    let h = Harness::day();
    let before: i64 = common::raw(&h.world.dir)
        .query_row("SELECT COUNT(*) FROM assistant_questions", [], |r| r.get(0))
        .unwrap();
    for id in bad_ids() {
        let client = RecordedClient::new(Vec::new());
        let emit = |_s| {};
        let deps = h.deps(&client, &emit);
        let asked = ask(
            &deps,
            &AssistantAskInput {
                text: "aaj ka commission".into(),
                conversation_id: Some(id.clone()),
            },
        );
        assert!(is_field_error(&asked), "ask {:.12?}: {asked:?}", id);
        let chip = quick(
            &deps,
            &AssistantQuickInput {
                chip: QuickChip::CashInHand,
                party_id: None,
                conversation_id: Some(id.clone()),
            },
        );
        assert!(is_field_error(&chip), "quick {:.12?}: {chip:?}", id);
        assert!(client.requests().is_empty(), "no model call");
    }
    let after: i64 = common::raw(&h.world.dir)
        .query_row("SELECT COUNT(*) FROM assistant_questions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(after, before, "a refused input wrote a log row");
}

#[test]
fn a_blank_id_starts_a_new_conversation_for_a_chip() {
    let h = Harness::day();
    let client = RecordedClient::new(Vec::new());
    let emit = |_s| {};
    let deps = h.deps(&client, &emit);
    let reply = quick(
        &deps,
        &AssistantQuickInput {
            chip: QuickChip::CashInHand,
            party_id: None,
            conversation_id: Some("   ".into()),
        },
    )
    .expect("a chip with a blank id");
    assert!(!reply.conversation_id.trim().is_empty());
}
