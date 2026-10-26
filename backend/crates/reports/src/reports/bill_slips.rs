//! The print data of R5 "Print All": one slip for each credit buyer (PRD 19.2 R5, FR-19.10,
//! FR-19.11, FR-19.13; E2E-36).
//!
//! The client's print of 18-10-26 (ادھاری تفصیل) has one row for each buyer; "Print All" gives each
//! buyer his own slip, with a page break between buyers. A slip holds the buyer, his credit lines
//! of the filter (the serial inside the slip, the line written "(item qty . rate)", its value and
//! its value after commission and لاگا, C5) and his sums. A one-time person (پختہ کھاتہ, FR-13.5)
//! has a slip of his own. A buyer who paid cash on the spot has no slip: his lines are نقدی.
//!
//! Every figure is a value stored on the line when the بکری was saved (Code Standards B20). This
//! file only lays them out: it formats with the same [`Fmt`] as every report print and calculates
//! nothing; the sums are made once, by checked addition, in `sales::buyer_bills::slips`, and the
//! slips add up to the totals of R5 and of R4. Every row is read (no paging, FR-19.11). The shared
//! header with the contacts and the trade mark joins in plan 17e (FR-19.13); until then the
//! header is the one of the other report prints.
//!
//! Example (10-10-26, English): slip 1 is "0020 Akram Toll Tax" with one row, serial 1,
//! "(Banana 20 . 600)", 12,000 and 12,880, total Nag 20. Slip 2 is "0021 Arshad Multani",
//! "(Banana 30 . 580)", 17,400 and 18,684. Shahid has no slip. The totals row reads 29,400 and
//! 31,564.

use mandi_core::auth::Language;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use serde::Serialize;
use ts_rs::TS;

use super::ids;
use super::model::{
    Cell, ColumnKind, DetailPart, ReportId, ReportPrintInput, ResolvedFilters, RowKind,
};
use super::print_data::{
    Fmt, PrintCell, PrintOptions, PrintReportColumn, PrintReportParty, PrintReportRow, filter_facts,
};
use super::sales::Slips;
use crate::error::ReportsError;
use crate::print::{PrintFact, TextDir};

/// One buyer's slip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BuyerBillSlip {
    /// The number of the slip in this print, 1 for the first.
    pub serial: String,
    /// The buyer: short code, name with tag. A one-time person has his person key.
    pub party: PrintReportParty,
    /// His lines, one cell for each column of the print: serial, detail, value, total.
    pub rows: Vec<PrintReportRow>,
    /// How many lines there are, as printed.
    pub row_count: String,
    /// The quantity of all his lines (نگ), as printed.
    pub qty: String,
    /// His sums, one cell for each column: nothing, nothing, the value, the total.
    pub totals: Vec<PrintCell>,
}

/// Everything a template needs to print all the buyer slips of R5.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BuyerBillsPrintData {
    /// The report: always R5.
    pub report: ReportId,
    /// The print language.
    pub language: Language,
    /// The writing direction.
    pub dir: TextDir,
    /// The paper class.
    pub paper: Paper,
    /// True when Urdu digits are used on this print.
    pub urdu_digits: bool,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The filters of the run, the facts under the title: dates, buyer, seller and so on
    /// (FR-19.10). They go to `reports_print_header` with the title.
    pub filters: Vec<PrintFact>,
    /// The columns of every slip.
    pub columns: Vec<PrintReportColumn>,
    /// The slips, in the Urdu order of the buyers' names. The template starts a new page for each.
    pub slips: Vec<BuyerBillSlip>,
    /// How many slips there are, as printed.
    pub slip_count: String,
    /// The quantity of all slips (نگ), as printed.
    pub qty: String,
    /// Fluent message ID of the label of the totals of all slips.
    pub totals_label_id: String,
    /// The sums of all slips, one cell for each column: the totals row of R5.
    pub totals: Vec<PrintCell>,
    /// Fluent message ID of the label of a slip's quantity total (total Nag).
    pub qty_label_id: String,
}

