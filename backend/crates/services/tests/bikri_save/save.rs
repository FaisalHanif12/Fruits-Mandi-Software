//! What a save writes and what it refuses (PRD FR-12.8, FR-12.9, FR-12.23, FR-13.10, FR-14.1,
//! FR-7.52, E3, E6, plan 13), on a real database. The golden numbers of Event 1 and Event 2 are
//! in `golden_day.rs`; here are the other cases of the money agent's list.

use mandi_services::bikri;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, FreightLotInput, FreightPaidBy, HeaderInput, LineChargeMethod,
    LotChargeMethod, SplitFreightInput, StoredMethod, split_freight,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::{assist, ledger};

use crate::helpers::*;
use crate::support::*;

fn save_unconfirmed(
    w: &World,
    d: &BikriDraft,
) -> Result<mandi_services::core::bikri::BikriSaved, mandi_services::ServiceError> {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: d.clone(),
            confirm_cash_below_zero: false,
        },
    )
}

#[test]
fn numbers_go_up_by_one_and_a_refused_save_does_not_use_one() {
    let w = world();
    assert_eq!(save_ok(&w, &event1(&w)).number, 1);
    // A refused draft (qty is not a number) leaves no row and takes no number.
    let mut bad = event1(&w);
    bad.lots[0].sales[0].qty = "abc".into();
    assert!(save(&w, &bad).is_err());
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_entries"), 1);
    let again = save_ok(&w, &event1(&w));
    assert_eq!((again.number, again.bikri_id), (2, 2));
}

#[test]
fn blank_rows_are_dropped_and_the_saved_lines_are_numbered_from_one() {
    // PRD FR-7.52: a row with no buyer and no rate is ignored and the save removes it.
    let w = world();
    let d = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![
            sale("s-1", w.akram, "20", "600"),
            blank_sale("s-2", "30"),
            sale("s-3", w.arshad, "30", "580"),
        ],
    )]);
    let saved = save_ok(&w, &d);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let lines: Vec<(i64, i64)> = detail.lots[0]
        .sales
        .iter()
        .map(|s| (s.line_no, s.qty))
        .collect();
    assert_eq!(lines, [(1, 20), (2, 30)]);
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_sales"), 2);
}

#[test]
fn all_the_errors_come_together_and_nothing_is_written() {
    let w = world();
    let before = snapshot(&raw_conn(&w));
    let mut d = event1(&w);
    d.lots[0].sales[0].qty = "abc".into();
    d.lots[0].sales[1].buyer_account_id = None;
    d.lots[0].qty_received = "0".into();
    let error = save(&w, &d).unwrap_err();
    assert!(
        rejects(&error, ErrorCode::ErrNotANumber, "lots[0].sales[0].qty"),
        "{error:?}"
    );
    assert!(
        rejects(
            &error,
            ErrorCode::ErrBuyerRequired,
            "lots[0].sales[1].buyer_account_id"
        ),
        "{error:?}"
    );
    assert!(
        rejects(
            &error,
            ErrorCode::ErrMustBeMoreThanZero,
            "lots[0].qty_received"
        ),
        "{error:?}"
    );
    assert_eq!(snapshot(&raw_conn(&w)), before);
}

fn raw_conn(w: &World) -> mandi_services::db::rusqlite::Connection {
    crate::common::raw(&w.dir)
}

#[test]
fn no_lot_no_sale_line_and_a_future_date_are_refused() {
    let w = world();
    let error = save(&w, &draft(vec![])).unwrap_err();
    assert!(rejects(&error, ErrorCode::ErrNoLots, "lots"), "{error:?}");
    let error = save(&w, &draft(vec![lot(&w, "lot-1", w.rahim, w.banana, "50")])).unwrap_err();
    assert!(
        rejects(&error, ErrorCode::ErrNoSaleLines, "lots"),
        "{error:?}"
    );
    let mut future = event1(&w);
    future.header.entry_date = "11-10-2026".into();
    let error = save(&w, &future).unwrap_err();
    assert!(
        rejects(&error, ErrorCode::ErrDateInFuture, "header.entry_date"),
        "{error:?}"
    );
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_entries"), 0);
}

