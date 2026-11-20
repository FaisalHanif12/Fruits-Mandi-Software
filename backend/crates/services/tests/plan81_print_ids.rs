//! Plan 81 item 4: no raw message ID on any print (NFR-URD-1, NFR-URD-22), and the real print data of
//! a partner's month for the frontend.
//!
//! The first test builds every table print of the worked day (Events 1 to 5) and checks that each
//! Fluent ID the table names exists in BOTH languages: the title, the heading of each column that is
//! not a name, the totals labels, the labels of the facts and every label, document kind and message
//! of a cell. A message that is missing makes the screen and the print show its own ID, which is
//! what happened to `print-partner-month-col-advances` in the Urdu print.
//!
//! The second test writes (or checks) the answer of `partners_statement_month_print_data`, as a pair
//! for each language, in `tests/fixtures/`. After a deliberate change:
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test plan81_print_ids`, read the diff and
//! tell `frontend`.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mandi_reports::dashboard::{
    BuyerDuesPrintInput, CashSalesDayPrintInput, CashSalesMonthPrintInput, CommissionPrintInput,
    LentOpenPrintInput, SellerDuesPrintInput, SellerDuesSide, TotalDuesPrintInput, advances_spec,
    buyer_dues_spec, cash_sales_day_spec, cash_sales_month_spec, commission_month_spec,
    lent_open_spec, seller_dues_spec, total_dues_spec,
};
use mandi_reports::export::l10n::L10n;
use mandi_reports::reports::{
    AdhariSheetPrintInput, Cell, CollectionSheetPrintInput, StatementRowsPrintInput, TableSpec,
    adhari_sheet_spec, collection_sheet_spec, partner_statement_month_print_data,
    partner_statement_month_spec, statement_rows_spec,
};
use mandi_services::core::auth::Language;
use mandi_services::core::lent::{LentPersonCreateInput, MoneyLentInput};
use mandi_services::core::statement::{
    PartnersStatementMonthInput, PartnersStatementMonthPrintInput,
};
use mandi_services::core::vouchers::AdvancesReportInput;
use mandi_services::{lent, partners, vouchers};
use reports_world::{full_day, payment};
use serde::Serialize;
use serde_json::Value;

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

/// Every Fluent ID a table names, with where it is used.
fn ids_of(spec: &TableSpec) -> BTreeSet<(String, String)> {
    let mut ids = BTreeSet::new();
    let mut add = |place: &str, id: &str| {
        ids.insert((place.to_owned(), id.to_owned()));
    };
    add("title", &spec.title_id);
    add("totals label", &spec.totals_label_id);
    for (index, column) in spec.columns.iter().enumerate() {
        // A heading that is a name (a partner's column) is not a message.
        if spec.column_names.get(index).is_some_and(Option::is_some) {
            continue;
        }
        add(&format!("column {}", column.key), &column.title_id);
    }
    for fact in &spec.facts {
        add("fact label", &fact.label_id);
    }
    for (label, _) in &spec.extra_totals {
        add("extra totals label", label);
    }
    let cells = spec
        .rows
        .iter()
        .flat_map(|row| row.cells.iter())
        .chain(spec.totals.iter())
        .chain(spec.extra_totals.iter().flat_map(|(_, cells)| cells.iter()));
    for cell in cells {
        match cell {
            Cell::Label { id } => add("cell label", id),
            Cell::Doc { type_id, .. } => add("cell document kind", type_id),
            Cell::Message { id, .. } => add("cell message", id),
            _ => {}
        }
    }
    ids
}

/// Fails with every ID of the table that is missing in a language.
fn every_id_exists(name: &str, spec: &TableSpec, missing: &mut Vec<String>) {
    for language in [Language::Ur, Language::En] {
        let l10n = L10n::new(language).unwrap();
        for (place, id) in ids_of(spec) {
            if id.is_empty() || !l10n.has(&id) {
                missing.push(format!("{name}: {place} `{id}` is missing in {language:?}"));
            }
        }
    }
}

#[test]
fn every_message_id_of_every_table_print_exists_in_both_languages() {
    let w = full_day();
    // Two more things on the books, so the tables that list them have rows: رحیم, who has nothing payable, is paid
    // 1,840 (PRD FR-15.7: he then owes the shop; کریم is still owed 10,160) and the shop lends اسلم 5,000.
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "1,840")).unwrap();
    let aslam = lent::person_create(
        &w.env.ctx,
        &w.owner,
        &LentPersonCreateInput {
            name_ur: "اسلم".into(),
            name_en: "Aslam".into(),
            phone: None,
            staff_id: None,
            confirmed_rules_words: vec!["اسلم".into(), "Aslam".into()],
        },
    )
    .unwrap()
    .account
    .id;
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(aslam),
            amount: "5,000".into(),
            reason: "x".into(),
            return_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let db = w.env.ctx.db();
    let [alif, _, _] = w.partners;
    let mut missing: Vec<String> = Vec::new();
    let mut without_rows: Vec<String> = Vec::new();
    let mut check = |name: &str, spec: TableSpec| {
        // A table with no row cannot show a missing ID of a cell, only of its frame.
        if spec.rows.is_empty() {
            without_rows.push(name.to_owned());
        }
        every_id_exists(name, &spec, &mut missing);
    };

    for language in [Language::Ur, Language::En] {
        // The partner's month (PRD FR-17.12, FR-17.13): the opening, a day with every column, the totals.
        let month = partners::statement_month(
            &w.env.ctx,
            &PartnersStatementMonthInput {
                partner_id: alif,
                month: Some("2026-10".into()),
            },
        )
        .unwrap();
        check(
            &format!("partner month {language:?}"),
            partner_statement_month_spec(&month, language),
        );
        check(
            &format!("commission month {language:?}"),
            commission_month_spec(
                db,
                &CommissionPrintInput {
                    month: None,
                    period: None,
                    party: None,
                    goods: None,
                    language,
                    paper: None,
                },
                true,
            )
            .unwrap(),
        );
        check(
            &format!("buyer dues {language:?}"),
            buyer_dues_spec(
                db,
                &BuyerDuesPrintInput {
                    search: None,
                    party_id: None,
                    sort: None,
                    language,
                    paper: None,
                },
            )
            .unwrap(),
        );
        for side in [
            SellerDuesSide::All,
            SellerDuesSide::ShopOwes,
            SellerDuesSide::SellerOwes,
        ] {
            check(
                &format!("seller dues {side:?} {language:?}"),
                seller_dues_spec(
                    db,
                    &SellerDuesPrintInput {
                        side,
                        party_id: None,
                        search: None,
                        sort: None,
                        language,
                        paper: None,
                    },
                )
                .unwrap(),
            );
        }
        check(
            &format!("total dues {language:?}"),
            total_dues_spec(
                db,
                &TotalDuesPrintInput {
                    language,
                    paper: None,
                },
            )
            .unwrap(),
        );
        check(
            &format!("money lent open {language:?}"),
            lent_open_spec(
                db,
                &LentOpenPrintInput {
                    search: None,
                    sort: None,
                    language,
                    paper: None,
                },
            )
            .unwrap(),
        );
        check(
            &format!("cash sales month {language:?}"),
            cash_sales_month_spec(
                db,
                &CashSalesMonthPrintInput {
                    month: None,
                    language,
                    paper: None,
                },
            )
            .unwrap(),
        );
        check(
            &format!("cash sales day {language:?}"),
            cash_sales_day_spec(
                db,
                &CashSalesDayPrintInput {
                    date: "10-10-2026".into(),
                    language,
                    paper: None,
                },
            )
            .unwrap(),
        );
        check(
            &format!("collection sheet {language:?}"),
            collection_sheet_spec(
                db,
                &CollectionSheetPrintInput {
                    date: "10-10-2026".into(),
                    search: None,
                    sort: None,
                    language,
                    paper: None,
                },
            )
            .unwrap(),
        );
        check(
            &format!("adhari sheet {language:?}"),
            adhari_sheet_spec(
                db,
                &AdhariSheetPrintInput {
                    date: "10-10-2026".into(),
                    language,
                    paper: None,
                },
            )
            .unwrap(),
        );
        // A buyer's statement, with the items under each row and with the edit history.
        for (items, include_history) in [(false, false), (true, true)] {
            check(
                &format!("statement rows items={items} {language:?}"),
                statement_rows_spec(
                    db,
                    &StatementRowsPrintInput {
                        account_id: w.akram,
                        from: None,
                        to: None,
                        items,
                        include_history,
                        language,
                        paper: None,
                    },
                )
                .unwrap(),
            );
        }
        let advances = vouchers::advances_report(
            &w.env.ctx,
            &AdvancesReportInput {
                seller_id: None,
                as_of: "10-10-2026".into(),
                offset: 0,
                limit: 0,
                search: None,
                only_open: false,
            },
        )
        .unwrap();
        check(
            &format!("advances {language:?}"),
            advances_spec(db, &advances, None).unwrap(),
        );
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
    // Every table has a row, so every cell kind of every table was checked.
    assert_eq!(
        without_rows,
        Vec::<String>::new(),
        "tables with no row: their cells were not checked"
    );
}

#[test]
fn the_check_sees_an_id_that_has_no_message() {
    // The check is only worth something if it fails on a gap: a heading, a totals label, a fact
    // label and a cell label with an ID that no file has are each named, in both languages.
    let w = full_day();
    let month = partners::statement_month(
        &w.env.ctx,
        &PartnersStatementMonthInput {
            partner_id: w.partners[0],
            month: Some("2026-10".into()),
        },
    )
    .unwrap();
    let mut spec = partner_statement_month_spec(&month, Language::En);
    spec.columns[7].title_id = "print-partner-month-col-no-such".into();
    spec.totals_label_id = "print-no-such-total".into();
    spec.facts[0].label_id = "print-no-such-fact".into();
    spec.rows[0].cells[0] = Cell::Label {
        id: "print-no-such-cell".into(),
    };
    let mut missing = Vec::new();
    every_id_exists("probe", &spec, &mut missing);
    assert_eq!(missing.len(), 8, "{missing:?}");
    for id in [
        "print-partner-month-col-no-such",
        "print-no-such-total",
        "print-no-such-fact",
        "print-no-such-cell",
    ] {
        assert_eq!(
            missing.iter().filter(|line| line.contains(id)).count(),
            2,
            "{id}: {missing:?}"
        );
    }
}

// ---- The real print data of a partner's month, for the frontend ----

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

/// One pair of files: the request exactly as the command takes it, and the real answer.
fn pair<R: Serialize, A: Serialize>(name: &str, request: &R, answer: &A) {
    let to_text = |value: Value| serde_json::to_string_pretty(&value).unwrap() + "\n";
    write_or_check(
        &format!("{name}_request.json"),
        &to_text(serde_json::to_value(request).unwrap()),
    );
    write_or_check(
        &format!("{name}.json"),
        &to_text(serde_json::to_value(answer).unwrap()),
    );
}

#[test]
fn the_print_data_of_a_partner_month_in_both_languages() {
    // Events 1 to 5 and رحیم's advance (PRD FR-17.13): الف in October 2026 has a day with a
    // commission, an expense, money taken and his part of the advance coming back, so every column
    // of the print has a figure. The totals and the closing are in words in the print language.
    let w = full_day();
    let alif = w.partners[0];
    for (name, language) in [("ur", Language::Ur), ("en", Language::En)] {
        let request = PartnersStatementMonthPrintInput {
            partner_id: alif,
            month: Some("2026-10".into()),
            language,
            paper: None,
        };
        let month = partners::statement_month(
            &w.env.ctx,
            &PartnersStatementMonthInput {
                partner_id: request.partner_id,
                month: request.month.clone(),
            },
        )
        .unwrap();
        let data =
            partner_statement_month_print_data(w.env.ctx.db(), &month, language, request.paper)
                .unwrap();
        // The print data carries IDs, the screen turns them into words: every heading of this print
        // must be a message in both languages.
        for language in [Language::Ur, Language::En] {
            let l10n = L10n::new(language).unwrap();
            for column in &data.columns {
                assert!(
                    l10n.has(&column.title_id),
                    "{name}: the column {} has no message `{}` in {language:?}",
                    column.key,
                    column.title_id
                );
            }
        }
        pair(
            &format!("partners_statement_month_print_data_{name}"),
            &request,
            &data,
        );
    }
}
