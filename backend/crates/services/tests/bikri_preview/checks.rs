//! Errors and warnings of the preview on a real database: shares, cash note, dates, credit
//! limit and rate check (PRD FR-12.4 to FR-12.7, FR-6.7, FR-7.55, plan 12).

use mandi_services::cash;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::bikri::BikriPreview;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::rules::{CreditLimitMode, OverridesSettings};
use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
use mandi_services::ledger::{self, PostingEvent};
use mandi_services::rules;

use crate::support::*;

fn number(
    list: &[mandi_services::core::error::FieldError],
    code: ErrorCode,
    key: &str,
) -> Option<i64> {
    list.iter()
        .find(|e| e.code == code)
        .and_then(|e| match e.values.get(key) {
            Some(ErrorValue::Number(n)) => Some(*n),
            _ => None,
        })
}

fn set_checks(w: &World, block: bool, rate_check: bool, percent: u32) {
    rules::set_overrides(
        &w.env.ctx,
        &w.owner,
        &OverridesSettings {
            allow_line_override: true,
            credit_limit_mode: if block {
                CreditLimitMode::Block
            } else {
                CreditLimitMode::Warn
            },
            rate_check_enabled: rate_check,
            rate_check_percent: percent,
        },
    )
    .unwrap();
}

#[test]
fn shares_that_do_not_total_100_stop_the_save_but_the_numbers_stay() {
    // PRD FR-12.8 check 7: no shares set, so the split cannot be made.
    let w = world_with(50_000, false);
    let p = preview(&w, &event1(&w));
    assert!(
        has(&p.errors, ErrorCode::ErrSharesNot100, "split"),
        "{:?}",
        p.errors
    );
    assert_eq!(
        number(&p.errors, ErrorCode::ErrSharesNot100, "total_bps"),
        Some(0)
    );
    assert!(p.split.is_empty());
    assert_eq!(
        r(p.summary.shared_income),
        3_948,
        "the totals are still there"
    );
    assert_eq!(r(p.lots[0].totals.as_ref().unwrap().net), 19_316);
}

#[test]
fn the_cash_note_asks_when_cash_goes_below_zero() {
    // PRD FR-6.7: cash 20,000 and نقد 25,000 leaves -5,000.
    let w = world_with(20_000, true);
    let mut big = lot(&w, "lot-1", w.rahim, w.banana, "50");
    big.cash_paid = Some("25,000".into());
    let p = preview(
        &w,
        &draft(vec![with_sales(
            big,
            vec![sale("s-1", w.akram, "50", "600")],
        )]),
    );
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    assert_eq!(
        (
            r(p.cash.cash_before_paisa),
            r(p.cash.change_paisa),
            r(p.cash.cash_after_paisa)
        ),
        (20_000, -25_000, -5_000)
    );
    assert!(p.cash.below_zero && p.cash.lowers_cash && p.cash.needs_confirm);

    // The shared piece the save will use: refused without the user's yes, accepted with it.
    let refused = cash::require_confirmed(&p.cash, false).unwrap_err();
    assert!(refused.has_code(ErrorCode::ErrCashBelowZeroNotConfirmed));
    let value = match &refused {
        mandi_services::ServiceError::Rejected(items) => {
            items[0].values.get("cash_after_paisa").cloned()
        }
        _ => None,
    };
    assert_eq!(value, Some(ErrorValue::Number(-500_000)));
    cash::require_confirmed(&p.cash, true).unwrap();
    // The same function from a read, as the other cash-out saves call it.
    let again = w
        .env
        .ctx
        .db()
        .read(|conn| cash::cash_after_entry(conn, rupees(-25_000)))
        .unwrap();
    assert_eq!(again, p.cash);
}

