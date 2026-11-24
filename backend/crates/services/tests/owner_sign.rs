//! Plan 73 T1 (PRD FR-21.4, FR-7.71, NFR-SEC-5): only the Owner's sign (دستخط) opens a closed day to
//! fix an entry. "adnan" is the sign until the Owner changes it; case and spaces at the ends do not
//! matter, and عدنان in Urdu letters is accepted too. A wrong sign opens nothing and writes an audit
//! entry without the typed text. The sign is a secret (PRD FR-7.71 v1.58, plan 96): the audit entry of
//! the reopen and of every edit saved in that fix records only `sign_given: true`, and no stamp, history
//! version, audit row or list ever carries the text. An edit of an open day has no `sign_given`.
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
use mandi_reports::audit::{
    AuditFilters, AuditListInput, AuditPage, AuditValue, EditHistoryInput, EditStampInput,
    edit_history, edit_stamps, list as audit_list,
};
use mandi_services::core::bikri::BikriFindInput;
use mandi_services::core::bikri::BikriSaved;
use mandi_services::core::edit::{DayFixCloseInput, DayFixOpenInput, DayFixOpened, EditEntity};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::db::audit::AuditEntry;
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

/// The new value (JSON text) of every audit `edit` of a بکری, oldest first.
fn bikri_edits(w: &World) -> Vec<Option<String>> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT new_json FROM audit_log WHERE action = 'edit' AND entity = 'bikri' ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([], |r| r.get(0))
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
fn an_edit_saved_in_the_fix_records_that_the_sign_was_given_and_never_its_text() {
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

    // The stamp of the edit: the detail, the list row, and the stamp command. It has no sign field.
    let detail = bikri::get(ctx, saved.bikri_id).unwrap();
    let stamp = detail.flags.edited.clone().unwrap();
    assert_eq!((stamp.count, stamp.reason.as_deref()), (1, Some("edit")));
    let listed = bikri::list(ctx, &Default::default()).unwrap();
    let row = listed.rows.iter().find(|r| r.id == saved.bikri_id).unwrap();
    assert!(row.flags.edited.is_some());
    let stamps = edit_stamps(
        ctx.db(),
        &EditStampInput {
            entity: EditEntity::Bikri,
            ids: vec![saved.bikri_id],
        },
    )
    .unwrap();
    assert!(stamps[0].stamp.is_some());

    // The history: the sign is not one of its changed fields.
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
    assert_eq!(version.reason.as_deref(), Some("edit"));
    assert!(
        version
            .changes
            .iter()
            .all(|c| c.path != "sign" && c.path != "sign_given")
    );
    assert!(
        version
            .changes
            .iter()
            .any(|c| c.path == "lots[0].sales[0].seller_rate_paisa"),
        "{:?}",
        version.changes
    );

    // The audit rows: the reopen (with its reason), the fix and the edit say the sign was given, and
    // none holds its text.
    for action in ["day_reopen", "day_fix_open", "edit"] {
        let rows = rows_of(&w, action);
        let last = rows.last().unwrap_or_else(|| panic!("no {action}"));
        let json: serde_json::Value =
            serde_json::from_str(last.0.as_deref().expect("a new value")).unwrap();
        assert_eq!(json["sign_given"], true, "{action}");
        assert!(json.get("sign").is_none(), "{action}: {json}");
    }
    assert_eq!(rows_of(&w, "day_reopen")[0].1.as_deref(), Some(REASON));
    let summary: String = raw(&w.dir)
        .query_row(
            "SELECT summary_json FROM day_closes WHERE action = 'reopen'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(summary.contains("\"sign_given\":true"), "{summary}");
    assert!(
        !summary.contains("adnan") && !summary.contains("\"sign\""),
        "{summary}"
    );

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
    assert!(
        bikri::get(ctx, open_day.bikri_id)
            .unwrap()
            .flags
            .edited
            .is_some()
    );
    // The edit of the open day wrote no `sign_given`; the edit of the closed day wrote one.
    let given: Vec<bool> = bikri_edits(&w)
        .iter()
        .map(|new| {
            serde_json::from_str::<serde_json::Value>(new.as_deref().unwrap_or("{}"))
                .unwrap()
                .get("sign_given")
                .is_some()
        })
        .collect();
    assert_eq!(given, [true, false]);
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
    assert!(
        bikri::get(ctx, today.bikri_id)
            .unwrap()
            .flags
            .edited
            .is_some()
    );
    let history = edit_history(
        ctx.db(),
        &EditHistoryInput {
            entity: EditEntity::Bikri,
            id: today.bikri_id,
        },
    )
    .unwrap();
    assert_eq!(history.versions.len(), 1);
    // The audit row of the closed day's edit says the sign was given; the edit of today does not.
    let edits = bikri_edits(&w);
    let given = |index: usize| {
        serde_json::from_str::<serde_json::Value>(edits[index].as_deref().unwrap())
            .unwrap()
            .get("sign_given")
            .is_some()
    };
    assert!(given(0), "the closed day's edit");
    assert!(!given(1), "today's edit");
    // Neither row holds the sign text.
    assert!(
        edits
            .iter()
            .all(|new| !new.as_deref().unwrap_or("").contains("adnan"))
    );
}

