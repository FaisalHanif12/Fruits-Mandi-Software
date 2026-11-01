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
        "default_marka_id": null, "collector_staff_id": null, "collection_weekday": null,
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
