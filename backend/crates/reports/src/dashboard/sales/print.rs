//! The print and the sheet of the sales page (plan 83, PRD FR-9.9, FR-19.3): the same table the
//! screen shows, with every row of the period, so the print and the sheet agree with the screen.
//!
//! The header names the period and, under a filter, the party, the item and the goods (PRD FR-19.10,
//! FR-19.13).

use mandi_core::auth::Language;
use mandi_core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_db::Db;

use super::model::{SalesPeriod, SalesPeriodInput, SalesPeriodRow, SalesPrintInput};
use super::read::sales_every_row;
use crate::analytics::print::goods_fact;
use crate::dashboard::commission::{CommissionPartyShown, CommissionPeriodKind, CommissionSide};
use crate::error::ReportsError;
use crate::print::PrintFact;
use crate::reports::{
    Cell, Column, ColumnKind, Row, RowKind, TablePrintData, TableSpec, table_print_data,
};

/// The title of a month, and of any other period.
const TITLE_MONTH: &str = "print-sales-title";
const TITLE_PERIOD: &str = "print-sales-title-period";
/// The heading of the date column, and the label of the totals row, as the commission page.
const COL_DATE: &str = "print-commission-col-date";
const TOTAL: &str = "print-buyer-dues-total";
/// The facts of the filters, shared with the other prints.
const FACT_ITEM: &str = "report-filter-item";

/// The print of a period: every day (or month) with a sale, and the period totals row.
///
/// # Errors
/// As [`super::sales_period`]; a database error.
pub fn sales_period_print_data(
    db: &Db,
    input: &SalesPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = sales_period_spec(db, input)?;
    table_print_data(db, input.language, input.paper, spec)
}

fn column(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

/// The party in the print language: his name with the tag, and his code unless he is a چکتا buyer.
fn party_text(shown: &CommissionPartyShown, language: Language) -> String {
    let party = &shown.party;
    let name = match language {
        Language::Ur => &party.name_ur,
        Language::En => &party.name_en,
    };
    let mut text = name.clone();
    if let Some(tag) = &party.tag {
        text.push(' ');
        text.push_str(tag);
    }
    if party.person_key.is_none() {
        text.push_str(&format!(" ({})", party.code));
    }
    text
}

/// The facts under the title: the period, and the party, the item and the goods when there are some.
fn facts_of(sales: &SalesPeriod, language: Language) -> Vec<PrintFact> {
    let fact = |id: &str, value: String| PrintFact {
        label_id: id.to_owned(),
        value,
    };
    let iso = |text: &str| {
        BusinessDate::parse_iso(text)
            .map(format_ddmmyyyy)
            .unwrap_or_default()
    };
    let (y, m) = sales.month.split_once('-').unwrap_or(("", ""));
    let mut facts = match sales.period.kind {
        CommissionPeriodKind::Month => vec![fact("print-fact-month", format!("{m}-{y}"))],
        CommissionPeriodKind::Day => vec![fact("print-fact-date", iso(&sales.period.from))],
        CommissionPeriodKind::Year => vec![fact("print-fact-year", sales.year.to_string())],
        CommissionPeriodKind::Range => vec![
            fact("print-fact-from", iso(&sales.period.from)),
            fact("print-fact-to", iso(&sales.period.to)),
        ],
    };
    if let Some(shown) = &sales.party {
        let label = match shown.side {
            CommissionSide::Seller => "print-fact-seller",
            CommissionSide::Buyer => "print-fact-buyer",
        };
        facts.push(fact(label, party_text(shown, language)));
    }
    if let Some(item) = &sales.item {
        let name = match language {
            Language::Ur => &item.name_ur,
            Language::En => &item.name_en,
        };
        facts.push(fact(FACT_ITEM, name.clone()));
    }
    // The goods say it all in their label, so their value is empty (as the analytics header).
    facts.extend(goods_fact(sales.goods));
    facts
}

/// The table of a period for print and Excel: the same cells the screen shows, in the order the
/// print and the sheet have them.
///
/// # Errors
/// As [`super::sales_period`]; a database error.
pub fn sales_period_spec(db: &Db, input: &SalesPrintInput) -> Result<TableSpec, ReportsError> {
    let sales = sales_every_row(
        db,
        &SalesPeriodInput {
            period: input.period.clone(),
            party: input.party.clone(),
            item_id: input.item_id,
            goods: input.goods,
            offset: None,
            limit: None,
        },
    )?;
    let by_month = sales.period.kind == CommissionPeriodKind::Year;
    let date_kind = if by_month {
        ColumnKind::Month
    } else {
        ColumnKind::Date
    };
    let columns = vec![
        column("date", COL_DATE, date_kind),
        column(
            "bikri_count",
            "print-cash-sales-col-bikri",
            ColumnKind::Number,
        ),
        column("qty", "print-col-qty", ColumnKind::Number),
        column("gross", "print-commission-col-sale", ColumnKind::Money),
        column("buyer_total", "report-col-buyer-total", ColumnKind::Money),
        column("cash", "print-cash-sales-col-cash", ColumnKind::Money),
        column("credit", "print-sale-col-credit", ColumnKind::Money),
    ];
    let cells = |row: &SalesPeriodRow, date: Cell| -> Vec<Cell> {
        vec![
            date,
            Cell::Number {
                value: row.bikri_count,
            },
            Cell::Number { value: row.qty },
            Cell::Money {
                paisa: row.gross_paisa,
            },
            Cell::Money {
                paisa: row.buyer_total_paisa,
            },
            Cell::Money {
                paisa: row.cash_paisa,
            },
            Cell::Money {
                paisa: row.credit_paisa,
            },
        ]
    };
    let date_cell = |key: &str| -> Cell {
        if by_month {
            let (year, mm) = key.split_once('-').unwrap_or(("", ""));
            Cell::Month {
                year: year.parse().unwrap_or_default(),
                month: mm.parse().unwrap_or_default(),
            }
        } else {
            Cell::Date {
                iso: key.to_owned(),
            }
        }
    };
    let rows: Vec<Row> = sales
        .rows
        .iter()
        .map(|r| Row {
            cells: cells(r, date_cell(&r.date)),
            source: None,
            kind: RowKind::Normal,
        })
        .collect();
    let count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    let title = if sales.period.kind == CommissionPeriodKind::Month {
        TITLE_MONTH
    } else {
        TITLE_PERIOD
    };
    Ok(TableSpec {
        title_id: title.to_owned(),
        facts: facts_of(&sales, input.language),
        column_names: vec![None; columns.len()],
        columns,
        totals: cells(&sales.totals, Cell::Empty),
        rows,
        totals_label_id: TOTAL.to_owned(),
        total_count: count,
        words_of: None,
        plain_sale: false,
        extra_totals: Vec::new(),
    })
}