#[test]
fn shares_that_do_not_total_100_refuse_the_save() {
    let w = world_with(50_000, false);
    let error = save(&w, &event1(&w)).unwrap_err();
    assert!(
        rejects(&error, ErrorCode::ErrSharesNot100, "split"),
        "{error:?}"
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM journal_entries WHERE source_type = 'bikri'"
        ),
        0
    );
}

#[test]
fn an_inactive_buyer_and_a_sold_qty_above_received_are_refused() {
    let w = world();
    mandi_services::accounts::set_active(&w.env.ctx, &w.owner, w.akram, false).unwrap();
    let error = save(&w, &event1(&w)).unwrap_err();
    assert!(
        rejects(
            &error,
            ErrorCode::ErrAccountInactive,
            "lots[0].sales[0].buyer_account_id"
        ),
        "{error:?}"
    );
    mandi_services::accounts::set_active(&w.env.ctx, &w.owner, w.akram, true).unwrap();
    let mut over = event1(&w);
    over.lots[0].sales[1].qty = "31".into();
    let error = save(&w, &over).unwrap_err();
    assert!(
        rejects(&error, ErrorCode::ErrQtyExceedsLot, "lots[0].sales[1].qty"),
        "{error:?}"
    );
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_entries"), 0);
}

#[test]
fn the_cash_confirmation_is_needed_only_when_cash_goes_below_zero() {
    let w = world_with(1_000, true);
    // Event 1 takes 5,000 of نقد out of 1,000.
    let error = save_unconfirmed(&w, &event1(&w)).unwrap_err();
    assert!(error.has_code(ErrorCode::ErrCashBelowZeroNotConfirmed));
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_entries"), 0);
    // A بکری that does not lower cash needs no confirmation.
    let mut quiet = event1(&w);
    quiet.lots[0].cash_paid = None;
    assert!(save_unconfirmed(&w, &quiet).is_ok());
}

// ---- Advances (PRD FR-15.8, FR-12.23) ----

fn adjusted(w: &World) -> i64 {
    scalar(
        w,
        "SELECT COALESCE(SUM(amount_paisa), 0) FROM advance_allocations WHERE is_active = 1",
    ) / 100
}

#[test]
fn an_advance_bigger_than_the_credit_stays_open_in_part() {
    // Advance 30,000, credit 24,316: adjusted 24,316, open 5,684 (money agent Q2).
    let w = world();
    give_advance(&w, w.rahim, "30,000");
    save_ok(&w, &event1(&w));
    assert_eq!(adjusted(&w), 24_316);
    assert_eq!(
        assist::advances(&w.env.ctx, w.rahim).unwrap().open_paisa,
        rupees(5_684)
    );
    // His ledger: 30,000 + 5,000 Dr, 24,316 Cr = 10,684 بنام.
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 10_684));
}

#[test]
fn two_advances_are_used_oldest_first() {
    let w = world();
    give_advance(&w, w.rahim, "10,000");
    give_advance(&w, w.rahim, "10,000");
    // A lot whose seller credit is 15,000: 25 × 600 = 15,000 gross, comm 900, so a bare lot with
    // a typed 0 charge gives 14,100; use freight to land on exactly 15,000? Simpler: check rows.
    let d = draft(vec![with_sales(
        bare_lot(&w, "lot-1", w.rahim, w.banana, "30"),
        vec![sale("s-1", w.akram, "30", "532")],
    )]);
    let saved = save_ok(&w, &d);
    // gross 15,960, comm 958 (957.6 rounded), credit 15,002? Read the real credit back.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let credit = detail.lots[0].net.rupees();
    let rows = rows(&w, "advance_allocations");
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(
        rows[0].2, 10_000,
        "the oldest advance is used in full first"
    );
    assert_eq!(rows[1].2, credit - 10_000);
    assert_eq!(adjusted(&w), credit);
}

#[test]
fn only_the_lot_with_the_tick_on_adjusts_an_advance() {
    // Two lots of one seller in one بکری: tick on for lot 1, off for lot 2.
    let w = world();
    give_advance(&w, w.rahim, "100,000");
    let mut second = with_sales(
        bare_lot(&w, "lot-2", w.rahim, w.banana, "10"),
        vec![sale("s-2", w.arshad, "10", "600")],
    );
    second.adjust_advance = false;
    let first = with_sales(
        bare_lot(&w, "lot-1", w.rahim, w.banana, "20"),
        vec![sale("s-1", w.akram, "20", "600")],
    );
    let saved = save_ok(&w, &draft(vec![first, second]));
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    // Lot 1: gross 12,000, comm 720, credit 11,280. Only that credit adjusts.
    assert_eq!(detail.lots[0].net.rupees(), 11_280);
    assert_eq!(adjusted(&w), 11_280);
    assert_eq!(rows(&w, "advance_allocations").len(), 1);
}

