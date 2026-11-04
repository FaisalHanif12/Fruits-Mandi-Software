//! Plan 47 (PRD FR-13.11, FR-13.10, FR-13.3, FR-12.9, E81, A79): a پختہ party's own account on a real
//! database. He is a full account of his side: his lines post exactly like a buyer's or a seller's,
//! with no person key, cash first.
//!
//! The numbers are PRD 4.5 (Event 2) with نذیر (a پختہ گاہک, code 0032) in the place of شاہد, and
//! with کمال (a پختہ بیوپاری, code 0033) in the place of the one-time seller of FR-13.10.
//!
//! Run it with `cargo test -p mandi-services --test pukhta_party`.
#![cfg(test)]

#[path = "bikri_save/../common/mod.rs"]
mod common;
#[allow(dead_code)]
#[path = "bikri_save/helpers.rs"]
mod helpers;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;

use helpers::*;
use mandi_reports::reports::{
    Cell, PaidFilter, ReportFilters, ReportId, ReportPage, ReportRunInput, run,
};
use mandi_services::core::accounts::{
    AccountInput, AccountRow, AccountType, BalanceSide, OpeningLineInput, OpeningsInput,
};
use mandi_services::core::bikri::{BikriDraft, PaymentType, PukhtaRegisterInput};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::vouchers::ReceiptInput;
use mandi_services::{ServiceError, accounts, assist, bikri, vouchers};
use serde_json::{Value, json};
use support::*;

/// The account form of the screen for a پختہ party (`pukhta` true) or an ordinary account.
fn form(kind: &str, pukhta: bool, ur: &str, en: &str, phone: &str, credit: bool) -> Value {
    json!({
        "id": null, "version": null, "account_type": kind, "is_pukhta_party": pukhta,
        "control_id": null, "code": "", "name_ur": ur, "name_en": en,
        "confirmed_rules_words": [], "father_or_firm_ur": "", "father_or_firm_en": "",
        "city_id": null, "adda_id": null, "phone1": phone, "phone2": "", "address": "",
        "linked_account_id": null, "opening_date": "", "opening_dr": "", "opening_cr": "",
        "opening_is_advance": false, "credit_allowed": credit, "credit_limit": "",
        "default_marka_id": null, "collector_staff_id": null,
        "is_active": true, "note": "", "tag": ""
    })
}

/// Saves a form, confirming the words of the spelling check like the screen does.
fn save_form(w: &World, request: &Value) -> Result<AccountRow, ServiceError> {
    let mut input: AccountInput = serde_json::from_value(request.clone()).unwrap();
    loop {
        match accounts::save(&w.env.ctx, &w.owner, &input) {
            Err(ServiceError::Rejected(items))
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(words)) = item.values.get("words") {
                        input
                            .confirmed_rules_words
                            .extend(words.split(' ').map(str::to_owned));
                    }
                }
            }
            other => return other,
        }
    }
}

fn nazir(w: &World, credit: bool) -> AccountRow {
    save_form(
        w,
        &form("buyer", true, "نذیر", "Nazir", "0300-1111111", credit),
    )
    .unwrap()
}

fn kamal(w: &World) -> AccountRow {
    save_form(w, &form("seller", true, "کمال", "Kamal", "", true)).unwrap()
}

/// PRD 4.5: Karim's 10 سیب, freight 1,000 paid by the firm, labour 5 per نگ, munshiana 50, phone
/// 20, نقد left empty, sold to one buyer at 1,200. The caller chooses the seller and the buyer.
fn event2(w: &World, seller: i64, buyer: i64, payment: Option<PaymentType>) -> BikriDraft {
    let mut line = sale("s-1", buyer, "10", "1200");
    line.payment_type = payment;
    let mut l = lot(w, "lot-1", seller, w.apple, "10");
    l.freight = Some("1,000".into());
    l.cash_paid = None;
    l.sales = vec![line];
    draft(vec![l])
}

