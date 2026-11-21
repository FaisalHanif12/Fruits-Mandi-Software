//! The automatic day close (plan 19, PRD Section 21 "Day", FR-21.1): it closes today at the
//! closing time through the same `day::close` the screen uses, only when the switch is on, today
//! is open and the Owner has no unsaved draft, and it never closes twice.
#![cfg(test)]

use chrono::{TimeZone, Utc};
use mandi_app_lib::day_auto::{Memory, Outcome, run_once};
use mandi_backup::day_settings::{self, DaySettings};
use mandi_services::auth::{self, FirstRunRaw};
use mandi_services::core::drafts::DraftSaveInput;
use mandi_services::core::edit::{DayFixCloseInput, DayFixOpenInput};
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::vouchers::{DayCloseInput, DayReopenInput};
use mandi_services::day::AutoCloseHold;
use mandi_services::settings::Firm;
use mandi_services::testing::TestEnv;
use mandi_services::{Secret, day, drafts, vouchers};
use tempfile::TempDir;

const PASSWORD: &str = "secret1";

fn world() -> (TempDir, TestEnv) {
    let dir = tempfile::tempdir().unwrap();
    let env = TestEnv::new(dir.path()).unwrap();
    auth::first_run(
        &env.ctx,
        &FirstRunRaw {
            firm: Firm {
                name_ur: "ماہر عدنان اینڈ برادرز".to_owned(),
                name_en: "Maher Adnan and Brothers".to_owned(),
                mandi_ur: "نئی سبزی منڈی".to_owned(),
                mandi_en: "New Sabzi Mandi".to_owned(),
                address: "Kot Radha Kishan".to_owned(),
                phone: "0300-1234567".to_owned(),
            },
            owner_username: "Owner".to_owned(),
            owner_display_name: "Maher Adnan".to_owned(),
            password: Secret::new(PASSWORD),
            password_repeat: Secret::new(PASSWORD),
        },
    )
    .unwrap();
    auth::login(&env.ctx, "owner", &Secret::new(PASSWORD)).unwrap();
    (dir, env)
}

/// Sets the clock to a shop time (Pakistan, UTC+5) on 10-10-26.
fn shop_time(env: &TestEnv, hour: u32, minute: u32) {
    let utc_hour = hour.checked_sub(5).unwrap();
    env.clock.set(
        Utc.with_ymd_and_hms(2026, 10, 10, utc_hour, minute, 0)
            .single()
            .unwrap(),
    );
}

fn account(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|conn| {
            conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
                r.get(0)
            })
        })
        .unwrap()
}

/// One posting today so there is a day to close: cash 1,000 against the opening capital.
fn post_something(env: &TestEnv) {
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    let owner = auth::require(&env.ctx).unwrap();
    vouchers::general_save(
        &env.ctx,
        &owner,
        &GeneralInput {
            date: "10-10-2026".into(),
            narration: None,
            lines: vec![
                line(account(env, "01010001"), "1,000", ""),
                line(account(env, "06010001"), "", "1,000"),
            ],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

fn switch(env: &TestEnv, on: bool, closing: &str) {
    day_settings::save(
        env.ctx.db(),
        None,
        &DaySettings {
            closing_time: closing.to_owned(),
            auto_day_close: on,
        },
    )
    .unwrap();
}

fn closed_up_to(env: &TestEnv) -> Option<String> {
    day::status(&env.ctx).unwrap().closed_up_to
}

#[test]
fn it_closes_today_at_the_closing_time_and_not_twice() {
    let (_dir, env) = world();
    shop_time(&env, 12, 0);
    post_something(&env);
    switch(&env, true, "21:00");
    let mut memory = Memory::default();

    // Before the closing time: nothing.
    shop_time(&env, 20, 59);
    assert_eq!(run_once(&env.ctx, &mut memory), Outcome::NotDue);
    assert_eq!(closed_up_to(&env), None);

    // At the closing time: today is closed, with an audit row under the Owner.
    shop_time(&env, 21, 0);
    assert_eq!(
        run_once(&env.ctx, &mut memory),
        Outcome::Closed("2026-10-10".to_owned())
    );
    assert_eq!(closed_up_to(&env).as_deref(), Some("2026-10-10"));
    let audited: i64 = env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM audit_log WHERE action = 'day_close' AND entity = 'day'",
                [],
                |r| r.get(0),
            )
        })
        .unwrap();
    assert_eq!(audited, 1, "the close wrote its audit row");
    // And one row says it was automatic, with the day.
    let automatic: Vec<String> = env
        .ctx
        .db()
        .read(|conn| {
            let mut statement = conn.prepare(
                "SELECT new_json FROM audit_log
                 WHERE action = 'day_auto_close' AND entity = 'day_settings'",
            )?;
            statement
                .query_map([], |r| r.get(0))?
                .collect::<Result<_, _>>()
        })
        .unwrap();
    assert_eq!(
        automatic,
        vec!["{\"closed_up_to\":\"2026-10-10\"}".to_owned()]
    );

    // Five minutes later: today is closed, nothing to do.
    shop_time(&env, 21, 5);
    assert_eq!(run_once(&env.ctx, &mut memory), Outcome::NotDue);
}

