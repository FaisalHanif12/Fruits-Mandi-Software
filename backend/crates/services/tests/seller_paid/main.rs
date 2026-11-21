//! A seller's money paid on the spot (موقع پر ادائیگی, PRD FR-12.23 v1.53, plan 91).
//!
//! No new posting: the money is the نقد of the seller's lots. These tests check the summary lines of
//! the preview (previous balance, today's credit, paid, cut, new balance), the spread of an amount
//! over his lots, the question the save asks when the amount is above today's credit, the edit, the
//! mixed screen, and that every figure agrees with the ledger after the save.
//!
//! The worked numbers are those of the plan: previous 30,000 بنام, credit 20,000, paid 5,000 gives
//! 15,000 بنام; two lots 12,000 and 8,000 with 15,000 paid give 12,000 and 3,000; previous 10,000
//! جمع, credit 20,000, paid 28,000 needs no question and leaves 2,000 جمع; previous 30,000 بنام,
//! credit 20,000, paid 25,000 asks and leaves 35,000 بنام.
//!
//! Run it with `cargo test -p mandi-services --test seller_paid`.
#![cfg(test)]

mod fixtures;

#[path = "../common/mod.rs"]
mod common;
#[allow(dead_code)]
#[path = "../bikri_preview/support.rs"]
mod support;

use mandi_services::bikri;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{
    BikriDraft, BikriPreview, BikriSaveInput, CarriedLotInput, EditOpenInput, EditPreviewInput,
    EditSaveInput, LineChargeInput, LineChargeMethod, LotInput, MixedSaleInput, MixedSaleSaveInput,
    SellerPaidInput, SellerPaidLines, SellerPaidSpread, SellerPaidSpreadInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::ReceiptInput;
use mandi_services::{ServiceError, ledger, vouchers};
use support::World;

use common::{raw, set_today};

// ---- helpers ----

/// A lot of `seller` with no charge and no advance tick: `qty` نگ received and sold to اکرم at
/// `rate`, the seller commission typed as 0% so the credit is exactly `qty × rate`. `cash` is the
/// نقد typed on the lot.
fn credit_lot(
    w: &World,
    key: &str,
    seller: i64,
    qty: &str,
    rate: &str,
    cash: Option<&str>,
) -> LotInput {
    let mut sale = support::sale(&format!("{key}-s"), w.akram, qty, rate);
    sale.seller_comm = Some(LineChargeInput {
        method: LineChargeMethod::Percent,
        value: "0".into(),
    });
    let mut lot = support::with_sales(support::bare_lot(w, key, seller, w.banana, qty), vec![sale]);
    lot.adjust_advance = false;
    lot.cash_paid = cash.map(str::to_owned);
    lot
}

/// One lot of رحیم: credit `rupees` (as `rupees / 1000` نگ at 1,000), `cash` given on the lot.
fn rahim_draft(w: &World, thousands: i64, cash: Option<&str>) -> BikriDraft {
    support::draft(vec![credit_lot(
        w,
        "lot-1",
        w.rahim,
        &thousands.to_string(),
        "1000",
        cash,
    )])
}

fn save_confirmed(
    w: &World,
    draft: &BikriDraft,
    confirm: bool,
) -> Result<mandi_services::core::bikri::BikriSaved, ServiceError> {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: confirm,
        },
    )
}

fn balance(w: &World, account: i64) -> (BalanceSide, i64) {
    let b = ledger::account_balance(&w.env.ctx, account).unwrap();
    (b.side, b.amount_paisa.rupees())
}

fn cash(w: &World) -> (BalanceSide, i64) {
    balance(w, support::id_of(&w.env, "01010001"))
}

/// A money-rich world, so that no test trips the cash question by accident.
fn rich_world() -> World {
    support::world_with(1_000_000, true)
}

/// رحیم owes the shop `rupees` (an advance in cash on 10-10-2026).
fn owes(w: &World, rupees: &str) {
    owes_on(w, rupees, "10-10-2026");
}

/// رحیم owes the shop `rupees` since `date` (an advance in cash).
fn owes_on(w: &World, rupees: &str, date: &str) {
    mandi_services::vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::AdvanceInput {
            confirm_cash_below_zero: true,
            date: date.into(),
            seller_account_id: w.rahim,
            amount: rupees.into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
            reason: None,
        },
    )
    .unwrap();
}

