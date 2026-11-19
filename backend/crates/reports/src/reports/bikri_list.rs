//! The print data of پچھلی بکری (PRD 12.7, FR-12.46, FR-19.3, FR-19.10, FR-19.13; plan 72 P8).
//!
//! The screen shows today's بکری and, below it, a window of days. A print follows the same choice
//! (the filters, or "show every بکری"): it holds every row of today's section and of the window,
//! grouped by day with a heading for each (date, count, خام بکری), then the totals of the window.
//!
//! The services read the whole list with the same query as the screen ([`BikriListWhole`]); nothing
//! is worked out here. Every figure is a stored value, formatted for the print language: digits 0 to
//! 9, grouping by the setting, names in the print language. Titles, column headings and the status
//! words are Fluent message IDs; the shared header and footer come from `reports_print_header` with
//! `title_id` and `filters` (FR-19.13), so no header is built here.
//!
//! Example (plan 72, today 12-11-2026): the window 05-11-2026 to 11-11-2026 prints one heading and
//! its rows for each of the 7 days, newest first; 06-11 to 10-11 have headings and no rows.

use chrono::{DateTime, Duration, Utc};
use mandi_core::auth::Language;
use mandi_core::bikri::{
    BikriDay, BikriList, BikriListRow, BikriListWhole, BikriListWholeInput, BikriStatus, lot_label,
};
use mandi_core::printing::Paper;
use mandi_db::Db;
use mandi_db::rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::common::{MasterTable, col, load_party, names_cell, names_of};
use super::ids;
use super::model::{ArgValue, Cell, Column, ColumnKind, MessageArg, Names};
use super::print_data::{Fmt, PrintCell, PrintOptions, PrintReportColumn, PrintReportRow, fact};
use crate::error::ReportsError;
use crate::print::load::load_grouping;
use crate::print::{PrintFact, TextDir};

/// Fluent message ID of the title of the print and the sheet.
pub const TITLE_ID: &str = "bikri-list-print-title";
/// The label of the fact "every بکری shown" (the switch پوری بکری دکھائیں).
const FILTER_ALL: &str = "bikri-list-print-all";
/// The label of the fact with the typed search.
const FILTER_SEARCH: &str = "bikri-list-print-search";
/// The status words, as the screen's own.
const STATUS_SAVED: &str = "bikri-status-saved";
const STATUS_CANCELLED: &str = "bikri-status-cancelled";
/// A status with the edited mark: "Saved · Edited". Argument `status`: a message.
const STATUS_EDITED: &str = "bikri-list-status-edited";
/// The heading of a day (argument `date`, argument `count`) and of a day with none (argument `date`).
pub(crate) const DAY_HEADING: &str = "bikri-list-day-heading";
pub(crate) const DAY_HEADING_EMPTY: &str = "bikri-list-day-heading-empty";
/// The heading of today's section (arguments `date` and `count`).
pub(crate) const TODAY_HEADING: &str = "bikri-list-today-heading";

/// Pakistan time is UTC plus five hours all year.
const PAKISTAN_OFFSET_HOURS: i64 = 5;

/// Input of `bikri_list_print_data`: the window and the filters of the screen, the print language
/// and the paper (A4 when left out). No page and no sort: every row is printed, newest day first.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct BikriListPrintInput {
    /// The window, the filters and "show everything".
    #[serde(flatten)]
    #[ts(flatten)]
    pub list: BikriListWholeInput,
    /// The print language (PRD FR-7.36): names, words and digits follow it.
    pub language: Language,
    /// The paper. Missing means A4.
    #[serde(default)]
    pub paper: Option<Paper>,
}

/// One day of the print: its heading and every row of it. Today's section has the same shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BikriListPrintDay {
    /// The day as printed, `DD-MM-YYYY`.
    pub date: String,
    /// The day, `YYYY-MM-DD`.
    pub date_iso: String,
    /// How many بکری the day has under the filter, as printed.
    pub count: String,
    /// Σ خام بکری of the day, as printed.
    pub gross: String,
    /// Every row of the day, newest first. Empty for a day with no بکری.
    pub rows: Vec<PrintReportRow>,
}

