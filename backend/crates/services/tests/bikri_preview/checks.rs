//! Errors and warnings of the preview on a real database: shares, cash note, dates, credit
//! limit and rate check (PRD FR-12.4 to FR-12.7, FR-6.7, FR-7.55, plan 12).

use mandi_services::cash;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::bikri::{BikriPreview, BikriSaveInput};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::rules::{CreditLimitMode, OverridesSettings};
use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
use mandi_services::ledger::{self, PostingEvent};
use mandi_services::rules;
use mandi_services::{ServiceError, bikri};

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
    let mut shahid = sale("s-1", w.chukta_khata, "10", "1,200");
    shahid.chukta = Some(mandi_services::core::bikri::ChuktaInput {
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

#[test]
fn a_line_over_the_lot_gets_its_error_and_no_figures_and_the_save_refuses_it() {
    // PRD FR-12.6 (v1.28), plan 57, Umar's own test: the lot holds 50 and the line is 70 at 1,200.
    // Before, the screen showed 84,000 / 5,040 / 89,600 for it.
    let w = world_with(50_000, true);
    let mut d = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![sale("s-1", w.akram, "70", "1200")],
    )]);
    let p = preview(&w, &d);
    assert!(
        has(
            &p.errors,
            ErrorCode::ErrQtyExceedsLot,
            "lots[0].sales[0].qty"
        ),
        "{:?}",
        p.errors
    );
    assert_eq!(
        number(&p.errors, ErrorCode::ErrQtyExceedsLot, "available"),
        Some(50)
    );
    assert!(p.lots[0].lines[0].calc.is_none(), "no figures for the line");
    assert!(p.lots[0].totals.is_none(), "the lot waits for the quantity");
    assert_eq!((p.lots[0].sold_qty, p.lots[0].balance_qty), (0, Some(50)));
    assert_eq!(r(p.summary.gross), 0);
    assert_eq!(r(p.summary.buyer_total), 0);
    assert_eq!(r(p.summary.shared_income), 0);
    // The save is the same function and refuses with the same error.
    let error = bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: d.clone(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert!(
        matches!(&error, ServiceError::Rejected(items)
            if items.iter().any(|i| i.code == ErrorCode::ErrQtyExceedsLot
                && i.field.as_deref() == Some("lots[0].sales[0].qty"))),
        "{error:?}"
    );
    // The quantity is fixed to 50 and the figures are back, with no other change: 60,000; commission
    // 3,600 on each side; لاگا 400; the buyer pays 64,000. The save works.
    d.lots[0].sales[0].qty = "50".into();
    let p = preview(&w, &d);
    assert!(
        !p.errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrQtyExceedsLot),
        "{:?}",
        p.errors
    );
    let c = calc(&p.lots[0], 0);
    assert_eq!(
        (
            r(c.seller_amount),
            r(c.seller_comm.amount),
            r(c.buyer_comm.amount),
            r(c.laga.amount),
            r(c.buyer_total)
        ),
        (60_000, 3_600, 3_600, 400, 64_000)
    );
    assert!(p.lots[0].totals.is_some());
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: d,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

#[test]
fn of_two_lines_the_one_that_crosses_the_lot_is_refused_and_the_totals_hold_the_other() {
    // Plan 57 section 2: 50 in the lot, lines of 30 and 30. Line 1: 36,000; 2,160 on each side;
    // لاگا 240; the buyer pays 38,400. Line 2 has 20 available and no figures. The lot and the
    // whole بکری hold line 1 only.
    let w = world_with(50_000, true);
    let d = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![
            sale("s-1", w.akram, "30", "1200"),
            sale("s-2", w.arshad, "30", "1200"),
        ],
    )]);
    let p = preview(&w, &d);
    assert_eq!(
        number(&p.errors, ErrorCode::ErrQtyExceedsLot, "available"),
        Some(20)
    );
    assert!(has(
        &p.errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[0].sales[1].qty"
    ));
    assert!(!has(
        &p.errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[0].sales[0].qty"
    ));
    let c = calc(&p.lots[0], 0);
    assert_eq!(
        (
            r(c.seller_amount),
            r(c.seller_comm.amount),
            r(c.buyer_comm.amount),
            r(c.laga.amount),
            r(c.buyer_total)
        ),
        (36_000, 2_160, 2_160, 240, 38_400)
    );
    assert!(p.lots[0].lines[1].calc.is_none());
    let t = p.lots[0].totals.as_ref().unwrap();
    assert_eq!(
        (r(t.gross), r(t.seller_comm), r(t.buyer_total)),
        (36_000, 2_160, 38_400)
    );
    assert_eq!(
        (r(p.summary.gross), r(p.summary.buyer_total)),
        (36_000, 38_400)
    );
    assert_eq!((p.lots[0].sold_qty, p.lots[0].balance_qty), (30, Some(20)));
}

// ---- A lot with nothing to post (plan 61 Section 6, plan 65 T2) ----

#[test]
fn a_lot_with_only_blank_rows_and_no_charges_gives_a_preview_with_no_sale_lines() {
    // `books_of` used to refuse this whole draft as ERR_JOURNAL_UNBALANCED (a bug signal): the lot
    // is complete but it has no complete line and no charge, so its journal has no line at all
    // (`LedgerError::Empty`). It is the user's mistake, not a bug: the preview comes back with
    // `ERR_NO_SALE_LINES` on `lots`, and the save is refused with the same code.
    let w = world();
    let d = draft(vec![with_sales(
        bare_lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![blank_sale("s-1", "50")],
    )]);
    let p = bikri::preview(&w.env.ctx, &d).expect("a preview, not a refusal");
    assert!(
        has(&p.errors, ErrorCode::ErrNoSaleLines, "lots"),
        "{:?}",
        p.errors
    );
    assert!(
        !p.errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrJournalUnbalanced),
        "{:?}",
        p.errors
    );
    let refused = bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: d,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal");
    };
    assert!(items.iter().any(|i| i.code == ErrorCode::ErrNoSaleLines));
    assert!(
        !items
            .iter()
            .any(|i| i.code == ErrorCode::ErrJournalUnbalanced)
    );
}

#[test]
fn a_lot_whose_only_line_is_incomplete_keeps_the_error_of_that_line() {
    // A buyer typed but no rate: the row is not blank, so it has its own error. The journal of the
    // lot is empty again, and the preview still comes back, with that error and no made-up
    // `ERR_NO_SALE_LINES` over it.
    let w = world();
    let mut row = sale("s-1", w.akram, "20", "600");
    row.seller_rate = String::new();
    let d = draft(vec![with_sales(
        bare_lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![row],
    )]);
    let p = bikri::preview(&w.env.ctx, &d).expect("a preview, not a refusal");
    assert!(
        has(
            &p.errors,
            ErrorCode::ErrRequired,
            "lots[0].sales[0].seller_rate"
        ),
        "{:?}",
        p.errors
    );
    assert!(
        !p.errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrJournalUnbalanced),
        "{:?}",
        p.errors
    );
}
