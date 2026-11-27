//! پیشگی typed on a lot (PRD FR-12.52 v1.57, FR-12.23, FR-15.8, FR-6.7, plan 95 item 11).
//!
//! A lot may carry an advance: money its seller takes when he brings the goods. It is its own journal
//! entry (posting type `advance`, source the بکری) written in the same save, just before the first
//! entry: a seller debit flagged as an advance and روکڑ credit. It is NOT a deduction of the sale: the
//! lot's net and its seller credit stay as they are.
//!
//! The worked numbers (money agent, plan 95): رحیم has 9,260 جمع from an earlier بکری; today's lot
//! credits him 52,900; نقد 30,000; advance 20,000.
//!
//! * tick OFF: his ledger is 12,160 جمع (-9,260 + 30,000 + 20,000 - 52,900), his advance stays open
//!   (20,000) and no matching row exists;
//! * tick ON: the same ledger, one matching row of 20,000 from his advance to the credit of the lot,
//!   his advance open is 0;
//! * an edit that changes the advance to 12,000 shows the SAME previous balance (9,260 جمع), and its
//!   new balance is -9,260 + 30,000 + 12,000 - 52,900 = 20,160 جمع;
//! * an edit to 0 leaves no live advance entry, and back to 20,000 brings every number back;
//! * a cancel puts رحیم back at 9,260 جمع and cash back to where it was.
//!
//! Run it with `cargo test -p mandi-services --test lot_advance`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::bikri;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, BikriSaved, CancelInput, EditOpenInput, EditPreviewInput,
    EditSaveInput, LineChargeInput, LineChargeMethod, LotInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::{ServiceError, ledger};
use support::World;

/// A lot of رحیم with no charge: `qty` نگ received and sold to اکرم at `rate`, the seller commission
/// typed as 0% so his credit is exactly `qty x rate`.
fn lot(
    w: &World,
    key: &str,
    qty: &str,
    rate: &str,
    cash: Option<&str>,
    advance: Option<&str>,
) -> LotInput {
    lot_of(w, w.rahim, key, qty, rate, cash, advance)
}

fn lot_of(
    w: &World,
    seller: i64,
    key: &str,
    qty: &str,
    rate: &str,
    cash: Option<&str>,
    advance: Option<&str>,
) -> LotInput {
    let mut sale = support::sale(&format!("{key}-s"), w.akram, qty, rate);
    sale.seller_comm = Some(LineChargeInput {
        method: LineChargeMethod::Percent,
        value: "0".into(),
    });
    let mut lot = support::with_sales(support::bare_lot(w, key, seller, w.banana, qty), vec![sale]);
    lot.adjust_advance = false;
    lot.cash_paid = cash.map(str::to_owned);
    lot.advance = advance.map(str::to_owned);
    lot
}

/// رحیم has 9,260 جمع: an earlier بکری credited him 9,260 and paid him nothing.
fn with_earlier_credit(w: &World) {
    let draft = support::draft(vec![lot(w, "old", "1", "9260", None, None)]);
    save(w, &draft, false).unwrap();
    assert_eq!(balance(w, w.rahim), (BalanceSide::Cr, 9_260));
}

fn today_draft(w: &World, tick: bool, advance: Option<&str>) -> BikriDraft {
    // 529 x 100 = 52,900 credit, نقد 30,000.
    let mut l = lot(w, "today", "529", "100", Some("30000"), advance);
    l.adjust_advance = tick;
    support::draft(vec![l])
}

fn save(w: &World, draft: &BikriDraft, confirm: bool) -> Result<BikriSaved, ServiceError> {
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

fn count(w: &World, sql: &str) -> i64 {
    w.env
        .ctx
        .db()
        .read(|c| c.query_row(sql, [], |r| r.get::<_, i64>(0)))
        .unwrap()
}

/// How many entries of the posting type `advance` are live (posted, not reversed).
fn live_advance_entries(w: &World) -> i64 {
    count(
        w,
        "SELECT COUNT(*) FROM journal_entries e WHERE e.posting_type = 'advance' AND e.source_type = 'bikri'
           AND e.status = 'posted'
           AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id AND r.status = 'posted')",
    )
}