/// Everything a template needs to print پچھلی بکری with every row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BikriListPrintData {
    /// The print language.
    pub language: Language,
    /// The writing direction.
    pub dir: TextDir,
    /// The paper class.
    pub paper: Paper,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The window and the filters, the facts under the title (FR-19.10). They go to
    /// `reports_print_header` with the title. With "show everything" the other filters are not here,
    /// and a fact says so.
    pub filters: Vec<PrintFact>,
    /// The columns, the same for every day.
    pub columns: Vec<PrintReportColumn>,
    /// Today's section. Null when `with_today` was off.
    pub today: Option<BikriListPrintDay>,
    /// The days of the window, newest first, empty days included.
    pub days: Vec<BikriListPrintDay>,
    /// How many بکری the print holds, today's section and the window together, as printed.
    pub row_count: String,
    /// Fluent message ID of the totals row label.
    pub totals_label_id: String,
    /// The totals of the window over the whole filter, one cell for each column.
    pub totals: Vec<PrintCell>,
}

/// The columns of the print and the sheet (PRD FR-12.46): number, day, time, truck, sellers, items,
/// نگ sold and received, خام بکری, صافی بکری, buyer total and status.
pub(crate) fn columns() -> Vec<Column> {
    vec![
        col("number", "bikri-list-col-number", ColumnKind::Text),
        col("date", "report-col-date", ColumnKind::Date),
        col("time", "bikri-list-col-time", ColumnKind::Text),
        col("truck", "bikri-list-col-truck", ColumnKind::Text),
        col("sellers", "bikri-list-col-sellers", ColumnKind::Names),
        col("items", "bikri-list-col-items", ColumnKind::Names),
        col("qty_sold", "bikri-saved-qty-sold", ColumnKind::Number),
        col("qty_received", "bikri-col-qty-received", ColumnKind::Number),
        col("gross", "glossary-kham-bikri", ColumnKind::Money),
        col("net", "glossary-safi-bikri", ColumnKind::Money),
        col(
            "buyer_total",
            "bikri-list-col-buyer-total",
            ColumnKind::Money,
        ),
        col("status", "bikri-list-col-status", ColumnKind::Message),
    ]
}

/// How many بکری the print or the sheet holds: today's section and the window.
///
/// # Errors
/// `ReportsError::Data` if the count does not fit.
pub(crate) fn printed_rows(whole: &BikriListWhole) -> Result<i64, ReportsError> {
    let today = whole.today.as_ref().map_or(0, |list| list.total_count);
    whole
        .window
        .total_count
        .checked_add(today)
        .ok_or_else(|| ReportsError::Data("too many rows to print".to_owned()))
}

/// The place of the خام بکری column: the heading rows of a sheet hold the day's total there.
pub(crate) const GROSS_COLUMN: usize = 8;

/// The time a بکری was saved in Pakistan time, `HH:MM` (the screen does the same).
fn local_time(created_at: &str) -> Result<String, ReportsError> {
    let saved = DateTime::parse_from_rfc3339(created_at)
        .map_err(|e| ReportsError::Data(format!("a saved time is wrong: {created_at}: {e}")))?
        .with_timezone(&Utc);
    let local = saved
        .checked_add_signed(Duration::hours(PAKISTAN_OFFSET_HOURS))
        .ok_or_else(|| ReportsError::Data(format!("a saved time is out of range: {created_at}")))?;
    Ok(local.format("%H:%M").to_string())
}

/// The names of the sellers of a بکری in lot order, each once, with its tag, in both scripts.
fn sellers_names(row: &BikriListRow) -> Names {
    let join = |pick: fn(&mandi_core::accounts::RecordRef) -> &str, separator: &str| {
        row.sellers
            .iter()
            .map(|seller| match &seller.tag {
                Some(tag) => format!("{} {tag}", pick(seller)),
                None => pick(seller).to_owned(),
            })
            .collect::<Vec<_>>()
            .join(separator)
    };
    Names {
        ur: join(|s| s.name_ur.as_str(), "، "),
        en: join(|s| s.name_en.as_str(), ", "),
    }
}