fn report(w: &World, id: ReportId, filters: ReportFilters) -> ReportPage {
    run(
        w.env.ctx.db(),
        &ReportRunInput {
            group_by_month: false,
            sort: None,
            id,
            filters,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
}

fn money_of(cell: &Cell) -> i64 {
    match cell {
        Cell::Money { paisa } => paisa.rupees(),
        Cell::Empty => 0,
        other => panic!("not money: {other:?}"),
    }
}

// ---- The account ------------------------------------------------------------------------------

#[test]
fn a_party_gets_the_next_code_of_the_pukhta_series_and_a_buyer_and_a_seller_share_it() {
    let w = world();
    let n = nazir(&w, false);
    assert_eq!((n.code.as_str(), n.serial), ("0032", 2));
    let k = kamal(&w);
    assert_eq!((k.code.as_str(), k.serial), ("0033", 3));
    // Side and mark: a full buyer and a full seller, not the shared account.
    assert_eq!(n.account_type, AccountType::Buyer);
    assert_eq!(k.account_type, AccountType::Seller);
    for row in [&n, &k] {
        assert!(row.is_pukhta_party && !row.is_pukhta_khata && !row.is_system);
        assert!(row.can_take_opening, "a party can have an opening");
    }
    assert_eq!(n.control.code, "0304");
    assert_eq!(k.control.code, "03011");
    // The ordinary series are not touched: the next buyer is 0022 (0020 and 0021 exist).
    let plain = save_form(&w, &form("buyer", false, "طارق", "Tariq", "", true)).unwrap();
    assert_eq!(plain.code, "0022");
    assert!(!plain.is_pukhta_party);
    // The next پختہ code never reuses 0030, 0031, 0032 or 0033.
    let next = accounts::next_code_for(&w.env.ctx, AccountType::Seller, true, None).unwrap();
    assert_eq!(next, "0034");
    // The shared accounts are as they were.
    for id in [w.pukhta_buyer, w.pukhta_seller] {
        let row = accounts::get(&w.env.ctx, id).unwrap();
        assert!(row.is_pukhta_khata && !row.is_pukhta_party);
    }
}

#[test]
fn a_typed_code_must_be_in_the_pukhta_series_and_free() {
    let w = world();
    let typed = |code: &str| {
        let mut f = form("buyer", true, "نذیر", "Nazir", "", false);
        f["code"] = json!(code);
        save_form(&w, &f)
    };
    // The shared accounts hold 0030 and 0031.
    for taken in ["0030", "0031"] {
        let error = typed(taken).unwrap_err();
        assert!(
            rejects(&error, ErrorCode::ErrCodeTaken, "code"),
            "{error:?}"
        );
    }
    // A buyer's or a seller's series is not the پختہ series.
    for wrong in ["0022", "0012", "0040", "03040001"] {
        let error = typed(wrong).unwrap_err();
        assert!(
            rejects(&error, ErrorCode::ErrCodeInvalid, "code"),
            "{wrong}: {error:?}"
        );
    }
    // A free code of the series is taken, and the next one goes on from it.
    assert_eq!(typed("0036").unwrap().code, "0036");
    let next = accounts::next_code_for(&w.env.ctx, AccountType::Buyer, true, None).unwrap();
    assert_eq!(next, "0037");
}

#[test]
fn only_a_seller_or_a_buyer_can_be_a_party_and_the_mark_never_changes() {
    let w = world();
    for kind in ["money_lent", "other"] {
        let mut f = form(kind, true, "اسلم", "Aslam", "", true);
        if kind == "other" {
            f["control_id"] = json!(1);
        }
        let error = save_form(&w, &f).unwrap_err();
        assert!(
            rejects(&error, ErrorCode::ErrFormatInvalid, "is_pukhta_party"),
            "{kind}: {error:?}"
        );
    }
    let error = accounts::next_code_for(&w.env.ctx, AccountType::Other, true, Some(1)).unwrap_err();
    assert!(rejects(
        &error,
        ErrorCode::ErrFormatInvalid,
        "is_pukhta_party"
    ));
    // An edit that tries to change the mark, in either direction, is refused.
    let n = nazir(&w, false);
    let mut edit = form("buyer", false, "نذیر", "Nazir", "0300-1111111", false);
    edit["id"] = json!(n.id);
    edit["version"] = json!(n.version);
    edit["code"] = json!(n.code);
    let error = save_form(&w, &edit).unwrap_err();
    assert!(rejects(
        &error,
        ErrorCode::ErrFormatInvalid,
        "is_pukhta_party"
    ));
    let akram = accounts::get(&w.env.ctx, w.akram).unwrap();
    let mut edit = form("buyer", true, "اکرم ٹول ٹیکس", "Akram Toll Tax", "", true);
    edit["id"] = json!(akram.id);
    edit["version"] = json!(akram.version);
    edit["code"] = json!(akram.code);
    let error = save_form(&w, &edit).unwrap_err();
    assert!(rejects(
        &error,
        ErrorCode::ErrFormatInvalid,
        "is_pukhta_party"
    ));
    // An edit that keeps the mark works, and the credit box can be ticked on later (cash first only
    // sets where it starts).
    let mut edit = form("buyer", true, "نذیر", "Nazir", "0300-1111111", true);
    edit["id"] = json!(n.id);
    edit["version"] = json!(n.version);
    edit["code"] = json!(n.code);
    let again = save_form(&w, &edit).unwrap();
    assert!(again.credit_allowed && again.is_pukhta_party);
}

#[test]
fn a_party_can_take_an_opening_balance_like_any_buyer() {
    // PRD FR-10.9 refuses an opening only on the two shared accounts, not on a party's own.
    let w = world();
    let n = nazir(&w, false);
    let preview = accounts::openings_preview(
        &w.env.ctx,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: n.id,
                version: n.version,
                opening_date: "10-10-2026".into(),
                opening_dr: "1,000".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    assert!(preview.rows[0].errors.is_empty(), "{:?}", preview.rows[0]);
    // The shared account still refuses.
    let shared = accounts::get(&w.env.ctx, w.pukhta_buyer).unwrap();
    assert!(!shared.can_take_opening);
}

// ---- A پختہ گاہک: cash first, then like any buyer ----------------------------------------------

#[test]
fn a_party_buyer_line_starts_as_cash_and_posts_exactly_like_the_shared_account_of_prd_4_5() {
    let w = world();
    let n = nazir(&w, false);
    let d = event2(&w, w.karim, n.id, None);
    let p = preview(&w, &d);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let c = calc(&p.lots[0], 0);
    assert_eq!(c.payment_type, PaymentType::Cash);
    // 10 x 1,200 = 12,000; commission 720; لاگا 80; he pays 12,800.
    assert_eq!(
        (
            r(c.buyer_value),
            r(c.buyer_comm.amount),
            r(c.laga.amount),
            r(c.buyer_total)
        ),
        (12_000, 720, 80, 12_800)
    );
    let saved = save_ok(&w, &d);
    let lines = entry_lines(&w, saved.entry_id);
    // The sale part is the buyer's debit; the cash part pays it back at once.
    assert_eq!(lines[0], ("0032".to_owned(), 12_800, 0, None), "{lines:?}");
    assert!(
        lines.contains(&("0032".to_owned(), 0, 12_800, None)),
        "{lines:?}"
    );
    assert!(lines.contains(&("01010001".to_owned(), 12_800, 0, None)));
    // The whole entry is 25,600 = 25,600 (PRD 4.5), and nothing carries a person key.
    assert_eq!(lines.iter().map(|l| l.1).sum::<i64>(), 25_600, "{lines:?}");
    assert_eq!(lines.iter().map(|l| l.2).sum::<i64>(), 25_600, "{lines:?}");
    assert!(lines.iter().all(|l| l.3.is_none()), "{lines:?}");
    // Karim's credit, the income lines, shared income 1,540 split 616 / 462 / 462.
    assert!(
        lines.contains(&("0011".to_owned(), 0, 10_160, None)),
        "{lines:?}"
    );
    assert_eq!(saved.preview.summary.shared_income.rupees(), 1_540);
    let split: Vec<i64> = saved
        .preview
        .split
        .iter()
        .map(|p| p.income_paisa.rupees())
        .collect();
    assert_eq!(split, [616, 462, 462]);
    // His own balance is 0: bought and paid at once.
    assert_eq!(balance(&w, n.id), (BalanceSide::Zero, 0));
    // Nothing was typed, so the stored line has no person.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert!(detail.lots[0].sales[0].pukhta.is_none());
    assert_totals_equal(&detail.totals, &saved.preview.summary, "a party buyer");
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM bikri_sales WHERE pukhta_person_key IS NOT NULL"
        ),
        0
    );
}

#[test]
fn the_same_sale_to_the_shared_account_and_to_a_party_post_the_same_amounts() {
    // E81: the two are separate records, but the money is the same.
    let w = world();
    let n = nazir(&w, false);
    let party = save_ok(&w, &event2(&w, w.karim, n.id, None));
    let mut typed = event2(&w, w.karim, w.pukhta_buyer, None);
    typed.lots[0].sales[0].pukhta = person("شاہد", "Shahid", "0300-0000000");
    let shared = save_ok(&w, &typed);
    let amounts = |entry_id| -> Vec<(i64, i64)> {
        let mut a: Vec<(i64, i64)> = entry_lines(&w, entry_id)
            .into_iter()
            .map(|l| (l.1, l.2))
            .collect();
        a.sort_unstable();
        a
    };
    assert_eq!(amounts(party.entry_id), amounts(shared.entry_id));
    assert_eq!(
        party.preview.summary.shared_income,
        shared.preview.summary.shared_income
    );
    // The typed person's lines are not on the party's account and the party's are not on the shared one.
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM journal_lines WHERE account_id = {} AND pukhta_person_key IS NOT NULL",
                n.id
            )
        ),
        0
    );
}

