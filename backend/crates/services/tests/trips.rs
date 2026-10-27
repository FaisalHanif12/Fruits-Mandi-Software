//! Market trips (PRD 12.10, plan 16b T2a) through the real services on the PRD worked day's world:
//! partners الف / ب / ج at 40 / 30 / 30, cash in hand 50,000, today 10-10-2026. The expected numbers
//! are the PRD's (E2E-34, E2E-34b, E2E-41, C21, C23 to C25) and are never changed to suit the code.
//! The own lot in بکری (E2E-34's sale to اکرم) is tested with the golden day.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::partners::PartnerUpdateInput;
use mandi_services::core::purchases::{
    PurchaseCancelInput, PurchaseGetInput, PurchaseInput, PurchaseRow, PurchaseStateFilter,
    PurchaseWriteOffInput, PurchaseWriteOffUndoInput, PurchasesListInput,
};
use mandi_services::core::trips::{
    OtherSaleInput, TripCostInput, TripEntryCancelInput, TripGetInput, TripInput, TripRow,
    TripSetPartnersInput, TripsListInput,
};
use mandi_services::core::ventures::{
    EntryPartInput, MoneySource, VenturePartnerInput, VentureStatus,
};
use mandi_services::{accounts, cash, ledger, partners, purchases, trips};
use support::{World, world, world_with};

fn shop(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

fn pocket(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Pocket,
    }
}

fn trip_input(date: &str, partners: Vec<VenturePartnerInput>) -> TripInput {
    TripInput {
        date: date.into(),
        market: Some("Lahore".into()),
        sent_person: None,
        note: None,
        partners,
    }
}

/// A trip of the three partners at 40 / 30 / 30, all paying from the shop's cash.
fn trip_of_three(w: &World) -> TripRow {
    let [a, b, c] = w.partners;
    trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input(
            "10-10-2026",
            vec![shop(a, "40"), shop(b, "30"), shop(c, "30")],
        ),
    )
    .unwrap()
}

