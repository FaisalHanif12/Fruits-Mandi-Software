//! Real answers of the plan 78 commands, as JSON files for the frontend (plan 78, Group 2 and the
//! money items): the three load buttons of the بکری screen, the card "where the capital is" and its
//! refusal while the partner section is locked, a garden refund, a change of a garden's rent with the
//! Owner's sign (and its refusals), and a partner's month with the advance lines.
//!
//! Each fixture is a pair, as in `garden_fixtures.rs`: `<name>_request.json` is exactly the `input` the
//! command takes, `<name>.json` the answer of the real service. The test compares the files with the
//! real answers and FAILS if they differ. After a deliberate change write them again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test plan78_fixtures`, read the diff and tell
//! `frontend`.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::core::auth::Language;
use mandi_services::core::edit::GardenHeaderEditInput;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{GardenEntryInput, GardenGetInput};
use mandi_services::core::partner_code::PartnerCodeSetInput;
use mandi_services::core::statement::PartnersStatementMonthInput;
use mandi_services::{
    ServiceError, bikri, cashbook, edit, gardens, partner_code, partners, vouchers,
};
use reports_world::*;
use serde::Serialize;
use serde_json::{Value, json};
use support::World;

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

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

fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

#[test]
fn the_three_load_buttons_of_the_bikri_screen() {
    // E2E-34's trip has 5 پیٹی مالٹا left, رحیم's lot has 30 of 50 left, a garden has 100 arrived.
    let (w, _trip, _purchase, _sale) = trip_34_world();
    let lot = support::with_sales(
        support::bare_lot(&w, "l-1", w.rahim, w.banana, "50"),
        vec![support::sale("s-1", w.akram, "20", "600")],
    );
    save(&w, &support::draft(vec![lot]));
    garden_with_arrival(&w, "100", "31-10-2026");
    let ctx = &w.env.ctx;
    pair(
        "bikri_goods_sources",
        &json!({}),
        &bikri::goods_sources(ctx).unwrap(),
    );
    for kind in ["trip", "garden"] {
        let request = json!({ "kind": kind });
        pair(
            &format!("bikri_goods_to_load_{kind}"),
            &request,
            &bikri::goods_to_load(ctx, &typed(&request)).unwrap(),
        );
    }
}

#[test]
fn the_card_where_the_capital_is_and_its_refusal_while_locked() {
    // After Events 1 to 5: 58,484 + 21,564 - 10,160 - 4,400 = 65,488.
    let w = full_day();
    let ctx = &w.env.ctx;
    pair(
        "cashbook_capital_where",
        &json!({}),
        &cashbook::capital_where(ctx).unwrap(),
    );
    partner_code::set(
        ctx,
        &w.owner,
        &PartnerCodeSetInput {
            password: "secret1".to_owned(),
            code: "4826".to_owned(),
            code_again: "4826".to_owned(),
        },
    )
    .unwrap();
    partner_code::lock(ctx).unwrap();
    let refused = partner_code::require_unlocked(ctx).unwrap_err();
    refusal(
        "cashbook_capital_where_locked",
        &json!({}),
        &refused,
        ErrorCode::ErrPartnerLocked,
    );
}

fn garden_entry_json(kind: &str, garden: i64, amount: &str) -> Value {
    json!({
        "type": kind, "garden_id": garden, "date": "10-10-2026", "amount": amount, "note": null
    })
}

fn header_json(
    w: &World,
    garden: &mandi_services::core::gardens::GardenRow,
    rent: &str,
    reason: &str,
    sign: Value,
) -> Value {
    let [a, b, _] = w.partners;
    json!({
        "id": garden.id,
        "version": garden.version,
        "garden": {
            "name_ur": "شاہ پور باغ",
            "name_en": "Shahpur Garden",
            "confirmed_rules_words": ["شاہ", "پور", "باغ", "Shahpur", "Garden"],
            "date": "10-10-2026",
            "item_id": w.banana,
            "item_ids": [],
            "rent": rent,
            "season_end": null,
            "note": null,
            "partners": [
                { "partner_id": a, "share": "60", "source": "shop" },
                { "partner_id": b, "share": "40", "source": "pocket" }
            ],
            "paid_now": null,
            "paid_now_reason": null,
            "parts": [],
            "confirm_cash_below_zero": false
        },
        "reason": reason,
        "sign": sign
    })
}

