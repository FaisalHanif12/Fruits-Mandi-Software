//! Plan 73 T1 (PRD FR-21.4, FR-7.71, NFR-SEC-5): only the Owner's sign (دستخط) opens a closed day to
//! fix an entry. "adnan" is the sign until the Owner changes it; case and spaces at the ends do not
//! matter, and عدنان in Urdu letters is accepted too. A wrong sign opens nothing and writes an audit
//! entry without the typed text. The sign is saved beside the reason in the audit entry of the reopen
//! and of every edit saved in that fix; the edit stamp and every edit history version return it. An
//! edit of an open day has no sign.
//!
//! The data is بکری 1 of the PRD worked day (PRD 4.4, اکرم 20 at 600), saved on 10-10-2026, with that
//! day closed and today 11-10-2026. The edit moves اکرم's rate to 610 (E2E-13).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;

use common::{OWNER_PASSWORD, assert_chain_ok, raw, set_today};
use mandi_reports::audit::{EditHistoryInput, EditStampInput, edit_history, edit_stamps};
use mandi_services::core::bikri::BikriSaved;
use mandi_services::core::edit::{DayFixCloseInput, DayFixOpenInput, DayFixOpened, EditEntity};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::secret::Secret;
use mandi_services::{ServiceError, bikri, day, owner_sign};
use support::World;

const REASON: &str = "ریٹ غلط تھا";

/// بکری 1 saved on 10-10-2026, that day closed, today 11-10-2026.
fn closed_day() -> (World, BikriSaved) {
    let w = support::world();
    let saved = support::save_old(&w, &support::event1(&w), false);
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    (w, saved)
}

fn fix_with(w: &World, reason: &str, sign: &str) -> Result<DayFixOpened, ServiceError> {
    day::fix_open(
        &w.env.ctx,
        &w.owner,
        &DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: reason.into(),
            sign: sign.into(),
        },
    )
}

fn fix(w: &World, sign: &str) -> Result<DayFixOpened, ServiceError> {
    fix_with(w, REASON, sign)
}

fn closed_up_to(w: &World) -> Option<String> {
    day::status(&w.env.ctx).unwrap().closed_up_to
}

/// `(code, field)` of each refusal of an error.
fn refusals(error: &ServiceError) -> Vec<(ErrorCode, Option<String>)> {
    match error {
        ServiceError::Rejected(items) => items.iter().map(|r| (r.code, r.field.clone())).collect(),
        other => panic!("not a refusal: {other:?}"),
    }
}

/// How many audit rows have this action.
fn count_of(w: &World, action: &str) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = ?1",
            [action],
            |r| r.get(0),
        )
        .unwrap()
}

/// The new value (JSON text) and the reason of every audit row of an action, oldest first.
fn rows_of(w: &World, action: &str) -> Vec<(Option<String>, Option<String>)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare("SELECT new_json, reason FROM audit_log WHERE action = ?1 ORDER BY id")
        .unwrap();
    statement
        .query_map([action], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// The rate of the first line of بکری 1, changed to 610: the E2E-13 edit.
fn edit_rate(w: &World, saved: &BikriSaved) {
    let opened = tools::open(w, saved.bikri_id, saved.entry_id);
    let mut draft = opened.draft.clone();
    draft
        .lots
        .first_mut()
        .unwrap()
        .sales
        .first_mut()
        .unwrap()
        .seller_rate = "610".into();
    tools::edit(w, &opened, &draft);
}

#[test]
fn the_owners_sign_opens_the_day_in_any_case_with_spaces_or_in_urdu() {
    let (w, _) = closed_day();
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));
    for sign in ["adnan", "ADNAN ", " Adnan", "عدنان"] {
        let opened = fix(&w, sign).unwrap_or_else(|e| panic!("{sign}: {e:?}"));
        assert_eq!(opened.days, ["2026-10-10"], "{sign}");
        assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-09"), "{sign}");
        // Close it again, so the next spelling opens a closed day too.
        let closed = day::fix_close(
            &w.env.ctx,
            &w.owner,
            &DayFixCloseInput {
                token: opened.token,
            },
        )
        .unwrap();
        assert_eq!(closed.closed, ["2026-10-10"], "{sign}");
        assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"), "{sign}");
    }
    assert_chain_ok(&w.dir);
}