/// The columns of a slip.
fn columns() -> Vec<PrintReportColumn> {
    [
        ("serial", ids::COL_SERIAL, ColumnKind::Number),
        ("detail", ids::COL_DETAIL, ColumnKind::Detail),
        ("value", ids::COL_VALUE, ColumnKind::Money),
        ("buyer_total", ids::COL_BUYER_TOTAL, ColumnKind::Money),
    ]
    .into_iter()
    .map(|(key, title_id, kind)| PrintReportColumn {
        key: key.to_owned(),
        title_id: title_id.to_owned(),
        kind,
    })
    .collect()
}

/// A row of sums: nothing under the serial and the detail.
fn sums(fmt: &Fmt, value: Paisa, total: Paisa) -> Result<Vec<PrintCell>, ReportsError> {
    fmt.cells(&[
        Cell::Empty,
        Cell::Empty,
        Cell::Money { paisa: value },
        Cell::Money { paisa: total },
    ])
}

/// Lays out the slips as print data. Pure: it reads nothing and calculates nothing.
///
/// # Errors
/// `ReportsError::Money` for an amount that does not fit, `ReportsError::Data` for a stored date
/// that is not a date.
pub(crate) fn build(
    slips: &Slips,
    filters: &ResolvedFilters,
    options: &PrintOptions,
) -> Result<BuyerBillsPrintData, ReportsError> {
    let fmt = Fmt::new(options);
    let mut out = Vec::with_capacity(slips.slips.len());
    for (index, slip) in slips.slips.iter().enumerate() {
        let mut rows = Vec::with_capacity(slip.lines.len());
        for (line_index, line) in slip.lines.iter().enumerate() {
            let serial = i64::try_from(line_index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .ok_or_else(|| ReportsError::Data("the line serial does not fit".to_owned()))?;
            let value = match &line.part {
                DetailPart::Sale { value_paisa, .. } => *value_paisa,
                _ => {
                    return Err(ReportsError::Data(
                        "a buyer slip line is not a sale line".to_owned(),
                    ));
                }
            };
            rows.push(PrintReportRow {
                cells: fmt.cells(&[
                    Cell::Number { value: serial },
                    Cell::Detail {
                        parts: vec![line.part.clone()],
                    },
                    Cell::Money { paisa: value },
                    Cell::Money { paisa: line.total },
                ])?,
                kind: RowKind::Normal,
            });
        }
        let serial = i64::try_from(index)
            .ok()
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| ReportsError::Data("the slip serial does not fit".to_owned()))?;
        out.push(BuyerBillSlip {
            serial: fmt.style.count(serial),
            party: fmt.party(&slip.party),
            row_count: fmt
                .style
                .count(i64::try_from(rows.len()).unwrap_or(i64::MAX)),
            rows,
            qty: fmt.style.count(slip.qty),
            totals: sums(&fmt, slip.value, slip.total)?,
        });
    }
    Ok(BuyerBillsPrintData {
        report: ReportId::R5,
        language: options.language,
        dir: TextDir::of(options.language),
        paper: options.paper,
        urdu_digits: fmt.style.urdu_digits(),
        title_id: ids::title_id("R5"),
        filters: filter_facts(&fmt, ReportId::R5, filters)?,
        columns: columns(),
        slip_count: fmt
            .style
            .count(i64::try_from(out.len()).unwrap_or(i64::MAX)),
        slips: out,
        qty: fmt.style.count(slips.qty),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: sums(&fmt, slips.value, slips.total)?,
        qty_label_id: ids::R35_TOTAL_QTY.to_owned(),
    })
}

/// Refuses an input that is not about R5: "Print All" has only this report.
///
/// # Errors
/// `ReportsError::Rejected` with `ERR_OUT_OF_RANGE` on `id`.
pub(crate) fn check_input(input: &ReportPrintInput) -> Result<(), ReportsError> {
    if input.id != ReportId::R5 {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "id",
        )]));
    }
    Ok(())
}