/// The active matching rows from a seller's advance lines, in rupees: `(amount)`, oldest first.
fn active_rows(w: &World) -> Vec<i64> {
    w.env
        .ctx
        .db()
        .read(|c| {
            let mut st = c.prepare(
                "SELECT amount_paisa FROM advance_allocations WHERE is_active = 1 ORDER BY id",
            )?;
            let rows = st
                .query_map([], |r| r.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok::<_, mandi_db::rusqlite::Error>(rows)
        })
        .unwrap()
        .into_iter()
        .map(|p| p / 100)
        .collect()
}

fn seller(
    p: &mandi_services::core::bikri::BikriPreview,
    id: i64,
) -> &mandi_services::core::bikri::SellerPaidLines {
    p.sellers
        .iter()
        .find(|s| s.seller_account_id == id)
        .unwrap()
}

#[test]
fn tick_off_the_advance_stays_open_and_the_ledger_is_12160_jama() {
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    let before = cash(&w).1;
    let draft = today_draft(&w, false, Some("20000"));
    let preview = support::preview(&w, &draft);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let s = seller(&preview, w.rahim);
    assert_eq!(
        (s.previous_paisa.rupees(), s.previous_side),
        (9_260, BalanceSide::Cr)
    );
    assert_eq!(s.credit_paisa.rupees(), 52_900);
    assert_eq!(s.paid_paisa.rupees(), 30_000);
    assert_eq!(s.advance_paisa.rupees(), 20_000);
    assert_eq!(s.carry_paisa.rupees(), 22_900);
    // -9,260 + 30,000 + 20,000 - 52,900 = -12,160.
    assert_eq!(
        (s.new_paisa.rupees(), s.new_side),
        (12_160, BalanceSide::Cr)
    );
    assert_eq!(s.working.message_id, "working-seller-new-balance-advance");
    // The advance is not a deduction: the lot's net is the credit less نقد only (C6, C7).
    assert_eq!(preview.summary.advance.rupees(), 20_000);
    assert_eq!(preview.summary.net.rupees(), 22_900);
    assert_eq!(preview.summary.seller_credit.rupees(), 52_900);
    // Cash: 30,000 نقد and 20,000 advance go out.
    assert_eq!(preview.cash.change_paisa.rupees(), -50_000);

    save(&w, &draft, false).unwrap();
    assert_eq!(live_advance_entries(&w), 1);
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 12_160));
    assert_eq!(cash(&w).1, before - 50_000);
    assert!(active_rows(&w).is_empty(), "tick off: nothing adjusted");
    // The advance line is a real advance: flagged, on his account.
    assert_eq!(
        count(
            &w,
            "SELECT COUNT(*) FROM journal_lines WHERE is_advance = 1 AND dr_paisa = 2000000"
        ),
        1
    );
    // The lot keeps the amount, and the بکری entry is the same as without it (no is_advance line).
    assert_eq!(
        count(&w, "SELECT SUM(advance_paisa) FROM bikri_lots") / 100,
        20_000
    );
}

#[test]
fn tick_on_the_credit_of_the_lot_adjusts_the_advance() {
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    let draft = today_draft(&w, true, Some("20000"));
    save(&w, &draft, false).unwrap();
    // One row: the 20,000 advance, adjusted by the credit of today's lot. His ledger is the same.
    assert_eq!(active_rows(&w), vec![20_000]);
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 12_160));
}