#[test]
fn cash_already_below_zero_does_not_ask_for_an_entry_that_puts_cash_in() {
    // Cash is -30,000 after a general entry; Event 2 puts 12,800 in, leaving -17,200.
    let w = world_with(0, true);
    let cash_id = id_of(&w.env, "01010001");
    let entry = build_entry(vec![
        LedgerLine::new(
            mandi_services::core::ids::AccountId::new(w.akram),
            Side::Dr,
            rupees(30_000),
        ),
        LedgerLine::new(
            mandi_services::core::ids::AccountId::new(cash_id),
            Side::Cr,
            rupees(30_000),
        ),
    ])
    .unwrap();
    w.env
        .ctx
        .db()
        .write(|tx| {
            ledger::post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::parse_iso("2026-10-10")
                        .unwrap(),
                    posting_type: PostingType::General,
                    source_type: SourceType::Voucher,
                    source_id: 1,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
    let mut karim = lot(&w, "lot-1", w.karim, w.apple, "10");
    karim.freight = Some("1,000".into());
    karim.cash_paid = None;
    let mut shahid = sale("s-1", w.pukhta_buyer, "10", "1,200");
    shahid.pukhta = Some(mandi_services::core::bikri::PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: String::new(),
    });
    let p = preview(&w, &draft(vec![with_sales(karim, vec![shahid])]));
    assert_eq!(r(p.cash.cash_before_paisa), -30_000);
    assert_eq!(r(p.cash.cash_after_paisa), -17_200);
    assert!(p.cash.below_zero && !p.cash.lowers_cash && !p.cash.needs_confirm);
    // But an entry that takes more cash out asks again.
    let out = preview(&w, &event1(&w));
    assert!(out.cash.needs_confirm);
}

#[test]
fn a_posting_day_after_today_is_refused_at_the_header_and_no_row_is_worked_out() {
    let w = world();
    let mut d = event1(&w);
    d.header.entry_date = "11-10-2026".into();
    let p = preview(&w, &d);
    assert!(has(
        &p.errors,
        ErrorCode::ErrDateInFuture,
        "header.entry_date"
    ));
    assert!(p.lots[0].lines.iter().all(|l| l.calc.is_none()));
    assert!(p.lots[0].totals.is_none());
    assert!(p.split.is_empty());
}

fn credit_limited(w_limit: &str, block: bool) -> (World, BikriPreview) {
    let w = world();
    let limited = add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "فضل دین",
        "Fazal Din",
        true,
        w_limit,
    );
    set_checks(&w, block, true, 30);
    let p = preview(
        &w,
        &draft(vec![with_sales(
            lot(&w, "lot-1", w.rahim, w.banana, "50"),
            vec![sale("s-1", limited, "20", "600")],
        )]),
    );
    (w, p)
}

#[test]
fn the_credit_limit_warns_or_blocks_by_the_setting() {
    // 12,880 against a limit of 12,879.
    let (_w, p) = credit_limited("12,879", false);
    let field = "lots[0].sales[0].buyer_account_id";
    assert!(
        has(&p.warnings, ErrorCode::ErrCreditLimitExceeded, field),
        "{:?}",
        p.warnings
    );
    assert_eq!(
        number(
            &p.warnings,
            ErrorCode::ErrCreditLimitExceeded,
            "limit_paisa"
        ),
        Some(1_287_900)
    );
    assert_eq!(
        number(
            &p.warnings,
            ErrorCode::ErrCreditLimitExceeded,
            "would_owe_paisa"
        ),
        Some(1_288_000)
    );
    assert!(p.errors.is_empty());
    let (_w, p) = credit_limited("12,879", true);
    assert!(has(&p.errors, ErrorCode::ErrCreditLimitExceeded, field));
    // Exactly at the limit: nothing about the limit (the 30 nag left unsold only warn).
    let (_w, p) = credit_limited("12,880", true);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    assert!(
        p.warnings
            .iter()
            .all(|e| e.code != ErrorCode::ErrCreditLimitExceeded)
    );
}