fn purchase_input(
    w: &World,
    trip: i64,
    qty: &str,
    rate: &str,
    freight: Option<&str>,
) -> PurchaseInput {
    PurchaseInput {
        trip_id: Some(trip),
        date: "10-10-2026".into(),
        supplier: Some("Orange Traders".into()),
        item_id: Some(w.banana),
        packing_id: Some(w.box_),
        qty: qty.into(),
        weight: None,
        rate: rate.into(),
        freight: freight.map(str::to_owned),
        labour: None,
        other_cost: None,
        note: None,
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
}

fn cash_now(w: &World) -> i64 {
    w.env.ctx.db().read(cash::cash_in_hand).unwrap().rupees()
}

/// Σ credit - Σ debit of an account, in rupees.
fn credit_balance(w: &World, account_id: i64) -> i64 {
    let conn = common::raw(&w.dir);
    let (dr, cr): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0) FROM journal_lines WHERE account_id = ?1",
            [account_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    (cr - dr) / 100
}

fn venture_account(w: &World, partner_id: i64) -> i64 {
    partners::get(&w.env.ctx, partner_id)
        .unwrap()
        .row
        .venture_account_id
}

/// The lines of a journal entry as `(account code, debit, credit)` in rupees.
fn lines(w: &World, entry_id: i64) -> Vec<(String, i64, i64)> {
    let conn = common::raw(&w.dir);
    conn.prepare(
        "SELECT a.code, l.dr_paisa / 100, l.cr_paisa / 100 FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id WHERE l.entry_id = ?1 ORDER BY l.line_no",
    )
    .unwrap()
    .query_map([entry_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
    .unwrap()
    .map(Result::unwrap)
    .collect()
}

fn entry_count(w: &World) -> i64 {
    common::raw(&w.dir)
        .query_row("SELECT COUNT(*) FROM journal_entries", [], |r| r.get(0))
        .unwrap()
}

fn balanced(w: &World) {
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

fn amounts(parts: &[mandi_services::core::ventures::EntryPart]) -> Vec<i64> {
    parts.iter().map(|p| p.amount_paisa.rupees()).collect()
}

fn first_error(error: &mandi_services::ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        mandi_services::ServiceError::Rejected(items) => {
            let first = items.first().unwrap();
            (first.code, first.field.clone())
        }
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn a_new_trip_starts_from_the_firms_shares_then_from_the_last_trip() {
    let w = world();
    let first = trips::defaults(&w.env.ctx).unwrap();
    assert!(!first.from_last_trip);
    let bps: Vec<i32> = first.partners.iter().map(|p| p.share_bps).collect();
    assert_eq!(bps, vec![4_000, 3_000, 3_000]);
    assert!(first.partners.iter().all(|p| p.source == MoneySource::Shop));

    let [a, b, _] = w.partners;
    trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input("10-10-2026", vec![shop(a, "60"), pocket(b, "40")]),
    )
    .unwrap();
    let next = trips::defaults(&w.env.ctx).unwrap();
    assert!(next.from_last_trip);
    let rows: Vec<(i32, MoneySource)> = next
        .partners
        .iter()
        .map(|p| (p.share_bps, p.source))
        .collect();
    assert_eq!(
        rows,
        vec![
            (6_000, MoneySource::Shop),
            (4_000, MoneySource::Pocket),
            (0, MoneySource::Shop)
        ]
    );
}

#[test]
fn shares_must_total_100_and_each_partner_is_listed_once() {
    let w = world();
    let [a, b, c] = w.partners;
    let err = trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input(
            "10-10-2026",
            vec![shop(a, "40"), shop(b, "30"), shop(c, "20")],
        ),
    )
    .unwrap_err();
    assert_eq!(
        first_error(&err),
        (ErrorCode::ErrSharesNot100, Some("partners".into()))
    );
    let err = trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input("10-10-2026", vec![shop(a, "50"), shop(a, "50")]),
    )
    .unwrap_err();
    assert_eq!(
        first_error(&err),
        (
            ErrorCode::ErrPartnerNotListed,
            Some("partners[1].partner_id".into())
        )
    );
    // 33.33 / 33.33 / 33.34 totals exactly 100%.
    trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input(
            "10-10-2026",
            vec![shop(a, "33.33"), shop(b, "33.33"), shop(c, "33.34")],
        ),
    )
    .unwrap();
    // Nothing was made by the refused ones.
    let list = trips::list(&w.env.ctx, &TripsListInput::default()).unwrap();
    assert_eq!(list.total_count, 1);
}

#[test]
fn e2e_34_the_purchase_is_split_and_the_shop_parts_post() {
    // PRD E2E-34: 20 x 500 + 400 freight = 10,400 from the shop's cash at 40 / 30 / 30.
    let w = world();
    let trip = trip_of_three(&w);
    let input = purchase_input(&w, trip.id, "20", "500", Some("400"));
    let preview = purchases::preview(&w.env.ctx, &input).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let numbers = preview.numbers.clone().unwrap();
    assert_eq!(numbers.value.rupees(), 10_000);
    assert_eq!(numbers.paid_now.rupees(), 10_400);
    assert_eq!(numbers.total_cost.rupees(), 10_400);
    assert_eq!(numbers.avg_cost.rupees(), 520);
    assert_eq!(amounts(&numbers.parts.parts), vec![4_160, 3_120, 3_120]);
    assert_eq!(numbers.parts.shop_total_paisa.rupees(), 10_400);
    assert_eq!(numbers.parts.pocket_total_paisa.rupees(), 0);
    assert_eq!(preview.cash.cash_after_paisa.rupees(), 39_600);
    assert_eq!(cash_now(&w), 50_000, "a preview saves nothing");

    let saved = purchases::save(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(
        saved.preview.numbers, preview.numbers,
        "the save works out what the preview showed"
    );
    assert_eq!(saved.number, 1);
    assert_eq!(saved.total_cost.rupees(), 10_400);
    assert_eq!(cash_now(&w), 39_600);
    let entry = saved.entry_id.unwrap();
    let [a, b, c] = w.partners;
    let rows = lines(&w, entry);
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].1, 4_160);
    assert_eq!(rows[1].1, 3_120);
    assert_eq!(rows[2].1, 3_120);
    assert_eq!((rows[3].0.as_str(), rows[3].2), ("01010001", 10_400));
    // Each partner's 0604 account is down by his part: it comes off his total at once.
    assert_eq!(credit_balance(&w, venture_account(&w, a)), -4_160);
    assert_eq!(credit_balance(&w, venture_account(&w, b)), -3_120);
    assert_eq!(credit_balance(&w, venture_account(&w, c)), -3_120);
    balanced(&w);

    let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id: saved.id }).unwrap();
    assert_eq!(row.qty_left, 20);
    assert_eq!(row.cost_left_paisa.rupees(), 10_400);
    assert_eq!(row.paid_now_paisa.rupees(), 10_400);
    assert_eq!(row.trip_cost_paisa.rupees(), 0);
    assert_eq!(amounts(&row.parts), vec![4_160, 3_120, 3_120]);
    assert!(row.can_cancel);
    assert_eq!(row.result_paisa.rupees(), 0);
}

