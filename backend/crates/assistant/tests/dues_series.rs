//! K8 (live run of 05-11-2026, the lead's task): the series of what the buyers owe
//! (`balance_series(BuyersDues)`, PRD FR-7.86 v1.27) was read in three passes over 3.5 million
//! lines and took 8 s. It is read in one pass over the months of the series and one over the
//! totals now, and a balance over the largest allowed amount is refused before the series is read.
//! The numbers must not change by a rupee: these tests hold the cases the pass could get wrong
//! (a balance before the series that includes an opening dated before the first month of trade,
//! a normal buyer account with a person key, the limit exactly at the largest amount, and one rupee
//! over it), and the four cases of the money review of the finished work (a buyer moved to the dead
//! amount before the series and on a day inside it, the lines of this month, and trade older than
//! the window). `forecast_gaps.rs` holds the rest (openings, the dead amount, "make regular", the
//! months as differences of R20). While this was built the old reading was run next to the new
//! one, in these tests, in `forecast_gaps.rs`, on the rich year of the oracle and on the 3.5
//! million lines of the speed volume, with no difference.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use mandi_assistant::results::Words;
use mandi_assistant::tools::{Env, State, Stop, call};
use mandi_reports::analytics::{BalanceSubject, balance_series};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{BikriSaveInput, LineChargeMethod, PaymentType, PukhtaInput};
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::dead::{DeadMarkInput, DeadReason};
use mandi_services::core::vouchers::{ReceiptInput, RemovedPaymentFields};
use mandi_services::domain::figures::question_numbers;
use mandi_services::{ServiceError, accounts, bikri, dead, vouchers};
use serde_json::json;

fn today() -> BusinessDate {
    BusinessDate::parse_iso("2026-10-10").unwrap()
}

/// An opening of `account_id` on `date`: `dr` or `cr` in rupees (the other empty).
fn open(w: &world::World, account_id: i64, date: &str, dr: &str, cr: &str) {
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id,
                version: accounts::get(&w.env.ctx, account_id).unwrap().version,
                opening_date: date.into(),
                opening_dr: dr.into(),
                opening_cr: cr.into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
}

/// The dues series of the world as (year, month, change in rupees), and the dues now in rupees.
fn series(w: &world::World) -> (i64, Vec<(i32, u32, i64)>) {
    let s = balance_series(w.env.ctx.db(), BalanceSubject::BuyersDues, today()).unwrap();
    (
        s.now.rupees(),
        s.changes
            .iter()
            .map(|m| (m.year, m.month, m.amount.rupees()))
            .collect(),
    )
}

