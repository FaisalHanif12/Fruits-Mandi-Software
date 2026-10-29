//! The shop's commission on the goods of trips and gardens that were sold in OUR shop (plan 36,
//! PRD FR-12.27, FR-12.37).
//!
//! It is seller commission + buyer commission + لاگا of the stored sale lines (the rows of
//! `analytics_facts`, one for each sale line, each with a sign: +1, or −1 for a reversal), summed
//! with that sign, so a cancelled or edited sale takes itself out (PRD FR-18.6, Code Standards
//! B20). A sale in another shop is not a bikri sale line and adds nothing.
//!
//! This is the definition of analytics views J and I and of report R36 (their column "commission
//! on its goods"): the sale facts of the purchase (`purchase_id`) or of the garden's goods (the
//! lots whose seller is the garden's account). It is NOT the seller-only commission column of R33.
//! Example (E2E-34): 540 + 540 + 120 = 1,200. (E2E-41: 6,720 + 6,720 + 640 = 14,080. E2E-40:
//! 3,600 + 3,600 + 800 = 8,000.)
//!
//! Every read is one grouped query through an index of `analytics_facts` (`purchase_id`,
//! `seller_account_id` is read through `idx_analytics_facts_seller_date`, `lot_id`): never a query
//! in a loop over rows (Code Standards B22, B23). The three kinds are summed apart so the working
//! can show them; the total is their checked sum.

use std::collections::HashMap;

use rusqlite::{Connection, params_from_iter};

/// How many ids go into one `IN (...)`.
const CHUNK: usize = 400;

/// The commission on some goods, by kind, in paisa: the sums of the sale facts with their sign.
/// Whole rupees (a multiple of 100) because every stored amount is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Commission {
    /// Seller commission.
    pub seller_paisa: i64,
    /// Buyer commission.
    pub buyer_paisa: i64,
    /// لاگا.
    pub laga_paisa: i64,
}

impl Commission {
    /// Seller commission + buyer commission + لاگا, or `None` if the sum does not fit.
    pub fn total_paisa(&self) -> Option<i64> {
        self.seller_paisa
            .checked_add(self.buyer_paisa)?
            .checked_add(self.laga_paisa)
    }

    /// Adds another commission, or `None` if a sum does not fit.
    pub fn checked_add(&self, other: &Commission) -> Option<Commission> {
        Some(Commission {
            seller_paisa: self.seller_paisa.checked_add(other.seller_paisa)?,
            buyer_paisa: self.buyer_paisa.checked_add(other.buyer_paisa)?,
            laga_paisa: self.laga_paisa.checked_add(other.laga_paisa)?,
        })
    }
}

/// Reads one grouped query for ids in chunks. `sql_for` gets the `?, ?, ...` list for a chunk and
/// returns a query whose rows are the id, then the six [`SUMS`].
fn grouped(
    conn: &Connection,
    ids: &[i64],
    sql_for: impl Fn(&str) -> String,
) -> rusqlite::Result<HashMap<i64, Commission>> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(CHUNK) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let mut statement = conn.prepare(&sql_for(&marks))?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((r.get::<_, i64>(0)?, net(r, 1)?))
        })?;
        for row in rows {
            let (id, commission) = row?;
            out.insert(id, commission);
        }
    }
    Ok(out)
}

/// The six sums of the facts `f`: seller, buyer and لاگا with sign +1, then with sign −1, summed
/// apart (SUM of an INTEGER `*_paisa` column only) and subtracted in Rust with checked maths
/// (Code Standards M5, B20), as report R36 does.
const SUMS: &str = "COALESCE(SUM(CASE WHEN f.sign > 0 THEN f.seller_comm_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign < 0 THEN f.seller_comm_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign > 0 THEN f.buyer_comm_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign < 0 THEN f.buyer_comm_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign > 0 THEN f.laga_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign < 0 THEN f.laga_paisa END), 0)";

