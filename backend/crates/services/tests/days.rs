//! Day close and reopen (plan 10, PRD FR-21.1, FR-21.2, FR-6.6, NFR-REL-8, acceptance D2-A15).
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountType, BalanceSide, OpeningLineInput, OpeningsInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{
    AdvanceInput, DayCloseInput, DayClosePreviewInput, DayReopenInput,
};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, day, vouchers};
use tempfile::TempDir;

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

/// An opening of 65,000 on cash and 65,000 on capital, dated `date`, with the clock on that day.
fn open_books(env: &TestEnv, owner: &auth::Actor, date: &str, day: (i32, u32, u32)) {
    set_today(env, day.0, day.1, day.2);
    let id = |code: &str| {
        env.ctx
            .db()
            .read(|c| mandi_services::db::accounts::id_by_code(c, code))
            .unwrap()
            .unwrap()
    };
    let line = |account: i64, dr: &str, cr: &str| OpeningLineInput {
        account_id: account,
        version: accounts::get(&env.ctx, account).unwrap().version,
        opening_date: date.into(),
        opening_dr: dr.into(),
        opening_cr: cr.into(),
        opening_is_advance: false,
    };
    accounts::openings_save(
        &env.ctx,
        owner,
        &OpeningsInput {
            lines: vec![
                line(id("01010001"), "65,000", ""),
                line(id("06010001"), "", "65,000"),
            ],
        },
    )
    .unwrap();
}

fn seller(env: &TestEnv, owner: &auth::Actor) -> i64 {
    accounts::save(
        &env.ctx,
        owner,
        &AccountInput {
            id: None,
            version: None,
            account_type: AccountType::Seller,
            control_id: None,
            code: String::new(),
            name_ur: "رحیم".into(),
            name_en: "Rahim".into(),
            confirmed_rules_words: Vec::new(),
            father_or_firm_ur: String::new(),
            father_or_firm_en: String::new(),
            city_id: None,
            adda_id: None,
            phone1: String::new(),
            phone2: String::new(),
            address: String::new(),
            linked_account_id: None,
            opening_date: String::new(),
            opening_dr: String::new(),
            opening_cr: String::new(),
            opening_is_advance: false,
            credit_allowed: true,
            credit_limit: String::new(),
            default_marka_id: None,
            collector_staff_id: None,
            collection_weekday: None,
            is_active: true,
            note: String::new(),
        },
    )
    .unwrap()
    .id
}

