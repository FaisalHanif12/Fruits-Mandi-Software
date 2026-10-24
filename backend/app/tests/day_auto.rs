//! The automatic day close (plan 19, PRD Section 21 "Day", FR-21.1): it closes today at the
//! closing time through the same `day::close` the screen uses, only when the switch is on, today
//! is open and the Owner has no unsaved draft, and it never closes twice.
#![cfg(test)]

use chrono::{TimeZone, Utc};
use mandi_app_lib::day_auto::{Memory, Outcome, run_once};
use mandi_backup::day_settings::{self, DaySettings};
use mandi_services::auth::{self, FirstRunRaw};
use mandi_services::core::drafts::DraftSaveInput;
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
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
        person_key: None,
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
