//! The spreadsheets of the readable daily pack (PRD FR-20.5, NFR-BAK-13): buyers, sellers, shop
//! expenses, personal expenses and partner shares, so the books can be read without the app.
//!
//! The pack language is the default language of the settings (PRD Section 21). All five sheets
//! come from one database read, as of the shop day:
//!
//! - buyers: R20, the dues of every buyer as of the day;
//! - sellers: R24, what every seller is owed and his open advance as of the day;
//! - shop expenses: the posted lines of the 0501 heads, from the first of the month to the day;
//! - personal expenses: the posted lines of the partners' 0603 accounts, same period;
//! - partner shares: R30, income, expenses and each partner's share, same period.
//!
//! Every sheet shows the day, the filters and the time it was made, so an old pack cannot be
//! mistaken for today's books. The files are plain `.xlsx`; the backup crate encrypts them like
//! every cloud object. Values only, no formulas. Reads only.
//!
//! Example (10-10-26, English): `buyers-2026-10-10.xlsx` holds Akram 2,880 and Arshad 18,684 and
//! the total 21,564.

mod lists;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mandi_core::auth::Language;
use mandi_core::dates::BusinessDate;
use mandi_core::money::Grouping;
use mandi_db::clock::Clock;
use mandi_db::{Db, DbOptions};

use crate::error::ReportsError;
use crate::export::cells::{CellContext, date_cell};
use crate::export::l10n::L10n;
use crate::export::model::{
    ColumnShape, ExportColumn, ExportProgress, ExportRow, ExportSheet, XCell,
};
use crate::export::report::{self, Header};
use crate::export::{fmt_of, header_of, write_workbook};
use crate::print::load::{FirmNames, load_default_language, load_firm, load_grouping};
use crate::reports::print_data::Fmt;
use crate::reports::{ReportFilters, ReportId, ReportPage};

#[doc(hidden)]
pub use lists::lines_sql;

/// One file of the pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackSheetFile {
    /// The file name, for example `buyers-2026-10-10.xlsx`.
    pub name: String,
    /// Where it was written.
    pub path: PathBuf,
}

/// The first day of the month of `day`, as `YYYY-MM-DD`.
fn month_start(day: BusinessDate) -> Result<String, ReportsError> {
    BusinessDate::from_ymd(day.year(), day.month(), 1)
        .map(BusinessDate::to_iso)
        .ok_or_else(|| ReportsError::Data("the first day of the month".to_owned()))
}

/// A listing as a sheet: date, document, account, narration, amount, with the total under the
/// amount.
fn listing_sheet(
    title_id: &str,
    listing: &lists::Listing,
    (from_iso, to_iso): (&str, &str),
    (fmt, l10n, header): (&Fmt, &L10n, &Header),
) -> Result<ExportSheet, ReportsError> {
    let title = l10n.text(title_id);
    let mut rows = Vec::with_capacity(listing.lines.len());
    for line in &listing.lines {
        rows.push(ExportRow {
            cells: vec![
                date_cell(&line.date_iso)?,
                XCell::Text(l10n.text(&crate::reports::ids::doc_id(&line.posting_type))),
                XCell::Text(format!(
                    "{} {}",
                    line.account_code,
                    fmt.name(&line.name_ur, &line.name_en)
                )),
                XCell::Text(line.narration.clone()),
                XCell::Money(line.amount),
            ],
            bold: false,
        });
    }
    let facts = vec![
        (l10n.text("report-filter-from"), fmt.style.date(from_iso)?),
        (l10n.text("report-filter-to"), fmt.style.date(to_iso)?),
        (
            l10n.text("report-print-row-count"),
            fmt.style.count(i64::try_from(rows.len()).unwrap_or(0)),
        ),
        (l10n.text("print-printed-at"), header.printed_at.clone()),
    ];
    Ok(ExportSheet {
        name: title.clone(),
        right_to_left: header.right_to_left,
        firm: header.firm.clone(),
        title,
        facts,
        summary: Vec::new(),
        columns: vec![
            ExportColumn {
                title: l10n.text("report-col-date"),
                shape: ColumnShape::Date,
            },
            ExportColumn {
                title: l10n.text("report-col-doc"),
                shape: ColumnShape::Text,
            },
            ExportColumn {
                title: l10n.text("report-col-account"),
                shape: ColumnShape::Text,
            },
            ExportColumn {
                title: l10n.text("report-col-narration"),
                shape: ColumnShape::Text,
            },
            ExportColumn {
                title: l10n.text("report-col-amount"),
                shape: ColumnShape::Money,
            },
        ],
        rows,
        totals_label: l10n.text("report-total"),
        totals: vec![
            XCell::Empty,
            XCell::Empty,
            XCell::Empty,
            XCell::Empty,
            XCell::Money(listing.total),
        ],
        debit_word: l10n.text("report-side-dr"),
        credit_word: l10n.text("report-side-cr"),
    })
}