fn give(
    env: &TestEnv,
    owner: &auth::Actor,
    rahim: i64,
    date: &str,
    amount: &str,
) -> Result<(), ServiceError> {
    vouchers::save_advance(
        &env.ctx,
        owner,
        &AdvanceInput {
            confirm_cash_below_zero: true,
            date: date.into(),
            seller_account_id: rahim,
            amount: amount.into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .map(|_| ())
}

fn preview(
    env: &TestEnv,
    up_to: &str,
) -> Result<mandi_services::core::vouchers::DayClosePreview, ServiceError> {
    day::close_preview(
        &env.ctx,
        &DayClosePreviewInput {
            up_to: up_to.into(),
        },
    )
}

fn close(
    env: &TestEnv,
    owner: &auth::Actor,
    up_to: &str,
) -> Result<day::DayStatusInfo, ServiceError> {
    day::close(
        &env.ctx,
        owner,
        &DayCloseInput {
            up_to: up_to.into(),
        },
    )
}

fn reopen(
    env: &TestEnv,
    owner: &auth::Actor,
    date: &str,
    reason: &str,
) -> Result<day::DayStatusInfo, ServiceError> {
    day::reopen(
        &env.ctx,
        owner,
        &DayReopenInput {
            date: date.into(),
            reason: reason.into(),
        },
    )
}

fn error_of(error: ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        ServiceError::Rejected(items) => (items[0].code, items[0].field.clone()),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn count(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn the_oldest_open_day_is_the_first_posting_or_today() {
    let (_dir, env, owner) = ready();
    // Nothing posted, nothing closed: the oldest open day is today.
    set_today(&env, 2026, 10, 10);
    let p = preview(&env, "").unwrap();
    assert_eq!(
        (p.from_date.as_str(), p.up_to.as_str(), p.days, p.entries),
        ("2026-10-10", "2026-10-10", 1, 0)
    );
    assert_eq!(p.cash_in_hand.side, BalanceSide::Zero);
    // Books opened on 31-08: the oldest open day is 31-08.
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    set_today(&env, 2026, 10, 10);
    let p = preview(&env, "").unwrap();
    assert_eq!(
        (p.from_date.as_str(), p.up_to.as_str(), p.days, p.entries),
        ("2026-08-31", "2026-08-31", 1, 2)
    );
    assert_eq!(
        (p.trial_dr_paisa, p.trial_cr_paisa),
        (rupees(65_000), rupees(65_000))
    );
    // A date before the first posting is a one-day close of that empty day, never 0 days.
    let p = preview(&env, "15-08-2026").unwrap();
    assert_eq!(
        (p.from_date.as_str(), p.up_to.as_str(), p.days, p.entries),
        ("2026-08-15", "2026-08-15", 1, 0)
    );
    assert_eq!(
        p.cash_in_hand.side,
        BalanceSide::Zero,
        "cash as of 15-08 is before the opening"
    );
    // A date up to today closes every day from the first posting: 31-08 to 10-10 is 41 days.
    let p = preview(&env, "10-10-2026").unwrap();
    assert_eq!(p.days, 41);
    assert_eq!(preview(&env, "09-10-2026").unwrap().days, 40);
}

#[test]
fn days_are_closed_in_order_and_an_earlier_day_is_refused() {
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    set_today(&env, 2026, 9, 3);
    // One day at a time, the oldest first.
    for (expected, today_open) in [
        ("2026-08-31", true),
        ("2026-09-01", true),
        ("2026-09-02", true),
    ] {
        let status = close(&env, &owner, "").unwrap();
        assert_eq!(status.closed_up_to.as_deref(), Some(expected));
        assert_eq!(status.today_open, today_open);
    }
    // A day that is already closed is refused, and so is a day after today.
    let error = close(&env, &owner, "01-09-2026").unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!("not a rejection")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrDateClosed, Some("up_to"))
    );
    assert_eq!(
        items[0].values.get("closed_up_to"),
        Some(&ErrorValue::Text("02-09-2026".into()))
    );
    assert_eq!(
        error_of(close(&env, &owner, "04-09-2026").unwrap_err()).0,
        ErrorCode::ErrDateInFuture
    );
    assert_eq!(
        error_of(preview(&env, "02-09-2026").unwrap_err()).0,
        ErrorCode::ErrDateClosed
    );
    // Several days in one step: closes up to and including the chosen day.
    set_today(&env, 2026, 9, 10);
    let status = close(&env, &owner, "07-09-2026").unwrap();
    assert_eq!(status.closed_up_to.as_deref(), Some("2026-09-07"));
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM day_closes WHERE action = 'close'"
        ),
        8
    );
    // Closing today is allowed (the close at the end of the day).
    assert_eq!(
        close(&env, &owner, "10-09-2026")
            .unwrap()
            .closed_up_to
            .as_deref(),
        Some("2026-09-10")
    );
    // Nothing is left open up to today: an empty close has nothing to do.
    let (code, field) = error_of(close(&env, &owner, "").unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrNothingToClose, Some("up_to"))
    );
    // The preview says "0 days" with today's numbers, so the screen can show it.
    let none = preview(&env, "").unwrap();
    assert_eq!(
        (none.days, none.up_to.as_str(), none.entries),
        (0, "2026-09-10", 0)
    );
}