/// The estimate tool for the buyers' dues, as the model calls it.
fn estimate(w: &world::World) -> Result<(), mandi_assistant::tools::ToolError> {
    let words = Words::new().unwrap();
    let question = "gahakon ke zimme agle mahino mein";
    let numbers = question_numbers(question);
    let env = Env {
        ctx: &w.env.ctx,
        actor: &w.owner,
        words: &words,
        language: Language::Ur,
        today: today(),
        partner_locked: false,
        numbers: &numbers,
        question,
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    let input = json!({
        "measure": "buyers_dues", "item": null, "seller": null, "buyer": null,
        "garden": null, "trip": null, "head": null, "partner": null, "party": null,
        "ahead": null, "until": null, "what_if": null, "percent": null, "target": null
    });
    match call(&env, &State::default(), "estimate", &input.to_string()) {
        Ok(_) => Ok(()),
        Err(Stop::Error(e)) => Err(e),
        Err(Stop::Locked) => panic!("locked"),
    }
}

#[test]
fn an_opening_dated_before_the_first_month_of_trade_is_in_the_balance_before_the_series() {
    // Akram buys in July, August and September only (11,400; 22,000; 32,600 with the 6% buyer
    // commission and لاگا 8 a نگ). Nothing is older than the window, so the series starts in July,
    // the first month with an entry of trade. His opening CREDIT of 5,000 dated 15-05 is not trade
    // (it is not a month of the series) but it is in his balance before July: -5,000. July's bill
    // takes him to 6,400, so what he owes grows by 6,400 and not by 11,400.
    let w = worked::months_world(&[
        (7, true, 100, 100),
        (8, true, 100, 200),
        (9, true, 100, 300),
    ]);
    assert_eq!(
        series(&w),
        (
            66_000,
            vec![(2026, 7, 11_400), (2026, 8, 22_000), (2026, 9, 32_600)]
        )
    );
    open(&w, w.akram, "15-05-2026", "", "5,000");
    assert_eq!(
        series(&w),
        (
            61_000,
            vec![(2026, 7, 6_400), (2026, 8, 22_000), (2026, 9, 32_600)]
        )
    );
    // The changes add up to the dues now less the 5,000 credit he began with: 61,000 = 6,400 +
    // 22,000 + 32,600.
    assert_eq!(6_400 + 22_000 + 32_600, 61_000);
}

#[test]
fn nothing_to_read_gives_an_empty_series() {
    // No trade at all: no month, and the estimate is too short, not an error.
    let w = worked::months_world(&[]);
    assert_eq!(series(&w), (0, Vec::new()));
    assert!(estimate(&w).is_ok());
}

#[test]
fn a_balance_over_the_largest_amount_is_refused_before_the_series_and_the_largest_is_not() {
    // The largest allowed amount is 10,00,00,00,000 rupees. Akram owes exactly that: the estimate
    // goes on (and is too short, with no months). One rupee more, from Arshad: refused at once with
    // ERR_OVERFLOW and no field, the very refusal of the domain, and the series is not read.
    let w = worked::months_world(&[]);
    open(&w, w.akram, "15-05-2026", "10,000,000,000", "");
    assert_eq!(series(&w).0, 10_000_000_000);
    assert!(estimate(&w).is_ok());
    open(&w, w.arshad, "15-05-2026", "1", "");
    let (now, changes) = series(&w);
    assert_eq!(now, 10_000_000_001);
    assert!(changes.is_empty(), "the series is not read: {changes:?}");
    let refused = estimate(&w).unwrap_err();
    assert_eq!(refused.code, "ERR_OVERFLOW");
    assert_eq!(refused.field, None);
}

#[test]
fn a_person_key_on_a_normal_buyer_account_is_refused_not_folded_in() {
    // Only the shared پختہ کھاتہ account holds many people (PRD FR-13.5), so a normal buyer account
    // is one owner. The database does not enforce it; the series checks it and stops with an error
    // rather than group the balance differently from R20.
    let w = worked::months_world(&[(8, true, 100, 100), (9, true, 100, 200)]);
    assert_eq!(series(&w).1.len(), 2);
    let sql = format!(
        "DROP TRIGGER journal_lines_no_update;
         UPDATE journal_lines SET pukhta_person_key = 'x' WHERE account_id = {};",
        w.akram
    );
    w.env
        .ctx
        .db()
        .write(|tx| {
            tx.execute_batch(&sql).map_err(ServiceError::from)?;
            Ok::<(), ServiceError>(())
        })
        .unwrap();
    let refused = balance_series(w.env.ctx.db(), BalanceSubject::BuyersDues, today());
    assert!(format!("{refused:?}").contains("person key"), "{refused:?}");
}

/// A one-time buyer (a person of the shared پختہ کھاتہ account) buys `qty` of Rahim's banana at
/// `rate` on credit, with no charges, on `date`.
fn one_time_buys(w: &world::World, person: (&str, &str, &str), qty: &str, rate: &str, date: &str) {
    let mut line = world::sale("s-1", w.pukhta_buyer, qty, rate);
    line.pukhta = Some(PukhtaInput {
        name_ur: person.0.into(),
        name_en: person.1.into(),
        phone: person.2.into(),
    });
    line.payment_type = Some(PaymentType::Credit);
    line.buyer_comm = world::typed(LineChargeMethod::Percent, "0");
    line.seller_comm = world::typed(LineChargeMethod::Percent, "0");
    line.laga = world::typed(LineChargeMethod::PerUnit, "0");
    let lot = world::with_sales(
        world::bare_lot(w, "lot-1", w.rahim, w.banana, qty),
        vec![line],
    );
    let mut draft = world::draft(vec![lot]);
    draft.header.entry_date = date.into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

#[test]
fn one_one_time_person_in_the_dead_amount_leaves_the_others_of_the_shared_account_alone() {
    // Shahid and Nasir are two persons of the one shared پختہ کھاتہ account: each is an owner of
    // his own. Akram buys 11,400 in August and 22,000 in September. Shahid buys 2,000 and Nasir
    // 3,000 on credit in August; Shahid is moved to the dead amount on 01-09 and pays 1,000 on
    // 16-09, a recovery after his move, which is a one-off and left out. The dues now (the tile)
    // have Akram's 33,400 and Nasir's 3,000 and no Shahid. August grows by 11,400 + 2,000 + 3,000;
    // September by Akram's 22,000 alone: Nasir does not move, and Shahid's payment does not count.
    let w = worked::months_world(&[(8, true, 100, 100), (9, true, 100, 200)]);
    let shahid = ("شاہد", "Shahid", "0300-0000000");
    let nasir = ("ناصر", "Nasir", "0300-1111111");
    one_time_buys(&w, shahid, "2", "1000", "16-08-2026");
    one_time_buys(&w, nasir, "3", "1000", "17-08-2026");
    let key = mandi_services::core::text::person_key(shahid.0, shahid.2);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.pukhta_buyer,
            pukhta_person_key: Some(key.clone()),
            entry_date: "01-09-2026".into(),
            reason: DeadReason::Left,
            note: String::new(),
        },
    )
    .unwrap();
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "16-09-2026".into(),
            account_id: w.pukhta_buyer,
            person_key: Some(key),
            amount: "1,000".into(),
            removed: RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();
    assert_eq!(
        series(&w),
        (36_400, vec![(2026, 8, 16_400), (2026, 9, 22_000)])
    );
}