fn seller(p: &BikriPreview, id: i64) -> &SellerPaidLines {
    p.sellers
        .iter()
        .find(|s| s.seller_account_id == id)
        .unwrap_or_else(|| panic!("no summary for {id}: {:?}", p.sellers))
}

fn spread(
    w: &World,
    draft: &BikriDraft,
    entry_id: Option<i64>,
    asked: &[(i64, &str)],
) -> SellerPaidSpread {
    bikri::seller_paid_spread(
        &w.env.ctx,
        &SellerPaidSpreadInput {
            draft: draft.clone(),
            entry_id,
            seller_paid: asked
                .iter()
                .map(|(seller_account_id, amount)| SellerPaidInput {
                    seller_account_id: *seller_account_id,
                    amount: (*amount).into(),
                })
                .collect(),
        },
    )
    .unwrap()
}

fn rupees_of(p: Paisa) -> i64 {
    p.rupees()
}

fn has(list: &[mandi_services::core::error::FieldError], code: ErrorCode, field: &str) -> bool {
    support::has(list, code, field)
}

// ---- the summary ----

#[test]
fn case_a_previous_30000_credit_20000_paid_5000_cut_15000_new_15000() {
    let w = rich_world();
    owes(&w, "30,000");
    let before = cash(&w);
    let draft = rahim_draft(&w, 20, Some("5000"));
    let preview = support::preview(&w, &draft);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let s = seller(&preview, w.rahim);
    assert_eq!(
        (rupees_of(s.previous_paisa), s.previous_side),
        (30_000, BalanceSide::Dr)
    );
    assert_eq!(rupees_of(s.credit_paisa), 20_000);
    assert_eq!(rupees_of(s.paid_paisa), 5_000);
    assert_eq!(rupees_of(s.cut_paisa), 15_000);
    assert_eq!(rupees_of(s.added_to_credit_paisa), 0);
    assert_eq!(rupees_of(s.over_paid_paisa), 0);
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (15_000, BalanceSide::Dr)
    );
    assert_eq!(rupees_of(s.limit_paisa), 20_000);
    assert!(!s.needs_confirm);
    // The cash of the entry: 5,000 goes out.
    assert_eq!(preview.cash.change_paisa.rupees(), -5_000);
    // The save writes what the preview showed: the seller's ledger agrees with the summary, cash
    // went down by the amount paid, and the entry balanced (a save would be refused otherwise).
    let saved = save_confirmed(&w, &draft, false).unwrap();
    assert_eq!(
        saved.preview.sellers, preview.sellers,
        "the save shows the preview's summary"
    );
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 15_000));
    let after = cash(&w);
    assert_eq!(before.1 - after.1, 5_000);
}

#[test]
fn case_b_two_lots_with_15000_give_12000_and_3000_and_the_save_keeps_them() {
    let w = rich_world();
    let draft = support::draft(vec![
        credit_lot(&w, "lot-1", w.rahim, "12", "1000", None),
        credit_lot(&w, "lot-2", w.rahim, "8", "1000", None),
    ]);
    let out = spread(&w, &draft, None, &[(w.rahim, "15000")]);
    let parts: Vec<(&str, i64)> = out
        .lots
        .iter()
        .map(|l| (l.key.as_str(), rupees_of(l.cash_paid_paisa)))
        .collect();
    assert_eq!(parts, vec![("lot-1", 12_000), ("lot-2", 3_000)]);
    // The preview has those نقد in place and one summary for him.
    assert!(out.preview.errors.is_empty(), "{:?}", out.preview.errors);
    let totals: Vec<i64> = out
        .preview
        .lots
        .iter()
        .map(|l| rupees_of(l.totals.as_ref().unwrap().cash_paid))
        .collect();
    assert_eq!(totals, vec![12_000, 3_000]);
    assert_eq!(out.preview.sellers.len(), 1);
    let s = &out.preview.sellers[0];
    assert_eq!(rupees_of(s.paid_paisa), 15_000);
    assert_eq!(rupees_of(s.credit_paisa), 20_000);
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (5_000, BalanceSide::Cr)
    );
    assert!(!s.needs_confirm);
    // The screen writes the values into the lot fields; the save takes them as typed.
    let mut typed = draft.clone();
    for (lot, part) in typed.lots.iter_mut().zip(&out.lots) {
        lot.cash_paid = Some(rupees_of(part.cash_paid_paisa).to_string());
    }
    let saved = save_confirmed(&w, &typed, false).unwrap();
    assert_eq!(saved.preview.sellers, out.preview.sellers);
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 5_000));
    // Each lot kept its own نقد.
    let conn = raw(&w.dir);
    let stored: Vec<i64> = conn
        .prepare("SELECT cash_paid_paisa FROM bikri_lots ORDER BY lot_no")
        .unwrap()
        .query_map([], |r| r.get::<_, i64>(0))
        .unwrap()
        .map(|r| r.unwrap() / 100)
        .collect();
    assert_eq!(stored, vec![12_000, 3_000]);
}

