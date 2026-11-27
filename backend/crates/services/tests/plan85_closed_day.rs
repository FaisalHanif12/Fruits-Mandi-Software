//! Plan 85 T1 (PRD FR-21.4 v1.49, FR-21.1, FR-21.2, FR-6.6): a NEW entry on a closed day is saved
//! through "reopen with the reason and the sign, save, close again", and the automatic close leaves
//! alone a day that is in use.
//!
//! `day_fix_open` takes no entry: only the date, the reason and the Owner's sign. The token it gives
//! holds the days it reopened, and `day_fix_close` closes exactly those days again with the checks of
//! FR-21.1. The data is the PRD worked day (4.4): بکری 1 on 10-10-2026 (اکرم 20 at 600, ارشد 30 at 580).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::{assert_chain_ok, raw, set_today};
use mandi_services::core::bikri::BikriDraft;
use mandi_services::core::edit::{DayFixCloseInput, DayFixOpenInput, DayFixOpened};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::{DayCloseInput, DayReopenInput};
use mandi_services::day::AutoCloseHold;
use mandi_services::{ServiceError, bikri, day};
use support::World;

const REASON: &str = "بکری دیر سے لکھی";
const SIGN: &str = "adnan";

fn close_up_to(w: &World, date: &str) {
    day::close(&w.env.ctx, &w.owner, &DayCloseInput { up_to: date.into() }).unwrap();
}

fn closed_up_to(w: &World) -> Option<String> {
    day::status(&w.env.ctx).unwrap().closed_up_to
}

fn fix_open(w: &World, date: &str, sign: &str) -> Result<DayFixOpened, ServiceError> {
    day::fix_open(
        &w.env.ctx,
        &w.owner,
        &DayFixOpenInput {
            date: date.into(),
            reason: REASON.into(),
            sign: sign.into(),
        },
    )
}

/// بکری 1 of the worked day, dated `date` (`DD-MM-YYYY`).
fn bikri_on(w: &World, date: &str) -> BikriDraft {
    let mut draft = support::event1(w);
    draft.header.entry_date = date.into();
    draft
}

fn save(
    w: &World,
    draft: &BikriDraft,
) -> Result<mandi_services::core::bikri::BikriSaved, ServiceError> {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: false,
        },
    )
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

/// The audit rows of the day area and of the بکری save, oldest first, as `(action, new_json)`.
fn audit_trail(w: &World) -> Vec<(String, String)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT action, COALESCE(new_json, '') FROM audit_log
             WHERE action IN ('day_close', 'day_reopen', 'day_fix_open', 'day_fix_close',
                              'day_fix_sign_wrong', 'create')
             ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn journal_dates(w: &World) -> Vec<String> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare("SELECT entry_date FROM journal_entries WHERE posting_type = 'bikri_first'")
        .unwrap();
    statement
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn a_new_bikri_on_a_closed_today_is_saved_through_reopen_and_closed_again() {
    let w = support::world();
    // Today is 10-10-2026 and the automatic close has closed it at night.
    support::save_old(&w, &support::event1(&w), false);
    close_up_to(&w, "10-10-2026");
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));

    // A second بکری dated today is refused: the day is closed, and the date is the field.
    let draft = bikri_on(&w, "10-10-2026");
    let refused = save(&w, &draft).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrDateClosed);

    // Reopen with the reason and the sign. No entry is named: only the date.
    let opened = fix_open(&w, "10-10-2026", SIGN).unwrap();
    assert_eq!(opened.days, vec!["2026-10-10".to_owned()]);
    assert_eq!(opened.closed_up_to.as_deref(), Some("2026-10-09"));

    // The same save now posts on its own date.
    let saved = save(&w, &draft).unwrap();
    assert_eq!(saved.number, 2);
    assert_eq!(journal_dates(&w), vec!["2026-10-10", "2026-10-10"]);

    // Closing again closes the days the token holds, and nothing stays open.
    let closed = day::fix_close(
        &w.env.ctx,
        &w.owner,
        &DayFixCloseInput {
            token: opened.token,
        },
    )
    .unwrap();
    assert_eq!(closed.closed, vec!["2026-10-10".to_owned()]);
    assert!(closed.stayed_open.is_empty());
    assert_eq!(closed.closed_up_to.as_deref(), Some("2026-10-10"));
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));
    assert!(!day::status(&w.env.ctx).unwrap().today_open);

    // The audit has the reopen with the reason and that the sign was given (never the sign, PRD
    // FR-21.4 v1.58), the save, and the close, in order.
    let trail = audit_trail(&w);
    let actions: Vec<&str> = trail.iter().map(|(a, _)| a.as_str()).collect();
    let position = |name: &str, from: usize| {
        actions[from..]
            .iter()
            .position(|a| *a == name)
            .map(|p| p + from)
            .unwrap_or_else(|| panic!("no {name} after {from} in {actions:?}"))
    };
    let reopen = position("day_reopen", 0);
    let fix = position("day_fix_open", reopen);
    let create = position("create", fix);
    let fix_close = position("day_fix_close", create);
    assert!(reopen < fix && fix < create && create < fix_close);
    for row in [&trail[reopen], &trail[fix]] {
        assert!(row.1.contains("\"sign_given\":true"), "{row:?}");
        assert!(!row.1.contains(SIGN), "{row:?}");
    }
    assert!(trail[fix_close].1.contains("2026-10-10"));
    assert_chain_ok(&w.dir);
}