#[test]
fn a_rent_relief_with_the_sign_and_the_refund_of_the_owner() {
    // Rent 50,000, all of it paid, a cost of 5,000, 30,000 from another shop (PRD FR-12.51 at 1/100).
    let w = support::world();
    let ctx = &w.env.ctx;
    let garden = garden_made_with(&w, "50,000", "50,000");
    garden_pay(&w, garden.id, true, "10-10-2026", "5,000", "labour");
    garden_other_sale(&w, garden.id, "10-10-2026", "1", "30,000");
    let current = |id: i64| {
        gardens::get(
            ctx,
            &GardenGetInput {
                id,
                history_offset: 0,
                history_limit: 0,
            },
        )
        .unwrap()
        .row
    };

    // The three refusals of a change of the rent, then the saved relief.
    let no_reason = header_json(&w, &garden, "45,000", "", json!("adnan"));
    let error =
        edit::garden_header_save(ctx, &w.owner, &typed::<GardenHeaderEditInput>(&no_reason))
            .unwrap_err();
    refusal(
        "gardens_header_edit_save_reason_required",
        &no_reason,
        &error,
        ErrorCode::ErrReasonRequired,
    );
    let wrong = header_json(
        &w,
        &garden,
        "45,000",
        "relief from the owner",
        json!("munshi"),
    );
    let error = edit::garden_header_save(ctx, &w.owner, &typed::<GardenHeaderEditInput>(&wrong))
        .unwrap_err();
    refusal(
        "gardens_header_edit_save_sign_wrong",
        &wrong,
        &error,
        ErrorCode::ErrSignWrong,
    );
    let ok = header_json(
        &w,
        &garden,
        "45,000",
        "relief from the owner",
        json!("adnan"),
    );
    let saved =
        edit::garden_header_save(ctx, &w.owner, &typed::<GardenHeaderEditInput>(&ok)).unwrap();
    pair("gardens_header_edit_save_relief", &ok, &saved);
    pair(
        "gardens_get_after_relief",
        &json!({ "id": garden.id, "history_offset": 0, "history_limit": 0 }),
        &gardens::get(
            ctx,
            &GardenGetInput {
                id: garden.id,
                history_offset: 0,
                history_limit: 0,
            },
        )
        .unwrap(),
    );

    // The owner owes 5,000: 6,000 is refused, 5,000 is saved.
    let too_much = garden_entry_json("refund", garden.id, "6,000");
    let error =
        gardens::entry_save(ctx, &w.owner, &typed::<GardenEntryInput>(&too_much)).unwrap_err();
    refusal(
        "gardens_entry_save_refund_too_much",
        &too_much,
        &error,
        ErrorCode::ErrReturnExceedsOpen,
    );
    let refund = garden_entry_json("refund", garden.id, "5,000");
    let saved = gardens::entry_save(ctx, &w.owner, &typed::<GardenEntryInput>(&refund)).unwrap();
    pair("gardens_entry_save_refund", &refund, &saved);
    assert_eq!(current(garden.id).owner_owes_paisa.rupees(), 0);
}

#[test]
fn a_partner_month_with_the_advance_lines() {
    // E2E-31 at رحیم's advance: 15,000 on 01-09-26 out, 10,000 back on 10-10-26 (4,000 / 3,000 / 3,000).
    let w = support::world();
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::AdvanceInput {
            confirm_cash_below_zero: false,
            date: "01-09-2026".into(),
            seller_account_id: w.rahim,
            amount: "15,000".into(),
            removed: Default::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "10,000")).unwrap();
    let b = w.partners[1];
    for (name, month) in [("2026-09", "2026-09"), ("2026-10", "2026-10")] {
        let request = json!({ "partner_id": b, "month": month });
        pair(
            &format!("partners_statement_month_advance_{name}"),
            &request,
            &partners::statement_month(&w.env.ctx, &typed::<PartnersStatementMonthInput>(&request))
                .unwrap(),
        );
    }
}