#[test]
fn a_negative_credit_is_a_plain_debit_and_adjusts_nothing() {
    // PRD FR-12.7: freight 20,000 on a lot that sells 6,000: the seller owes the firm.
    let w = world();
    give_advance(&w, w.rahim, "5,000");
    let mut l = bare_lot(&w, "lot-1", w.rahim, w.banana, "10");
    l.freight = Some("20,000".into());
    let d = draft(vec![with_sales(l, vec![sale("s-1", w.akram, "10", "600")])]);
    let saved = save_ok(&w, &d);
    let lines = entry_lines(&w, saved.entry_id);
    // gross 6,000, comm 360, charges 20,000: credit = -14,360, written as a debit of 14,360.
    assert!(
        lines.contains(&("030110001".to_owned(), 14_360, 0, None)),
        "{lines:?}"
    );
    assert_eq!(adjusted(&w), 0);
    assert!(saved.preview.lots[0].net_is_negative);
    // The debit is not an advance: his open advance is still just the 5,000 voucher.
    assert_eq!(
        assist::advances(&w.env.ctx, w.rahim).unwrap().open_paisa,
        rupees(5_000)
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM journal_lines WHERE is_advance = 1"
        ),
        1
    );
}

#[test]
fn a_credit_of_exactly_zero_makes_no_seller_line() {
    // gross 1,000, comm 60, freight 940: credit 0.
    let w = world();
    let mut l = bare_lot(&w, "lot-1", w.rahim, w.banana, "1");
    l.freight = Some("940".into());
    let saved = save_ok(
        &w,
        &draft(vec![with_sales(l, vec![sale("s-1", w.akram, "1", "1000")])]),
    );
    let lines = entry_lines(&w, saved.entry_id);
    assert!(!lines.iter().any(|l| l.0 == "030110001"), "{lines:?}");
    assert_eq!(
        bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0]
            .net
            .rupees(),
        0
    );
}

// ---- A lot with no sale, freight, charges ----

#[test]
fn a_lot_with_no_sale_is_saved_with_its_charges_and_a_negative_net() {
    // PRD E3, FR-12.7: 10 received, freight 1,000, labour 50, munshiana 50, phone 20.
    let w = world();
    let mut idle = bare_lot(&w, "lot-2", w.karim, w.apple, "10");
    idle.freight = Some("1,000".into());
    idle.labour = manual("50");
    idle.munshiana = manual("50");
    idle.phone = manual("20");
    let busy = with_sales(
        bare_lot(&w, "lot-1", w.rahim, w.banana, "20"),
        vec![sale("s-1", w.akram, "20", "600")],
    );
    let saved = save_ok(&w, &draft(vec![busy, idle]));
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let idle = &detail.lots[1];
    assert_eq!(
        (idle.qty_received, idle.qty_sold, idle.net.rupees()),
        (10, 0, -1_120)
    );
    assert!(idle.sales.is_empty());
    let lines = entry_lines(&w, saved.entry_id);
    // The seller owes 1,120: a debit of 1,120 on کریم; the charges are credits.
    assert!(
        lines.contains(&("030110002".to_owned(), 1_120, 0, None)),
        "{lines:?}"
    );
    assert!(
        lines.contains(&("02010002".to_owned(), 0, 1_000, None)),
        "{lines:?}"
    );
    // A received move for the idle lot and none sold.
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE lot_id = (SELECT id FROM bikri_lots WHERE lot_no = 2)"
        ),
        10
    );
    // Phone 20 of the idle lot joins the shared income: 720 + 720 + 160 + 20 = 1,620.
    assert_eq!(saved.preview.summary.shared_income.rupees(), 1_620);
    // The بکری still needs a sale line somewhere.
    let only_idle = draft(vec![bare_lot(&w, "lot-1", w.karim, w.apple, "10")]);
    assert!(save(&w, &only_idle).is_err());
}