#[test]
fn an_advance_is_not_taken_without_a_nuqd_and_over_cash_it_asks_once() {
    // Cash 40,000: 30,000 نقد + 20,000 advance takes it to -10,000 and asks, one question.
    let w = support::world_with(40_000, true);
    with_earlier_credit(&w);
    let draft = today_draft(&w, false, Some("30000"));
    let draft = BikriDraft {
        lots: draft
            .lots
            .into_iter()
            .map(|mut l| {
                l.advance = Some("20000".into());
                l
            })
            .collect(),
        ..draft
    };
    let preview = support::preview(&w, &draft);
    assert!(preview.cash.needs_confirm);
    assert_eq!(preview.cash.cash_after_paisa.rupees(), -10_000);
    let refused = save(&w, &draft, false).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal");
    };
    assert!(
        items
            .iter()
            .any(|i| i.code == ErrorCode::ErrCashBelowZeroNotConfirmed)
    );
    assert_eq!(live_advance_entries(&w), 0, "nothing is written");
    save(&w, &draft, true).unwrap();
    assert_eq!(live_advance_entries(&w), 1);
}

#[test]
fn a_bad_advance_is_refused_at_its_field() {
    let w = support::world_with(1_000_000, true);
    // Decimals and a negative sum are the usual money errors, at the field of the advance.
    for text in ["850.5", "-5"] {
        let draft = today_draft(&w, false, Some(text));
        let preview = support::preview(&w, &draft);
        assert!(
            preview.errors.iter().any(|e| e.field == "lots[0].advance"),
            "{text}: {:?}",
            preview.errors
        );
    }
}

/// An edit of the posting, as the screen opens it.
struct Posting {
    bikri_id: i64,
    entry_id: i64,
}

fn open(w: &World, p: &Posting) -> (BikriDraft, i64) {
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: p.bikri_id,
            entry_id: p.entry_id,
        },
    )
    .unwrap();
    (opened.draft, opened.version)
}

fn edit_preview(
    w: &World,
    p: &Posting,
    draft: &BikriDraft,
) -> mandi_services::core::bikri::BikriPreview {
    let answer = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: p.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(answer.errors.is_empty(), "{:?}", answer.errors);
    answer
}

fn edit_save(w: &World, p: &mut Posting, version: i64, draft: &BikriDraft) {
    let done = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: p.entry_id,
            version,
            draft: draft.clone(),
            reason: Some("a change".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    p.entry_id = done.entry_id;
}

#[test]
fn an_edit_shows_the_same_previous_balance_and_writes_the_new_advance() {
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    let saved = save(&w, &today_draft(&w, true, Some("20000")), false).unwrap();
    let cash_now = cash(&w).1;
    let mut p = Posting {
        bikri_id: saved.bikri_id,
        entry_id: saved.entry_id,
    };
    // The edit screen opens the lot WITH its advance.
    let (mut draft, version) = open(&w, &p);
    assert_eq!(draft.lots[0].advance.as_deref(), Some("20000"));
    // 20,000 -> 12,000. The previous balance is still 9,260 جمع (the old advance is not counted).
    draft.lots[0].advance = Some("12000".into());
    let preview = edit_preview(&w, &p, &draft);
    let s = seller(&preview, w.rahim);
    assert_eq!(
        (s.previous_paisa.rupees(), s.previous_side),
        (9_260, BalanceSide::Cr)
    );
    // -9,260 + 30,000 + 12,000 - 52,900 = -20,160.
    assert_eq!(
        (s.new_paisa.rupees(), s.new_side),
        (20_160, BalanceSide::Cr)
    );
    // Cash: now holds -50,000; the new posting takes -42,000: 8,000 comes back.
    assert_eq!(preview.cash.change_paisa.rupees(), 8_000);
    edit_save(&w, &mut p, version, &draft);
    assert_eq!(live_advance_entries(&w), 1);
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 20_160));
    assert_eq!(cash(&w).1, cash_now + 8_000);
    // The old row is off and the new one is 12,000.
    assert_eq!(active_rows(&w), vec![12_000]);

    // 12,000 -> 0: no live advance entry, no row, and the ledger is -9,260 + 30,000 - 52,900.
    let (mut draft, version) = open(&w, &p);
    assert_eq!(draft.lots[0].advance.as_deref(), Some("12000"));
    draft.lots[0].advance = None;
    edit_save(&w, &mut p, version, &draft);
    assert_eq!(live_advance_entries(&w), 0);
    assert!(active_rows(&w).is_empty());
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 32_160));
    assert_eq!(cash(&w).1, cash_now + 20_000);

    // 0 -> 20,000 again: every number of the first save is back, and the advance is adjusted again.
    let (mut draft, version) = open(&w, &p);
    assert_eq!(draft.lots[0].advance, None);
    let preview = {
        draft.lots[0].advance = Some("20000".into());
        edit_preview(&w, &p, &draft)
    };
    let s = seller(&preview, w.rahim);
    assert_eq!(
        (s.previous_paisa.rupees(), s.previous_side),
        (9_260, BalanceSide::Cr)
    );
    assert_eq!(
        (s.new_paisa.rupees(), s.new_side),
        (12_160, BalanceSide::Cr)
    );
    edit_save(&w, &mut p, version, &draft);
    assert_eq!(live_advance_entries(&w), 1);
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 12_160));
    assert_eq!(cash(&w).1, cash_now);
    assert_eq!(
        active_rows(&w),
        vec![20_000],
        "an advance added by an edit still ranks first"
    );
}

