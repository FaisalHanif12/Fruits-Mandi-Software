//! The statement print with a row for each entry (plan 78 A7, PRD FR-12.47, FR-19.7, FR-19.13).
//!
//! The rows are those of the statement screen (`statement_rows`), so the print cannot differ from
//! it. A table: date, entry, one line of detail, بنام, جمع, بقایا. The balance brought forward (when
//! the period starts after the first entry) is the first row, the totals and the closing balance are
//! the last, with the closing amount in words. About 25 rows go on one A4 page.
//!
//! The detail is one line: the items and their quantities. With `items` on, each row has its items in
//! full instead (a buyer's sale lines, a seller's lots), one short line for each item. An entry with
//! no items (a payment, an opening) shows its reason or narration as typed.
//!
//! Example (plan 78 A7, احسان, with the figures of s10): the brought forward row is left out because the period starts at the
//! first entry; ابتدائی بقایا 20,000 جمع (20,000 جمع); بکری 1, آم-1 200 (160,280 جمع, 180,280 جمع);
//! ادائیگی 1, 100,000 بنام (80,280 جمع); totals 100,000, 180,280; closing 80,280 جمع and its words.

use mandi_core::accounts::BalanceSide;
use mandi_core::auth::Language;
use mandi_core::printing::Paper;
use mandi_db::Db;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::ids;
use super::model::{Cell, Column, ColumnKind, PartyRef, Row, RowKind};
use super::print_data::{Fmt, PrintOptions, fact};
use super::statement_rows::{StatementRowsInput, StatementRowsRow, whole};
use super::table_print::{TablePrintData, TableSpec, table_print_data};
use crate::error::ReportsError;
use crate::print::load::load_grouping;

/// Fluent message IDs of the print (the words are in the Fluent files of the frontend).
pub const TITLE_ID: &str = "print-statement-rows-title";
const TOTAL_ID: &str = "print-statement-rows-total";
const BROUGHT_FORWARD_ID: &str = "print-statement-rows-brought-forward";
const COL_DATE: &str = "print-statement-rows-col-date";
const COL_ENTRY: &str = "print-statement-rows-col-entry";
const COL_DETAIL: &str = "print-statement-rows-col-detail";
const COL_DEBIT: &str = "print-statement-rows-col-debit";
const COL_CREDIT: &str = "print-statement-rows-col-credit";
const COL_BALANCE: &str = "print-statement-rows-col-balance";

/// Input of `party_ledgers_statement_rows_print_data`.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct StatementRowsPrintInput {
    /// The party: a buyer or a seller.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The first day, `DD-MM-YYYY` or a shortcut. Missing: from the first entry.
    #[serde(default)]
    #[ts(optional)]
    pub from: Option<String>,
    /// The last day. Missing: up to now.
    #[serde(default)]
    #[ts(optional)]
    pub to: Option<String>,
    /// True prints each row's items in full, one short line each. Off by default (one line of items).
    #[serde(default)]
    pub items: bool,
    /// An entry that an edit or a cancel replaced is left out, as on the screen. True prints every
    /// entry, the replaced ones too.
    #[serde(default)]
    pub include_history: bool,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    #[ts(optional)]
    pub paper: Option<Paper>,
}

/// The items of a row in one line: "آم-1 200، سیب-1 10".
fn short_line(fmt: &Fmt, row: &StatementRowsRow, separator: &str) -> String {
    row.short
        .iter()
        .map(|s| format!("{} {}", fmt.names(&s.item), fmt.style.count(s.qty)))
        .collect::<Vec<_>>()
        .join(separator)
}

