//! The total dues page (plan 79 item 2b, PRD FR-9.7) and the page of money lent (PRD 9.1, FR-9.1):
//! the tile کل اگراہی is four parts, buyers, sellers, the dead amount and the people who borrowed
//! (ادھار رقم), and the page shows each part with its total and the grand total.
//!
//! The totals are the figures the tile is made of (`tiles::build`), so the grand total equals the
//! tile by construction. The rows of each section come from the page of that section:
//! buyers `dashboard_buyer_dues`, sellers `dashboard_seller_dues` with `seller_owes`, the dead
//! amount `reports_dead_amount`, the people who borrowed `dashboard_lent_open`.
//!
//! Example (after Events 1 to 5): buyers 21,564 (2), sellers 0, dead amount 0, others 0; grand
//! total 21,564.

use std::cmp::Ordering;

use mandi_core::auth::Language;
use mandi_core::error::FieldError;
use mandi_core::links::PersonType;
use mandi_core::money::Paisa;
use mandi_core::paging::{SortDirection, SortInput};
use mandi_core::printing::Paper;
use mandi_core::text::{UrduCollator, search_en, search_ur};
use mandi_db::Db;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::buyer_dues::matches;
use super::lent;
use crate::error::ReportsError;
use crate::print::PrintFact;
use crate::reports::collections::dues::buyers_balances_now;
use crate::reports::collections::payables::candidates;
use crate::reports::common::{add, in_snapshot, load_parties, today as today_of};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::{
    Cell, Column, ColumnKind, PartyRef, Row, RowKind, TablePrintData, TableSpec, table_print_data,
};

/// The rows a page holds when no limit is sent.
const PAGE: usize = 100;
/// The most rows one page holds.
const MAX_PAGE: usize = 100;
/// The columns a sort may name.
const SORT_COLUMNS: [&str; 3] = ["party", "open", "oldest_open_date"];

/// The four parts of کل اگراہی (PRD 9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TotalDuesSection {
    /// What buyers owe.
    Buyers,
    /// What sellers owe beyond their advances.
    Sellers,
    /// The dead amount.
    Dead,
    /// Open money lent (ادھار رقم): staff, garden owners and others, leaving out the dead amount.
    Others,
}

/// One part with its total.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TotalDuesPart {
    /// Which part.
    pub section: TotalDuesSection,
    /// How many people are in it.
    #[ts(type = "number")]
    pub count: i64,
    /// What they owe the shop.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// What `dashboard_total_dues` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TotalDues {
    /// The four parts, always in the order buyers, sellers, dead, others.
    pub parts: Vec<TotalDuesPart>,
    /// The four added: the tile کل اگراہی.
    #[ts(type = "Paisa")]
    pub grand_total_paisa: Paisa,
}

/// Input of `dashboard_lent_open`.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct LentOpenInput {
    /// Part of the code or a name in either script. Null or empty lists every person.
    #[serde(default)]
    pub search: Option<String>,
    /// The column to sort the whole filter by: `party`, `open` or `oldest_open_date`. Null is the
    /// most open first.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 100. 0 gives 100.
    #[serde(default)]
    pub limit: u32,
}

/// One person who borrowed and has something open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LentOpenRow {
    /// The person (his ادھار رقم account). His loans open with `lent_list`.
    pub party: PartyRef,
    /// His type (staff, garden owner, trip person, other).
    pub person_type: PersonType,
    /// What he still owes: the open amount of his loans (PRD FR-16.5).
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// The day of his oldest open loan, `YYYY-MM-DD`; null if none.
    pub oldest_open_date: Option<String>,
    /// How much of it is past its return-by day.
    #[ts(type = "Paisa")]
    pub overdue_paisa: Paisa,
}

/// What `dashboard_lent_open` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LentOpenPage {
    /// The page.
    pub rows: Vec<LentOpenRow>,
    /// How many people match.
    #[ts(type = "number")]
    pub total_count: i64,
    /// What they owe together. With no search it is the tile ادھار رقم باقی.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
    /// How much of it is past its return-by day.
    #[ts(type = "Paisa")]
    pub overdue_paisa: Paisa,
}

/// Input of `dashboard_total_dues_print_data`.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct TotalDuesPrintInput {
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}

/// Input of `dashboard_lent_open_print_data`: the filter of the screen and the print language.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct LentOpenPrintInput {
    /// The search of the screen.
    #[serde(default)]
    pub search: Option<String>,
    /// The sort of the screen.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}

fn count_of(len: usize) -> i64 {
    i64::try_from(len).unwrap_or(i64::MAX)
}

