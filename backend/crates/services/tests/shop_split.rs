//! Plan 35 (T2): many item lines in one entry, and the split between our shop and another shop
//! (PRD FR-12.41, R1 to R7, decisions D1 and D2). The expected numbers are the PRD's (E2E-34: 20 at
//! 500 with freight 400, cost 10,400, C21 7,800 for 15) and plan 35 Section 3. A line of an entry is
//! compared with the same purchase saved on its own, rupee for rupee.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::arrivals::{
    ArrivalEntryInput, ArrivalLineInput, ArrivalRow, ArrivalWriteOffInput,
};
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, BikriSaved, HeaderInput, LotInput, ReadyGoodsInput, StockListInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{
    GardenEntryCancelInput, GardenEntryInput, GardenInput, GardenOtherSaleInput, GardenRow,
};
use mandi_services::core::purchases::{
    PurchaseEntryInput, PurchaseGetInput, PurchaseInput, PurchaseLineInput, PurchaseRow,
    PurchaseWriteOffInput,
};
use mandi_services::core::split::SplitInput;
use mandi_services::core::trips::{OtherSaleInput, TripEntryCancelInput, TripInput};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, bikri, gardens, purchases, trips};
use support::{World, bare_lot, sale, with_sales, world};

fn day(w: &World, d: u32) {
    set_today(&w.env, 2026, 10, d);
}

fn partners_of(w: &World) -> Vec<VenturePartnerInput> {
    let [a, b, c] = w.partners;
    [(a, "40"), (b, "30"), (c, "30")]
        .into_iter()
        .map(|(partner_id, share)| VenturePartnerInput {
            partner_id,
            share: share.into(),
            source: MoneySource::Shop,
        })
        .collect()
}

fn new_trip(w: &World) -> i64 {
    trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: partners_of(w),
        },
    )
    .unwrap()
    .id
}

fn new_garden(w: &World) -> GardenRow {
    gardens::create(
        &w.env.ctx,
        &w.owner,
        &GardenInput {
            name_ur: "شاہ پور باغ".into(),
            name_en: "Shahpur Garden".into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "Shahpur".into(),
                "Garden".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            rent: "50,000".into(),
            season_end: None,
            note: None,
            partners: partners_of(w),
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .row
}

fn line(item: i64, w: &World, qty: &str, rate: &str, other: Option<&str>) -> PurchaseLineInput {
    PurchaseLineInput {
        key: format!("k-{item}"),
        item_id: Some(item),
        packing_id: Some(w.box_),
        qty: qty.into(),
        weight: None,
        rate: rate.into(),
        freight: None,
        labour: None,
        other_cost: None,
        note: None,
        other_qty: other.map(str::to_owned),
        shop_name: other.map(|_| "Haji Shop".to_owned()),
    }
}

fn entry(trip: i64, lines: Vec<PurchaseLineInput>) -> PurchaseEntryInput {
    PurchaseEntryInput {
        trip_id: Some(trip),
        date: "10-10-2026".into(),
        supplier: None,
        sell_date: None,
        parts: Vec::new(),
        confirm_cash_below_zero: false,
        lines,
    }
}

fn purchase(w: &World, id: i64) -> PurchaseRow {
    purchases::get(&w.env.ctx, &PurchaseGetInput { id }).unwrap()
}

fn has(e: &ServiceError, code: ErrorCode, field: &str) -> bool {
    match e {
        ServiceError::Rejected(list) => list
            .iter()
            .any(|r| r.code == code && r.field.as_deref() == Some(field)),
        _ => false,
    }
}

fn own_account(w: &World) -> i64 {
    w.env
        .ctx
        .db()
        .read(|c| mandi_db::accounts::id_by_code(c, "030119998"))
        .unwrap()
        .unwrap()
}

fn draft_on(date: &str, lots: Vec<LotInput>) -> BikriDraft {
    BikriDraft {
        header: HeaderInput {
            entry_date: date.into(),
            ..support::header()
        },
        lots,
    }
}

fn own_lot(w: &World, purchase_id: i64, qty: &str, rate: &str) -> LotInput {
    with_sales(
        LotInput {
            purchase_id: Some(purchase_id),
            ..bare_lot(w, "lot-1", own_account(w), w.banana, qty)
        },
        vec![sale("s-1", w.akram, qty, rate)],
    )
}

fn save_sale(w: &World, d: &BikriDraft) -> Result<BikriSaved, ServiceError> {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: d.clone(),
            confirm_cash_below_zero: false,
        },
    )
}