/// Everything the five sheets are made from, read in one go.
struct PackData {
    language: Language,
    firm: FirmNames,
    grouping: Grouping,
    buyers: ReportPage,
    sellers: ReportPage,
    shares: ReportPage,
    shop: lists::Listing,
    personal: lists::Listing,
    first_iso: String,
    day_iso: String,
}

/// Reads the books once for all five sheets, as of the day.
fn read_data(db: &Db, today: BusinessDate) -> Result<PackData, ReportsError> {
    let day_iso = today.to_iso();
    let first_iso = month_start(today)?;
    let as_of = ReportFilters {
        as_of: day_iso.clone(),
        ..ReportFilters::default()
    };
    let period = ReportFilters {
        from: first_iso.clone(),
        to: day_iso.clone(),
        ..ReportFilters::default()
    };
    // One read transaction for all five sheets, so they come from the same moment.
    db.read(|conn| {
        crate::reports::common::in_snapshot(conn, |conn| {
            Ok(PackData {
                language: load_default_language(conn)?,
                firm: load_firm(conn)?,
                grouping: load_grouping(conn)?,
                buyers: crate::reports::page_in(conn, db, ReportId::R20, &as_of)?,
                sellers: crate::reports::page_in(conn, db, ReportId::R24, &as_of)?,
                shares: crate::reports::page_in(conn, db, ReportId::R30, &period)?,
                shop: lists::read(conn, lists::SHOP_EXPENSES, &first_iso, &day_iso)?,
                personal: lists::read(conn, lists::PERSONAL_EXPENSES, &first_iso, &day_iso)?,
                first_iso: first_iso.clone(),
                day_iso: day_iso.clone(),
            })
        })
    })
}

/// Makes the five spreadsheets from a SNAPSHOT file, opened for reading only. The pack travels
/// with a backup, so it must hold the numbers of that backup and not of the live books, which
/// may have changed since (PRD FR-20.5, NFR-BAK-13). `clock` gives the time written on the
/// sheets; the snapshot is never written, upgraded or locked for writing.
///
/// # Errors
/// As [`build_sheets`]; `ReportsError::Db` when the snapshot is not a database of this app's
/// schema.
pub fn build_sheets_from_snapshot(
    snapshot: &Path,
    clock: Arc<dyn Clock>,
    day: &str,
    out_dir: &Path,
) -> Result<Vec<PackSheetFile>, ReportsError> {
    let options = DbOptions {
        clock,
        ..DbOptions::new(out_dir)
    };
    let db = Db::open_read_only(snapshot, options)?;
    build_sheets(&db, day, out_dir)
}

/// Makes the five spreadsheets of the pack for one shop day in `out_dir`.
///
/// # Errors
/// `ReportsError::Data` for a day that is not a date; a database or money error;
/// `ReportsError::File` when a file cannot be written.
pub fn build_sheets(
    db: &Db,
    day: &str,
    out_dir: &Path,
) -> Result<Vec<PackSheetFile>, ReportsError> {
    let today = BusinessDate::parse_iso(day)
        .ok_or_else(|| ReportsError::Data(format!("the pack day is not a date: {day}")))?;
    let data = read_data(db, today)?;
    let now = db.clock().now_utc();
    let l10n = L10n::new(data.language)?;
    let fmt = fmt_of(data.language, data.grouping, &data.firm, now);
    let header = header_of(&fmt, &data.firm, data.language, now);
    let ctx = CellContext {
        fmt: &fmt,
        l10n: &l10n,
    };
    let listing = |title_id: &str, found: &lists::Listing| {
        listing_sheet(
            title_id,
            found,
            (&data.first_iso, &data.day_iso),
            (&fmt, &l10n, &header),
        )
    };
    let sheets = [
        ("buyers", report::sheet(&data.buyers, &ctx, &header)?),
        ("sellers", report::sheet(&data.sellers, &ctx, &header)?),
        (
            "shop-expenses",
            listing("report-doc-shop-expense", &data.shop)?,
        ),
        (
            "personal-expenses",
            listing("report-doc-personal-expense", &data.personal)?,
        ),
        (
            "partner-shares",
            report::sheet(&data.shares, &ctx, &header)?,
        ),
    ];
    let day_iso = today.to_iso();
    let mut files = Vec::with_capacity(sheets.len());
    for (stem, sheet) in sheets {
        let name = format!("{stem}-{day_iso}.xlsx");
        let path = out_dir.join(&name);
        let mut ignore = |_: ExportProgress| {};
        write_workbook(&[sheet], &path, &mut ignore)?;
        files.push(PackSheetFile { name, path });
    }
    Ok(files)
}