#[test]
fn a_new_bikri_on_an_older_closed_day_reopens_every_later_day_and_closes_them_all_again() {
    let w = support::world();
    // 10-10, 11-10 and 12-10 are closed in one step; today is 12-10.
    support::save_old(&w, &support::event1(&w), false);
    set_today(&w.env, 2026, 10, 12);
    close_up_to(&w, "12-10-2026");

    // A بکری of 11-10 was forgotten. The token holds 11-10 and 12-10.
    let draft = bikri_on(&w, "11-10-2026");
    assert_eq!(
        save(&w, &draft).unwrap_err().code(),
        ErrorCode::ErrDateClosed
    );
    let opened = fix_open(&w, "11-10-2026", SIGN).unwrap();
    assert_eq!(
        opened.days,
        vec!["2026-10-11".to_owned(), "2026-10-12".to_owned()]
    );
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));
    save(&w, &draft).unwrap();

    let closed = day::fix_close(
        &w.env.ctx,
        &w.owner,
        &DayFixCloseInput {
            token: opened.token,
        },
    )
    .unwrap();
    assert_eq!(
        closed.closed,
        vec!["2026-10-11".to_owned(), "2026-10-12".to_owned()]
    );
    assert!(closed.stayed_open.is_empty());
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-12"));
    assert_eq!(journal_dates(&w), vec!["2026-10-10", "2026-10-11"]);
    assert_chain_ok(&w.dir);
}

#[test]
fn a_wrong_sign_opens_nothing_and_a_new_bikri_stays_refused() {
    let w = support::world();
    support::save_old(&w, &support::event1(&w), false);
    close_up_to(&w, "10-10-2026");
    let draft = bikri_on(&w, "10-10-2026");

    for typed in ["munshi", "عدنان خان", "adnan1"] {
        let refused = fix_open(&w, "10-10-2026", typed).unwrap_err();
        assert_eq!(refused.code(), ErrorCode::ErrSignWrong, "{typed}");
    }
    // Nothing opened and nothing is held: the day is still closed, the save is still refused, and a
    // token nobody was given closes nothing.
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));
    assert_eq!(
        save(&w, &draft).unwrap_err().code(),
        ErrorCode::ErrDateClosed
    );
    assert_eq!(day::auto_close_hold(&w.env.ctx).unwrap(), None);
    let stray = day::fix_close(
        &w.env.ctx,
        &w.owner,
        &DayFixCloseInput {
            token: "fix-0000".into(),
        },
    )
    .unwrap_err();
    assert_eq!(stray.code(), ErrorCode::ErrFixTokenInvalid);
    // Each wrong try is one audit row, and none of them holds the typed text.
    assert_eq!(count_of(&w, "day_fix_sign_wrong"), 3);
    assert_eq!(count_of(&w, "day_fix_open"), 0);
    for (action, new_json) in audit_trail(&w) {
        if action == "day_fix_sign_wrong" {
            for typed in ["munshi", "عدنان خان", "adnan1"] {
                assert!(!new_json.contains(typed), "{new_json}");
            }
        }
    }
}

#[test]
fn leaving_without_saving_closes_the_same_days_again() {
    let w = support::world();
    support::save_old(&w, &support::event1(&w), false);
    close_up_to(&w, "10-10-2026");
    let opened = fix_open(&w, "10-10-2026", SIGN).unwrap();
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-09"));
    // The Owner changes his mind: the close of the fix still has the same days.
    let closed = day::fix_close(
        &w.env.ctx,
        &w.owner,
        &DayFixCloseInput {
            token: opened.token,
        },
    )
    .unwrap();
    assert_eq!(closed.closed, vec!["2026-10-10".to_owned()]);
    assert_eq!(closed_up_to(&w).as_deref(), Some("2026-10-10"));
}