#[test]
fn a_wrong_sign_opens_nothing_changes_nothing_and_is_logged_without_the_text() {
    let (w, saved) = closed_day();
    for wrong in ["munshi", "اکرم", "adnaan", "adn an"] {
        let error = fix(&w, wrong).unwrap_err();
        assert_eq!(
            refusals(&error),
            [(ErrorCode::ErrSignWrong, Some("sign".into()))],
            "{wrong}"
        );
    }
    // The day is still closed, nothing was reopened, the entry still asks for a reopen.
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));
    assert_eq!(count_of(&w, "day_reopen"), 0);
    assert_eq!(count_of(&w, "day_fix_open"), 0);
    assert!(
        bikri::get(&w.env.ctx, saved.bikri_id)
            .unwrap()
            .flags
            .needs_reopen
    );
    let reopens: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM day_closes WHERE action = 'reopen'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(reopens, 0);
    // Each wrong try is in the audit log, and no audit row holds what was typed.
    assert_eq!(count_of(&w, "day_fix_sign_wrong"), 4);
    let typed: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM audit_log
             WHERE COALESCE(old_json, '') || COALESCE(new_json, '') || COALESCE(reason, '')
                   LIKE '%munshi%'
                OR COALESCE(new_json, '') LIKE '%adnaan%'
                OR COALESCE(new_json, '') LIKE '%adn an%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(typed, 0);
    // The right sign still opens it afterwards.
    assert!(fix(&w, "adnan").is_ok());
    assert_chain_ok(&w.dir);
}

#[test]
fn a_missing_sign_or_reason_is_refused_at_its_field_and_nothing_is_written() {
    let (w, _) = closed_day();
    for empty in ["", "   "] {
        let error = fix(&w, empty).unwrap_err();
        assert_eq!(
            refusals(&error),
            [(ErrorCode::ErrSignRequired, Some("sign".into()))]
        );
    }
    // Both missing: both are told, the reason first.
    let both = fix_with(&w, " ", "").unwrap_err();
    assert_eq!(
        refusals(&both),
        [
            (ErrorCode::ErrReasonRequired, Some("reason".into())),
            (ErrorCode::ErrSignRequired, Some("sign".into()))
        ]
    );
    // A right sign with no reason is a missing reason, as before.
    let no_reason = fix_with(&w, " ", "adnan").unwrap_err();
    assert_eq!(
        refusals(&no_reason),
        [(ErrorCode::ErrReasonRequired, Some("reason".into()))]
    );
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));
    for action in ["day_reopen", "day_fix_open", "day_fix_sign_wrong"] {
        assert_eq!(count_of(&w, action), 0, "{action}");
    }
}

#[test]
fn an_edit_saved_in_the_fix_shows_the_sign_everywhere_its_stamp_shows() {
    let (w, saved) = closed_day();
    let ctx = &w.env.ctx;
    // An entry of an open day (11-10-2026, today), saved before the fix: it is not in the fix.
    let mut open_draft = support::draft(vec![support::with_sales(
        support::bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
        vec![support::sale("s-1", w.akram, "4", "600")],
    )]);
    open_draft.header.entry_date = "11-10-2026".into();
    let open_day = support::save_old(&w, &open_draft, false);
    // Open with "ADNAN " (the spelling does not matter); the sign that is saved is the Owner's.
    fix(&w, "ADNAN ").unwrap();
    edit_rate(&w, &saved);

    // The stamp of the edit: the detail, the list row, and the stamp command.
    let detail = bikri::get(ctx, saved.bikri_id).unwrap();
    let stamp = detail.flags.edited.clone().unwrap();
    assert_eq!(stamp.sign.as_deref(), Some("adnan"));
    assert_eq!((stamp.count, stamp.reason.as_deref()), (1, Some("edit")));
    let listed = bikri::list(ctx, &Default::default()).unwrap();
    let row = listed.rows.iter().find(|r| r.id == saved.bikri_id).unwrap();
    assert_eq!(
        row.flags.edited.as_ref().and_then(|s| s.sign.as_deref()),
        Some("adnan")
    );
    let stamps = edit_stamps(
        ctx.db(),
        &EditStampInput {
            entity: EditEntity::Bikri,
            ids: vec![saved.bikri_id],
        },
    )
    .unwrap();
    assert_eq!(
        stamps[0].stamp.as_ref().and_then(|s| s.sign.as_deref()),
        Some("adnan")
    );

    // The history: the version has the sign, and the sign is not one of its changed fields.
    let history = edit_history(
        ctx.db(),
        &EditHistoryInput {
            entity: EditEntity::Bikri,
            id: saved.bikri_id,
        },
    )
    .unwrap();
    assert_eq!(history.versions.len(), 1);
    let version = &history.versions[0];
    assert_eq!(version.sign.as_deref(), Some("adnan"));
    assert_eq!(version.reason.as_deref(), Some("edit"));
    assert!(version.changes.iter().all(|c| c.path != "sign"));
    assert!(
        version
            .changes
            .iter()
            .any(|c| c.path == "lots[0].sales[0].seller_rate_paisa"),
        "{:?}",
        version.changes
    );

    // The audit rows: the reopen (with its reason), the fix and the edit hold the sign.
    for action in ["day_reopen", "day_fix_open", "edit"] {
        let rows = rows_of(&w, action);
        let last = rows.last().unwrap_or_else(|| panic!("no {action}"));
        let json: serde_json::Value =
            serde_json::from_str(last.0.as_deref().expect("a new value")).unwrap();
        assert_eq!(json["sign"], "adnan", "{action}");
    }
    assert_eq!(rows_of(&w, "day_reopen")[0].1.as_deref(), Some(REASON));
    let summary: String = raw(&w.dir)
        .query_row(
            "SELECT summary_json FROM day_closes WHERE action = 'reopen'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(summary.contains("\"sign\":\"adnan\""), "{summary}");

    // An edit of a day the fix did not reopen has no sign, even while the fix is open.
    let opened = tools::open(&w, open_day.bikri_id, open_day.entry_id);
    let mut draft = opened.draft.clone();
    draft
        .lots
        .first_mut()
        .unwrap()
        .sales
        .first_mut()
        .unwrap()
        .seller_rate = "610".into();
    tools::edit(&w, &opened, &draft);
    let open_stamp = bikri::get(ctx, open_day.bikri_id)
        .unwrap()
        .flags
        .edited
        .unwrap();
    assert_eq!(open_stamp.sign, None);
    assert_chain_ok(&w.dir);
}