/// The items of a بکری, one for each lot with its counter, in lot order: "Mango-2, Banana-1"
/// (PRD FR-12.45).
fn items_names(row: &BikriListRow) -> Names {
    let join = |pick: fn(&mandi_core::bikri::LotName) -> &str, separator: &str| {
        row.lots
            .iter()
            .map(|lot| lot_label(pick(lot), lot.counter))
            .collect::<Vec<_>>()
            .join(separator)
    };
    Names {
        ur: join(|l| l.item.name_ur.as_str(), "، "),
        en: join(|l| l.item.name_en.as_str(), ", "),
    }
}

/// The cells of one row, one for each of [`columns`]. Stored values only.
///
/// # Errors
/// `ReportsError::Data` for a stored time that is not a time.
pub(crate) fn row_cells(row: &BikriListRow) -> Result<Vec<Cell>, ReportsError> {
    let status_id = match row.status {
        BikriStatus::Saved => STATUS_SAVED,
        BikriStatus::Cancelled => STATUS_CANCELLED,
    };
    // The edited mark of FR-7.71: "Saved · Edited".
    let status = if row.flags.edited.is_some() {
        Cell::Message {
            id: STATUS_EDITED.to_owned(),
            args: vec![MessageArg {
                name: "status".to_owned(),
                value: ArgValue::Message {
                    id: status_id.to_owned(),
                    args: Vec::new(),
                },
            }],
        }
    } else {
        Cell::Label {
            id: status_id.to_owned(),
        }
    };
    let sellers = sellers_names(row);
    let items = items_names(row);
    Ok(vec![
        Cell::Text {
            text: row.number.to_string(),
        },
        Cell::Date {
            iso: row.entry_date.clone(),
        },
        Cell::Text {
            text: local_time(&row.created_at)?,
        },
        // A new بکری has no truck (PRD 12.2): its cell is empty; an old one shows its truck.
        super::common::text_cell(row.truck_no.clone()),
        names_cell(Some(sellers.ur), Some(sellers.en)),
        names_cell(Some(items.ur), Some(items.en)),
        Cell::Number {
            value: row.qty_sold,
        },
        Cell::Number {
            value: row.qty_received,
        },
        Cell::Money { paisa: row.gross },
        Cell::Money { paisa: row.net },
        Cell::Money {
            paisa: row.buyer_total,
        },
        status,
    ])
}

/// The totals row of the window: the stored sums over the whole filter, never a sum of the rows.
pub(crate) fn totals_cells(list: &BikriList) -> Vec<Cell> {
    let mut cells = vec![Cell::Empty; 6];
    cells.push(Cell::Number {
        value: list.totals.qty_sold,
    });
    cells.push(Cell::Number {
        value: list.totals.qty_received,
    });
    cells.push(Cell::Money {
        paisa: list.totals.gross,
    });
    cells.push(Cell::Money {
        paisa: list.totals.net,
    });
    cells.push(Cell::Money {
        paisa: list.totals.buyer_total,
    });
    cells.push(Cell::Empty);
    cells
}

