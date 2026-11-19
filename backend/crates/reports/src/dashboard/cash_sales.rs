//! The cash sales page (plan 79 item 2c, PRD FR-9.8): the tile آج کی نقدی is the money buyers paid
//! on the spot (موقع پر جمع, PRD FR-12.44) today. The page shows one month of days, and one day's
//! buyers.
//!
//! Every figure is the stored `paid_paisa` of the live sale lines of the day (a cancelled or replaced
//! line is not live), so the tile, a day, a month and R3 agree. A وصولی is not نقدی.
//!
//! A buyer who paid in two بکری of one day is one buyer in the day row of the month and in the tile, and
//! two rows on the day page (`buyer_count` there counts the same way as the month).
//!
//! Example (after Events 1 to 5, 10-10-26): the tile 12,800 in 1 buyer; the month row for 10-10-26
//! 1 buyer, 12,800; its day: شاہد (چکتا), بکری 2, bill 12,800, نقدی 12,800.

use std::collections::BTreeMap;

use mandi_core::auth::Language;
use mandi_core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_core::months::Month;
use mandi_core::printing::Paper;
use mandi_core::text::UrduCollator;
use mandi_db::Db;
use mandi_db::rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::ReportsError;
use crate::print::PrintFact;
use crate::reports::common::{
    add, chukta_party, in_snapshot, load_parties, paisa, parse_day, today as today_of,
};
use crate::reports::{
    Cell, Column, ColumnKind, PartyRef, Row, RowKind, TablePrintData, TableSpec, table_print_data,
};

/// The rows a page of one day holds when no limit is sent.
const PAGE: usize = 100;
/// The most rows one page holds.
const MAX_PAGE: usize = 100;

/// Input of `dashboard_cash_sales_month`.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct CashSalesMonthInput {
    /// The month, `YYYY-MM`. Null means the current month. A month after today is refused.
    #[serde(default)]
    pub month: Option<String>,
}

/// One day of the month with money paid on the spot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashSalesDayRow {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// How many buyers paid something on the spot that day.
    #[ts(type = "number")]
    pub buyer_count: i64,
    /// What they paid on the spot together.
    #[ts(type = "Paisa")]
    pub cash_paisa: Paisa,
}

/// What `dashboard_cash_sales_month` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashSalesMonth {
    /// The month shown, `YYYY-MM`.
    pub month: String,
    /// The days of the month that have money paid on the spot, oldest first.
    pub rows: Vec<CashSalesDayRow>,
    /// The buyers of all the days added up (a buyer who paid on two days counts twice).
    #[ts(type = "number")]
    pub buyer_days_total: i64,
    /// The month total.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// Input of `dashboard_cash_sales_day`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CashSalesDayInput {
    /// The day, `DD-MM-YYYY` or a shortcut (PRD FR-7.19).
    pub date: String,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 100. 0 gives 100.
    #[serde(default)]
    pub limit: u32,
}

/// One buyer of a day, in one بکری.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashSalesBuyerRow {
    /// The buyer. A چکتا buyer is a person key on the account 0030, with his names and phone.
    pub party: PartyRef,
    /// True for a چکتا buyer.
    pub is_chukta: bool,
    /// The بکری, to open it.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub bikri_number: i64,
    /// His bill in that بکری that day: the buyer totals of his lines.
    #[ts(type = "Paisa")]
    pub bill_paisa: Paisa,
    /// What he paid on the spot.
    #[ts(type = "Paisa")]
    pub cash_paisa: Paisa,
}

/// What `dashboard_cash_sales_day` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashSalesDay {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The page of buyers who paid something, by بکری number then name.
    pub rows: Vec<CashSalesBuyerRow>,
    /// How many rows the day has (a buyer in two بکری has two rows).
    #[ts(type = "number")]
    pub total_count: i64,
    /// How many different buyers paid something on the spot: the number of the day's row of the
    /// month page and of the tile.
    #[ts(type = "number")]
    pub buyer_count: i64,
    /// The bills of all the rows.
    #[ts(type = "Paisa")]
    pub bill_total_paisa: Paisa,
    /// What they paid on the spot: the day's row of the month and the tile (for today).
    #[ts(type = "Paisa")]
    pub cash_total_paisa: Paisa,
}

fn reject(code: ErrorCode, field: &str) -> ReportsError {
    ReportsError::Rejected(vec![FieldError::new(code, field)])
}