#[test]
fn the_rest_of_the_amount_goes_on_the_last_lot_even_above_its_credit() {
    let w = rich_world();
    let draft = support::draft(vec![
        credit_lot(&w, "lot-1", w.rahim, "12", "1000", None),
        credit_lot(&w, "lot-2", w.rahim, "8", "1000", None),
    ]);
    let out = spread(&w, &draft, None, &[(w.rahim, "30000")]);
    let parts: Vec<i64> = out
        .lots
        .iter()
        .map(|l| rupees_of(l.cash_paid_paisa))
        .collect();
    assert_eq!(parts, vec![12_000, 18_000]);
    assert_eq!(rupees_of(out.preview.sellers[0].paid_paisa), 30_000);
    assert!(out.preview.sellers[0].needs_confirm);
}

#[test]
fn case_c_the_shop_owed_him_10000_credit_20000_paid_28000_needs_no_question() {
    let w = rich_world();
    // An earlier بکری of the same day leaves the shop owing him 10,000 (credit 10,000, no cash).
    save_confirmed(&w, &rahim_draft(&w, 10, None), false).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 10_000));
    let draft = rahim_draft(&w, 20, Some("28000"));
    let preview = support::preview(&w, &draft);
    let s = seller(&preview, w.rahim);
    assert_eq!(
        (rupees_of(s.previous_paisa), s.previous_side),
        (10_000, BalanceSide::Cr)
    );
    assert_eq!(rupees_of(s.over_paid_paisa), 8_000);
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (2_000, BalanceSide::Cr)
    );
    assert_eq!(rupees_of(s.limit_paisa), 30_000);
    assert!(!s.needs_confirm);
    // No question, so the save needs no confirmation.
    save_confirmed(&w, &draft, false).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 2_000));
}

#[test]
fn case_d_above_todays_credit_the_save_asks_and_writes_nothing_until_confirmed() {
    let w = rich_world();
    owes(&w, "30,000");
    let draft = rahim_draft(&w, 20, Some("25000"));
    let preview = support::preview(&w, &draft);
    let s = seller(&preview, w.rahim);
    assert_eq!(rupees_of(s.over_paid_paisa), 5_000);
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (35_000, BalanceSide::Dr)
    );
    assert_eq!(rupees_of(s.limit_paisa), 20_000);
    assert!(s.needs_confirm);
    // The preview only warns: it has no error for it.
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let counts_before = rows_written(&w);
    let refused = save_confirmed(&w, &draft, false).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("expected a rejection");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].code, ErrorCode::ErrSellerPaidAboveToday);
    assert_eq!(items[0].field.as_deref(), Some("sellers[0].paid"));
    let value = |name: &str| match items[0].values.get(name) {
        Some(mandi_services::core::error::ErrorValue::Number(n)) => *n,
        other => panic!("{name}: {other:?}"),
    };
    assert_eq!(value("credit_paisa"), 2_000_000);
    assert_eq!(value("limit_paisa"), 2_000_000);
    assert_eq!(value("paid_paisa"), 2_500_000);
    assert_eq!(rows_written(&w), counts_before, "nothing was written");
    // Confirmed, it saves and the ledger shows 35,000 بنام.
    save_confirmed(&w, &draft, true).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 35_000));
}

