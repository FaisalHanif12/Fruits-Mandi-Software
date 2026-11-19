//! The seller dues page (plan 79 item 2a, PRD FR-9.6): the tiles بیوپاریوں کو ادائیگی باقی and
//! بیوپاریوں کے ذمے open one list of sellers, like the buyer dues page (`buyer_dues`).
//!
//! The rows are the very sellers and workings of R24 (`candidates`), so each side's total equals
//! its tile and R24 by construction (PRD FR-15.9, FR-7.59). A seller's history opens with
//! `party_ledgers_statement_rows` (opening balance first, one row for each بکری).
//!
//! Example (plan 79 s01): فیصل F 72,740 جمع, احسان 120,780 جمع, عدنان 9,260 جمع. The shop owes
//! sellers 202,780 (the tile); sellers owe the shop 0.

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
use mandi_db::ledger::{DayEdge, edge_day_of_account};
use mandi_db::rusqlite::{Connection, params_from_iter};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::buyer_dues::{cities, matches};
use crate::error::ReportsError;
use crate::print::PrintFact;
use crate::reports::collections::payables::{Seller, candidates};
use crate::reports::common::{add, in_snapshot, placeholders, today as today_of};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::{
    Cell, Column, ColumnKind, Names, PartyRef, Row, RowKind, TablePrintData, TableSpec,
    table_print_data,
};

/// The rows a page holds when no limit is sent.
const PAGE: usize = 100;
/// The most rows one page holds.
const MAX_PAGE: usize = 100;
/// The columns a sort may name.
const SORT_COLUMNS: [&str; 7] = [
    "party",
    "payable",
    "advance",
    "owes",
    "balance",
    "city",
    "last_change_date",
];

/// Which sellers a page lists (PRD FR-9.6).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SellerDuesSide {
    /// سب: every seller with a balance, an advance or something payable. The default.
    #[default]
    All,
    /// The shop owes the seller: his open payable (PRD FR-7.59). The tile بیوپاریوں کو ادائیگی باقی.
    ShopOwes,
    /// The seller owes the shop beyond his open advances (PRD FR-15.9). The tile بیوپاریوں کے ذمے.
    SellerOwes,
}

/// Input of `dashboard_seller_dues`.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct SellerDuesInput {
    /// Which sellers. Missing means `all`.
    #[serde(default)]
    pub side: SellerDuesSide,
    /// One seller. Null means سب (all sellers).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub party_id: Option<i64>,
    /// Part of the code, the running number or a name in either script. Null or empty lists all.
    #[serde(default)]
    pub search: Option<String>,
    /// The column to sort the whole filter by: `party`, `payable`, `advance`, `owes`, `balance`,
    /// `city` or `last_change_date`. Null is by name, in the Urdu order.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 100. 0 gives 100.
    #[serde(default)]
    pub limit: u32,
}

/// One seller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SellerDuesRow {
    /// The seller.
    pub party: PartyRef,
    /// His city, or null.
    pub city: Option<Names>,
    /// His ledger balance with its side: جمع when the shop owes him, بنام when he owes the shop or
    /// holds an open advance. The balance of his party card and of R13.
    pub balance: Balance,
    /// What is payable to him (PRD FR-7.59). 0 when nothing.
    #[ts(type = "Paisa")]
    pub payable_paisa: Paisa,
    /// His open advances (PRD FR-15.8).
    #[ts(type = "Paisa")]
    pub advance_open_paisa: Paisa,
    /// What he owes the shop beyond his advances (PRD FR-15.9).
    #[ts(type = "Paisa")]
    pub owes_paisa: Paisa,
    /// The day of his last posting, `YYYY-MM-DD`; null when he has none.
    pub last_change_date: Option<String>,
}

/// What `dashboard_seller_dues` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SellerDuesPage {
    /// The page.
    pub rows: Vec<SellerDuesRow>,
    /// How many sellers the filter (side, party, search) matches.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The shop owes these sellers: Σ payable of the sellers the party and the search match, whatever
    /// the side. With no party and no search it is the tile بیوپاریوں کو ادائیگی باقی.
    #[ts(type = "Paisa")]
    pub shop_owes_paisa: Paisa,
    /// Sellers owe the shop: Σ of what they owe beyond their advances. With no party and no search
    /// it is the tile بیوپاریوں کے ذمے.
    #[ts(type = "Paisa")]
    pub seller_owes_paisa: Paisa,
    /// Σ of their open advances (the tile پیشگی باقی).
    #[ts(type = "Paisa")]
    pub advances_open_paisa: Paisa,
    /// The ledger balances of the rows of the whole filter added up, with the side of the result.
    pub total: Balance,
}

/// Input of `dashboard_seller_dues_print_data`: the filter of the screen and the print language.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct SellerDuesPrintInput {
    /// Which sellers.
    #[serde(default)]
    pub side: SellerDuesSide,
    /// One seller, or null for all.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub party_id: Option<i64>,
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