/// Reads the six sums that start at column `first` and nets them.
fn net(r: &rusqlite::Row<'_>, first: usize) -> rusqlite::Result<Commission> {
    let mut sums = [0_i64; 6];
    for (index, slot) in sums.iter_mut().enumerate() {
        *slot = r.get(first + index)?;
    }
    let [sp, sm, bp, bm, lp, lm] = sums;
    let minus = |plus: i64, minus: i64, column: usize| {
        plus.checked_sub(minus)
            .ok_or(rusqlite::Error::IntegralValueOutOfRange(column, plus))
    };
    Ok(Commission {
        seller_paisa: minus(sp, sm, first)?,
        buyer_paisa: minus(bp, bm, first + 2)?,
        laga_paisa: minus(lp, lm, first + 4)?,
    })
}

/// The commission on each purchase's goods, by purchase id (analytics view J, by purchase). A
/// purchase with no sale in our shop is not in the map.
///
/// # Errors
/// Any SQLite error.
pub fn by_purchases(
    conn: &Connection,
    purchase_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, Commission>> {
    grouped(conn, purchase_ids, |marks| {
        format!(
            "SELECT f.purchase_id, {SUMS} FROM analytics_facts f
             WHERE f.purchase_id IN ({marks}) GROUP BY f.purchase_id"
        )
    })
}

/// The commission on each trip's goods, by trip id: the sum over its purchases (analytics view J,
/// by trip).
///
/// # Errors
/// Any SQLite error.
pub fn by_trips(conn: &Connection, trip_ids: &[i64]) -> rusqlite::Result<HashMap<i64, Commission>> {
    grouped(conn, trip_ids, |marks| {
        format!(
            "SELECT p.venture_id, {SUMS}
             FROM purchases p JOIN analytics_facts f ON f.purchase_id = p.id
             WHERE p.venture_id IN ({marks}) GROUP BY p.venture_id"
        )
    })
}

/// The commission on each garden's goods, by the garden's account id: the lots whose seller is the
/// garden's account, over the whole life of the garden (analytics view I, report R36).
///
/// # Errors
/// Any SQLite error.
pub fn by_garden_accounts(
    conn: &Connection,
    account_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, Commission>> {
    grouped(conn, account_ids, |marks| {
        format!(
            "SELECT f.seller_account_id, {SUMS} FROM analytics_facts f
             WHERE f.seller_account_id IN ({marks}) GROUP BY f.seller_account_id"
        )
    })
}

/// The commission on the goods sold from each arrival (مال آیا), by arrival id: the facts of the
/// lots linked to it. Goods of a garden sold in a lot with no arrival are in the garden's figure
/// and in no arrival's.
///
/// # Errors
/// Any SQLite error.
pub fn by_arrivals(
    conn: &Connection,
    arrival_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, Commission>> {
    grouped(conn, arrival_ids, |marks| {
        format!(
            "SELECT l.arrival_id, {SUMS}
             FROM bikri_lots l JOIN analytics_facts f ON f.lot_id = l.id
             WHERE l.arrival_id IN ({marks}) GROUP BY l.arrival_id"
        )
    })
}

/// The commission on the goods of the purchases that match a filter's `WHERE` (the same text the
/// purchase totals use, with `p` the purchases): one query for the total row of a list.
///
/// # Errors
/// Any SQLite error.
pub fn of_purchases_where(
    conn: &Connection,
    where_sql: &str,
    args: impl rusqlite::Params,
) -> rusqlite::Result<Commission> {
    conn.query_row(
        &format!(
            "SELECT {SUMS} FROM purchases p JOIN analytics_facts f ON f.purchase_id = p.id
             {where_sql} AND p.status = 'saved'"
        ),
        args,
        |r| net(r, 0),
    )
}

/// The commission on the goods sold from the arrivals that match a filter's `WHERE` (with `a` the
/// arrivals), through the lots linked to them: one query for the total row of a list.
///
/// # Errors
/// Any SQLite error.
pub fn of_arrivals_where(
    conn: &Connection,
    where_sql: &str,
    args: impl rusqlite::Params,
) -> rusqlite::Result<Commission> {
    conn.query_row(
        &format!(
            "SELECT {SUMS} FROM garden_arrivals a
             JOIN bikri_lots l ON l.arrival_id = a.id
             JOIN analytics_facts f ON f.lot_id = l.id
             {where_sql} AND a.status = 'saved'"
        ),
        args,
        |r| net(r, 0),
    )
}