#[test]
fn e2e_41_a_pocket_part_posts_nothing_and_the_truck_is_spread_over_the_purchase() {
    // PRD E2E-41: الف 50% from the shop, ب 50% from his pocket, ج not in the trip.
    let w = world();
    let [a, b, c] = w.partners;
    let trip = trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input(
            "10-10-2026",
            vec![shop(a, "50"), pocket(b, "50"), shop(c, "0")],
        ),
    )
    .unwrap();
    assert_eq!(trip.partners.len(), 2, "ج is not in the trip");
    // 80 x 1,200 = 96,000 gives 48,000 / 48,000.
    let saved = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "80", "1,200", None),
    )
    .unwrap();
    assert_eq!(cash_now(&w), 2_000, "only الف's 48,000 left the cash");
    let rows = lines(&w, saved.entry_id.unwrap());
    assert_eq!(rows.len(), 2);
    assert_eq!(credit_balance(&w, venture_account(&w, a)), -48_000);
    assert_eq!(
        credit_balance(&w, venture_account(&w, b)),
        0,
        "ب paid from his pocket"
    );
    // The truck: 4,000, spread over the one purchase; parts 2,000 / 2,000.
    let cost_input = TripCostInput {
        trip_id: Some(trip.id),
        date: "10-10-2026".into(),
        amount: "4,000".into(),
        reason: "truck".into(),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    };
    let preview = trips::cost_preview(&w.env.ctx, &cost_input).unwrap();
    let numbers = preview.numbers.unwrap();
    assert_eq!(amounts(&numbers.parts.parts), vec![2_000, 2_000]);
    assert_eq!(numbers.spread.len(), 1);
    assert_eq!(numbers.spread[0].amount_paisa.rupees(), 4_000);
    assert_eq!(preview.cash.cash_after_paisa.rupees(), 0);
    assert!(!preview.cash.needs_confirm, "cash 0 is not below 0");
    let cost = trips::cost_save(&w.env.ctx, &w.owner, &cost_input).unwrap();
    assert_eq!(cash_now(&w), 0);
    assert_eq!(cost.number, 1);
    let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id: saved.id }).unwrap();
    assert_eq!(row.total_cost_paisa.rupees(), 100_000);
    assert_eq!(row.trip_cost_paisa.rupees(), 4_000);
    assert_eq!(row.cost_left_paisa.rupees(), 100_000);
    assert_eq!(row.avg_cost_paisa.rupees(), 1_250);
    assert!(!row.can_cancel, "it carries a part of a trip cost");
    assert_eq!(credit_balance(&w, venture_account(&w, a)), -50_000);
    balanced(&w);
    // The trip's money in: الف 50,000 from the shop, ب 50,000 from his pocket.
    let detail = trips::get(
        &w.env.ctx,
        &TripGetInput {
            id: trip.id,
            entries_offset: 0,
            entries_limit: 0,
        },
    )
    .unwrap();
    let mine: Vec<(i64, i64, i64)> = detail
        .partner_results
        .iter()
        .map(|p| {
            (
                p.money_in_shop_paisa.rupees(),
                p.money_in_pocket_paisa.rupees(),
                p.net_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(mine, vec![(50_000, 0, -50_000), (0, 50_000, -50_000)]);
    assert_eq!(detail.entries.len(), 2);
    assert_eq!(detail.entries_count, 2);
}

#[test]
fn a_purchase_from_pockets_only_has_no_journal_entry() {
    let w = world();
    let [a, b, c] = w.partners;
    let trip = trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input(
            "10-10-2026",
            vec![pocket(a, "40"), pocket(b, "30"), pocket(c, "30")],
        ),
    )
    .unwrap();
    let before = entry_count(&w);
    let saved = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    assert_eq!(saved.entry_id, None);
    assert_eq!(entry_count(&w), before);
    assert_eq!(cash_now(&w), 50_000);
    let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id: saved.id }).unwrap();
    assert_eq!(amounts(&row.parts), vec![4_160, 3_120, 3_120]);
    // Cancelling it posts nothing either, and the stock goes with it.
    let cancelled = purchases::cancel(
        &w.env.ctx,
        &w.owner,
        &PurchaseCancelInput {
            id: saved.id,
            version: saved.version,
            reason: "wrong trip".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(cancelled.reversal_entry_id, None);
    assert_eq!(entry_count(&w), before);
    let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id: saved.id }).unwrap();
    assert_eq!(row.qty_left, 0);
    assert!(!row.can_cancel);
}

#[test]
fn the_sources_of_one_entry_can_differ_from_the_trips_defaults() {
    let w = world();
    let trip = trip_of_three(&w);
    let [a, b, c] = w.partners;
    let mut input = purchase_input(&w, trip.id, "20", "500", Some("400"));
    input.parts = vec![
        EntryPartInput {
            partner_id: a,
            source: MoneySource::Shop,
        },
        EntryPartInput {
            partner_id: b,
            source: MoneySource::Pocket,
        },
        EntryPartInput {
            partner_id: c,
            source: MoneySource::Shop,
        },
    ];
    let saved = purchases::save(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(cash_now(&w), 50_000 - 4_160 - 3_120);
    assert_eq!(credit_balance(&w, venture_account(&w, b)), 0);
    // A list that misses a partner of the trip is refused.
    input.parts.pop();
    let preview = purchases::preview(&w.env.ctx, &input).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrPartnerNotListed,
        "parts"
    ));
    assert!(saved.entry_id.is_some());
}

#[test]
fn c24_a_trip_cost_is_spread_by_the_nag_bought() {
    // PRD C24: 1,000 over purchases of 30 and 20 nag is 600 and 400.
    let w = world();
    let trip = trip_of_three(&w);
    let first = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "30", "100", None),
    )
    .unwrap();
    let second = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "100", None),
    )
    .unwrap();
    let cost = TripCostInput {
        trip_id: Some(trip.id),
        date: "10-10-2026".into(),
        amount: "1,000".into(),
        reason: "food".into(),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    };
    let saved = trips::cost_save(&w.env.ctx, &w.owner, &cost).unwrap();
    let totals: Vec<i64> = [first.id, second.id]
        .iter()
        .map(|id| {
            purchases::get(&w.env.ctx, &PurchaseGetInput { id: *id })
                .unwrap()
                .trip_cost_paisa
                .rupees()
        })
        .collect();
    assert_eq!(totals, vec![600, 400]);
    // Cancelling the cost takes exactly those parts off again, and gives the cash back.
    let cash_before = cash_now(&w);
    trips::cost_cancel(
        &w.env.ctx,
        &w.owner,
        &TripEntryCancelInput {
            id: saved.id,
            version: saved.version,
            reason: "wrong".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(cash_now(&w), cash_before + 1_000);
    for id in [first.id, second.id] {
        let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id }).unwrap();
        assert_eq!(row.trip_cost_paisa.rupees(), 0);
        assert!(row.can_cancel);
    }
    balanced(&w);
    let err = trips::cost_cancel(
        &w.env.ctx,
        &w.owner,
        &TripEntryCancelInput {
            id: saved.id,
            version: saved.version,
            reason: "again".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(first_error(&err).0, ErrorCode::ErrTripEntryCancelled);
}

#[test]
fn a_trip_cost_needs_goods_left_and_a_reason() {
    let w = world();
    let trip = trip_of_three(&w);
    let cost = TripCostInput {
        trip_id: Some(trip.id),
        date: "10-10-2026".into(),
        amount: "4,000".into(),
        reason: "truck".into(),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    };
    let err = trips::cost_save(&w.env.ctx, &w.owner, &cost).unwrap_err();
    assert_eq!(
        first_error(&err),
        (ErrorCode::ErrTripNoStock, Some("trip_id".into()))
    );
    purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", None),
    )
    .unwrap();
    let blank = TripCostInput {
        reason: "  ".into(),
        ..cost.clone()
    };
    let preview = trips::cost_preview(&w.env.ctx, &blank).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrReasonRequired,
        "reason"
    ));
    let zero = TripCostInput {
        amount: "0".into(),
        ..cost
    };
    let preview = trips::cost_preview(&w.env.ctx, &zero).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrMustBeMoreThanZero,
        "amount"
    ));
}

