//! The Excel export on the PRD worked day (plan 22 round 10, PRD FR-18.4, FR-19.3, FR-20.5,
//! NFR-SEC-13, Code Standards M13).
//!
//! Every report of the engine and every analytics view is exported, in English and in Urdu, and
//! the file is read back as the text Excel stores. Every money, count, balance and date cell of
//! the page must be in the sheet with exactly the page's value; the totals row is the engine's.
//! The expected numbers are the PRD's (R18 is the E2E-10 table, R20 the E2E-33 dues, view A the
//! Section 18.4 table). The sheet is only read here, never by the app.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
mod xlsx_support;

use common::*;
use mandi_reports::ReportsError;
use mandi_reports::analytics::{AnalyticsFilters, AnalyticsPage, AnalyticsRunInput, AnalyticsView};
use mandi_reports::export::l10n::L10n;
use mandi_reports::export::{
    AnalyticsExportInput, ExportColumn, ExportProgress, ExportRow, ExportSheet, ExportStage,
    ReportExportInput, XCell, export_analytics, export_report, write_workbook,
};
use mandi_reports::pack;
use mandi_reports::reports::{Cell, FilterKind, ReportFilters, ReportId, ReportPage};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::ErrorCode;
use reports_world::*;
use std::path::{Path, PathBuf};
use xlsx_support::{Sheet, Value};

/// Every report in the catalogue, so a report added later is exported and checked at once.
fn all_reports() -> Vec<ReportId> {
    let ids: Vec<ReportId> = mandi_reports::reports::catalog()
        .iter()
        .map(|e| e.id)
        .collect();
    assert!(ids.len() >= 22, "{} reports", ids.len());
    ids
}

const ALL_VIEWS: [AnalyticsView; 6] = [
    AnalyticsView::A,
    AnalyticsView::B,
    AnalyticsView::C,
    AnalyticsView::D,
    AnalyticsView::E,
    AnalyticsView::H,
];

fn day_filters(w: &support::World, id: ReportId) -> ReportFilters {
    let entry = mandi_reports::reports::catalog()
        .into_iter()
        .find(|e| e.id == id)
        .unwrap();
    // A ledger or a statement needs its account; Akram has lines on the worked day.
    let mut filters = if entry.party_required {
        party(w.akram)
    } else {
        ReportFilters::default()
    };
    // Each report takes the filters its catalogue entry lists.
    let takes = entry.filters;
    if takes.contains(&FilterKind::DateRange) {
        filters.from = "2026-10-10".into();
        filters.to = "2026-10-10".into();
    }
    if takes.contains(&FilterKind::AsOf) {
        filters.as_of = "2026-10-10".into();
    }
    filters
}

fn page_of(w: &support::World, id: ReportId) -> ReportPage {
    report(w, id, day_filters(w, id), 0, 1_000)
}

fn view_of(w: &support::World, view: AnalyticsView) -> AnalyticsPage {
    mandi_reports::analytics::run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view,
            filters: AnalyticsFilters::default(),
            offset: 0,
            limit: 1_000,
        },
    )
    .unwrap()
}

fn export_to(w: &support::World, id: ReportId, language: Language, dir: &Path) -> (PathBuf, Sheet) {
    let path = dir.join(format!("{}-{:?}.xlsx", id.code(), language));
    let done = export_report(
        w.env.ctx.db(),
        &ReportExportInput {
            id,
            filters: day_filters(w, id),
            language,
            path: path.display().to_string(),
        },
        &mut |_| {},
    )
    .unwrap();
    assert!(done.size_bytes > 0 && done.path == path.display().to_string());
    let sheet = xlsx_support::read(&path);
    (path, sheet)
}

/// Days since the Excel epoch (1899-12-30) of a stored date, as Excel keeps a date.
fn excel_serial(iso: &str) -> i64 {
    fn civil(y: i64, m: i64, d: i64) -> i64 {
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let mp = (m + 9) % 12;
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }
    let day = BusinessDate::parse_iso(iso).unwrap();
    civil(
        i64::from(day.year()),
        i64::from(day.month()),
        i64::from(day.day()),
    ) - civil(1899, 12, 30)
}

