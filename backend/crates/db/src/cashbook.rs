//! The روکڑ بک in SQL (PRD 16.1, FR-16.14 v1.60, FR-19.11, plan 17, plan 38 B2, plan 99,
//! NFR-SPD-24): the rows of cash, one page at a time, with the sums over the whole range.
//!
//! No rules here. Sums are `SUM()` on INTEGER columns only (money agent Section 9); the caller
//! subtracts in checked `Paisa` maths. Reads posted entries only (PRD FR-6.1).
//!
//! **A row** is a journal entry that moves cash and a **part** of it: part 0 is the whole entry; a
//! بکری posting is split into part 1, what cash received on it (موقع پر جمع), and part 2, what cash
//! paid out (نقد ادائیگی). Each row has one **kind** ([`kind_sql`], one definition for the list, the
//! type filter and the summary).
//!
//! **In force**: an entry that is neither reversed nor a reversal. A reversal is posted on the day
//! of the entry it reverses, so leaving the pairs out changes no day-end balance.
//!
//! The cash account has hundreds of thousands of lines, so a page must not read them all:
//!
//! - **Natural order** (by day, then entry, then part): the page walks `journal_entries` by the day
//!   index and stops after `offset + limit` rows, so the first pages cost a few hundred lookups. The
//!   running cash of the first row comes from the sums of the entries before the page, or, past the
//!   middle of the book, from the total less the entries after it.
//! - **By another column or by amount** the rows are read and sorted; the running cash is not given.
//! - **Totals** of all entries come from the covering index when no day is asked for, and from the
//!   day index when a range is (the last answers are kept, `memo`); the pairs not in force are read
//!   from the index of reversals and taken off.

use std::collections::HashMap;

use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};

use crate::listing::MonthSums;
use crate::memo::{self, Sums};
use crate::row_source_sql;

/// An entry `e` that is in force: not a reversal and not reversed.
const IN_FORCE: &str = "e.reverses_entry_id IS NULL
    AND NOT EXISTS (SELECT 1 FROM journal_entries rv WHERE rv.reverses_entry_id = e.id)";

/// The posting type an entry `e` stands for: a reversal stands for the type of what it reverses.
const BASE_TYPE: &str = "(CASE WHEN e.posting_type = 'reversal'
    THEN (SELECT og.posting_type FROM journal_entries og WHERE og.id = e.reverses_entry_id)
    ELSE e.posting_type END)";

/// The three parts a row can be, as the inner side of a `CROSS JOIN` (which keeps the walk of
/// the entries outside).
const PARTS: &str = "(SELECT 0 AS part UNION ALL SELECT 1 UNION ALL SELECT 2)";

/// True when entry `e` is split into a part for cash in and a part for cash out: a بکری posting.
fn split_sql() -> String {
    format!("(e.source_type = 'bikri' AND {BASE_TYPE} IN ('bikri_first', 'bikri_later'))")
}

/// True when `(e, p.part)` is a row: the part fits the entry and it has a cash line of its side.
/// `cash` is the id of the cash account (a number, not text from the user). Without `split`, every
/// entry is one row (part 0), as R12 shows it.
fn is_row_sql(cash: i64, split: bool) -> String {
    let split = if split { split_sql() } else { "0".to_owned() };
    format!(
        "(CASE WHEN {split} THEN p.part > 0 ELSE p.part = 0 END)
         AND EXISTS (SELECT 1 FROM journal_lines cx WHERE cx.account_id = {cash} AND cx.entry_id = e.id
                     AND (p.part = 0 OR (p.part = 1 AND cx.dr_paisa > 0) OR (p.part = 2 AND cx.cr_paisa > 0)))"
    )
}

/// What cash received on the row `(e, p.part)`.
fn in_sql(cash: i64) -> String {
    format!(
        "(CASE WHEN p.part = 2 THEN 0 ELSE (SELECT COALESCE(SUM(ci.dr_paisa), 0) FROM journal_lines ci
            WHERE ci.account_id = {cash} AND ci.entry_id = e.id) END)"
    )
}

/// What cash paid out on the row `(e, p.part)`.
fn out_sql(cash: i64) -> String {
    format!(
        "(CASE WHEN p.part = 1 THEN 0 ELSE (SELECT COALESCE(SUM(co.cr_paisa), 0) FROM journal_lines co
            WHERE co.account_id = {cash} AND co.entry_id = e.id) END)"
    )
}

/// The voucher column `column` of an entry `e` that is a voucher's.
fn voucher_of(column: &str) -> String {
    format!(
        "(SELECT vq.{column} FROM vouchers vq WHERE e.source_type = 'voucher' AND vq.id = e.source_id)"
    )
}