/// The facts under the title: the window, then the filters that apply, or the switch (FR-19.10).
///
/// # Errors
/// A database error; `ReportsError::Data` for a window that has no days.
pub(crate) fn facts(
    conn: &Connection,
    fmt: &Fmt,
    input: &BikriListWholeInput,
    window: &BikriList,
) -> Result<Vec<PrintFact>, ReportsError> {
    let mut facts = Vec::new();
    // The days of the window come newest first; its ends are the first and last of them.
    if let (Some(last), Some(first)) = (window.days.last(), window.days.first()) {
        facts.push(fact(ids::FILTER_FROM, fmt.style.date(&last.date)?));
        facts.push(fact(ids::FILTER_TO, fmt.style.date(&first.date)?));
    }
    if input.all {
        // The filters keep their values on the screen but are set aside (PRD FR-12.46).
        facts.push(fact(FILTER_ALL, String::new()));
        return Ok(facts);
    }
    if let Some(id) = input.seller_id
        && let Some(party) = load_party(conn, id)?
    {
        facts.push(fact(ids::FILTER_SELLER, fmt.party_text(&party)));
    }
    if let Some(id) = input.item_id
        && let Some(names) = names_of(conn, MasterTable::Items, id)?
    {
        facts.push(fact(ids::FILTER_ITEM, fmt.names(&names)));
    }
    if let Some(status) = input.status {
        let label = match status {
            BikriStatus::Saved => STATUS_SAVED,
            BikriStatus::Cancelled => STATUS_CANCELLED,
        };
        facts.push(fact(label, String::new()));
    }
    if let Some(text) = input.search.as_deref().map(str::trim)
        && !text.is_empty()
    {
        facts.push(fact(FILTER_SEARCH, text.to_owned()));
    }
    Ok(facts)
}

/// One day with its heading figures and its rows formatted for print.
fn print_day(
    fmt: &Fmt,
    day: &BikriDay,
    rows: &[&BikriListRow],
) -> Result<BikriListPrintDay, ReportsError> {
    let mut printed = Vec::with_capacity(rows.len());
    for row in rows {
        printed.push(PrintReportRow {
            cells: fmt.cells(&row_cells(row)?)?,
            kind: super::model::RowKind::Normal,
        });
    }
    Ok(BikriListPrintDay {
        date: fmt.style.date(&day.date)?,
        date_iso: day.date.clone(),
        count: fmt.style.count(day.count),
        gross: fmt.style.money(day.gross_paisa)?,
        rows: printed,
    })
}

/// The print data of پچھلی بکری with every row (PRD FR-12.46, FR-19.10), for the print language.
/// Read only.
///
/// # Errors
/// A database error; `ReportsError::Data` for a stored value that is wrong.
pub fn bikri_list_print_data(
    db: &Db,
    whole: &BikriListWhole,
    input: &BikriListPrintInput,
) -> Result<BikriListPrintData, ReportsError> {
    let paper = input.paper.unwrap_or(Paper::A4);
    let (grouping, facts) = db.read(|conn| {
        let grouping = load_grouping(conn)?;
        let fmt = Fmt::new(&PrintOptions {
            language: input.language,
            grouping,
            paper,
        });
        let facts = facts(conn, &fmt, &input.list, &whole.window)?;
        Ok::<_, ReportsError>((grouping, facts))
    })?;
    let fmt = Fmt::new(&PrintOptions {
        language: input.language,
        grouping,
        paper,
    });
    // Today's section: one heading, every row of today.
    let today = match &whole.today {
        Some(list) => {
            let rows: Vec<&BikriListRow> = list.rows.iter().collect();
            let day = list
                .days
                .first()
                .ok_or_else(|| ReportsError::Data("today's section has no day".to_owned()))?;
            Some(print_day(&fmt, day, &rows)?)
        }
        None => None,
    };
    // The window: the rows of each day, in the order the list sent them (newest first).
    let mut days = Vec::with_capacity(whole.window.days.len());
    for day in &whole.window.days {
        let rows: Vec<&BikriListRow> = whole
            .window
            .rows
            .iter()
            .filter(|row| row.entry_date == day.date)
            .collect();
        days.push(print_day(&fmt, day, &rows)?);
    }
    let totals = fmt.cells(&totals_cells(&whole.window))?;
    Ok(BikriListPrintData {
        language: input.language,
        dir: TextDir::of(input.language),
        paper,
        title_id: TITLE_ID.to_owned(),
        filters: facts,
        columns: columns()
            .into_iter()
            .map(|c| PrintReportColumn {
                key: c.key,
                title_id: c.title_id,
                kind: c.kind,
            })
            .collect(),
        today,
        days,
        row_count: fmt.style.count(printed_rows(whole)?),
        totals_label_id: ids::TOTAL.to_owned(),
        totals,
    })
}