#[test]
fn freight_paid_by_the_seller_is_kept_but_not_deducted_or_posted() {
    // PRD E6: freight 3,000 paid by the seller: deductions 7,084, net 22,316, credit 27,316.
    let w = world();
    let mut d = event1(&w);
    d.lots[0].freight_paid_by = FreightPaidBy::Seller;
    let saved = save_ok(&w, &d);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let lot = &detail.lots[0];
    assert_eq!(
        (lot.freight.rupees(), lot.freight_paid_by),
        (3_000, FreightPaidBy::Seller)
    );
    assert_eq!((lot.deductions.rupees(), lot.net.rupees()), (7_084, 22_316));
    // The freight is shown on the lot but is not in the deductions or the totals.
    assert_totals_equal(
        &detail.totals,
        &saved.preview.summary,
        "seller-paid freight",
    );
    assert_eq!(detail.totals.freight.rupees(), 0);
    assert_eq!(detail.totals.seller_credit.rupees(), 27_316);
    let lines = entry_lines(&w, saved.entry_id);
    assert!(
        lines.contains(&("030110001".to_owned(), 0, 27_316, None)),
        "{lines:?}"
    );
    assert!(!lines.iter().any(|l| l.0 == "02010002"));
}

#[test]
fn the_header_freight_split_posts_one_credit_to_the_freight_account() {
    // PRD C17: 1,000 over 30 and 20 نگ is 600 and 400; both post to کرایہ کھاتہ.
    let w = world();
    let parts = split_freight(&SplitFreightInput {
        total_freight: "1,000".into(),
        lots: vec![
            FreightLotInput {
                key: "lot-1".into(),
                qty_received: "30".into(),
            },
            FreightLotInput {
                key: "lot-2".into(),
                qty_received: "20".into(),
            },
        ],
    })
    .unwrap();
    let parts: Vec<i64> = parts.iter().map(|p| p.freight_paisa.as_paisa()).collect();
    assert_eq!(parts, [60_000, 40_000]);
    let mut one = with_sales(
        bare_lot(&w, "lot-1", w.rahim, w.banana, "30"),
        vec![sale("s-1", w.akram, "30", "600")],
    );
    one.freight = Some("600".into());
    let mut two = with_sales(
        bare_lot(&w, "lot-2", w.karim, w.apple, "20"),
        vec![sale("s-2", w.arshad, "20", "600")],
    );
    two.freight = Some("400".into());
    let mut d = draft(vec![one, two]);
    d.header.total_freight = Some("1,000".into());
    let saved = save_ok(&w, &d);
    let lines = entry_lines(&w, saved.entry_id);
    let freight: i64 = lines
        .iter()
        .filter(|l| l.0 == "02010002")
        .map(|l| l.2)
        .sum();
    assert_eq!(freight, 1_000);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(detail.total_freight.map(|f| f.rupees()), Some(1_000));
}

#[test]
fn lot_charges_keep_their_method_and_value() {
    // Typed manual labour 250 is kept as manual / 0 / 250; per نگ 5 on 50 as per_unit / 5 / 250.
    let w = world();
    let mut manual_labour = event1(&w);
    manual_labour.lots[0].labour = manual("250");
    let saved = save_ok(&w, &manual_labour);
    let lot = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0];
    assert_eq!(
        (
            lot.labour.method,
            lot.labour.value_paisa.rupees(),
            lot.labour.amount.rupees()
        ),
        (LotChargeMethod::Manual, 0, 250)
    );
    let saved = save_ok(&w, &event1(&w));
    let lot = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0];
    assert_eq!(
        (
            lot.labour.method,
            lot.labour.value_paisa.rupees(),
            lot.labour.amount.rupees()
        ),
        (LotChargeMethod::PerUnit, 5, 250)
    );
}

#[test]
fn a_commission_typed_on_a_line_is_kept_as_manual() {
    // PRD FR-11.10: 4% typed on 12,000 is 480, kept with method manual and 400 bps.
    let w = world();
    let mut d = event1(&w);
    d.lots[0].sales[0].seller_comm = typed(LineChargeMethod::Percent, "4");
    let saved = save_ok(&w, &d);
    let sale = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].sales[0];
    assert_eq!(
        (
            sale.seller_comm.method,
            sale.seller_comm.value_bps,
            sale.seller_comm.amount.rupees()
        ),
        (StoredMethod::Manual, Some(400), 480)
    );
    // The default commission of the other line keeps its percent and its rule.
    let other = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].sales[1];
    assert_eq!(
        (other.seller_comm.method, other.seller_comm.value_bps),
        (StoredMethod::Percent, Some(600))
    );
    assert!(other.seller_comm.rule_id.is_some());
}

