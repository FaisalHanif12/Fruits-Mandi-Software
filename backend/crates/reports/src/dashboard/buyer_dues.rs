//! The simple buyer dues page (plan 76 point 9, PRD 9.1 v1.41): the tile گاہکوں کے ذمے opens one
//! row for each buyer who owes: code, name, city, balance and the day his account last changed.
//! No age bands (R20 keeps them for the assistant).
//!
//! The rows are the very buyers and balances R20 uses (`buyers_balances_now`), so the total of the
//! whole filter equals the tile and R20 (PRD FR-19.9) by construction. People in the dead amount
//! and the چکتا کھاتہ are not in them. The total is over every row the search matches, never over
//! the page (Code Standards B31).
//!
//! Example (after Events 1 to 5): اکرم ٹول ٹیکس 2,880 (last change 10-10-26), ارشد ملتانی
//! 18,684 (10-10-26); total 21,564.

use std::cmp::Ordering;
use std::collections::HashMap;

use mandi_core::accounts::{Balance, BalanceSide};
use mandi_core::auth::Language;
use mandi_core::error::FieldError;
use mandi_core::money::Paisa;
use mandi_core::paging::{SortDirection, SortInput};
use mandi_core::printing::Paper;
use mandi_core::text::{UrduCollator, search_en, search_ur};
use mandi_db::Db;
use mandi_db::rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::ReportsError;
use crate::print::PrintFact;
use crate::reports::collections::dues::{Buyer, buyers_balances_now};
use crate::reports::common::{add, in_snapshot, today as today_of};
use crate::reports::{
    Cell, Column, ColumnKind, Names, PartyRef, Row, RowKind, TablePrintData, TableSpec,
    table_print_data,
};

/// The rows a page holds when no limit is sent.
const PAGE: usize = 100;
/// The most rows one page holds.
const MAX_PAGE: usize = 100;
/// The columns a sort may name.
const SORT_COLUMNS: [&str; 4] = ["party", "balance", "city", "last_change_date"];

/// Input of `dashboard_buyer_dues`.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct BuyerDuesInput {
    /// Part of the code, the running number or a name in either script. Null or empty lists every
    /// buyer who owes.
    #[serde(default)]
    pub search: Option<String>,
    /// One buyer. Null means سب (all buyers; plan 79 item 1). A name or a code typed in `search`
    /// narrows the same way.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub party_id: Option<i64>,
    /// The column to sort the whole filter by: `party`, `balance`, `city` or `last_change_date`.
    /// Null is by name, in the Urdu order.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 100. 0 gives 100.
    #[serde(default)]
    pub limit: u32,
}

/// One buyer who owes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BuyerDuesRow {
    /// The buyer.
    pub party: PartyRef,
    /// His city, or null.
    pub city: Option<Names>,
    /// What he owes, with its side (always بنام).
    pub balance: Balance,
    /// The day of his last posting, `YYYY-MM-DD`; null when he has none.
    pub last_change_date: Option<String>,
}

/// What `dashboard_buyer_dues` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BuyerDuesPage {
    /// The page.
    pub rows: Vec<BuyerDuesRow>,
    /// How many buyers match.
    #[ts(type = "number")]
    pub total_count: i64,
    /// What they owe together: the tile and the total of R20 when nothing is searched.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// The cities by id.
pub(super) fn cities(conn: &Connection) -> Result<HashMap<i64, Names>, ReportsError> {
    let mut statement = conn.prepare("SELECT id, name_ur, name_en FROM cities")?;
    let rows = statement
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                Names {
                    ur: r.get(1)?,
                    en: r.get(2)?,
                },
            ))
        })?
        .collect::<Result<HashMap<_, _>, _>>()?;
    Ok(rows)
}

/// The day of the last posted line of each buyer account, in one grouped read.
fn last_changes(conn: &Connection) -> Result<HashMap<i64, String>, ReportsError> {
    let mut statement = conn.prepare(
        "SELECT l.account_id, MAX(e.entry_date)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code = '0304'
         GROUP BY l.account_id",
    )?;
    let rows = statement
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<HashMap<_, _>, _>>()?;
    Ok(rows)
}

