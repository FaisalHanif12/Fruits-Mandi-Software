//! Reads and writes of the matching rows (PRD FR-15.4, FR-15.5, FR-15.8, plan 13): which bills
//! and credits of an account (or one person of a پختہ کھاتہ account) are still open, and the
//! rows `bill_allocations` and `advance_allocations`. SQL only: the oldest-first working is the
//! pure `mandi_domain::matching::match_bills`, and the services decide what to match.
//!
//! A line takes part only while it is **live**: a line of a posted entry that is not reversed and
//! is not itself a reversal. A reversed entry and its reversal cancel each other, so neither is a
//! bill or a credit any more (money agent 8.3).
//!
//! Money is never added in SQL except with `SUM()` on `INTEGER` columns (money agent Section 9):
//! each query hands back the line amount and the allocated amount, and the services subtract
//! them with checked maths. Rows are never deleted; an undo sets `is_active = 0` (Code Standards
//! B17).

use rusqlite::{Connection, params};

/// The place of a line in the order of its account, oldest first: day, not an opening, order key
/// of its entry, entry id, line number.
type AccountOrder = (String, bool, i64, i64, i64);

/// The live-entry condition on `journal_entries e`.
pub(crate) const LIVE: &str = "e.status = 'posted' AND e.reverses_entry_id IS NULL
    AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id AND r.status = 'posted')";

/// The order of the lines of an account, oldest first: by date, an opening balance before the
/// other entries of its day (it is the balance the day starts with, so a corrected opening that
/// was saved later is still the oldest line of its day), then by the order key, then by the order
/// of saving. The order key (migration 0005) is the entry's own id unless an edit gave the new
/// posting the key of the posting it replaced, so an edited posting keeps its place in its day
/// and does not move behind a بکری saved later the same day (plan 16, money agent finding 1 of
/// plan 15). One order for every matching query, so the canonical working and the fast paths
/// always agree (PRD FR-10.9, FR-15.4, FR-15.8).
pub(crate) const ORDER: &str =
    "e.entry_date, (e.posting_type <> 'opening'), COALESCE(e.order_key, e.id), e.id, l.line_no";

/// A line with the part of it that is already matched, in paisa. The services work out what is
/// still open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineMatch {
    /// The journal line.
    pub line_id: i64,
    /// Its amount (`dr_paisa` for a bill or an advance, `cr_paisa` for a credit).
    pub amount_paisa: i64,
    /// The sum of its active matching rows.
    pub matched_paisa: i64,
}

fn read(r: &rusqlite::Row<'_>) -> rusqlite::Result<LineMatch> {
    Ok(LineMatch {
        line_id: r.get(0)?,
        amount_paisa: r.get(1)?,
        matched_paisa: r.get(2)?,
    })
}

/// A live line of a buyer key with its two amounts, for the canonical matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLine {
    /// The journal line.
    pub line_id: i64,
    /// Debit, paisa (0 on a credit line).
    pub dr_paisa: i64,
    /// Credit, paisa (0 on a debit line).
    pub cr_paisa: i64,
    /// Its journal entry.
    pub entry_id: i64,
    /// The day of its entry, `YYYY-MM-DD`.
    pub entry_date: String,
    /// True if its entry is an opening balance, which sorts first on its day.
    pub is_opening: bool,
    /// The order key of its entry: its own id unless an edit gave it the key of the posting it
    /// replaced (migration 0005). Lines of one day are in the order of this key, then entry id.
    pub order_key: i64,
}

fn read_key_line(r: &rusqlite::Row<'_>) -> rusqlite::Result<KeyLine> {
    Ok(KeyLine {
        line_id: r.get(0)?,
        dr_paisa: r.get(1)?,
        cr_paisa: r.get(2)?,
        entry_id: r.get(3)?,
        entry_date: r.get(4)?,
        is_opening: r.get(5)?,
        order_key: r.get(6)?,
    })
}