fn other_sale(w: &World, trip: i64, purchase_id: i64, qty: &str, money: &str) -> OtherSaleInput {
    let _ = w;
    OtherSaleInput {
        trip_id: Some(trip),
        date: "10-10-2026".into(),
        shop: Some("Haji Shop".into()),
        purchase_id: Some(purchase_id),
        qty: qty.into(),
        amount: money.into(),
        note: None,
    }
}

fn totals(w: &World) -> Vec<(i64, i64, i64)> {
    w.env
        .ctx
        .db()
        .read(|c| mandi_db::ledger::account_totals(c, None))
        .unwrap()
        .into_iter()
        .map(|(account_id, t)| (account_id, t.dr_paisa, t.cr_paisa))
        .collect()
}

// ---- R1, R7: a one-line entry is exactly today's purchase ----

#[test]
fn a_one_line_entry_gives_exactly_the_purchase_of_today_rupee_for_rupee() {
    // E2E-34: 20 مالٹا at 500 with freight 400: cost 10,400, paid from the shop's cash 4,160 / 3,120 /
    // 3,120 at 40 / 30 / 30. Two identical worlds: one saves it as a single purchase, the other as a
    // one-line entry. Everything must be the same.
    let single = world();
    let many = world();
    for w in [&single, &many] {
        day(w, 10);
    }
    let (ts, tm) = (new_trip(&single), new_trip(&many));
    let mut one = line(single.banana, &single, "20", "500", None);
    one.freight = Some("400".into());
    let single_saved = purchases::save(
        &single.env.ctx,
        &single.owner,
        &PurchaseInput {
            trip_id: Some(ts),
            date: "10-10-2026".into(),
            supplier: Some("Haji".into()),
            sell_date: None,
            item_id: one.item_id,
            packing_id: one.packing_id,
            qty: one.qty.clone(),
            weight: None,
            rate: one.rate.clone(),
            freight: one.freight.clone(),
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
            other_qty: None,
            shop_name: None,
        },
    )
    .unwrap();
    let mut e = entry(tm, vec![one]);
    e.supplier = Some("Haji".into());
    let preview = purchases::entry_preview(&many.env.ctx, &e).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let many_saved = purchases::entry_save(&many.env.ctx, &many.owner, &e).unwrap();
    assert_eq!(many_saved.saved.len(), 1);
    let (a, b) = (&single_saved, &many_saved.saved[0]);
    assert_eq!(
        (a.number, a.version, a.total_cost),
        (b.number, b.version, b.total_cost)
    );
    assert_eq!(
        a.preview, b.preview,
        "the numbers of the line are the numbers of the purchase"
    );
    assert_eq!(preview.lines[0].numbers, a.preview.numbers);
    assert_eq!(preview.paid_now_total, many_saved.paid_now_total);
    // The partners' parts of a one-line entry are the parts of the purchase, working and all.
    assert_eq!(
        preview.parts.as_ref(),
        a.preview.numbers.as_ref().map(|n| &n.parts)
    );
    assert_eq!(a.total_cost.rupees(), 10_400);
    // The books are identical: every account's debit and credit.
    assert_eq!(totals(&single), totals(&many));
    // The stored purchase is identical, field for field (ids differ by trip only if trips differ).
    let (ra, rb) = (purchase(&single, a.id), purchase(&many, b.id));
    assert_eq!(
        serde_json::to_value(&ra).unwrap(),
        serde_json::to_value(&rb).unwrap()
    );
    assert_eq!(
        (ra.qty_other, ra.qty_our_left, ra.qty_other_left),
        (0, 20, 0)
    );
}

// ---- R1: many lines, one transaction, one cash question ----