fn rows_written(w: &World) -> Vec<i64> {
    let conn = raw(&w.dir);
    [
        "bikri_entries",
        "bikri_lots",
        "journal_entries",
        "journal_lines",
        "audit_log",
    ]
    .iter()
    .map(|t| {
        conn.query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| r.get(0))
            .unwrap()
    })
    .collect()
}

#[test]
fn event_1_still_gives_the_worked_day_numbers_with_his_open_advance() {
    let w = support::world();
    owes(&w, "15,000");
    let preview = support::preview(&w, &support::event1(&w));
    // E2E-1 unchanged.
    assert_eq!(rupees_of(preview.summary.gross), 29_400);
    assert_eq!(rupees_of(preview.summary.deductions), 10_084);
    assert_eq!(rupees_of(preview.summary.net), 19_316);
    let s = seller(&preview, w.rahim);
    assert_eq!(
        (rupees_of(s.previous_paisa), s.previous_side),
        (15_000, BalanceSide::Dr)
    );
    assert_eq!(rupees_of(s.credit_paisa), 24_316);
    assert_eq!(rupees_of(s.paid_paisa), 5_000);
    assert_eq!(rupees_of(s.cut_paisa), 15_000);
    assert_eq!(rupees_of(s.added_to_credit_paisa), 4_316);
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (4_316, BalanceSide::Cr)
    );
    assert!(!s.needs_confirm);
    save_confirmed(&w, &support::event1(&w), false).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 4_316));
}

// ---- the spread command ----

#[test]
fn an_empty_amount_is_ignored_and_zero_puts_zero_on_every_lot() {
    let w = rich_world();
    let draft = support::draft(vec![
        credit_lot(&w, "lot-1", w.rahim, "12", "1000", Some("700")),
        credit_lot(&w, "lot-2", w.rahim, "8", "1000", Some("300")),
    ]);
    // Empty: nothing is placed and the lots keep the نقد they have.
    let out = spread(&w, &draft, None, &[(w.rahim, "  ")]);
    assert!(out.lots.is_empty());
    assert_eq!(rupees_of(out.preview.sellers[0].paid_paisa), 1_000);
    // Zero: 0 on all his lots.
    let out = spread(&w, &draft, None, &[(w.rahim, "0")]);
    let parts: Vec<i64> = out
        .lots
        .iter()
        .map(|l| rupees_of(l.cash_paid_paisa))
        .collect();
    assert_eq!(parts, vec![0, 0]);
    assert_eq!(rupees_of(out.preview.sellers[0].paid_paisa), 0);
}

#[test]
fn a_sum_is_taken_and_only_the_named_seller_changes() {
    let w = rich_world();
    let draft = support::draft(vec![
        credit_lot(&w, "lot-1", w.rahim, "12", "1000", None),
        credit_lot(&w, "lot-2", w.karim, "8", "1000", Some("300")),
    ]);
    let out = spread(&w, &draft, None, &[(w.rahim, "5000+2000")]);
    let parts: Vec<(&str, i64)> = out
        .lots
        .iter()
        .map(|l| (l.key.as_str(), rupees_of(l.cash_paid_paisa)))
        .collect();
    assert_eq!(parts, vec![("lot-1", 7_000)]);
    // Karim keeps his own نقد.
    assert_eq!(rupees_of(seller(&out.preview, w.karim).paid_paisa), 300);
    assert_eq!(rupees_of(seller(&out.preview, w.rahim).paid_paisa), 7_000);
}

#[test]
fn a_seller_with_no_lot_that_takes_cash_is_refused_at_his_field() {
    let w = rich_world();
    let draft = rahim_draft(&w, 20, None);
    // Karim has no lot in this بکری.
    let out = spread(&w, &draft, None, &[(w.karim, "100")]);
    assert!(has(
        &out.preview.errors,
        ErrorCode::ErrFormatInvalid,
        "seller_paid[0].seller_account_id"
    ));
    assert!(out.lots.is_empty());
    // 0 for him is nothing to do.
    let out = spread(&w, &draft, None, &[(w.karim, "0")]);
    assert!(out.preview.errors.is_empty(), "{:?}", out.preview.errors);
}

