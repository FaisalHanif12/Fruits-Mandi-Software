//! Plan 44 T2a part 1: the edit model on the three edits that exist (بکری, voucher, پھل): the date
//! never changes (`ERR_EDIT_DATE_FIXED`), the reason is needed only for an entry that is not of
//! today, and a preview is a rehearsal of the save: the change list and the effect are what the
//! save does, and nothing stays (PRD FR-7.70, E2E-13, E2E-45, FR-15.3, E2E-42).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{EditOpenInput, EditPreviewInput, EditSaveInput};
use mandi_services::core::edit::{ChangeValue, EditChange, EditPlaceKind, EditPreview};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::VoucherEditInput;
use mandi_services::{ServiceError, bikri, edit, ledger, vouchers};
use reports_world::{full_day, save, worked_day};
use serde_json::{Value, json};

// ---- The real answers as files for the frontend ----

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

/// One pair of files: the request and the real answer.
fn pair<A: serde::Serialize>(name: &str, request: &Value, answer: &A) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

/// A refusal as a command returns it. `ref` is a stand-in: the real one is a counter.
fn wire(error: &ServiceError) -> Value {
    let ServiceError::Rejected(items) = error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    json!({
        "ok": false,
        "errors": items
            .iter()
            .map(|i| json!({ "code": i.code, "field": i.field, "values": i.values }))
            .collect::<Vec<_>>(),
        "ref": "E-20261010-0001"
    })
}

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn money(rupees: i64) -> ChangeValue {
    ChangeValue::Money { paisa: rs(rupees) }
}

fn scalar(w: &support::World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

fn find<'a>(changes: &'a [EditChange], field: &str, kind: EditPlaceKind) -> &'a EditChange {
    changes
        .iter()
        .find(|c| c.field == field && c.place.kind == kind)
        .unwrap_or_else(|| panic!("no change {field} {kind:?} in {changes:?}"))
}

/// What a rehearsal must leave alone: the journal, the audit log and the stock moves.
fn books(w: &support::World) -> (i64, i64, i64, i64) {
    (
        scalar(w, "SELECT COUNT(*) FROM journal_entries"),
        scalar(w, "SELECT COUNT(*) FROM journal_lines"),
        scalar(w, "SELECT COUNT(*) FROM audit_log"),
        scalar(w, "SELECT COUNT(*) FROM stock_moves"),
    )
}

fn receipt_edit(
    w: &support::World,
    id: i64,
    version: i64,
    amount: &str,
    date: &str,
    reason: &str,
) -> VoucherEditInput {
    serde_json::from_value(json!({
        "id": id, "version": version, "reason": reason, "confirm_cash_below_zero": false,
        "entry": { "type": "receipt", "date": date, "account_id": w.akram,
                   "amount": amount, "collector_staff_id": null, "narration": "" }
    }))
    .unwrap()
}

fn error_codes(preview: &EditPreview) -> Vec<(ErrorCode, String)> {
    preview
        .errors
        .iter()
        .map(|e| (e.code, e.field.clone()))
        .collect()
}

