//! Advances to sellers in the partners' totals, read from the books (PRD FR-17.13, plan 78 B4).
//!
//! The pure work is `mandi_core::advances::replay`. This module reads what it needs and nothing is
//! stored: the live advance lines (a seller debit flagged `is_advance` whose entry is not reversed),
//! the shares of each advance's day, the ACTIVE rows of `advance_allocations` (PRD FR-15.8), and
//! every live entry of each seller with an advance, in one grouped query, so the part his own credit
//! covers is known after each entry (PRD FR-17.13 v1.61). Because the matching rows are the
//! canonical ones, a re-match after an edit or a cancel changes the events with it.
//!
//! The shares of an advance are those of its own day. When no share was set by then (an opening
//! advance, or an advance made before the partners were set up) the first shares ever set are used
//! [A, PRD FR-17.13]. When no share was ever set the advance has no parts.

use std::collections::HashMap;

use mandi_core::accounts::RecordRef;
use mandi_core::advances::{
    AdvanceError, AdvanceEvent, AdvanceEventKind, AdvanceLineIn, SellerStepIn, SettlementIn,
    partner_net, replay,
};
use mandi_core::money::{Bps, MoneyError, Paisa};
use rusqlite::{Connection, params};

use crate::matching::{LIVE, ORDER};
use crate::partners::shares_on;

/// Why the advance events could not be read.
#[derive(Debug)]
pub enum AdvanceReadError {
    /// A database error.
    Db(rusqlite::Error),
    /// The events could not be worked out (shares not 100%, matching rows that settle too much).
    Advance(AdvanceError),
    /// A stored value is not what it should be (a share outside 0 to 10,000, an amount that is not
    /// whole rupees).
    Data(&'static str),
}

impl From<rusqlite::Error> for AdvanceReadError {
    fn from(error: rusqlite::Error) -> Self {
        AdvanceReadError::Db(error)
    }
}

impl From<AdvanceError> for AdvanceReadError {
    fn from(error: AdvanceError) -> Self {
        AdvanceReadError::Advance(error)
    }
}

impl From<MoneyError> for AdvanceReadError {
    fn from(error: MoneyError) -> Self {
        AdvanceReadError::Advance(AdvanceError::Money(error))
    }
}

/// Every event of every advance, with the seller of each.
#[derive(Debug, Clone, Default)]
pub struct Advances {
    /// The events, advance by advance.
    pub events: Vec<AdvanceEvent>,
    /// The seller of each advance line, by line id, as the statement words it.
    pub sellers: HashMap<i64, RecordRef>,
}

impl Advances {
    /// What a partner's parts of the advances still have out, from `from` to `to` (both included,
    /// `None` leaves an end open): taken off his کل رقم.
    ///
    /// # Errors
    /// `Overflow`.
    pub fn net(
        &self,
        partner_id: i64,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Paisa, MoneyError> {
        partner_net(&self.events, partner_id, from, to)
    }
}

fn whole(value: i64) -> Result<Paisa, AdvanceReadError> {
    Paisa::try_from_paisa(value)
        .map_err(|_| AdvanceReadError::Data("an advance is not whole rupees"))
}

fn shares_for(
    conn: &Connection,
    date: &str,
    cache: &mut HashMap<String, Vec<(i64, Bps)>>,
    first: &mut Option<Vec<(i64, Bps)>>,
) -> Result<Vec<(i64, Bps)>, AdvanceReadError> {
    if let Some(found) = cache.get(date) {
        return Ok(found.clone());
    }
    let to_bps = |rows: Vec<(i64, i64)>| -> Result<Vec<(i64, Bps)>, AdvanceReadError> {
        rows.into_iter()
            .map(|(id, bps)| {
                i32::try_from(bps)
                    .ok()
                    .and_then(|b| Bps::new(b).ok())
                    .map(|b| (id, b))
                    .ok_or(AdvanceReadError::Data("a stored share is not 0 to 10,000"))
            })
            .collect()
    };
    let mut shares = to_bps(shares_on(conn, date)?)?;
    if shares.is_empty() {
        // No share by that day: the first shares ever set.
        if first.is_none() {
            let day: Option<String> =
                conn.query_row("SELECT MIN(effective_from) FROM partner_shares", [], |r| {
                    r.get(0)
                })?;
            *first = Some(match day {
                Some(day) => to_bps(shares_on(conn, &day)?)?,
                None => Vec::new(),
            });
        }
        shares = first.clone().unwrap_or_default();
    }
    cache.insert(date.to_owned(), shares.clone());
    Ok(shares)
}

/// Reads and replays every advance in the books.
///
/// # Errors
/// A database error; `Advance(Money(SharesNot100))` when the shares of an advance's day do not
/// total 100%; `Advance(OverSettled)` when its matching rows settle more than it.
pub fn read_all(conn: &Connection) -> Result<Advances, AdvanceReadError> {
    let mut lines: Vec<AdvanceLineIn> = Vec::new();
    let mut sellers: HashMap<i64, RecordRef> = HashMap::new();
    let mut cache: HashMap<String, Vec<(i64, Bps)>> = HashMap::new();
    let mut first: Option<Vec<(i64, Bps)>> = None;
    {
        let mut statement = conn.prepare(&format!(
            "SELECT l.id, e.id, e.entry_date, a.id, a.code, a.name_ur, a.name_en, a.tag, l.dr_paisa
             FROM journal_lines l
             JOIN journal_entries e ON e.id = l.entry_id
             JOIN accounts a ON a.id = l.account_id
             WHERE l.is_advance = 1 AND l.dr_paisa > 0 AND {LIVE}
             ORDER BY {ORDER}"
        ))?;
        let rows = statement.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                RecordRef {
                    id: r.get(3)?,
                    code: r.get(4)?,
                    name_ur: r.get(5)?,
                    name_en: r.get(6)?,
                    tag: r.get(7)?,
                },
                r.get::<_, i64>(8)?,
            ))
        })?;
        for row in rows {
            let (line_id, entry_id, date, seller, amount) = row?;
            let shares = shares_for(conn, &date, &mut cache, &mut first)?;
            lines.push(AdvanceLineIn {
                line_id,
                entry_id,
                date,
                seller_account_id: seller.id,
                amount: whole(amount)?,
                shares,
            });
            sellers.insert(line_id, seller);
        }
    }
    let mut settlements: Vec<SettlementIn> = Vec::new();
    {
        let mut statement = conn.prepare(
            "SELECT a.id, a.advance_line_id, ce.id, cl.line_no, a.amount_paisa
             FROM advance_allocations a
             JOIN journal_lines cl ON cl.id = a.credit_line_id
             JOIN journal_entries ce ON ce.id = cl.entry_id
             WHERE a.is_active = 1 AND ce.status = 'posted'
             ORDER BY a.id",
        )?;
        let rows = statement.query_map(params![], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })?;
        for row in rows {
            let (allocation_id, advance_line_id, credit_entry_id, line_no, amount) = row?;
            // Only the rows of a live advance line count (a reversed advance has no events).
            if !sellers.contains_key(&advance_line_id) {
                continue;
            }
            settlements.push(SettlementIn {
                allocation_id,
                advance_line_id,
                credit_entry_id,
                credit_line_no: line_no,
                amount: whole(amount)?,
            });
        }
    }
    let events = replay(&lines, &settlements, &seller_steps(conn)?)?;
    Ok(Advances { events, sellers })
}

