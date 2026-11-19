//! Plan 76: the four tables that are not engine reports, exported as sheets (PRD FR-19.3,
//! FR-19.11): the commission month, the buyer dues, the اگراہی sheet and a month of a partner's
//! statement. Each sheet is made from the same table as its print, so its rows and totals are the
//! page's. Every expected number is the PRD's (4.4, 4.5, 4.9, E2E-9, E2E-37) and is never changed
//! to suit the code.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
mod xlsx_support;

use mandi_reports::dashboard::{
    BuyerDuesPrintInput, CommissionPrintInput, buyer_dues_spec, commission_month_spec,
};
use mandi_reports::export::export_table;
use mandi_reports::export::l10n::L10n;
use mandi_reports::reports::{
    AdhariSheetPrintInput, CollectionSheetPrintInput, TableSpec, adhari_sheet_spec,
    collection_sheet_spec, partner_statement_month_spec,
};
use mandi_services::core::auth::Language;
use mandi_services::core::statement::PartnersStatementMonthInput;
use mandi_services::partners;
use reports_world::full_day;
use xlsx_support::{Sheet, Value};

fn export(w: &support::World, spec: &TableSpec, language: Language, name: &str) -> Sheet {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(format!("{name}.xlsx"));
    let done = export_table(
        w.env.ctx.db(),
        spec,
        language,
        &path.display().to_string(),
        &mut |_| {},
    )
    .unwrap();
    assert!(done.size_bytes > 0);
    assert_eq!(
        done.rows,
        u64::try_from(spec.rows.len() + spec.extra_totals.len()).unwrap()
    );
    xlsx_support::read(&path)
}

/// The row of the heading: the one that holds the title of the first column.
fn head_row(sheet: &Sheet, spec: &TableSpec, language: Language) -> u32 {
    let l10n = L10n::new(language).unwrap();
    sheet
        .find_text(0, &l10n.text(&spec.columns[0].title_id))
        .expect("the heading row")
}

fn nums(sheet: &Sheet, row: u32, from: u32, to: u32) -> Vec<String> {
    (from..to).map(|c| sheet.num(row, c)).collect()
}

#[test]
fn the_commission_month_is_a_sheet_with_the_day_and_the_partner_shares() {
    let w = full_day();
    let input = CommissionPrintInput {
        month: None,
        period: None,
        party: None,
        goods: None,
        language: Language::En,
        paper: None,
    };
    let spec = commission_month_spec(w.env.ctx.db(), &input, true).unwrap();
    let sheet = export(&w, &spec, Language::En, "commission");
    let head = head_row(&sheet, &spec, Language::En);
    // 10-10-26: seller 2,484, buyer 2,484, لاگا 480, total 5,448, shared 5,488; الف 2,196, ب 1,646, ج 1,646.
    let want = [
        "2484", "2484", "480", "5448", "5488", "2196", "1646", "1646",
    ];
    assert_eq!(nums(&sheet, head + 1, 1, 9), want);
    // The month totals row is the backend's: the same figures.
    assert_eq!(nums(&sheet, head + 2, 1, 9), want);
    assert!(matches!(sheet.get(head + 1, 0), Value::Num(_)));
    assert!(sheet.frozen && sheet.formulas == 0);
    // A partner's column is headed with his name in the sheet language.
    assert!(
        sheet
            .row_texts(head)
            .iter()
            .any(|t| t.contains("Partner A") || t.contains("Partner")),
        "{:?}",
        sheet.row_texts(head)
    );
    // Without the partner section open the partner columns are not in the sheet.
    let locked = commission_month_spec(w.env.ctx.db(), &input, false).unwrap();
    assert!(locked.columns.len() < spec.columns.len());
    let sheet = export(&w, &locked, Language::En, "commission-locked");
    let head = head_row(&sheet, &locked, Language::En);
    assert_eq!(nums(&sheet, head + 1, 1, 6), &want[..5]);
}

