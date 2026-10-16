//! The day changes at midnight on the shop's clock, not at midnight UTC (PRD FR-6.6, NFR-REL-11,
//! Code Standards G9). Owner: `robustness`.
//!
//! "Today" decides the open-day window: a posting may be dated after the closed-up-to date and
//! no later than today. In the tests the shop clock is Pakistan time, UTC+5. The edge cases are
//! the two seconds around midnight there, the same two seconds in UTC (which must change
//! nothing), the end of a month, and the leap day.
#![cfg(test)]

mod common;

use chrono::{TimeZone, Utc};
use common::*;
use mandi_services::core::accounts::{AccountInput, AccountType};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::{AdvanceInput, VoucherMode};
use mandi_services::{accounts, auth, vouchers};

fn seller() -> AccountInput {
    AccountInput {
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
    }
}

fn advance(to: i64, date: &str) -> AdvanceInput {
    AdvanceInput {
        date: date.into(),
        seller_account_id: to,
        amount: "100".into(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        expected_goods: String::new(),
        narration: String::new(),
    }
}

#[test]
fn the_day_changes_at_midnight_on_the_shop_clock_and_not_at_midnight_utc() {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = accounts::save(&env.ctx, &owner, &seller()).unwrap().id;
    let cases = [
        // (UTC time, the dates that must work, the dates that must be refused as after today)
        // 18:59:59 UTC is 23:59:59 in Pakistan: still 10-10.
        (
            Utc.with_ymd_and_hms(2026, 10, 10, 18, 59, 59).unwrap(),
            "10-10-2026",
            "11-10-2026",
        ),
        // 19:00:00 UTC is 00:00:00 on 11-10 in Pakistan.
        (
            Utc.with_ymd_and_hms(2026, 10, 10, 19, 0, 0).unwrap(),
            "11-10-2026",
            "12-10-2026",
        ),
        // Midnight UTC is 05:00 in Pakistan: the same day as the 19:00 UTC before it.
        (
            Utc.with_ymd_and_hms(2026, 10, 11, 0, 0, 0).unwrap(),
            "11-10-2026",
            "12-10-2026",
        ),
        // The turn of a month, a year and a leap day.
        (
            Utc.with_ymd_and_hms(2026, 10, 31, 19, 0, 0).unwrap(),
            "01-11-2026",
            "02-11-2026",
        ),
        (
            Utc.with_ymd_and_hms(2026, 12, 31, 19, 0, 0).unwrap(),
            "01-01-2027",
            "02-01-2027",
        ),
        (
            Utc.with_ymd_and_hms(2028, 2, 28, 19, 0, 0).unwrap(),
            "29-02-2028",
            "01-03-2028",
        ),
        (
            Utc.with_ymd_and_hms(2028, 2, 29, 19, 0, 0).unwrap(),
            "01-03-2028",
            "02-03-2028",
        ),
    ];
    // The clock only moves forward in this loop, so no earlier day is ever closed on the way.
    for (now, ok_date, future_date) in cases {
        env.clock.set(now);
        let refused =
            vouchers::save_advance(&env.ctx, &owner, &advance(rahim, future_date)).unwrap_err();
        assert!(
            refused.has_code(ErrorCode::ErrDateInFuture)
                || refused.has_code(ErrorCode::ErrDateClosed),
            "{now}: {future_date} must be after today: {refused:?}"
        );
        let before: i64 = raw(&dir)
            .query_row("SELECT COUNT(*) FROM vouchers", [], |r| r.get(0))
            .unwrap();
        vouchers::save_advance(&env.ctx, &owner, &advance(rahim, ok_date))
            .unwrap_or_else(|e| panic!("{now}: {ok_date} must be today or earlier: {e:?}"));
        let after: i64 = raw(&dir)
            .query_row("SELECT COUNT(*) FROM vouchers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(after, before + 1, "{now}: {ok_date}");
    }
    assert_chain_ok(&dir);
}