#[test]
fn a_rate_difference_posts_to_the_rate_difference_account_on_either_side() {
    // Buyer rate 610 against seller rate 600 on 20 نگ: +200 جمع on 04010005.
    let w = world();
    let mut d = event1(&w);
    d.lots[0].sales[0].rate_differs = true;
    d.lots[0].sales[0].buyer_rate = Some("610".into());
    let saved = save_ok(&w, &d);
    assert!(entry_lines(&w, saved.entry_id).contains(&("04010005".to_owned(), 0, 200, None)));
    // Buyer rate 590: -200, written as a debit of 200 (money agent: entry 31,552 = 31,552,
    // shared income 3,736 splits 1,496 / 1,120 / 1,120).
    let mut d = event1(&w);
    d.lots[0].sales[0].rate_differs = true;
    d.lots[0].sales[0].buyer_rate = Some("590".into());
    let saved = save_ok(&w, &d);
    let lines = entry_lines(&w, saved.entry_id);
    assert!(
        lines.contains(&("04010005".to_owned(), 200, 0, None)),
        "{lines:?}"
    );
    assert_eq!(lines[0], ("03040001".to_owned(), 12_668, 0, None));
    // Sale part 31,552 = 31,552, and the cash part adds 5,000 to each side.
    assert_eq!(lines.iter().map(|l| l.1).sum::<i64>(), 36_552, "{lines:?}");
    assert_eq!(lines.iter().map(|l| l.2).sum::<i64>(), 36_552, "{lines:?}");
    assert_eq!(saved.preview.summary.shared_income.rupees(), 3_736);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_totals_equal(
        &detail.totals,
        &saved.preview.summary,
        "rate difference -200",
    );
    assert_eq!(detail.totals.rate_diff.rupees(), -200);
    let split: Vec<i64> = saved
        .preview
        .split
        .iter()
        .map(|p| p.income_paisa.rupees())
        .collect();
    assert_eq!(split, [1_496, 1_120, 1_120]);
}

// ---- The one-time person (PRD FR-13) ----

#[test]
fn a_one_time_seller_is_paid_on_the_spot() {
    // PRD FR-13.10: gross 12,000, deductions before نقد 1,840: نقد starts at 10,160, net 0.
    let w = world();
    let mut line = sale("s-1", w.akram, "10", "1200");
    line.payment_type = None;
    let mut l = lot(&w, "lot-1", w.pukhta_seller, w.apple, "10");
    l.pukhta = person("ناصر", "Nasir", "0300-1111111");
    l.freight = Some("1,000".into());
    l.cash_paid = None;
    l.sales = vec![line];
    let saved = save_ok(&w, &draft(vec![l]));
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let lot = &detail.lots[0];
    assert_eq!((lot.cash_paid.rupees(), lot.net.rupees()), (10_160, 0));
    assert_totals_equal(&detail.totals, &saved.preview.summary, "one-time seller");
    assert_eq!(lot.pukhta.as_ref().unwrap().person_key, "ناصر|03001111111");
    let key = Some("ناصر|03001111111".to_owned());
    let lines = entry_lines(&w, saved.entry_id);
    assert!(
        lines.contains(&("030119999".to_owned(), 0, 10_160, key.clone())),
        "{lines:?}"
    );
    assert!(
        lines.contains(&("030119999".to_owned(), 10_160, 0, key.clone())),
        "{lines:?}"
    );
    // His own balance is 0.
    let balance = ledger::person_balance(&w.env.ctx, w.pukhta_seller, "ناصر|03001111111").unwrap();
    assert_eq!(balance.side, BalanceSide::Zero);
}