#[test]
fn the_buyer_dues_sheet_has_every_buyer_and_the_total() {
    let w = full_day();
    let spec = buyer_dues_spec(
        w.env.ctx.db(),
        &BuyerDuesPrintInput {
            search: None,
            party_id: None,
            sort: None,
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    let sheet = export(&w, &spec, Language::En, "dues");
    let head = head_row(&sheet, &spec, Language::En);
    // After Events 1 to 5: ارشد 18,684 and اکرم 2,880 (the Urdu order of the names, in both
    // languages), total 21,564 = the tile.
    assert_eq!(sheet.num(head + 1, 2), "18684");
    assert_eq!(sheet.num(head + 2, 2), "2880");
    assert_eq!(sheet.num(head + 3, 2), "21564");
}

#[test]
fn the_collection_sheet_is_a_sheet_with_the_totals_of_the_day() {
    let w = full_day();
    let spec = collection_sheet_spec(
        w.env.ctx.db(),
        &CollectionSheetPrintInput {
            date: "10-10-2026".into(),
            search: None,
            sort: None,
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    let sheet = export(&w, &spec, Language::En, "sheet");
    let head = head_row(&sheet, &spec, Language::En);
    // Plan 76 point 2: previous 0, bills 44,364, paid on the spot 12,800, total due 31,564 (the
    // 10,000 وصولی is not on this sheet).
    assert_eq!(
        nums(&sheet, head + 4, 3, 7),
        ["0", "44364", "12800", "31564"]
    );
    // Each of the three buyers is a row, in the Urdu order of the names: ارشد, اکرم, شاہد.
    assert_eq!(
        (1..=3).map(|i| sheet.num(head + i, 6)).collect::<Vec<_>>(),
        ["18684", "12880", "0"]
    );
}

/// Plan 80 Q-R5-2 (PRD v1.43 R5, FR-19.3): the buyers-only ادھاری تفصیل sheet of a day is a sheet too
/// (`tables_export_excel`, kind `adhari_sheet`), made from the table of its print. After Events 1 to 5
/// on 10-10-26: today's bills 44,364, paid on the spot 12,800, credit of today 31,564, previous 0,
/// total credit 31,564. Money and counts go through `rupees_as_f64` and `count_as_f64` (M13).
#[test]
fn the_adhari_sheet_is_a_sheet_with_the_totals_of_the_day() {
    let w = full_day();
    let spec = adhari_sheet_spec(
        w.env.ctx.db(),
        &AdhariSheetPrintInput {
            date: "10-10-2026".into(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    let sheet = export(&w, &spec, Language::En, "adhari");
    let head = head_row(&sheet, &spec, Language::En);
    // Three buyers bought today (اکرم, ارشد, شاہد), one row each, then the totals row.
    assert_eq!(
        nums(&sheet, head + 4, 2, 7),
        ["44364", "12800", "31564", "0", "31564"]
    );
    assert_eq!(
        (1..=3).map(|i| sheet.num(head + i, 2)).collect::<Vec<_>>(),
        ["18684", "12880", "12800"]
    );
}

#[test]
fn a_month_of_a_partner_statement_is_a_sheet_with_the_opening_and_the_closing() {
    let w = full_day();
    let a = w.partners[0];
    let month = partners::statement_month(
        &w.env.ctx,
        &PartnersStatementMonthInput {
            partner_id: a,
            month: Some("2026-10".into()),
        },
    )
    .unwrap();
    let spec = partner_statement_month_spec(&month, Language::En);
    let sheet = export(&w, &spec, Language::En, "statement");
    let head = head_row(&sheet, &spec, Language::En);
    // الف: commission 2,196, expense -1,200, money taken -2,000, his part of رحیم's advance coming
    // back +6,000 (PRD FR-17.13, v1.43), day net 4,996; کل رقم -6,000 at the start and -1,004 at the end
    // (E2E-9 plus the advance). Row 1 is the opening, row 2 the day, row 3 the totals. The column
    // "advances" is the 7th figure, so the running total is column 9.
    assert_eq!(sheet.num(head + 1, 9), "-6000");
    assert_eq!(
        nums(&sheet, head + 2, 1, 10),
        [
            "2196", "-1200", "0", "0", "-2000", "0", "6000", "4996", "-1004"
        ]
    );
    assert_eq!(sheet.num(head + 3, 9), "-1004");
    // Urdu reads right to left and has the same numbers.
    let urdu = export(&w, &spec, Language::Ur, "statement-ur");
    assert!(urdu.right_to_left);
    let head = head_row(&urdu, &spec, Language::Ur);
    assert_eq!(urdu.num(head + 2, 9), "-1004");
}