/// The four parts of کل اگراہی and their sum, read in one moment.
///
/// # Errors
/// A database or money error.
pub fn total_dues(db: &Db) -> Result<TotalDues, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let today_iso = today.to_iso();
            let (mut buyers, mut sellers, mut owing) = (Paisa::ZERO, Paisa::ZERO, 0_usize);
            let balances = buyers_balances_now(conn, today)?;
            for buyer in &balances {
                add(&mut buyers, buyer.balance)?;
            }
            let typed = Typed {
                today: Some(today),
                ..Typed::default()
            };
            let (list, _) = candidates(conn, &typed, &today_iso, true)?;
            for seller in &list {
                if let Some(w) = seller.working
                    && w.owes_firm > Paisa::ZERO
                {
                    owing = owing.checked_add(1).ok_or_else(|| {
                        ReportsError::Data("the seller count does not fit".to_owned())
                    })?;
                    add(&mut sellers, w.owes_firm)?;
                }
            }
            let (dead, dead_count) = crate::dead::tile(conn)?;
            let facts = lent::facts(conn, today)?;
            let mut others = Paisa::ZERO;
            for person in &facts.people {
                add(&mut others, person.open)?;
            }
            let mut grand = buyers;
            add(&mut grand, sellers)?;
            add(&mut grand, dead)?;
            add(&mut grand, others)?;
            Ok(TotalDues {
                parts: vec![
                    TotalDuesPart {
                        section: TotalDuesSection::Buyers,
                        count: count_of(balances.len()),
                        total_paisa: buyers,
                    },
                    TotalDuesPart {
                        section: TotalDuesSection::Sellers,
                        count: count_of(owing),
                        total_paisa: sellers,
                    },
                    TotalDuesPart {
                        section: TotalDuesSection::Dead,
                        count: dead_count,
                        total_paisa: dead,
                    },
                    TotalDuesPart {
                        section: TotalDuesSection::Others,
                        count: count_of(facts.people.len()),
                        total_paisa: others,
                    },
                ],
                grand_total_paisa: grand,
            })
        })
    })
}

/// Every person who borrowed and still owes and matches, sorted, with the totals over all of them.
fn load_lent(
    db: &Db,
    search: Option<&str>,
    sort: Option<&SortInput>,
) -> Result<(Vec<LentOpenRow>, Paisa, Paisa), ReportsError> {
    if let Some(sort) = sort {
        sort.check(&SORT_COLUMNS)
            .map_err(|e: FieldError| ReportsError::Rejected(vec![e]))?;
    }
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let facts = lent::facts(conn, today)?;
            let ids: Vec<i64> = facts.people.iter().map(|p| p.account_id).collect();
            let parties = load_parties(conn, &ids)?;
            let text = search.map_or("", str::trim);
            let (text_ur, text_en) = (search_ur(text), search_en(text));
            let mut rows: Vec<LentOpenRow> = Vec::new();
            for person in facts.people {
                let Some(party) = parties.get(&person.account_id).cloned() else {
                    return Err(ReportsError::Data(format!(
                        "person account {} is missing",
                        person.account_id
                    )));
                };
                if !text.is_empty() && !matches(&party, text, &text_ur, &text_en) {
                    continue;
                }
                rows.push(LentOpenRow {
                    party,
                    person_type: PersonType::parse(&person.person_type)
                        .unwrap_or(PersonType::Other),
                    open_paisa: person.open,
                    oldest_open_date: person.oldest_open_date,
                    overdue_paisa: person.overdue,
                });
            }
            let (mut total, mut overdue) = (Paisa::ZERO, Paisa::ZERO);
            for row in &rows {
                add(&mut total, row.open_paisa)?;
                add(&mut overdue, row.overdue_paisa)?;
            }
            let collator = UrduCollator::new().ok();
            let by_name = |a: &LentOpenRow, b: &LentOpenRow| match &collator {
                Some(c) => c.compare(&a.party.name_ur, &b.party.name_ur),
                None => a.party.name_ur.cmp(&b.party.name_ur),
            };
            match sort {
                // The most open first, then by name.
                None => {
                    rows.sort_by(|a, b| b.open_paisa.cmp(&a.open_paisa).then_with(|| by_name(a, b)))
                }
                Some(sort) => {
                    let ordering = |a: &LentOpenRow, b: &LentOpenRow| -> Ordering {
                        match sort.column.as_str() {
                            "open" => a.open_paisa.cmp(&b.open_paisa),
                            "oldest_open_date" => a.oldest_open_date.cmp(&b.oldest_open_date),
                            _ => by_name(a, b),
                        }
                    };
                    let direction: SortDirection = sort.direction;
                    rows.sort_by(|a, b| direction.apply(ordering(a, b)));
                }
            }
            Ok((rows, total, overdue))
        })
    })
}

/// The people who borrowed and still owe, a page of them, with the total of all that match.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `sort.column`; a database or money error.
pub fn lent_open(db: &Db, input: &LentOpenInput) -> Result<LentOpenPage, ReportsError> {
    let (rows, total, overdue) = load_lent(db, input.search.as_deref(), input.sort.as_ref())?;
    let total_count = count_of(rows.len());
    let limit = match input.limit as usize {
        0 => PAGE,
        n => n.min(MAX_PAGE),
    };
    let rows = rows
        .into_iter()
        .skip(input.offset as usize)
        .take(limit)
        .collect();
    Ok(LentOpenPage {
        rows,
        total_count,
        total_paisa: total,
        overdue_paisa: overdue,
    })
}