// ---- What the automatic close must leave alone (the answers of `auto_close_hold`) ----

#[test]
fn a_fix_in_progress_holds_the_automatic_close_until_it_is_closed() {
    let w = support::world();
    support::save_old(&w, &support::event1(&w), false);
    assert_eq!(day::auto_close_hold(&w.env.ctx).unwrap(), None);
    close_up_to(&w, "10-10-2026");
    assert_eq!(day::auto_close_hold(&w.env.ctx).unwrap(), None);

    let opened = fix_open(&w, "10-10-2026", SIGN).unwrap();
    assert_eq!(
        day::auto_close_hold(&w.env.ctx).unwrap(),
        Some(AutoCloseHold::FixInProgress)
    );
    day::fix_close(
        &w.env.ctx,
        &w.owner,
        &DayFixCloseInput {
            token: opened.token,
        },
    )
    .unwrap();
    assert_eq!(day::auto_close_hold(&w.env.ctx).unwrap(), None);
}

#[test]
fn a_fix_nobody_finished_stops_holding_after_the_hold_time() {
    let w = support::world();
    support::save_old(&w, &support::event1(&w), false);
    close_up_to(&w, "10-10-2026");
    fix_open(&w, "10-10-2026", SIGN).unwrap();
    assert_eq!(
        day::auto_close_hold(&w.env.ctx).unwrap(),
        Some(AutoCloseHold::FixInProgress)
    );
    // The app was closed in the middle of the fix: nobody holds the token any more.
    let opened_at = w.env.ctx.now_utc();
    w.env
        .clock
        .set(opened_at + chrono::Duration::hours(day::FIX_HOLD_HOURS - 1));
    assert_eq!(
        day::auto_close_hold(&w.env.ctx).unwrap(),
        Some(AutoCloseHold::FixInProgress)
    );
    w.env
        .clock
        .set(opened_at + chrono::Duration::hours(day::FIX_HOLD_HOURS + 1));
    assert_eq!(day::auto_close_hold(&w.env.ctx).unwrap(), None);
}

#[test]
fn a_day_the_owner_reopened_by_hand_is_held_until_he_closes_it() {
    let w = support::world();
    support::save_old(&w, &support::event1(&w), false);
    set_today(&w.env, 2026, 10, 12);
    close_up_to(&w, "12-10-2026");
    assert_eq!(day::auto_close_hold(&w.env.ctx).unwrap(), None);

    // He reopens 11-10 by hand (and with it 12-10), with his sign.
    day::reopen(
        &w.env.ctx,
        &w.owner,
        &DayReopenInput {
            date: "11-10-2026".into(),
            reason: "غلطی درست کرنی ہے".into(),
            sign: SIGN.into(),
        },
    )
    .unwrap();
    assert_eq!(
        day::auto_close_hold(&w.env.ctx).unwrap(),
        Some(AutoCloseHold::ReopenedByHand)
    );
    // The fact is in the data file, so a restart keeps it.
    let stored: String = raw(&w.dir)
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'day_manual_reopen'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        stored.contains("2026-10-11") && stored.contains("2026-10-12"),
        "{stored}"
    );

    // He closes only 11-10: 12-10 is still his, so the hold stays.
    close_up_to(&w, "11-10-2026");
    assert_eq!(
        day::auto_close_hold(&w.env.ctx).unwrap(),
        Some(AutoCloseHold::ReopenedByHand)
    );
    // He closes up to the last day he opened: the hold ends.
    close_up_to(&w, "12-10-2026");
    assert_eq!(day::auto_close_hold(&w.env.ctx).unwrap(), None);
}

#[test]
fn a_second_reopen_by_hand_widens_the_hold() {
    let w = support::world();
    support::save_old(&w, &support::event1(&w), false);
    set_today(&w.env, 2026, 10, 12);
    close_up_to(&w, "12-10-2026");
    let reopen = |date: &str| {
        day::reopen(
            &w.env.ctx,
            &w.owner,
            &DayReopenInput {
                date: date.into(),
                reason: "غلطی".into(),
                sign: SIGN.into(),
            },
        )
        .unwrap();
    };
    reopen("12-10-2026");
    close_up_to(&w, "12-10-2026");
    assert_eq!(day::auto_close_hold(&w.env.ctx).unwrap(), None);
    reopen("11-10-2026");
    assert_eq!(
        day::auto_close_hold(&w.env.ctx).unwrap(),
        Some(AutoCloseHold::ReopenedByHand)
    );
    close_up_to(&w, "12-10-2026");
    assert_eq!(day::auto_close_hold(&w.env.ctx).unwrap(), None);
}