fn signed(side: BalanceSide, rupees: i64) -> i64 {
    match side {
        BalanceSide::Dr => rupees,
        BalanceSide::Cr => -rupees,
        BalanceSide::Zero => 0,
    }
}

/// One cell of a page against the sheet.
fn expect_cell(sheet: &Sheet, row: u32, col: u32, cell: &Cell, l10n: &L10n, what: &str) {
    let got = sheet.get(row, col);
    let at = format!("{what} row {row} column {col}");
    match cell {
        Cell::Empty => assert_eq!(got, &Value::Blank, "{at}"),
        Cell::Money { paisa } => {
            assert_eq!(got, &Value::Num(paisa.rupees().to_string()), "{at}");
        }
        Cell::Number { value } => assert_eq!(got, &Value::Num(value.to_string()), "{at}"),
        Cell::Balance { paisa, side } => {
            let want = signed(*side, paisa.rupees());
            assert_eq!(got, &Value::Num(want.to_string()), "{at}");
        }
        Cell::Date { iso } => assert_eq!(got, &Value::Num(excel_serial(iso).to_string()), "{at}"),
        Cell::Label { id } => assert_eq!(got, &Value::Text(l10n.text(id)), "{at}"),
        Cell::Percent { hundredths } => {
            let text = format!("{}.{:02}%", hundredths / 100, hundredths % 100);
            assert_eq!(got, &Value::Text(text), "{at}");
        }
        _ => assert!(
            matches!(got, Value::Text(t) if !t.is_empty()),
            "{at}: {got:?}"
        ),
    }
}

/// The row of the heading: the one whose texts are exactly the column titles.
fn header_row(sheet: &Sheet, titles: &[String]) -> u32 {
    (0..=sheet.last_row())
        .find(|r| sheet.row_texts(*r) == titles)
        .unwrap_or_else(|| panic!("no heading row {titles:?}"))
}

fn check_page(
    sheet: &Sheet,
    columns: &[String],
    rows: &[Vec<Cell>],
    totals: &[Cell],
    l10n: &L10n,
    what: &str,
) {
    let head = header_row(sheet, columns);
    for (i, cells) in rows.iter().enumerate() {
        let row = head + 1 + u32::try_from(i).unwrap();
        for (col, cell) in cells.iter().enumerate() {
            expect_cell(sheet, row, u32::try_from(col).unwrap(), cell, l10n, what);
        }
    }
    if !totals.is_empty() {
        let row = head + 1 + u32::try_from(rows.len()).unwrap();
        for (col, cell) in totals.iter().enumerate() {
            // The label takes the first empty total cell.
            if matches!(cell, Cell::Empty) {
                continue;
            }
            expect_cell(sheet, row, u32::try_from(col).unwrap(), cell, l10n, what);
        }
    }
    assert!(
        sheet.frozen && sheet.formulas == 0,
        "{what}: frozen heading, no formulas"
    );
    assert!(sheet.tab.chars().count() <= 31, "{what}");
}

fn check_report(w: &support::World, id: ReportId, language: Language, dir: &Path) {
    let page = page_of(w, id);
    let (_, sheet) = export_to(w, id, language, dir);
    let l10n = L10n::new(language).unwrap();
    let columns: Vec<String> = page
        .columns
        .iter()
        .map(|c| l10n.text(&c.title_id))
        .collect();
    let rows: Vec<Vec<Cell>> = page.rows.iter().map(|r| r.cells.clone()).collect();
    let what = format!("{} {language:?}", id.code());
    check_page(&sheet, &columns, &rows, &page.totals, &l10n, &what);
    assert_eq!(sheet.right_to_left, language == Language::Ur, "{what}");
    assert!(
        sheet.all_texts().contains(&l10n.text(&page.title_id)),
        "{what} title"
    );
    // Every message the sheet uses exists in the language: nothing fell back to its ID.
    for text in sheet.all_texts() {
        assert!(
            !text.starts_with("report-") && !text.starts_with("analytics-"),
            "{what}: {text}"
        );
    }
}