#[test]
fn e2e_34_and_34b_with_the_sale_in_another_shop_the_result_and_the_closing() {
    // The purchase of E2E-34 (10,400), 15 sold in another shop for 8,460, the last 5 written off.
    let w = world();
    let trip = trip_of_three(&w);
    let [a, b, c] = w.partners;
    let purchase = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    let sale_input = OtherSaleInput {
        trip_id: Some(trip.id),
        date: "10-10-2026".into(),
        shop: Some("Haji Shop".into()),
        purchase_id: Some(purchase.id),
        qty: "15".into(),
        amount: "8,460".into(),
        note: None,
    };
    let preview = trips::other_sale_preview(&w.env.ctx, &sale_input).unwrap();
    let numbers = preview.numbers.unwrap();
    assert_eq!(numbers.cost_paisa.rupees(), 7_800, "C21: 10,400 x 15 / 20");
    assert_eq!(numbers.result_paisa.rupees(), 660);
    let credits: Vec<i64> = numbers
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(credits, vec![3_384, 2_538, 2_538]);
    assert_eq!(numbers.qty_left_after, 5);
    assert_eq!(numbers.cost_left_after_paisa.rupees(), 2_600);

    let cash_before = cash_now(&w);
    let sale = trips::other_sale_save(&w.env.ctx, &w.owner, &sale_input).unwrap();
    assert!(!sale.trip_closed);
    assert_eq!(cash_now(&w), cash_before + 8_460);
    let rows = lines(&w, sale.entry_id);
    assert_eq!((rows[0].0.as_str(), rows[0].1), ("01010001", 8_460));
    assert_eq!(rows.iter().map(|r| r.2).sum::<i64>(), 8_460);
    // Partner totals so far: -4,160 + 3,384 = -776, -3,120 + 2,538 = -582 twice.
    assert_eq!(credit_balance(&w, venture_account(&w, a)), -776);
    assert_eq!(credit_balance(&w, venture_account(&w, b)), -582);
    assert_eq!(credit_balance(&w, venture_account(&w, c)), -582);
    let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id: purchase.id }).unwrap();
    assert_eq!((row.qty_sold, row.qty_left), (15, 5));
    assert_eq!(row.cost_sold_paisa.rupees(), 7_800);
    assert_eq!(row.cost_left_paisa.rupees(), 2_600);
    assert_eq!(row.other_shop_paisa.rupees(), 8_460);
    assert_eq!(row.result_paisa.rupees(), 660);
    // The goods in the stock list are the 5 left, marked with their trip.
    let stock = mandi_services::bikri::stock_list(
        &w.env.ctx,
        &mandi_services::core::bikri::StockListInput::default(),
    )
    .unwrap();
    assert_eq!(stock.own_rows.len(), 1);
    assert_eq!(stock.own_rows[0].balance, 5);
    assert_eq!(stock.own_rows[0].cost_left_paisa.rupees(), 2_600);
    assert_eq!(stock.own_rows[0].trip.id, trip.id);
    // More than the stock left is refused.
    let too_many = OtherSaleInput {
        qty: "6".into(),
        ..sale_input.clone()
    };
    let preview = trips::other_sale_preview(&w.env.ctx, &too_many).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrQtyExceedsLot,
        "qty"
    ));

    // E2E-34b: on 11-10-26 the last 5 spoil. Nothing posts.
    set_today(&w.env, 2026, 10, 11);
    let before = entry_count(&w);
    let off = purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: purchase.id,
            qty: "5".into(),
            reason: "spoiled".into(),
        },
    )
    .unwrap();
    assert_eq!(entry_count(&w), before, "a write-off posts nothing");
    assert_eq!(
        off.cost_paisa.rupees(),
        2_600,
        "the last nag take all the cost left"
    );
    assert!(off.trip_closed);
    assert_eq!((off.qty_left, off.cost_left_paisa.rupees()), (0, 0));
    assert_eq!(
        credit_balance(&w, venture_account(&w, a)),
        -776,
        "partner totals do not move"
    );
    let detail = trips::get(
        &w.env.ctx,
        &TripGetInput {
            id: trip.id,
            entries_offset: 0,
            entries_limit: 0,
        },
    )
    .unwrap();
    assert_eq!(detail.row.status, VentureStatus::Closed);
    assert_eq!(
        detail.row.result_paisa.rupees(),
        -1_940,
        "8,460 - 7,800 - 2,600"
    );
    assert_eq!(detail.row.cost_written_off_paisa.rupees(), 2_600);
    assert_eq!(detail.row.cost_sold_paisa.rupees(), 7_800);
    let parts: Vec<i64> = detail
        .partner_results
        .iter()
        .map(|p| p.net_paisa.rupees())
        .collect();
    assert_eq!(parts, vec![-776, -582, -582]);
    assert_eq!(parts.iter().sum::<i64>(), -1_940);
    // The trip is closed: no new purchase or cost.
    let err = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "5", "100", None),
    )
    .unwrap_err();
    assert_eq!(first_error(&err).0, ErrorCode::ErrTripClosed);
    let preview =
        purchases::preview(&w.env.ctx, &purchase_input(&w, trip.id, "5", "100", None)).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrTripClosed,
        "trip_id"
    ));
    // A cost of a closed trip is refused the same way (plan 16b acceptance 4).
    let cost_on_closed = TripCostInput {
        trip_id: Some(trip.id),
        date: "11-10-2026".into(),
        amount: "500".into(),
        reason: "truck".into(),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    };
    let preview = trips::cost_preview(&w.env.ctx, &cost_on_closed).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrTripClosed,
        "trip_id"
    ));
    let err = trips::cost_save(&w.env.ctx, &w.owner, &cost_on_closed).unwrap_err();
    assert_eq!(first_error(&err).0, ErrorCode::ErrTripClosed);
    balanced(&w);

    // Undoing the write-off gives back exactly the qty and the cost, and opens the trip again.
    let undone = purchases::write_off_undo(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffUndoInput {
            move_id: off.move_id,
            reason: "not spoiled".into(),
        },
    )
    .unwrap();
    assert!(undone.trip_reopened);
    assert_eq!(
        (undone.qty_left, undone.cost_left_paisa.rupees()),
        (5, 2_600)
    );
    let detail = trips::get(
        &w.env.ctx,
        &TripGetInput {
            id: trip.id,
            entries_offset: 0,
            entries_limit: 0,
        },
    )
    .unwrap();
    assert_eq!(detail.row.status, VentureStatus::Open);
    assert_eq!(detail.row.result_paisa.rupees(), 660);

    // Cancelling the sale takes the money out again and gives the 15 and 7,800 back.
    let cancelled = trips::other_sale_cancel(
        &w.env.ctx,
        &w.owner,
        &TripEntryCancelInput {
            id: sale.id,
            version: sale.version,
            reason: "wrong shop".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert!(cancelled.reversal_entry_id.is_some());
    assert_eq!(cash_now(&w), cash_before);
    let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id: purchase.id }).unwrap();
    assert_eq!((row.qty_sold, row.qty_left), (0, 20));
    assert_eq!(row.cost_left_paisa.rupees(), 10_400);
    assert_eq!(credit_balance(&w, venture_account(&w, a)), -4_160);
    balanced(&w);
}