#[test]
fn the_credit_limit_counts_what_the_buyer_already_owes() {
    // He owes 5,000 from an earlier general entry; 5,000 + 12,880 is above 17,000.
    let w = world();
    let limited = add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "فضل دین",
        "Fazal Din",
        true,
        "17,000",
    );
    let cash_id = id_of(&w.env, "01010001");
    let entry = build_entry(vec![
        LedgerLine::new(
            mandi_services::core::ids::AccountId::new(limited),
            Side::Dr,
            rupees(5_000),
        ),
        LedgerLine::new(
            mandi_services::core::ids::AccountId::new(cash_id),
            Side::Cr,
            rupees(5_000),
        ),
    ])
    .unwrap();
    w.env
        .ctx
        .db()
        .write(|tx| {
            ledger::post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::parse_iso("2026-10-10")
                        .unwrap(),
                    posting_type: PostingType::General,
                    source_type: SourceType::Voucher,
                    source_id: 1,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
    let p = preview(
        &w,
        &draft(vec![with_sales(
            lot(&w, "lot-1", w.rahim, w.banana, "50"),
            vec![sale("s-1", limited, "20", "600")],
        )]),
    );
    assert_eq!(
        number(
            &p.warnings,
            ErrorCode::ErrCreditLimitExceeded,
            "would_owe_paisa"
        ),
        Some(1_788_000)
    );
}

#[test]
fn the_rate_check_uses_the_saved_sales_of_the_last_7_days() {
    // 10 nag at 1,000 on 09-10-2026: the average is 1,000.
    let w = world();
    insert_saved(&w, &{
        let mut s = SavedSale::new(1, "2026-10-09", w.karim, w.arshad, w.banana, w.box_);
        s.qty = 10;
        s.rate = 1_000;
        s
    });
    // A huge sale 7 days back (03-10) is outside the 04-10 to 10-10 window.
    insert_saved(&w, &{
        let mut s = SavedSale::new(2, "2026-10-03", w.karim, w.arshad, w.banana, w.box_);
        s.qty = 1_000;
        s.rate = 1;
        s
    });
    let d = |rate: &str| {
        draft(vec![with_sales(
            lot(&w, "lot-1", w.rahim, w.banana, "50"),
            vec![sale("s-1", w.akram, "20", rate)],
        )])
    };
    let far = |p: &BikriPreview| {
        p.warnings
            .iter()
            .any(|e| e.code == ErrorCode::ErrRateFarFromAverage)
    };
    let p = preview(&w, &d("1,300"));
    assert!(!far(&p), "1,300 is exactly +30%: {:?}", p.warnings);
    let p = preview(&w, &d("1,301"));
    let field = "lots[0].sales[0].seller_rate";
    assert!(
        has(&p.warnings, ErrorCode::ErrRateFarFromAverage, field),
        "{:?}",
        p.warnings
    );
    assert_eq!(
        number(
            &p.warnings,
            ErrorCode::ErrRateFarFromAverage,
            "average_paisa"
        ),
        Some(100_000)
    );
    assert!(p.errors.is_empty(), "a rate warning never blocks");
    assert!(p.lots[0].lines[0].warnings.iter().any(|e| e.field == field));
    let p = preview(&w, &d("699"));
    assert!(has(&p.warnings, ErrorCode::ErrRateFarFromAverage, field));
    // Switched off: no warning.
    set_checks(&w, false, false, 30);
    assert!(!far(&preview(&w, &d("5,000"))));
    // A wider band: +60% allows 1,600.
    set_checks(&w, false, true, 60);
    assert!(!far(&preview(&w, &d("1,600"))));
    assert!(far(&preview(&w, &d("1,601"))));
}

#[test]
fn a_sale_cancelled_in_the_facts_is_taken_out_of_the_average() {
    // A fact row with sign -1 (a reversal) takes the original out (PRD FR-18.6).
    let w = world();
    let (bikri_id, lot_id, sale_id) = insert_saved(
        &w,
        &SavedSale::new(1, "2026-10-09", w.karim, w.arshad, w.banana, w.box_),
    );
    let conn = crate::common::raw(&w.dir);
    conn.execute(
        "INSERT INTO analytics_facts (journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id,
            seller_account_id, buyer_account_id, qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa,
            buyer_comm_paisa, laga_paisa, sign)
         SELECT journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id, seller_account_id,
            buyer_account_id, qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa,
            laga_paisa, -1 FROM analytics_facts WHERE sale_id = ?1",
        [sale_id],
    )
    .unwrap();
    let _ = (bikri_id, lot_id);
    let p = preview(
        &w,
        &draft(vec![with_sales(
            lot(&w, "lot-1", w.rahim, w.banana, "50"),
            vec![sale("s-1", w.akram, "20", "5,000")],
        )]),
    );
    assert!(
        p.warnings
            .iter()
            .all(|e| e.code != ErrorCode::ErrRateFarFromAverage),
        "no sales left, nothing to compare with: {:?}",
        p.warnings
    );
}

#[test]
fn a_lot_cannot_use_the_own_goods_account_or_a_buyer_as_seller() {
    let w = world();
    let p = preview(
        &w,
        &draft(vec![with_sales(
            lot(&w, "lot-1", w.akram, w.banana, "50"),
            vec![sale("s-1", w.rahim, "20", "600")],
        )]),
    );
    assert!(has(
        &p.errors,
        ErrorCode::ErrNotASeller,
        "lots[0].seller_account_id"
    ));
    assert!(has(
        &p.errors,
        ErrorCode::ErrNotABuyer,
        "lots[0].sales[0].buyer_account_id"
    ));
}