/// A receipt from a normal buyer (or from one person of the shared account).
fn receipt(w: &world::World, account_id: i64, date: &str, amount: &str) {
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: date.into(),
            account_id,
            person_key: None,
            amount: amount.into(),
            removed: RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();
}

/// A credit sale of Rahim's banana to a buyer on a day: the buyer pays value + 6% + 8 a نگ.
fn sells_to(w: &world::World, buyer: i64, date: &str, qty: &str, rate: &str) {
    let lot = world::with_sales(
        world::bare_lot(w, "lot-1", w.rahim, w.banana, qty),
        vec![world::sale("s-1", buyer, qty, rate)],
    );
    let mut draft = world::draft(vec![lot]);
    draft.header.entry_date = date.into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

/// Moves a normal buyer to the dead amount on a day.
fn mark_dead(w: &world::World, account_id: i64, date: &str) {
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id,
            pukhta_person_key: None,
            entry_date: date.into(),
            reason: DeadReason::Left,
            note: String::new(),
        },
    )
    .unwrap();
}

#[test]
fn a_buyer_moved_to_the_dead_amount_before_the_series_adds_nothing_to_it() {
    // Money review K8, case 1. Akram buys 11,400 / 22,000 / 32,600 in April, May and June. Arshad
    // has an opening of 5,000 on 15-02, is moved to the dead amount on 01-03 (before the series,
    // which starts in April) and pays 2,000 on 20-05: a recovery after his move. It is a one-off
    // and not a pace, so May stays 22,000 (it would be 20,000 if the payment counted). The dues
    // now are Akram's 66,000: Arshad is out of them.
    let w = worked::months_world(&[
        (4, true, 100, 100),
        (5, true, 100, 200),
        (6, true, 100, 300),
    ]);
    open(&w, w.arshad, "15-02-2026", "5,000", "");
    mark_dead(&w, w.arshad, "01-03-2026");
    receipt(&w, w.arshad, "20-05-2026", "2,000");
    let months = [(2026, 4, 11_400), (2026, 5, 22_000), (2026, 6, 32_600)];
    let mut want: Vec<(i32, u32, i64)> = months.to_vec();
    want.extend([(2026, 7, 0), (2026, 8, 0), (2026, 9, 0)]);
    assert_eq!(series(&w), (66_000, want));
}