/// The kind of the row `(e, p.part)`: the words of `mandi_core::cashbook::CashKind` (PRD FR-16.14
/// v1.60). A reversal has the kind of what it reverses; on a بکری reversal the sides are swapped (the
/// money given back to a seller comes in).
pub fn kind_sql(cash: i64) -> String {
    let entered = voucher_of("entered_as");
    // The پیشگی typed to a buyer is a payment whose party is a buyer (the lines of the entry are
    // read, so an old version of an edited voucher keeps its own party).
    let to_buyer =
        "EXISTS (SELECT 1 FROM journal_lines bl JOIN accounts ba ON ba.id = bl.account_id
        WHERE bl.entry_id = e.id AND ba.party_type = 'buyer')";
    let venture_entry = "(SELECT vk.kind FROM venture_entries vk WHERE e.source_type = 'venture' AND vk.id = e.source_id)";
    let venture_kind = row_source_sql::VENTURE_KIND;
    format!(
        "(CASE {BASE_TYPE}
            WHEN 'bikri_first' THEN (CASE WHEN (p.part = 1) = (e.posting_type <> 'reversal') THEN 'bikri_paid' ELSE 'bikri_seller_cash' END)
            WHEN 'bikri_later' THEN (CASE WHEN (p.part = 1) = (e.posting_type <> 'reversal') THEN 'bikri_paid' ELSE 'bikri_seller_cash' END)
            WHEN 'advance' THEN (CASE WHEN e.source_type = 'bikri' THEN 'bikri_advance' ELSE 'advance' END)
            WHEN 'payment' THEN (CASE WHEN {to_buyer} THEN (CASE WHEN {entered} = 'advance' THEN 'buyer_advance' ELSE 'cash_lent' END) ELSE 'payment' END)
            WHEN 'receipt' THEN 'receipt'
            WHEN 'money_lent' THEN 'money_lent'
            WHEN 'money_lent_return' THEN 'money_lent_return'
            WHEN 'shop_expense' THEN 'shop_expense'
            WHEN 'personal_expense' THEN 'personal_expense'
            WHEN 'personal_return' THEN 'personal_return'
            WHEN 'capital_in' THEN 'capital_in'
            WHEN 'capital_out' THEN 'capital_out'
            WHEN 'khata_out' THEN 'khata_out'
            WHEN 'khata_in' THEN 'khata_in'
            WHEN 'purchase' THEN 'purchase'
            WHEN 'trip_cost' THEN 'trip_cost'
            WHEN 'garden_payment' THEN (CASE WHEN {venture_entry} = 'cost' THEN 'garden_cost' ELSE 'garden_installment' END)
            WHEN 'venture_receipt' THEN (CASE WHEN {venture_entry} = 'refund' THEN 'garden_refund'
                WHEN {venture_kind} = 'garden' THEN 'garden_other_sale' ELSE 'trip_other_sale' END)
            WHEN 'general' THEN 'general'
            WHEN 'fruit_taken' THEN 'fruit'
            WHEN 'opening' THEN (CASE WHEN e.source_type = 'account' AND e.source_id = {cash} THEN 'opening_cash'
                WHEN EXISTS (SELECT 1 FROM partners po WHERE e.source_type = 'account' AND po.capital_account_id = e.source_id) THEN 'opening_capital'
                ELSE 'opening_balance' END)
            ELSE 'other' END)"
    )
}

/// The key of one row: an entry and a part (see the module text).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowKey {
    /// The journal entry.
    pub entry_id: i64,
    /// 0 the whole entry, 1 cash received on a بکری, 2 cash paid out on a بکری.
    pub part: i64,
}

/// One row of a page: its key and, in day order without a type filter, the running sums.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageKey {
    /// The row.
    pub key: RowKey,
    /// Σ of cash received over this row and every row before it in the set of rows shown, in day
    /// order. `None` when the page is not in day order or a type filter is set.
    pub run_in_paisa: Option<i64>,
    /// Σ of cash paid out, as above.
    pub run_out_paisa: Option<i64>,
}

/// What one row shows, from its entry and its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CashRow {
    /// The row.
    pub key: RowKey,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The stored posting type.
    pub posting_type: String,
    /// The kind of [`kind_sql`].
    pub kind: String,
    /// `live`, `reversed` or `reversal`.
    pub status: String,
    /// What cash received on this row, paisa.
    pub in_paisa: i64,
    /// What cash paid out on this row, paisa.
    pub out_paisa: i64,
    /// The number of its voucher, بکری, purchase, garden or trip entry or پھل, if it has one.
    pub number: Option<i64>,
    /// The note of the entry. Empty if none.
    pub narration: String,
    /// The reason or the "paid to" of its voucher. Empty if none.
    pub reason: String,
    /// What the entry came from: `bikri`, `voucher`, `purchase`, `venture`, `fruit`, `account` or
    /// `month_close`.
    pub source_type: String,
    /// The id of that source. For `venture` it is the id of one entry of a trip or garden.
    pub source_id: i64,
    /// The trip or garden a `venture` entry belongs to. `None` for any other source.
    pub venture_id: Option<i64>,
    /// `trip` or `garden`, with `venture_id`.
    pub venture_kind: Option<String>,
    /// The trip or garden of a venture or purchase row: `(id, kind, number, name_ur, name_en)`.
    pub venture: Option<VentureName>,
}

/// The trip or garden a row names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VentureName {
    /// `ventures.id`.
    pub id: i64,
    /// `trip` or `garden`.
    pub kind: String,
    /// Its number in the series of its kind.
    pub number: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
}

/// The day of the first posted entry that moved cash, or `None` when none did. It walks the entries
/// by day and stops at the first that touches the cash account, so it costs a few lookups.
///
/// # Errors
/// Any SQLite error.
pub fn first_entry_date(conn: &Connection, cash_id: i64) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT e.entry_date FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
         WHERE e.status = 'posted'
           AND EXISTS (SELECT 1 FROM journal_lines l WHERE l.entry_id = e.id AND l.account_id = ?1)
         ORDER BY e.entry_date, e.id LIMIT 1",
        [cash_id],
        |r| r.get(0),
    )
    .map(Some)
    .or_else(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Ok(None),
        other => Err(other),
    })
}