#[test]
fn a_lot_that_is_not_complete_yet_takes_no_cash_and_the_request_says_so() {
    let w = rich_world();
    let mut lot = credit_lot(&w, "lot-1", w.rahim, "20", "1000", None);
    lot.item_id = None;
    let out = spread(&w, &support::draft(vec![lot]), None, &[(w.rahim, "100")]);
    assert!(has(
        &out.preview.errors,
        ErrorCode::ErrFormatInvalid,
        "seller_paid[0].seller_account_id"
    ));
    assert!(out.preview.sellers.is_empty());
}

#[test]
fn the_same_seller_twice_a_bad_amount_and_a_negative_amount_are_refused_at_their_fields() {
    let w = rich_world();
    let draft = rahim_draft(&w, 20, None);
    let out = spread(
        &w,
        &draft,
        None,
        &[(w.rahim, "100"), (w.rahim, "200"), (w.karim, "abc")],
    );
    let errors = &out.preview.errors;
    assert!(has(
        errors,
        ErrorCode::ErrFormatInvalid,
        "seller_paid[1].seller_account_id"
    ));
    assert!(has(
        errors,
        ErrorCode::ErrNotANumber,
        "seller_paid[2].amount"
    ));
    // The same seller twice: the first entry stands, the second is the mistake.
    assert_eq!(rupees_of(seller(&out.preview, w.rahim).paid_paisa), 100);
    // A negative amount is refused at its field.
    let out = spread(&w, &draft, None, &[(w.rahim, "-5")]);
    assert!(
        has(
            &out.preview.errors,
            ErrorCode::ErrNegativeNotAllowed,
            "seller_paid[0].amount"
        ),
        "{:?}",
        out.preview.errors
    );
}

#[test]
fn a_trip_goods_seller_or_a_garden_is_not_a_seller_who_takes_cash() {
    // اپنا مال has no summary and takes no cash, so a request for it has no lot to go on.
    let w = rich_world();
    let own = support::id_of(&w.env, "030119998");
    let mut lot = credit_lot(&w, "lot-1", own, "20", "1000", None);
    lot.purchase_id = Some(999);
    let out = spread(&w, &support::draft(vec![lot]), None, &[(own, "100")]);
    assert!(out.preview.sellers.is_empty());
    assert!(has(
        &out.preview.errors,
        ErrorCode::ErrFormatInvalid,
        "seller_paid[0].seller_account_id"
    ));
}

// ---- previous balance, edit, two بکری a day ----

#[test]
fn two_bikri_of_one_day_the_second_starts_where_the_first_ended() {
    let w = rich_world();
    owes(&w, "30,000");
    save_confirmed(&w, &rahim_draft(&w, 10, Some("1000")), false).unwrap();
    // 30,000 + 1,000 - 10,000 = 21,000 بنام.
    let second = support::preview(&w, &rahim_draft(&w, 20, Some("5000")));
    let s = seller(&second, w.rahim);
    assert_eq!(
        (rupees_of(s.previous_paisa), s.previous_side),
        (21_000, BalanceSide::Dr)
    );
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (6_000, BalanceSide::Dr)
    );
}

#[test]
fn an_edit_shows_his_balance_before_that_posting_not_what_came_after_it() {
    let w = rich_world();
    owes(&w, "30,000");
    let draft = rahim_draft(&w, 20, Some("5000"));
    let saved = save_confirmed(&w, &draft, false).unwrap();
    // The same day he pays 15,000 back (a وصولی on his account): the ledger is now 0.
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "10-10-2026".into(),
            account_id: w.rahim,
            amount: "15000".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();
    assert_eq!(balance(&w, w.rahim).1, 0);
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    // Opened as saved, the lot keeps its نقد: the summary is that of the day he was paid.
    let preview = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: saved.entry_id,
            draft: opened.draft.clone(),
        },
    )
    .unwrap();
    let s = seller(&preview, w.rahim);
    assert_eq!(
        (rupees_of(s.previous_paisa), s.previous_side),
        (30_000, BalanceSide::Dr),
        "before this posting, not the ledger now"
    );
    assert_eq!(rupees_of(s.credit_paisa), 20_000);
    assert_eq!(rupees_of(s.paid_paisa), 5_000);
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (15_000, BalanceSide::Dr)
    );
    // The same through the spread command in its edit form: 8,000 now, spread on the one lot.
    let out = spread(
        &w,
        &opened.draft,
        Some(saved.entry_id),
        &[(w.rahim, "8000")],
    );
    assert_eq!(rupees_of(out.lots[0].cash_paid_paisa), 8_000);
    let s = seller(&out.preview, w.rahim);
    assert_eq!(rupees_of(s.previous_paisa), 30_000);
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (18_000, BalanceSide::Dr)
    );
}

