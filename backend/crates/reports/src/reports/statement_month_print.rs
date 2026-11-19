//! The print of a partner's statement for one month (plan 76 point 11, in the style of the party
//! statement print of plan 74): the opening کل رقم, one row for each day with a movement, the
//! month's totals with the closing, and the closing in words in the print language (PRD FR-19.7).
//! Every figure is the one `partners_statement_month` gave; the print writes it in the print
//! language and adds nothing up.

use mandi_core::auth::Language;
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use mandi_core::statement::{PartnerStatementMonth, StatementColumns};
use mandi_db::Db;

use super::model::{Cell, Column, ColumnKind, Names, Row, RowKind};
use super::table_print::{TablePrintData, TableSpec, table_print_data};
use crate::error::ReportsError;
use crate::print::PrintFact;

/// Writes a month of a partner's statement for print.
///
/// # Errors
/// A database error; a stored amount that cannot be written in words.
pub fn partner_statement_month_print_data(
    db: &Db,
    month: &PartnerStatementMonth,
    language: Language,
    paper: Option<Paper>,
) -> Result<TablePrintData, ReportsError> {
    let spec = partner_statement_month_spec(month, language);
    table_print_data(db, language, paper, spec)
}

/// The table of a month of a partner's statement for print and Excel (plan 76 point 11).
///
pub fn partner_statement_month_spec(
    month: &PartnerStatementMonth,
    language: Language,
) -> TableSpec {
    let column = |key: &str, kind: ColumnKind| Column {
        key: key.to_owned(),
        title_id: format!("print-partner-month-col-{}", key.replace('_', "-")),
        kind,
    };
    let columns = vec![
        column("date", ColumnKind::Date),
        column("commission", ColumnKind::Money),
        column("expense", ColumnKind::Money),
        column("gardens_trips", ColumnKind::Money),
        column("fruit", ColumnKind::Money),
        column("taken_returned", ColumnKind::Money),
        column("capital", ColumnKind::Money),
        column("advances", ColumnKind::Money),
        column("net", ColumnKind::Money),
        column("running", ColumnKind::Money),
    ];
    let money = |paisa: Paisa| Cell::Money { paisa };
    let figures = |c: &StatementColumns, lead: Cell, running: Paisa| -> Vec<Cell> {
        vec![
            lead,
            money(c.commission_paisa),
            money(c.expense_paisa),
            money(c.gardens_trips_paisa),
            money(c.fruit_paisa),
            money(c.taken_returned_paisa),
            money(c.capital_paisa),
            money(c.advances_paisa),
            money(c.net_paisa),
            money(running),
        ]
    };
    let mut rows = vec![Row {
        cells: vec![
            Cell::Label {
                id: "print-partner-month-opening".to_owned(),
            },
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            money(month.opening_total_paisa),
        ],
        source: None,
        kind: RowKind::Group,
    }];
    for day in &month.days {
        rows.push(Row {
            cells: figures(
                &day.columns,
                Cell::Date {
                    iso: day.date.clone(),
                },
                day.running_paisa,
            ),
            source: None,
            kind: RowKind::Normal,
        });
    }
    let count = i64::try_from(month.days.len()).unwrap_or(i64::MAX);
    let (y, m) = month.month.split_once('-').unwrap_or(("", ""));
    let names = Names {
        ur: month.partner.name_ur.clone(),
        en: month.partner.name_en.clone(),
    };
    let partner_text = match language {
        Language::Ur => names.ur,
        Language::En => names.en,
    };
    let closing = month.closing_total_paisa;
    // The amount in words is of the size of the closing; its side is on the page, not in the words.
    let words = if closing.is_negative() {
        closing.checked_neg().ok()
    } else {
        Some(closing)
    }
    .filter(|a| *a != Paisa::ZERO);
    TableSpec {
        title_id: "print-partner-month-title".to_owned(),
        facts: vec![
            PrintFact {
                label_id: "print-fact-partner".to_owned(),
                value: partner_text,
            },
            PrintFact {
                label_id: "print-fact-month".to_owned(),
                value: format!("{m}-{y}"),
            },
        ],
        column_names: vec![None; columns.len()],
        columns,
        rows,
        totals_label_id: "print-partner-month-total".to_owned(),
        totals: figures(&month.totals, Cell::Empty, closing),
        total_count: count,
        words_of: words,
        plain_sale: false,
        extra_totals: Vec::new(),
    }
}
