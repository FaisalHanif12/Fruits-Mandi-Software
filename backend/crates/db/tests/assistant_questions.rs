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
        total_millis: None,
        model_millis: None,
        tool_millis: None,
        calls_json: None,
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

#[test]
fn the_time_of_a_run_is_kept_with_its_row_and_an_old_row_has_none() {
    // Migration 17 (plan 51 P51-B2): total, model and tool milliseconds and the calls as JSON.
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let calls = r#"[{"n":1,"model_ms":2310,"tokens_in":7020,"tokens_cached":6979,"tokens_out":74,"tokens_reasoning":50,"rewrite":false,"failed":false,"tools":[{"name":"find","ms":3,"ok":true}]}]"#;
    let mut timed = question("model", true, "2026-10-10", "ikram");
    timed.total_millis = Some(2_400);
    timed.model_millis = Some(2_310);
    timed.tool_millis = Some(3);
    timed.calls_json = Some(calls);
    let a = assistant::insert(&conn, &timed).unwrap();
    let b = assistant::insert(
        &conn,
        &question("chip", false, "2026-10-10", "cash_in_hand"),
    )
    .unwrap();
    let row = assistant::get(&conn, a).unwrap().unwrap();
    assert_eq!(
        (row.total_millis, row.model_millis, row.tool_millis),
        (Some(2_400), Some(2_310), Some(3))
    );
    assert_eq!(row.calls_json.as_deref(), Some(calls));
    let chip = assistant::get(&conn, b).unwrap().unwrap();
    assert_eq!(
        (
            chip.total_millis,
            chip.model_millis,
            chip.tool_millis,
            chip.calls_json
        ),
        (None, None, None, None)
    );
    // A negative time and a calls text that is not JSON are refused by the table.
    for (column, value) in [
        ("total_millis", "-1"),
        ("model_millis", "-5"),
        ("tool_millis", "-2"),
        ("calls_json", "'not json'"),
    ] {
        let sql = format!(
            "INSERT INTO assistant_questions (asked_at, asked_day, question, language, \
             conversation_id, source, counted, outcome, {column}) \
             VALUES ('2026-10-10T13:12:00.000Z', '2026-10-10', 'x', 'ur', 'c', 'chip', 0, \
             'answered', {value})"
        );
        assert!(conn.execute(&sql, []).is_err(), "{column} {value}");
    }
}

#[test]
fn the_new_columns_stay_append_only() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let mut timed = question("model", true, "2026-10-10", "ikram");
    timed.total_millis = Some(100);
    let id = assistant::insert(&conn, &timed).unwrap();
    for column in ["total_millis", "model_millis", "tool_millis"] {
        let error = conn
            .execute(
                &format!("UPDATE assistant_questions SET {column} = 7 WHERE id = ?1"),
                [id],
            )
            .unwrap_err();
        assert!(
            error.to_string().contains("ERR_APPEND_ONLY"),
            "{column}: {error}"
        );
    }
    let error = conn
        .execute("DELETE FROM assistant_questions WHERE id = ?1", [id])
        .unwrap_err();
    assert!(error.to_string().contains("ERR_APPEND_ONLY"));
}
