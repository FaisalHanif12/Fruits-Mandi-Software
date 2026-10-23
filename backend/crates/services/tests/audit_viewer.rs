//! The audit viewer (plan 22 early, PRD FR-21.3, NFR-SEC-5, Code Standards B31): the paged list
//! with its filters, the fields that changed, the hidden secrets, the days in Pakistan time, the
//! options for the filter boxes, and that reading changes nothing.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::ReportsError;
use mandi_reports::audit::{
    AuditChange, AuditFilters, AuditListInput, AuditPage, AuditValue, list, options,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::money::{Grouping, Paisa};
use mandi_services::core::rules::{CreditLimitMode, OverridesSettings};
use mandi_services::db::audit::AuditEntry;
use mandi_services::{rules, settings_ext};
use reports_world::*;
use serde_json::json;

fn page(w: &support::World, filters: AuditFilters, offset: i64, limit: i64) -> AuditPage {
    list(
        w.env.ctx.db(),
        &AuditListInput {
            filters,
            offset,
            limit,
        },
    )
    .unwrap()
}

fn refused(w: &support::World, filters: AuditFilters) -> Vec<(ErrorCode, String)> {
    match list(
        w.env.ctx.db(),
        &AuditListInput {
            filters,
            offset: 0,
            limit: 0,
        },
    ) {
        Err(ReportsError::Rejected(items)) => {
            items.into_iter().map(|i| (i.code, i.field)).collect()
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

fn entity(name: &str) -> AuditFilters {
    AuditFilters {
        entity: name.into(),
        ..AuditFilters::default()
    }
}

fn overrides(percent: u32) -> OverridesSettings {
    OverridesSettings {
        allow_line_override: true,
        credit_limit_mode: CreditLimitMode::Warn,
        rate_check_enabled: true,
        rate_check_percent: percent,
    }
}

/// The worked day, then: 10-10 the grouping becomes lakh; 11-10 the rate check becomes 40%; and
/// at 20:30 UTC on 10-10 (01:30 on 11-10 in Pakistan) a row with a secret in it.
fn world() -> support::World {
    let w = worked_day();
    set_today(&w.env, 2026, 10, 10);
    settings_ext::set_number_grouping(&w.env.ctx, &w.owner, Grouping::Lakh).unwrap();
    use chrono::TimeZone;
    w.env.clock.set(
        chrono::Utc
            .with_ymd_and_hms(2026, 10, 10, 20, 30, 0)
            .single()
            .unwrap(),
    );
    w.env
        .ctx
        .db()
        .write(|tx| {
            w.env.ctx.db().auditor().write(
                tx,
                &AuditEntry {
                    user_id: Some(w.owner.user_id),
                    action: "edit".into(),
                    entity: "user".into(),
                    entity_id: Some(w.owner.user_id),
                    old_json: Some(json!({ "password_hash": "$argon2id$old-secret", "name": "A" })),
                    new_json: Some(json!({ "password_hash": "$argon2id$new-secret", "name": "A" })),
                    reason: Some("new password".into()),
                },
            )
        })
        .unwrap();
    set_today(&w.env, 2026, 10, 11);
    rules::set_overrides(&w.env.ctx, &w.owner, &overrides(40)).unwrap();
    w
}

fn total_rows(w: &support::World) -> i64 {
    raw(&w.dir)
        .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn the_list_is_newest_first_with_the_count_of_the_whole_filter() {
    let w = world();
    let all = page(&w, AuditFilters::default(), 0, 0);
    assert_eq!(all.total_count, total_rows(&w));
    assert_eq!(all.limit, 100);
    // Newest first by the time shown, then by entry number. (The test clock is moved back and
    // forth by the world, so the order of the times is not always the order of the numbers.)
    let keys: Vec<(String, i64)> = all.rows.iter().map(|r| (r.at.clone(), r.id)).collect();
    let mut sorted = keys.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(keys, sorted, "newest first");
    // The newest entry is the rate check on 11-10, by the Owner.
    let newest = &all.rows[0];
    assert_eq!(
        (newest.action.as_str(), newest.entity.as_str()),
        ("edit", "settings")
    );
    assert_eq!(newest.date, "2026-10-11");
    assert_eq!(newest.user.as_ref().unwrap().username, "owner");
}

#[test]
fn the_changed_fields_show_old_and_new_and_money_is_money() {
    let w = world();
    let settings = page(&w, entity("settings"), 0, 0);
    let grouping = settings
        .rows
        .iter()
        .find(|r| r.new_json.as_deref().is_some_and(|j| j.contains("lakh")))
        .unwrap();
    assert_eq!(
        grouping.changes,
        [AuditChange {
            path: "number_grouping".into(),
            old: AuditValue::Text {
                text: "international".into()
            },
            new: AuditValue::Text {
                text: "lakh".into()
            },
        }]
    );
    let rate = &settings.rows[0];
    assert_eq!(
        rate.changes,
        [AuditChange {
            path: "overrides.rate_check_percent".into(),
            old: AuditValue::Number { value: 30 },
            new: AuditValue::Number { value: 40 },
        }]
    );
    // An opening or a posting audit holds paisa fields: they are money, not plain numbers.
    let money_change = page(&w, AuditFilters::default(), 0, 0)
        .rows
        .iter()
        .flat_map(|r| r.changes.iter())
        .find(|c| c.path.ends_with("_paisa"))
        .cloned();
    if let Some(change) = money_change {
        assert!(
            matches!(change.new, AuditValue::Money { .. } | AuditValue::Null),
            "{change:?}"
        );
    }
    let _ = Paisa::ZERO;
}

#[test]
fn secrets_never_reach_the_screen_but_a_change_to_one_shows() {
    let w = world();
    let rows = page(&w, entity("user"), 0, 0).rows;
    let row = rows.iter().find(|r| r.action == "edit").unwrap();
    for text in [
        row.old_json.as_deref().unwrap(),
        row.new_json.as_deref().unwrap(),
    ] {
        assert!(
            !text.contains("argon2id") && !text.contains("secret"),
            "{text}"
        );
        assert!(text.contains("[hidden]"));
    }
    assert_eq!(
        row.changes,
        [AuditChange {
            path: "password_hash".into(),
            old: AuditValue::Hidden,
            new: AuditValue::Hidden,
        }]
    );
    assert_eq!(row.reason.as_deref(), Some("new password"));
    // No row of the whole list carries a stored hash.
    let everything = serde_json::to_string(&format!(
        "{:?}",
        page(&w, AuditFilters::default(), 0, 1000).rows
    ))
    .unwrap();
    assert!(!everything.contains("argon2id$"), "a hash got out");
}

#[test]
fn the_filters_narrow_by_user_action_entity_record_and_words() {
    let w = world();
    let by_user = page(
        &w,
        AuditFilters {
            user_id: Some(w.owner.user_id),
            ..AuditFilters::default()
        },
        0,
        0,
    );
    assert!(by_user.total_count > 0);
    assert_eq!(by_user.filters.user.as_ref().unwrap().username, "owner");
    // Entity and action.
    let edits = page(
        &w,
        AuditFilters {
            action: "edit".into(),
            entity: "settings".into(),
            ..AuditFilters::default()
        },
        0,
        0,
    );
    let raw_edits: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'edit' AND entity = 'settings'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(raw_edits >= 2, "the grouping and the rate check");
    assert_eq!(edits.total_count, raw_edits);
    // A record of a kind.
    let user = page(
        &w,
        AuditFilters {
            entity: "user".into(),
            entity_id: Some(w.owner.user_id),
            ..AuditFilters::default()
        },
        0,
        0,
    );
    assert!(
        user.rows
            .iter()
            .all(|r| r.entity_id == Some(w.owner.user_id))
    );
    // Words in the reason or in the old and new values, in any case.
    let words = |text: &str| {
        page(
            &w,
            AuditFilters {
                text: text.into(),
                ..AuditFilters::default()
            },
            0,
            0,
        )
        .total_count
    };
    assert_eq!(words("LAKH"), 1);
    assert_eq!(words("rate_check_percent"), 1);
    assert_eq!(words("new password"), 1);
    assert_eq!(words("no such words anywhere"), 0);
    // A percent sign is only a letter here.
    assert_eq!(words("%"), 0);
}

#[test]
fn the_days_are_days_in_pakistan_time() {
    let w = world();
    let on = |from: &str, to: &str| {
        page(
            &w,
            AuditFilters {
                from: from.into(),
                to: to.into(),
                entity: "settings".into(),
                ..AuditFilters::default()
            },
            0,
            0,
        )
    };
    // Every settings entry with the day it has in Pakistan time; the filter must agree.
    let settings = page(&w, entity("settings"), 0, 1000).rows;
    let on_days = |from: &str, to: &str| -> i64 {
        i64::try_from(
            settings
                .iter()
                .filter(|r| r.date.as_str() >= from && r.date.as_str() <= to)
                .count(),
        )
        .unwrap()
    };
    assert!(
        settings.iter().any(|r| r.date == "2026-10-10"),
        "the grouping, 09:00 UTC"
    );
    assert!(
        settings.iter().any(|r| r.date == "2026-10-11"),
        "the rate check"
    );
    assert_eq!(
        on("10-10-2026", "10-10-2026").total_count,
        on_days("2026-10-10", "2026-10-10")
    );
    assert_eq!(
        on("11-10-2026", "11-10-2026").total_count,
        on_days("2026-10-11", "2026-10-11")
    );
    assert_eq!(
        on("10-10-2026", "11-10-2026").total_count,
        on_days("2026-10-10", "2026-10-11")
    );
    assert_eq!(on("12-10-2026", "").total_count, 0);
    assert_eq!(
        on("", "09-10-2026").total_count,
        on_days("0000-00-00", "2026-10-09")
    );
    // The row of 20:30 UTC on 10-10 is 01:30 on 11-10 in Pakistan.
    let late = page(&w, entity("user"), 0, 0);
    let row = late.rows.iter().find(|r| r.action == "edit").unwrap();
    assert_eq!(
        (row.date.as_str(), row.time.as_str()),
        ("2026-10-11", "01:30:00")
    );
    let on_11 = page(
        &w,
        AuditFilters {
            from: "11-10-2026".into(),
            to: "11-10-2026".into(),
            entity: "user".into(),
            action: "edit".into(),
            ..AuditFilters::default()
        },
        0,
        0,
    );
    assert_eq!(on_11.total_count, 1);
    let on_10 = page(
        &w,
        AuditFilters {
            to: "10-10-2026".into(),
            entity: "user".into(),
            action: "edit".into(),
            ..AuditFilters::default()
        },
        0,
        0,
    );
    assert_eq!(on_10.total_count, 0, "01:30 on 11-10 is not on 10-10");
    // The echo has the days as stored.
    assert_eq!(
        on("10-10-2026", "11-10-2026").filters.from.as_deref(),
        Some("2026-10-10")
    );
}

#[test]
fn pages_join_to_the_whole_list_and_keep_the_count() {
    let w = world();
    let whole = page(&w, AuditFilters::default(), 0, 1000);
    let mut joined = Vec::new();
    let mut offset = 0;
    loop {
        let part = page(&w, AuditFilters::default(), offset, 4);
        assert_eq!(part.total_count, whole.total_count);
        if part.rows.is_empty() {
            break;
        }
        offset += 4;
        joined.extend(part.rows);
    }
    assert_eq!(joined, whole.rows);
}

#[test]
fn bad_filters_are_refused_together_with_their_fields() {
    let w = world();
    assert_eq!(
        refused(
            &w,
            AuditFilters {
                user_id: Some(9_999),
                entity_id: Some(1),
                from: "31-02-2026".into(),
                to: "abc".into(),
                action: "a\u{7}b".into(),
                text: "x".repeat(101),
                ..AuditFilters::default()
            }
        )
        .into_iter()
        .map(|(_, field)| field)
        .collect::<Vec<_>>(),
        [
            "filters.user_id",
            "filters.action",
            "filters.text",
            "filters.entity",
            "filters.from",
            "filters.to"
        ]
    );
    assert_eq!(
        refused(
            &w,
            AuditFilters {
                from: "11-10-2026".into(),
                to: "10-10-2026".into(),
                ..AuditFilters::default()
            }
        ),
        [(ErrorCode::ErrDateRange, "filters.to".to_owned())]
    );
    let bad_page = list(
        w.env.ctx.db(),
        &AuditListInput {
            filters: AuditFilters::default(),
            offset: -1,
            limit: 5_000,
        },
    );
    assert!(matches!(bad_page, Err(ReportsError::Rejected(items)) if items.len() == 2));
}

#[test]
fn the_options_list_the_users_actions_and_kinds_found_in_the_log() {
    let w = world();
    let found = options(w.env.ctx.db()).unwrap();
    assert_eq!(found.users.len(), 1);
    assert_eq!(found.users[0].username, "owner");
    for action in ["create", "edit"] {
        assert!(
            found.actions.iter().any(|a| a == action),
            "{action} in {:?}",
            found.actions
        );
    }
    for entity in ["settings", "user"] {
        assert!(
            found.entities.iter().any(|e| e == entity),
            "{entity} in {:?}",
            found.entities
        );
    }
    let mut sorted = found.actions.clone();
    sorted.sort();
    assert_eq!(found.actions, sorted, "in order");
}

#[test]
fn reading_the_viewer_changes_nothing_and_the_filters_use_the_indexes() {
    let w = world();
    let before = total_rows(&w);
    page(&w, AuditFilters::default(), 0, 0);
    options(w.env.ctx.db()).unwrap();
    assert_eq!(total_rows(&w), before);
    assert_chain_ok(&w.dir);
    // Code Standards B18: a range of days and a user are found through the indexes.
    use mandi_reports::audit::plans::page as plan_page;
    let conn = raw(&w.dir);
    let plan = |query: (String, Vec<mandi_services::db::rusqlite::types::Value>)| -> String {
        let mut statement = conn
            .prepare(&format!("EXPLAIN QUERY PLAN {}", query.0))
            .unwrap();
        statement
            .query_map(
                mandi_services::db::rusqlite::params_from_iter(query.1.iter()),
                |r| r.get::<_, String>(3),
            )
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .join(" | ")
    };
    let dated = plan(plan_page(
        Some("2026-10-09T19:00:00"),
        Some("2026-10-10T19:00:00"),
        None,
    ));
    assert!(dated.contains("idx_audit_log_at"), "{dated}");
    let by_user = plan(plan_page(None, None, Some(1)));
    assert!(by_user.contains("idx_audit_log_"), "{by_user}");
    assert!(!dated.contains("SCAN a"), "{dated}");
}