#[test]
fn credit_typed_on_a_cash_only_party_is_refused_and_ticking_credit_on_makes_a_normal_credit_buyer()
{
    let w = world();
    let n = nazir(&w, false);
    let credit = event2(&w, w.karim, n.id, Some(PaymentType::Credit));
    let p = preview(&w, &credit);
    assert!(
        p.errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrCreditNotAllowed
                && e.field == "lots[0].sales[0].payment_type"),
        "{:?}",
        p.errors
    );
    // The Owner ticks credit on his account (edit keeps the mark).
    let mut edit = form("buyer", true, "نذیر", "Nazir", "0300-1111111", true);
    edit["id"] = json!(n.id);
    edit["version"] = json!(n.version);
    edit["code"] = json!(n.code);
    save_form(&w, &edit).unwrap();
    // Now the line starts as credit and he owes 12,800 as one bill.
    let d = event2(&w, w.karim, n.id, None);
    assert_eq!(
        calc(&preview(&w, &d).lots[0], 0).payment_type,
        PaymentType::Credit
    );
    let saved = save_ok(&w, &d);
    assert_eq!(balance(&w, n.id), (BalanceSide::Dr, 12_800));
    let lines = entry_lines(&w, saved.entry_id);
    // A credit line has no cash part: the entry is the sale part alone, 12,800 = 12,800.
    assert_eq!(lines.iter().map(|l| l.1).sum::<i64>(), 12_800, "{lines:?}");
    assert!(
        !lines.contains(&("0032".to_owned(), 0, 12_800, None)),
        "{lines:?}"
    );
    // A receipt of 12,800 settles the bill (oldest first, PRD FR-15.4) and his balance is 0.
    let receipt = json!({ "date": "10-10-2026", "account_id": n.id, "person_key": null,
        "amount": "12,800", "collector_staff_id": null, "narration": "" });
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &serde_json::from_value::<ReceiptInput>(receipt).unwrap(),
    )
    .unwrap();
    assert_eq!(balance(&w, n.id), (BalanceSide::Zero, 0));
    let bill_rows = rows(&w, "bill_allocations");
    assert_eq!(
        bill_rows.iter().map(|r| r.2).sum::<i64>(),
        12_800,
        "{bill_rows:?}"
    );
}

// ---- A پختہ بیوپاری: paid on the spot ----------------------------------------------------------

#[test]
fn a_party_seller_is_paid_his_net_on_the_spot_and_his_balance_is_zero() {
    let w = world();
    let k = kamal(&w);
    let d = event2(&w, k.id, w.akram, None);
    let p = preview(&w, &d);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let saved = save_ok(&w, &d);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let lot = &detail.lots[0];
    // PRD FR-13.10 numbers: gross 12,000, deductions before نقد 1,840, so نقد is 10,160 and the net 0
    // (the old video's lot with کل خرچہ equal to خام بکری).
    assert_eq!((lot.cash_paid.rupees(), lot.net.rupees()), (10_160, 0));
    assert!(lot.pukhta.is_none(), "no person is typed on his lot");
    assert_totals_equal(&detail.totals, &saved.preview.summary, "a party seller");
    let lines = entry_lines(&w, saved.entry_id);
    // His credit of the sale part and the cash part that pays it out (the first party made in this
    // world, so his code is 0032).
    assert_eq!(k.code, "0032");
    assert!(
        lines.contains(&("0032".to_owned(), 0, 10_160, None)),
        "{lines:?}"
    );
    assert!(
        lines.contains(&("0032".to_owned(), 10_160, 0, None)),
        "{lines:?}"
    );
    assert_eq!(
        lines.iter().map(|l| l.1).sum::<i64>(),
        lines.iter().map(|l| l.2).sum::<i64>()
    );
    assert!(lines.iter().all(|l| l.3.is_none()), "{lines:?}");
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
}