#[test]
fn an_edit_that_leaves_the_amount_alone_keeps_each_lots_cash_and_an_edit_above_the_limit_asks() {
    let w = rich_world();
    owes(&w, "30,000");
    let draft = support::draft(vec![
        credit_lot(&w, "lot-1", w.rahim, "12", "1000", Some("4000")),
        credit_lot(&w, "lot-2", w.rahim, "8", "1000", Some("1000")),
    ]);
    let saved = save_confirmed(&w, &draft, false).unwrap();
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let cash_of = |d: &BikriDraft| -> Vec<Option<String>> {
        d.lots.iter().map(|l| l.cash_paid.clone()).collect()
    };
    // Nothing asked: the lots come back with the نقد they were saved with.
    assert_eq!(
        cash_of(&opened.draft)
            .iter()
            .map(|c| c.as_deref().map(|t| t.replace(',', "")))
            .collect::<Vec<_>>(),
        vec![Some("4000".to_owned()), Some("1000".to_owned())]
    );
    let preview = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: saved.entry_id,
            draft: opened.draft.clone(),
        },
    )
    .unwrap();
    assert_eq!(rupees_of(seller(&preview, w.rahim).paid_paisa), 5_000);
    // Now 25,000 is asked (above 20,000 credit with 30,000 owed): the edit asks before it saves.
    let out = spread(
        &w,
        &opened.draft,
        Some(saved.entry_id),
        &[(w.rahim, "25000")],
    );
    assert!(seller(&out.preview, w.rahim).needs_confirm);
    let mut typed = opened.draft.clone();
    for (lot, part) in typed.lots.iter_mut().zip(&out.lots) {
        lot.cash_paid = Some(rupees_of(part.cash_paid_paisa).to_string());
    }
    let edit = |confirm: bool| {
        bikri::edit_save(
            &w.env.ctx,
            &w.owner,
            &EditSaveInput {
                entry_id: saved.entry_id,
                version: opened.version,
                draft: typed.clone(),
                reason: Some("more paid".into()),
                confirm_cash_below_zero: confirm,
            },
        )
    };
    let refused = edit(false).unwrap_err();
    assert!(
        matches!(&refused, ServiceError::Rejected(items)
            if items.iter().any(|i| i.code == ErrorCode::ErrSellerPaidAboveToday)),
        "{refused:?}"
    );
    edit(true).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 35_000));
}

// ---- the mixed screen ----

