//! Queries for `vouchers` and for the day state (Architecture 15.3, 15.7, plan 10). SQL only:
//! the checks are in `mandi_core::vouchers` and `mandi_services::vouchers`. A voucher is never
//! deleted: a cancelled voucher keeps its number and its row (Code Standards D3).

use mandi_core::accounts::RecordRef;
use mandi_core::money::Paisa;
use mandi_core::vouchers::{LastAdvance, VoucherMode, VoucherRow, VoucherType};
use rusqlite::{Connection, OptionalExtension, params};

/// The columns of a new voucher that the user chooses or the framework gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewVoucher {
    /// The kind.
    pub voucher_type: VoucherType,
    /// The number in its series.
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The account.
    pub account_id: Option<i64>,
    /// The amount in paisa.
    pub amount_paisa: i64,
    /// How the money moves.
    pub mode: VoucherMode,
    /// The bank account row (`bank_accounts.id`), for bank and cheque.
    pub bank_account_id: Option<i64>,
    /// The cheque number, for a cheque.
    pub cheque_no: Option<String>,
    /// The cheque date, `YYYY-MM-DD`, for a cheque.
    pub cheque_date: Option<String>,
    /// The expected goods of an advance.
    pub expected_goods: Option<String>,
    /// The note.
    pub narration: Option<String>,
}

/// The next number of a series: the biggest number ever given plus one. Cancelled vouchers keep
/// their rows, so a number is never used again.
///
/// # Errors
/// Any SQLite error.
pub fn next_number(conn: &Connection, voucher_type: VoucherType) -> rusqlite::Result<i64> {
    let last: i64 = conn.query_row(
        "SELECT COALESCE(MAX(number), 0) FROM vouchers WHERE type = ?1",
        [voucher_type.as_str()],
        |r| r.get(0),
    )?;
    Ok(last.saturating_add(1))
}

/// Adds a voucher and returns its id.
///
/// # Errors
/// Any SQLite error (a CHECK that the checks above should have caught).
pub fn insert(
    conn: &Connection,
    voucher: &NewVoucher,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO vouchers (created_at, created_by, updated_at, updated_by, type, number, entry_date,
             account_id, amount_paisa, mode, bank_account_id, cheque_no, cheque_date, expected_goods, narration)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            now,
            user_id,
            voucher.voucher_type.as_str(),
            voucher.number,
            voucher.entry_date,
            voucher.account_id,
            voucher.amount_paisa,
            voucher.mode.as_str(),
            voucher.bank_account_id,
            voucher.cheque_no,
            voucher.cheque_date,
            voucher.expected_goods,
            voucher.narration,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Names the journal entry that holds the voucher's posting.
///
/// # Errors
/// Any SQLite error.
pub fn set_current_entry(
    conn: &Connection,
    voucher_id: i64,
    entry_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE vouchers SET current_entry_id = ?2, updated_at = ?3, updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![voucher_id, entry_id, now, user_id],
    )?;
    Ok(())
}

/// A bank account row as a voucher needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BankFacts {
    /// The ledger account that holds its money (under control 0102).
    pub account_id: i64,
    /// False when hidden from pickers.
    pub is_active: bool,
}

/// A bank account (`bank_accounts.id`), if there is one.
///
/// # Errors
/// Any SQLite error.
pub fn bank_account(conn: &Connection, id: i64) -> rusqlite::Result<Option<BankFacts>> {
    conn.query_row(
        "SELECT account_id, is_active FROM bank_accounts WHERE id = ?1",
        [id],
        |r| {
            Ok(BankFacts {
                account_id: r.get(0)?,
                is_active: r.get(1)?,
            })
        },
    )
    .optional()
}

fn bad(column: usize, what: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        rusqlite::types::Type::Text,
        format!("vouchers: {what}").into(),
    )
}

