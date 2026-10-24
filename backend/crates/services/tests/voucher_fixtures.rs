//! Real answers of the plan 15 commands, as JSON files for the frontend (plan 15, T4).
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used), and `<name>.json` is the answer the
//! real service gives for it on the PRD worked day. A refusal is a pair too: its answer has the
//! shape of the error a command returns (`ok`, `errors`, `ref`), where `ref` is a stand-in
//! because the real reference number is a counter. The frontend copies the files to
//! `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ, so a fixture can
//! never go stale. After a deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test voucher_fixtures`, look at the
//! diff, and tell `frontend`.
//!
//! The story is the worked day of the PRD: an advance to رحیم, Event 1, then the receipt, the
//! payment and the cash lent of Events 3, 4 and E2E-32, and the rest of plan 15. From plan 17b
//! (PRD 15.1, v1.6) a voucher has no mode, bank or cheque: all money is روکڑ, and a request that
//! still carries one of them is a refusal fixture.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use mandi_services::core::accounts::{AccountInput, SearchGlobalInput};
use mandi_services::core::bikri::{BikriSaveInput, LineChargeMethod, PaymentType, PukhtaInput};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::vouchers::{
    AdvanceInput, AdvancesReportInput, AssistAdvancesInput, CollectionListInput, MakeRegularInput,
    OpenBillsInput, OpenPayablesInput, PaymentInput, ReceiptInput, ReceiptsInput,
    VoucherCancelInput, VoucherEditInput, VoucherGetInput, VoucherPreviewInput, VouchersListInput,
};
use mandi_services::{ServiceError, bikri, search, vouchers};
use serde::Serialize;
use serde_json::{Value, json};
use support::World;

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Writes the file (when asked to) or checks that it holds exactly this text.
fn write_or_check(file: &str, text: &str) {
    let path = folder().join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::create_dir_all(folder()).unwrap();
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
fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
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

/// A pair for a refusal; checks the code too.
fn refusal(name: &str, request: &Value, error: &ServiceError, code: ErrorCode) {
    assert_eq!(error.code(), code, "{name}");
    pair(name, request, &wire(error));
}

/// The request read back into the real input type, so the file is exactly what the command takes.
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

/// Repeats a save that asks the user to confirm rule-made spellings (PRD FR-7.66), confirming
/// what it asks, and gives back the answer and the words that were confirmed.
fn confirming<T>(
    mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>,
) -> (T, Vec<String>) {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match attempt(&words) {
            Ok(done) => return (done, words),
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

// ---- The JSON of the entries, as the screen sends them ----

fn receipt_json(account: i64, amount: &str) -> Value {
    json!({
        "date": "10-10-2026", "account_id": account, "person_key": null, "amount": amount,
        "collector_staff_id": null, "narration": ""
    })
}

fn payment_json(account: i64, amount: &str, reason: &str) -> Value {
    json!({
        "date": "10-10-2026", "account_id": account, "person_key": null, "amount": amount,
        "reason": reason, "narration": "", "confirm_cash_below_zero": false
    })
}

fn advance_json(seller: i64, amount: &str) -> Value {
    json!({
        "date": "10-10-2026", "seller_account_id": seller, "amount": amount,
        "expected_goods": "", "narration": "", "confirm_cash_below_zero": false
    })
}

/// The entry of a preview: the same fields with the type of the voucher.
fn entry(kind: &str, fields: &Value) -> Value {
    let mut value = fields.clone();
    value["type"] = json!(kind);
    value
}

fn preview_request(kind: &str, fields: &Value, replacing: Option<i64>) -> Value {
    json!({ "entry": entry(kind, fields), "replacing_voucher_id": replacing })
}

/// The form of an existing account, to change one field of it.
fn account_form(w: &World, id: i64) -> AccountInput {
    let row = mandi_services::accounts::get(&w.env.ctx, id).unwrap();
    AccountInput {
        id: Some(id),
        version: Some(row.version),
        account_type: mandi_services::core::accounts::AccountType::Buyer,
        control_id: None,
        code: row.code.clone(),
        name_ur: row.name_ur.clone(),
        name_en: row.name_en.clone(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: row.father_or_firm_ur.clone(),
        father_or_firm_en: row.father_or_firm_en.clone(),
        city_id: row.city.as_ref().map(|c| c.id),
        adda_id: None,
        phone1: row.phone1.clone(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: row.credit_allowed,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
    }
}

fn make_regular_request(key: &str, words: &[String]) -> Value {
    json!({
        "person_key": key,
        "side": "buyer",
        "account_input": {
            "id": null, "version": null, "account_type": "buyer", "control_id": null,
            "code": "", "name_ur": "شاہد", "name_en": "Shahid", "confirmed_rules_words": words,
            "father_or_firm_ur": "", "father_or_firm_en": "", "city_id": null, "adda_id": null,
            "phone1": "03000000000", "phone2": "", "address": "", "linked_account_id": null,
            "opening_date": "", "opening_dr": "", "opening_cr": "", "opening_is_advance": false,
            "credit_allowed": true, "credit_limit": "", "default_marka_id": null,
            "collector_staff_id": null, "collection_weekday": null, "is_active": true, "note": ""
        }
    })
}

/// A بکری whose one buyer is the one-time person Shahid on credit, at no commission.
fn one_time_buyer_owes(w: &World) {
    let mut line = support::sale("s-1", w.pukhta_buyer, "2", "1000");
    line.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    line.payment_type = Some(PaymentType::Credit);
    line.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
    line.seller_comm = support::typed(LineChargeMethod::Percent, "0");
    line.laga = support::typed(LineChargeMethod::PerUnit, "0");
    let mut lot = support::bare_lot(w, "lot-1", w.rahim, w.banana, "2");
    lot.sales = vec![line];
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::draft(vec![lot]),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

#[test]
fn the_real_answers_of_the_plan_15_commands_are_the_fixtures() {
    let w = support::world();
    let (ctx, owner) = (&w.env.ctx, &w.owner);
    let preview = |kind: &str, fields: &Value, replacing: Option<i64>| {
        let request = preview_request(kind, fields, replacing);
        let answer = vouchers::preview(ctx, &typed::<VoucherPreviewInput>(&request)).unwrap();
        (request, answer)
    };

    // 1. An advance to رحیم (E2E-7, پیشگی): the preview, then the save.
    let advance = advance_json(w.rahim, "15,000");
    let (request, answer) = preview("advance", &advance, None);
    assert!(answer.errors.is_empty());
    pair("vouchers_preview_advance", &request, &answer);
    let advance_saved =
        vouchers::save_advance(ctx, owner, &typed::<AdvanceInput>(&advance)).unwrap();
    pair("vouchers_save_advance", &advance, &advance_saved);

    // 2. Event 1 (PRD 4.4): اکرم owes 12,880, رحیم is owed 24,316 less his advance and نقد.
    bikri::save(
        ctx,
        owner,
        &BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let bills_request = json!({ "account_id": w.akram, "person_key": null });
    let bills = vouchers::open_bills(ctx, &typed::<OpenBillsInput>(&bills_request)).unwrap();
    assert_eq!(bills.open_paisa.rupees(), 12_880);
    pair("assist_open_bills_akram", &bills_request, &bills);
    let payables_request =
        json!({ "seller_id": w.rahim, "person_key": null, "include_settled": false });
    let payables =
        vouchers::open_payables(ctx, &typed::<OpenPayablesInput>(&payables_request)).unwrap();
    assert_eq!(payables.open_paisa.rupees(), 4_316);
    pair("assist_open_payables_rahim", &payables_request, &payables);

    // 3. Event 3: اکرم pays 10,000 (E2E-6).
    let receipt = receipt_json(w.akram, "10,000");
    let (request, answer) = preview("receipt", &receipt, None);
    assert_eq!(answer.new_balance.amount_paisa.rupees(), 2_880);
    pair("vouchers_preview_receipt_buyer", &request, &answer);
    let receipt_saved =
        vouchers::save_receipt(ctx, owner, &typed::<ReceiptInput>(&receipt)).unwrap();
    pair("vouchers_save_receipt", &receipt, &receipt_saved);

    // 4. Event 4: رحیم is paid his 4,316 (E2E-7).
    let payment = payment_json(w.rahim, "4,316", "");
    let (request, answer) = preview("payment", &payment, None);
    pair("vouchers_preview_payment_seller", &request, &answer);
    let payment_saved =
        vouchers::save_payment(ctx, owner, &typed::<PaymentInput>(&payment)).unwrap();
    pair("vouchers_save_payment", &payment, &payment_saved);

    // 5. Cash lent to اکرم (E2E-32): a payment to a buyer, with its reason.
    let lent = payment_json(w.akram, "3,000", "نقد ادھار");
    let (request, answer) = preview("payment", &lent, None);
    pair("vouchers_preview_payment_buyer", &request, &answer);
    let lent_saved = vouchers::save_payment(ctx, owner, &typed::<PaymentInput>(&lent)).unwrap();
    pair("vouchers_save_payment_cash_lent", &lent, &lent_saved);

    // 6. Refusals: cash lent without a reason, and a seller returning what he does not owe.
    let no_reason = payment_json(w.akram, "1,000", "");
    let error = vouchers::save_payment(ctx, owner, &typed::<PaymentInput>(&no_reason)).unwrap_err();
    refusal(
        "vouchers_save_payment_to_buyer_needs_reason",
        &no_reason,
        &error,
        ErrorCode::ErrPaymentToBuyerNeedsReason,
    );
    let too_much = receipt_json(w.rahim, "1,000");
    let error = vouchers::save_receipt(ctx, owner, &typed::<ReceiptInput>(&too_much)).unwrap_err();
    refusal(
        "vouchers_save_receipt_seller_return_too_much",
        &too_much,
        &error,
        ErrorCode::ErrReturnExceedsOpen,
    );

    // 7. کریم: a payment when nothing is payable (the note of FR-7.68), a payment that takes cash
    // below zero (FR-6.7), then an advance to him and a part of it returned (FR-15.12).
    let nothing = payment_json(w.karim, "1,000", "");
    let (request, answer) = preview("payment", &nothing, None);
    assert!(answer.payable_is_zero_for_seller);
    pair(
        "vouchers_preview_payment_seller_nothing_payable",
        &request,
        &answer,
    );
    let big = payment_json(w.karim, "9,900,000", "");
    let (request, answer) = preview("payment", &big, None);
    assert!(answer.cash.needs_confirm);
    pair(
        "vouchers_preview_payment_cash_below_zero",
        &request,
        &answer,
    );
    let error = vouchers::save_payment(ctx, owner, &typed::<PaymentInput>(&big)).unwrap_err();
    refusal(
        "vouchers_save_payment_cash_below_zero",
        &big,
        &error,
        ErrorCode::ErrCashBelowZeroNotConfirmed,
    );
    let karim_advance = advance_json(w.karim, "5,000");
    vouchers::save_advance(ctx, owner, &typed::<AdvanceInput>(&karim_advance)).unwrap();
    let advances_request = json!({ "seller_id": w.karim });
    let advances = mandi_services::assist::advances(
        ctx,
        typed::<AssistAdvancesInput>(&advances_request).seller_id,
    )
    .unwrap();
    pair("assist_advances_karim", &advances_request, &advances);
    let (request, answer) = preview("receipt", &receipt_json(w.karim, "2,000"), None);
    assert!(answer.errors.is_empty());
    pair("vouchers_preview_receipt_seller_return", &request, &answer);

    // 8. A receipt from ارشد, and an old client that still sends a mode, a bank account and a
    // cheque (PRD 15.1, v1.6): refused at each name it sent, and nothing is posted.
    let arshad_receipt = receipt_json(w.arshad, "5,000");
    let arshad_saved =
        vouchers::save_receipt(ctx, owner, &typed::<ReceiptInput>(&arshad_receipt)).unwrap();
    pair(
        "vouchers_save_receipt_arshad",
        &arshad_receipt,
        &arshad_saved,
    );
    let entries_before = journal_entry_count(&w);
    let mut old_fields = receipt_json(w.arshad, "5,000");
    old_fields["mode"] = json!("cheque");
    old_fields["bank_account_id"] = json!(5);
    old_fields["cheque_no"] = json!("000123");
    old_fields["cheque_date"] = json!("12-10-2026");
    let error =
        vouchers::save_receipt(ctx, owner, &typed::<ReceiptInput>(&old_fields)).unwrap_err();
    let ServiceError::Rejected(items) = &error else {
        panic!("not a refusal: {error:?}");
    };
    assert_eq!(
        items
            .iter()
            .map(|i| i.field.as_deref().unwrap_or(""))
            .collect::<Vec<_>>(),
        ["mode", "bank_account_id", "cheque_no", "cheque_date"]
    );
    refusal(
        "vouchers_save_receipt_old_payment_fields",
        &old_fields,
        &error,
        ErrorCode::ErrFieldNotAllowed,
    );
    assert_eq!(journal_entry_count(&w), entries_before, "nothing is posted");

    // 9. The slips, in both languages (PRD FR-15.2).
    for (name, voucher_id, language) in [
        (
            "reports_voucher_print_data_receipt_ur",
            receipt_saved.voucher_id,
            "ur",
        ),
        (
            "reports_voucher_print_data_payment_en",
            payment_saved.voucher_id,
            "en",
        ),
        (
            "reports_voucher_print_data_advance_ur",
            advance_saved.voucher_id,
            "ur",
        ),
        (
            "reports_voucher_print_data_receipt_en",
            arshad_saved.voucher_id,
            "en",
        ),
    ] {
        let request = json!({ "voucher_id": voucher_id, "language": language });
        let slip = mandi_reports::print::voucher_print_data(
            ctx.db(),
            &typed::<mandi_reports::print::VoucherPrintDataInput>(&request),
        )
        .unwrap();
        pair(name, &request, &slip);
    }

    // 10. One voucher, its edit (the preview takes the old one back first), and a cancel.
    let get_request = json!({ "id": receipt_saved.voucher_id });
    let row = vouchers::get(ctx, typed::<VoucherGetInput>(&get_request).id).unwrap();
    pair("vouchers_get_receipt", &get_request, &row);
    let smaller = receipt_json(w.akram, "9,000");
    let (request, answer) = preview("receipt", &smaller, Some(receipt_saved.voucher_id));
    pair("vouchers_preview_receipt_edit", &request, &answer);
    let edit_request = json!({
        "id": receipt_saved.voucher_id, "version": row.version,
        "entry": entry("receipt", &smaller), "reason": "رقم درست کی",
        "confirm_cash_below_zero": false
    });
    let edited = vouchers::edit(ctx, owner, &typed::<VoucherEditInput>(&edit_request)).unwrap();
    pair("vouchers_edit_receipt", &edit_request, &edited);
    let lent_row = vouchers::get(ctx, lent_saved.voucher_id).unwrap();
    let cancel_request = json!({
        "id": lent_saved.voucher_id, "version": lent_row.version, "reason": "غلط اندراج",
        "confirm_cash_below_zero": false
    });
    let cancelled =
        vouchers::cancel(ctx, owner, &typed::<VoucherCancelInput>(&cancel_request)).unwrap();
    pair("vouchers_cancel_cash_lent", &cancel_request, &cancelled);

    // 11. The lists: payments (one cancelled, so the total leaves it out), and all types (no total).
    let list_request = json!({
        "voucher_type": "payment", "from": "", "to": "", "account_id": null, "person_key": null,
        "offset": 0, "limit": 0
    });
    let list = vouchers::list(ctx, &typed::<VouchersListInput>(&list_request)).unwrap();
    assert_eq!(list.rows.len(), 2);
    assert_eq!(list.total_count, 2);
    assert_eq!(list.totals.cancelled_count, 1);
    assert_eq!(list.totals.total_paisa.map(|t| t.rupees()), Some(4_316));
    pair("vouchers_list_payments", &list_request, &list);
    let all_request = json!({
        "voucher_type": null, "from": "", "to": "", "account_id": null, "person_key": null,
        "offset": 0, "limit": 0
    });
    let all = vouchers::list(ctx, &typed::<VouchersListInput>(&all_request)).unwrap();
    assert_eq!(all.totals.total_paisa, None);
    pair("vouchers_list_all", &all_request, &all);
    // The second page of two rows: the count and the totals are still over the whole filter.
    let page_request = json!({
        "voucher_type": null, "from": "", "to": "", "account_id": null, "person_key": null,
        "offset": 2, "limit": 2
    });
    let page = vouchers::list(ctx, &typed::<VouchersListInput>(&page_request)).unwrap();
    assert_eq!(page.rows.len(), 2);
    assert_eq!(page.total_count, all.total_count);
    assert_eq!(page.totals, all.totals);
    assert_eq!(page.rows, all.rows[2..4].to_vec());
    pair("vouchers_list_all_page_2", &page_request, &page);

    // 12. The advances report (R26 data).
    let report_request = json!({ "seller_id": null, "as_of": "" });
    let report =
        vouchers::advances_report(ctx, &typed::<AdvancesReportInput>(&report_request)).unwrap();
    pair("advances_report_all", &report_request, &report);

    // 13. The collection round: the buyers whose collection day is Saturday, then the round's
    // receipts: one saved, one blank (skipped) and one refused.
    for id in [w.akram, w.arshad] {
        let mut input = account_form(&w, id);
        input.collection_weekday = Some(5);
        confirming(|words| {
            let mut attempt = input.clone();
            attempt.confirmed_rules_words = words.to_vec();
            mandi_services::accounts::save(ctx, owner, &attempt)
        });
    }
    let round_request =
        json!({ "date": "10-10-2026", "collector_staff_id": null, "city_id": null });
    let round =
        vouchers::collection_list(ctx, &typed::<CollectionListInput>(&round_request)).unwrap();
    assert!(!round.groups.is_empty());
    pair("assist_collection_list_saturday", &round_request, &round);
    let batch_request = json!({ "rows": [
        receipt_json(w.arshad, "1,000"),
        receipt_json(w.akram, ""),
        receipt_json(w.rahim, "1,000")
    ] });
    let batch =
        vouchers::save_receipts_batch(ctx, owner, &typed::<ReceiptsInput>(&batch_request)).unwrap();
    assert_eq!(
        batch
            .rows
            .iter()
            .map(|r| (r.saved.is_some(), r.skipped, r.errors.len()))
            .collect::<Vec<_>>(),
        [(true, false, 0), (false, true, 0), (false, false, 1)]
    );
    pair("vouchers_save_receipts_batch_round", &batch_request, &batch);

    // 14. Make a one-time buyer regular (PRD FR-13.8): Shahid owes 2,000.
    one_time_buyer_owes(&w);
    let key = mandi_services::core::text::person_key("شاہد", "0300-0000000");
    let (made, words) = confirming(|words| {
        vouchers::make_regular(
            ctx,
            owner,
            &typed::<MakeRegularInput>(&make_regular_request(&key, words)),
        )
    });
    assert_eq!(made.moved.amount_paisa.rupees(), 2_000);
    pair(
        "pukhta_make_regular_shahid",
        &make_regular_request(&key, &words),
        &made,
    );

    // 15. The global search finds vouchers by their number.
    let search_request = json!({ "text": "1", "limit": 20 });
    let hits = search::global(ctx, owner, &typed::<SearchGlobalInput>(&search_request)).unwrap();
    assert!(
        hits.iter().any(|hit| matches!(
            hit,
            mandi_services::core::accounts::GlobalHit::Voucher { .. }
        )),
        "a voucher hit"
    );
    pair("search_global_voucher_number", &search_request, &hits);

    // 16. Past 200 open rows the reads give the oldest 200 and say how many there are: ارشد with
    // 205 loans of 100, کریم with 205 جنرل credits of 1,000.
    for _ in 0..205 {
        let loan = payment_json(w.arshad, "100", "نقد ادھار");
        let mut input = typed::<PaymentInput>(&loan);
        input.confirm_cash_below_zero = true;
        vouchers::save_payment(ctx, owner, &input).unwrap();
    }
    let request = json!({ "account_id": w.arshad, "person_key": null });
    let truncated = vouchers::open_bills(ctx, &typed::<OpenBillsInput>(&request)).unwrap();
    assert_eq!((truncated.bills.len(), truncated.truncated), (200, true));
    pair("assist_open_bills_truncated", &request, &truncated);
    post_general_credits(&w, 205);
    let request = json!({ "seller_id": w.karim, "person_key": null, "include_settled": false });
    let truncated = vouchers::open_payables(ctx, &typed::<OpenPayablesInput>(&request)).unwrap();
    assert_eq!((truncated.postings.len(), truncated.truncated), (200, true));
    pair("assist_open_payables_truncated", &request, &truncated);
}

/// How many journal entries there are, to prove that a refusal posted nothing.
fn journal_entry_count(w: &World) -> i64 {
    w.env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM journal_entries", [], |r| r.get(0))
                .map_err(mandi_services::ServiceError::from)
        })
        .unwrap()
}

/// `count` جنرل credits of 1,000 on کریم, each against cash (so he is owed 1,000 more each time).
fn post_general_credits(w: &World, count: i64) {
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;
    let cash = support::id_of(&w.env, "01010001");
    for i in 0..count {
        let entry = build_entry(vec![
            LedgerLine::new(AccountId::new(cash), Side::Dr, support::rupees(1_000)),
            LedgerLine::new(AccountId::new(w.karim), Side::Cr, support::rupees(1_000)),
        ])
        .unwrap();
        w.env
            .ctx
            .db()
            .write(|tx| {
                mandi_services::ledger::post(
                    &w.env.ctx,
                    tx,
                    w.owner.user_id,
                    &PostingEvent {
                        date: mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10)
                            .unwrap(),
                        posting_type: PostingType::General,
                        source_type: SourceType::Voucher,
                        source_id: 9_500 + i,
                        narration: None,
                        reverses: None,
                        entry,
                    },
                )
            })
            .unwrap();
    }
}
