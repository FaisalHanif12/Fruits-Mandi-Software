//! The unsaved work of a screen (PRD FR-12.10, NFR-REL-2, plan 13).

use mandi_services::core::drafts::{
    DraftDiscardInput, DraftGetInput, DraftSaveInput, MAX_DRAFT_BYTES,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::drafts;

use crate::helpers::*;
use crate::support::*;

fn put(w: &World, screen: &str, payload: &str) -> Result<(), mandi_services::ServiceError> {
    drafts::save(
        &w.env.ctx,
        &w.owner,
        &DraftSaveInput {
            screen: screen.into(),
            payload_json: payload.into(),
        },
    )
    .map(|_| ())
}

fn get(w: &World, screen: &str) -> Option<(String, String)> {
    drafts::get(
        &w.env.ctx,
        &w.owner,
        &DraftGetInput {
            screen: screen.into(),
        },
    )
    .unwrap()
    .map(|d| (d.payload_json, d.saved_at))
}

#[test]
fn a_draft_is_kept_replaced_and_thrown_away() {
    let w = world();
    assert!(get(&w, "bikri").is_none());
    put(&w, "bikri", "{\"lots\":[1]}").unwrap();
    assert_eq!(get(&w, "bikri").unwrap().0, "{\"lots\":[1]}");
    // One row for the user and the screen: a second save replaces it.
    put(&w, "bikri", "{\"lots\":[1,2]}").unwrap();
    assert_eq!(get(&w, "bikri").unwrap().0, "{\"lots\":[1,2]}");
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM drafts"), 1);
    // Another screen has its own draft.
    put(&w, "vouchers", "{}").unwrap();
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM drafts"), 2);
    drafts::discard(
        &w.env.ctx,
        &w.owner,
        &DraftDiscardInput {
            screen: "bikri".into(),
        },
    )
    .unwrap();
    assert!(get(&w, "bikri").is_none());
    assert!(get(&w, "vouchers").is_some());
    // Throwing away nothing is fine.
    drafts::discard(
        &w.env.ctx,
        &w.owner,
        &DraftDiscardInput {
            screen: "bikri".into(),
        },
    )
    .unwrap();
}

#[test]
fn a_payload_that_is_not_json_or_is_over_1_mb_is_refused() {
    let w = world();
    for bad in ["", "not json", "{\"a\":", "{'a':1}", "[1,]"] {
        let error = put(&w, "bikri", bad).unwrap_err();
        assert!(
            rejects(&error, ErrorCode::ErrDraftInvalid, "payload_json"),
            "{bad}: {error:?}"
        );
    }
    // Exactly 1 MB of valid JSON is fine; one byte more is refused.
    let filler = |len: usize| format!("\"{}\"", "a".repeat(len - 2));
    put(&w, "bikri", &filler(MAX_DRAFT_BYTES)).unwrap();
    let error = put(&w, "bikri", &filler(MAX_DRAFT_BYTES + 1)).unwrap_err();
    assert!(
        rejects(&error, ErrorCode::ErrDraftInvalid, "payload_json"),
        "{error:?}"
    );
    // The refused one did not replace the kept one.
    assert_eq!(get(&w, "bikri").unwrap().0.len(), MAX_DRAFT_BYTES);
}

#[test]
fn a_screen_name_is_a_plain_name() {
    let w = world();
    for bad in ["", "Bikri", "bikri screen", "../x", "a'b", &"a".repeat(41)] {
        let error = put(&w, bad, "{}").unwrap_err();
        assert!(
            rejects(&error, ErrorCode::ErrFormatInvalid, "screen"),
            "{bad}: {error:?}"
        );
    }
    assert!(
        drafts::get(
            &w.env.ctx,
            &w.owner,
            &DraftGetInput {
                screen: "x y".into()
            }
        )
        .is_err()
    );
    assert!(put(&w, "bikri.v2-draft_1", "{}").is_ok());
}

#[test]
fn a_saved_bikri_removes_its_own_draft_only() {
    let w = world();
    put(&w, "bikri", "{\"typing\":1}").unwrap();
    put(&w, "vouchers", "{\"typing\":2}").unwrap();
    save_ok(&w, &event1(&w));
    assert!(get(&w, "bikri").is_none());
    assert!(get(&w, "vouchers").is_some());
}

#[test]
fn a_refused_save_keeps_the_draft() {
    let w = world();
    put(&w, "bikri", "{\"typing\":1}").unwrap();
    let mut bad = event1(&w);
    bad.lots[0].qty_received = "abc".into();
    assert!(save(&w, &bad).is_err());
    assert!(get(&w, "bikri").is_some());
}

#[test]
fn a_draft_writes_no_audit_row_and_survives_a_new_session_of_the_app() {
    let w = world();
    let audit = scalar(&w, "SELECT COUNT(*) FROM audit_log");
    put(&w, "bikri", "{\"a\":1}").unwrap();
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM audit_log"), audit);
    // NFR-REL-2: it is in the data file, so another connection (a restart) reads it.
    let conn = crate::common::raw(&w.dir);
    let text: String = conn
        .query_row(
            "SELECT payload_json FROM drafts WHERE screen = 'bikri'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(text, "{\"a\":1}");
}