#[test]
fn a_voucher_review_is_the_save_without_the_save() {
    // FR-15.3 on the worked day: اکرم's receipt of 10,000 is edited to 4,000. He owed 2,880, so he
    // owes 8,880 after it, and cash goes 58,484 → 52,484.
    let w = full_day();
    let ctx = &w.env.ctx;
    let (id, version): (i64, i64) = raw(&w.dir)
        .query_row(
            "SELECT id, version FROM vouchers WHERE type = 'receipt'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let input = receipt_edit(&w, id, version, "4,000", "10-10-2026", "");
    let before = books(&w);

    // The voucher opens as the typed entry the edit takes back (the real answers for frontend).
    for (name, kind) in [("receipt", "receipt"), ("payment", "payment")] {
        let voucher_id: i64 = raw(&w.dir)
            .query_row("SELECT id FROM vouchers WHERE type = ?1", [kind], |r| {
                r.get(0)
            })
            .unwrap();
        let opened = vouchers::open_for_edit(
            ctx,
            &mandi_services::core::vouchers::VoucherGetInput { id: voucher_id },
        )
        .unwrap();
        pair(
            &format!("vouchers_open_for_edit_{name}"),
            &json!({ "id": voucher_id }),
            &opened,
        );
    }

    let preview = edit::voucher_review(ctx, &input).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    pair(
        "vouchers_edit_preview_receipt_4000",
        &json!({
            "id": id, "version": version, "reason": "", "confirm_cash_below_zero": false,
            "entry": { "type": "receipt", "date": "10-10-2026", "account_id": w.akram,
                       "amount": "4,000", "collector_staff_id": null,
                       "narration": "" }
        }),
        &preview,
    );
    let review = &preview.review;
    assert_eq!(review.entry_date, "2026-10-10");
    // Today's entry: the reason is optional. The day is open, so no reopen.
    assert!(!review.reason_required && !review.needs_reopen);
    let amount = find(&review.changes, "amount", EditPlaceKind::Entry);
    assert_eq!(
        (&amount.before, &amount.after),
        (&money(10_000), &money(4_000))
    );
    // The party: اکرم 2,880 → 8,880 بنام.
    let akram = review
        .effect
        .parties
        .iter()
        .find(|p| p.account.id == w.akram)
        .expect("اکرم is in the effect");
    assert_eq!(
        (akram.before.amount_paisa, akram.before.side),
        (rs(2_880), BalanceSide::Dr)
    );
    assert_eq!(
        (akram.after.amount_paisa, akram.after.side),
        (rs(8_880), BalanceSide::Dr)
    );
    // Cash: 58,484 → 52,484, and it stays above 0, so no question.
    let cash = review.effect.cash.as_ref().expect("cash moved");
    assert_eq!(
        (cash.before_paisa, cash.after_paisa),
        (rs(58_484), rs(52_484))
    );
    assert!(!cash.needs_confirm);
    assert!(review.effect.partners.is_empty() && !review.effect.partners_locked);

    // A rehearsal leaves nothing: the books, the audit log and the balance are as they were.
    assert_eq!(books(&w), before);
    let (side, left) = {
        let b = ledger::account_balance(ctx, w.akram).unwrap();
        (b.side, b.amount_paisa)
    };
    assert_eq!((side, left), (BalanceSide::Dr, rs(2_880)));
    assert_eq!(
        scalar(&w, "SELECT version FROM vouchers WHERE type = 'receipt'"),
        version
    );

    // The save does what the review said (Code Standards B5).
    let saved = vouchers::edit(ctx, &w.owner, &input).unwrap();
    assert_eq!(
        (saved.new_balance.side, saved.new_balance.amount_paisa),
        (BalanceSide::Dr, rs(8_880))
    );
}

/// The payment of 4,316 to رحیم on the worked day, as an edit to `amount`.
fn payment_edit(
    w: &support::World,
    id: i64,
    version: i64,
    amount: &str,
    confirm: bool,
) -> VoucherEditInput {
    serde_json::from_value(json!({
        "id": id, "version": version, "reason": "", "confirm_cash_below_zero": false,
        "entry": { "type": "payment", "date": "10-10-2026", "account_id": w.rahim,
                   "amount": amount, "reason": "", "narration": "",
                   "confirm_cash_below_zero": confirm }
    }))
    .unwrap()
}

#[test]
fn a_payment_review_shows_the_seller_on_the_credit_side_and_asks_when_cash_goes_below_zero() {
    // The payment of 4,316 to رحیم (Event 4) is edited to 1,000: he is owed 3,316 after it (جمع), and
    // cash goes 58,484 → 61,800. Edited to 100,000, cash would be 58,484 + 4,316 − 100,000 =
    // −37,200: the review says it needs the user's yes (PRD FR-6.7) and the save refuses without it.
    let w = full_day();
    let ctx = &w.env.ctx;
    let (id, version): (i64, i64) = raw(&w.dir)
        .query_row(
            "SELECT id, version FROM vouchers WHERE type = 'payment'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let before = books(&w);

    let small = edit::voucher_review(ctx, &payment_edit(&w, id, version, "1,000", false)).unwrap();
    assert!(small.errors.is_empty(), "{:?}", small.errors);
    let rahim = small
        .review
        .effect
        .parties
        .iter()
        .find(|p| p.account.id == w.rahim)
        .expect("رحیم is in the effect");
    assert_eq!(
        (rahim.before.amount_paisa, rahim.before.side),
        (rs(0), BalanceSide::Zero)
    );
    assert_eq!(
        (rahim.after.amount_paisa, rahim.after.side),
        (rs(3_316), BalanceSide::Cr)
    );
    let cash = small.review.effect.cash.as_ref().expect("cash moved");
    assert_eq!(
        (cash.before_paisa, cash.after_paisa),
        (rs(58_484), rs(61_800))
    );
    assert!(!cash.needs_confirm);

    let big = edit::voucher_review(ctx, &payment_edit(&w, id, version, "100,000", false)).unwrap();
    let cash = big.review.effect.cash.as_ref().expect("cash moved");
    assert_eq!(
        (cash.before_paisa, cash.after_paisa),
        (rs(58_484), rs(-37_200))
    );
    assert!(cash.needs_confirm);
    let refused = vouchers::edit(
        ctx,
        &w.owner,
        &payment_edit(&w, id, version, "100,000", false),
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal")
    };
    assert_eq!(items[0].code, ErrorCode::ErrCashBelowZeroNotConfirmed);
    assert_eq!(
        books(&w),
        before,
        "reviews and a refusal leave nothing behind"
    );
}

#[test]
fn the_date_never_changes_and_the_reason_is_for_an_entry_that_is_not_of_today() {
    let w = full_day();
    let ctx = &w.env.ctx;
    let (id, version): (i64, i64) = raw(&w.dir)
        .query_row(
            "SELECT id, version FROM vouchers WHERE type = 'receipt'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    // A changed date is refused in the review and in the save, at the date field, naming the date
    // the entry keeps.
    let moved = receipt_edit(&w, id, version, "4,000", "09-10-2026", "");
    let preview = edit::voucher_review(ctx, &moved).unwrap();
    assert_eq!(
        error_codes(&preview),
        [(ErrorCode::ErrEditDateFixed, "entry.date".to_owned())]
    );
    let refused = vouchers::edit(ctx, &w.owner, &moved).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal")
    };
    assert_eq!(items[0].code, ErrorCode::ErrEditDateFixed);
    assert_eq!(
        items[0].values.get("entry_date"),
        Some(&ErrorValue::Text("2026-10-10".to_owned()))
    );
    assert_eq!(
        scalar(&w, "SELECT version FROM vouchers WHERE type = 'receipt'"),
        version
    );

    // Tomorrow the entry is not of today: the review says a reason is needed, and the save refuses
    // an edit without one. The review itself does not fail for a missing reason.
    set_today(&w.env, 2026, 10, 11);
    let same_day = receipt_edit(&w, id, version, "4,000", "10-10-2026", "");
    let preview = edit::voucher_review(ctx, &same_day).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(preview.review.reason_required);
    let refused = vouchers::edit(ctx, &w.owner, &same_day).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrReasonRequired, Some("reason"))
    );
    // With a reason it saves.
    let with_reason = receipt_edit(&w, id, version, "4,000", "10-10-2026", "غلط رقم");
    vouchers::edit(ctx, &w.owner, &with_reason).unwrap();
}