#[test]
fn a_typed_cash_wins_over_the_default_and_a_typed_zero_is_not_empty() {
    let w = world();
    let k = kamal(&w);
    // نقد 5,000 typed: he is paid 5,000 and 5,160 stays to pay him.
    let mut d = event2(&w, k.id, w.akram, None);
    d.lots[0].cash_paid = Some("5,000".into());
    let saved = save_ok(&w, &d);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(
        (
            detail.lots[0].cash_paid.rupees(),
            detail.lots[0].net.rupees()
        ),
        (5_000, 5_160)
    );
    assert_eq!(balance(&w, k.id), (BalanceSide::Cr, 5_160));
    // A typed 0 is a typed value: nothing is paid on the spot.
    let w = world();
    let k = kamal(&w);
    let mut d = event2(&w, k.id, w.akram, None);
    d.lots[0].cash_paid = Some("0".into());
    let saved = save_ok(&w, &d);
    assert_eq!(
        bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0]
            .cash_paid
            .rupees(),
        0
    );
    assert_eq!(balance(&w, k.id), (BalanceSide::Cr, 10_160));
}

#[test]
fn an_ordinary_seller_with_no_cash_typed_is_still_paid_nothing_on_the_spot() {
    let w = world();
    let saved = save_ok(&w, &event2(&w, w.karim, w.akram, None));
    let lot = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0];
    assert_eq!(lot.cash_paid.rupees(), 0);
    assert_eq!(balance(&w, w.karim), (BalanceSide::Cr, 10_160));
}

#[test]
fn editing_a_party_sellers_lot_pays_the_new_net_on_the_spot() {
    // The edit form gets the field back empty, so the default follows the new net: the rate goes
    // from 1,200 to 1,300, gross 13,000, commission 780, deductions before نقد 1,900, credit 11,100.
    let w = world();
    let k = kamal(&w);
    let saved = save_ok(&w, &event2(&w, k.id, w.akram, None));
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    assert_eq!(
        opened.draft.lots[0].cash_paid, None,
        "the field comes back empty"
    );
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "1300".into();
    tools::edit(&w, &opened, &d);
    let lot = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0];
    assert_eq!((lot.cash_paid.rupees(), lot.net.rupees()), (11_100, 0));
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
    // A partial payment typed by the Owner is kept on an edit.
    let w = world();
    let k = kamal(&w);
    let mut first = event2(&w, k.id, w.akram, None);
    first.lots[0].cash_paid = Some("5,000".into());
    let saved = save_ok(&w, &first);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    assert_eq!(opened.draft.lots[0].cash_paid.as_deref(), Some("5000"));
}

// ---- A پختہ بیوپاری with an open advance (PRD FR-13.11, A79, Umar 01-11-2026) --------------------

/// How much advance the card of an account shows as open, in rupees.
fn advance_open(w: &World, id: i64) -> i64 {
    assist::party_card(&w.env.ctx, id)
        .unwrap()
        .advance_open
        .expect("a seller's card shows his advance")
        .rupees()
}

#[test]
fn a_party_seller_with_an_advance_is_paid_the_credit_less_the_advance_and_ends_at_zero() {
    // The credit 10,160 (PRD 4.5) adjusts his open advance of 5,000, so the cash first default is
    // 10,160 - 5,000 = 5,160 and his balance is 0 after the save: 5,000 + 5,160 = 10,160.
    let w = world();
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    assert_eq!(balance(&w, k.id), (BalanceSide::Dr, 5_000));
    let d = event2(&w, k.id, w.akram, None);
    let shown = preview(&w, &d);
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    assert_eq!(
        (r(shown.summary.cash_paid), r(shown.summary.net)),
        (5_160, 5_000),
        "the preview shows the default before the save"
    );
    // The cash check (FR-6.7) sees the same amount that is paid: 5,160 goes out, nothing comes in.
    assert_eq!(r(shown.cash.change_paisa), -5_160);
    let saved = save_ok(&w, &d);
    assert_eq!(
        saved.preview, shown,
        "the save is the preview (Code Standards B5)"
    );
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert_eq!(
        (
            detail.lots[0].cash_paid.rupees(),
            detail.lots[0].net.rupees()
        ),
        (5_160, 5_000)
    );
    assert_totals_equal(&detail.totals, &saved.preview.summary, "advance and cash");
    let lines = entry_lines(&w, saved.entry_id);
    assert!(
        lines.contains(&("0032".to_owned(), 0, 10_160, None)),
        "{lines:?}"
    );
    assert!(
        lines.contains(&("0032".to_owned(), 5_160, 0, None)),
        "{lines:?}"
    );
    assert_eq!(
        lines.iter().map(|l| l.1).sum::<i64>(),
        lines.iter().map(|l| l.2).sum::<i64>()
    );
    // The credit adjusted the advance, all of it; his ledger is 0 and nothing is open.
    assert_eq!(
        rows(&w, "advance_allocations")
            .iter()
            .map(|r| r.2)
            .sum::<i64>(),
        5_000
    );
    assert_eq!(advance_open(&w, k.id), 0);
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
    // The working says how the default came about.
    assert_eq!(
        shown.lots[0]
            .totals
            .as_ref()
            .map(|t| t.workings.cash_paid.message_id.as_str()),
        Some("working-cash-pukhta-advance")
    );
}

#[test]
fn with_the_tick_off_the_advance_stays_open_and_the_cash_is_the_whole_credit() {
    let w = world();
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    let mut d = event2(&w, k.id, w.akram, None);
    d.lots[0].adjust_advance = false;
    let saved = save_ok(&w, &d);
    let lot = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0];
    assert_eq!((lot.cash_paid.rupees(), lot.net.rupees()), (10_160, 0));
    // The advance is still his: open 5,000, and his ledger shows it as 5,000 بنام.
    assert_eq!(advance_open(&w, k.id), 5_000);
    assert_eq!(balance(&w, k.id), (BalanceSide::Dr, 5_000));
    assert!(rows(&w, "advance_allocations").is_empty());
}