/// The sums over all posted entries of a range, in force or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRange {
    /// How many entries move cash in the range.
    pub count: i64,
    /// Σ debits on cash, paisa.
    pub in_paisa: i64,
    /// Σ credits on cash, paisa.
    pub out_paisa: i64,
}

/// A set of rows: how many and what they add up to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RowSums {
    /// Rows.
    pub rows: i64,
    /// Σ cash received, paisa.
    pub in_paisa: i64,
    /// Σ cash paid out, paisa.
    pub out_paisa: i64,
}

/// The sums of a range: every row, and the rows in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeSums {
    /// Every row, the reversed ones and the reversals too.
    pub all: RowSums,
    /// The rows in force.
    pub live: RowSums,
}

/// How a page is ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    /// By day, then entry, then part, oldest first (`desc` false) or newest first. The only order
    /// with a running cash.
    Date {
        /// Newest first.
        desc: bool,
    },
    /// By what cash received (`out` false) or paid out (`out` true).
    Amount {
        /// Sort by the amount paid out.
        out: bool,
        /// Biggest first.
        desc: bool,
    },
    /// By another column: the SQL expression over `r` (see [`SORT_COLUMNS`]) with its direction.
    Other(String),
}

/// The columns the cash book can be sorted by: the name of the field of the row and its SQL
/// expression over the rows `r`. `running_paisa` is the day order, so it is not a sort of its own.
pub const SORT_COLUMNS: [(&str, &str); 6] = [
    ("entry_date", "r.entry_date"),
    ("posting_type", "r.posting_type"),
    ("number", NUMBER_OF_ENTRY),
    ("narration", "r.narration"),
    ("in_paisa", "r.in_p"),
    ("out_paisa", "r.out_p"),
];

/// The number of the voucher or بکری of a row `r`, for a sort by number.
const NUMBER_OF_ENTRY: &str = "CASE r.source_type
    WHEN 'voucher' THEN (SELECT v.number FROM vouchers v WHERE v.id = r.source_id)
    WHEN 'bikri' THEN (SELECT b.number FROM bikri_entries b WHERE b.id = r.source_id)
    WHEN 'purchase' THEN (SELECT pu.number FROM purchases pu WHERE pu.id = r.source_id)
    WHEN 'venture' THEN (SELECT vn.number FROM venture_entries vn WHERE vn.id = r.source_id)
END";

/// Which rows of a range are read.
#[derive(Debug, Clone, Copy)]
pub struct Filter<'a> {
    /// The cash account.
    pub cash_id: i64,
    /// First day, `YYYY-MM-DD`, included; `None` is open.
    pub from: Option<&'a str>,
    /// Last day, included; `None` is open.
    pub to: Option<&'a str>,
    /// True to read the reversed postings and the reversals too.
    pub show_reversed: bool,
    /// Only these kinds (the words of [`kind_sql`]); empty is every kind.
    pub kinds: &'a [&'a str],
    /// True to split a بکری posting into its cash in and its cash out (the روکڑ بک screen); false
    /// for one row an entry (R12).
    pub split: bool,
}

/// `AND` conditions on the day of an entry `e`, only for the ends that are set, and their values.
fn day_conditions(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
    let mut sql = String::new();
    let mut binds = Vec::new();
    if let Some(from) = from {
        sql.push_str(" AND e.entry_date >= ?");
        binds.push(Value::Text(from.to_owned()));
    }
    if let Some(to) = to {
        sql.push_str(" AND e.entry_date <= ?");
        binds.push(Value::Text(to.to_owned()));
    }
    (sql, binds)
}

/// `AND` the entry is in force, unless every entry is asked for.
fn force_sql(show_reversed: bool) -> String {
    if show_reversed {
        String::new()
    } else {
        format!(" AND {IN_FORCE}")
    }
}

/// `AND` the kind of the row is one of `kinds`, with the values to bind; nothing when empty.
fn kinds_sql(cash: i64, kinds: &[&str]) -> (String, Vec<Value>) {
    if kinds.is_empty() {
        return (String::new(), Vec::new());
    }
    let marks = vec!["?"; kinds.len()].join(", ");
    (
        format!(" AND {} IN ({marks})", kind_sql(cash)),
        kinds.iter().map(|k| Value::Text((*k).to_owned())).collect(),
    )
}

/// True if every entry in the file is posted (a draft only exists inside a save). Uses the index
/// of unposted entries, so it is instant.
fn no_drafts(conn: &Connection) -> rusqlite::Result<bool> {
    let any: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM journal_entries WHERE status <> 'posted')",
        [],
        |r| r.get(0),
    )?;
    Ok(!any)
}