#[test]
fn the_last_nag_of_a_part_sale_take_all_the_cost_left_and_no_more() {
    // C21 over two sales: 10,400 over 20 nag: 15 take 7,800, then 5 take the 2,600 that is left.
    let w = world();
    let trip = trip_of_three(&w);
    let purchase = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    let sell = |qty: &str, money: &str| OtherSaleInput {
        trip_id: Some(trip.id),
        date: "10-10-2026".into(),
        shop: None,
        purchase_id: Some(purchase.id),
        qty: qty.into(),
        amount: money.into(),
        note: None,
    };
    trips::other_sale_save(&w.env.ctx, &w.owner, &sell("15", "9,000")).unwrap();
    let last = trips::other_sale_save(&w.env.ctx, &w.owner, &sell("5", "2,000")).unwrap();
    assert!(last.trip_closed);
    let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id: purchase.id }).unwrap();
    assert_eq!(row.cost_released_paisa.rupees(), 10_400);
    assert_eq!(row.cost_sold_paisa.rupees(), 10_400);
    assert_eq!(row.result_paisa.rupees(), 11_000 - 10_400);
}

#[test]
fn purchase_refusals() {
    let w = world();
    let trip = trip_of_three(&w);
    // Dated before its trip.
    set_today(&w.env, 2026, 10, 12);
    let late = trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input("12-10-2026", vec![shop(w.partners[0], "100")]),
    )
    .unwrap();
    let mut early = purchase_input(&w, late.id, "5", "100", None);
    early.date = "11-10-2026".into();
    let preview = purchases::preview(&w.env.ctx, &early).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrPurchaseBeforeTrip,
        "date"
    ));
    // No trip, a missing item, a weight item without its kilograms.
    let mut bad = purchase_input(&w, trip.id, "5", "100", None);
    bad.trip_id = None;
    bad.item_id = None;
    let preview = purchases::preview(&w.env.ctx, &bad).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrRequired,
        "trip_id"
    ));
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrRequired,
        "item_id"
    ));
    let mut weight = purchase_input(&w, trip.id, "5", "100", None);
    weight.item_id = Some(w.weight_fruit);
    let preview = purchases::preview(&w.env.ctx, &weight).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrWeightRequired,
        "weight"
    ));
    // A decimal rate is refused.
    let preview =
        purchases::preview(&w.env.ctx, &purchase_input(&w, trip.id, "5", "10.5", None)).unwrap();
    assert!(support::has(
        &preview.errors,
        ErrorCode::ErrDecimalsNotAllowed,
        "rate"
    ));
}