/// The table of the statement for the print and Excel.
///
/// # Errors
/// As `statement_rows`; a stored amount that cannot be written in words.
pub fn statement_rows_spec(
    db: &Db,
    input: &StatementRowsPrintInput,
) -> Result<TableSpec, ReportsError> {
    let data = whole(
        db,
        &StatementRowsInput {
            account_id: input.account_id,
            from: input.from.clone(),
            to: input.to.clone(),
            include_history: input.include_history,
            newest_first: false,
            offset: 0,
            limit: 0,
        },
    )?;
    let grouping = db.read(load_grouping)?;
    let fmt = Fmt::new(&PrintOptions {
        language: input.language,
        grouping,
        paper: input.paper.unwrap_or(Paper::A4),
    });
    let separator = match input.language {
        Language::Ur => "، ",
        Language::En => ", ",
    };
    let column = |key: &str, title: &str, kind: ColumnKind| Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    };
    let columns = vec![
        column("date", COL_DATE, ColumnKind::Date),
        column("entry", COL_ENTRY, ColumnKind::Doc),
        column("detail", COL_DETAIL, ColumnKind::Detail),
        column("debit", COL_DEBIT, ColumnKind::Money),
        column("credit", COL_CREDIT, ColumnKind::Money),
        column("balance", COL_BALANCE, ColumnKind::Balance),
    ];
    let balance_cell = |b: &mandi_core::accounts::Balance| Cell::Balance {
        paisa: b.amount_paisa,
        side: b.side,
    };
    let amount = |p: mandi_core::money::Paisa| {
        if p == mandi_core::money::Paisa::ZERO {
            Cell::Empty
        } else {
            Cell::Money { paisa: p }
        }
    };

    let mut rows = Vec::with_capacity(data.rows.len().saturating_add(1));
    // What stood before the first day is the first row.
    if let (Some(from), true) = (
        data.from.as_ref(),
        data.brought_forward.side != BalanceSide::Zero,
    ) {
        rows.push(Row {
            cells: vec![
                Cell::Date { iso: from.clone() },
                Cell::Label {
                    id: BROUGHT_FORWARD_ID.to_owned(),
                },
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                balance_cell(&data.brought_forward),
            ],
            source: None,
            kind: RowKind::Normal,
        });
    }
    for row in &data.rows {
        let detail = if input.items && !row.parts.is_empty() {
            Cell::Detail {
                parts: row.parts.clone(),
            }
        } else if !row.short.is_empty() {
            Cell::Text {
                text: short_line(&fmt, row, separator),
            }
        } else if !row.parts.is_empty() {
            Cell::Detail {
                parts: row.parts.clone(),
            }
        } else if let Some(text) = &row.narration {
            Cell::Text { text: text.clone() }
        } else {
            Cell::Empty
        };
        rows.push(Row {
            cells: vec![
                Cell::Date {
                    iso: row.date.clone(),
                },
                Cell::Doc {
                    type_id: row.type_id.clone(),
                    number: row.number,
                },
                detail,
                amount(row.dr_paisa),
                amount(row.cr_paisa),
                balance_cell(&row.balance),
            ],
            source: Some(row.source),
            kind: RowKind::Normal,
        });
    }
    let total_count = i64::try_from(data.rows.len()).unwrap_or(i64::MAX);

    let party = PartyRef {
        phone: None,
        dead: None,
        account_id: data.head.account_id,
        short_code: mandi_core::accounts::short_code_of(&data.head.code),
        code: data.head.code.clone(),
        name_ur: data.head.name_ur.clone(),
        name_en: data.head.name_en.clone(),
        tag: data.head.tag.clone(),
        person_key: None,
    };
    let mut facts = Vec::new();
    if let Some(from) = &data.from {
        facts.push(fact(ids::FILTER_FROM, fmt.style.date(from)?));
    }
    if let Some(to) = &data.to {
        facts.push(fact(ids::FILTER_TO, fmt.style.date(to)?));
    }
    facts.push(fact(ids::FILTER_PARTY, fmt.party_text(&party)));
    Ok(TableSpec {
        title_id: TITLE_ID.to_owned(),
        facts,
        column_names: vec![None; columns.len()],
        columns,
        rows,
        totals_label_id: TOTAL_ID.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: data.dr_total,
            },
            Cell::Money {
                paisa: data.cr_total,
            },
            balance_cell(&data.closing),
        ],
        total_count,
        words_of: Some(data.closing.amount_paisa).filter(|a| *a != mandi_core::money::Paisa::ZERO),
        plain_sale: false,
        extra_totals: Vec::new(),
    })
}

/// The print of one party's statement, a row for each entry, with every row of the period.
///
/// # Errors
/// As [`statement_rows_spec`].
pub fn statement_rows_print_data(
    db: &Db,
    input: &StatementRowsPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = statement_rows_spec(db, input)?;
    table_print_data(db, input.language, input.paper, spec)
}