/// The request as JSON, from the real input type, so the file is exactly what the command takes.
fn input_json<T: Serialize>(input: &T) -> Value {
    serde_json::to_value(input).unwrap()
}

#[test]
fn the_prints_of_the_worked_day_for_the_sale_print_the_slips_the_bills_and_the_adhari_sheet() {
    // Events 1 to 5 (PRD 4.4 to 4.9): real names and numbers for the screens.
    use mandi_reports::reports::{
        AdhariSheetInput, AdhariSheetPrintInput, DaySlipsInput, SlipSections, adhari_sheet,
        adhari_sheet_print_data, day_slips, sale_print, seller_bills,
    };
    let w = full_day();
    let db = w.env.ctx.db();
    let slips = |sections, language| DaySlipsInput {
        date: "10-10-2026".into(),
        bikri_id: None,
        bikri_ids: None,
        buyer_ids: None,
        sections,
        language,
        paper: None,
    };
    for (name, language) in [("en", Language::En), ("ur", Language::Ur)] {
        let request = input_json(&slips(SlipSections::Both, language));
        pair(
            &format!("reports_sale_print_data_{name}"),
            &request,
            &sale_print(db, &typed(&request)).unwrap(),
        );
    }
    for (name, language) in [("en", Language::En), ("ur", Language::Ur)] {
        let request = input_json(&slips(SlipSections::Buyers, language));
        pair(
            &format!("reports_buyer_parchi_print_data_{name}"),
            &request,
            &day_slips(db, &typed(&request), "print-parchi-title").unwrap(),
        );
        let request = input_json(&slips(SlipSections::Sellers, language));
        pair(
            &format!("reports_seller_bills_print_data_{name}"),
            &request,
            &seller_bills(db, &typed(&request)).unwrap(),
        );
    }
    // The request is the command's own input (only `date` is given; the other fields have defaults).
    let request = json!({ "date": "10-10-2026" });
    pair(
        "reports_adhari_sheet",
        &request,
        &adhari_sheet(db, &typed::<AdhariSheetInput>(&request)).unwrap(),
    );
    for (name, language) in [("en", Language::En), ("ur", Language::Ur)] {
        let request = input_json(&AdhariSheetPrintInput {
            date: "10-10-2026".into(),
            language,
            paper: None,
        });
        pair(
            &format!("reports_adhari_sheet_print_data_{name}"),
            &request,
            &adhari_sheet_print_data(db, &typed(&request)).unwrap(),
        );
    }
}

#[test]
fn a_party_statement_with_the_opening_row_first_and_a_later_window_with_the_brought_forward() {
    // احسان (D8 world): an opening of 20,000 جمع dated the day of his بکری, then 100,000 paid to him
    // the next day (PRD FR-19.16, FR-12.47): the opening is the first row; a window that starts on
    // 11-10-2026 opens with what was brought forward.
    use mandi_reports::reports::{StatementRowsInput, statement_rows};
    let (w, _) = d8_paid_world();
    let ehsan: i64 = w
        .env
        .ctx
        .db()
        .read(|c| {
            c.query_row("SELECT id FROM accounts WHERE name_en = 'Ehsan'", [], |r| {
                r.get(0)
            })
        })
        .unwrap();
    mandi_services::accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: ehsan,
                version: mandi_services::accounts::get(&w.env.ctx, ehsan)
                    .unwrap()
                    .version,
                opening_date: "10-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: "20,000".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    common::set_today(&w.env, 2026, 10, 11);
    let mut pay = payment(ehsan, "100,000");
    pay.date = "11-10-2026".into();
    pay.confirm_cash_below_zero = true;
    vouchers::save_payment(&w.env.ctx, &w.owner, &pay).unwrap();
    for (name, from) in [("whole", None), ("from_11_10", Some("11-10-2026"))] {
        let request = input_json(&StatementRowsInput {
            account_id: ehsan,
            from: from.map(str::to_owned),
            to: None,
            include_history: false,
            newest_first: false,
            offset: 0,
            limit: 0,
        });
        pair(
            &format!("party_ledgers_statement_rows_{name}"),
            &request,
            &statement_rows(w.env.ctx.db(), &typed(&request)).unwrap(),
        );
    }
}