#[test]
fn a_three_line_entry_makes_three_purchases_each_posted_as_one_purchase() {
    let w = world();
    day(&w, 10);
    let t = new_trip(&w);
    // Plan 35 Section 3: line 1 مالٹا-like 80 at 1,200 (50 for our shop, 30 for another), line 2 40 at
    // 900, line 3 10 at 500.
    let e = entry(
        t,
        vec![
            line(w.banana, &w, "80", "1,200", Some("30")),
            line(w.apple, &w, "40", "900", None),
            line(w.mooli, &w, "10", "500", None),
        ],
    );
    let preview = purchases::entry_preview(&w.env.ctx, &e).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let paid: Vec<i64> = preview
        .lines
        .iter()
        .map(|l| l.numbers.as_ref().unwrap().paid_now.rupees())
        .collect();
    assert_eq!(paid, [96_000, 36_000, 5_000]);
    assert_eq!(preview.paid_now_total.rupees(), 137_000);
    // The partners' parts of the whole entry: 40% of 96,000 + 36,000 + 5,000, summed over the lines.
    let parts = preview.parts.as_ref().expect("the lines work out");
    assert_eq!(parts.shop_total_paisa.rupees(), 137_000);
    assert_eq!(parts.pocket_total_paisa.rupees(), 0);
    assert_eq!(parts.parts[0].amount_paisa.rupees(), 54_800);
    assert_eq!(parts.parts[0].working.message_id, "working-entry-part-sum");
    assert_eq!(
        parts
            .parts
            .iter()
            .map(|p| p.amount_paisa.rupees())
            .sum::<i64>(),
        137_000,
        "the parts add up to the entry"
    );
    let saved = purchases::entry_save(&w.env.ctx, &w.owner, &{
        let mut e = e.clone();
        e.confirm_cash_below_zero = true; // 137,000 is more than the 50,000 of cash in hand
        e
    })
    .unwrap();
    let numbers: Vec<i64> = saved.saved.iter().map(|s| s.number).collect();
    assert_eq!(numbers, [1, 2, 3], "own numbers in the order of the lines");
    assert_eq!(saved.paid_now_total.rupees(), 137_000);
    // Each line has its own posting.
    let entries: Vec<Option<i64>> = saved.saved.iter().map(|s| s.entry_id).collect();
    assert!(entries.iter().all(Option::is_some));
    assert_eq!(
        entries
            .iter()
            .flatten()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    // The split of line 1.
    let first = purchase(&w, saved.saved[0].id);
    assert_eq!(
        (
            first.qty_other,
            first.qty_our_left,
            first.qty_other_left,
            first.shop_name.as_deref()
        ),
        (30, 50, 30, Some("Haji Shop"))
    );
}

#[test]
fn the_parts_of_an_entry_are_the_sum_of_the_per_line_splits_not_one_split_of_the_total() {
    // Two lines of 1,003 at 40 / 30 / 30. Each line is split on its own (C23): 1,003 gives 401 / 300
    // / 300 with 2 over, and the leftover goes to the largest share, so 403 / 300 / 300. The entry
    // is 806 / 600 / 600 (2,006). One split of 2,006 would give 804 / 601 / 601: the preview must
    // show what the save really posts, which is the first.
    let w = world();
    day(&w, 10);
    let t = new_trip(&w);
    let e = entry(
        t,
        vec![
            line(w.banana, &w, "1", "1,003", None),
            line(w.apple, &w, "1", "1,003", None),
        ],
    );
    let preview = purchases::entry_preview(&w.env.ctx, &e).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let amounts = |parts: &mandi_services::core::ventures::EntryParts| -> Vec<i64> {
        parts
            .parts
            .iter()
            .map(|p| p.amount_paisa.rupees())
            .collect()
    };
    let shown = amounts(preview.parts.as_ref().expect("the lines work out"));
    assert_eq!(shown, [806, 600, 600]);
    assert_eq!(shown.iter().sum::<i64>(), 2_006);
    // What the save stores for the two purchases, added up per partner, is the same.
    let saved = purchases::entry_save(&w.env.ctx, &w.owner, &e).unwrap();
    let mut stored = [0_i64; 3];
    for done in &saved.saved {
        for (slot, part) in stored.iter_mut().zip(&purchase(&w, done.id).parts) {
            *slot += part.amount_paisa.rupees();
        }
    }
    assert_eq!(stored.to_vec(), shown);
}

#[test]
fn the_cash_question_is_asked_once_for_all_the_lines_together() {
    let w = world();
    day(&w, 10);
    let t = new_trip(&w);
    // Cash in hand is 50,000 (the world). Each line costs 30,000: one line alone is fine, two
    // together take cash to −10,000.
    let two = entry(
        t,
        vec![
            line(w.banana, &w, "30", "1,000", None),
            line(w.apple, &w, "30", "1,000", None),
        ],
    );
    let preview = purchases::entry_preview(&w.env.ctx, &two).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(
        preview.cash.needs_confirm,
        "both lines together go below zero"
    );
    assert_eq!(preview.cash.cash_after_paisa.rupees(), -10_000);
    // Each line shows only its own part, but the note is for the entry.
    let refused = purchases::entry_save(&w.env.ctx, &w.owner, &two).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrCashBelowZeroNotConfirmed);
    assert!(
        purchases::list(
            &w.env.ctx,
            &mandi_services::core::purchases::PurchasesListInput::default()
        )
        .unwrap()
        .rows
        .is_empty()
    );
    let mut confirmed = two;
    confirmed.confirm_cash_below_zero = true;
    assert_eq!(
        purchases::entry_save(&w.env.ctx, &w.owner, &confirmed)
            .unwrap()
            .saved
            .len(),
        2
    );
}