/// The count of entries, Σ in and Σ out of every posted entry in a range, in force or not. `from`
/// and `to` are `YYYY-MM-DD` and included; `None` leaves that end open. The sums are the cash that
/// moved: a reversed pair adds the same to both, so the net is the balance.
///
/// # Errors
/// Any SQLite error.
pub fn range(
    conn: &Connection,
    cash_id: i64,
    from: Option<&str>,
    to: Option<&str>,
) -> rusqlite::Result<CashRange> {
    let what = format!("cash {cash_id} {from:?} {to:?}");
    let sums = memo::sums(conn, &what, || {
        if from.is_none() && to.is_none() && no_drafts(conn)? {
            // The whole book: the covering index of the account alone, no join with the entries.
            return conn.query_row(
                "SELECT COUNT(DISTINCT entry_id), COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0)
                 FROM journal_lines WHERE account_id = ?1",
                [cash_id],
                |r| {
                    Ok(Sums {
                        count: r.get(0)?,
                        dr_paisa: r.get(1)?,
                        cr_paisa: r.get(2)?,
                    })
                },
            );
        }
        let (days, mut binds) = day_conditions(from, to);
        binds.insert(0, Value::Integer(cash_id));
        conn.query_row(
            &format!(
                "SELECT COUNT(DISTINCT e.id), COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                 FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
                 JOIN journal_lines l ON l.entry_id = e.id AND l.account_id = ?
                 WHERE e.status = 'posted'{days}"
            ),
            params_from_iter(binds.iter()),
            |r| {
                Ok(Sums {
                    count: r.get(0)?,
                    dr_paisa: r.get(1)?,
                    cr_paisa: r.get(2)?,
                })
            },
        )
    })?;
    Ok(CashRange {
        count: sums.count,
        in_paisa: sums.dr_paisa,
        out_paisa: sums.cr_paisa,
    })
}

/// The rows and sums of a range, all and in force, without a type filter. A reversal and the entry
/// it reverses are on the same day, so a pair is inside the range when its reversal is.
///
/// # Errors
/// Any SQLite error; a sum that does not fit.
pub fn range_sums(
    conn: &Connection,
    cash_id: i64,
    from: Option<&str>,
    to: Option<&str>,
) -> rusqlite::Result<RangeSums> {
    let whole = range(conn, cash_id, from, to)?;
    // The pairs not in force, read from the reversals alone. A pair moves, in and out each, what
    // its reversal moves in and out together.
    let (days, mut binds) = day_conditions(from, to);
    binds.insert(0, Value::Integer(cash_id));
    let (pairs, moved_dr, moved_cr): (i64, i64, i64) = conn.query_row(
        &format!(
            "SELECT COUNT(DISTINCT e.id), COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_entries e INDEXED BY idx_journal_entries_reverses
             JOIN journal_lines l ON l.entry_id = e.id AND l.account_id = ?
             WHERE e.reverses_entry_id IS NOT NULL AND e.status = 'posted'{days}"
        ),
        params_from_iter(binds.iter()),
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let moved = moved_dr.checked_add(moved_cr).ok_or_else(overflow)?;
    // The بکری postings with both sides: they are two rows each.
    let split = split_sql();
    let (days, binds) = day_conditions(from, to);
    let (both, both_live): (i64, i64) = conn.query_row(
        &format!(
            "SELECT COUNT(*), COALESCE(SUM(CASE WHEN {IN_FORCE} THEN 1 ELSE 0 END), 0)
             FROM journal_entries e
             WHERE e.source_type = 'bikri' AND e.status = 'posted'{days} AND {split}
               AND EXISTS (SELECT 1 FROM journal_lines d WHERE d.account_id = {cash_id} AND d.entry_id = e.id AND d.dr_paisa > 0)
               AND EXISTS (SELECT 1 FROM journal_lines c WHERE c.account_id = {cash_id} AND c.entry_id = e.id AND c.cr_paisa > 0)"
        ),
        params_from_iter(binds.iter()),
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let all = RowSums {
        rows: whole.count.checked_add(both).ok_or_else(overflow)?,
        in_paisa: whole.in_paisa,
        out_paisa: whole.out_paisa,
    };
    let live = RowSums {
        rows: pairs
            .checked_mul(2)
            .and_then(|n| whole.count.checked_sub(n))
            .and_then(|n| n.checked_add(both_live))
            .ok_or_else(overflow)?,
        in_paisa: whole.in_paisa.checked_sub(moved).ok_or_else(overflow)?,
        out_paisa: whole.out_paisa.checked_sub(moved).ok_or_else(overflow)?,
    };
    Ok(RangeSums { all, live })
}

/// The rows `r` of a filter: `(entry_id, part, entry_date, posting_type, source_type, source_id,
/// narration, kind, in_p, out_p)`, with the values of its `?` marks.
fn rows_cte(filter: &Filter<'_>) -> (String, Vec<Value>) {
    let cash = filter.cash_id;
    let (days, mut binds) = day_conditions(filter.from, filter.to);
    let force = force_sql(filter.show_reversed);
    let (kinds, kind_binds) = kinds_sql(cash, filter.kinds);
    binds.extend(kind_binds);
    let (is_row, kind, in_p, out_p) = (
        is_row_sql(cash, filter.split),
        kind_sql(cash),
        in_sql(cash),
        out_sql(cash),
    );
    let sql = format!(
        "WITH r AS (
             SELECT e.id AS entry_id, p.part AS part, e.entry_date AS entry_date,
                    e.posting_type AS posting_type, e.source_type AS source_type,
                    e.source_id AS source_id, COALESCE(e.narration, '') AS narration,
                    {kind} AS kind, {in_p} AS in_p, {out_p} AS out_p
             FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
             CROSS JOIN {PARTS} AS p
             WHERE e.status = 'posted'{days}{force}{kinds} AND {is_row})"
    );
    (sql, binds)
}

/// The count and sums of the rows of a filter (a type filter's totals).
///
/// # Errors
/// Any SQLite error.
pub fn filtered_sums(conn: &Connection, filter: &Filter<'_>) -> rusqlite::Result<RowSums> {
    let (cte, binds) = rows_cte(filter);
    conn.query_row(
        &format!(
            "{cte} SELECT COUNT(*), COALESCE(SUM(r.in_p), 0), COALESCE(SUM(r.out_p), 0) FROM r"
        ),
        params_from_iter(binds.iter()),
        |r| {
            Ok(RowSums {
                rows: r.get(0)?,
                in_paisa: r.get(1)?,
                out_paisa: r.get(2)?,
            })
        },
    )
}

/// The count and sums of each kind of the rows of a filter.
///
/// # Errors
/// Any SQLite error.
pub fn sums_by_kind(
    conn: &Connection,
    filter: &Filter<'_>,
) -> rusqlite::Result<Vec<(String, RowSums)>> {
    let (cte, binds) = rows_cte(filter);
    let mut statement = conn.prepare(&format!(
        "{cte} SELECT r.kind, COUNT(*), COALESCE(SUM(r.in_p), 0), COALESCE(SUM(r.out_p), 0)
         FROM r GROUP BY r.kind"
    ))?;
    let rows = statement.query_map(params_from_iter(binds.iter()), |r| {
        Ok((
            r.get(0)?,
            RowSums {
                rows: r.get(1)?,
                in_paisa: r.get(2)?,
                out_paisa: r.get(3)?,
            },
        ))
    })?;
    rows.collect()
}

fn overflow() -> rusqlite::Error {
    rusqlite::Error::IntegralValueOutOfRange(0, i64::MAX)
}

/// Σ in and Σ out of the cash entries of the set in a range whose (day, id) is on one side of
/// `key`: `later` true is after it, false before it.
fn sums_beside(
    conn: &Connection,
    filter: &Filter<'_>,
    key: (&str, i64),
    later: bool,
) -> rusqlite::Result<(i64, i64)> {
    let (days, mut binds) = day_conditions(filter.from, filter.to);
    let force = force_sql(filter.show_reversed);
    let op = if later { ">" } else { "<" };
    let mut all = vec![Value::Integer(filter.cash_id)];
    all.append(&mut binds);
    all.push(Value::Text(key.0.to_owned()));
    all.push(Value::Text(key.0.to_owned()));
    all.push(Value::Integer(key.1));
    conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
             JOIN journal_lines l ON l.entry_id = e.id AND l.account_id = ?
             WHERE e.status = 'posted'{days}{force}
               AND (e.entry_date {op} ? OR (e.entry_date = ? AND e.id {op} ?))"
        ),
        params_from_iter(all.iter()),
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
}