#[test]
fn a_cancel_reverses_the_advance_and_gives_back_its_cash() {
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    let cash_before = cash(&w).1;
    let saved = save(&w, &today_draft(&w, true, Some("20000")), false).unwrap();
    let version = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: saved.bikri_id,
            version,
            reason: "a mistake".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(live_advance_entries(&w), 0);
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 9_260));
    assert_eq!(cash(&w).1, cash_before);
    assert!(active_rows(&w).is_empty());
}

#[test]
fn two_lots_of_one_seller_each_keep_their_advance_and_the_first_credit_takes_both() {
    // Lot 1: credit 52,900, نقد 30,000, advance 20,000, tick on. Lot 2: credit 9,400 (94 x 100), advance
    // 5,000, tick on. Both advances are adjusted oldest first by the credits: lot 1's credit takes
    // 20,000 of lot 1's and 5,000 of lot 2's (the advance lines come first in his order).
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    let mut a = lot(&w, "a", "529", "100", Some("30000"), Some("20000"));
    a.adjust_advance = true;
    let mut b = lot(&w, "b", "94", "100", None, Some("5000"));
    b.adjust_advance = true;
    let draft = support::draft(vec![a, b]);
    let preview = support::preview(&w, &draft);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let s = seller(&preview, w.rahim);
    assert_eq!(s.advance_paisa.rupees(), 25_000);
    assert_eq!(s.credit_paisa.rupees(), 62_300);
    // -9,260 + 30,000 + 25,000 - 62,300 = -16,560.
    assert_eq!(
        (s.new_paisa.rupees(), s.new_side),
        (16_560, BalanceSide::Cr)
    );
    assert_eq!(preview.summary.advance.rupees(), 25_000);
    save(&w, &draft, false).unwrap();
    // One entry, two advance lines (one for each lot) and one cash credit of 25,000.
    assert_eq!(live_advance_entries(&w), 1);
    assert_eq!(
        count(
            &w,
            "SELECT COUNT(*) FROM journal_lines WHERE is_advance = 1"
        ),
        2
    );
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 16_560));
    assert_eq!(active_rows(&w), vec![20_000, 5_000]);
}