#[test]
fn a_mixed_screen_gives_a_seller_with_a_carried_lot_and_a_new_lot_one_summary_that_matches_the_ledger()
 {
    let w = rich_world();
    // Event 1 with only 45 of 50 sold (E2E-16): رحیم has 5 نگ left and the shop owes him what the books
    // say after that بکری (read below, not typed).
    let first = support::draft(vec![support::with_sales(
        support::lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![
            support::sale("s-1", w.akram, "20", "600"),
            support::sale("s-2", w.arshad, "25", "580"),
        ],
    )]);
    let saved = save_confirmed(&w, &first, false).unwrap();
    set_today(&w.env, 2026, 10, 11);
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    let before = balance(&w, w.rahim);
    let input = MixedSaleInput {
        date: "11-10-2026".into(),
        carried: vec![CarriedLotInput {
            key: String::new(),
            lot_id,
            sales: vec![support::sale("c-1", w.arshad, "5", "560")],
        }],
        new_lots: vec![credit_lot(&w, "new-1", w.rahim, "10", "1000", Some("1000"))],
    };
    let preview = bikri::mixed_preview(&w.env.ctx, &input).unwrap();
    assert!(
        preview.preview.errors.is_empty(),
        "{:?}",
        preview.preview.errors
    );
    assert_eq!(
        preview.preview.sellers.len(),
        1,
        "one summary for one seller"
    );
    let s = &preview.preview.sellers[0];
    assert_eq!(s.seller_account_id, w.rahim);
    // He is owed what the first بکری left: previous is the ledger before the whole save.
    assert_eq!(
        (s.previous_paisa.rupees(), s.previous_side),
        (before.1, before.0)
    );
    // His credit is the later posting of the carried lot plus the new lot (nothing is rounded again).
    assert_eq!(rupees_of(s.credit_paisa), 2_632 + 10_000);
    assert_eq!(rupees_of(s.paid_paisa), 1_000);
    let saved = bikri::mixed_save(
        &w.env.ctx,
        &w.owner,
        &MixedSaleSaveInput {
            date: input.date.clone(),
            carried: input.carried.clone(),
            new_lots: input.new_lots.clone(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(saved.preview.sellers, preview.preview.sellers);
    // The ledger after the whole save is the summary's new balance.
    let after = balance(&w, w.rahim);
    assert_eq!((after.1, after.0), (s.new_paisa.rupees(), s.new_side));
}

#[test]
fn the_mixed_save_asks_for_a_seller_paid_above_todays_credit() {
    let w = rich_world();
    set_today(&w.env, 2026, 10, 10);
    let input = MixedSaleSaveInput {
        date: "10-10-2026".into(),
        carried: vec![],
        new_lots: vec![credit_lot(
            &w,
            "new-1",
            w.rahim,
            "10",
            "1000",
            Some("15000"),
        )],
        confirm_cash_below_zero: false,
    };
    let refused = bikri::mixed_save(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert!(
        matches!(&refused, ServiceError::Rejected(items)
            if items.iter().any(|i| i.code == ErrorCode::ErrSellerPaidAboveToday)),
        "{refused:?}"
    );
    let confirmed = MixedSaleSaveInput {
        confirm_cash_below_zero: true,
        ..input
    };
    bikri::mixed_save(&w.env.ctx, &w.owner, &confirmed).unwrap();
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Dr, 5_000));
}

// ---- the previous balance after earlier edits ----

/// Opens a posting for edit and saves it again with `change` applied to its draft. Returns the new
/// entry of the posting.
fn edit_again(
    w: &World,
    bikri_id: i64,
    entry_id: i64,
    change: &dyn Fn(&mut BikriDraft),
) -> mandi_services::core::bikri::BikriEdited {
    let opened = bikri::open_for_edit(&w.env.ctx, &EditOpenInput { bikri_id, entry_id }).unwrap();
    let mut draft = opened.draft.clone();
    change(&mut draft);
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id,
            version: opened.version,
            draft,
            reason: Some("again".into()),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
}

fn edit_preview_of(
    w: &World,
    bikri_id: i64,
    entry_id: i64,
) -> mandi_services::core::bikri::BikriPreview {
    let opened = bikri::open_for_edit(&w.env.ctx, &EditOpenInput { bikri_id, entry_id }).unwrap();
    bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id,
            draft: opened.draft,
        },
    )
    .unwrap()
}

#[test]
fn a_second_edit_shows_the_same_previous_balance_as_the_first() {
    let w = rich_world();
    owes_on(&w, "30,000", "09-10-2026");
    let saved = save_confirmed(&w, &rahim_draft(&w, 20, Some("5000")), false).unwrap();
    // The same day, after the بکری, he pays 3,000 back.
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "10-10-2026".into(),
            account_id: w.rahim,
            amount: "3000".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();
    let first = edit_preview_of(&w, saved.bikri_id, saved.entry_id);
    let s = seller(&first, w.rahim);
    assert_eq!(
        (rupees_of(s.previous_paisa), s.previous_side),
        (30_000, BalanceSide::Dr)
    );
    // Edit it (the note changes nothing in the money) and look again: the posting has a new entry
    // with a higher number than the payment, but it is the same posting.
    let edited = edit_again(&w, saved.bikri_id, saved.entry_id, &|d| {
        d.lots[0].sales[0].note = Some("changed".into());
    });
    assert!(edited.entry_id > saved.entry_id);
    let second = edit_preview_of(&w, saved.bikri_id, edited.entry_id);
    let s = seller(&second, w.rahim);
    assert_eq!(
        (rupees_of(s.previous_paisa), s.previous_side),
        (30_000, BalanceSide::Dr),
        "the same previous balance after an edit"
    );
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (15_000, BalanceSide::Dr)
    );
    // And again.
    let edited2 = edit_again(&w, saved.bikri_id, edited.entry_id, &|d| {
        d.lots[0].sales[0].note = Some("changed twice".into());
    });
    let third = edit_preview_of(&w, saved.bikri_id, edited2.entry_id);
    assert_eq!(rupees_of(seller(&third, w.rahim).previous_paisa), 30_000);
}