#[test]
fn a_bikri_review_lists_the_line_and_what_it_moves_e2e_13() {
    // E2E-13: اکرم's rate 600 → 610 on بکری 1. He pays 12,200 + 732 + 160 = 13,092. Shared income
    // 3,972, split 1,590 / 1,191 / 1,191 (was 1,580 / 1,184 / 1,184).
    let w = worked_day();
    let saved = save(&w, &support::event1(&w));
    let ctx = &w.env.ctx;
    let opened = bikri::open_for_edit(
        ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    draft.lots[0].sales[0].buyer_rate = Some("610".into());
    let input = EditPreviewInput {
        entry_id: opened.entry_id,
        draft: draft.clone(),
    };
    let before = books(&w);

    let preview = edit::bikri_review(ctx, &input).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    pair(
        "bikri_edit_review_akram_610",
        &json!({ "entry_id": opened.entry_id, "draft": serde_json::to_value(&draft).unwrap() }),
        &preview,
    );
    let review = &preview.review;
    assert_eq!(review.entry_date, "2026-10-10");
    assert!(!review.reason_required && !review.needs_reopen);
    // The line of اکرم (lot 1, line 1): rate, commission, total; the other line is untouched.
    for c in &review.changes {
        eprintln!(
            "CHG {:?} {:?} {:?} {} {:?} -> {:?}",
            c.place.kind, c.place.lot_no, c.place.line_no, c.field, c.before, c.after
        );
    }
    let rate = find(&review.changes, "seller_rate", EditPlaceKind::Line);
    assert_eq!((rate.place.lot_no, rate.place.line_no), (Some(1), Some(1)));
    assert_eq!(rate.place.label_en.as_deref(), Some("Akram Toll Tax"));
    assert_eq!((&rate.before, &rate.after), (&money(600), &money(610)));
    let total = find(&review.changes, "buyer_total", EditPlaceKind::Line);
    assert_eq!(
        (&total.before, &total.after),
        (&money(12_880), &money(13_092))
    );
    assert!(
        review
            .changes
            .iter()
            .filter(|c| c.place.kind == EditPlaceKind::Line)
            .all(|c| c.place.line_no == Some(1)),
        "the second line did not change: {:?}",
        review.changes
    );
    // The lot: خام بکری 29,400 → 29,600 and the net 19,316 → 19,504.
    let net = find(&review.changes, "net", EditPlaceKind::Lot);
    assert_eq!((&net.before, &net.after), (&money(19_316), &money(19_504)));
    // The parties: اکرم 12,880 → 13,092 بنام; رحیم's credit grows by 188 (24,316 → 24,504).
    let akram = review
        .effect
        .parties
        .iter()
        .find(|p| p.account.id == w.akram)
        .unwrap();
    assert_eq!(
        (akram.before.amount_paisa, akram.after.amount_paisa),
        (rs(12_880), rs(13_092))
    );
    let rahim = review
        .effect
        .parties
        .iter()
        .find(|p| p.account.id == w.rahim)
        .unwrap();
    assert_eq!(
        (rahim.before.amount_paisa, rahim.after.amount_paisa),
        (rs(4_316), rs(4_504))
    );
    // The partners' share of the posting's income: 1,580 / 1,184 / 1,184 → 1,590 / 1,191 / 1,191.
    let shares: Vec<(Paisa, Paisa)> = review
        .effect
        .partners
        .iter()
        .map(|p| (p.before_paisa, p.after_paisa))
        .collect();
    assert_eq!(
        shares,
        [
            (rs(1_580), rs(1_590)),
            (rs(1_184), rs(1_191)),
            (rs(1_184), rs(1_191))
        ]
    );
    assert!(
        review.effect.cash.is_none(),
        "an edit of a credit sale moves no cash"
    );

    // Nothing stayed, and the real save matches the review.
    assert_eq!(books(&w), before);
    let edited = bikri::edit_save(
        ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(edited.preview.summary.shared_income.rupees(), 3_972);
}

#[test]
fn the_partner_figures_of_a_review_are_of_the_posting_being_edited_e2e_16b() {
    // E2E-16b: the later sale of 5 نگ to ارشد at 560 on 11-10-26 is its own posting: shared income
    // 376, split 152 / 112 / 112. Editing its rate to 570 gives value 2,850, commissions 171 and 171,
    // لاگا 40, shared income 382, split 154 / 114 / 114. The review shows these two postings only,
    // not the whole بکری (the first posting's 1,424 / 1,068 / 1,068 stay out of it).
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save(&w, &draft);
    set_today(&w.env, 2026, 10, 11);
    let stock = bikri::stock_list(
        &w.env.ctx,
        &mandi_services::core::bikri::StockListInput::default(),
    )
    .unwrap();
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    let done = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::LaterSaleSaveInput {
            lot_id: stock.rows[0].lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: done.bikri_id,
            entry_id: done.entry_id,
        },
    )
    .unwrap();
    let mut edited = opened.draft.clone();
    let sale = edited
        .lots
        .iter_mut()
        .flat_map(|l| l.sales.iter_mut())
        .find(|s| s.seller_rate == "560")
        .expect("the later sale line");
    sale.seller_rate = "570".into();
    sale.buyer_rate = Some("570".into());
    let before = books(&w);
    let preview = edit::bikri_review(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: edited,
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert_eq!(preview.review.entry_date, "2026-10-11");
    assert!(!preview.review.reason_required);
    let shares: Vec<(Paisa, Paisa)> = preview
        .review
        .effect
        .partners
        .iter()
        .map(|p| (p.before_paisa, p.after_paisa))
        .collect();
    assert_eq!(
        shares,
        [(rs(152), rs(154)), (rs(112), rs(114)), (rs(112), rs(114))]
    );
    let arshad = preview
        .review
        .effect
        .parties
        .iter()
        .find(|p| p.account.id == w.arshad)
        .unwrap();
    assert_eq!(
        (arshad.before.amount_paisa, arshad.after.amount_paisa),
        (rs(15_570 + 3_008), rs(15_570 + 3_061))
    );
    assert_eq!(books(&w), before, "a review leaves nothing behind");
}

#[test]
fn a_print_of_an_edited_entry_says_when_it_was_revised() {
    // PRD FR-7.71: a reprint after a correction carries the day of the last edit, so it cannot be
    // mistaken for the first print. A document that was never edited has none.
    use mandi_reports::print::{BikriPrintDataInput, VoucherPrintDataInput};
    use mandi_services::core::auth::Language;
    use mandi_services::core::printing::PrintDoc;
    let w = full_day();
    let ctx = &w.env.ctx;
    let bikri_print = |language| {
        mandi_reports::print::bikri_print_data(
            ctx.db(),
            &BikriPrintDataInput {
                bikri_id: 1,
                doc: PrintDoc::SellerBill,
                language,
                only_buyer_id: None,
            },
        )
        .unwrap()
        .revised_on
    };
    let (id, version): (i64, i64) = raw(&w.dir)
        .query_row(
            "SELECT id, version FROM vouchers WHERE type = 'receipt'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let slip = || {
        mandi_reports::print::voucher_print_data(
            ctx.db(),
            &VoucherPrintDataInput {
                voucher_id: id,
                language: Language::En,
            },
        )
        .unwrap()
        .revised_on
    };
    assert_eq!((bikri_print(Language::En), slip()), (None, None));

    let entry_id = scalar(
        &w,
        "SELECT id FROM journal_entries WHERE posting_type = 'bikri_first' AND source_id = 1",
    );
    let opened = bikri::open_for_edit(
        ctx,
        &EditOpenInput {
            bikri_id: 1,
            entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    draft.lots[0].sales[0].buyer_rate = Some("610".into());
    bikri::edit_save(
        ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    vouchers::edit(
        ctx,
        &w.owner,
        &receipt_edit(&w, id, version, "4,000", "10-10-2026", ""),
    )
    .unwrap();
    assert_eq!(bikri_print(Language::En).as_deref(), Some("10-10-2026"));
    assert_eq!(bikri_print(Language::Ur).as_deref(), Some("10-10-2026"));
    assert_eq!(slip().as_deref(), Some("10-10-2026"));
}

#[test]
fn a_bikri_edit_of_an_earlier_day_needs_a_reason_and_the_date_stays() {
    let w = worked_day();
    let saved = save(&w, &support::event1(&w));
    let ctx = &w.env.ctx;
    let opened = bikri::open_for_edit(
        ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    set_today(&w.env, 2026, 10, 11);

    let preview = edit::bikri_review(
        ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(preview.review.reason_required);
    let save_input = |reason: Option<&str>| EditSaveInput {
        entry_id: opened.entry_id,
        version: opened.version,
        draft: draft.clone(),
        reason: reason.map(str::to_owned),
        confirm_cash_below_zero: false,
    };
    let refused = bikri::edit_save(ctx, &w.owner, &save_input(None)).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrReasonRequired, Some("reason"))
    );
    bikri::edit_save(ctx, &w.owner, &save_input(Some("اکرم کا ریٹ"))).unwrap();
}

#[test]
fn a_rule_that_saved_lines_used_may_change_only_its_end_date_and_its_active_flag() {
    use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput};
    let w = worked_day();
    let ctx = &w.env.ctx;
    let rule =
        |id: Option<i64>, version: Option<i64>, value: &str, to: &str, active: bool| RuleInput {
            id,
            version,
            side: ChargeSide::Seller,
            charge_type: ChargeType::Commission,
            account_id: Some(w.rahim),
            item_id: Some(w.banana),
            item_group: None,
            method: ChargeMethod::Percent,
            value: value.into(),
            from_date: "01-10-2026".into(),
            to_date: to.into(),
            is_active: active,
            note: String::new(),
        };
    // رحیم + کیلا 5%: not used yet, so it can be changed in any way.
    let made =
        mandi_services::rules::save(ctx, &w.owner, &rule(None, None, "5", "", true)).unwrap();
    assert!(!made.row.used);
    let changed = mandi_services::rules::save(
        ctx,
        &w.owner,
        &rule(Some(made.row.id), Some(made.row.version), "4", "", true),
    )
    .unwrap();
    assert!(!changed.row.used);

    // Event 1 sells رحیم's کیلا by that rule: the rule is used by the two lines (seller commission
    // and nothing else of it), and a second look at the row says so.
    save(&w, &support::event1(&w));
    let row = mandi_services::rules::get(ctx, made.row.id).unwrap();
    assert!(row.used);
    let before = books(&w);

    // The value is locked; the refusal names the field and the number of saved lines.
    let refused = mandi_services::rules::save(
        ctx,
        &w.owner,
        &rule(Some(row.id), Some(row.version), "6", "", true),
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrRuleUsed, Some("value"))
    );
    assert_eq!(
        items[0].values.get("saved_lines"),
        Some(&ErrorValue::Number(2))
    );
    pair(
        "rules_save_used_refused",
        &json!({ "id": row.id, "version": row.version, "side": "seller", "charge_type": "commission",
                 "account_id": w.rahim, "item_id": w.banana, "item_group": null, "method": "percent",
                 "value": "6", "from_date": "01-10-2026", "to_date": "", "is_active": true, "note": "" }),
        &wire(&ServiceError::Rejected(items.clone())),
    );
    assert_eq!(books(&w), before);

    // The end date and the active flag are free.
    let ended = mandi_services::rules::save(
        ctx,
        &w.owner,
        &rule(Some(row.id), Some(row.version), "4", "31-10-2026", true),
    )
    .unwrap();
    assert_eq!(ended.row.to_date.as_deref(), Some("2026-10-31"));
    let off = mandi_services::rules::save(
        ctx,
        &w.owner,
        &rule(
            Some(ended.row.id),
            Some(ended.row.version),
            "4",
            "31-10-2026",
            false,
        ),
    )
    .unwrap();
    assert!(!off.row.is_active && off.row.used);
}

#[test]
fn only_the_lines_in_force_count_for_a_used_rule() {
    use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput};
    let w = worked_day();
    let ctx = &w.env.ctx;
    let made = mandi_services::rules::save(
        ctx,
        &w.owner,
        &RuleInput {
            id: None,
            version: None,
            side: ChargeSide::Seller,
            charge_type: ChargeType::Commission,
            account_id: Some(w.rahim),
            item_id: Some(w.banana),
            item_group: None,
            method: ChargeMethod::Percent,
            value: "5".into(),
            from_date: "01-10-2026".into(),
            to_date: String::new(),
            is_active: true,
            note: String::new(),
        },
    )
    .unwrap();
    let saved = save(&w, &support::event1(&w));
    // An edit (E2E-13) replaces اکرم's line: the old line is history, the new one is in force, so
    // the rule still stands behind two lines, not three.
    let opened = bikri::open_for_edit(
        ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    draft.lots[0].sales[0].buyer_rate = Some("610".into());
    bikri::edit_save(
        ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let row = mandi_services::rules::get(ctx, made.row.id).unwrap();
    let refused = mandi_services::rules::save(
        ctx,
        &w.owner,
        &RuleInput {
            id: Some(row.id),
            version: Some(row.version),
            side: ChargeSide::Seller,
            charge_type: ChargeType::Commission,
            account_id: Some(w.rahim),
            item_id: Some(w.banana),
            item_group: None,
            method: ChargeMethod::Percent,
            value: "7".into(),
            from_date: "01-10-2026".into(),
            to_date: String::new(),
            is_active: true,
            note: String::new(),
        },
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal")
    };
    assert_eq!(
        items[0].values.get("saved_lines"),
        Some(&ErrorValue::Number(2))
    );
}