/// The vouchers of a kind in a date range (`YYYY-MM-DD`, both included, either may be open),
/// by day and number.
///
/// # Errors
/// Any SQLite error, or a row whose text is not one of the known words.
pub fn list(
    conn: &Connection,
    voucher_type: Option<VoucherType>,
    from: Option<&str>,
    to: Option<&str>,
) -> rusqlite::Result<Vec<VoucherRow>> {
    let mut statement = conn.prepare(
        "SELECT v.id, v.type, v.number, v.entry_date, v.account_id, a.code, a.name_ur, a.name_en,
                v.amount_paisa, v.mode, v.bank_account_id, v.cheque_no, v.cheque_date,
                v.expected_goods, v.narration, v.status
         FROM vouchers v LEFT JOIN accounts a ON a.id = v.account_id
         WHERE (?1 IS NULL OR v.type = ?1) AND (?2 IS NULL OR v.entry_date >= ?2)
           AND (?3 IS NULL OR v.entry_date <= ?3)
         ORDER BY v.entry_date, v.number, v.id",
    )?;
    let rows = statement.query_map(
        params![voucher_type.map(VoucherType::as_str), from, to],
        |r| {
            let account_id: Option<i64> = r.get(4)?;
            Ok(VoucherRow {
                id: r.get(0)?,
                voucher_type: VoucherType::parse(&r.get::<_, String>(1)?)
                    .ok_or_else(|| bad(1, "type"))?,
                number: r.get(2)?,
                entry_date: r.get(3)?,
                account: account_id.map(|id| RecordRef {
                    id,
                    code: r
                        .get::<_, Option<String>>(5)
                        .ok()
                        .flatten()
                        .unwrap_or_default(),
                    name_ur: r
                        .get::<_, Option<String>>(6)
                        .ok()
                        .flatten()
                        .unwrap_or_default(),
                    name_en: r
                        .get::<_, Option<String>>(7)
                        .ok()
                        .flatten()
                        .unwrap_or_default(),
                }),
                amount_paisa: Paisa::try_from_paisa(r.get(8)?).map_err(|_| bad(8, "amount"))?,
                mode: VoucherMode::parse(&r.get::<_, String>(9)?).ok_or_else(|| bad(9, "mode"))?,
                bank_account_id: r.get(10)?,
                cheque_no: r.get::<_, Option<String>>(11)?.unwrap_or_default(),
                cheque_date: r.get(12)?,
                expected_goods: r.get::<_, Option<String>>(13)?.unwrap_or_default(),
                narration: r.get::<_, Option<String>>(14)?.unwrap_or_default(),
                status: r.get(15)?,
            })
        },
    )?;
    rows.collect()
}

/// The last پیشگی voucher of a seller that is still in force.
///
/// # Errors
/// Any SQLite error.
pub fn last_advance(conn: &Connection, seller_id: i64) -> rusqlite::Result<Option<LastAdvance>> {
    conn.query_row(
        "SELECT entry_date, amount_paisa, number FROM vouchers
         WHERE type = 'advance' AND account_id = ?1 AND status = 'saved'
         ORDER BY entry_date DESC, number DESC LIMIT 1",
        [seller_id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        },
    )
    .optional()?
    .map(|(entry_date, paisa, voucher_number)| {
        Ok(LastAdvance {
            entry_date,
            amount_paisa: Paisa::try_from_paisa(paisa).map_err(|_| bad(1, "amount"))?,
            voucher_number,
        })
    })
    .transpose()
}

/// One advance line of a seller and what has been adjusted against it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceLine {
    /// `voucher` or `opening` (or the other source types, as stored).
    pub source_type: String,
    /// The voucher number, for source `voucher`.
    pub voucher_number: Option<i64>,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// What was given, paisa.
    pub given_paisa: i64,
    /// What has been adjusted against it, paisa.
    pub adjusted_paisa: i64,
}

/// The seller's advance lines: posted debit lines flagged as an advance, without those of an
/// entry that has been reversed, oldest first (PRD FR-15.8). Adjusted counts only active
/// matching rows whose credit line is still in force, the same rule as the card's open
/// advance.
///
/// # Errors
/// Any SQLite error.
pub fn advance_lines(conn: &Connection, seller_id: i64) -> rusqlite::Result<Vec<AdvanceLine>> {
    let mut statement = conn.prepare(
        "SELECT e.source_type, v.number, e.entry_date, l.dr_paisa,
                COALESCE((SELECT SUM(a.amount_paisa)
                          FROM advance_allocations a JOIN journal_lines cl ON cl.id = a.credit_line_id
                          WHERE a.advance_line_id = l.id AND a.is_active = 1
                            AND NOT EXISTS (SELECT 1 FROM journal_entries r
                                            WHERE r.reverses_entry_id = cl.entry_id AND r.status = 'posted')), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id
         WHERE l.account_id = ?1 AND l.is_advance = 1 AND e.status = 'posted'
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
         ORDER BY e.entry_date, e.id, l.line_no",
    )?;
    let rows = statement.query_map([seller_id], |r| {
        Ok(AdvanceLine {
            source_type: r.get(0)?,
            voucher_number: r.get(1)?,
            entry_date: r.get(2)?,
            given_paisa: r.get(3)?,
            adjusted_paisa: r.get(4)?,
        })
    })?;
    rows.collect()
}

// ---- The day state (PRD FR-6.6, FR-21.1, FR-21.2) ----

/// The date of the oldest posted entry, if there is one.
///
/// # Errors
/// Any SQLite error.
pub fn first_posting_date(conn: &Connection) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT MIN(entry_date) FROM journal_entries WHERE status = 'posted'",
        [],
        |r| r.get(0),
    )
}