/// Whether the day of the last change is read for every seller, or for the rows of one page only.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Changes {
    /// Every seller: a sort by that day and the print need it for all of them.
    All,
    /// The rows of this page only, read in the same snapshot as the balances.
    Page { offset: usize, limit: usize },
}

/// The sellers of a sheet: the rows asked for and the totals over the whole filter.
struct Loaded {
    rows: Vec<SellerDuesRow>,
    total_count: i64,
    shop_owes: Paisa,
    seller_owes: Paisa,
    advances_open: Paisa,
    total: Balance,
}

/// The city of each account, by account id.
fn city_of_accounts(
    conn: &Connection,
    ids: &[i64],
) -> Result<HashMap<i64, Option<i64>>, ReportsError> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(500) {
        let sql = format!(
            "SELECT id, city_id FROM accounts WHERE id IN ({})",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement
            .query_map(params_from_iter(chunk.iter()), |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        out.extend(rows);
    }
    Ok(out)
}

/// The day of the last posted line of each seller account, in one grouped read.
fn last_changes(conn: &Connection) -> Result<HashMap<i64, String>, ReportsError> {
    let mut statement = conn.prepare(
        "SELECT l.account_id, MAX(e.entry_date)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code = '03011'
         GROUP BY l.account_id",
    )?;
    let rows = statement
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<HashMap<_, _>, _>>()?;
    Ok(rows)
}

fn row_of(seller: Seller) -> SellerDuesRow {
    let (payable, advance, owes) = match seller.working {
        Some(w) => (w.open, w.advances_open, w.owes_firm),
        None => (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO),
    };
    SellerDuesRow {
        party: seller.party,
        city: None,
        balance: seller.balance,
        payable_paisa: payable,
        advance_open_paisa: advance,
        owes_paisa: owes,
        last_change_date: None,
    }
}

fn keeps(side: SellerDuesSide, row: &SellerDuesRow) -> bool {
    match side {
        SellerDuesSide::All => true,
        SellerDuesSide::ShopOwes => row.payable_paisa > Paisa::ZERO,
        SellerDuesSide::SellerOwes => row.owes_paisa > Paisa::ZERO,
    }
}

/// Every seller of the filter, sorted, with the totals over all of them.
fn load_all(
    db: &Db,
    side: SellerDuesSide,
    party_id: Option<i64>,
    search: Option<&str>,
    sort: Option<&SortInput>,
    changes_for: Changes,
) -> Result<Loaded, ReportsError> {
    if let Some(sort) = sort {
        sort.check(&SORT_COLUMNS)
            .map_err(|e: FieldError| ReportsError::Rejected(vec![e]))?;
    }
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let typed = Typed {
                today: Some(today),
                party_id,
                ..Typed::default()
            };
            let (sellers, _) = candidates(conn, &typed, &today.to_iso(), true)?;
            let text = search.map_or("", str::trim);
            let (text_ur, text_en) = (search_ur(text), search_en(text));
            let matched: Vec<SellerDuesRow> = sellers
                .into_iter()
                .filter(|s| text.is_empty() || matches(&s.party, text, &text_ur, &text_en))
                .map(row_of)
                .collect();
            // The side totals are over the party and the search, whatever the side (plan 79 2a).
            let (mut shop_owes, mut seller_owes, mut advances_open) =
                (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
            for row in &matched {
                add(&mut shop_owes, row.payable_paisa)?;
                add(&mut seller_owes, row.owes_paisa)?;
                add(&mut advances_open, row.advance_open_paisa)?;
            }
            let mut rows: Vec<SellerDuesRow> =
                matched.into_iter().filter(|r| keeps(side, r)).collect();
            let (mut dr, mut cr) = (Paisa::ZERO, Paisa::ZERO);
            for row in &rows {
                match row.balance.side {
                    BalanceSide::Dr => add(&mut dr, row.balance.amount_paisa)?,
                    BalanceSide::Cr => add(&mut cr, row.balance.amount_paisa)?,
                    BalanceSide::Zero => {}
                }
            }
            let total = Balance::from_totals(dr, cr)?;
            // The city and, for a sort or a print, the last change of every row.
            let ids: Vec<i64> = rows.iter().map(|r| r.party.account_id).collect();
            let city_ids = city_of_accounts(conn, &ids)?;
            let names = cities(conn)?;
            for row in &mut rows {
                row.city = city_ids
                    .get(&row.party.account_id)
                    .copied()
                    .flatten()
                    .and_then(|id| names.get(&id).cloned());
            }
            if changes_for == Changes::All {
                let changes = last_changes(conn)?;
                for row in &mut rows {
                    row.last_change_date = changes.get(&row.party.account_id).cloned();
                }
            }
            let collator = UrduCollator::new().ok();
            let by_name = |a: &SellerDuesRow, b: &SellerDuesRow| match &collator {
                Some(c) => c.compare(&a.party.name_ur, &b.party.name_ur),
                None => a.party.name_ur.cmp(&b.party.name_ur),
            };
            match sort {
                None => rows.sort_by(by_name),
                Some(sort) => {
                    let ordering = |a: &SellerDuesRow, b: &SellerDuesRow| -> Ordering {
                        match sort.column.as_str() {
                            "payable" => a.payable_paisa.cmp(&b.payable_paisa),
                            "advance" => a.advance_open_paisa.cmp(&b.advance_open_paisa),
                            "owes" => a.owes_paisa.cmp(&b.owes_paisa),
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
                for row in &mut rows {
                    row.last_change_date =
                        edge_day_of_account(conn, row.party.account_id, DayEdge::Last)?;
                }
            }
            Ok(Loaded {
                rows,
                total_count,
                shop_owes,
                seller_owes,
                advances_open,
                total,
            })
        })
    })
}

/// The sellers of the filter, a page of them, with the totals of both sides.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `sort.column`; a database or money error.
pub fn seller_dues(db: &Db, input: &SellerDuesInput) -> Result<SellerDuesPage, ReportsError> {
    let by_change = input
        .sort
        .as_ref()
        .is_some_and(|s| s.column == "last_change_date");
    let limit = match input.limit as usize {
        0 => PAGE,
        n => n.min(MAX_PAGE),
    };
    let offset = input.offset as usize;
    let loaded = load_all(
        db,
        input.side,
        input.party_id,
        input.search.as_deref(),
        input.sort.as_ref(),
        if by_change {
            Changes::All
        } else {
            Changes::Page { offset, limit }
        },
    )?;
    let rows = if by_change {
        loaded.rows.into_iter().skip(offset).take(limit).collect()
    } else {
        loaded.rows
    };
    Ok(SellerDuesPage {
        rows,
        total_count: loaded.total_count,
        shop_owes_paisa: loaded.shop_owes,
        seller_owes_paisa: loaded.seller_owes,
        advances_open_paisa: loaded.advances_open,
        total: loaded.total,
    })
}

/// The print of the seller dues: every seller of the filter, with the totals in the last row.
///
/// # Errors
/// As [`seller_dues`]; a stored amount that cannot be written in words.
pub fn seller_dues_spec(db: &Db, input: &SellerDuesPrintInput) -> Result<TableSpec, ReportsError> {
    let loaded = load_all(
        db,
        input.side,
        input.party_id,
        input.search.as_deref(),
        input.sort.as_ref(),
        // The print has the day of the last change of every seller.
        Changes::All,
    )?;
    let column = |key: &str, title: &str, kind: ColumnKind| Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    };
    let columns = vec![
        column("party", ids::COL_SELLER, ColumnKind::Party),
        column("city", "print-buyer-dues-col-city", ColumnKind::Names),
        column("payable", ids::COL_PAYABLE, ColumnKind::Money),
        column("advance", ids::COL_ADVANCE_OPEN, ColumnKind::Money),
        column("owes", ids::COL_OWES_FIRM, ColumnKind::Money),
        column("balance", ids::COL_BALANCE, ColumnKind::Balance),
        column(
            "last_change_date",
            "print-buyer-dues-col-last",
            ColumnKind::Date,
        ),
    ];
    let money = |p: Paisa| {
        if p == Paisa::ZERO {
            Cell::Empty
        } else {
            Cell::Money { paisa: p }
        }
    };
    let count = i64::try_from(loaded.rows.len()).unwrap_or(i64::MAX);
    // The totals row adds the rows above it, whatever the side filter (PRD FR-19.5).
    let (mut shown_payable, mut shown_advance, mut shown_owes) =
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    for row in &loaded.rows {
        add(&mut shown_payable, row.payable_paisa)?;
        add(&mut shown_advance, row.advance_open_paisa)?;
        add(&mut shown_owes, row.owes_paisa)?;
    }
    let rows = loaded
        .rows
        .into_iter()
        .map(|r| Row {
            cells: vec![
                Cell::Party { party: r.party },
                r.city.map_or(Cell::Empty, |names| Cell::Names { names }),
                money(r.payable_paisa),
                money(r.advance_open_paisa),
                money(r.owes_paisa),
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
    let spec = TableSpec {
        title_id: "print-seller-dues-title".to_owned(),
        facts,
        column_names: vec![None; columns.len()],
        columns,
        rows,
        totals_label_id: "print-buyer-dues-total".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            money(shown_payable),
            money(shown_advance),
            money(shown_owes),
            Cell::Balance {
                paisa: loaded.total.amount_paisa,
                side: loaded.total.side,
            },
            Cell::Empty,
        ],
        total_count: count,
        words_of: None,
        plain_sale: false,
        extra_totals: Vec::new(),
    };
    Ok(spec)
}

/// The print of the same table, in the print language.
///
/// # Errors
/// As [`seller_dues_spec`].
pub fn seller_dues_print_data(
    db: &Db,
    input: &SellerDuesPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = seller_dues_spec(db, input)?;
    table_print_data(db, input.language, input.paper, spec)
}