/// The buyers who paid on the spot on each day of a range `?1` to `?2`: the lines in force of the
/// postings of those days, found through the date index of the journal entry (Code Standards B18).
const DAYS_SQL: &str =
    "SELECT je.entry_date, s.buyer_account_id, s.chukta_person_key, SUM(s.paid_paisa)
     FROM journal_entries je
     JOIN bikri_sales s ON s.journal_entry_id = je.id
     WHERE s.is_active = 1 AND je.status = 'posted'
       AND je.entry_date >= ?1 AND je.entry_date <= ?2
     GROUP BY je.entry_date, s.buyer_account_id, s.chukta_person_key
     HAVING SUM(s.paid_paisa) > 0";

/// The buyers of one day `?1`, one row for each buyer in each بکری.
const DAY_BUYERS_SQL: &str =
    "SELECT s.buyer_account_id, s.chukta_person_key, MAX(s.chukta_name_ur),
            MAX(s.chukta_name_en), b.id, b.number,
            SUM(s.buyer_total_paisa), SUM(s.paid_paisa)
     FROM journal_entries je
     JOIN bikri_sales s ON s.journal_entry_id = je.id
     JOIN bikri_entries b ON b.id = s.bikri_id
     WHERE s.is_active = 1 AND je.status = 'posted' AND je.entry_date = ?1
     GROUP BY s.buyer_account_id, s.chukta_person_key, b.id
     HAVING SUM(s.paid_paisa) > 0";

/// The SQL of the cash sales page for the query plan test (Code Standards B18).
#[doc(hidden)]
pub mod plans {
    /// The sums of each day of a range (`?1` first day, `?2` last day).
    pub fn days() -> &'static str {
        super::DAYS_SQL
    }

    /// The buyers of one day (`?1`).
    pub fn day_buyers() -> &'static str {
        super::DAY_BUYERS_SQL
    }
}

/// What the buyers paid on the spot on the days of a range, from the sale lines in force (the
/// lines of R3): the days that have some, with the buyers who paid and the money. The range is
/// found through the date index of the journal entry the line was posted in (Code Standards B18).
fn days_in(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<BTreeMap<String, (i64, Paisa)>, ReportsError> {
    let mut statement = conn.prepare(DAYS_SQL)?;
    let rows = statement
        .query_map([from, to], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut days: BTreeMap<String, (i64, Paisa)> = BTreeMap::new();
    for (day, paid) in rows {
        let entry = days.entry(day).or_insert((0, Paisa::ZERO));
        entry.0 = entry
            .0
            .checked_add(1)
            .ok_or_else(|| ReportsError::Data("the buyer count does not fit".to_owned()))?;
        add(&mut entry.1, paisa(paid)?)?;
    }
    Ok(days)
}

/// The tile آج کی نقدی: what the buyers paid on the spot on one day and how many buyers paid.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit.
pub(crate) fn of_day(conn: &Connection, day: &str) -> Result<(Paisa, i64), ReportsError> {
    Ok(days_in(conn, day, day)?
        .remove(day)
        .map_or((Paisa::ZERO, 0), |(count, money)| (money, count)))
}

/// One month of days with money paid on the spot.
///
/// # Errors
/// `ERR_FORMAT_INVALID` and `ERR_DATE_IN_FUTURE` on `month`; a database or money error.
pub fn cash_sales_month(
    db: &Db,
    input: &CashSalesMonthInput,
) -> Result<CashSalesMonth, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let current = Month::of(today);
            let month = match input
                .month
                .as_deref()
                .map(str::trim)
                .filter(|m| !m.is_empty())
            {
                None => current,
                Some(text) => Month::parse(text)
                    .ok_or_else(|| reject(ErrorCode::ErrFormatInvalid, "month"))?,
            };
            if month > current {
                return Err(reject(ErrorCode::ErrDateInFuture, "month"));
            }
            let (Some(first), Some(last)) = (month.first_day(), month.last_day()) else {
                return Err(reject(ErrorCode::ErrFormatInvalid, "month"));
            };
            let days = days_in(conn, &first.to_iso(), &last.to_iso())?;
            let (mut buyer_days, mut total) = (0_i64, Paisa::ZERO);
            let mut rows = Vec::with_capacity(days.len());
            for (date, (count, money)) in days {
                buyer_days = buyer_days
                    .checked_add(count)
                    .ok_or_else(|| ReportsError::Data("the buyer count does not fit".to_owned()))?;
                add(&mut total, money)?;
                rows.push(CashSalesDayRow {
                    date,
                    buyer_count: count,
                    cash_paisa: money,
                });
            }
            Ok(CashSalesMonth {
                month: month.key(),
                rows,
                buyer_days_total: buyer_days,
                total_paisa: total,
            })
        })
    })
}