/// Every live line of an account (or one person of a پختہ کھاتہ account), oldest first by
/// `(entry_date, opening first, order key, entry id, line_no)`: the bills (debits) and the credits the canonical matching
/// works from (PRD FR-15.4).
///
/// # Errors
/// Any SQLite error.
pub fn live_key_lines(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<Vec<KeyLine>> {
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.dr_paisa, l.cr_paisa, e.id, e.entry_date, e.posting_type = 'opening',
                COALESCE(e.order_key, e.id)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.pukhta_person_key IS ?2 AND {LIVE}
         ORDER BY {ORDER}"
    ))?;
    let rows = statement.query_map(params![account_id, person_key], read_key_line)?;
    rows.collect()
}

/// An active bill matching row of a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveRow {
    /// Row id.
    pub id: i64,
    /// The bill (debit line).
    pub bill_line_id: i64,
    /// The credit line.
    pub payment_line_id: i64,
    /// The amount, paisa.
    pub amount_paisa: i64,
    /// True for a pinned pair: both lines belong to the same بکری posting, so it is a نقدی line
    /// settled at once by its own cash (PRD FR-12.9). A pinned row is never changed by a
    /// re-match; only a reversal switches it off.
    pub pinned: bool,
}

/// The active bill rows of a key.
///
/// # Errors
/// Any SQLite error.
pub fn active_bill_rows(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<Vec<ActiveRow>> {
    let mut statement = conn.prepare(
        "SELECT a.id, a.bill_line_id, a.payment_line_id, a.amount_paisa,
                EXISTS (SELECT 1 FROM journal_lines b
                        JOIN journal_lines p ON p.entry_id = b.entry_id
                        JOIN journal_entries e ON e.id = b.entry_id
                        WHERE b.id = a.bill_line_id AND p.id = a.payment_line_id
                          AND e.posting_type IN ('bikri_first', 'bikri_later'))
         FROM bill_allocations a
         WHERE a.account_id = ?1 AND a.pukhta_person_key IS ?2 AND a.is_active = 1
         ORDER BY a.id",
    )?;
    let rows = statement.query_map(params![account_id, person_key], |r| {
        Ok(ActiveRow {
            id: r.get(0)?,
            bill_line_id: r.get(1)?,
            payment_line_id: r.get(2)?,
            amount_paisa: r.get(3)?,
            pinned: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// Switches a bill matching row off, with who and when. The row stays (Code Standards B17).
/// Returns false if it was already off.
///
/// # Errors
/// Any SQLite error.
pub fn deactivate_bill_allocation(
    conn: &Connection,
    id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE bill_allocations SET is_active = 0, undone_at = ?2, undone_by = ?3,
             updated_at = ?2, updated_by = ?3, version = version + 1
         WHERE id = ?1 AND is_active = 1",
        params![id, now, user_id],
    )?;
    Ok(changed > 0)
}

/// The live debit lines of an account (or one person) that are not matched in full, oldest first:
/// the open bills (PRD FR-15.4).
///
/// # Errors
/// Any SQLite error.
pub fn open_bills(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<Vec<LineMatch>> {
    open_lines(conn, account_id, person_key, "dr_paisa", "bill_line_id")
}

/// The live credit lines of an account (or one person) that are not used in full against bills,
/// oldest first: the unmatched credits (PRD FR-15.5).
///
/// # Errors
/// Any SQLite error.
pub fn open_credits(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<Vec<LineMatch>> {
    open_lines(conn, account_id, person_key, "cr_paisa", "payment_line_id")
}

/// The lines on one side of a key with something left to match, oldest first. `side` and
/// `link` are fixed names from this file, never user text. The amount is filtered first, so
/// for an ordinary account (no person key on any line) the covering index on the amounts
/// answers without reading the lines: a buyer with 55,000 lines costs a few milliseconds.
fn open_lines(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
    side: &str,
    link: &str,
) -> rusqlite::Result<Vec<LineMatch>> {
    // A line of an ordinary account has no person key, so the condition is only needed for a
    // person of a پختہ کھاتہ account.
    let person = if person_key.is_some() {
        "AND l.pukhta_person_key = ?2"
    } else {
        "AND ?2 IS NULL"
    };
    let matched = format!(
        "COALESCE((SELECT SUM(a.amount_paisa) FROM bill_allocations a
                   WHERE a.{link} = l.id AND a.is_active = 1), 0)"
    );
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.{side}, {matched}
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.{side} > 0 {person} AND {LIVE}
           AND l.{side} > {matched}
         ORDER BY {ORDER}"
    ))?;
    let rows = statement.query_map(params![account_id, person_key], read)?;
    rows.collect()
}

/// What the open bills of a key add up to, when its matching rows are canonical: the debit less the
/// credit if that is above 0, else 0. Pinned pairs and settled bills cancel out of both sums. One
/// read of the covering index on the amounts. The caller must ask BEFORE it posts a new line,
/// because the rows are canonical for the lines that exist, not for the line about to be matched.
///
/// # Errors
/// Any SQLite error.
pub fn expected_open_bills(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<i64> {
    Ok(expected_pair(conn, account_id, person_key)?.0)
}

/// What the unmatched credits of a key add up to, when its matching rows are canonical: the credit
/// less the debit if that is above 0, else 0. Asked before a new line is posted, like
/// [`expected_open_bills`].
///
/// # Errors
/// Any SQLite error.
pub fn expected_unmatched_credit(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<i64> {
    Ok(expected_pair(conn, account_id, person_key)?.1)
}

/// Both of [`expected_open_bills`] and [`expected_unmatched_credit`] from one read: (open bills,
/// unmatched credit). One of the two is always 0.
///
/// # Errors
/// Any SQLite error.
pub fn expected_pair(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<(i64, i64)> {
    let totals = crate::ledger::totals_of(conn, account_id, person_key, None)?;
    Ok((
        difference(totals.dr_paisa, totals.cr_paisa)?.max(0),
        difference(totals.cr_paisa, totals.dr_paisa)?.max(0),
    ))
}

/// `a - b`, checked (PRD money rule M7). The sums it is given are whole-rupee totals of a
/// ledger, far from the limits, so an error here means the data is broken.
pub(crate) fn difference(a: i64, b: i64) -> rusqlite::Result<i64> {
    a.checked_sub(b)
        .ok_or(rusqlite::Error::IntegralValueOutOfRange(0, a))
}

/// [`open_bills`] for a key whose rows are canonical and whose open bills add up to `expected`
/// (see [`expected_open_bills`]). It walks back from the newest entry and stops as soon as the open
/// amounts found add up to `expected`, so a buyer with years of settled bills costs a few reads,
/// not one for each bill. The answer is the same as [`open_bills`] for canonical rows; if the walk
/// does not end with exactly `expected` (rows that are not canonical) it falls back to
/// [`open_bills`]. An exact total does not prove the rows are canonical: every writer keeps them
/// so (a re-match after anything but a plain new line), and the guard only catches a broken state
/// that shows itself in the total.
///
/// # Errors
/// Any SQLite error.
pub fn open_bills_upto(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
    expected: i64,
) -> rusqlite::Result<Vec<LineMatch>> {
    open_lines_walk(
        conn,
        account_id,
        person_key,
        "dr_paisa",
        "bill_line_id",
        expected,
    )
}

/// [`open_credits`] by the same walk; see [`open_bills_upto`].
///
/// # Errors
/// Any SQLite error.
pub fn open_credits_upto(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
    expected: i64,
) -> rusqlite::Result<Vec<LineMatch>> {
    open_lines_walk(
        conn,
        account_id,
        person_key,
        "cr_paisa",
        "payment_line_id",
        expected,
    )
}

fn open_lines_walk(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
    side: &str,
    link: &str,
    expected: i64,
) -> rusqlite::Result<Vec<LineMatch>> {
    if expected <= 0 {
        return Ok(Vec::new());
    }
    let person = if person_key.is_some() {
        "AND l.pukhta_person_key = ?2"
    } else {
        "AND ?2 IS NULL"
    };
    // The newest entry first by the index on (account, entry): no sort, and the rows come one at
    // a time, so stopping early reads nothing more.
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.{side}, e.entry_date, e.posting_type = 'opening', l.entry_id, l.line_no,
                COALESCE(e.order_key, e.id)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.{side} > 0 {person} AND {LIVE}
         ORDER BY l.entry_id DESC, l.line_no DESC"
    ))?;
    let mut matched_of = conn.prepare_cached(&format!(
        "SELECT COALESCE(SUM(amount_paisa), 0) FROM bill_allocations
         WHERE {link} = ?1 AND is_active = 1"
    ))?;
    let mut rows = statement.query(params![account_id, person_key])?;
    // (date, not opening, order key, entry id, line no) is the order of the account, oldest
    // first. The walk goes by entry id, which is not that order once an edit has kept the key of
    // the posting it replaced; it still finds every open line, because it stops only when the
    // open amounts add up to `expected`, and the sort below puts them in the account's order.
    let mut found: Vec<(AccountOrder, LineMatch)> = Vec::new();
    let mut total: i64 = 0;
    while let Some(row) = rows.next()? {
        let line_id: i64 = row.get(0)?;
        let amount: i64 = row.get(1)?;
        let matched: i64 = matched_of.query_row([line_id], |r| r.get(0))?;
        if amount > matched {
            let is_opening: bool = row.get(3)?;
            found.push((
                (
                    row.get(2)?,
                    !is_opening,
                    row.get(6)?,
                    row.get(4)?,
                    row.get(5)?,
                ),
                LineMatch {
                    line_id,
                    amount_paisa: amount,
                    matched_paisa: matched,
                },
            ));
            total = difference(amount, matched).and_then(|open| {
                total
                    .checked_add(open)
                    .ok_or(rusqlite::Error::IntegralValueOutOfRange(0, total))
            })?;
            if total >= expected {
                break;
            }
        }
    }
    drop(rows);
    if total != expected {
        // The rows are not canonical for this key: read everything, as before. In canonical rows
        // the walk stops with the open amounts adding up to exactly `expected`; a different total
        // (short, or over because open bills and unmatched credit both exist) means the walk could
        // hold the wrong lines.
        return open_lines(conn, account_id, person_key, side, link);
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(found.into_iter().map(|(_, line)| line).collect())
}

/// True if an entry dated after `date` has a line of this key. A new line is the newest of the
/// key (last in the oldest-first order) only when this is false: entries get ids in the order they
/// are saved, so an entry of the same day with a lower id is older. With no entry after `date`
/// (the usual case: `date` is today) it costs one look in the index of entry dates.
///
/// # Errors
/// Any SQLite error.
pub fn has_later_entry(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
    date: &str,
) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM journal_entries e
                        WHERE e.entry_date > ?1 AND e.status = 'posted'
                          AND EXISTS (SELECT 1 FROM journal_lines l
                                      WHERE l.entry_id = e.id AND l.account_id = ?2
                                        AND l.pukhta_person_key IS ?3))",
        params![date, account_id, person_key],
        |r| r.get(0),
    )
}

/// The live advance lines of a seller (debit lines flagged `is_advance`) that are not adjusted
/// in full, oldest first (PRD FR-15.8).
///
/// # Errors
/// Any SQLite error.
pub fn open_advances(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<LineMatch>> {
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.dr_paisa,
                COALESCE((SELECT SUM(a.amount_paisa) FROM advance_allocations a
                          WHERE a.advance_line_id = l.id AND a.is_active = 1), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.is_advance = 1 AND l.dr_paisa > 0 AND {LIVE}
           AND l.dr_paisa > COALESCE((SELECT SUM(a.amount_paisa) FROM advance_allocations a
                                      WHERE a.advance_line_id = l.id AND a.is_active = 1), 0)
         ORDER BY {ORDER}"
    ))?;
    let rows = statement.query_map(params![account_id], read)?;
    rows.collect()
}

/// The live advance lines of a seller (debit lines flagged `is_advance`) with their full
/// amounts, oldest first: what the canonical advance working starts from.
///
/// # Errors
/// Any SQLite error.
pub fn live_advance_lines(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<KeyLine>> {
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.dr_paisa, l.cr_paisa, e.id, e.entry_date, e.posting_type = 'opening',
                COALESCE(e.order_key, e.id)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.is_advance = 1 AND l.dr_paisa > 0 AND {LIVE}
         ORDER BY {ORDER}"
    ))?;
    let rows = statement.query_map(params![account_id], read_key_line)?;
    rows.collect()
}

/// A credit line of a seller that may adjust an advance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellerCredit {
    /// The journal line.
    pub line_id: i64,
    /// Its amount, paisa.
    pub amount_paisa: i64,
    /// Its entry.
    pub entry_id: i64,
    /// `bikri_first`, `bikri_later` or `receipt`.
    pub posting_type: String,
}