/// Plan 92 (PRD FR-21.4, Section 21): the sign is changed with the password AND the current sign.
/// A missing or wrong current sign changes nothing, is checked after the password, and a wrong one
/// is audited without the typed text.
#[test]
fn a_missing_or_wrong_current_sign_changes_nothing() {
    let (w, _) = closed_day();
    let ctx = &w.env.ctx;
    let set = |password: &str, current: &str, sign: &str| {
        owner_sign::set_owner_sign(ctx, &w.owner, &Secret::new(password), current, sign)
    };
    // Missing: refused at `current_sign`, nothing audited.
    assert_eq!(
        refusals(&set(OWNER_PASSWORD, "  ", "sana").unwrap_err()),
        [(ErrorCode::ErrSignRequired, Some("current_sign".into()))]
    );
    assert_eq!(count_of(&w, "owner_sign_change_sign_wrong"), 0);
    // The password is checked first: a wrong password with a wrong current sign says the password.
    assert_eq!(
        refusals(&set("not-the-password", "munshi", "sana").unwrap_err()),
        [(ErrorCode::ErrWrongPassword, Some("password".into()))]
    );
    // Wrong (and the new sign typed as if it were the current one): refused, audited, no text kept.
    for typed in ["munshi", "sana"] {
        assert_eq!(
            refusals(&set(OWNER_PASSWORD, typed, "sana").unwrap_err()),
            [(ErrorCode::ErrSignWrong, Some("current_sign".into()))],
            "{typed}"
        );
    }
    assert_eq!(count_of(&w, "owner_sign_change_sign_wrong"), 2);
    assert!(
        rows_of(&w, "owner_sign_change_sign_wrong")
            .iter()
            .all(|(new, _)| {
                let text = new.clone().unwrap_or_default().to_lowercase();
                !text.contains("munshi") && !text.contains("sana")
            })
    );
    // Nothing changed: the sign is still adnan, so it still opens, and no change was audited.
    assert!(
        rows_of(&w, "edit")
            .iter()
            .all(|(new, _)| { !new.clone().unwrap_or_default().contains("owner_sign") })
    );
    let opened = fix(&w, "adnan").unwrap();
    day::fix_close(
        ctx,
        &w.owner,
        &DayFixCloseInput {
            token: opened.token,
        },
    )
    .unwrap();
    // The right current sign, in any case and in Urdu while it is the default, changes it.
    set(OWNER_PASSWORD, " ADNAN", "sana").unwrap();
    // Now only the new sign is the current one.
    assert_eq!(
        refusals(&set(OWNER_PASSWORD, "adnan", "other").unwrap_err()),
        [(ErrorCode::ErrSignWrong, Some("current_sign".into()))]
    );
    set(OWNER_PASSWORD, "SANA ", "other").unwrap();
    assert!(fix(&w, "other").is_ok());
    assert_chain_ok(&w.dir);
}