#[test]
fn two_one_time_buyers_keep_separate_keys_and_pairs() {
    let w = world();
    let mut a = sale("s-1", w.pukhta_buyer, "10", "1000");
    a.pukhta = person("شاہد", "Shahid", "0300-0000000");
    let mut b = sale("s-2", w.pukhta_buyer, "10", "1000");
    b.pukhta = person("ناصر", "Nasir", "0300-1111111");
    let mut c = sale("s-3", w.pukhta_buyer, "10", "1000");
    // The same name with the Arabic heh (U+0647) and the same phone is the same person as شاہد.
    c.pukhta = person("\u{0634}\u{0627}\u{0647}\u{062F}", "Shahid", "03000000000");
    let l = with_sales(
        bare_lot(&w, "lot-1", w.rahim, w.banana, "30"),
        vec![a, b, c],
    );
    let saved = save_ok(&w, &draft(vec![l]));
    let keys: Vec<String> = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0]
        .sales
        .iter()
        .map(|s| s.pukhta.as_ref().unwrap().person_key.clone())
        .collect();
    assert_eq!(
        keys,
        ["شاہد|03000000000", "ناصر|03001111111", "شاہد|03000000000"]
    );
    let by_key = |key: &str| ledger::person_balance(&w.env.ctx, w.pukhta_buyer, key).unwrap();
    assert_eq!(by_key("شاہد|03000000000").side, BalanceSide::Zero);
    // Each cash sale is pinned to its own cash line: three pairs of 10,000 + comm + laga.
    assert_eq!(rows(&w, "bill_allocations").len(), 3);
}

#[test]
fn a_one_time_person_without_a_name_or_a_credit_buyer_without_a_phone_is_refused() {
    let w = world();
    let mut a = sale("s-1", w.pukhta_buyer, "10", "1000");
    a.pukhta = person("", "", "");
    let l = with_sales(bare_lot(&w, "lot-1", w.rahim, w.banana, "10"), vec![a]);
    let error = save(&w, &draft(vec![l])).unwrap_err();
    assert!(
        rejects(
            &error,
            ErrorCode::ErrPukhtaNameRequired,
            "lots[0].sales[0].pukhta.name_ur"
        ),
        "{error:?}"
    );
    let mut b = sale("s-1", w.pukhta_buyer, "10", "1000");
    b.pukhta = person("شاہد", "Shahid", "");
    b.payment_type = Some(mandi_services::core::bikri::PaymentType::Credit);
    let l = with_sales(bare_lot(&w, "lot-1", w.rahim, w.banana, "10"), vec![b]);
    let error = save(&w, &draft(vec![l])).unwrap_err();
    assert!(
        rejects(
            &error,
            ErrorCode::ErrPukhtaPhoneRequired,
            "lots[0].sales[0].pukhta.phone"
        ),
        "{error:?}"
    );
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_entries"), 0);
}

// ---- The saved header, the audit row ----

#[test]
fn the_header_is_kept_and_a_save_writes_one_audit_row() {
    let w = world();
    let mut d = event1(&w);
    d.header = HeaderInput {
        book: Some("A-7".into()),
        driver_name: Some("Bashir".into()),
        driver_phone: Some("0301-2345678".into()),
        total_freight: Some("3,000".into()),
        note: Some("first truck".into()),
        bill_date: "10-10-2026".into(),
        zamindar_account_id: Some(w.rahim),
        ..d.header
    };
    let saved = save_ok(&w, &d);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(
        (
            detail.book.as_deref(),
            detail.truck_no.as_deref(),
            detail.driver_name.as_deref(),
            detail.driver_phone.as_deref(),
            detail.note.as_deref(),
            detail.entry_date.as_str(),
            detail.zamindar.as_ref().map(|z| z.id)
        ),
        (
            Some("A-7"),
            Some("LES-1234"),
            Some("Bashir"),
            Some("03012345678"),
            Some("first truck"),
            "2026-10-10",
            Some(w.rahim)
        )
    );
    assert_eq!(detail.postings.len(), 1);
    assert_eq!(detail.postings[0].posting_type, "bikri_first");
    assert_eq!(detail.postings[0].total.rupees(), 36_564);
    let split: Vec<i64> = detail
        .split
        .iter()
        .map(|s| s.income_paisa.rupees())
        .collect();
    assert_eq!(split, [1_580, 1_184, 1_184]);
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'bikri' AND action = 'create'"
        ),
        1
    );
    crate::common::assert_chain_ok(&w.dir);
}