#[test]
fn a_cancel_is_refused_once_something_was_sold_or_a_cost_was_spread_and_the_trip_can_close() {
    let w = world();
    let trip = trip_of_three(&w);
    let sold = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "10", "100", None),
    )
    .unwrap();
    let kept = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "10", "100", None),
    )
    .unwrap();
    trips::other_sale_save(
        &w.env.ctx,
        &w.owner,
        &OtherSaleInput {
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            shop: None,
            purchase_id: Some(sold.id),
            qty: "10".into(),
            amount: "1,500".into(),
            note: None,
        },
    )
    .unwrap();
    let cancel = |id: i64, version: i64| {
        purchases::cancel(
            &w.env.ctx,
            &w.owner,
            &PurchaseCancelInput {
                id,
                version,
                reason: "wrong".into(),
                confirm_cash_below_zero: false,
            },
        )
    };
    let sold_now = purchases::get(&w.env.ctx, &PurchaseGetInput { id: sold.id }).unwrap();
    let err = cancel(sold.id, sold_now.version).unwrap_err();
    assert_eq!(
        first_error(&err),
        (ErrorCode::ErrPurchaseHasSales, Some("id".into()))
    );
    // A cost spread over the other purchase (the only one with goods left) blocks its cancel.
    let cost = trips::cost_save(
        &w.env.ctx,
        &w.owner,
        &TripCostInput {
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            amount: "500".into(),
            reason: "truck".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let err = cancel(kept.id, kept.version).unwrap_err();
    assert_eq!(first_error(&err).0, ErrorCode::ErrVersionChanged);
    let current = purchases::get(&w.env.ctx, &PurchaseGetInput { id: kept.id }).unwrap();
    let err = cancel(kept.id, current.version).unwrap_err();
    assert_eq!(
        first_error(&err),
        (ErrorCode::ErrPurchaseCarriesTripCost, Some("id".into()))
    );
    // Cancel the cost first, then the purchase: every other purchase is sold out, so the trip closes.
    trips::cost_cancel(
        &w.env.ctx,
        &w.owner,
        &TripEntryCancelInput {
            id: cost.id,
            version: cost.version,
            reason: "no".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let current = purchases::get(&w.env.ctx, &PurchaseGetInput { id: kept.id }).unwrap();
    cancel(kept.id, current.version).unwrap();
    let detail = trips::get(
        &w.env.ctx,
        &TripGetInput {
            id: trip.id,
            entries_offset: 0,
            entries_limit: 0,
        },
    )
    .unwrap();
    assert_eq!(
        detail.row.status,
        VentureStatus::Closed,
        "PRD FR-12.33 after a cancel too"
    );
    assert_eq!(detail.row.purchase_count, 1);
    balanced(&w);
}

#[test]
fn a_trip_cost_cannot_be_cancelled_after_its_purchase_sold_something() {
    let w = world();
    let trip = trip_of_three(&w);
    let purchase = purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "100", None),
    )
    .unwrap();
    let cost = trips::cost_save(
        &w.env.ctx,
        &w.owner,
        &TripCostInput {
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            amount: "1,000".into(),
            reason: "truck".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    trips::other_sale_save(
        &w.env.ctx,
        &w.owner,
        &OtherSaleInput {
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            shop: None,
            purchase_id: Some(purchase.id),
            qty: "5".into(),
            amount: "800".into(),
            note: None,
        },
    )
    .unwrap();
    let err = trips::cost_cancel(
        &w.env.ctx,
        &w.owner,
        &TripEntryCancelInput {
            id: cost.id,
            version: cost.version,
            reason: "no".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(
        first_error(&err),
        (ErrorCode::ErrTripCostHasSales, Some("id".into()))
    );
}

#[test]
fn shares_change_only_before_the_first_entry() {
    let w = world();
    let [a, b, c] = w.partners;
    let trip = trip_of_three(&w);
    let changed = trips::set_partners(
        &w.env.ctx,
        &w.owner,
        &TripSetPartnersInput {
            id: trip.id,
            version: trip.version,
            partners: vec![shop(a, "50"), pocket(b, "50"), shop(c, "0")],
        },
    )
    .unwrap();
    assert_eq!(changed.partners.len(), 2);
    assert!(!changed.shares_locked);
    // A stale version is refused.
    let err = trips::set_partners(
        &w.env.ctx,
        &w.owner,
        &TripSetPartnersInput {
            id: trip.id,
            version: trip.version,
            partners: vec![shop(a, "100")],
        },
    )
    .unwrap_err();
    assert_eq!(first_error(&err).0, ErrorCode::ErrVersionChanged);
    purchases::save(
        &w.env.ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "10", "100", None),
    )
    .unwrap();
    let err = trips::set_partners(
        &w.env.ctx,
        &w.owner,
        &TripSetPartnersInput {
            id: trip.id,
            version: changed.version,
            partners: vec![shop(a, "100")],
        },
    )
    .unwrap_err();
    assert_eq!(
        first_error(&err),
        (ErrorCode::ErrVentureSharesLocked, Some("partners".into()))
    );
    let detail = trips::get(
        &w.env.ctx,
        &TripGetInput {
            id: trip.id,
            entries_offset: 0,
            entries_limit: 0,
        },
    )
    .unwrap();
    assert!(detail.row.shares_locked);
}

#[test]
fn cash_below_zero_asks_for_the_shop_parts_only() {
    let w = world_with(5_000, true);
    let trip = trip_of_three(&w);
    // 20 x 500 = 10,000 from the shop would take cash 5,000 to -5,000.
    let input = purchase_input(&w, trip.id, "20", "500", None);
    let preview = purchases::preview(&w.env.ctx, &input).unwrap();
    assert!(preview.cash.needs_confirm);
    assert_eq!(preview.cash.cash_after_paisa.rupees(), -5_000);
    let before = entry_count(&w);
    let err = purchases::save(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(first_error(&err).0, ErrorCode::ErrCashBelowZeroNotConfirmed);
    assert_eq!(entry_count(&w), before, "a refused save writes nothing");
    let confirmed = PurchaseInput {
        confirm_cash_below_zero: true,
        ..input.clone()
    };
    purchases::save(&w.env.ctx, &w.owner, &confirmed).unwrap();
    assert_eq!(cash_now(&w), -5_000);
    // The same purchase paid from pockets asks nothing.
    let [a, b, c] = w.partners;
    let pockets = trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input(
            "10-10-2026",
            vec![pocket(a, "40"), pocket(b, "30"), pocket(c, "30")],
        ),
    )
    .unwrap();
    let preview = purchases::preview(
        &w.env.ctx,
        &purchase_input(&w, pockets.id, "20", "500", None),
    )
    .unwrap();
    assert!(!preview.cash.needs_confirm);
}

#[test]
fn the_0604_accounts_take_no_opening_and_a_partner_in_an_open_trip_stays_active() {
    let w = world_with(50_000, false);
    let [a, b, c] = w.partners;
    let acc = venture_account(&w, a);
    let version = accounts::get(&w.env.ctx, acc).unwrap().version;
    let err = accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: acc,
                version,
                opening_date: "10-10-2026".into(),
                opening_dr: "100".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    );
    assert!(err.is_err(), "an opening on a 0604 account is refused");
    // Partner ج is in an open trip, so he cannot be made inactive (PRD A63).
    trips::create(
        &w.env.ctx,
        &w.owner,
        &trip_input("10-10-2026", vec![shop(a, "50"), shop(b, "50")]),
    )
    .unwrap();
    let detail = partners::get(&w.env.ctx, a).unwrap();
    let update = |id: i64, version: i64, active: bool| {
        partners::update(
            &w.env.ctx,
            &w.owner,
            &PartnerUpdateInput {
                id,
                version,
                name_ur: "الف".into(),
                name_en: "Partner A".into(),
                is_active: active,
                confirmed_rules_words: Vec::new(),
            },
        )
    };
    let err = update(a, detail.row.version, false).unwrap_err();
    assert_eq!(
        first_error(&err),
        (ErrorCode::ErrPartnerInOpenVenture, Some("is_active".into()))
    );
    // ج is not in the trip.
    let outside = partners::get(&w.env.ctx, c).unwrap();
    let c_row = PartnerUpdateInput {
        id: c,
        version: outside.row.version,
        name_ur: "ج".into(),
        name_en: "Partner C".into(),
        is_active: false,
        confirmed_rules_words: Vec::new(),
    };
    partners::update(&w.env.ctx, &w.owner, &c_row).unwrap();
}

#[test]
fn lists_load_page_by_page_with_totals_over_the_whole_filter() {
    let w = world();
    let [a, b, _] = w.partners;
    for n in 0..3 {
        let trip = trips::create(
            &w.env.ctx,
            &w.owner,
            &trip_input("10-10-2026", vec![shop(a, "50"), shop(b, "50")]),
        )
        .unwrap();
        purchases::save(
            &w.env.ctx,
            &w.owner,
            &purchase_input(&w, trip.id, "10", &format!("{}", 100 + n), None),
        )
        .unwrap();
    }
    let page = trips::list(
        &w.env.ctx,
        &TripsListInput {
            limit: 2,
            ..TripsListInput::default()
        },
    )
    .unwrap();
    assert_eq!(page.rows.len(), 2);
    assert_eq!(page.total_count, 3);
    assert_eq!(page.rows[0].number, 3, "newest first");
    assert_eq!(page.totals.qty_left, 30);
    assert_eq!(page.totals.cost_paisa.rupees(), 1_000 + 1_010 + 1_020);
    let next = trips::list(
        &w.env.ctx,
        &TripsListInput {
            limit: 2,
            offset: 2,
            ..TripsListInput::default()
        },
    )
    .unwrap();
    assert_eq!(next.rows.len(), 1);
    assert_eq!(
        next.totals.qty_left, 30,
        "the totals are over the whole filter"
    );
    let open_only = trips::list(
        &w.env.ctx,
        &TripsListInput {
            status: Some(VentureStatus::Closed),
            ..TripsListInput::default()
        },
    )
    .unwrap();
    assert_eq!(open_only.total_count, 0);
    let purchases_page = purchases::list(
        &w.env.ctx,
        &PurchasesListInput {
            limit: 2,
            state: Some(PurchaseStateFilter::InStock),
            ..PurchasesListInput::default()
        },
    )
    .unwrap();
    assert_eq!(purchases_page.rows.len(), 2);
    assert_eq!(purchases_page.total_count, 3);
    assert_eq!(purchases_page.totals.qty_left, 30);
    let summary = trips::summary(&w.env.ctx).unwrap();
    assert_eq!(
        (
            summary.open_trips,
            summary.purchases_with_stock,
            summary.qty_left
        ),
        (3, 3, 30)
    );
    assert_eq!(summary.cost_left_paisa.rupees(), 3_030);
    let _: Option<PurchaseRow> = None;
}

/// Plan 33 T1: the own-lot picker and the new sale ask `purchases_list` for a day. The answer for
/// `from = to = that day` with `state = in_stock` is the purchases of that day with goods left, with
/// everything a ready lot needs: date, trip, number, item, packing and qty left. A purchase of a
/// closed trip, one with nothing left, a cancelled one and one of another day are not in it.
#[test]
fn plan_33_the_purchases_of_one_day_with_goods_left_are_what_a_new_sale_needs() {
    let w = world();
    let [a, b, c] = w.partners;
    let make_trip = |date: &str| {
        trips::create(
            &w.env.ctx,
            &w.owner,
            &trip_input(date, vec![shop(a, "40"), shop(b, "30"), shop(c, "30")]),
        )
        .unwrap()
    };
    let (trip_a, trip_b, trip_c) = (
        make_trip("10-10-2026"),
        make_trip("10-10-2026"),
        make_trip("10-10-2026"),
    );
    let save = |trip: i64, qty: &str, rate: &str| {
        purchases::save(
            &w.env.ctx,
            &w.owner,
            &purchase_input(&w, trip, qty, rate, None),
        )
        .unwrap()
    };
    // Today 10-10: خ-1 (trip A, 20), خ-2 (trip B, 30), خ-3 (trip C, 10, then all written off, so
    // trip C closes), خ-4 (trip A, 5, cancelled).
    let p1 = save(trip_a.id, "20", "50");
    let p2 = save(trip_b.id, "30", "40");
    let p3 = save(trip_c.id, "10", "10");
    let p4 = save(trip_a.id, "5", "10");
    assert_eq!(
        (p1.number, p2.number, p3.number, p4.number),
        (1, 2, 3, 4),
        "the numbers go up in the order of saving"
    );
    purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: p3.id,
            qty: "10".into(),
            reason: "spoiled".into(),
        },
    )
    .unwrap();
    purchases::cancel(
        &w.env.ctx,
        &w.owner,
        &PurchaseCancelInput {
            id: p4.id,
            version: p4.version,
            reason: "wrong".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let closed = trips::list(
        &w.env.ctx,
        &TripsListInput {
            status: Some(VentureStatus::Closed),
            ..TripsListInput::default()
        },
    )
    .unwrap();
    assert_eq!(
        closed.rows.iter().map(|t| t.id).collect::<Vec<_>>(),
        [trip_c.id],
        "a trip whose goods are all written off closes by itself"
    );
    // Tomorrow, 11-10: a purchase on the open trip A.
    set_today(&w.env, 2026, 10, 11);
    let p5 = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            date: "11-10-2026".into(),
            ..purchase_input(&w, trip_a.id, "8", "30", None)
        },
    )
    .unwrap();

    let day = |date: &str| {
        let mut rows = purchases::list(
            &w.env.ctx,
            &PurchasesListInput {
                from: Some(date.into()),
                to: Some(date.into()),
                state: Some(PurchaseStateFilter::InStock),
                ..PurchasesListInput::default()
            },
        )
        .unwrap()
        .rows;
        // The list is newest first; a new sale takes the lots in purchase number order.
        rows.sort_by_key(|r| r.number);
        rows
    };
    let tenth = day("10-10-2026");
    assert_eq!(
        tenth.iter().map(|r| r.number).collect::<Vec<_>>(),
        [1, 2],
        "خ-3 (closed trip, nothing left) and خ-4 (cancelled) are not there"
    );
    let first = &tenth[0];
    assert_eq!(first.entry_date, "2026-10-10");
    assert_eq!(
        (first.trip.id, first.trip.number),
        (trip_a.id, trip_a.number)
    );
    assert_eq!(first.item.id, w.banana);
    assert_eq!(first.packing.id, w.box_);
    assert_eq!((first.qty, first.qty_left), (20, 20));
    assert!(!first.trip.name_en.is_empty() && !first.item.name_ur.is_empty());
    assert_eq!(tenth[1].trip.id, trip_b.id);
    assert_eq!(tenth[1].qty_left, 30);
    // The other day has its own purchase only, and the written-off and cancelled ones never come.
    let eleventh = day("11-10-2026");
    assert_eq!(
        eleventh.iter().map(|r| r.id).collect::<Vec<_>>(),
        [p5.id],
        "a purchase of another day is not in the day"
    );
    assert_eq!(eleventh[0].entry_date, "2026-10-11");
    assert!(day("12-10-2026").is_empty());
    // Without a day, the same filter gives all the goods left, and still nothing of the closed trip.
    let all = purchases::list(
        &w.env.ctx,
        &PurchasesListInput {
            state: Some(PurchaseStateFilter::InStock),
            ..PurchasesListInput::default()
        },
    )
    .unwrap();
    assert_eq!(all.total_count, 3);
    assert!(all.rows.iter().all(|r| r.trip.id != trip_c.id));
    // A part sold or written off lowers the qty left that the day shows; nothing left drops it.
    purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: p2.id,
            qty: "12".into(),
            reason: "spoiled".into(),
        },
    )
    .unwrap();
    assert_eq!(day("10-10-2026")[1].qty_left, 18);
    purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: p2.id,
            qty: "18".into(),
            reason: "spoiled".into(),
        },
    )
    .unwrap();
    assert_eq!(
        day("10-10-2026")
            .iter()
            .map(|r| r.number)
            .collect::<Vec<_>>(),
        [1]
    );
    // A bad day is a clear refusal on its field.
    let error = purchases::list(
        &w.env.ctx,
        &PurchasesListInput {
            from: Some("31-02-2026".into()),
            ..PurchasesListInput::default()
        },
    )
    .unwrap_err();
    assert_eq!(first_error(&error).1.as_deref(), Some("from"));
}