#[test]
fn the_owner_changes_his_sign_with_his_password_and_only_the_new_sign_opens() {
    let (w, _) = closed_day();
    let ctx = &w.env.ctx;
    let set_from = |password: &str, current: &str, sign: &str| {
        owner_sign::set_owner_sign(ctx, &w.owner, &Secret::new(password), current, sign)
    };
    // The sign he has now is typed again (plan 92): "adnan" until he changes it.
    let set = |password: &str, sign: &str| set_from(password, "adnan", sign);
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
    // The sign he has now is the current sign from here on.
    set_from(OWNER_PASSWORD, &"y".repeat(40), "  Sana ").unwrap();
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
    // The change is in the audit log without the sign; the reopen says the sign was given, not which.
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
            .is_some_and(|json| json.contains("\"sign_given\":true")
                && !json.to_lowercase().contains("sana")),
        "{reopened:?}"
    );
    assert_chain_ok(&w.dir);
}

/// Plan 80 Q-R2-2 (PRD FR-21.2): the plain reopen asks the Owner's sign too. A wrong or missing sign
/// opens nothing, writes one audit entry without the typed text, and the right sign is kept in the
/// audit entry of the reopen beside the reason.
#[test]
fn the_plain_reopen_asks_the_owners_sign_too() {
    use mandi_services::core::vouchers::DayReopenInput;
    let (w, _) = closed_day();
    let reopen = |sign: &str, reason: &str| {
        day::reopen(
            &w.env.ctx,
            &w.owner,
            &DayReopenInput {
                date: "10-10-2026".into(),
                reason: reason.into(),
                sign: sign.into(),
            },
        )
    };
    // Missing sign, with a reason: refused on `sign`, nothing audited, still closed.
    let error = reopen("  ", REASON).unwrap_err();
    assert_eq!(
        refusals(&error),
        [(ErrorCode::ErrSignRequired, Some("sign".into()))]
    );
    assert_eq!(count_of(&w, "day_reopen_sign_wrong"), 0);
    // A wrong sign: refused, audited without what was typed, still closed.
    let error = reopen("munshi", REASON).unwrap_err();
    assert_eq!(
        refusals(&error),
        [(ErrorCode::ErrSignWrong, Some("sign".into()))]
    );
    assert_eq!(count_of(&w, "day_reopen_sign_wrong"), 1);
    assert!(
        !rows_of(&w, "day_reopen_sign_wrong")[0]
            .0
            .clone()
            .unwrap_or_default()
            .contains("munshi")
    );
    assert_eq!(count_of(&w, "day_reopen"), 0);
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));
    // The right sign opens the day; the audit entry says it was given, beside the reason.
    let status = reopen("ADNAN ", REASON).unwrap();
    assert_eq!(status.closed_up_to.as_deref(), Some("2026-10-09"));
    let rows = rows_of(&w, "day_reopen");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1.as_deref(), Some(REASON));
    let new = rows[0].0.clone().unwrap();
    assert!(
        new.contains("\"sign_given\":true") && !new.contains("adnan"),
        "{rows:?}"
    );
    assert_chain_ok(&w.dir);
}

/// A sign that cannot be a part of any name, number or word in the books, so that finding it in a
/// response means the sign leaked.
const SECRET_SIGN: &str = "zq7-sign-4417";

/// The audit viewer, one page of every row, filtered by `text` (empty: no filter).
fn audit_page(w: &World, text: &str) -> AuditPage {
    audit_list(
        w.env.ctx.db(),
        &AuditListInput {
            group_by_month: false,
            sort: None,
            filters: AuditFilters {
                text: text.to_owned(),
                ..AuditFilters::default()
            },
            offset: 0,
            limit: 1_000,
        },
    )
    .unwrap()
}