#[test]
fn it_does_nothing_when_the_switch_is_off() {
    let (_dir, env) = world();
    shop_time(&env, 12, 0);
    post_something(&env);
    switch(&env, false, "21:00");
    shop_time(&env, 22, 0);
    assert_eq!(run_once(&env.ctx, &mut Memory::default()), Outcome::NotDue);
    assert_eq!(closed_up_to(&env), None);
}

#[test]
fn it_waits_while_the_owner_has_an_unsaved_draft() {
    let (_dir, env) = world();
    shop_time(&env, 12, 0);
    post_something(&env);
    switch(&env, true, "21:00");
    let owner = auth::require(&env.ctx).unwrap();
    drafts::save(
        &env.ctx,
        &owner,
        &DraftSaveInput {
            screen: "bikri".into(),
            payload_json: "{}".into(),
        },
    )
    .unwrap();
    shop_time(&env, 21, 30);
    let mut memory = Memory::default();
    assert_eq!(run_once(&env.ctx, &mut memory), Outcome::SkippedDrafts);
    assert_eq!(closed_up_to(&env), None);
    // The draft is saved or discarded: the next tick closes the day.
    drafts::discard(
        &env.ctx,
        &owner,
        &mandi_services::core::drafts::DraftDiscardInput {
            screen: "bikri".into(),
        },
    )
    .unwrap();
    assert_eq!(
        run_once(&env.ctx, &mut memory),
        Outcome::Closed("2026-10-10".to_owned())
    );
}

#[test]
fn the_switch_is_off_until_the_owner_turns_it_on() {
    let (_dir, env) = world();
    let settings = env.ctx.db().read(day_settings::load).unwrap();
    assert!(!settings.auto_day_close);
    assert_eq!(settings.closing_time, "21:00");
    // A fresh book at 22:00 with a day to close still does nothing.
    shop_time(&env, 12, 0);
    post_something(&env);
    shop_time(&env, 22, 0);
    assert_eq!(run_once(&env.ctx, &mut Memory::default()), Outcome::NotDue);
    assert_eq!(closed_up_to(&env), None);
}

// ---- Plan 85: a day in use is never closed under the Owner's hands (PRD FR-21.4 v1.49) ----

const SIGN: &str = "adnan";

/// Sets the clock to a shop time (UTC+5) on a day of October 2026.
fn shop_time_on(env: &TestEnv, day: u32, hour: u32, minute: u32) {
    env.clock.set(
        Utc.with_ymd_and_hms(2026, 10, day, hour.checked_sub(5).unwrap(), minute, 0)
            .single()
            .unwrap(),
    );
}

fn close_by_hand(env: &TestEnv, up_to: &str) {
    let owner = auth::require(&env.ctx).unwrap();
    day::close(
        &env.ctx,
        &owner,
        &DayCloseInput {
            up_to: up_to.into(),
        },
    )
    .unwrap();
}

fn fix_open(env: &TestEnv) -> String {
    let owner = auth::require(&env.ctx).unwrap();
    day::fix_open(
        &env.ctx,
        &owner,
        &DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: "نئی بکری لکھنی ہے".into(),
            sign: SIGN.into(),
        },
    )
    .unwrap()
    .token
}