#[test]
fn two_sellers_over_cash_ask_once_with_the_cash_after_and_save_when_confirmed() {
    // رحیم: credit 52,900, نقد 30,000, advance 20,000. کریم: credit 10,000, advance 3,000. Cash 50,000:
    // 50,000 - 30,000 - 20,000 - 3,000 = -3,000.
    let w = support::world_with(50_000, true);
    let a = lot(&w, "a", "529", "100", Some("30000"), Some("20000"));
    let b = lot_of(&w, w.karim, "b", "100", "100", None, Some("3000"));
    let draft = support::draft(vec![a, b]);
    let preview = support::preview(&w, &draft);
    assert!(preview.cash.needs_confirm);
    assert_eq!(preview.cash.cash_after_paisa.rupees(), -3_000);
    let refused = save(&w, &draft, false).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal");
    };
    let cash_item = items
        .iter()
        .find(|i| i.code == ErrorCode::ErrCashBelowZeroNotConfirmed)
        .expect("the cash question");
    assert_eq!(
        cash_item.values.get("cash_after_paisa"),
        Some(&mandi_services::core::error::ErrorValue::Number(-300_000))
    );
    save(&w, &draft, true).unwrap();
    // کریم: 3,000 advance open, his credit 10,000 stays payable; the ledger is 7,000 جمع.
    assert_eq!(balance(&w, w.karim), (BalanceSide::Cr, 7_000));
    assert_eq!(cash(&w), (BalanceSide::Cr, 3_000));
}

#[test]
fn the_daily_books_check_is_clean_with_an_advance_and_finds_one_that_changed() {
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    save(&w, &today_draft(&w, true, Some("20000")), false).unwrap();
    let clean = mandi_services::health::compare_connection(&common::raw(&w.dir)).unwrap();
    assert_eq!(clean.count, 0, "{:?}", clean.reasons);
    // A lot that says 25,000 while the journal paid 20,000 is a difference of the books.
    let conn = common::raw(&w.dir);
    conn.execute(
        "UPDATE bikri_lots SET advance_paisa = 2500000 WHERE advance_paisa = 2000000",
        [],
    )
    .unwrap();
    let found = mandi_services::health::compare_connection(&conn).unwrap();
    assert!(found.count >= 1, "{:?}", found.reasons);
    assert!(
        found
            .reasons
            .iter()
            .any(|d| d.table == "bikri_lots" && d.what.contains("advance")),
        "{:?}",
        found.reasons
    );
}

#[test]
fn the_seller_slip_the_statement_and_the_sale_print_show_the_advance() {
    use mandi_reports::reports::{DaySlipsInput, SlipSections, sale_print, seller_bills};
    use mandi_services::core::auth::Language;
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    save(&w, &today_draft(&w, false, Some("20000")), false).unwrap();
    let input = DaySlipsInput {
        date: "10-10-2026".into(),
        bikri_id: None,
        bikri_ids: None,
        buyer_ids: None,
        sections: SlipSections::Sellers,
        language: Language::En,
        paper: None,
    };
    // The seller bill (his slip) has a پیشگی line, and today's left holds it:
    // net sale 52,900 (credit), paid 30,000, advance 20,000: left = -52,900 + 30,000 + 20,000.
    let bills = seller_bills(w.env.ctx.db(), &input).unwrap();
    let text = format!("{:?}", bills.sellers);
    assert!(text.contains("report-r35-advance"), "{text}");
    assert!(text.contains("20,000"), "{text}");
    // The whole sale print has an advance column in the sellers table, last, with the 20,000, and
    // the net and deductions are the same as without it.
    let print = sale_print(w.env.ctx.db(), &input).unwrap();
    let keys: Vec<&str> = print
        .sellers
        .columns
        .iter()
        .map(|c| c.key.as_str())
        .collect();
    assert_eq!(keys.last().copied(), Some("advance"), "{keys:?}");
    let totals = format!("{:?}", print.sellers.totals);
    assert!(totals.contains("20,000"), "{totals}");
}

/// رحیم with credit 52,900, نقد 50,000 and advance 20,000.
fn over_today(w: &World, tick: bool) -> BikriDraft {
    let mut l = lot(w, "today", "529", "100", Some("50000"), Some("20000"));
    l.adjust_advance = tick;
    support::draft(vec![l])
}

