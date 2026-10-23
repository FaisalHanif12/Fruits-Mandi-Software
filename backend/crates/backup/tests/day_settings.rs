//! The Day group of the settings (plan 22 early, PRD Section 21, FR-20.3, FR-21.1): the closing
//! time shared with the backup settings, and the auto day close switch that is only stored.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod harness;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod support;

use harness::*;
use mandi_backup::day_settings::{DaySettings, load, save};
use mandi_backup::error::BackupCode;
use mandi_backup::settings;

fn count_audit(rig: &Rig, entity: &str) -> i64 {
    raw(rig)
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE entity = ?1",
            [entity],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn the_day_group_starts_at_nine_pm_with_auto_close_off() {
    let rig = small_world();
    let day = rig.w.env.ctx.db().read(load).unwrap();
    assert_eq!(
        day,
        DaySettings {
            closing_time: "21:00".into(),
            auto_day_close: false
        }
    );
}

#[test]
fn the_closing_time_is_the_backup_closing_time_and_a_change_is_audited_once() {
    let rig = small_world();
    let db = rig.w.env.ctx.db();
    let user = Some(rig.w.owner.user_id);
    let saved = save(
        db,
        user,
        &DaySettings {
            closing_time: "22:30".into(),
            auto_day_close: true,
        },
    )
    .unwrap();
    assert_eq!(saved.closing_time, "22:30");
    // One source: the backup settings hold the same time, and nothing else of them changed.
    let backup = db.read(settings::load).unwrap();
    assert_eq!(backup.closing_time, "22:30");
    assert_eq!(backup.cloud_prefix, "mandi");
    assert_eq!(db.read(load).unwrap().closing_time, "22:30");
    assert!(db.read(load).unwrap().auto_day_close);
    // The time went to the backup row's audit entry, the switch to its own.
    assert_eq!(count_audit(&rig, "backup_settings"), 1);
    assert_eq!(count_audit(&rig, "day_settings"), 1);
    let (old, new): (String, String) = raw(&rig)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE entity = 'day_settings'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(old, r#"{"auto_day_close":false}"#);
    assert_eq!(new, r#"{"auto_day_close":true}"#);
    // The same values again write nothing.
    save(
        db,
        user,
        &DaySettings {
            closing_time: "22:30".into(),
            auto_day_close: true,
        },
    )
    .unwrap();
    assert_eq!(count_audit(&rig, "backup_settings"), 1);
    assert_eq!(count_audit(&rig, "day_settings"), 1);
    // Only the switch: one more day_settings row, no backup row.
    save(
        db,
        user,
        &DaySettings {
            closing_time: "22:30".into(),
            auto_day_close: false,
        },
    )
    .unwrap();
    assert_eq!(count_audit(&rig, "backup_settings"), 1);
    assert_eq!(count_audit(&rig, "day_settings"), 2);
    // The audit chain is whole.
    mandi_services::db::audit::verify_audit_chain(&raw(&rig), &common::audit_key()).unwrap();
}

#[test]
fn a_bad_closing_time_is_refused_with_its_bounds_and_nothing_is_written() {
    let rig = small_world();
    let db = rig.w.env.ctx.db();
    for bad in ["25:00", "9:00", "", "12:60", "ab:cd"] {
        let error = save(
            db,
            None,
            &DaySettings {
                closing_time: bad.into(),
                auto_day_close: true,
            },
        )
        .unwrap_err();
        let items = error.all();
        assert_eq!(items.len(), 1, "{bad}");
        assert_eq!(items[0].code, BackupCode::ErrOutOfRange);
        assert_eq!(items[0].field.as_deref(), Some("closing_time"));
        assert!(items[0].values.contains_key("min") && items[0].values.contains_key("max"));
    }
    let day = db.read(load).unwrap();
    assert_eq!(day.closing_time, "21:00");
    assert!(!day.auto_day_close, "a refused save changes nothing");
    assert_eq!(count_audit(&rig, "day_settings"), 0);
}