#[test]
fn editing_a_later_bikri_counts_an_earlier_one_of_the_day_that_was_edited_after_it() {
    let w = rich_world();
    owes_on(&w, "30,000", "09-10-2026");
    // A: credit 10,000. B: credit 20,000, 5,000 paid.
    let a = save_confirmed(&w, &rahim_draft(&w, 10, None), false).unwrap();
    let b = save_confirmed(&w, &rahim_draft(&w, 20, Some("5000")), false).unwrap();
    let before = edit_preview_of(&w, b.bikri_id, b.entry_id);
    assert_eq!(rupees_of(seller(&before, w.rahim).previous_paisa), 20_000);
    // A is edited up to 12,000 after B was saved: its new entry has a higher number than B's.
    edit_again(&w, a.bikri_id, a.entry_id, &|d| {
        d.lots[0].qty_received = "12".into();
        d.lots[0].sales[0].qty = "12".into();
    });
    let after = edit_preview_of(&w, b.bikri_id, b.entry_id);
    let s = seller(&after, w.rahim);
    assert_eq!(
        (rupees_of(s.previous_paisa), s.previous_side),
        (18_000, BalanceSide::Dr),
        "30,000 less the 12,000 of A as it stands now"
    );
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (3_000, BalanceSide::Dr)
    );
}

#[test]
fn editing_a_later_sale_counts_a_payment_written_between_two_later_sales_of_the_same_bikri() {
    use mandi_services::core::bikri::{LaterSaleSaveInput, StockListInput};
    use mandi_services::core::vouchers::PaymentInput;
    let w = rich_world();
    // 10 نگ received, 6 sold on the 10th: رحیم is owed 6,000.
    let mut lot = credit_lot(&w, "lot-1", w.rahim, "6", "1000", None);
    lot.qty_received = "10".into();
    let first = save_confirmed(&w, &support::draft(vec![lot]), false).unwrap();
    let later = |date: &str, key: &str| {
        let lot_id = bikri::stock_list(&w.env.ctx, &StockListInput::default())
            .unwrap()
            .rows[0]
            .lot_id;
        let mut sale = support::sale(key, w.akram, "2", "1000");
        sale.seller_comm = Some(LineChargeInput {
            method: LineChargeMethod::Percent,
            value: "0".into(),
        });
        bikri::later_sale_save(
            &w.env.ctx,
            &w.owner,
            &LaterSaleSaveInput {
                lot_id,
                date: date.into(),
                sales: vec![sale],
                confirm_cash_below_zero: true,
            },
        )
        .unwrap()
    };
    // Later sale A on the 11th (2,000), then on the 12th a payment of 1,000 to him, then later sale B
    // (2,000) the same day. All three share the بکری; the payment was written before B.
    set_today(&w.env, 2026, 10, 11);
    later("11-10-2026", "a-1");
    set_today(&w.env, 2026, 10, 12);
    vouchers::save_payment(
        &w.env.ctx,
        &w.owner,
        &PaymentInput {
            date: "12-10-2026".into(),
            account_id: w.rahim,
            amount: "1000".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            reason: String::new(),
            narration: String::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let b = later("12-10-2026", "b-1");
    assert_eq!(b.bikri_id, first.bikri_id);
    let preview = edit_preview_of(&w, b.bikri_id, b.entry_id);
    let s = seller(&preview, w.rahim);
    // Before B: 6,000 + 2,000 owed to him, less the 1,000 paid = 7,000 جمع.
    assert_eq!(
        (rupees_of(s.previous_paisa), s.previous_side),
        (7_000, BalanceSide::Cr)
    );
    assert_eq!(rupees_of(s.credit_paisa), 2_000);
    assert_eq!(rupees_of(s.paid_paisa), 0);
    assert_eq!(
        (rupees_of(s.new_paisa), s.new_side),
        (9_000, BalanceSide::Cr)
    );
}