/// Every live entry of each seller who has an advance line, seller by seller and in the order of
/// his ledger, with what it moves his ledger by (credits less debits). One grouped query.
fn seller_steps(conn: &Connection) -> Result<Vec<SellerStepIn>, AdvanceReadError> {
    let mut statement = conn.prepare(&format!(
        "SELECT l.account_id, e.id, e.entry_date, SUM(l.cr_paisa), SUM(l.dr_paisa)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id IN (SELECT account_id FROM journal_lines
                                WHERE is_advance = 1 AND dr_paisa > 0)
           AND {LIVE}
         GROUP BY l.account_id, e.id
         ORDER BY l.account_id, e.entry_date, (e.posting_type <> 'opening'),
                  COALESCE(e.order_key, e.id), e.id"
    ))?;
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, i64>(4)?,
        ))
    })?;
    let mut steps = Vec::new();
    for row in rows {
        let (seller_account_id, entry_id, date, credit, debit) = row?;
        steps.push(SellerStepIn {
            seller_account_id,
            entry_id,
            date,
            net_credit: whole(credit)?.checked_sub(whole(debit)?)?,
        });
    }
    Ok(steps)
}

/// What the advances of every partner still have out at the end of `as_of` (or now): Σ of his
/// parts of the uncovered advances, dead sellers included. The books balance with it (PRD FR-17.9
/// v1.61): Σ کل رقم + ابتدائی سرمایہ + this = equity.
///
/// # Errors
/// As [`read_all`].
pub fn open_parts_total(conn: &Connection, as_of: Option<&str>) -> Result<Paisa, AdvanceReadError> {
    Ok(read_all(conn)?.open_parts_total(as_of)?)
}

impl Advances {
    /// What the advances of every partner still have out at the end of `as_of` (or now), from the
    /// events already read: Σ out less Σ back.
    ///
    /// # Errors
    /// `Overflow`.
    pub fn open_parts_total(&self, as_of: Option<&str>) -> Result<Paisa, MoneyError> {
        let mut sum = Paisa::ZERO;
        for event in &self.events {
            if as_of.is_some_and(|d| event.date.as_str() > d) {
                continue;
            }
            sum = match event.kind {
                AdvanceEventKind::Out => sum.checked_add(event.part)?,
                AdvanceEventKind::Back => sum.checked_sub(event.part)?,
            };
        }
        Ok(sum)
    }
}