fn asks_seller(error: &ServiceError) -> bool {
    match error {
        ServiceError::Rejected(items) => items.iter().any(|i| {
            i.code == ErrorCode::ErrSellerPaidAboveToday
                && i.field.as_deref() == Some("sellers[0].paid")
        }),
        _ => false,
    }
}

#[test]
fn with_the_tick_on_the_advance_counts_for_the_question_above_todays_credit() {
    // Lead decision, plan 95: previous 0, credit 52,900, نقد 50,000, advance 20,000. With the tick ON
    // the advance is taken off today's credit, so 50,000 + 20,000 = 70,000 is above 52,900 and he is
    // asked. With the tick OFF the advance stays open and only the نقد 50,000 counts: no question.
    let w = support::world_with(1_000_000, true);
    let on = over_today(&w, true);
    let preview = support::preview(&w, &on);
    let s = seller(&preview, w.rahim);
    assert!(s.needs_confirm);
    assert_eq!(s.paid_paisa.rupees(), 50_000, "paid is still the نقد");
    assert_eq!(s.advance_adjusted_paisa.rupees(), 20_000);
    assert_eq!(s.limit_paisa.rupees(), 52_900);
    let refused = save(&w, &on, false).unwrap_err();
    assert!(asks_seller(&refused), "{refused:?}");
    assert_eq!(live_advance_entries(&w), 0, "nothing is written");
    save(&w, &on, true).unwrap();
    assert_eq!(live_advance_entries(&w), 1);

    let w = support::world_with(1_000_000, true);
    let off = over_today(&w, false);
    let preview = support::preview(&w, &off);
    let s = seller(&preview, w.rahim);
    assert!(!s.needs_confirm);
    assert_eq!(s.advance_adjusted_paisa.rupees(), 0);
    save(&w, &off, false).unwrap();
}