/// The day of the last posted line of some buyer accounts, one cheap read for each (a small account
/// by its lines, a big one by walking the days from the newest, see
/// `mandi_db::ledger::edge_day_of_account`). A page reads its own 100, not the books of every buyer
/// (NFR-SPD-24).
pub(super) fn last_changes_of(
    conn: &Connection,
    ids: &[i64],
) -> Result<HashMap<i64, String>, ReportsError> {
    let mut out = HashMap::new();
    for id in ids {
        let day =
            mandi_db::ledger::edge_day_of_account(conn, *id, mandi_db::ledger::DayEdge::Last)?;
        if let Some(day) = day {
            out.insert(*id, day);
        }
    }
    Ok(out)
}

/// True when the typed text is in the buyer's code, running number or either name.
pub(super) fn matches(party: &PartyRef, text: &str, text_ur: &str, text_en: &str) -> bool {
    party.code.contains(text)
        || party.short_code.as_deref() == Some(text)
        || search_ur(&party.name_ur).contains(text_ur)
        || search_en(&party.name_en).contains(text_en)
}

/// Whether the day of the last change is read for every buyer, or for the rows of one page only.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Changes {
    /// Every buyer: a sort by that day and the print need it for all of them.
    All,
    /// The rows of this page only, read in the same snapshot as the balances, so a row's day can
    /// never be newer than its balance. The rows returned are then the page, not every buyer.
    Page { offset: usize, limit: usize },
}

/// The buyers of a sheet: the rows asked for, how many matched, and the total of all of them.
struct Loaded {
    /// Every row, or the page rows for [`Changes::Page`].
    rows: Vec<BuyerDuesRow>,
    /// How many buyers matched.
    total_count: i64,
    /// What all the matching buyers owe.
    total: Paisa,
}

/// Every buyer who owes and matches, sorted, with the total of them all.
fn load_all(db: &Db, input: &BuyerDuesInput, changes_for: Changes) -> Result<Loaded, ReportsError> {
    if let Some(sort) = &input.sort {
        sort.check(&SORT_COLUMNS)
            .map_err(|e: FieldError| ReportsError::Rejected(vec![e]))?;
    }
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let typed = input.search.as_deref().map_or("", str::trim);
            let (text_ur, text_en) = (search_ur(typed), search_en(typed));
            let buyers: Vec<Buyer> = buyers_balances_now(conn, today)?
                .into_iter()
                .filter(|b| input.party_id.is_none_or(|id| b.party.account_id == id))
                .filter(|b| typed.is_empty() || matches(&b.party, typed, &text_ur, &text_en))
                .collect();
            let cities = cities(conn)?;
            let changes = match changes_for {
                Changes::All => last_changes(conn)?,
                Changes::Page { .. } => HashMap::new(),
            };
            let mut rows: Vec<BuyerDuesRow> = buyers
                .into_iter()
                .map(|b| BuyerDuesRow {
                    city: b.city_id.and_then(|id| cities.get(&id).cloned()),
                    last_change_date: changes.get(&b.party.account_id).cloned(),
                    balance: Balance {
                        amount_paisa: b.balance,
                        side: if b.balance == Paisa::ZERO {
                            BalanceSide::Zero
                        } else {
                            BalanceSide::Dr
                        },
                    },
                    party: b.party,
                })
                .collect();
            let mut total = Paisa::ZERO;
            for row in &rows {
                add(&mut total, row.balance.amount_paisa)?;
            }
            // By name in the Urdu order unless a column is asked for. The sort is stable.
            let collator = UrduCollator::new().ok();
            let by_name = |a: &BuyerDuesRow, b: &BuyerDuesRow| match &collator {
                Some(c) => c.compare(&a.party.name_ur, &b.party.name_ur),
                None => a.party.name_ur.cmp(&b.party.name_ur),
            };
            match &input.sort {
                None => rows.sort_by(by_name),
                Some(sort) => {
                    let ordering = |a: &BuyerDuesRow, b: &BuyerDuesRow| -> Ordering {
                        match sort.column.as_str() {
                            "balance" => a.balance.amount_paisa.cmp(&b.balance.amount_paisa),
                            "city" => a
                                .city
                                .as_ref()
                                .map(|n| n.ur.as_str())
                                .cmp(&b.city.as_ref().map(|n| n.ur.as_str())),
                            "last_change_date" => a.last_change_date.cmp(&b.last_change_date),
                            _ => by_name(a, b),
                        }
                    };
                    let direction: SortDirection = sort.direction;
                    rows.sort_by(|a, b| direction.apply(ordering(a, b)));
                }
            }
            let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
            if let Changes::Page { offset, limit } = changes_for {
                rows = rows.into_iter().skip(offset).take(limit).collect();
                let ids: Vec<i64> = rows.iter().map(|r| r.party.account_id).collect();
                let changes = last_changes_of(conn, &ids)?;
                for row in &mut rows {
                    row.last_change_date = changes.get(&row.party.account_id).cloned();
                }
            }
            Ok(Loaded {
                rows,
                total_count,
                total,
            })
        })
    })
}