/// One key of the walk: the row, its day and what it moved.
struct Walked {
    key: RowKey,
    entry_date: String,
    in_paisa: i64,
    out_paisa: i64,
}

/// The cash received and paid out of one entry, both sides.
fn entry_cash(conn: &Connection, cash_id: i64, entry_id: i64) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0) FROM journal_lines
         WHERE account_id = ?1 AND entry_id = ?2",
        [cash_id, entry_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
}

fn add2(a: (i64, i64), b: (i64, i64)) -> rusqlite::Result<(i64, i64)> {
    Ok((
        a.0.checked_add(b.0).ok_or_else(overflow)?,
        a.1.checked_add(b.1).ok_or_else(overflow)?,
    ))
}

/// One page of the rows in day order, with the running sums unless a type filter is set. See the
/// module doc.
fn page_by_date(
    conn: &Connection,
    filter: &Filter<'_>,
    set: RowSums,
    desc: bool,
    (offset, limit): (i64, i64),
) -> rusqlite::Result<Vec<PageKey>> {
    let cash = filter.cash_id;
    let (days, mut binds) = day_conditions(filter.from, filter.to);
    let force = force_sql(filter.show_reversed);
    let (kinds, kind_binds) = kinds_sql(cash, filter.kinds);
    binds.extend(kind_binds);
    let direction = if desc { "DESC" } else { "ASC" };
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    let (is_row, in_p, out_p) = (is_row_sql(cash, filter.split), in_sql(cash), out_sql(cash));
    let mut statement = conn.prepare(&format!(
        "SELECT e.id, e.entry_date, p.part, {in_p}, {out_p}
         FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
         CROSS JOIN {PARTS} AS p
         WHERE e.status = 'posted'{days}{force}{kinds} AND {is_row}
         ORDER BY e.entry_date {direction}, e.id {direction}, p.part {direction} LIMIT ? OFFSET ?"
    ))?;
    let walked: Vec<Walked> = statement
        .query_map(params_from_iter(binds.iter()), |r| {
            Ok(Walked {
                key: RowKey {
                    entry_id: r.get(0)?,
                    part: r.get(2)?,
                },
                entry_date: r.get(1)?,
                in_paisa: r.get(3)?,
                out_paisa: r.get(4)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    // With a type filter the running cash is not given (PRD FR-16.14 v1.60).
    let unrun = |rows: &[Walked]| {
        rows.iter()
            .map(|w| PageKey {
                key: w.key,
                run_in_paisa: None,
                run_out_paisa: None,
            })
            .collect()
    };
    if !filter.kinds.is_empty() {
        return Ok(unrun(&walked));
    }
    let (Some(first), Some(last)) = (walked.first(), walked.last()) else {
        return Ok(Vec::new());
    };
    let page_sums = walked.iter().try_fold((0_i64, 0_i64), |sum, w| {
        add2(sum, (w.in_paisa, w.out_paisa))
    })?;
    // `walked_before` is the sums of the rows the walk passed before the page: the entries beside
    // the first one, and the other part of the first entry when the walk passed it already.
    let walked_before: (i64, i64) = if offset <= set.rows / 2 {
        // `desc` walks from the newest, so what is before the page is what is later in day order.
        let beside = sums_beside(conn, filter, (&first.entry_date, first.key.entry_id), desc)?;
        let passed = match (desc, first.key.part) {
            (false, 2) => (entry_cash(conn, cash, first.key.entry_id)?.0, 0),
            (true, 1) => (0, entry_cash(conn, cash, first.key.entry_id)?.1),
            _ => (0, 0),
        };
        add2(beside, passed)?
    } else {
        let beside = sums_beside(conn, filter, (&last.entry_date, last.key.entry_id), !desc)?;
        let still = match (desc, last.key.part) {
            (false, 1) => (0, entry_cash(conn, cash, last.key.entry_id)?.1),
            (true, 2) => (entry_cash(conn, cash, last.key.entry_id)?.0, 0),
            _ => (0, 0),
        };
        let after = add2(beside, still)?;
        (
            set.in_paisa
                .checked_sub(page_sums.0)
                .and_then(|v| v.checked_sub(after.0))
                .ok_or_else(overflow)?,
            set.out_paisa
                .checked_sub(page_sums.1)
                .and_then(|v| v.checked_sub(after.1))
                .ok_or_else(overflow)?,
        )
    };
    // The running sums in day order. Oldest first: the walked rows are the earlier ones. Newest
    // first: the first row has everything except what was walked before it.
    let (mut run_in, mut run_out) = if desc {
        (
            set.in_paisa
                .checked_sub(walked_before.0)
                .ok_or_else(overflow)?,
            set.out_paisa
                .checked_sub(walked_before.1)
                .ok_or_else(overflow)?,
        )
    } else {
        walked_before
    };
    let mut keys = Vec::with_capacity(walked.len());
    for w in &walked {
        let (shown_in, shown_out) = if desc {
            // This row's running sums include itself; the next (older) row takes it off.
            let shown = (run_in, run_out);
            run_in = run_in.checked_sub(w.in_paisa).ok_or_else(overflow)?;
            run_out = run_out.checked_sub(w.out_paisa).ok_or_else(overflow)?;
            shown
        } else {
            run_in = run_in.checked_add(w.in_paisa).ok_or_else(overflow)?;
            run_out = run_out.checked_add(w.out_paisa).ok_or_else(overflow)?;
            (run_in, run_out)
        };
        keys.push(PageKey {
            key: w.key,
            run_in_paisa: Some(shown_in),
            run_out_paisa: Some(shown_out),
        });
    }
    Ok(keys)
}

/// One page of the rows sorted by another column or by amount. This reads every row of the range.
fn page_sorted(
    conn: &Connection,
    filter: &Filter<'_>,
    order: &str,
    (offset, limit): (i64, i64),
) -> rusqlite::Result<Vec<PageKey>> {
    let (cte, mut binds) = rows_cte(filter);
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    let mut statement = conn.prepare(&format!(
        "{cte} SELECT r.entry_id, r.part FROM r
         ORDER BY {order}, r.entry_date, r.entry_id, r.part LIMIT ? OFFSET ?"
    ))?;
    let keys = statement.query_map(params_from_iter(binds.iter()), |r| {
        Ok(PageKey {
            key: RowKey {
                entry_id: r.get(0)?,
                part: r.get(1)?,
            },
            run_in_paisa: None,
            run_out_paisa: None,
        })
    })?;
    keys.collect()
}

/// One page of the rows of the filter in the order asked for. Only [`Order::Date`] without a type
/// filter has the running sums. `set` is the sums of the rows shown (in force, or all with
/// `show_reversed`), which the running sums are counted from.
///
/// # Errors
/// Any SQLite error.
pub fn page(
    conn: &Connection,
    filter: &Filter<'_>,
    set: RowSums,
    order: &Order,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<Vec<PageKey>> {
    match order {
        Order::Date { desc } => page_by_date(conn, filter, set, *desc, (offset, limit)),
        Order::Amount { out, desc } => {
            let column = if *out { "r.out_p" } else { "r.in_p" };
            let direction = if *desc { "DESC" } else { "ASC" };
            page_sorted(
                conn,
                filter,
                &format!("{column} {direction}"),
                (offset, limit),
            )
        }
        Order::Other(sql) => page_sorted(conn, filter, sql, (offset, limit)),
    }
}

/// What each row of a page shows, in no particular order (one query).
///
/// # Errors
/// Any SQLite error.
pub fn rows(
    conn: &Connection,
    cash_id: i64,
    keys: &[RowKey],
) -> rusqlite::Result<HashMap<RowKey, CashRow>> {
    let mut out = HashMap::new();
    for chunk in keys.chunks(500) {
        rows_chunk(conn, cash_id, chunk, &mut out)?;
    }
    Ok(out)
}

/// [`rows`] for one chunk of keys.
fn rows_chunk(
    conn: &Connection,
    cash_id: i64,
    keys: &[RowKey],
    out: &mut HashMap<RowKey, CashRow>,
) -> rusqlite::Result<()> {
    let marks = vec!["(?, ?)"; keys.len()].join(", ");
    let (venture_id, venture_kind) = (row_source_sql::VENTURE_ID, row_source_sql::VENTURE_KIND);
    let (kind, in_p, out_p) = (kind_sql(cash_id), in_sql(cash_id), out_sql(cash_id));
    let reason = voucher_of("reason");
    let mut statement = conn.prepare(&format!(
        "WITH k(entry_id, part) AS (VALUES {marks}),
         x AS (
             SELECT e.id AS entry_id, p.part AS part, e.entry_date AS entry_date,
                    e.posting_type AS posting_type, {kind} AS kind,
                    CASE WHEN e.reverses_entry_id IS NOT NULL THEN 'reversal'
                         WHEN NOT ({IN_FORCE}) THEN 'reversed' ELSE 'live' END AS status,
                    {in_p} AS in_p, {out_p} AS out_p,
                    CASE e.source_type
                        WHEN 'voucher' THEN {voucher_number}
                        WHEN 'bikri' THEN (SELECT b.number FROM bikri_entries b WHERE b.id = e.source_id)
                        WHEN 'purchase' THEN (SELECT pu.number FROM purchases pu WHERE pu.id = e.source_id)
                        WHEN 'venture' THEN (SELECT vn.number FROM venture_entries vn WHERE vn.id = e.source_id)
                        WHEN 'fruit' THEN (SELECT ft.number FROM fruit_takes ft WHERE ft.id = e.source_id)
                    END AS number,
                    COALESCE(e.narration, '') AS narration, COALESCE({reason}, '') AS reason,
                    e.source_type AS source_type, e.source_id AS source_id,
                    {venture_id} AS venture_id, {venture_kind} AS venture_kind,
                    CASE e.source_type
                        WHEN 'venture' THEN {venture_id}
                        WHEN 'purchase' THEN (SELECT pv.venture_id FROM purchases pv WHERE pv.id = e.source_id)
                    END AS named_venture
             FROM k AS p JOIN journal_entries e ON e.id = p.entry_id)
         SELECT x.entry_id, x.part, x.entry_date, x.posting_type, x.kind, x.status, x.in_p, x.out_p,
                x.number, x.narration, x.reason, x.source_type, x.source_id, x.venture_id,
                x.venture_kind, vt.id, vt.kind, vt.number, vt.name_ur, vt.name_en
         FROM x LEFT JOIN ventures vt ON vt.id = x.named_venture",
        voucher_number = voucher_of("number"),
    ))?;
    let values: Vec<i64> = keys.iter().flat_map(|k| [k.entry_id, k.part]).collect();
    let found = statement.query_map(params_from_iter(values.iter()), |r| {
        let venture = match r.get::<_, Option<i64>>(15)? {
            Some(id) => Some(VentureName {
                id,
                kind: r.get(16)?,
                number: r.get(17)?,
                name_ur: r.get(18)?,
                name_en: r.get(19)?,
            }),
            None => None,
        };
        Ok(CashRow {
            key: RowKey {
                entry_id: r.get(0)?,
                part: r.get(1)?,
            },
            entry_date: r.get(2)?,
            posting_type: r.get(3)?,
            kind: r.get(4)?,
            status: r.get(5)?,
            in_paisa: r.get(6)?,
            out_paisa: r.get(7)?,
            number: r.get(8)?,
            narration: r.get(9)?,
            reason: r.get(10)?,
            source_type: r.get(11)?,
            source_id: r.get(12)?,
            venture_id: r.get(13)?,
            venture_kind: r.get(14)?,
            venture,
        })
    })?;
    for row in found {
        let row = row?;
        out.insert(row.key, row);
    }
    Ok(())
}

/// A name that a line of an entry points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    /// Row id.
    pub id: i64,
    /// Its code (empty when it has none).
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Its tag, if any.
    pub tag: Option<String>,
}

/// One line of an entry of a page, with what the screen names it by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryLine {
    /// The entry.
    pub entry_id: i64,
    /// Its line number.
    pub line_no: i64,
    /// The account, by its own names. A چکتا line carries the names of its person (with the code
    /// of the account 0030).
    pub account: Named,
    /// The code of the account's control (`0602`, `0605`...).
    pub control_code: String,
    /// `seller`, `buyer`, ... from the account.
    pub party_type: String,
    /// Debit, paisa.
    pub dr_paisa: i64,
    /// Credit, paisa.
    pub cr_paisa: i64,
    /// The partner whose capital or drawings account this is.
    pub partner: Option<Named>,
    /// The expense head this account is.
    pub expense_head: Option<Named>,
}

/// Every line of these entries, in entry and line order (one query).
///
/// # Errors
/// Any SQLite error.
pub fn entry_lines(
    conn: &Connection,
    entry_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, Vec<EntryLine>>> {
    let mut out: HashMap<i64, Vec<EntryLine>> = HashMap::new();
    for chunk in entry_ids.chunks(500) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let mut statement = conn.prepare(&format!(
            "SELECT l.entry_id, l.line_no, a.id, a.code, a.name_ur, a.name_en, a.tag, k.code,
                    a.party_type, l.dr_paisa, l.cr_paisa, c.name_ur, c.name_en,
                    pt.id, pt.name_ur, pt.name_en, eh.id, eh.code, eh.name_ur, eh.name_en
             FROM journal_lines l
             JOIN accounts a ON a.id = l.account_id
             JOIN account_controls k ON k.id = a.control_id
             LEFT JOIN chukta_buyers c ON c.person_key = l.chukta_person_key
             LEFT JOIN partners pt ON k.code IN ('0602', '0603')
                  AND (pt.capital_account_id = a.id OR pt.drawings_account_id = a.id)
             LEFT JOIN expense_heads eh ON eh.account_id = a.id
             WHERE l.entry_id IN ({marks})
             ORDER BY l.entry_id, l.line_no"
        ))?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            let mut account = Named {
                id: r.get(2)?,
                code: r.get(3)?,
                name_ur: r.get(4)?,
                name_en: r.get(5)?,
                tag: r.get(6)?,
            };
            // A چکتا line shows its person's names.
            if let (Some(name_ur), Some(name_en)) = (
                r.get::<_, Option<String>>(11)?,
                r.get::<_, Option<String>>(12)?,
            ) {
                account.name_ur = name_ur;
                account.name_en = name_en;
                account.tag = None;
            }
            let partner = match r.get::<_, Option<i64>>(13)? {
                Some(id) => Some(Named {
                    id,
                    code: String::new(),
                    name_ur: r.get(14)?,
                    name_en: r.get(15)?,
                    tag: None,
                }),
                None => None,
            };
            let expense_head = match r.get::<_, Option<i64>>(16)? {
                Some(id) => Some(Named {
                    id,
                    code: r.get(17)?,
                    name_ur: r.get(18)?,
                    name_en: r.get(19)?,
                    tag: None,
                }),
                None => None,
            };
            Ok(EntryLine {
                entry_id: r.get(0)?,
                line_no: r.get(1)?,
                account,
                control_code: r.get(7)?,
                party_type: r.get(8)?,
                dr_paisa: r.get(9)?,
                cr_paisa: r.get(10)?,
                partner,
                expense_head,
            })
        })?;
        for row in rows {
            let row = row?;
            out.entry(row.entry_id).or_default().push(row);
        }
    }
    Ok(out)
}

/// Σ debit and Σ credit of the lines on the capital accounts (the partners' under 0602 and the
/// firm's ابتدائی سرمایہ 06010001) of the openings, capital put in and capital taken back that are
/// in force, dated on or before `up_to` (`None` is every day): the card کل سرمایہ (PRD FR-16.14
/// v1.60).
///
/// # Errors
/// Any SQLite error.
pub fn capital_sums(conn: &Connection, up_to: Option<&str>) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             FROM accounts a
             JOIN account_controls k ON k.id = a.control_id
             JOIN journal_lines l ON l.account_id = a.id
             JOIN journal_entries e ON e.id = l.entry_id
             WHERE (k.code = '0602' OR a.code = '06010001')
               AND e.status = 'posted' AND e.posting_type IN ('opening', 'capital_in', 'capital_out')
               AND {IN_FORCE} AND (?1 IS NULL OR e.entry_date <= ?1)"
        ),
        [up_to],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
}