/// The live credit lines of a seller from the postings that can adjust an advance (PRD FR-15.8,
/// FR-15.12): the first posting of a بکری, a later sale of its leftover goods (plan 16) and a
/// receipt from the seller. Oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn seller_credit_lines(
    conn: &Connection,
    account_id: i64,
) -> rusqlite::Result<Vec<SellerCredit>> {
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.cr_paisa, e.id, e.posting_type
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.cr_paisa > 0 AND {LIVE}
           AND e.posting_type IN ('bikri_first', 'bikri_later', 'receipt')
         ORDER BY {ORDER}"
    ))?;
    let rows = statement.query_map(params![account_id], |r| {
        Ok(SellerCredit {
            line_id: r.get(0)?,
            amount_paisa: r.get(1)?,
            entry_id: r.get(2)?,
            posting_type: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// A lot of a seller in one of its postings, to tell which credit line is its own. The credit of a
/// posting (PRD C8) is worked out from the lines of that posting and, for the first posting only,
/// the lot charges, which never change after the first posting (PRD FR-12.17); the running
/// totals of the lot are not used, because a later sale changes them. A later sale charges no lot
/// charge, so its charge columns are 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostingLot {
    /// The posting.
    pub entry_id: i64,
    /// The lot.
    pub lot_id: i64,
    /// True for the first posting of the lot, false for a later sale.
    pub is_first: bool,
    /// The tick "پیشگی سے کاٹیں" (PRD FR-12.23).
    pub adjust_advance: bool,
    /// The sum of the seller amounts of the lot's lines in that posting, paisa.
    pub gross_paisa: i64,
    /// The sum of the seller commissions of those lines, paisa.
    pub comm_paisa: i64,
    /// The freight typed on the lot, paisa (0 for a later sale).
    pub freight_paisa: i64,
    /// True if the firm paid the driver (the freight is then deducted).
    pub freight_by_firm: bool,
    /// مزدوری, paisa (0 for a later sale).
    pub labour_paisa: i64,
    /// منشیانہ, paisa (0 for a later sale).
    pub munshiana_paisa: i64,
    /// کرایہ سٹور, paisa (0 for a later sale).
    pub store_rent_paisa: i64,
    /// فون, paisa (0 for a later sale).
    pub phone_paisa: i64,
}

/// The postings of the lots of a seller: the first posting of each lot in lot order, then the later
/// sales of it by posting. A posting that an edit or a cancel replaced is also listed (its lines
/// were switched off); the callers use only the postings that are live.
///
/// # Errors
/// Any SQLite error.
pub fn posting_lots(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<PostingLot>> {
    let mut statement = conn.prepare(
        "SELECT l.first_entry_id, l.id, 1, l.adjust_advance,
                COALESCE((SELECT SUM(s.seller_amount_paisa) FROM bikri_sales s
                          WHERE s.lot_id = l.id AND s.journal_entry_id = l.first_entry_id
                            AND s.is_active = 1), 0),
                COALESCE((SELECT SUM(s.seller_comm_paisa) FROM bikri_sales s
                          WHERE s.lot_id = l.id AND s.journal_entry_id = l.first_entry_id
                            AND s.is_active = 1), 0),
                l.freight_paisa, l.freight_paid_by = 'firm', l.labour_paisa, l.munshiana_paisa,
                l.store_rent_paisa, l.phone_paisa, l.lot_no
         FROM bikri_lots l
         WHERE l.seller_account_id = ?1 AND l.first_entry_id IS NOT NULL
         UNION ALL
         SELECT s.journal_entry_id, l.id, 0, l.adjust_advance,
                SUM(s.seller_amount_paisa), SUM(s.seller_comm_paisa), 0, 0, 0, 0, 0, 0, l.lot_no
         FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id
         WHERE l.seller_account_id = ?1 AND l.first_entry_id IS NOT NULL
           AND s.journal_entry_id <> l.first_entry_id AND s.is_active = 1
         GROUP BY s.journal_entry_id, l.id
         ORDER BY 1, 13",
    )?;
    let rows = statement.query_map(params![account_id], |r| {
        Ok(PostingLot {
            entry_id: r.get(0)?,
            lot_id: r.get(1)?,
            is_first: r.get(2)?,
            adjust_advance: r.get(3)?,
            gross_paisa: r.get(4)?,
            comm_paisa: r.get(5)?,
            freight_paisa: r.get(6)?,
            freight_by_firm: r.get(7)?,
            labour_paisa: r.get(8)?,
            munshiana_paisa: r.get(9)?,
            store_rent_paisa: r.get(10)?,
            phone_paisa: r.get(11)?,
        })
    })?;
    rows.collect()
}

/// An active advance matching row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvanceRow {
    /// Row id.
    pub id: i64,
    /// The advance line.
    pub advance_line_id: i64,
    /// The credit line.
    pub credit_line_id: i64,
    /// The amount, paisa.
    pub amount_paisa: i64,
}

/// The active advance rows of a seller.
///
/// # Errors
/// Any SQLite error.
pub fn active_advance_rows(
    conn: &Connection,
    account_id: i64,
) -> rusqlite::Result<Vec<AdvanceRow>> {
    let mut statement = conn.prepare(
        "SELECT id, advance_line_id, credit_line_id, amount_paisa FROM advance_allocations
         WHERE account_id = ?1 AND is_active = 1 ORDER BY id",
    )?;
    let rows = statement.query_map(params![account_id], |r| {
        Ok(AdvanceRow {
            id: r.get(0)?,
            advance_line_id: r.get(1)?,
            credit_line_id: r.get(2)?,
            amount_paisa: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// Switches an advance matching row off, with who and when. The row stays (Code Standards B17).
/// Returns false if it was already off.
///
/// # Errors
/// Any SQLite error.
pub fn deactivate_advance_allocation(
    conn: &Connection,
    id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE advance_allocations SET is_active = 0, undone_at = ?2, undone_by = ?3,
             updated_at = ?2, updated_by = ?3, version = version + 1
         WHERE id = ?1 AND is_active = 1",
        params![id, now, user_id],
    )?;
    Ok(changed > 0)
}

impl LiveLine {
    /// True for the credit of an opening balance. In the payable working of a seller it is his
    /// OLDEST posting whatever its date (PRD FR-7.59); see `vouchers::payable_credits_oldest_first`
    /// for the same rule in SQL.
    pub fn is_opening_credit(&self) -> bool {
        self.posting_type == "opening" && self.cr_paisa > 0
    }
}

/// A live line of an account, with its date, for read-time workings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveLine {
    /// The journal line.
    pub line_id: i64,
    /// The person key, for a پختہ کھاتہ account.
    pub person_key: Option<String>,
    /// Its entry.
    pub entry_id: i64,
    /// The day of its entry, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Debit, paisa.
    pub dr_paisa: i64,
    /// Credit, paisa.
    pub cr_paisa: i64,
    /// The posting type of its entry (`bikri_first`, `payment`, ...).
    pub posting_type: String,
    /// The kind of record its entry belongs to (`bikri`, `voucher`, `account`, ...).
    pub source_type: String,
    /// The number of that record where it has one: the بکری number or the voucher number.
    pub number: Option<i64>,
    /// True for an advance line.
    pub is_advance: bool,
    /// The sum of its active advance rows (as the advance line or as the credit line).
    pub advance_matched_paisa: i64,
    /// The order key of its entry: its own id unless an edit gave it the key of the posting it
    /// replaced (migration 0005).
    pub order_key: i64,
}

impl LiveLine {
    /// True if its entry is an opening balance, which sorts first on its day.
    pub fn is_opening(&self) -> bool {
        self.posting_type == "opening"
    }
}

/// Every live line of an account (all persons of a پختہ کھاتہ account), oldest first, with the
/// part of each line that advance rows take (as the advance or as the credit). Used by the
/// read-time working of what a seller is still owed (PRD FR-7.59). Only reads.
///
/// # Errors
/// Any SQLite error.
pub fn live_lines(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<LiveLine>> {
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.pukhta_person_key, e.id, e.entry_date, l.dr_paisa, l.cr_paisa, e.posting_type, l.is_advance,
                COALESCE((SELECT SUM(a.amount_paisa) FROM advance_allocations a
                          WHERE a.is_active = 1 AND (a.advance_line_id = l.id OR a.credit_line_id = l.id)), 0),
                e.source_type,
                CASE e.source_type
                    WHEN 'bikri' THEN (SELECT b.number FROM bikri_entries b WHERE b.id = e.source_id)
                    WHEN 'voucher' THEN (SELECT v.number FROM vouchers v WHERE v.id = e.source_id)
                END,
                COALESCE(e.order_key, e.id)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND {LIVE}
         ORDER BY {ORDER}"
    ))?;
    let rows = statement.query_map(params![account_id], |r| {
        Ok(LiveLine {
            line_id: r.get(0)?,
            person_key: r.get(1)?,
            entry_id: r.get(2)?,
            entry_date: r.get(3)?,
            dr_paisa: r.get(4)?,
            cr_paisa: r.get(5)?,
            posting_type: r.get(6)?,
            is_advance: r.get(7)?,
            advance_matched_paisa: r.get(8)?,
            source_type: r.get(9)?,
            number: r.get(10)?,
            order_key: r.get(11)?,
        })
    })?;
    rows.collect()
}

/// The sum of the active bill rows of each of these bill lines. A line with none is not in the
/// answer.
///
/// # Errors
/// Any SQLite error.
pub fn bill_matched(
    conn: &Connection,
    bill_line_ids: &[i64],
) -> rusqlite::Result<std::collections::HashMap<i64, i64>> {
    if bill_line_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let marks = vec!["?"; bill_line_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT bill_line_id, SUM(amount_paisa) FROM bill_allocations
         WHERE is_active = 1 AND bill_line_id IN ({marks}) GROUP BY bill_line_id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(bill_line_ids), |r| {
        Ok((r.get(0)?, r.get(1)?))
    })?;
    rows.collect()
}

/// A matching row to write. Both lines must be on `account_id` and `person_key` (the triggers
/// check it), the bill line a debit and the payment line a credit.
#[derive(Debug, Clone, Copy)]
pub struct NewBillAllocation<'a> {
    /// The debit line that is paid.
    pub bill_line_id: i64,
    /// The credit line that pays it.
    pub payment_line_id: i64,
    /// The account of both.
    pub account_id: i64,
    /// The person key of both, for a پختہ کھاتہ account.
    pub person_key: Option<&'a str>,
    /// The amount, paisa.
    pub amount_paisa: i64,
}

/// Adds a bill matching row.
///
/// # Errors
/// Any SQLite error (the triggers refuse wrong lines and an amount above a line).
pub fn insert_bill_allocation(
    conn: &Connection,
    row: &NewBillAllocation<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO bill_allocations (created_at, created_by, updated_at, updated_by, bill_line_id, payment_line_id,
             account_id, pukhta_person_key, amount_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            now,
            user_id,
            row.bill_line_id,
            row.payment_line_id,
            row.account_id,
            row.person_key,
            row.amount_paisa
        ],
    )?;
    Ok(())
}

/// An advance matching row to write.
#[derive(Debug, Clone, Copy)]
pub struct NewAdvanceAllocation<'a> {
    /// The advance (a debit line flagged `is_advance`).
    pub advance_line_id: i64,
    /// The seller credit line that adjusts it.
    pub credit_line_id: i64,
    /// The seller account.
    pub account_id: i64,
    /// The person key, for a پختہ کھاتہ account (never for an advance in practice).
    pub person_key: Option<&'a str>,
    /// The amount, paisa.
    pub amount_paisa: i64,
}

/// Adds an advance matching row.
///
/// # Errors
/// Any SQLite error (the triggers refuse wrong lines and an amount above a line).
pub fn insert_advance_allocation(
    conn: &Connection,
    row: &NewAdvanceAllocation<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO advance_allocations (created_at, created_by, updated_at, updated_by, advance_line_id,
             credit_line_id, account_id, pukhta_person_key, amount_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            now,
            user_id,
            row.advance_line_id,
            row.credit_line_id,
            row.account_id,
            row.person_key,
            row.amount_paisa
        ],
    )?;
    Ok(())
}