#[test]
fn an_advance_bigger_than_the_credit_leaves_no_cash_and_the_rest_of_it_open() {
    let w = world();
    let k = kamal(&w);
    give_advance(&w, k.id, "20,000");
    let saved = save_ok(&w, &event2(&w, k.id, w.akram, None));
    let lot = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0];
    assert_eq!((lot.cash_paid.rupees(), lot.net.rupees()), (0, 10_160));
    // 10,160 of the 20,000 is adjusted; 9,840 is still open and is what he owes the firm.
    assert_eq!(advance_open(&w, k.id), 9_840);
    assert_eq!(balance(&w, k.id), (BalanceSide::Dr, 9_840));
    let lines = entry_lines(&w, saved.entry_id);
    assert!(
        lines.iter().all(|l| !(l.0 == "0032" && l.1 > 0)),
        "no cash part for him: {lines:?}"
    );
}

#[test]
fn editing_the_lot_pays_the_new_credit_less_the_advance() {
    // The rate goes from 1,200 to 1,300: credit 11,100, advance 5,000, cash 6,100, balance 0.
    let w = world();
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    let saved = save_ok(&w, &event2(&w, k.id, w.akram, None));
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    assert_eq!(
        opened.draft.lots[0].cash_paid, None,
        "the default comes back empty, so it follows the new credit"
    );
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "1300".into();
    tools::edit(&w, &opened, &d);
    let lot = &bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0];
    assert_eq!((lot.cash_paid.rupees(), lot.net.rupees()), (6_100, 5_000));
    assert_eq!(advance_open(&w, k.id), 0);
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
    // A typed 0 is kept as typed on an edit, even when the default is 0 too (the advance covers the
    // credit): an edit never turns "pay nothing now" into a payment.
    let w = world();
    let k = kamal(&w);
    give_advance(&w, k.id, "20,000");
    let mut zero = event2(&w, k.id, w.akram, None);
    zero.lots[0].cash_paid = Some("0".into());
    let saved = save_ok(&w, &zero);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    assert_eq!(opened.draft.lots[0].cash_paid.as_deref(), Some("0"));
    // A cash the Owner typed that is not the default is kept on an edit.
    let w = world();
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    let mut typed = event2(&w, k.id, w.akram, None);
    typed.lots[0].cash_paid = Some("3,000".into());
    let saved = save_ok(&w, &typed);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    assert_eq!(opened.draft.lots[0].cash_paid.as_deref(), Some("3000"));
}

#[test]
fn two_lots_of_one_party_seller_share_his_advance_in_lot_order() {
    // Advance 12,000; credit 10,160 each: lot 1 uses 10,160 (cash 0), lot 2 the 1,840 left (cash
    // 8,320). His ledger: 12,000 + 8,320 - 20,320 = 0.
    let w = world();
    let k = kamal(&w);
    give_advance(&w, k.id, "12,000");
    let second = {
        let mut line = sale("s-2", w.arshad, "10", "1200");
        line.payment_type = None;
        let mut l = lot(&w, "lot-2", k.id, w.apple, "10");
        l.freight = Some("1,000".into());
        l.cash_paid = None;
        l.sales = vec![line];
        l
    };
    let mut d = event2(&w, k.id, w.akram, None);
    d.lots.push(second);
    let saved = save_ok(&w, &d);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let cash: Vec<i64> = detail.lots.iter().map(|l| l.cash_paid.rupees()).collect();
    assert_eq!(cash, [0, 8_320]);
    assert_eq!(advance_open(&w, k.id), 0);
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
}