/// The count of rows and the cash in and out of each month from the month of `first_day` to the
/// month of `last_day` (both `YYYY-MM-DD`), clipped to the range of the filter, the rows in force
/// (or of the type filter): one query for each month, each read through the day index. These are
/// the months a page touches (plan 38 B2).
///
/// # Errors
/// Any SQLite error.
pub fn months_between(
    conn: &Connection,
    filter: &Filter<'_>,
    (first_day, last_day): (&str, &str),
) -> rusqlite::Result<Vec<MonthSums>> {
    let (low, high) = if first_day <= last_day {
        (first_day, last_day)
    } else {
        (last_day, first_day)
    };
    let (Some(start), Some(end)) = (low.get(..7), high.get(..7)) else {
        return Ok(Vec::new());
    };
    let parse = |m: &str| -> Option<(i32, u32)> {
        Some((m.get(..4)?.parse().ok()?, m.get(5..7)?.parse().ok()?))
    };
    let (Some((mut year, mut month)), Some(last)) = (parse(start), parse(end)) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    // A page cannot touch more months than this (a 100-row page of a sparse book).
    for _ in 0..240 {
        let key = format!("{year:04}-{month:02}");
        let lo = format!("{key}-01");
        let hi = format!("{key}-31");
        let lo = filter.from.map_or(
            lo.clone(),
            |f| if f > lo.as_str() { f.to_owned() } else { lo },
        );
        let hi = filter.to.map_or(
            hi.clone(),
            |t| if t < hi.as_str() { t.to_owned() } else { hi },
        );
        let month_filter = Filter {
            from: Some(&lo),
            to: Some(&hi),
            show_reversed: false,
            split: true,
            ..*filter
        };
        let sums = if filter.kinds.is_empty() {
            range_sums(conn, filter.cash_id, Some(&lo), Some(&hi))?.live
        } else {
            filtered_sums(conn, &month_filter)?
        };
        // A month with no row gets no heading, as when the months were grouped from the rows.
        if sums.rows > 0 {
            out.push(MonthSums {
                month: key,
                count: sums.rows,
                sums: vec![sums.in_paisa, sums.out_paisa],
            });
        }
        if (year, month) >= last {
            break;
        }
        if month == 12 {
            year += 1;
            month = 1;
        } else {
            month += 1;
        }
    }
    Ok(out)
}