#[test]
fn an_edit_that_leaves_his_amount_does_not_ask_again() {
    let w = support::world_with(1_000_000, true);
    let saved = save(&w, &over_today(&w, true), true).unwrap();
    let mut p = Posting {
        bikri_id: saved.bikri_id,
        entry_id: saved.entry_id,
    };
    // The same lot, the same amount (50,000 + 20,000): not asked, as plan 91 (Umar, 22-11-2026).
    let (draft, version) = open(&w, &p);
    let preview = edit_preview(&w, &p, &draft);
    assert!(!seller(&preview, w.rahim).needs_confirm);
    edit_save(&w, &mut p, version, &draft);
    // A bigger advance (20,000 -> 25,000) changes his amount: asked.
    let (mut draft, version) = open(&w, &p);
    draft.lots[0].advance = Some("25000".into());
    let preview = edit_preview(&w, &p, &draft);
    assert!(seller(&preview, w.rahim).needs_confirm);
    let refused = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: p.entry_id,
            version,
            draft: draft.clone(),
            reason: Some("a change".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert!(asks_seller(&refused), "{refused:?}");
}

// ---- the real answers as JSON files for the frontend (plan 95 item 11) ----

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn write_or_check(file: &str, text: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{file} is not the real answer any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

fn pair<A: serde::Serialize>(name: &str, request: &serde_json::Value, answer: &A) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

/// Files (tests/fixtures/): `lot_advance_preview_tick_off` (+ `_request`): `bikri_preview` of the
/// worked case; `lot_advance_slip_r35_ur|en`, `lot_advance_seller_bills_ur|en` (the slip-made seller
/// bill), `lot_advance_seller_bill_r1_ur|en` (the R1 bill of one بکری), `lot_advance_sale_print_ur|en`.
#[test]
fn the_lot_advance_fixtures_are_the_real_answers() {
    use mandi_reports::reports::{
        DaySlipsInput, ReportPrintInput, SlipSections, print_data, sale_print, seller_bills,
    };
    use mandi_services::core::auth::Language;
    use serde_json::json;
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    // Tick off: the advance stays open; the numbers of the worked case.
    let draft = today_draft(&w, false, Some("20000"));
    let request = json!({ "draft": serde_json::to_value(&draft).unwrap() });
    pair(
        "lot_advance_preview_tick_off",
        &request,
        &support::preview(&w, &draft),
    );
    let saved = save(&w, &draft, false).unwrap();
    for (name, language) in [("en", Language::En), ("ur", Language::Ur)] {
        let lang = if language == Language::Ur { "ur" } else { "en" };
        // The R35 slip of رحیم on the day.
        let request = json!({
            "id": "R35",
            "filters": {
                "shortcut": null, "from": "", "to": "", "as_of": "10-10-2026",
                "party": { "account_id": w.rahim, "person_key": null },
                "seller_id": null, "zamindar_id": null, "item_id": null, "city_id": null,
                "adda_id": null, "detail": false, "partner_id": null, "garden_id": null,
                "bikri_id": null
            },
            "language": lang, "paper": "a4", "sort": null, "group_by_month": false
        });
        let input: ReportPrintInput = serde_json::from_value(request.clone()).unwrap();
        let slip = print_data(w.env.ctx.db(), &input).unwrap();
        let lines = serde_json::to_value(&slip).unwrap();
        let labels: Vec<&str> = lines["summary"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l["label_id"].as_str().unwrap())
            .collect();
        let at = |label: &str| labels.iter().position(|l| *l == label).unwrap();
        assert_eq!(
            at("report-r35-advance"),
            at("report-r35-paid-today") + 1,
            "{labels:?}"
        );
        pair(&format!("lot_advance_slip_r35_{name}"), &request, &slip);
        // The seller bills made from slips, and the whole sale print, of the day.
        let day = json!({
            "date": "10-10-2026", "bikri_id": null, "bikri_ids": null, "buyer_ids": null,
            "sections": "both", "language": lang, "paper": "a4",
        });
        let input: DaySlipsInput = serde_json::from_value(day.clone()).unwrap();
        let bills = seller_bills(
            w.env.ctx.db(),
            &DaySlipsInput {
                sections: SlipSections::Sellers,
                ..input.clone()
            },
        )
        .unwrap();
        pair(&format!("lot_advance_seller_bills_{name}"), &day, &bills);
        pair(
            &format!("lot_advance_sale_print_{name}"),
            &day,
            &sale_print(w.env.ctx.db(), &input).unwrap(),
        );
        // The R1 seller bill of the one بکری: the advance line comes after the net sale.
        let r1_request =
            json!({ "bikri_id": saved.bikri_id, "doc": "seller_bill", "language": lang });
        let r1_input: mandi_reports::print::BikriPrintDataInput =
            serde_json::from_value(r1_request.clone()).unwrap();
        let r1 = mandi_reports::print::bikri_print_data(w.env.ctx.db(), &r1_input).unwrap();
        let text = serde_json::to_string(&r1).unwrap();
        assert!(text.contains("print-seller-advance"), "{text}");
        pair(
            &format!("lot_advance_seller_bill_r1_{name}"),
            &r1_request,
            &r1,
        );
    }
}

#[test]
fn an_undo_gives_back_a_draft_with_the_advance_and_the_cash() {
    use mandi_services::core::bikri::UndoInput;
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    let cash_before = cash(&w).1;
    let saved = save(&w, &today_draft(&w, true, Some("20000")), false).unwrap();
    let undone = bikri::undo(
        &w.env.ctx,
        &w.owner,
        &UndoInput {
            id: saved.bikri_id,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // The draft to correct keeps the advance, and every number of the books is back.
    assert_eq!(undone.draft.lots[0].advance.as_deref(), Some("20000"));
    assert_eq!(live_advance_entries(&w), 0);
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 9_260));
    assert_eq!(cash(&w).1, cash_before);
    let found = mandi_services::health::compare_connection(&common::raw(&w.dir)).unwrap();
    assert_eq!(found.count, 0, "{:?}", found.reasons);
}

#[test]
fn the_books_check_stays_clean_after_an_edit_and_a_cancel_of_a_bikri_with_an_advance() {
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    let saved = save(&w, &today_draft(&w, true, Some("20000")), false).unwrap();
    let mut p = Posting {
        bikri_id: saved.bikri_id,
        entry_id: saved.entry_id,
    };
    let (mut draft, version) = open(&w, &p);
    draft.lots[0].advance = Some("12000".into());
    edit_save(&w, &mut p, version, &draft);
    let found = mandi_services::health::compare_connection(&common::raw(&w.dir)).unwrap();
    assert_eq!(found.count, 0, "after the edit: {:?}", found.reasons);
    let version = bikri::get(&w.env.ctx, p.bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: p.bikri_id,
            version,
            reason: "a mistake".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let found = mandi_services::health::compare_connection(&common::raw(&w.dir)).unwrap();
    assert_eq!(found.count, 0, "after the cancel: {:?}", found.reasons);
}

#[test]
fn the_last_advance_of_the_seller_card_includes_the_advance_typed_on_a_lot() {
    // FR-7.60: his card shows the date and amount of his last advance. A voucher of 15,000 on
    // 09-10-2026, then the lot advance of 20,000 on 10-10-2026: the last one is the lot's, with the
    // بکری number and no voucher number. Cancelling the بکری puts the voucher back as the last.
    let w = support::world_with(1_000_000, true);
    mandi_services::vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::AdvanceInput {
            confirm_cash_below_zero: true,
            date: "09-10-2026".into(),
            seller_account_id: w.rahim,
            amount: "15,000".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
            reason: None,
        },
    )
    .unwrap();
    let saved = save(&w, &today_draft(&w, false, Some("20000")), false).unwrap();
    let last = mandi_services::assist::advances(&w.env.ctx, w.rahim)
        .unwrap()
        .last
        .unwrap();
    assert_eq!(
        (last.entry_date.as_str(), last.amount_paisa.rupees()),
        ("2026-10-10", 20_000)
    );
    assert_eq!(last.voucher_number, None);
    let number = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().number;
    assert_eq!(last.bikri_number, Some(number));
    // The open advances list names the source too.
    let info = mandi_services::assist::advances(&w.env.ctx, w.rahim).unwrap();
    assert!(info.open.iter().any(|o| o.source == "bikri"));
    let version = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: saved.bikri_id,
            version,
            reason: "a mistake".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let last = mandi_services::assist::advances(&w.env.ctx, w.rahim)
        .unwrap()
        .last
        .unwrap();
    assert_eq!(
        (
            last.entry_date.as_str(),
            last.amount_paisa.rupees(),
            last.voucher_number
        ),
        ("2026-10-09", 15_000, Some(1))
    );
}

#[test]
fn the_cash_book_row_of_a_lot_advance_says_bikri_advance_with_the_number_of_the_sale() {
    use mandi_services::core::cashbook::CashbookInput;
    let w = support::world_with(1_000_000, true);
    with_earlier_credit(&w);
    let saved = save(&w, &today_draft(&w, false, Some("20000")), false).unwrap();
    let number = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().number;
    let book = mandi_services::cashbook::get(
        &w.env.ctx,
        &CashbookInput {
            from: None,
            to: None,
            offset: 0,
            limit: 0,
            sort: None,
            group_by_month: false,
            show_reversed: false,
            kinds: None,
        },
    )
    .unwrap();
    let row = book
        .rows
        .iter()
        .find(|r| r.posting_type == "bikri_advance")
        .expect("the advance row");
    assert_eq!(row.number, Some(number));
    assert_eq!(row.out_paisa.rupees(), 20_000);
    // The بکری's own cash row (نقد 30,000) stays a بکری row.
    assert!(
        book.rows
            .iter()
            .any(|r| r.posting_type == "bikri_first" && r.out_paisa.rupees() == 30_000)
    );
}