#[test]
fn a_close_writes_the_history_the_audit_row_and_the_new_date() {
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    let rahim = seller(&env, &owner);
    set_today(&env, 2026, 9, 1);
    give(&env, &owner, rahim, "01-09-2026", "15,000").unwrap();
    set_today(&env, 2026, 10, 10);
    close(&env, &owner, "09-10-2026").unwrap();
    // One history row for each of the 40 days closed (Architecture 15.3), oldest first.
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM day_closes"), 40);
    let rows: Vec<(String, String, i64, String)> = raw(&dir)
        .prepare("SELECT entry_date, action, by_user_id, summary_json FROM day_closes ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(rows.first().map(|r| r.0.as_str()), Some("2026-08-31"));
    assert_eq!(rows.last().map(|r| r.0.as_str()), Some("2026-10-09"));
    assert!(rows.iter().all(|r| r.1 == "close" && r.2 == owner.user_id));
    // The last day holds the numbers of E2E-19: 65,000 = 65,000, cash 50,000 (debit), no entry
    // that day.
    let last = &rows[rows.len() - 1].3;
    assert!(
        last.contains("\"trial_dr_paisa\":6500000")
            && last.contains("\"trial_cr_paisa\":6500000")
            && last.contains("\"cash_in_hand_paisa\":5000000")
            && last.contains("\"cash_in_hand_side\":\"dr\"")
            && last.contains("\"entries\":0"),
        "{last}"
    );
    // The audit row has the range.
    let (old, new): (String, String) = raw(&dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE action = 'day_close'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(
        old.contains("null")
            && new.contains("2026-10-09")
            && new.contains("\"days\":40")
            && new.contains("\"entries\":3"),
        "{old} {new}"
    );
    assert_chain_ok(&dir);
}

#[test]
fn closing_three_days_in_one_call_writes_three_rows_each_with_its_own_numbers() {
    // Lead decision 12-10-2026: one day_closes row for each day closed, with that day's trial
    // totals, cash and entries, all in the one transaction of the call.
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    let rahim = seller(&env, &owner);
    set_today(&env, 2026, 9, 1);
    give(&env, &owner, rahim, "01-09-2026", "15,000").unwrap();
    set_today(&env, 2026, 9, 3);
    close(&env, &owner, "02-09-2026").unwrap();
    let rows: Vec<(String, String)> = raw(&dir)
        .prepare(
            "SELECT entry_date, summary_json FROM day_closes WHERE action = 'close' ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        rows.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        ["2026-08-31", "2026-09-01", "2026-09-02"]
    );
    let numbers = |json: &str| -> serde_json::Value { serde_json::from_str(json).unwrap() };
    // 31-08: the two openings. Cash 65,000 debit, capital 65,000 credit: 65,000 = 65,000.
    let first = numbers(&rows[0].1);
    assert_eq!(
        (
            first["trial_dr_paisa"].as_i64(),
            first["trial_cr_paisa"].as_i64(),
            first["cash_in_hand_paisa"].as_i64(),
            first["entries"].as_i64()
        ),
        (Some(6_500_000), Some(6_500_000), Some(6_500_000), Some(2))
    );
    // 01-09: the advance moves 15,000 from cash to رحیم; the trial is still 65,000 = 65,000.
    let second = numbers(&rows[1].1);
    assert_eq!(
        (
            second["trial_dr_paisa"].as_i64(),
            second["trial_cr_paisa"].as_i64(),
            second["cash_in_hand_paisa"].as_i64(),
            second["entries"].as_i64()
        ),
        (Some(6_500_000), Some(6_500_000), Some(5_000_000), Some(1))
    );
    // 02-09: nothing posted; the numbers repeat.
    let third = numbers(&rows[2].1);
    assert_eq!(
        (
            third["trial_dr_paisa"].as_i64(),
            third["cash_in_hand_paisa"].as_i64(),
            third["entries"].as_i64()
        ),
        (Some(6_500_000), Some(5_000_000), Some(0))
    );
    assert_eq!(
        day::status(&env.ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-09-02")
    );
    // The days add up to the one-step numbers of the preview.
    let preview_total: i64 = rows
        .iter()
        .map(|r| numbers(&r.1)["entries"].as_i64().unwrap())
        .sum();
    assert_eq!(preview_total, 3);
}

#[test]
fn rows_stay_exact_when_cash_passes_through_zero_and_flips_side() {
    // Money review 12-10-2026: the running sums must follow an account that reaches zero, changes
    // side, or has no lines before the range. Second close, so the start is not empty.
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    let rahim = seller(&env, &owner);
    set_today(&env, 2026, 9, 1);
    close(&env, &owner, "31-08-2026").unwrap();
    set_today(&env, 2026, 9, 3);
    // 01-09: all the cash goes to رحیم, so the cash is exactly 0 at the end of the day.
    give(&env, &owner, rahim, "01-09-2026", "65,000").unwrap();
    // 02-09: five thousand more, so the cash is 5,000 on the credit side.
    give(&env, &owner, rahim, "02-09-2026", "5,000").unwrap();
    close(&env, &owner, "03-09-2026").unwrap();
    let rows: Vec<(String, String)> = raw(&dir)
        .prepare(
            "SELECT entry_date, summary_json FROM day_closes WHERE action = 'close' ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        rows.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        ["2026-08-31", "2026-09-01", "2026-09-02", "2026-09-03"]
    );
    let figures = |json: &str| {
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        (
            v["trial_dr_paisa"].as_i64().unwrap(),
            v["trial_cr_paisa"].as_i64().unwrap(),
            v["cash_in_hand_paisa"].as_i64().unwrap(),
            v["cash_in_hand_side"].as_str().unwrap().to_owned(),
            v["entries"].as_i64().unwrap(),
        )
    };
    // 01-09: رحیم 65,000 debit, capital 65,000 credit; cash zero.
    assert_eq!(
        figures(&rows[1].1),
        (6_500_000, 6_500_000, 0, "zero".to_owned(), 1)
    );
    // 02-09: debit rahim 70,000; credit capital 65,000 + cash 5,000.
    assert_eq!(
        figures(&rows[2].1),
        (7_000_000, 7_000_000, 500_000, "cr".to_owned(), 1)
    );
    // 03-09: nothing posted; the numbers repeat.
    assert_eq!(
        figures(&rows[3].1),
        (7_000_000, 7_000_000, 500_000, "cr".to_owned(), 0)
    );
}

#[test]
fn a_day_that_does_not_balance_on_its_own_stops_the_whole_close() {
    // Damage outside the app that cancels out by the last day is still found: every day is
    // checked, and nothing is written (one transaction).
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    let rahim = seller(&env, &owner);
    set_today(&env, 2026, 9, 2);
    give(&env, &owner, rahim, "01-09-2026", "15,000").unwrap();
    let conn = raw(&dir);
    conn.execute_batch("DROP TRIGGER journal_lines_no_update;")
        .unwrap();
    // Out by 100 paisa on 31-08 (cash debited 65,000.00 + 1 rupee), mended on 01-09 (the cash
    // credit of the advance is 1 rupee more). By the last day the books balance again, so only
    // a check of each day finds the damage.
    conn.execute(
        "UPDATE journal_lines SET dr_paisa = dr_paisa + 100 WHERE dr_paisa = 6500000 AND account_id = (SELECT id FROM accounts WHERE code = '01010001')",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE journal_lines SET cr_paisa = cr_paisa + 100 WHERE cr_paisa = 1500000 AND account_id = (SELECT id FROM accounts WHERE code = '01010001')",
        [],
    )
    .unwrap();
    let preview_last = preview(&env, "02-09-2026");
    assert!(preview_last.is_ok(), "the last day alone looks fine");
    assert_eq!(
        preview_last.unwrap().trial_difference.side,
        BalanceSide::Zero
    );
    set_today(&env, 2026, 9, 3);
    let (code, field) = error_of(close(&env, &owner, "02-09-2026").unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrTrialNotBalanced, Some("up_to"))
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM day_closes"), 0);
    assert_eq!(day::status(&env.ctx).unwrap().closed_up_to, None);
}

#[test]
fn the_preview_counts_only_up_to_the_chosen_day() {
    let (_dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    let rahim = seller(&env, &owner);
    set_today(&env, 2026, 9, 5);
    give(&env, &owner, rahim, "05-09-2026", "15,000").unwrap();
    let before = preview(&env, "04-09-2026").unwrap();
    assert_eq!(
        (before.cash_in_hand.amount_paisa, before.entries),
        (rupees(65_000), 2)
    );
    let after = preview(&env, "05-09-2026").unwrap();
    assert_eq!(
        (after.cash_in_hand.amount_paisa, after.entries),
        (rupees(50_000), 3)
    );
    assert_eq!(after.trial_dr_paisa, rupees(65_000));
}

#[test]
fn a_trial_balance_that_does_not_balance_stops_the_close() {
    // NFR-REL-8. The books cannot be out through the app; simulate damage outside it.
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    set_today(&env, 2026, 9, 3);
    let conn = raw(&dir);
    conn.execute_batch("DROP TRIGGER journal_lines_no_update;")
        .unwrap();
    conn.execute("UPDATE journal_lines SET dr_paisa = dr_paisa + 100 WHERE dr_paisa > 0 AND id = (SELECT MIN(id) FROM journal_lines WHERE dr_paisa > 0)", []).unwrap();
    // The preview says so with the numbers; the close refuses and moves nothing.
    let p = preview(&env, "02-09-2026").unwrap();
    assert_ne!(p.trial_dr_paisa, p.trial_cr_paisa);
    assert_ne!(p.trial_difference.side, BalanceSide::Zero);
    assert_eq!(p.trial_difference.amount_paisa, rupees(1));
    let (code, field) = error_of(close(&env, &owner, "02-09-2026").unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrTrialNotBalanced, Some("up_to"))
    );
    assert_eq!(day::status(&env.ctx).unwrap().closed_up_to, None);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM day_closes"), 0);
}

#[test]
fn a_draft_entry_found_at_rest_stops_the_close() {
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    raw(&dir)
        .execute("INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id) VALUES ('2026-09-01', 'general', 'voucher', 1)", [])
        .unwrap();
    assert!(matches!(
        preview(&env, "").unwrap_err(),
        ServiceError::Data(_)
    ));
    assert!(matches!(
        close(&env, &owner, "").unwrap_err(),
        ServiceError::Data(_)
    ));
}

#[test]
fn reopening_needs_a_reason_and_opens_that_day_and_every_later_day() {
    // PRD FR-21.2: reopening a date reopens it and every later date.
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    let rahim = seller(&env, &owner);
    set_today(&env, 2026, 10, 10);
    close(&env, &owner, "09-10-2026").unwrap();
    // A voucher on a closed day is refused.
    assert_eq!(
        give(&env, &owner, rahim, "05-10-2026", "100")
            .unwrap_err()
            .code(),
        ErrorCode::ErrDateClosed
    );
    // The reason is required.
    for reason in ["", "   "] {
        let (code, field) = error_of(reopen(&env, &owner, "05-10-2026", reason).unwrap_err());
        assert_eq!(
            (code, field.as_deref()),
            (ErrorCode::ErrReasonRequired, Some("reason"))
        );
    }
    assert_eq!(
        day::status(&env.ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-10-09"),
        "nothing moved"
    );
    // A day that is not closed has nothing to reopen.
    let (code, field) = error_of(reopen(&env, &owner, "10-10-2026", "x").unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrDayNotClosed, Some("date"))
    );
    assert_eq!(
        error_of(reopen(&env, &owner, "31-02-2026", "x").unwrap_err()).0,
        ErrorCode::ErrDateInvalid
    );
    // The real reopen.
    let status = reopen(&env, &owner, "05-10-2026", "قیمت میں غلطی").unwrap();
    assert_eq!(status.closed_up_to.as_deref(), Some("2026-10-04"));
    give(&env, &owner, rahim, "05-10-2026", "100").unwrap();
    give(&env, &owner, rahim, "09-10-2026", "100").unwrap();
    assert_eq!(
        give(&env, &owner, rahim, "04-10-2026", "100")
            .unwrap_err()
            .code(),
        ErrorCode::ErrDateClosed
    );
    // History and audit row, with the reason.
    let (date, action, reason): (String, String, Option<String>) = raw(&dir)
        .query_row(
            "SELECT entry_date, action, reason FROM day_closes WHERE action = 'reopen'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        (date.as_str(), action.as_str(), reason.as_deref()),
        ("2026-10-05", "reopen", Some("قیمت میں غلطی"))
    );
    let (audit_reason, old, new): (String, String, String) = raw(&dir)
        .query_row(
            "SELECT reason, old_json, new_json FROM audit_log WHERE action = 'day_reopen'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(audit_reason, "قیمت میں غلطی");
    assert!(
        old.contains("2026-10-09") && new.contains("2026-10-04"),
        "{old} {new}"
    );
    assert_chain_ok(&dir);
    // Close again, in order.
    close(&env, &owner, "09-10-2026").unwrap();
    assert_eq!(
        day::status(&env.ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-10-09")
    );
}

#[test]
fn a_month_that_was_month_closed_cannot_be_reopened() {
    // PRD FR-21.2: until its month close is reversed.
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    set_today(&env, 2026, 10, 10);
    close(&env, &owner, "09-10-2026").unwrap();
    let entry: i64 = count(&dir, "SELECT MIN(id) FROM journal_entries");
    let conn = raw(&dir);
    conn.execute(
        "INSERT INTO month_closes (month, journal_entry_id, is_active) VALUES ('2026-09', ?1, 1)",
        [entry],
    )
    .unwrap();
    // September is month-closed: reopening a date in September, or in October (a later month
    // reopens only if no earlier closed month is covered) — October alone is allowed.
    let (code, field) = error_of(reopen(&env, &owner, "15-09-2026", "x").unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrMonthClosed, Some("date"))
    );
    assert_eq!(
        day::status(&env.ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-10-09")
    );
    reopen(&env, &owner, "05-10-2026", "x").unwrap();
    // A month close that was reversed no longer blocks.
    conn.execute("UPDATE month_closes SET is_active = 0", [])
        .unwrap();
    reopen(&env, &owner, "15-09-2026", "x").unwrap();
    assert_eq!(
        day::status(&env.ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-09-14")
    );
    // An active close of October blocks reopening any date in October or earlier.
    conn.execute(
        "UPDATE month_closes SET month = '2026-10', is_active = 1",
        [],
    )
    .unwrap();
    close(&env, &owner, "09-10-2026").unwrap();
    assert_eq!(
        error_of(reopen(&env, &owner, "05-10-2026", "x").unwrap_err()).0,
        ErrorCode::ErrMonthClosed
    );
    assert_eq!(
        error_of(reopen(&env, &owner, "15-09-2026", "x").unwrap_err()).0,
        ErrorCode::ErrMonthClosed
    );
}

#[test]
fn random_close_and_reopen_steps_keep_the_window_right() {
    // Property: the closed-up-to date only moves forward on a close, becomes date - 1 on a
    // reopen, is never after today, and a posting is accepted exactly when
    // closed-up-to < date <= today.
    let (_dir, env, owner) = ready();
    open_books(&env, &owner, "31-08-2026", (2026, 8, 31));
    let rahim = seller(&env, &owner);
    set_today(&env, 2026, 9, 28);
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let date_of = |d: u64| format!("{:02}-09-2026", 1 + d % 28);
    let mut closed: Option<u64> = None; // day of September, or none
    for _ in 0..120 {
        let d = next() % 28;
        match next() % 3 {
            0 => {
                let result = close(&env, &owner, &date_of(d));
                let day_number = d + 1;
                if closed.is_none_or(|c| day_number > c) {
                    result.unwrap();
                    closed = Some(day_number);
                } else {
                    assert_eq!(error_of(result.unwrap_err()).0, ErrorCode::ErrDateClosed);
                }
            }
            1 => {
                let result = reopen(&env, &owner, &date_of(d), "random");
                let day_number = d + 1;
                if closed.is_some_and(|c| day_number <= c) {
                    result.unwrap();
                    closed = Some(day_number - 1).filter(|c| *c > 0);
                    if day_number == 1 {
                        // The day before 01-09 is 31-08, which is closed up to that day.
                        closed = None;
                    }
                } else {
                    assert_eq!(error_of(result.unwrap_err()).0, ErrorCode::ErrDayNotClosed);
                }
            }
            _ => {
                let result = give(&env, &owner, rahim, &date_of(d), "10");
                let day_number = d + 1;
                let open = closed.is_none_or(|c| day_number > c);
                assert_eq!(
                    result.is_ok(),
                    open,
                    "posting on day {day_number} with closed {closed:?}"
                );
            }
        }
        let status = day::status(&env.ctx).unwrap();
        match (status.closed_up_to.as_deref(), closed) {
            (Some(text), Some(c)) => assert_eq!(text, format!("2026-09-{c:02}")),
            (Some(text), None) => assert_eq!(text, "2026-08-31"),
            (None, None) => {}
            (None, Some(c)) => panic!("model says closed up to {c}, database says none"),
        }
        assert!(
            status
                .closed_up_to
                .as_deref()
                .is_none_or(|c| c <= status.today.as_str())
        );
    }
}

#[test]
fn ten_years_closed_in_one_step_write_a_row_for_each_day_and_stay_fast() {
    // The first close of an old file may cover years: one pass over the journal, one row for
    // each day, one transaction.
    let (dir, env, owner) = ready();
    open_books(&env, &owner, "01-01-2016", (2016, 1, 1));
    set_today(&env, 2026, 1, 1);
    let started = std::time::Instant::now();
    close(&env, &owner, "31-12-2025").unwrap();
    let took = started.elapsed();
    // 01-01-2016 to 31-12-2025 is 3,653 days (three leap years, 2016, 2020 and 2024, more than a plain ten).
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM day_closes"), 3_653);
    assert_eq!(
        day::status(&env.ctx).unwrap().closed_up_to.as_deref(),
        Some("2025-12-31")
    );
    eprintln!("3,653 days closed in {took:?} (debug build)");
    assert!(took.as_millis() < 5_000, "{took:?}");
}