#[test]
fn a_mistake_in_any_line_saves_nothing_and_points_at_the_line_and_the_field() {
    let w = world();
    day(&w, 10);
    let t = new_trip(&w);
    let mut bad = line(w.apple, &w, "0", "900", None);
    bad.rate = String::new();
    let e = entry(
        t,
        vec![
            line(w.banana, &w, "10", "500", None),
            line(w.apple, &w, "10", "500", Some("11")),
            bad,
        ],
    );
    let preview = purchases::entry_preview(&w.env.ctx, &e).unwrap();
    let fields: Vec<&str> = preview.errors.iter().map(|e| e.field.as_str()).collect();
    assert!(fields.contains(&"lines[1].other_qty"), "{fields:?}");
    assert!(fields.contains(&"lines[2].qty"), "{fields:?}");
    assert!(fields.contains(&"lines[2].rate"), "{fields:?}");
    let split_error = preview
        .errors
        .iter()
        .find(|e| e.field == "lines[1].other_qty")
        .unwrap();
    assert_eq!(split_error.code, ErrorCode::ErrSplitAboveQty);
    assert!(
        preview.lines[0].numbers.is_some(),
        "a good line still works out"
    );
    assert!(preview.lines[1].numbers.is_none());
    let error = purchases::entry_save(&w.env.ctx, &w.owner, &e).unwrap_err();
    assert!(has(
        &error,
        ErrorCode::ErrSplitAboveQty,
        "lines[1].other_qty"
    ));
    assert!(
        purchases::list(
            &w.env.ctx,
            &mandi_services::core::purchases::PurchasesListInput::default()
        )
        .unwrap()
        .rows
        .is_empty(),
        "nothing was saved"
    );
    // The shared fields are listed once, not once for each line.
    let mut closed = entry(
        t,
        vec![
            line(w.banana, &w, "1", "500", None),
            line(w.apple, &w, "1", "500", None),
        ],
    );
    closed.date = "01-01-2020".into();
    let shared = purchases::entry_preview(&w.env.ctx, &closed).unwrap();
    assert_eq!(
        shared.errors.iter().filter(|e| e.field == "date").count(),
        1
    );
    // No line, and too many lines.
    let none = purchases::entry_preview(&w.env.ctx, &entry(t, Vec::new())).unwrap();
    assert!(
        none.errors
            .iter()
            .any(|e| e.field == "lines" && e.code == ErrorCode::ErrRequired)
    );
    let many: Vec<PurchaseLineInput> = (0..101)
        .map(|_| line(w.banana, &w, "1", "500", None))
        .collect();
    let big = purchases::entry_preview(&w.env.ctx, &entry(t, many)).unwrap();
    assert!(
        big.errors
            .iter()
            .any(|e| e.field == "lines" && e.code == ErrorCode::ErrTooManyLines)
    );
}

// ---- R3, R4: the split ----