/// One buyer of the day in one بکری, before his party is loaded.
struct Group {
    account_id: i64,
    person_key: Option<String>,
    name_ur: Option<String>,
    name_en: Option<String>,
    bikri_id: i64,
    bikri_number: i64,
    bill: Paisa,
    cash: Paisa,
}

/// Every buyer who paid on the spot on one day, sorted, with the totals over all of them.
fn load_day(db: &Db, date: &str) -> Result<CashSalesDay, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let text = date.trim();
            if text.is_empty() {
                return Err(reject(ErrorCode::ErrRequired, "date"));
            }
            let day: BusinessDate = parse_day(text, today).map_err(|code| reject(code, "date"))?;
            if day > today {
                return Err(reject(ErrorCode::ErrDateInFuture, "date"));
            }
            let iso = day.to_iso();
            let mut statement = conn.prepare(DAY_BUYERS_SQL)?;
            let raw = statement
                .query_map([iso.as_str()], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, i64>(4)?,
                        r.get::<_, i64>(5)?,
                        r.get::<_, i64>(6)?,
                        r.get::<_, i64>(7)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            let mut groups = Vec::with_capacity(raw.len());
            let (mut bill_total, mut cash_total) = (Paisa::ZERO, Paisa::ZERO);
            for (account_id, person_key, name_ur, name_en, bikri_id, bikri_number, bill, cash) in
                raw
            {
                let (bill, cash) = (paisa(bill)?, paisa(cash)?);
                add(&mut bill_total, bill)?;
                add(&mut cash_total, cash)?;
                groups.push(Group {
                    account_id,
                    person_key,
                    name_ur,
                    name_en,
                    bikri_id,
                    bikri_number,
                    bill,
                    cash,
                });
            }
            let total_count = i64::try_from(groups.len()).unwrap_or(i64::MAX);
            let buyers: std::collections::HashSet<(i64, Option<&str>)> = groups
                .iter()
                .map(|g| (g.account_id, g.person_key.as_deref()))
                .collect();
            let buyer_count = i64::try_from(buyers.len()).unwrap_or(i64::MAX);
            let ids: Vec<i64> = groups.iter().map(|g| g.account_id).collect();
            let parties = load_parties(conn, &ids)?;
            // By بکری number, then by name in the Urdu order, then by person key.
            let collator = UrduCollator::new().ok();
            let mut people: Vec<(Group, PartyRef)> = Vec::with_capacity(groups.len());
            for g in groups {
                let Some(base) = parties.get(&g.account_id).cloned() else {
                    return Err(ReportsError::Data(format!(
                        "buyer account {} is missing",
                        g.account_id
                    )));
                };
                let party = chukta_party(
                    base,
                    g.person_key.clone(),
                    g.name_ur.as_deref(),
                    g.name_en.as_deref(),
                );
                people.push((g, party));
            }
            people.sort_by(|(ga, pa), (gb, pb)| {
                ga.bikri_number
                    .cmp(&gb.bikri_number)
                    .then_with(|| match &collator {
                        Some(c) => c.compare(&pa.name_ur, &pb.name_ur),
                        None => pa.name_ur.cmp(&pb.name_ur),
                    })
                    .then_with(|| pa.person_key.cmp(&pb.person_key))
            });
            let rows = people
                .into_iter()
                .map(|(g, party)| CashSalesBuyerRow {
                    is_chukta: g.person_key.is_some(),
                    party,
                    bikri_id: g.bikri_id,
                    bikri_number: g.bikri_number,
                    bill_paisa: g.bill,
                    cash_paisa: g.cash,
                })
                .collect();
            Ok(CashSalesDay {
                date: iso,
                rows,
                total_count,
                buyer_count,
                bill_total_paisa: bill_total,
                cash_total_paisa: cash_total,
            })
        })
    })
}

/// One day's buyers who paid on the spot.
///
/// # Errors
/// The date codes on `date`; a database or money error.
pub fn cash_sales_day(db: &Db, input: &CashSalesDayInput) -> Result<CashSalesDay, ReportsError> {
    let mut day = load_day(db, &input.date)?;
    let limit = match input.limit as usize {
        0 => PAGE,
        n => n.min(MAX_PAGE),
    };
    day.rows = day
        .rows
        .into_iter()
        .skip(input.offset as usize)
        .take(limit)
        .collect();
    Ok(day)
}

/// Input of `dashboard_cash_sales_month_print_data`.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct CashSalesMonthPrintInput {
    /// The month, `YYYY-MM`. Null means the current month.
    #[serde(default)]
    pub month: Option<String>,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}

/// Input of `dashboard_cash_sales_day_print_data`.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct CashSalesDayPrintInput {
    /// The day, `DD-MM-YYYY` or a shortcut.
    pub date: String,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}