#[test]
fn an_edit_of_an_open_day_has_no_sign_and_the_sign_of_a_closed_fix_does_not_leak() {
    let (w, saved) = closed_day();
    let ctx = &w.env.ctx;
    let opened = fix(&w, "adnan").unwrap();
    edit_rate(&w, &saved);
    day::fix_close(
        ctx,
        &w.owner,
        &DayFixCloseInput {
            token: opened.token,
        },
    )
    .unwrap();
    // After the fix is closed, an entry of today (an open day) is edited with no sign.
    let mut draft = support::draft(vec![support::with_sales(
        support::bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
        vec![support::sale("s-1", w.akram, "4", "600")],
    )]);
    draft.header.entry_date = "11-10-2026".into();
    let today = support::save_old(&w, &draft, false);
    edit_rate(&w, &today);
    let stamp = bikri::get(ctx, today.bikri_id)
        .unwrap()
        .flags
        .edited
        .unwrap();
    assert_eq!(stamp.sign, None);
    let history = edit_history(
        ctx.db(),
        &EditHistoryInput {
            entity: EditEntity::Bikri,
            id: today.bikri_id,
        },
    )
    .unwrap();
    assert_eq!(history.versions[0].sign, None);
    // The entry of the closed day keeps the sign it was edited with.
    let first = bikri::get(ctx, saved.bikri_id)
        .unwrap()
        .flags
        .edited
        .unwrap();
    assert_eq!(first.sign.as_deref(), Some("adnan"));
}

#[test]
fn the_owner_changes_his_sign_with_his_password_and_only_the_new_sign_opens() {
    let (w, _) = closed_day();
    let ctx = &w.env.ctx;
    let set = |password: &str, sign: &str| {
        owner_sign::set_owner_sign(ctx, &w.owner, &Secret::new(password), sign)
    };
    // A wrong password is refused at the password; a missing, long or odd sign at the sign.
    assert_eq!(
        refusals(&set("not-the-password", "sana").unwrap_err()),
        [(ErrorCode::ErrWrongPassword, Some("password".into()))]
    );
    assert_eq!(
        refusals(&set(OWNER_PASSWORD, "  ").unwrap_err()),
        [(ErrorCode::ErrRequired, Some("sign".into()))]
    );
    let long = set(OWNER_PASSWORD, &"x".repeat(41)).unwrap_err();
    assert_eq!(
        refusals(&long),
        [(ErrorCode::ErrTextTooLong, Some("sign".into()))]
    );
    assert_eq!(
        refusals(&set(OWNER_PASSWORD, "sa\nna").unwrap_err()),
        [(ErrorCode::ErrControlChars, Some("sign".into()))]
    );
    // Nothing changed so far: adnan still opens.
    let opened = fix(&w, "adnan").unwrap();
    day::fix_close(
        ctx,
        &w.owner,
        &DayFixCloseInput {
            token: opened.token,
        },
    )
    .unwrap();
    // A sign of exactly 40 characters is fine; so is a short one.
    set(OWNER_PASSWORD, &"y".repeat(40)).unwrap();
    set(OWNER_PASSWORD, "  Sana ").unwrap();
    // The old sign and its Urdu spelling are refused now; the new one opens, in any case.
    for old in ["adnan", "عدنان", &"y".repeat(40)] {
        assert_eq!(
            refusals(&fix(&w, old).unwrap_err()),
            [(ErrorCode::ErrSignWrong, Some("sign".into()))],
            "{old}"
        );
    }
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));
    assert!(fix(&w, " SANA").is_ok());
    // The change is in the audit log without the sign; the reopen holds the sign it was opened with.
    let changed = rows_of(&w, "edit");
    let last = changed.last().unwrap().0.clone().unwrap();
    assert!(
        last.contains("owner_sign") && !last.to_lowercase().contains("sana"),
        "{last}"
    );
    let reopened = rows_of(&w, "day_reopen");
    assert!(
        reopened
            .last()
            .unwrap()
            .0
            .as_deref()
            .is_some_and(|json| json.contains("\"sign\":\"Sana\"")),
        "{reopened:?}"
    );
    assert_chain_ok(&w.dir);
}
