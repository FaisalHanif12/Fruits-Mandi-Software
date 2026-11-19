//! Plan 80 R6-3 (PRD FR-7.76 point 2, FR-7.44): a question of the assistant that ended because the
//! two places of a figure differed is one item of the attention list for 7 days.
//!
//! Example: a question that ended `records_differ` on 12-11-26 is on the list on 19-11-26 (7 days)
//! and is gone on 20-11-26.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_reports::dashboard::{self, AttentionKind, AttentionList, BackupFacts};
use support::World;

fn bell(w: &World) -> AttentionList {
    dashboard::attention(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
}

fn differ_item(list: &AttentionList) -> Option<&mandi_reports::dashboard::AttentionItem> {
    list.items
        .iter()
        .find(|i| i.kind == AttentionKind::RecordsDiffer)
}

/// Writes one row of the question log with the outcome and the day.
fn log(w: &World, outcome: &'static str, day: &str) {
    let q = mandi_services::db::assistant::NewQuestion {
        asked_at: "2026-11-10T09:00:00.000Z",
        asked_day: day,
        asked_by: None,
        question: "akram ka bakaya",
        language: "ur",
        conversation_id: "c1",
        parent_id: None,
        source: "model",
        counted: true,
        sources_json: "[]",
        outcome,
        tool_calls_json: "[]",
        answer_json: None,
        model: Some("openai/gpt-6-luna"),
        model_calls: Some(1),
        tokens_in: Some(100),
        tokens_cached: Some(0),
        tokens_out: Some(10),
        tokens_reasoning: Some(0),
        finish_reason: Some("stop"),
        total_millis: None,
        model_millis: None,
        tool_millis: None,
        calls_json: None,
    };
    w.env
        .ctx
        .db()
        .write(|tx| {
            mandi_services::db::assistant::insert(tx, &q)
                .map(|_| ())
                .map_err(mandi_services::db::DbError::from)
        })
        .unwrap();
}

#[test]
fn a_records_difference_in_the_last_seven_days_is_one_attention_item() {
    let w = reports_world::worked_day();
    set_today(&w.env, 2026, 11, 19);
    // Nothing logged: no item.
    assert!(differ_item(&bell(&w)).is_none());
    // Two questions with a difference (one 7 days ago, one today), and others that are not one.
    log(&w, "records_differ", "2026-11-12");
    log(&w, "records_differ", "2026-11-19");
    log(&w, "answered", "2026-11-19");
    log(&w, "checks_failed", "2026-11-19");
    let list = bell(&w);
    let item = differ_item(&list).expect("an item for the questions whose two places differed");
    // One item for all of them: its count is the number of questions; its age is the newest's.
    assert_eq!(item.count, 2);
    assert_eq!(item.days, Some(0));
    assert!(item.first.is_none() && item.amount_paisa.is_none());
    // The bell number counts it.
    assert_eq!(
        list.total_count,
        list.items.iter().map(|i| i.count).sum::<i64>()
    );
}

#[test]
fn a_records_difference_leaves_the_list_after_seven_days() {
    let w = reports_world::worked_day();
    log(&w, "records_differ", "2026-11-12");
    // 7 days ago: still listed, with its age.
    set_today(&w.env, 2026, 11, 19);
    let list = bell(&w);
    let item = differ_item(&list).expect("listed on the seventh day");
    assert_eq!((item.count, item.days), (1, Some(7)));
    // 8 days ago: gone.
    set_today(&w.env, 2026, 11, 20);
    assert!(differ_item(&bell(&w)).is_none());
}

#[test]
fn other_outcomes_never_make_the_item() {
    let w = reports_world::worked_day();
    set_today(&w.env, 2026, 11, 19);
    for outcome in [
        "answered",
        "estimate",
        "advice",
        "which_one",
        "not_understood",
        "checks_failed",
        "locked",
        "limit_reached",
        "offline",
        "error",
    ] {
        log(&w, outcome, "2026-11-19");
    }
    assert!(differ_item(&bell(&w)).is_none());
}