#[test]
fn a_buyer_moved_to_the_dead_amount_on_a_day_loses_the_lines_of_that_day_and_after() {
    // Money review K8, case 2. Arshad has an opening of 5,000 and pays 1,000 on 31-03: an entry of
    // trade, so the series starts in March, and what the buyers owe goes down by his 1,000. He is
    // moved to the dead amount on 01-04 and pays 2,000 that very day: a line on the day of his move
    // is after it, so it is left out (April would be 9,400 if it counted). Akram buys 11,400 in
    // April. The dues now are Akram's 11,400: Arshad is out of them.
    let w = worked::months_world(&[(4, true, 100, 100)]);
    open(&w, w.arshad, "15-02-2026", "5,000", "");
    receipt(&w, w.arshad, "31-03-2026", "1,000");
    mark_dead(&w, w.arshad, "01-04-2026");
    receipt(&w, w.arshad, "01-04-2026", "2,000");
    let mut want = vec![(2026, 3, -1_000), (2026, 4, 11_400)];
    want.extend((5..=9).map(|m| (2026, m, 0)));
    assert_eq!(series(&w), (11_400, want));
}

#[test]
fn the_lines_of_this_month_move_the_dues_now_and_never_a_month_of_the_series() {
    // Money review K8, case 3. Akram has an opening CREDIT of 5,000 on 15-02 and buys 11,400 in
    // April, so April grows by 6,400 (the credit is used up first). On 05-10 (this month, today is
    // 10-10) he buys 1,140 more and Arshad has an opening of 3,000 dated 03-10. Neither is in a
    // month of the series, and neither is in the balance before it: April is still 6,400 (it
    // would be 7,540 if the October sale were left in the balance before). The dues now have all:
    // -5,000 + 11,400 + 1,140 + 3,000.
    let w = worked::months_world(&[(4, true, 100, 100)]);
    open(&w, w.akram, "15-02-2026", "", "5,000");
    sells_to(&w, w.akram, "05-10-2026", "10", "100");
    open(&w, w.arshad, "03-10-2026", "3,000", "");
    let mut want = vec![(2026, 4, 6_400)];
    want.extend((5..=9).map(|m| (2026, m, 0)));
    assert_eq!(series(&w), (10_540, want));
}

#[test]
fn trade_older_than_the_window_gives_twelve_months_and_a_debit_clipped_at_zero() {
    // Money review K8, case 4. Akram paid 3,000 in advance on 15-09-2025, before the window of 12
    // months (October 2025 to September 2026): that is older trade, so the series starts in October
    // 2025 with six quiet months. His balance before October is -3,000. April's bill of 11,400
    // takes him to 8,400, so what he owes grows by 8,400 and not by 11,400; May and June grow by
    // 22,000 and 32,600. The dues now: -3,000 + 66,000.
    let w = worked::months_world(&[
        (4, true, 100, 100),
        (5, true, 100, 200),
        (6, true, 100, 300),
    ]);
    receipt(&w, w.akram, "15-09-2025", "3,000");
    let mut want: Vec<(i32, u32, i64)> = vec![(2025, 10, 0), (2025, 11, 0), (2025, 12, 0)];
    want.extend((1..=3).map(|m| (2026, m, 0)));
    want.extend([(2026, 4, 8_400), (2026, 5, 22_000), (2026, 6, 32_600)]);
    want.extend((7..=9).map(|m| (2026, m, 0)));
    assert_eq!(series(&w), (63_000, want));
}