/// The buyers who owe, a page of them, and the total of all the matching ones.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `sort.column`; a database or money error.
pub fn buyer_dues(db: &Db, input: &BuyerDuesInput) -> Result<BuyerDuesPage, ReportsError> {
    // A sort by the day of the last change needs it for every buyer; any other order reads it for
    // the page only, in the same snapshot as the balances.
    let by_change = input
        .sort
        .as_ref()
        .is_some_and(|s| s.column == "last_change_date");
    let limit = match input.limit as usize {
        0 => PAGE,
        n => n.min(MAX_PAGE),
    };
    let offset = input.offset as usize;
    let Loaded {
        rows,
        total_count,
        total,
    } = load_all(
        db,
        input,
        if by_change {
            Changes::All
        } else {
            Changes::Page { offset, limit }
        },
    )?;
    let page: Vec<BuyerDuesRow> = if by_change {
        rows.into_iter().skip(offset).take(limit).collect()
    } else {
        rows
    };
    Ok(BuyerDuesPage {
        rows: page,
        total_count,
        total_paisa: total,
    })
}

/// Input of `dashboard_buyer_dues_print_data`: the filter of the screen and the print language.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct BuyerDuesPrintInput {
    /// The search of the screen.
    #[serde(default)]
    pub search: Option<String>,
    /// One buyer, or null for all (plan 79 item 1).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub party_id: Option<i64>,
    /// The sort of the screen.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}

/// The print of the buyer dues: every buyer of the filter, with the total in the last row.
///
/// # Errors
/// As [`buyer_dues`]; a stored amount that cannot be written in words.
pub fn buyer_dues_print_data(
    db: &Db,
    input: &BuyerDuesPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = buyer_dues_spec(db, input)?;
    table_print_data(db, input.language, input.paper, spec)
}

/// The table of the buyer dues for print and Excel (plan 76 point 9).
///
/// # Errors
/// As [`buyer_dues`].
pub fn buyer_dues_spec(db: &Db, input: &BuyerDuesPrintInput) -> Result<TableSpec, ReportsError> {
    let Loaded { rows, total, .. } = load_all(
        db,
        &BuyerDuesInput {
            search: input.search.clone(),
            party_id: input.party_id,
            sort: input.sort.clone(),
            offset: 0,
            limit: 0,
        },
        // The print has the day of the last change of every buyer.
        Changes::All,
    )?;
    let column = |key: &str, title: &str, kind: ColumnKind| Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    };
    let columns = vec![
        column("party", "print-buyer-dues-col-party", ColumnKind::Party),
        column("city", "print-buyer-dues-col-city", ColumnKind::Names),
        column(
            "balance",
            "print-buyer-dues-col-balance",
            ColumnKind::Balance,
        ),
        column(
            "last_change_date",
            "print-buyer-dues-col-last",
            ColumnKind::Date,
        ),
    ];
    let count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    let table_rows = rows
        .into_iter()
        .map(|r| Row {
            cells: vec![
                Cell::Party { party: r.party },
                r.city.map_or(Cell::Empty, |names| Cell::Names { names }),
                Cell::Balance {
                    paisa: r.balance.amount_paisa,
                    side: r.balance.side,
                },
                r.last_change_date
                    .map_or(Cell::Empty, |iso| Cell::Date { iso }),
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
    Ok(TableSpec {
        title_id: "print-buyer-dues-title".to_owned(),
        facts,
        column_names: vec![None; columns.len()],
        columns,
        rows: table_rows,
        totals_label_id: "print-buyer-dues-total".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Balance {
                paisa: total,
                side: if total == Paisa::ZERO {
                    BalanceSide::Zero
                } else {
                    BalanceSide::Dr
                },
            },
            Cell::Empty,
        ],
        total_count: count,
        words_of: Some(total).filter(|t| *t != Paisa::ZERO),
        plain_sale: false,
        extra_totals: Vec::new(),
    })
}
