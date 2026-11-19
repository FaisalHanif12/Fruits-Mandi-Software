//! Real answers of the plan 17 T2a commands, as JSON files for the frontend (plan 17, T4): the
//! روکڑ بک, the one خرچہ screen, ادھار رقم, the money a partner took and the overdue data.
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used; a command with no input has `null`), and
//! `<name>.json` is the answer the real service gives for it. A refusal is a pair too: its answer
//! has the shape of the error a command returns (`ok`, `errors`, `ref`), where `ref` is a
//! stand-in because the real reference number is a counter. The frontend copies the files to
//! `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ, so a fixture can
//! never go stale. After a deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test day_money_fixtures`, look at the
//! diff, and tell `frontend`.
//!
//! The story is the PRD worked day: بکری 1 of Event 1, then the electricity bill and الف's 2,000
//! of Event 5, his return, and اسلم's loan of E2E-30 and E2E-30b. Last month's electricity (on
//! 05-09-26) is there for "repeat last month".
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::*;
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::core::cashbook::CashbookInput;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::expenses::{
    ExpensePreviewInput, PartnerTakesInput, RepeatLastMonthInput,
};
use mandi_services::core::lent::{
    LentListInput, LentPersonCreateInput, LentReportInput, MoneyLentInput, MoneyLentReturnInput,
};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
use mandi_services::{ServiceError, bikri, cashbook, lent, masters, partners, vouchers};
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