#[test]
fn ready_lots_take_only_our_shops_part_and_two_open_sales_cannot_oversell_it() {
    let w = world();
    day(&w, 10);
    let t = new_trip(&w);
    let saved = purchases::entry_save(
        &w.env.ctx,
        &w.owner,
        &entry(t, vec![line(w.banana, &w, "80", "50", Some("30"))]),
    )
    .unwrap();
    let id = saved.saved[0].id;
    // The ready list gives the 50 for our shop, not the 80.
    let ready = bikri::ready_goods(
        &w.env.ctx,
        &ReadyGoodsInput {
            date: "10-10-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(ready.rows.len(), 1);
    assert_eq!((ready.rows[0].qty, ready.rows[0].qty_left), (80, 50));
    // An own lot of 51 is refused with what our shop has.
    let too_many = support::preview(
        &w,
        &draft_on("10-10-2026", vec![own_lot(&w, id, "51", "60")]),
    );
    assert!(support::has(
        &too_many.errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[0].qty_received"
    ));
    // 50 are fine; the cost is C21 over ALL 80 (cost 4,000): 50 take 4,000 × 50 / 80 = 2,500.
    let ok = support::preview(
        &w,
        &draft_on("10-10-2026", vec![own_lot(&w, id, "50", "60")]),
    );
    assert!(ok.errors.is_empty(), "{:?}", ok.errors);
    let cost = ok.lots[0].own.as_ref().unwrap().cost_paisa;
    assert_eq!(
        cost.rupees(),
        2_500,
        "the last نگ of our part do not take the other part's cost"
    );
    // Two sales open at once on the same goods: the second is refused on qty.
    let first = draft_on("10-10-2026", vec![own_lot(&w, id, "30", "60")]);
    let second = draft_on("10-10-2026", vec![own_lot(&w, id, "30", "60")]);
    save_sale(&w, &first).unwrap();
    let error = save_sale(&w, &second).unwrap_err();
    assert!(
        has(&error, ErrorCode::ErrQtyExceedsLot, "lots[0].qty_received"),
        "{error:?}"
    );
    let after = purchase(&w, id);
    assert_eq!(
        (after.qty_our_sold, after.qty_our_left, after.qty_other_left),
        (30, 20, 30)
    );
    // Once our shop's 50 are sold, nothing is ready, though 30 are left for the other shop.
    save_sale(
        &w,
        &draft_on("10-10-2026", vec![own_lot(&w, id, "20", "60")]),
    )
    .unwrap();
    assert!(
        bikri::ready_goods(
            &w.env.ctx,
            &ReadyGoodsInput {
                date: "10-10-2026".into()
            }
        )
        .unwrap()
        .rows
        .is_empty()
    );
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(stock.own_rows.len(), 1);
    let row = &stock.own_rows[0];
    assert_eq!(
        (row.balance, row.qty_our_left, row.qty_other_left),
        (30, 0, 30)
    );
    assert_eq!(row.shop_name.as_deref(), Some("Haji Shop"));
}

#[test]
fn the_split_changes_while_goods_are_left_and_never_below_what_a_part_sold() {
    let w = world();
    day(&w, 10);
    let t = new_trip(&w);
    let saved = purchases::entry_save(
        &w.env.ctx,
        &w.owner,
        &entry(t, vec![line(w.banana, &w, "80", "50", None)]),
    )
    .unwrap();
    let id = saved.saved[0].id;
    let split = |version: i64, other: &str| {
        purchases::change_split(
            &w.env.ctx,
            &w.owner,
            &SplitInput {
                id,
                version,
                other_qty: other.into(),
                shop_name: Some("Haji Shop".into()),
            },
        )
    };
    let v = purchase(&w, id).version;
    let set = split(v, "30").unwrap();
    assert_eq!(
        (set.qty_other, set.qty_our_left, set.qty_other_left),
        (30, 50, 30)
    );
    assert_eq!(set.shop_name.as_deref(), Some("Haji Shop"));
    // Above the qty.
    assert!(has(
        &split(set.version, "81").unwrap_err(),
        ErrorCode::ErrSplitAboveQty,
        "other_qty"
    ));
    // Our shop sells 40: the other part may not take more than 80 − 40 = 40.
    save_sale(
        &w,
        &draft_on("10-10-2026", vec![own_lot(&w, id, "40", "60")]),
    )
    .unwrap();
    let v = purchase(&w, id).version;
    let e = split(v, "41").unwrap_err();
    assert!(has(&e, ErrorCode::ErrSplitBelowSold, "other_qty"));
    // The most other shops can have is 40; the least is what they sold (0).
    let v = purchase(&w, id).version;
    assert_eq!(split(v, "40").unwrap().qty_our_left, 0);
    // An old version is refused.
    assert!(has(
        &split(v, "10").unwrap_err(),
        ErrorCode::ErrVersionChanged,
        "version"
    ));
}

// ---- R5, D1: a sale in another shop ----

#[test]
fn a_sale_in_another_shop_takes_the_other_part_first_and_moves_the_rest_and_a_cancel_moves_it_back()
{
    let w = world();
    day(&w, 10);
    let t = new_trip(&w);
    // 80 at 50 (cost 4,000), 30 for another shop.
    let saved = purchases::entry_save(
        &w.env.ctx,
        &w.owner,
        &entry(t, vec![line(w.banana, &w, "80", "50", Some("30"))]),
    )
    .unwrap();
    let id = saved.saved[0].id;
    // 20 fit in the other part: nothing moves.
    let small = other_sale(&w, t, id, "20", "1,200");
    let p = trips::other_sale_preview(&w.env.ctx, &small).unwrap();
    let n = p.numbers.unwrap();
    assert_eq!(
        (
            n.qty_from_our_shop,
            n.qty_other_left_after,
            n.qty_left_after
        ),
        (0, 10, 60)
    );
    assert_eq!(n.cost_paisa.rupees(), 1_000, "C21: 4,000 × 20 / 80");
    let small_saved = trips::other_sale_save(&w.env.ctx, &w.owner, &small).unwrap();
    // 25 more: the other part has 10, so 15 move from our shop's part. The cost is C21 over all
    // 60 left: 3,000 × 25 / 60 = 1,250.
    let big = other_sale(&w, t, id, "25", "2,000");
    let p = trips::other_sale_preview(&w.env.ctx, &big)
        .unwrap()
        .numbers
        .unwrap();
    assert_eq!((p.qty_from_our_shop, p.qty_other_left_after), (15, 0));
    assert_eq!(
        (p.qty_our_left_after, p.qty_left_after),
        (35, 35),
        "our part 50 less the 15 moved; 60 left less the 25 sold"
    );
    assert_eq!(p.cost_paisa.rupees(), 1_250);
    let big_saved = trips::other_sale_save(&w.env.ctx, &w.owner, &big).unwrap();
    let row = purchase(&w, id);
    assert_eq!(
        (
            row.qty_other,
            row.qty_other_sold,
            row.qty_our_left,
            row.qty_other_left
        ),
        (45, 45, 35, 0)
    );
    // More than all that is left is refused with the whole qty left.
    let too_many = other_sale(&w, t, id, "36", "100");
    let e = trips::other_sale_save(&w.env.ctx, &w.owner, &too_many).unwrap_err();
    assert!(has(&e, ErrorCode::ErrQtyExceedsLot, "qty"));
    // The ready lot now has 35.
    let ready = bikri::ready_goods(
        &w.env.ctx,
        &ReadyGoodsInput {
            date: "10-10-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(ready.rows[0].qty_left, 35);
    // Cancelling the big sale gives back 25 to the other part and moves the 15 back to our shop.
    trips::other_sale_cancel(
        &w.env.ctx,
        &w.owner,
        &TripEntryCancelInput {
            id: big_saved.id,
            version: big_saved.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let back = purchase(&w, id);
    assert_eq!(
        (
            back.qty_other,
            back.qty_other_sold,
            back.qty_our_left,
            back.qty_other_left
        ),
        (30, 20, 50, 10)
    );
    let _ = small_saved;
}

// ---- A write-off comes off our shop's part first ----

#[test]
fn a_write_off_comes_off_our_shops_part_first_then_the_other_part() {
    let w = world();
    day(&w, 10);
    let t = new_trip(&w);
    let saved = purchases::entry_save(
        &w.env.ctx,
        &w.owner,
        &entry(t, vec![line(w.banana, &w, "20", "50", Some("5"))]),
    )
    .unwrap();
    let id = saved.saved[0].id;
    save_sale(
        &w,
        &draft_on("10-10-2026", vec![own_lot(&w, id, "10", "60")]),
    )
    .unwrap();
    // Our part: 15, sold 10, left 5. Other part 5. Write off 7: 5 from ours, 2 from the other.
    purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: id,
            qty: "7".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    let row = purchase(&w, id);
    assert_eq!(
        (row.qty_left, row.qty_our_left, row.qty_other_left),
        (3, 0, 3)
    );
    // The ready list no longer has it; the split can still move within its range.
    assert!(
        bikri::ready_goods(
            &w.env.ctx,
            &ReadyGoodsInput {
                date: "10-10-2026".into()
            }
        )
        .unwrap()
        .rows
        .is_empty()
    );
}

// ---- R2, R5, D1: garden goods arrived, many lines, and a sale in another shop ----

fn arrival_line(item: i64, w: &World, qty: &str, other: Option<&str>) -> ArrivalLineInput {
    ArrivalLineInput {
        key: format!("k-{item}"),
        item_id: Some(item),
        packing_id: Some(w.box_),
        qty: qty.into(),
        weight: None,
        note: None,
        other_qty: other.map(str::to_owned),
        shop_name: other.map(|_| "Haji Shop".to_owned()),
    }
}

fn arrivals_entry(garden: i64, lines: Vec<ArrivalLineInput>) -> ArrivalEntryInput {
    ArrivalEntryInput {
        garden_id: Some(garden),
        date: "10-10-2026".into(),
        sell_date: None,
        lines,
    }
}

fn garden_sale(garden: i64, arrival: Option<i64>, qty: &str, money: &str) -> GardenOtherSaleInput {
    GardenOtherSaleInput {
        garden_id: Some(garden),
        date: "10-10-2026".into(),
        shop: Some("Haji Shop".into()),
        arrival_id: arrival,
        qty: qty.into(),
        amount: money.into(),
        note: None,
    }
}

#[test]
fn a_goods_arrived_entry_holds_many_lines_and_each_is_its_own_arrival() {
    let w = world();
    day(&w, 10);
    let g = new_garden(&w);
    let e = arrivals_entry(
        g.id,
        vec![
            arrival_line(w.banana, &w, "100", Some("40")),
            arrival_line(w.apple, &w, "30", None),
        ],
    );
    let preview = gardens::arrivals::entry_preview(&w.env.ctx, &e).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let saved = gardens::arrivals::entry_save(&w.env.ctx, &w.owner, &e).unwrap();
    let rows: Vec<&ArrivalRow> = saved.rows.iter().collect();
    assert_eq!(rows.iter().map(|r| r.number).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(
        (
            rows[0].qty_other,
            rows[0].qty_our_left,
            rows[0].qty_other_left
        ),
        (40, 60, 40)
    );
    assert_eq!((rows[1].qty_other, rows[1].qty_our_left), (0, 30));
    // The ready list takes only our shop's part: 60 and 30.
    let ready = bikri::ready_goods(
        &w.env.ctx,
        &ReadyGoodsInput {
            date: "10-10-2026".into(),
        },
    )
    .unwrap();
    let left: Vec<i64> = ready.rows.iter().map(|r| r.qty_left).collect();
    assert_eq!(left, [60, 30]);
    // A bad line saves nothing and points at the line.
    let bad = arrivals_entry(
        g.id,
        vec![
            arrival_line(w.banana, &w, "10", None),
            arrival_line(w.apple, &w, "5", Some("9")),
        ],
    );
    let error = gardens::arrivals::entry_save(&w.env.ctx, &w.owner, &bad).unwrap_err();
    assert!(has(
        &error,
        ErrorCode::ErrSplitAboveQty,
        "lines[1].other_qty"
    ));
    let list = gardens::arrivals::list(
        &w.env.ctx,
        &mandi_services::core::arrivals::ArrivalsListInput::default(),
    )
    .unwrap();
    assert_eq!(list.total_count, 2, "the bad entry saved nothing");
    // No line.
    let none =
        gardens::arrivals::entry_preview(&w.env.ctx, &arrivals_entry(g.id, Vec::new())).unwrap();
    assert!(
        none.errors
            .iter()
            .any(|e| e.field == "lines" && e.code == ErrorCode::ErrRequired)
    );
}

#[test]
fn a_garden_sale_in_another_shop_names_the_arrival_and_takes_the_other_part_first() {
    let w = world();
    day(&w, 10);
    let g = new_garden(&w);
    let hand = gardens::entry_preview(
        &w.env.ctx,
        &GardenEntryInput::OtherSale(garden_sale(g.id, None, "5", "1,000")),
    )
    .unwrap();
    assert!(
        hand.errors.is_empty(),
        "a garden with no arrival is typed by hand as before"
    );
    let saved = gardens::arrivals::entry_save(
        &w.env.ctx,
        &w.owner,
        &arrivals_entry(g.id, vec![arrival_line(w.banana, &w, "100", Some("30"))]),
    )
    .unwrap();
    let a = saved.rows[0].id;
    // The garden has an arrival now: the sale must name one.
    let e = gardens::entry_preview(
        &w.env.ctx,
        &GardenEntryInput::OtherSale(garden_sale(g.id, None, "5", "1,000")),
    )
    .unwrap();
    assert!(
        e.errors
            .iter()
            .any(|x| x.field == "arrival_id" && x.code == ErrorCode::ErrRequired)
    );
    // 50 are sold in another shop: the other part has 30, so 20 move from our shop's part.
    let sale = GardenEntryInput::OtherSale(garden_sale(g.id, Some(a), "50", "20,000"));
    let numbers = gardens::entry_preview(&w.env.ctx, &sale)
        .unwrap()
        .numbers
        .expect("the sale works out");
    assert_eq!(
        (
            numbers.qty_from_our_shop,
            numbers.qty_other_left_after,
            numbers.qty_our_left_after,
            numbers.qty_left_after
        ),
        (Some(20), Some(0), Some(50), Some(50))
    );
    let done = gardens::entry_save(&w.env.ctx, &w.owner, &sale).unwrap();
    let row = gardens::arrivals::get(
        &w.env.ctx,
        &mandi_services::core::arrivals::ArrivalGetInput { id: a },
    )
    .unwrap();
    assert_eq!(
        (
            row.qty_other,
            row.qty_other_sold,
            row.qty_our_left,
            row.qty_other_left,
            row.qty_left
        ),
        (50, 50, 50, 0, 50)
    );
    // Above all that is left.
    let e = gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::OtherSale(garden_sale(g.id, Some(a), "51", "1,000")),
    )
    .unwrap_err();
    assert!(has(&e, ErrorCode::ErrQtyExceedsLot, "qty"));
    // Cancel gives everything back, including what was moved.
    gardens::entry_cancel(
        &w.env.ctx,
        &w.owner,
        &GardenEntryCancelInput {
            id: done.id,
            version: done.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let back = gardens::arrivals::get(
        &w.env.ctx,
        &mandi_services::core::arrivals::ArrivalGetInput { id: a },
    )
    .unwrap();
    assert_eq!(
        (
            back.qty_other,
            back.qty_other_sold,
            back.qty_our_left,
            back.qty_other_left
        ),
        (30, 0, 70, 30)
    );
    // The arrival of another garden is not accepted.
    let other = new_second_garden(&w);
    let e = gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::OtherSale(garden_sale(other, Some(a), "1", "100")),
    );
    assert!(e.is_err());
    // Written off for good: nothing is left to sell anywhere.
    gardens::arrivals::write_off(
        &w.env.ctx,
        &w.owner,
        &ArrivalWriteOffInput {
            arrival_id: a,
            qty: "100".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
}

fn new_second_garden(w: &World) -> i64 {
    gardens::create(
        &w.env.ctx,
        &w.owner,
        &GardenInput {
            name_ur: "ملتان باغ".into(),
            name_en: "Multan Garden".into(),
            confirmed_rules_words: vec![
                "ملتان".into(),
                "باغ".into(),
                "Multan".into(),
                "Garden".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            rent: "50,000".into(),
            season_end: None,
            note: None,
            partners: partners_of(w),
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .row
    .id
}

// ---- D2: the bell counts only our shop's goods ----

#[test]
fn the_bell_counts_only_the_goods_for_our_shop() {
    use mandi_reports::dashboard::{AttentionKind, BackupFacts};
    let w = world();
    day(&w, 10);
    let t = new_trip(&w);
    purchases::entry_save(
        &w.env.ctx,
        &w.owner,
        &entry(t, vec![line(w.banana, &w, "20", "50", Some("20"))]),
    )
    .unwrap();
    let waiting = |w: &World| {
        mandi_reports::dashboard::attention(
            w.env.ctx.db(),
            BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        )
        .unwrap()
        .items
        .into_iter()
        .find(|i| i.kind == AttentionKind::StockWaiting)
    };
    // All 20 are for another shop: nothing waits in our shop, however old.
    day(&w, 20);
    assert!(waiting(&w).is_none());
}