/// How many journal entries are not posted. A draft never stays after a save, so any is a bug.
///
/// # Errors
/// Any SQLite error.
pub fn unposted_entries(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'",
        [],
        |r| r.get(0),
    )
}

/// How many posted entries are dated from `from` to `to` (both included).
///
/// # Errors
/// Any SQLite error.
pub fn entries_between(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM journal_entries WHERE status = 'posted' AND entry_date >= ?1 AND entry_date <= ?2",
        params![from, to],
        |r| r.get(0),
    )
}

/// What one account did on one day: the sums of its posted lines dated that day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Movement {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The account.
    pub account_id: i64,
    /// Σ debits that day, paisa.
    pub dr_paisa: i64,
    /// Σ credits that day, paisa.
    pub cr_paisa: i64,
}

/// What each account did on each day from `from` to `to` (both included), in day order. One
/// query, so a close of many days reads the journal once (Code Standards B22).
///
/// # Errors
/// Any SQLite error.
pub fn movements_between(
    conn: &Connection,
    from: &str,
    to: &str,
) -> rusqlite::Result<Vec<Movement>> {
    let mut statement = conn.prepare(
        "SELECT e.entry_date, l.account_id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE e.status = 'posted' AND e.entry_date >= ?1 AND e.entry_date <= ?2
         GROUP BY e.entry_date, l.account_id ORDER BY e.entry_date, l.account_id",
    )?;
    let rows = statement.query_map(params![from, to], |r| {
        Ok(Movement {
            date: r.get(0)?,
            account_id: r.get(1)?,
            dr_paisa: r.get(2)?,
            cr_paisa: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// How many posted entries are dated on each day from `from` to `to`; days with none are not in
/// the answer.
///
/// # Errors
/// Any SQLite error.
pub fn entries_per_day(
    conn: &Connection,
    from: &str,
    to: &str,
) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut statement = conn.prepare(
        "SELECT entry_date, COUNT(*) FROM journal_entries
         WHERE status = 'posted' AND entry_date >= ?1 AND entry_date <= ?2
         GROUP BY entry_date ORDER BY entry_date",
    )?;
    let rows = statement.query_map(params![from, to], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// Moves the "closed up to" date (`None` clears it).
///
/// # Errors
/// Any SQLite error.
pub fn set_closed_up_to(
    conn: &Connection,
    date: Option<&str>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE day_state SET closed_up_to_date = ?1, updated_at = ?2, updated_by = ?3, version = version + 1
         WHERE id = 1",
        params![date, now, user_id],
    )?;
    Ok(())
}

/// Writes a row of the day-close history (`close` or `reopen`).
///
/// # Errors
/// Any SQLite error.
pub fn insert_day_close(
    conn: &Connection,
    entry_date: &str,
    action: &str,
    user_id: i64,
    now: &str,
    reason: Option<&str>,
    summary_json: &str,
) -> rusqlite::Result<()> {
    // Cached: a long close writes thousands of these rows one after another.
    conn.prepare_cached(
        "INSERT INTO day_closes (created_at, created_by, updated_at, updated_by, entry_date, action, by_user_id, at, reason, summary_json)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?2, ?1, ?5, ?6)",
    )?
    .execute(params![now, user_id, entry_date, action, reason, summary_json])?;
    Ok(())
}

/// True if a month-close that is still in force covers `month` (`YYYY-MM`) or a later month
/// (PRD FR-21.2: such a month cannot be reopened until its month close is reversed).
///
/// # Errors
/// Any SQLite error.
pub fn month_closed_from(conn: &Connection, month: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM month_closes WHERE is_active = 1 AND month >= ?1)",
        [month],
        |r| r.get(0),
    )
}