/// Plan 96 T4 (PRD FR-7.71 and FR-21.4, v1.58): the sign is a secret. No response of the stamp, the
/// history, the audit viewer (also by its text search), the بکری detail, list and find, or the day
/// status carries its text, and the audit rows hold only `sign_given`. A row written before it was
/// kept as `sign_given` (the log is append-only) still holds the text; every reader hides it.
#[test]
fn no_response_carries_the_sign_text() {
    let (w, saved) = closed_day();
    let ctx = &w.env.ctx;
    owner_sign::set_owner_sign(
        ctx,
        &w.owner,
        &Secret::new(OWNER_PASSWORD),
        "adnan",
        SECRET_SIGN,
    )
    .unwrap();
    // The reopen, the fix and the edit with the Owner's own sign.
    let opened = fix(&w, &format!(" {} ", SECRET_SIGN.to_uppercase())).unwrap();
    edit_rate(&w, &saved);
    // A row as the app wrote it before v1.58: the sign text under `sign`, in the new value.
    ctx.db()
        .write(|tx| {
            ctx.db().auditor().write(
                tx,
                &AuditEntry {
                    user_id: Some(w.owner.user_id),
                    action: "edit".into(),
                    entity: "bikri".into(),
                    entity_id: Some(saved.bikri_id),
                    old_json: Some(serde_json::json!({ "version": 1 })),
                    new_json: Some(serde_json::json!({ "version": 2, "sign": SECRET_SIGN })),
                    reason: Some("an old row".into()),
                },
            )
        })
        .unwrap();
    // A fix that is still open, and after it is closed.
    for closed in [false, true] {
        if closed {
            day::fix_close(
                ctx,
                &w.owner,
                &DayFixCloseInput {
                    token: opened.token.clone(),
                },
            )
            .unwrap();
        }
        let history = edit_history(
            ctx.db(),
            &EditHistoryInput {
                entity: EditEntity::Bikri,
                id: saved.bikri_id,
            },
        )
        .unwrap();
        let stamps = edit_stamps(
            ctx.db(),
            &EditStampInput {
                entity: EditEntity::Bikri,
                ids: vec![saved.bikri_id],
            },
        )
        .unwrap();
        let responses = [
            ("history", serde_json::to_string(&history).unwrap()),
            ("stamps", serde_json::to_string(&stamps).unwrap()),
            (
                "bikri detail",
                serde_json::to_string(&bikri::get(ctx, saved.bikri_id).unwrap()).unwrap(),
            ),
            (
                "bikri list",
                serde_json::to_string(&bikri::list(ctx, &Default::default()).unwrap()).unwrap(),
            ),
            (
                "bikri find",
                serde_json::to_string(&bikri::find(ctx, &BikriFindInput::default()).unwrap())
                    .unwrap(),
            ),
            ("day status", format!("{:?}", day::status(ctx).unwrap())),
            (
                "audit list",
                serde_json::to_string(&audit_page(&w, "")).unwrap(),
            ),
        ];
        for (name, json) in &responses {
            assert!(
                !json.to_lowercase().contains(SECRET_SIGN),
                "the sign is in the {name} (fix closed: {closed}): {json}"
            );
        }
        // The old row shows its sign as a hidden field, and the history leaves it out.
        let page = audit_page(&w, "an old row");
        assert_eq!(page.rows.len(), 1);
        assert!(
            page.rows[0]
                .changes
                .iter()
                .any(|c| c.path == "sign" && c.new == AuditValue::Hidden),
            "{:?}",
            page.rows[0].changes
        );
        assert!(history.versions.iter().all(|v| {
            v.changes
                .iter()
                .all(|c| c.path != "sign" && c.path != "sign_given")
        }));
        // A search for the sign finds nothing, in the old row or in the new ones.
        assert!(audit_page(&w, SECRET_SIGN).rows.is_empty());
        assert!(audit_page(&w, "ZQ7").rows.is_empty());
    }
    // The new rows say it was given, and the stored text is only in the old row.
    for action in ["day_reopen", "day_fix_open", "edit"] {
        let rows = rows_of(&w, action);
        assert!(!rows.is_empty(), "{action}");
        for (new, _) in rows
            .iter()
            .filter(|(_, reason)| reason.as_deref() != Some("an old row"))
        {
            let text = new.clone().unwrap_or_default();
            assert!(
                !text.to_lowercase().contains(SECRET_SIGN),
                "{action}: {text}"
            );
            if action != "edit" || text.contains("sign_given") {
                assert!(text.contains("\"sign_given\":true"), "{action}: {text}");
            }
        }
    }
    let summary: String = raw(&w.dir)
        .query_row(
            "SELECT summary_json FROM day_closes WHERE action = 'reopen'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!summary.to_lowercase().contains(SECRET_SIGN), "{summary}");
    // The pending-fix setting and every other setting never held the text either, except the sign
    // the Owner chose, which is the one stored value (a follow-up for Umar: keep it as a hash).
    let leaked: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM settings WHERE key <> 'owner_sign' AND instr(lower(value_json), ?1) > 0",
            [SECRET_SIGN],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(leaked, 0);
    assert_chain_ok(&w.dir);
}