fn column(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

fn money_cell(p: Paisa) -> Cell {
    if p == Paisa::ZERO {
        Cell::Empty
    } else {
        Cell::Money { paisa: p }
    }
}

/// The print of the four parts of کل اگراہی and the grand total (PRD FR-9.7, FR-19.10).
///
/// # Errors
/// A database or money error; a stored amount that cannot be written in words.
pub fn total_dues_spec(db: &Db, _input: &TotalDuesPrintInput) -> Result<TableSpec, ReportsError> {
    let total = total_dues(db)?;
    let columns = vec![
        column("section", "print-total-dues-col-section", ColumnKind::Label),
        column("count", "print-total-dues-col-count", ColumnKind::Number),
        column("total", "print-total-dues-col-total", ColumnKind::Money),
    ];
    let rows = total
        .parts
        .iter()
        .map(|part| Row {
            cells: vec![
                Cell::Label {
                    id: match part.section {
                        TotalDuesSection::Buyers => "print-total-dues-buyers",
                        TotalDuesSection::Sellers => "print-total-dues-sellers",
                        TotalDuesSection::Dead => "print-total-dues-dead",
                        TotalDuesSection::Others => "print-total-dues-others",
                    }
                    .to_owned(),
                },
                Cell::Number { value: part.count },
                Cell::Money {
                    paisa: part.total_paisa,
                },
            ],
            source: None,
            kind: RowKind::Normal,
        })
        .collect::<Vec<_>>();
    let spec = TableSpec {
        title_id: "print-total-dues-title".to_owned(),
        facts: Vec::new(),
        column_names: vec![None; columns.len()],
        columns,
        total_count: count_of(rows.len()),
        rows,
        totals_label_id: "print-buyer-dues-total".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: total.grand_total_paisa,
            },
        ],
        words_of: Some(total.grand_total_paisa).filter(|t| *t != Paisa::ZERO),
        plain_sale: false,
        extra_totals: Vec::new(),
    };
    Ok(spec)
}

/// The print of the same table, in the print language.
///
/// # Errors
/// As [`total_dues_spec`].
pub fn total_dues_print_data(
    db: &Db,
    input: &TotalDuesPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = total_dues_spec(db, input)?;
    table_print_data(db, input.language, input.paper, spec)
}

/// The print of the people who borrowed and still owe: every person of the filter, with the total.
///
/// # Errors
/// As [`lent_open`]; a stored amount that cannot be written in words.
pub fn lent_open_spec(db: &Db, input: &LentOpenPrintInput) -> Result<TableSpec, ReportsError> {
    let (rows, total, overdue) = load_lent(db, input.search.as_deref(), input.sort.as_ref())?;
    let columns = vec![
        column("person", ids::COL_PERSON, ColumnKind::Party),
        column("type", "print-lent-open-col-type", ColumnKind::Label),
        column("open", ids::COL_OPEN, ColumnKind::Money),
        column("oldest_open_date", ids::COL_OLDEST_DATE, ColumnKind::Date),
        column("overdue", "print-lent-open-col-overdue", ColumnKind::Money),
    ];
    let count = count_of(rows.len());
    let table_rows = rows
        .into_iter()
        .map(|r| Row {
            cells: vec![
                Cell::Party { party: r.party },
                Cell::Label {
                    id: format!(
                        "cashbook-person-type-{}",
                        r.person_type.as_str().replace('_', "-")
                    ),
                },
                Cell::Money {
                    paisa: r.open_paisa,
                },
                r.oldest_open_date
                    .map_or(Cell::Empty, |iso| Cell::Date { iso }),
                money_cell(r.overdue_paisa),
            ],
            source: None,
            kind: RowKind::Normal,
        })
        .collect();
    let facts = match input
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(text) => vec![PrintFact {
            label_id: "print-fact-search".to_owned(),
            value: text.to_owned(),
        }],
        None => Vec::new(),
    };
    let spec = TableSpec {
        title_id: "print-lent-open-title".to_owned(),
        facts,
        column_names: vec![None; columns.len()],
        columns,
        rows: table_rows,
        totals_label_id: "print-buyer-dues-total".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: total },
            Cell::Empty,
            money_cell(overdue),
        ],
        total_count: count,
        words_of: Some(total).filter(|t| *t != Paisa::ZERO),
        plain_sale: false,
        extra_totals: Vec::new(),
    };
    Ok(spec)
}

/// The print of the same table, in the print language.
///
/// # Errors
/// As [`lent_open_spec`].
pub fn lent_open_print_data(
    db: &Db,
    input: &LentOpenPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = lent_open_spec(db, input)?;
    table_print_data(db, input.language, input.paper, spec)
}