fn head(w: &World, ur: &str, en: &str) -> i64 {
    confirming(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::ExpenseHead(ExpenseHeadInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: ur.into(),
                name_en: en.into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
    masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .iter()
        .find(|r| r.head.name_en == en)
        .unwrap()
        .id
}

/// The worked-day world with the shares of 40 / 30 / 30 started on 31-08-2026, cash 50,000, and
/// today 10-10-2026.
fn world() -> World {
    let w = support::world_with(50_000, false);
    set_today(&w.env, 2026, 8, 31);
    partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &PartnersSetSharesInput {
            effective_from: "31-08-2026".into(),
            shares: [
                (w.partners[0], "40"),
                (w.partners[1], "30"),
                (w.partners[2], "30"),
            ]
            .iter()
            .map(|&(partner_id, share)| ShareInput {
                partner_id,
                share: share.to_owned(),
            })
            .collect(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 10);
    w
}

#[test]
fn the_expense_and_lent_fixtures_are_the_real_answers() {
    let w = world();
    let (a, b) = (w.partners[0], w.partners[1]);
    let electricity = head(&w, "بجلی", "Electricity");
    let salaries = head(&w, "تنخواہ", "Salaries");

    // Last month's electricity, for "repeat last month".
    let last_month = json!({
        "date": "05-09-2026", "expense_head_id": electricity, "amount": "2,800",
        "paid_to": "WAPDA", "narration": "bill", "confirm_cash_below_zero": false
    });
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &typed(&last_month)).unwrap();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();

    // The expense heads, most used first.
    pair(
        "masters_expense_heads",
        &Value::Null,
        &masters::expense_heads(&w.env.ctx).unwrap(),
    );
    let repeat = json!({ "month": "2026-10" });
    pair(
        "vouchers_repeat_last_month_preview",
        &repeat,
        &vouchers::repeat_last_month_preview(&w.env.ctx, &typed::<RepeatLastMonthInput>(&repeat))
            .unwrap(),
    );

    // E2E-39: the electricity bill, previewed and saved; then A's 2,000.
    let shop = json!({
        "date": "10-10-2026", "expense_head_id": electricity, "amount": "3,000",
        "paid_to": "Partner A", "narration": null, "confirm_cash_below_zero": false
    });
    let preview_shop = json!({ "entry": { "type": "shop_expense", "date": "10-10-2026",
        "expense_head_id": electricity, "amount": "3,000", "paid_to": "Partner A" },
        "replacing_voucher_id": null });
    pair(
        "vouchers_preview_expense_shop",
        &preview_shop,
        &vouchers::preview_expense(&w.env.ctx, &typed::<ExpensePreviewInput>(&preview_shop))
            .unwrap(),
    );
    pair(
        "vouchers_save_shop_expense",
        &shop,
        &vouchers::save_shop_expense(&w.env.ctx, &w.owner, &typed(&shop)).unwrap(),
    );
    let missing_head = json!({
        "date": "10-10-2026", "expense_head_id": null, "amount": "100.50",
        "confirm_cash_below_zero": false
    });
    refusal(
        "vouchers_save_shop_expense_mistakes",
        &missing_head,
        &vouchers::save_shop_expense(&w.env.ctx, &w.owner, &typed(&missing_head)).unwrap_err(),
        ErrorCode::ErrDecimalsNotAllowed,
    );

    let preview_personal = json!({ "entry": { "type": "personal_expense", "date": "10-10-2026",
        "partner_id": a, "amount": "2,000", "reason": "گھر کا خرچ" },
        "replacing_voucher_id": null });
    pair(
        "vouchers_preview_expense_personal",
        &preview_personal,
        &vouchers::preview_expense(&w.env.ctx, &typed::<ExpensePreviewInput>(&preview_personal))
            .unwrap(),
    );
    let personal = json!({
        "date": "10-10-2026", "partner_id": a, "amount": "2,000", "reason": "گھر کا خرچ",
        "return_by": "20-10-2026", "taken_by": null, "narration": null,
        "confirm_cash_below_zero": false
    });
    pair(
        "vouchers_save_personal_expense",
        &personal,
        &vouchers::save_personal_expense(&w.env.ctx, &w.owner, &typed(&personal)).unwrap(),
    );
    let before_return = json!({ "partner_id": a, "offset": 0, "limit": 0 });
    pair(
        "partner_takes",
        &before_return,
        &partners::takes(&w.env.ctx, &typed::<PartnerTakesInput>(&before_return)).unwrap(),
    );
    let too_much =
        json!({ "date": "10-10-2026", "partner_id": a, "amount": "2,500", "narration": null });
    refusal(
        "vouchers_save_personal_return_exceeds",
        &too_much,
        &vouchers::save_personal_return(&w.env.ctx, &w.owner, &typed(&too_much)).unwrap_err(),
        ErrorCode::ErrReturnExceedsOpen,
    );
    let back = json!({ "date": "10-10-2026", "partner_id": a, "amount": "500", "narration": "first part" });
    pair(
        "vouchers_save_personal_return",
        &back,
        &vouchers::save_personal_return(&w.env.ctx, &w.owner, &typed(&back)).unwrap(),
    );
    // A cash question: B takes more than the cash that is left.
    let big = json!({
        "date": "10-10-2026", "partner_id": b, "amount": "60,000", "reason": "plot",
        "confirm_cash_below_zero": false
    });
    refusal(
        "vouchers_save_personal_expense_cash_confirm",
        &big,
        &vouchers::save_personal_expense(&w.env.ctx, &w.owner, &typed(&big)).unwrap_err(),
        ErrorCode::ErrCashBelowZeroNotConfirmed,
    );

    // ادھار رقم: اسلم, E2E-30.
    let (person, words) = confirming(|words| {
        lent::person_create(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateInput {
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                phone: Some("0300-1234567".into()),
                staff_id: None,
                confirmed_rules_words: words.to_vec(),
            },
        )
    });
    let create = json!({
        "name_ur": "اسلم", "name_en": "Aslam", "phone": "0300-1234567", "staff_id": null,
        "confirmed_rules_words": words
    });
    let _ = typed::<LentPersonCreateInput>(&create);
    pair("lent_person_create", &create, &person);
    let aslam = person.account.id;
    let early = json!({
        "date": "10-10-2026", "person_account_id": aslam, "amount": "5,000",
        "reason": "گھریلو ضرورت", "return_by": "05-10-2026", "narration": null,
        "confirm_cash_below_zero": false
    });
    refusal(
        "vouchers_save_money_lent_return_by_before_entry",
        &early,
        &vouchers::save_money_lent(&w.env.ctx, &w.owner, &typed(&early)).unwrap_err(),
        ErrorCode::ErrReturnByBeforeEntry,
    );
    let lend = json!({
        "date": "10-10-2026", "person_account_id": aslam, "amount": "5,000",
        "reason": "گھریلو ضرورت", "return_by": "15-10-2026", "narration": null,
        "confirm_cash_below_zero": false
    });
    pair(
        "vouchers_save_money_lent",
        &lend,
        &vouchers::save_money_lent(&w.env.ctx, &w.owner, &typed::<MoneyLentInput>(&lend)).unwrap(),
    );
    set_today(&w.env, 2026, 10, 11);
    let cash_back = json!({
        "date": "11-10-2026", "person_account_id": aslam, "amount": "3,000", "mode": "cash",
        "salary_head_id": null, "narration": null
    });
    pair(
        "vouchers_save_money_lent_return_cash",
        &cash_back,
        &vouchers::save_money_lent_return(
            &w.env.ctx,
            &w.owner,
            &typed::<MoneyLentReturnInput>(&cash_back),
        )
        .unwrap(),
    );
    let list = json!({ "person_id": aslam, "offset": 0, "limit": 0 });
    pair(
        "lent_list",
        &list,
        &lent::list(&w.env.ctx, &typed::<LentListInput>(&list)).unwrap(),
    );
    let report = json!({ "as_of": null, "offset": 0, "limit": 0 });
    pair(
        "lent_report",
        &report,
        &lent::report(&w.env.ctx, &typed::<LentReportInput>(&report)).unwrap(),
    );
    set_today(&w.env, 2026, 10, 16);
    pair(
        "assist_attention_overdue",
        &Value::Null,
        &lent::attention_overdue(&w.env.ctx).unwrap(),
    );
    set_today(&w.env, 2026, 10, 12);
    let salary_back = json!({
        "date": "12-10-2026", "person_account_id": aslam, "amount": "2,000", "mode": "salary",
        "salary_head_id": salaries, "narration": null
    });
    pair(
        "vouchers_save_money_lent_return_salary",
        &salary_back,
        &vouchers::save_money_lent_return(
            &w.env.ctx,
            &w.owner,
            &typed::<MoneyLentReturnInput>(&salary_back),
        )
        .unwrap(),
    );

    // The cash book: the whole story, and one day, in pages of 5.
    let day = json!({ "from": "10-10-2026", "to": "10-10-2026", "offset": 0, "limit": 5 });
    pair(
        "cashbook_get_day",
        &day,
        &cashbook::get(&w.env.ctx, &typed::<CashbookInput>(&day)).unwrap(),
    );
    let next = json!({ "from": "10-10-2026", "to": "12-10-2026", "offset": 5, "limit": 5 });
    pair(
        "cashbook_get_second_page",
        &next,
        &cashbook::get(&w.env.ctx, &typed::<CashbookInput>(&next)).unwrap(),
    );
}

/// Plan 17 T2c: the day close preview with its full summary (PRD FR-21.1, 4.9) for the frontend.
/// بکری 1, the electricity bill and الف's 2,000 are on 10-10-26: 1 بکری, خام 29,400, commission
/// 3,928, shared income 3,948, shop expense 3,000 split 1,200 / 900 / 900.
#[test]
fn the_day_close_preview_fixture_is_the_real_answer() {
    use mandi_services::core::vouchers::DayClosePreviewInput;
    let w = world();
    let electricity = head(&w, "بجلی", "Electricity");
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let shop = json!({
        "date": "10-10-2026", "expense_head_id": electricity, "amount": "3,000",
        "paid_to": null, "narration": null, "confirm_cash_below_zero": false
    });
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &typed(&shop)).unwrap();
    let request = json!({ "up_to": "" });
    let answer =
        mandi_services::day::close_preview(&w.env.ctx, &typed::<DayClosePreviewInput>(&request))
            .unwrap();
    assert_eq!(answer.summary.bikri_count, 1);
    assert_eq!(answer.summary.gross_paisa.rupees(), 29_400);
    assert_eq!(answer.summary.commission_paisa.rupees(), 3_928);
    assert_eq!(answer.summary.shared_income_paisa.rupees(), 3_948);
    pair("day_close_preview_summary", &request, &answer);
}

/// Plan 17 T2c, more states of the same screen: goods still left in باقی مال on the day (E2E-16:
/// 45 of 50 sold, 5 نگ left, خام 26,500, commission 3,540, shared income 3,560), and the refusal
/// when the day asked for is already closed.
#[test]
fn the_day_close_preview_stock_left_and_closed_day_fixtures_are_the_real_answers() {
    use mandi_services::core::vouchers::{DayCloseInput, DayClosePreviewInput};
    let w = world();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let request = json!({ "up_to": "10-10-2026" });
    let answer =
        mandi_services::day::close_preview(&w.env.ctx, &typed::<DayClosePreviewInput>(&request))
            .unwrap();
    let s = &answer.summary;
    assert_eq!(
        (
            s.bikri_count,
            s.gross_paisa.rupees(),
            s.commission_paisa.rupees(),
            s.shared_income_paisa.rupees(),
            s.open_lots,
            s.open_qty
        ),
        (1, 26_500, 3_540, 3_560, 1, 5)
    );
    pair("day_close_preview_summary_stock_left", &request, &answer);

    // After the close, asking for the same day again is refused (ERR_DATE_CLOSED on up_to).
    mandi_services::day::close(
        &w.env.ctx,
        &w.owner,
        &typed::<DayCloseInput>(&json!({ "up_to": "10-10-2026" })),
    )
    .unwrap();
    let error =
        mandi_services::day::close_preview(&w.env.ctx, &typed::<DayClosePreviewInput>(&request))
            .unwrap_err();
    refusal(
        "day_close_preview_summary_closed_day",
        &request,
        &error,
        ErrorCode::ErrDateClosed,
    );
}