#[test]
fn it_leaves_a_day_held_by_a_fix_and_closes_nothing_until_the_fix_ends() {
    let (_dir, env) = world();
    shop_time_on(&env, 10, 12, 0);
    post_something(&env);
    switch(&env, true, "21:00");
    // The day was closed, and the Owner reopens it to save a new entry (the fix flow).
    close_by_hand(&env, "10-10-2026");
    let token = fix_open(&env);
    assert_eq!(closed_up_to(&env).as_deref(), Some("2026-10-09"));

    // The closing time passes while he works: every tick leaves the day open.
    let mut memory = Memory::default();
    for (hour, minute) in [(21, 0), (21, 5), (22, 30)] {
        shop_time_on(&env, 10, hour, minute);
        assert_eq!(
            run_once(&env.ctx, &mut memory),
            Outcome::Held(AutoCloseHold::FixInProgress)
        );
        assert_eq!(closed_up_to(&env).as_deref(), Some("2026-10-09"));
    }

    // The fix ends: its own close takes the days, and the timer finds nothing to do.
    let owner = auth::require(&env.ctx).unwrap();
    let closed = day::fix_close(&env.ctx, &owner, &DayFixCloseInput { token }).unwrap();
    assert_eq!(closed.closed, vec!["2026-10-10".to_owned()]);
    assert_eq!(closed_up_to(&env).as_deref(), Some("2026-10-10"));
    assert_eq!(run_once(&env.ctx, &mut memory), Outcome::NotDue);
}

#[test]
fn it_leaves_a_day_the_owner_reopened_by_hand_until_he_closes_it() {
    let (_dir, env) = world();
    shop_time_on(&env, 10, 12, 0);
    post_something(&env);
    switch(&env, true, "21:00");

    // At 21:00 the timer closes today, as it always did.
    shop_time_on(&env, 10, 21, 0);
    assert_eq!(
        run_once(&env.ctx, &mut Memory::default()),
        Outcome::Closed("2026-10-10".to_owned())
    );

    // At 21:03 the Owner reopens it by hand, with his reason and his sign.
    shop_time_on(&env, 10, 21, 3);
    let owner = auth::require(&env.ctx).unwrap();
    day::reopen(
        &env.ctx,
        &owner,
        &DayReopenInput {
            date: "10-10-2026".into(),
            reason: "ایک اندراج رہ گیا".into(),
            sign: SIGN.into(),
        },
    )
    .unwrap();
    assert_eq!(closed_up_to(&env).as_deref(), Some("2026-10-09"));

    // Before the change the next tick closed it again within five minutes. Now it is his: a new
    // thread (a restart) holds just the same, because the fact is in the data file.
    for (hour, minute) in [(21, 5), (21, 30), (23, 0)] {
        shop_time_on(&env, 10, hour, minute);
        assert_eq!(
            run_once(&env.ctx, &mut Memory::default()),
            Outcome::Held(AutoCloseHold::ReopenedByHand)
        );
        assert_eq!(closed_up_to(&env).as_deref(), Some("2026-10-09"));
    }

    // He closes it himself. The next day the timer works as before.
    close_by_hand(&env, "10-10-2026");
    shop_time_on(&env, 10, 23, 30);
    assert_eq!(run_once(&env.ctx, &mut Memory::default()), Outcome::NotDue);
    shop_time_on(&env, 11, 21, 30);
    assert_eq!(
        run_once(&env.ctx, &mut Memory::default()),
        Outcome::Closed("2026-10-11".to_owned())
    );
    assert_eq!(closed_up_to(&env).as_deref(), Some("2026-10-11"));
}

#[test]
fn it_closes_an_idle_day_when_a_fix_was_left_behind() {
    let (_dir, env) = world();
    shop_time_on(&env, 10, 12, 0);
    post_something(&env);
    switch(&env, true, "21:00");
    close_by_hand(&env, "10-10-2026");
    let token = fix_open(&env);

    // The app was closed in the middle of the fix. That evening the day is still held ...
    shop_time_on(&env, 10, 21, 30);
    assert_eq!(
        run_once(&env.ctx, &mut Memory::default()),
        Outcome::Held(AutoCloseHold::FixInProgress)
    );
    // ... but the next evening nobody holds the token (the hold time has passed), so the timer
    // closes the open days as it closes any idle day.
    shop_time_on(&env, 11, 21, 30);
    assert_eq!(
        run_once(&env.ctx, &mut Memory::default()),
        Outcome::Closed("2026-10-11".to_owned())
    );
    assert_eq!(closed_up_to(&env).as_deref(), Some("2026-10-11"));
    // The old token closes nothing.
    let owner = auth::require(&env.ctx).unwrap();
    assert!(day::fix_close(&env.ctx, &owner, &DayFixCloseInput { token }).is_err());
}

#[test]
fn it_closes_an_idle_day_after_the_closing_time() {
    // The timer still works for a day nobody is using: no fix, no reopen by hand, no draft.
    let (_dir, env) = world();
    shop_time_on(&env, 10, 12, 0);
    post_something(&env);
    switch(&env, true, "21:00");
    shop_time_on(&env, 10, 21, 40);
    assert_eq!(
        run_once(&env.ctx, &mut Memory::default()),
        Outcome::Closed("2026-10-10".to_owned())
    );
}