fn column(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

/// The print of one month of cash sales: a row for each day, the month total in the last row.
///
/// # Errors
/// As [`cash_sales_month`]; a stored amount that cannot be written in words.
pub fn cash_sales_month_spec(
    db: &Db,
    input: &CashSalesMonthPrintInput,
) -> Result<TableSpec, ReportsError> {
    let month = cash_sales_month(
        db,
        &CashSalesMonthInput {
            month: input.month.clone(),
        },
    )?;
    let columns = vec![
        column("date", "print-commission-col-date", ColumnKind::Date),
        column("buyers", "print-cash-sales-col-buyers", ColumnKind::Number),
        column("cash", "print-cash-sales-col-cash", ColumnKind::Money),
    ];
    let rows = month
        .rows
        .iter()
        .map(|r| Row {
            cells: vec![
                Cell::Date {
                    iso: r.date.clone(),
                },
                Cell::Number {
                    value: r.buyer_count,
                },
                Cell::Money {
                    paisa: r.cash_paisa,
                },
            ],
            source: None,
            kind: RowKind::Normal,
        })
        .collect::<Vec<_>>();
    let (y, m) = month.month.split_once('-').unwrap_or(("", ""));
    let spec = TableSpec {
        title_id: "print-cash-sales-month-title".to_owned(),
        facts: vec![PrintFact {
            label_id: "print-fact-month".to_owned(),
            value: format!("{m}-{y}"),
        }],
        column_names: vec![None; columns.len()],
        columns,
        total_count: i64::try_from(rows.len()).unwrap_or(i64::MAX),
        rows,
        totals_label_id: "print-buyer-dues-total".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Number {
                value: month.buyer_days_total,
            },
            Cell::Money {
                paisa: month.total_paisa,
            },
        ],
        words_of: Some(month.total_paisa).filter(|t| *t != Paisa::ZERO),
        plain_sale: false,
        extra_totals: Vec::new(),
    };
    Ok(spec)
}

/// The print of the same table, in the print language.
///
/// # Errors
/// As [`cash_sales_month_spec`].
pub fn cash_sales_month_print_data(
    db: &Db,
    input: &CashSalesMonthPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = cash_sales_month_spec(db, input)?;
    table_print_data(db, input.language, input.paper, spec)
}

/// The print of one day's cash sales: every buyer who paid, with the totals in the last row.
///
/// # Errors
/// As [`cash_sales_day`]; a stored amount that cannot be written in words.
pub fn cash_sales_day_spec(
    db: &Db,
    input: &CashSalesDayPrintInput,
) -> Result<TableSpec, ReportsError> {
    let day = load_day(db, &input.date)?;
    let columns = vec![
        column("buyer", crate::reports::ids::COL_BUYER, ColumnKind::Party),
        column("bikri", "print-cash-sales-col-bikri", ColumnKind::Doc),
        column("bill", "print-cash-sales-col-bill", ColumnKind::Money),
        column("cash", "print-cash-sales-col-cash", ColumnKind::Money),
    ];
    let rows = day
        .rows
        .iter()
        .map(|r| Row {
            cells: vec![
                Cell::Party {
                    party: r.party.clone(),
                },
                Cell::Doc {
                    type_id: "report-doc-bikri-first".to_owned(),
                    number: Some(r.bikri_number),
                },
                Cell::Money {
                    paisa: r.bill_paisa,
                },
                Cell::Money {
                    paisa: r.cash_paisa,
                },
            ],
            source: None,
            kind: RowKind::Normal,
        })
        .collect::<Vec<_>>();
    let date = BusinessDate::parse_iso(&day.date)
        .map(format_ddmmyyyy)
        .unwrap_or_default();
    let spec = TableSpec {
        title_id: "print-cash-sales-day-title".to_owned(),
        facts: vec![PrintFact {
            label_id: "print-fact-date".to_owned(),
            value: date,
        }],
        column_names: vec![None; columns.len()],
        columns,
        total_count: day.total_count,
        rows,
        totals_label_id: "print-buyer-dues-total".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: day.bill_total_paisa,
            },
            Cell::Money {
                paisa: day.cash_total_paisa,
            },
        ],
        words_of: Some(day.cash_total_paisa).filter(|t| *t != Paisa::ZERO),
        plain_sale: false,
        extra_totals: Vec::new(),
    };
    Ok(spec)
}

/// The print of the same table, in the print language.
///
/// # Errors
/// As [`cash_sales_day_spec`].
pub fn cash_sales_day_print_data(
    db: &Db,
    input: &CashSalesDayPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = cash_sales_day_spec(db, input)?;
    table_print_data(db, input.language, input.paper, spec)
}