#[test]
fn undo_gives_the_advance_back_and_the_new_draft_follows_the_default_again() {
    // Undo (PRD FR-7.47) cancels the posting by reversal, so the advance is open again, and opens
    // the same values as a new unsaved بکری with the cash left empty, so the default is worked out
    // again on the save.
    let w = world();
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    let saved = save_ok(&w, &event2(&w, k.id, w.akram, None));
    assert_eq!(advance_open(&w, k.id), 0);
    let undone = bikri::undo(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::UndoInput {
            id: saved.bikri_id,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert_eq!(undone.draft.lots[0].cash_paid, None);
    assert_eq!(advance_open(&w, k.id), 5_000);
    assert_eq!(balance(&w, k.id), (BalanceSide::Dr, 5_000));
    // Saving the draft again gives the same numbers as the first time.
    let again = save_ok(&w, &undone.draft);
    let lot = &bikri::get(&w.env.ctx, again.bikri_id).unwrap().lots[0];
    assert_eq!((lot.cash_paid.rupees(), lot.net.rupees()), (5_160, 5_000));
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
}

// ---- The advance taken from a LATER posting: a warning (Umar, option 2) -------------------------

/// The same lot as `event2` on another day.
fn event2_on(w: &World, day: &str, seller: i64, buyer: i64) -> BikriDraft {
    let mut d = event2(w, seller, buyer, None);
    d.header.entry_date = day.into();
    d
}

/// The `taken_paisa` of the advance warning of a preview, in rupees, with the field it sits on.
fn taken_warning(warnings: &[mandi_services::core::error::FieldError]) -> Option<(String, i64)> {
    warnings
        .iter()
        .find(|e| e.code == ErrorCode::ErrAdvanceTakenFromLater)
        .map(|e| {
            let taken = match e.values.get("taken_paisa") {
                Some(ErrorValue::Number(n)) => *n / 100,
                other => panic!("taken_paisa is missing: {other:?}"),
            };
            (e.field.clone(), taken)
        })
}

#[test]
fn a_back_dated_posting_that_takes_the_advance_of_a_later_one_warns_with_the_amount() {
    // Advance 5,000 (10-10). The بکری of 12-10 is paid 5,160 and ends at 0. A back-dated بکری of
    // 11-10 comes before it, so it gets the advance and the 12-10 posting loses its 5,000: the
    // Owner is told X = 5,000 on the نقد, the default stays 5,160, and his balance is not 0 after.
    let w = world();
    common::set_today(&w.env, 2026, 10, 12);
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    let first = save_ok(&w, &event2_on(&w, "12-10-2026", k.id, w.akram));
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
    // The newest posting takes nothing from anybody: no warning.
    assert_eq!(
        taken_warning(&preview(&w, &event2_on(&w, "12-10-2026", k.id, w.arshad)).warnings),
        None
    );
    // The back-dated one warns.
    let back = event2_on(&w, "11-10-2026", k.id, w.arshad);
    let shown = preview(&w, &back);
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    assert_eq!(
        taken_warning(&shown.warnings),
        Some(("lots[0].cash_paid".to_owned(), 5_000))
    );
    assert_eq!(r(shown.summary.cash_paid), 5_160, "the default is the same");
    // The save is the preview, warning included, and nothing blocks it.
    let saved = save_ok(&w, &back);
    assert_eq!(saved.preview, shown);
    // What the warning promised: the 11-10 credit holds the advance, the 12-10 one lost it, so the
    // firm owes him 5,000 (paid 15,320 against two credits of 20,320). Books balance.
    assert_eq!(advance_open(&w, k.id), 0);
    assert_eq!(balance(&w, k.id), (BalanceSide::Cr, 5_000));
    let _ = first;
}

#[test]
fn editing_an_earlier_posting_warns_when_it_takes_more_advance_from_a_later_one() {
    // Advance 12,000. B1 (credit 10,160) takes 10,160 and is paid 0; B2 takes the 1,840 left and is
    // paid 8,320. B1 is edited to rate 1,400: credit 12,040, it now takes all 12,000, so B2 loses
    // 1,840: X = 1,840. Both on 12-10, B1 saved first so its place is earlier.
    let w = world();
    common::set_today(&w.env, 2026, 10, 12);
    let k = kamal(&w);
    give_advance(&w, k.id, "12,000");
    let b1 = save_ok(&w, &event2_on(&w, "12-10-2026", k.id, w.akram));
    let _b2 = save_ok(&w, &event2_on(&w, "12-10-2026", k.id, w.arshad));
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
    let opened = tools::open(&w, b1.bikri_id, b1.entry_id);
    // Opened as it was saved: the same credit, nothing is taken from B2.
    let same = tools::preview(&w, &opened, &opened.draft);
    assert_eq!(taken_warning(&same.warnings), None);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "1400".into();
    let shown = tools::preview(&w, &opened, &d);
    assert_eq!(
        taken_warning(&shown.warnings),
        Some(("lots[0].cash_paid".to_owned(), 1_840))
    );
    // A smaller credit takes less: the later posting gains, nothing is taken, no warning.
    let mut smaller = opened.draft.clone();
    smaller.lots[0].sales[0].seller_rate = "1100".into();
    assert_eq!(
        taken_warning(&tools::preview(&w, &opened, &smaller).warnings),
        None
    );
    // The edit goes through with the same preview.
    let edited = tools::edit(&w, &opened, &d);
    assert_eq!(edited.preview, shown);
    // What the warning promised, read from the books: B1 now holds all 12,000, B2 holds nothing but
    // keeps the 8,320 it was paid, so the firm owes him X = 1,840. B1's cash was 0 and stays 0 (a 0
    // reopens as typed, so an edit never turns "pay nothing now" into a payment): the 40 of its new
    // credit above the advance is not paid either, which makes 1,840 + 40 = 1,880 in all.
    assert_eq!(advance_open(&w, k.id), 0);
    assert_eq!(balance(&w, k.id), (BalanceSide::Cr, 1_880));
}

#[test]
fn a_chain_of_later_postings_all_lose_to_the_new_one_and_x_is_the_sum() {
    // Advance 5,000. P1 (12-10, credit 3,170) holds 3,170 and P2 (14-10, credit 11,160) the 1,830
    // left. A back-dated posting of 11-10 with credit 4,580 comes first and takes 4,580: P1 keeps
    // 420 (loses 2,750) and P2 keeps 0 (loses 1,830). X = 2,750 + 1,830 = 4,580. Afterwards the firm
    // owes him X: advance 5,000 + cash 0 + 9,330 + 0 - credits 18,910.
    let w = world();
    common::set_today(&w.env, 2026, 10, 14);
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    let priced = |day: &str, rate: &str, buyer: i64| {
        let mut d = event2_on(&w, day, k.id, buyer);
        d.lots[0].freight = None;
        d.lots[0].sales[0].seller_rate = rate.into();
        d
    };
    save_ok(&w, &priced("12-10-2026", "350", w.akram));
    save_ok(&w, &priced("14-10-2026", "1200", w.arshad));
    assert_eq!(advance_open(&w, k.id), 0);
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
    let back = priced("11-10-2026", "500", w.akram);
    let shown = preview(&w, &back);
    assert_eq!(
        taken_warning(&shown.warnings),
        Some(("lots[0].cash_paid".to_owned(), 4_580))
    );
    assert_eq!(r(shown.summary.seller_credit), 4_580);
    assert_eq!(
        r(shown.summary.cash_paid),
        0,
        "the new credit takes the advance first"
    );
    let saved = save_ok(&w, &back);
    assert_eq!(saved.preview, shown);
    assert_eq!(advance_open(&w, k.id), 0);
    assert_eq!(balance(&w, k.id), (BalanceSide::Cr, 4_580));
}

#[test]
fn an_advance_given_after_the_new_posting_is_not_its_to_take() {
    // The advance is given on 13-10 and held by the posting of 14-10. A posting of 11-10 comes
    // before the advance, so it can take none of it: no cash is saved by it, and no warning.
    let w = world();
    common::set_today(&w.env, 2026, 10, 14);
    let k = kamal(&w);
    helpers::give_advance_on(&w, k.id, "5,000", "13-10-2026");
    save_ok(&w, &event2_on(&w, "14-10-2026", k.id, w.akram));
    assert_eq!(balance(&w, k.id), (BalanceSide::Zero, 0));
    let shown = preview(&w, &event2_on(&w, "11-10-2026", k.id, w.arshad));
    assert_eq!(taken_warning(&shown.warnings), None);
    assert_eq!(
        r(shown.summary.cash_paid),
        10_160,
        "no advance is free before 13-10"
    );
}

#[test]
fn no_advance_warning_for_the_common_cases() {
    let w = world();
    common::set_today(&w.env, 2026, 10, 12);
    let k = kamal(&w);
    // No advance at all.
    assert_eq!(
        taken_warning(&preview(&w, &event2_on(&w, "11-10-2026", k.id, w.akram)).warnings),
        None
    );
    give_advance(&w, k.id, "5,000");
    save_ok(&w, &event2_on(&w, "12-10-2026", k.id, w.akram));
    // Tick off: the back-dated lot takes no advance.
    let mut off = event2_on(&w, "11-10-2026", k.id, w.arshad);
    off.lots[0].adjust_advance = false;
    assert_eq!(taken_warning(&preview(&w, &off).warnings), None);
    // An ordinary seller with an advance has no cash default and no warning.
    give_advance(&w, w.karim, "5,000");
    save_ok(&w, &event2_on(&w, "12-10-2026", w.karim, w.akram));
    assert_eq!(
        taken_warning(&preview(&w, &event2_on(&w, "11-10-2026", w.karim, w.arshad)).warnings),
        None
    );
}

#[test]
fn two_lots_of_one_seller_warn_once_with_the_total_on_the_first_lot() {
    let w = world();
    common::set_today(&w.env, 2026, 10, 12);
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    save_ok(&w, &event2_on(&w, "12-10-2026", k.id, w.akram));
    let mut d = event2_on(&w, "11-10-2026", k.id, w.arshad);
    let mut second = lot(&w, "lot-2", k.id, w.apple, "10");
    second.freight = Some("1,000".into());
    second.cash_paid = None;
    second.sales = vec![sale("s-2", w.akram, "10", "1200")];
    d.lots.push(second);
    let shown = preview(&w, &d);
    let warnings: Vec<_> = shown
        .warnings
        .iter()
        .filter(|e| e.code == ErrorCode::ErrAdvanceTakenFromLater)
        .collect();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(
        taken_warning(&shown.warnings),
        Some(("lots[0].cash_paid".to_owned(), 5_000))
    );
}

/// What the active advance rows give to the credit lines of one entry, in rupees.
fn held_by_entry(w: &World, entry_id: i64) -> i64 {
    raw_conn(w)
        .query_row(
            "SELECT COALESCE(SUM(a.amount_paisa), 0) / 100 FROM advance_allocations a
             JOIN journal_lines l ON l.id = a.credit_line_id
             WHERE a.is_active = 1 AND l.entry_id = ?1",
            [entry_id],
            |r| r.get(0),
        )
        .unwrap()
}

fn raw_conn(w: &World) -> mandi_services::db::rusqlite::Connection {
    common::raw(&w.dir)
}

#[test]
fn a_seller_receipt_among_the_later_credits_loses_advance_too_and_x_counts_it() {
    // PRD FR-15.12: کمال returns 3,000 of his 5,000 advance in cash on 12-10; the receipt is a credit
    // that holds 3,000 of it. A back-dated بکری of 11-10 (credit 4,580) comes before the receipt and
    // takes 4,580 first: the receipt keeps 420 and loses 2,580 = X. The ledger afterwards: advance
    // 5,000 - receipt 3,000 - credit 4,580 (no cash) = 2,580 جمع, as the warning says.
    let w = world();
    common::set_today(&w.env, 2026, 10, 12);
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    let receipt = json!({ "date": "12-10-2026", "account_id": k.id, "person_key": null,
        "amount": "3,000", "collector_staff_id": null, "narration": "" });
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &serde_json::from_value::<ReceiptInput>(receipt).unwrap(),
    )
    .unwrap();
    assert_eq!(advance_open(&w, k.id), 2_000);
    let mut back = event2_on(&w, "11-10-2026", k.id, w.akram);
    back.lots[0].freight = None;
    back.lots[0].sales[0].seller_rate = "500".into();
    let shown = preview(&w, &back);
    assert_eq!(r(shown.summary.seller_credit), 4_580);
    assert_eq!(
        taken_warning(&shown.warnings),
        Some(("lots[0].cash_paid".to_owned(), 2_580))
    );
    assert_eq!(
        r(shown.summary.cash_paid),
        0,
        "5,000 is free for it, more than its credit"
    );
    let saved = save_ok(&w, &back);
    assert_eq!(saved.preview, shown);
    assert_eq!(advance_open(&w, k.id), 0);
    assert_eq!(balance(&w, k.id), (BalanceSide::Cr, 2_580));
    assert_eq!(held_by_entry(&w, saved.entry_id), 4_580);
}

#[test]
fn a_back_dated_later_sale_shifts_the_advance_with_no_warning_today() {
    // PINS TODAY'S BEHAVIOUR until Umar decides (money agent review 3, (c)1; the question is open):
    // a later sale of a lot also adjusts the advance (FR-12.23), so one dated before another posting
    // of the same seller takes that posting's advance, and no warning is shown (the warning is for
    // first postings only). Books balance; nothing else moves.
    let w = world();
    common::set_today(&w.env, 2026, 10, 12);
    let k = kamal(&w);
    give_advance(&w, k.id, "5,000");
    // P1 (10-10): 20 received, 2 sold, credit 1,086, holds 1,086.
    let mut first = event2_on(&w, "10-10-2026", k.id, w.akram);
    first.lots[0].qty_received = "20".into();
    first.lots[0].sales[0].qty = "2".into();
    let p1 = save_ok(&w, &first);
    assert_eq!(held_by_entry(&w, p1.entry_id), 1_086);
    // P2 (12-10): credit 10,160, holds the 3,914 left.
    let p2 = save_ok(&w, &event2_on(&w, "12-10-2026", k.id, w.arshad));
    assert_eq!(held_by_entry(&w, p2.entry_id), 3_914);
    // The later sale of P1's lot, dated 11-10 (before P2): 10 more at 1,200, credit 11,280.
    let lot_id = bikri::get(&w.env.ctx, p1.bikri_id).unwrap().lots[0].id;
    let mut line = sale("s-3", w.akram, "10", "1200");
    line.payment_type = None;
    let input = mandi_services::core::bikri::LaterSaleInput {
        lot_id,
        date: "11-10-2026".into(),
        sales: vec![line.clone()],
    };
    let shown = bikri::later_sale_preview(&w.env.ctx, &input).unwrap();
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    assert_eq!(
        taken_warning(&shown.warnings),
        None,
        "no warning for a later sale today"
    );
    let done = bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::LaterSaleSaveInput {
            lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // The later sale's credit took P2's 3,914; P2 holds nothing now and was paid 6,246 in cash.
    assert_eq!(held_by_entry(&w, done.entry_id), 3_914);
    assert_eq!(held_by_entry(&w, p2.entry_id), 0);
    assert_eq!(advance_open(&w, k.id), 0);
    assert_eq!(balance(&w, k.id), (BalanceSide::Cr, 11_280));
}

// ---- The register R9 and the screen of the same lines -------------------------------------------

#[test]
fn r9_and_the_register_list_a_partys_lines_next_to_the_typed_people() {
    let w = world();
    let n = nazir(&w, false);
    let k = kamal(&w);
    // One typed person (شاہد) and one party buyer on the same day; a party seller's lot on the same
    // بکری as the party buyer's line.
    let mut typed = event2(&w, w.karim, w.pukhta_buyer, None);
    typed.lots[0].sales[0].pukhta = person("شاہد", "Shahid", "0300-0000000");
    save_ok(&w, &typed);
    save_ok(&w, &event2(&w, k.id, n.id, None));

    let page = report(&w, ReportId::R9, ReportFilters::default());
    // The typed person's buyer line, the party buyer's line and the party seller's lot.
    assert_eq!(page.total_count, 3);
    let party_of = |row: usize| match &page.rows[row].cells[3] {
        Cell::Party { party } => (
            party.code.clone(),
            party.name_en.clone(),
            party.person_key.clone(),
        ),
        other => panic!("not a party: {other:?}"),
    };
    // Rows are oldest first, then by بکری number: row 0 is the typed person.
    let (code, name, key) = party_of(0);
    assert_eq!((code.as_str(), name.as_str()), ("0030", "Shahid"));
    assert!(key.is_some());
    // The party buyer shows his account code and name, and no person key.
    let (code, name, key) = party_of(1);
    assert_eq!((code.as_str(), name.as_str()), ("0032", "Nazir"));
    assert!(key.is_none());
    let (code, name, key) = party_of(2);
    assert_eq!((code.as_str(), name.as_str()), ("0033", "Kamal"));
    assert!(key.is_none());
    // His line: 12,800, commission 720, bought and paid at once. The seller lot: gross 12,000,
    // commission 720, nothing open.
    let status = |row: usize| match &page.rows[row].cells[9] {
        Cell::Label { id } => id.clone(),
        other => panic!("not a label: {other:?}"),
    };
    assert_eq!(
        (
            money_of(&page.rows[1].cells[7]),
            money_of(&page.rows[1].cells[8])
        ),
        (12_800, 720)
    );
    assert_eq!(status(1), "report-status-paid");
    assert_eq!(
        (
            money_of(&page.rows[2].cells[7]),
            money_of(&page.rows[2].cells[8])
        ),
        (12_000, 720)
    );
    assert_eq!(status(2), "report-status-paid");
    // The totals cover all three rows (3 x 10 نگ).
    assert_eq!(money_of(&page.totals[7]), 12_800 + 12_800 + 12_000);
    // The party filter takes his account, and a typed person's filter leaves him out.
    let only = report(
        &w,
        ReportId::R9,
        ReportFilters {
            party: Some(mandi_reports::reports::PartyFilter {
                account_id: n.id,
                person_key: None,
            }),
            ..ReportFilters::default()
        },
    );
    assert_eq!(only.total_count, 1);
    // A credit party with a bill open shows as due, and the paid filter leaves it out.
    let mut edit = form("buyer", true, "نذیر", "Nazir", "0300-1111111", true);
    edit["id"] = json!(n.id);
    edit["version"] = json!(accounts::get(&w.env.ctx, n.id).unwrap().version);
    edit["code"] = json!(n.code);
    save_form(&w, &edit).unwrap();
    save_ok(&w, &event2(&w, w.karim, n.id, None));
    let due = report(
        &w,
        ReportId::R9,
        ReportFilters {
            status: Some(PaidFilter::Due),
            ..ReportFilters::default()
        },
    );
    assert_eq!(due.total_count, 1);
    assert_eq!(money_of(&due.rows[0].cells[10]), 12_800);
    // The register of the screen lists the same lines as R9 and agrees on paid or due.
    let register = bikri::register(
        &w.env.ctx,
        &PukhtaRegisterInput {
            from: String::new(),
            to: String::new(),
            side: None,
            name: None,
            phone: None,
            search: None,
            state: None,
            show_finished: true,
            group_by_month: false,
            sort: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(register.total_count, 4);
    let due_rows: Vec<_> = register.rows.iter().filter(|r| !r.paid).collect();
    assert_eq!(due_rows.len(), 1);
    assert_eq!(
        (due_rows[0].open.rupees(), due_rows[0].name_en.as_str()),
        (12_800, "Nazir")
    );
    assert!(
        due_rows[0].person_key.is_empty(),
        "a party has no person key"
    );
}