// ---- The functions that make the numbers ----------------------------------------------------

#[test]
fn every_report_exports_with_every_number_equal_to_the_page_in_english() {
    let w = full_day();
    let dir = tempfile::tempdir().unwrap();
    for id in all_reports() {
        check_report(&w, id, Language::En, dir.path());
    }
}

#[test]
fn every_report_exports_in_urdu_and_reads_right_to_left() {
    let w = full_day();
    let dir = tempfile::tempdir().unwrap();
    for id in all_reports() {
        check_report(&w, id, Language::Ur, dir.path());
    }
}

#[test]
fn the_trial_balance_sheet_is_the_e2e_10_table() {
    let w = full_day();
    let dir = tempfile::tempdir().unwrap();
    let (_, sheet) = export_to(&w, ReportId::R18, Language::En, dir.path());
    let l10n = L10n::new(Language::En).unwrap();
    assert_eq!(sheet.tab, "Trial Balance");
    let total = sheet
        .find_text(0, &l10n.text("report-total"))
        .or_else(|| sheet.find_text(1, &l10n.text("report-total")))
        .unwrap();
    let nums: Vec<String> = (0..6)
        .filter_map(|c| match sheet.get(total, c) {
            Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(nums, ["85048", "85048"], "both sides of the trial balance");
    // Cash 58,484, Akram 2,880, Arshad 18,684 are debits in the sheet; Karim 10,160 a credit.
    let numbers: Vec<String> = (0..=sheet.last_row())
        .flat_map(|r| (0..6).map(move |c| (r, c)))
        .filter_map(|(r, c)| match sheet.get(r, c) {
            Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    for want in [
        "58484", "2880", "18684", "3000", "2000", "10160", "2484", "480", "300", "4000", "100",
        "65000",
    ] {
        assert!(numbers.contains(&want.to_owned()), "{want} in {numbers:?}");
    }
}

#[test]
fn buyer_dues_are_the_e2e_33_numbers_and_a_balance_keeps_its_side() {
    let w = full_day();
    let dir = tempfile::tempdir().unwrap();
    let (_, sheet) = export_to(&w, ReportId::R20, Language::En, dir.path());
    let numbers: Vec<String> = (0..=sheet.last_row())
        .flat_map(|r| (0..12).map(move |c| (r, c)))
        .filter_map(|(r, c)| match sheet.get(r, c) {
            Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    for want in ["2880", "18684", "21564"] {
        assert!(numbers.contains(&want.to_owned()), "{want} in {numbers:?}");
    }
    // The total 21,564 stands in the totals row, under the balance column.
    let total = (0..=sheet.last_row())
        .find(|r| {
            sheet
                .row_texts(*r)
                .contains(&L10n::new(Language::En).unwrap().text("report-total"))
        })
        .unwrap();
    let in_row: Vec<String> = (0..12)
        .filter_map(|c| match sheet.get(total, c) {
            Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    assert!(in_row.contains(&"21564".to_owned()), "{in_row:?}");
    // Akram's ledger closes at 2,880 debit; a credit side would be negative.
    let (_, ledger) = export_to(&w, ReportId::R13, Language::En, dir.path());
    assert!(ledger.styles.contains("Debit") && ledger.styles.contains("Credit"));
    let page = page_of(&w, ReportId::R13);
    let closing = page
        .summary
        .iter()
        .filter(|s| s.label_id == "report-closing-balance")
        .find_map(|s| match &s.cell {
            Cell::Balance { paisa, side } => Some(signed(*side, paisa.rupees())),
            _ => None,
        });
    assert_eq!(closing, Some(2_880));
    // The sheet carries it as the number 2880 next to the words "Closing balance".
    let row = ledger
        .find_text(
            0,
            &L10n::new(Language::En)
                .unwrap()
                .text("report-closing-balance"),
        )
        .unwrap();
    assert_eq!(ledger.num(row, 1), "2880");
}

#[test]
fn seller_payables_show_karim_owed_10160() {
    let w = full_day();
    let dir = tempfile::tempdir().unwrap();
    let (_, sheet) = export_to(&w, ReportId::R24, Language::En, dir.path());
    let numbers: Vec<String> = (0..=sheet.last_row())
        .flat_map(|r| (0..12).map(move |c| (r, c)))
        .filter_map(|(r, c)| match sheet.get(r, c) {
            Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    assert!(numbers.contains(&"10160".to_owned()), "{numbers:?}");
    // A credit balance is negative in the sheet: Karim's ledger balance is -10160.
    assert!(numbers.contains(&"-10160".to_owned()), "{numbers:?}");
}

#[test]
fn the_partner_split_sheet_has_the_e2e_9_shares() {
    let w = full_day();
    let dir = tempfile::tempdir().unwrap();
    let (_, sheet) = export_to(&w, ReportId::R30, Language::En, dir.path());
    let numbers: Vec<String> = (0..=sheet.last_row())
        .flat_map(|r| (0..10).map(move |c| (r, c)))
        .filter_map(|(r, c)| match sheet.get(r, c) {
            Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    for want in [
        "2196", "1646", "1200", "900", "996", "746", "5488", "3000", "2488",
    ] {
        assert!(numbers.contains(&want.to_owned()), "{want} in {numbers:?}");
    }
}

// ---- Analytics -------------------------------------------------------------------------------

fn check_view(w: &support::World, view: AnalyticsView, language: Language, dir: &Path) -> Sheet {
    let page = view_of(w, view);
    let path = dir.join(format!("view-{}-{language:?}.xlsx", view.code()));
    export_analytics(
        w.env.ctx.db(),
        &AnalyticsExportInput {
            view,
            filters: AnalyticsFilters::default(),
            language,
            path: path.display().to_string(),
        },
        &mut |_| {},
    )
    .unwrap();
    let sheet = xlsx_support::read(&path);
    let l10n = L10n::new(language).unwrap();
    let columns: Vec<String> = page
        .columns
        .iter()
        .filter(|c| c.heading.is_none())
        .map(|c| l10n.text(&c.title_id))
        .collect();
    // A matrix view has columns named by a month or an item: those headings are checked by
    // their rows below, so the full heading row is compared only for the plain views.
    if columns.len() == page.columns.len() {
        let rows: Vec<Vec<Cell>> = page.rows.iter().map(|r| r.cells.clone()).collect();
        check_page(&sheet, &columns, &rows, &page.totals, &l10n, view.code());
    } else {
        let first = l10n.text(&page.columns[0].title_id);
        let head = sheet.find_text(0, &first).unwrap();
        for (i, row) in page.rows.iter().enumerate() {
            let at = head + 1 + u32::try_from(i).unwrap();
            for (col, cell) in row.cells.iter().enumerate() {
                expect_cell(
                    &sheet,
                    at,
                    u32::try_from(col).unwrap(),
                    cell,
                    &l10n,
                    view.code(),
                );
            }
        }
    }
    assert_eq!(sheet.right_to_left, language == Language::Ur);
    sheet
}

#[test]
fn every_view_exports_with_every_number_equal_to_the_page() {
    let w = full_day();
    let dir = tempfile::tempdir().unwrap();
    for language in [Language::En, Language::Ur] {
        for view in ALL_VIEWS {
            check_view(&w, view, language, dir.path());
        }
    }
}

#[test]
fn view_a_is_the_section_18_4_table() {
    let w = full_day();
    let dir = tempfile::tempdir().unwrap();
    let sheet = check_view(&w, AnalyticsView::A, Language::En, dir.path());
    let numbers: Vec<String> = (0..=sheet.last_row())
        .flat_map(|r| (0..12).map(move |c| (r, c)))
        .filter_map(|(r, c)| match sheet.get(r, c) {
            Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    for want in [
        "50", "29400", "588", "1764", "400", "3928", "10", "12000", "1200", "720", "80", "1520",
        "60", "41400", "2484", "480", "5448",
    ] {
        assert!(numbers.contains(&want.to_owned()), "{want} in {numbers:?}");
    }
    // The shares are text, never a number (72.10% and 27.90%, 100.00% in the total).
    let texts = sheet.all_texts();
    for want in ["72.10%", "27.90%", "100.00%"] {
        assert!(texts.contains(&want.to_owned()), "{want} in {texts:?}");
    }
}

// ---- Refusals and the file -------------------------------------------------------------------

#[test]
fn a_bad_path_or_filter_is_refused_with_its_field_and_leaves_no_file() {
    let w = full_day();
    let dir = tempfile::tempdir().unwrap();
    let try_path = |path: String| {
        export_report(
            w.env.ctx.db(),
            &ReportExportInput {
                id: ReportId::R18,
                filters: ReportFilters::default(),
                language: Language::En,
                path,
            },
            &mut |_| {},
        )
    };
    for bad in [
        "relative.xlsx".to_owned(),
        dir.path().join("report.csv").display().to_string(),
        dir.path()
            .join("missing")
            .join("a.xlsx")
            .display()
            .to_string(),
        dir.path().display().to_string(),
    ] {
        match try_path(bad.clone()) {
            Err(ReportsError::Rejected(items)) => {
                assert_eq!(items.len(), 1, "{bad}");
                assert_eq!(
                    (items[0].code, items[0].field.as_str()),
                    (ErrorCode::ErrOutOfRange, "path")
                );
            }
            other => panic!("{bad}: {other:?}"),
        }
    }
    // A bad filter is refused with its own field, before any file is made.
    let target = dir.path().join("late.xlsx");
    let result = export_report(
        w.env.ctx.db(),
        &ReportExportInput {
            id: ReportId::R11,
            filters: ReportFilters {
                from: "not a date".into(),
                ..ReportFilters::default()
            },
            language: Language::En,
            path: target.display().to_string(),
        },
        &mut |_| {},
    );
    assert!(matches!(result, Err(ReportsError::Rejected(_))));
    assert!(!target.exists());
    assert!(
        std::fs::read_dir(dir.path()).unwrap().next().is_none(),
        "no partial file"
    );
}

fn big_sheet(rows: usize) -> ExportSheet {
    ExportSheet {
        name: "Big".into(),
        right_to_left: false,
        firm: "Firm".into(),
        title: "Big".into(),
        facts: Vec::new(),
        summary: Vec::new(),
        columns: vec![ExportColumn {
            title: "Amount".into(),
            shape: mandi_reports::export::ColumnShape::Money,
        }],
        rows: (0..rows)
            .map(|i| ExportRow {
                cells: vec![XCell::Money(rupees(i64::try_from(i).unwrap() * 100))],
                bold: false,
            })
            .collect(),
        totals_label: "Total".into(),
        totals: Vec::new(),
        debit_word: "Dr".into(),
        credit_word: "Cr".into(),
    }
}

#[test]
fn a_long_export_reports_progress_and_the_file_has_every_row() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("big.xlsx");
    let mut seen: Vec<ExportProgress> = Vec::new();
    write_workbook(&[big_sheet(12_000)], &path, &mut |p| seen.push(p)).unwrap();
    assert_eq!(seen.first().map(|p| p.stage), Some(ExportStage::Writing));
    assert_eq!(
        seen.last().map(|p| (p.stage, p.done, p.total)),
        Some((ExportStage::Done, 12_000, 12_000))
    );
    assert!(seen.len() > 20, "{} reports", seen.len());
    assert!(seen.windows(2).all(|pair| pair[0].done <= pair[1].done));
    let sheet = xlsx_support::read(&path);
    // Heading at row 3 (firm, title, blank), then 12,000 rows; the last holds 11,999 rupees.
    let head = sheet.find_text(0, "Amount").unwrap();
    assert_eq!(sheet.num(head + 12_000, 0), "1199900");
    assert_eq!(sheet.last_row(), head + 12_000);
    assert!(!path.with_extension("xlsx.partial").exists());
}

#[test]
fn more_rows_than_excel_holds_are_refused_and_an_oversized_amount_leaves_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("huge.xlsx");
    let result = write_workbook(&[big_sheet(1_048_500)], &path, &mut |_| {});
    assert!(matches!(result, Err(ReportsError::Rejected(_))));
    assert!(!path.exists());
    // An amount of 10^15 rupees would show a wrong last digit in Excel: refused, no half file.
    let mut sheet = big_sheet(1);
    sheet.rows[0].cells = vec![XCell::Money(rupees(1_000_000_000_000_000))];
    let result = write_workbook(&[sheet], &path, &mut |_| {});
    assert!(matches!(result, Err(ReportsError::Money(_))));
    assert!(!path.exists() && !dir.path().join("huge.xlsx.partial").exists());
}

#[test]
fn the_largest_allowed_amount_is_written_in_full() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("top.xlsx");
    let mut sheet = big_sheet(1);
    sheet.rows[0].cells = vec![XCell::Money(rupees(10_000_000_000))];
    write_workbook(&[sheet], &path, &mut |_| {}).unwrap();
    let read = xlsx_support::read(&path);
    let head = read.find_text(0, "Amount").unwrap();
    assert_eq!(read.num(head + 1, 0), "10000000000");
}

// ---- The readable pack -----------------------------------------------------------------------

fn pack_numbers(path: &Path) -> (Sheet, Vec<String>) {
    let sheet = xlsx_support::read(path);
    let numbers = (0..=sheet.last_row())
        .flat_map(|r| (0..12).map(move |c| (r, c)))
        .filter_map(|(r, c)| match sheet.get(r, c) {
            Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    (sheet, numbers)
}

#[test]
fn the_pack_has_five_sheets_with_the_worked_day_numbers() {
    let w = full_day();
    let out = tempfile::tempdir().unwrap();
    let files = pack::build_sheets(w.env.ctx.db(), "2026-10-10", out.path()).unwrap();
    let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "buyers-2026-10-10.xlsx",
            "sellers-2026-10-10.xlsx",
            "shop-expenses-2026-10-10.xlsx",
            "personal-expenses-2026-10-10.xlsx",
            "partner-shares-2026-10-10.xlsx"
        ]
    );
    let get = |name: &str| {
        files
            .iter()
            .find(|f| f.name.starts_with(name))
            .unwrap()
            .path
            .clone()
    };
    // Buyers: Akram 2,880 and Arshad 18,684, the total 21,564 (E2E-33).
    let (buyers, nums) = pack_numbers(&get("buyers"));
    for want in ["2880", "18684", "21564"] {
        assert!(nums.contains(&want.to_owned()), "{want} in {nums:?}");
    }
    // The default language is Urdu: the sheet reads right to left.
    assert!(buyers.right_to_left);
    // Sellers: Karim is owed 10,160.
    let (_, nums) = pack_numbers(&get("sellers"));
    assert!(nums.contains(&"10160".to_owned()), "{nums:?}");
    assert!(
        nums.contains(&"-10160".to_owned()),
        "a credit balance is negative: {nums:?}"
    );
    // Shop expenses: one line of 3,000 and the total 3,000 (the journal's 0501 net).
    let (shop, nums) = pack_numbers(&get("shop-expenses"));
    assert_eq!(
        nums.iter().filter(|n| *n == "3000").count(),
        2,
        "line and total: {nums:?}"
    );
    assert!(shop.all_texts().iter().any(|t| t.contains("05010001")));
    // Personal expenses: Alif took 2,000.
    let (_, nums) = pack_numbers(&get("personal-expenses"));
    assert_eq!(nums.iter().filter(|n| *n == "2000").count(), 2, "{nums:?}");
    // Partner shares: income 5,488, expenses 3,000, profit 2,488.
    let (_, nums) = pack_numbers(&get("partner-shares"));
    for want in ["996", "746", "5488", "3000", "2488"] {
        assert!(nums.contains(&want.to_owned()), "{want} in {nums:?}");
    }
    for file in &files {
        assert_eq!(xlsx_support::read(&file.path).formulas, 0);
    }
}

#[test]
fn the_pack_follows_the_default_language_of_the_settings() {
    let w = full_day();
    mandi_services::settings::set_default_language(&w.env.ctx, &w.owner, Language::En).unwrap();
    let out = tempfile::tempdir().unwrap();
    let files = pack::build_sheets(w.env.ctx.db(), "2026-10-10", out.path()).unwrap();
    let l10n = L10n::new(Language::En).unwrap();
    let (buyers, _) = pack_numbers(&files[0].path);
    assert!(!buyers.right_to_left);
    assert_eq!(buyers.tab, l10n.text("report-r20-title"));
    let (shop, _) = pack_numbers(&files[2].path);
    assert_eq!(shop.tab, l10n.text("report-doc-shop-expense"));
}

#[test]
fn a_reversed_expense_shows_with_its_reversal_and_the_total_equals_the_journal() {
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::{PostingEvent, post, reverse};
    let w = full_day();
    let expense = support::id_of(&w.env, "05010001");
    let cash = support::id_of(&w.env, "01010001");
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(expense), Side::Dr, rupees(500)),
        LedgerLine::new(AccountId::new(cash), Side::Cr, rupees(500)),
    ])
    .unwrap();
    let date = mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10).unwrap();
    w.env
        .ctx
        .db()
        .write(|tx| {
            let posted = post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date,
                    posting_type: PostingType::ShopExpense,
                    source_type: SourceType::Voucher,
                    source_id: 9_003,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )?;
            reverse(&w.env.ctx, tx, w.owner.user_id, posted.entry_id, "wrong")
        })
        .unwrap();
    let out = tempfile::tempdir().unwrap();
    let files = pack::build_sheets(w.env.ctx.db(), "2026-10-10", out.path()).unwrap();
    let (shop, nums) = pack_numbers(&files[2].path);
    // 3,000, then 500 and its reversal -500, and the total 3,000 again.
    assert!(
        nums.contains(&"500".to_owned()) && nums.contains(&"-500".to_owned()),
        "{nums:?}"
    );
    assert_eq!(nums.iter().filter(|n| *n == "3000").count(), 2, "{nums:?}");
    let _ = shop;
    // The same total from the journal itself.
    let net: i64 = raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(l.dr_paisa), 0) - COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_lines l JOIN accounts a ON a.id = l.account_id
             JOIN journal_entries e ON e.id = l.entry_id
             WHERE a.code LIKE '0501%' AND e.status = 'posted' AND e.source_type <> 'month_close'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(net, 300_000);
}

#[test]
fn money_a_partner_gives_back_nets_off_his_personal_expense_in_the_pack() {
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::{PostingEvent, post};
    let w = full_day();
    let drawings = support::id_of(&w.env, "06030001");
    let cash = support::id_of(&w.env, "01010001");
    // Alif returns 500 of the 2,000 he took (ذاتی خرچہ واپسی, E2E-29 in small).
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(cash), Side::Dr, rupees(500)),
        LedgerLine::new(AccountId::new(drawings), Side::Cr, rupees(500)),
    ])
    .unwrap();
    let date = mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10).unwrap();
    w.env
        .ctx
        .db()
        .write(|tx| {
            post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date,
                    posting_type: PostingType::PersonalReturn,
                    source_type: SourceType::Voucher,
                    source_id: 9_004,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
    let out = tempfile::tempdir().unwrap();
    let files = pack::build_sheets(w.env.ctx.db(), "2026-10-10", out.path()).unwrap();
    let (_, nums) = pack_numbers(&files[3].path);
    // 2,000 taken, 500 returned (a credit, so -500), and the net 1,500 in the total.
    assert!(
        nums.contains(&"2000".to_owned()) && nums.contains(&"-500".to_owned()),
        "{nums:?}"
    );
    assert!(nums.contains(&"1500".to_owned()), "{nums:?}");
}

#[test]
fn the_pack_listing_query_uses_the_indexes() {
    let w = full_day();
    let conn = raw(&w.dir);
    let plan: Vec<String> = conn
        .prepare(&format!("EXPLAIN QUERY PLAN {}", pack::lines_sql()))
        .unwrap()
        .query_map(["0501", "0502", "2026-10-01", "2026-10-31"], |row| {
            row.get::<_, String>(3)
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let text = plan.join("\n");
    assert!(
        !text.contains("SCAN l") && !text.contains("SCAN journal_lines"),
        "{text}"
    );
}

#[test]
fn the_pack_holds_the_snapshot_numbers_even_when_the_live_books_change_afterwards() {
    use mandi_services::core::ids::AccountId;
    use mandi_services::db::rusqlite::{Connection, backup::Backup};
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::{PostingEvent, post};
    let w = full_day();
    // The snapshot: a consistent copy of the books as they are now (what a backup takes).
    let dir = tempfile::tempdir().unwrap();
    let snapshot = dir.path().join("snapshot.db");
    {
        let live = raw(&w.dir);
        let mut copy = Connection::open(&snapshot).unwrap();
        Backup::new(&live, &mut copy)
            .unwrap()
            .run_to_completion(64, std::time::Duration::ZERO, None)
            .unwrap();
    }
    // The books move on: Akram pays 1,000 and a 700 shop expense is posted.
    mandi_services::vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "1,000"))
        .unwrap();
    let expense = support::id_of(&w.env, "05010001");
    let cash = support::id_of(&w.env, "01010001");
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(expense), Side::Dr, rupees(700)),
        LedgerLine::new(AccountId::new(cash), Side::Cr, rupees(700)),
    ])
    .unwrap();
    let date = mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10).unwrap();
    w.env
        .ctx
        .db()
        .write(|tx| {
            post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date,
                    posting_type: PostingType::ShopExpense,
                    source_type: SourceType::Voucher,
                    source_id: 9_005,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
    let before = std::fs::read(&snapshot).unwrap();
    let from_snapshot = tempfile::tempdir().unwrap();
    let files = pack::build_sheets_from_snapshot(
        &snapshot,
        w.env.ctx.db().clock().clone(),
        "2026-10-10",
        from_snapshot.path(),
    )
    .unwrap();
    // The sheets hold the numbers of the snapshot: buyers 21,564, shop expenses 3,000.
    let (_, buyers) = pack_numbers(&files[0].path);
    assert!(
        buyers.contains(&"21564".to_owned()) && !buyers.contains(&"20564".to_owned()),
        "{buyers:?}"
    );
    let (_, shop) = pack_numbers(&files[2].path);
    assert_eq!(shop.iter().filter(|n| *n == "3000").count(), 2, "{shop:?}");
    assert!(
        !shop.contains(&"700".to_owned()) && !shop.contains(&"3700".to_owned()),
        "{shop:?}"
    );
    let (_, cash_sellers) = pack_numbers(&files[1].path);
    assert!(cash_sellers.contains(&"10160".to_owned()));
    // The snapshot file itself was only read.
    assert_eq!(std::fs::read(&snapshot).unwrap(), before);
    // The live books really are different: the same pack from them shows 20,564 and 3,700.
    let from_live = tempfile::tempdir().unwrap();
    let live = pack::build_sheets(w.env.ctx.db(), "2026-10-10", from_live.path()).unwrap();
    let (_, buyers_now) = pack_numbers(&live[0].path);
    assert!(buyers_now.contains(&"20564".to_owned()), "{buyers_now:?}");
    let (_, shop_now) = pack_numbers(&live[2].path);
    assert!(shop_now.contains(&"3700".to_owned()), "{shop_now:?}");
    // A file that is not this app's database gives no pack.
    let junk = dir.path().join("junk.db");
    std::fs::write(&junk, b"not a database").unwrap();
    let none = tempfile::tempdir().unwrap();
    assert!(
        pack::build_sheets_from_snapshot(
            &junk,
            w.env.ctx.db().clock().clone(),
            "2026-10-10",
            none.path()
        )
        .is_err()
    );
    assert!(std::fs::read_dir(none.path()).unwrap().next().is_none());
}

#[test]
fn a_pack_for_a_bad_day_is_refused() {
    let w = full_day();
    let out = tempfile::tempdir().unwrap();
    assert!(pack::build_sheets(w.env.ctx.db(), "not-a-day", out.path()).is_err());
    assert!(std::fs::read_dir(out.path()).unwrap().next().is_none());
}
