//! The question log of the assistant (plan 48, migration 0015, PRD FR-7.84, FR-7.81): append-only,
//! and the daily count is the number of today's counted rows.
#![cfg(test)]

mod common;

use mandi_db::assistant::{self, NewQuestion};
use tempfile::tempdir;

fn question<'a>(source: &'a str, counted: bool, day: &'a str, text: &'a str) -> NewQuestion<'a> {
    NewQuestion {
        asked_at: "2026-10-10T13:12:00.000Z",
        asked_day: day,
        asked_by: None,
        question: text,
        language: "ur",
        conversation_id: "c-1",
        parent_id: None,
        source,
        counted,
        sources_json: "[]",
        outcome: "answered",
        tool_calls_json: "[]",
        answer_json: if source == "chip" { None } else { Some("{}") },
        model: if source == "chip" {
            None
        } else {
            Some("openai/gpt-6-luna")
        },
        model_calls: if source == "chip" { None } else { Some(3) },
        tokens_in: None,
        tokens_cached: None,
        tokens_out: None,
        tokens_reasoning: None,
        finish_reason: None,
    }
}

#[test]
fn the_daily_count_is_the_counted_rows_of_the_day() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let a = assistant::insert(&conn, &question("model", true, "2026-10-10", "ikram")).unwrap();
    let b = assistant::insert(
        &conn,
        &question("chip", false, "2026-10-10", "cash_in_hand"),
    )
    .unwrap();
    assistant::insert(&conn, &question("test", true, "2026-10-10", "test")).unwrap();
    assistant::insert(&conn, &question("model", true, "2026-10-09", "kal")).unwrap();
    // A continuation is its own row, linked to its question, and not counted.
    let mut next = question("continue", false, "2026-10-10", "ikram");
    next.parent_id = Some(a);
    let c = assistant::insert(&conn, &next).unwrap();
    assert_eq!(assistant::count_counted_on(&conn, "2026-10-10").unwrap(), 2);
    assert_eq!(assistant::count_counted_on(&conn, "2026-10-09").unwrap(), 1);
    assert_eq!(assistant::count_counted_on(&conn, "2026-10-11").unwrap(), 0);
    assert_eq!(assistant::total(&conn).unwrap(), 5);
    assert_eq!(assistant::get(&conn, b).unwrap().unwrap().source, "chip");
    assert_eq!(assistant::get(&conn, 999).unwrap(), None);
    let continued: Vec<i64> = assistant::continuations(&conn, a)
        .unwrap()
        .iter()
        .map(|r| r.id)
        .collect();
    assert_eq!(continued, vec![c]);
    // Newest first, in pages.
    let page = assistant::list(&conn, 0, 2).unwrap();
    assert_eq!(page.len(), 2);
    assert!(page[0].id > page[1].id || page[0].asked_at >= page[1].asked_at);
    assert_eq!(assistant::list(&conn, 4, 2).unwrap().len(), 1);
}

#[test]
fn the_log_is_append_only_and_checks_its_lists() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let id = assistant::insert(&conn, &question("model", true, "2026-10-10", "ikram")).unwrap();
    let update = conn.execute(
        "UPDATE assistant_questions SET outcome = 'error' WHERE id = ?1",
        [id],
    );
    assert!(update.unwrap_err().to_string().contains("ERR_APPEND_ONLY"));
    let delete = conn.execute("DELETE FROM assistant_questions WHERE id = ?1", [id]);
    assert!(delete.unwrap_err().to_string().contains("ERR_APPEND_ONLY"));
    let replace = conn.execute(
        "INSERT OR REPLACE INTO assistant_questions (id, asked_at, asked_day, question, language, \
         conversation_id, source, counted, outcome) \
         VALUES (?1, 'x', '2026-10-10', 'q', 'ur', 'c', 'chip', 0, 'answered')",
        [id],
    );
    assert!(replace.unwrap_err().to_string().contains("ERR_APPEND_ONLY"));
    // A chip never counts; an outcome and a language outside the lists are refused.
    assert!(assistant::insert(&conn, &question("chip", true, "2026-10-10", "x")).is_err());
    let mut bad = question("model", true, "2026-10-10", "x");
    bad.outcome = "wrong";
    assert!(assistant::insert(&conn, &bad).is_err());
    let mut bad = question("model", true, "2026-10-10", "x");
    bad.language = "fr";
    assert!(assistant::insert(&conn, &bad).is_err());
    let mut bad = question("model", true, "2026-10-10", "x");
    bad.asked_day = "10-10-2026";
    assert!(assistant::insert(&conn, &bad).is_err());
    assert_eq!(assistant::total(&conn).unwrap(), 1);
}
