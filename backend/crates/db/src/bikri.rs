//! Reads for the بکری entry screen (PRD 12, plan 12): the facts of the accounts, items and
//! masters a draft names, the saved rates behind the rate hint and the rate check, and the
//! memories of trucks and sellers. Everything here only reads (Code Standards B29). The save
//! tables are written in plan 13.
//!
//! Every statement uses parameters (Code Standards B14). A list of ids is read with one
//! statement that has one `?` for each id (Code Standards B22).

use rusqlite::{Connection, OptionalExtension, params_from_iter};

/// The facts about an account that the maths and the checks of a بکری need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewAccount {
    /// Row id.
    pub id: i64,
    /// The control code, for example `03011` (seller) or `0304` (buyer).
    pub control_code: String,
    /// True for the two پختہ کھاتہ accounts.
    pub is_pukhta_khata: bool,
    /// True for a system account such as نقدی منڈی.
    pub is_system: bool,
    /// The account code, for example `03040000`.
    pub code: String,
    /// False when hidden from pickers.
    pub is_active: bool,
    /// False means cash only (PRD 10.2 "ادھار کی اجازت").
    pub credit_allowed: bool,
    /// The credit limit in paisa, if one is set.
    pub credit_limit_paisa: Option<i64>,
}

/// An item, as the maths needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewItem {
    /// Row id.
    pub id: i64,
    /// `fruit` or `vegetable`.
    pub item_group: String,
    /// `unit` or `weight`.
    pub sold_by: String,
    /// False when hidden from pickers.
    pub is_active: bool,
}

/// The master tables a lot or a header points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasterTable {
    /// `packings`.
    Packings,
    /// `markas`.
    Markas,
    /// `stores`.
    Stores,
    /// `addas`.
    Addas,
}

impl MasterTable {
    /// The table name. A fixed list, never user text.
    const fn name(self) -> &'static str {
        match self {
            MasterTable::Packings => "packings",
            MasterTable::Markas => "markas",
            MasterTable::Stores => "stores",
            MasterTable::Addas => "addas",
        }
    }
}

/// The comma-separated `?` marks for a list of ids.
fn marks(count: usize) -> String {
    vec!["?"; count].join(", ")
}

/// The facts of these accounts. An id that does not exist is not in the answer.
///
/// # Errors
/// Any SQLite error.
pub fn accounts(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<PreviewAccount>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut statement = conn.prepare(&format!(
        "SELECT a.id, c.code, a.is_pukhta_khata, a.is_system, a.code, a.is_active,
                a.credit_allowed, a.credit_limit_paisa
         FROM accounts a JOIN account_controls c ON c.id = a.control_id
         WHERE a.id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids), |r| {
        Ok(PreviewAccount {
            id: r.get(0)?,
            control_code: r.get(1)?,
            is_pukhta_khata: r.get(2)?,
            is_system: r.get(3)?,
            code: r.get(4)?,
            is_active: r.get(5)?,
            credit_allowed: r.get(6)?,
            credit_limit_paisa: r.get(7)?,
        })
    })?;
    rows.collect()
}

/// The facts of these items. An id that does not exist is not in the answer.
///
/// # Errors
/// Any SQLite error.
pub fn items(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<PreviewItem>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut statement = conn.prepare(&format!(
        "SELECT id, item_group, sold_by, is_active FROM items WHERE id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids), |r| {
        Ok(PreviewItem {
            id: r.get(0)?,
            item_group: r.get(1)?,
            sold_by: r.get(2)?,
            is_active: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// Which of these ids exist in a master table.
///
/// # Errors
/// Any SQLite error.
pub fn existing_ids(
    conn: &Connection,
    table: MasterTable,
    ids: &[i64],
) -> rusqlite::Result<Vec<i64>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut statement = conn.prepare(&format!(
        "SELECT id FROM {} WHERE id IN ({})",
        table.name(),
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids), |r| r.get(0))?;
    rows.collect()
}

/// A saved sale rate, with the day it was saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedRate {
    /// The seller's rate in paisa.
    pub rate_paisa: i64,
    /// The day of the sale, `YYYY-MM-DD`.
    pub entry_date: String,
}

/// The newest saved seller rate of an item and packing on or before a day (PRD FR-7.53): the
/// newest day first, and on that day the last line saved. A line replaced by an edit or
/// cancelled (`is_active = 0`) does not count.
///
/// # Errors
/// Any SQLite error.
pub fn last_rate(
    conn: &Connection,
    item_id: i64,
    packing_id: i64,
    on_or_before: &str,
) -> rusqlite::Result<Option<SavedRate>> {
    conn.query_row(
        "SELECT s.seller_rate_paisa, s.entry_date
         FROM bikri_lots l JOIN bikri_sales s ON s.lot_id = l.id
         WHERE l.item_id = ?1 AND l.packing_id = ?2 AND s.is_active = 1 AND s.entry_date <= ?3
         ORDER BY s.entry_date DESC, s.id DESC
         LIMIT 1",
        rusqlite::params![item_id, packing_id, on_or_before],
        |r| {
            Ok(SavedRate {
                rate_paisa: r.get(0)?,
                entry_date: r.get(1)?,
            })
        },
    )
    .optional()
}

/// The saved sales of an item in a span of days, for the rate check (PRD FR-7.55): the sum of
/// `sign × qty` and the sum of `sign × seller amount` over the facts, so a cancelled or edited
/// line (a reversal row with sign −1) takes its original out. `SUM()` on INTEGER columns only
/// (CLAUDE.md gotchas).
///
/// # Errors
/// Any SQLite error.
pub fn item_sales_between(
    conn: &Connection,
    item_id: i64,
    from: &str,
    to: &str,
) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT COALESCE(SUM(sign * qty), 0), COALESCE(SUM(sign * seller_amount_paisa), 0)
         FROM analytics_facts WHERE item_id = ?1 AND entry_date BETWEEN ?2 AND ?3",
        rusqlite::params![item_id, from, to],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
}

/// The last saved بکری of a truck.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruckBikri {
    /// The بکری number.
    pub number: i64,
    /// Its day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The truck stand.
    pub adda_id: Option<i64>,
    /// The زمیندار.
    pub zamindar_account_id: Option<i64>,
    /// The driver's name.
    pub driver_name: Option<String>,
    /// The driver's phone.
    pub driver_phone: Option<String>,
}

/// The newest saved بکری of exactly this truck number, ignoring letter case (PRD FR-7.49).
/// A cancelled بکری does not count.
///
/// # Errors
/// Any SQLite error.
pub fn truck_last(conn: &Connection, truck_no: &str) -> rusqlite::Result<Option<TruckBikri>> {
    conn.query_row(
        "SELECT number, entry_date, adda_id, zamindar_account_id, driver_name, driver_phone
         FROM bikri_entries
         WHERE truck_no = ?1 COLLATE NOCASE AND status = 'saved'
         ORDER BY entry_date DESC, id DESC
         LIMIT 1",
        [truck_no],
        |r| {
            Ok(TruckBikri {
                number: r.get(0)?,
                entry_date: r.get(1)?,
                adda_id: r.get(2)?,
                zamindar_account_id: r.get(3)?,
                driver_name: r.get(4)?,
                driver_phone: r.get(5)?,
            })
        },
    )
    .optional()
}

/// The number of a saved, not cancelled بکری of this truck on this day, if any (PRD FR-7.50).
/// If there are several, the first one.
///
/// # Errors
/// Any SQLite error.
pub fn truck_same_day(
    conn: &Connection,
    truck_no: &str,
    entry_date: &str,
) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT number FROM bikri_entries
         WHERE truck_no = ?1 COLLATE NOCASE AND entry_date = ?2 AND status = 'saved'
         ORDER BY id LIMIT 1",
        rusqlite::params![truck_no, entry_date],
        |r| r.get(0),
    )
    .optional()
}

/// A truck number used before, with the day of its last بکری.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruckName {
    /// The truck number as saved.
    pub truck_no: String,
    /// The day of its last بکری, `YYYY-MM-DD`.
    pub last_date: String,
}

/// Truck numbers that start with the typed text, newest first (PRD FR-7.49). `%`, `_` and `\` in
/// the text are taken as plain characters. A cancelled بکری does not count.
///
/// # Errors
/// Any SQLite error.
pub fn truck_numbers_starting_with(
    conn: &Connection,
    prefix: &str,
    limit: i64,
) -> rusqlite::Result<Vec<TruckName>> {
    let mut escaped = String::with_capacity(prefix.len().saturating_add(1));
    for c in prefix.chars() {
        if matches!(c, '%' | '_' | '\\') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped.push('%');
    let mut statement = conn.prepare(
        "SELECT truck_no, MAX(entry_date) AS last_date
         FROM bikri_entries
         WHERE truck_no LIKE ?1 ESCAPE '\\' AND status = 'saved'
         GROUP BY truck_no COLLATE NOCASE
         ORDER BY last_date DESC, truck_no
         LIMIT ?2",
    )?;
    let rows = statement.query_map(rusqlite::params![escaped, limit], |r| {
        Ok(TruckName {
            truck_no: r.get(0)?,
            last_date: r.get(1)?,
        })
    })?;
    rows.collect()
}

/// What a seller's last lot holds (PRD FR-7.51).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastLot {
    /// The item.
    pub item_id: i64,
    /// The packing.
    pub packing_id: i64,
    /// The مارکہ, if the lot had one.
    pub marka_id: Option<i64>,
}

/// The newest lot of a seller in a saved, not cancelled بکری.
///
/// # Errors
/// Any SQLite error.
pub fn seller_last_lot(conn: &Connection, seller_id: i64) -> rusqlite::Result<Option<LastLot>> {
    conn.query_row(
        "SELECT l.item_id, l.packing_id, l.marka_id
         FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
         WHERE l.seller_account_id = ?1 AND e.status = 'saved'
         ORDER BY e.entry_date DESC, l.id DESC
         LIMIT 1",
        [seller_id],
        |r| {
            Ok(LastLot {
                item_id: r.get(0)?,
                packing_id: r.get(1)?,
                marka_id: r.get(2)?,
            })
        },
    )
    .optional()
}

/// A seller's default مارکہ (`accounts.default_marka_id`).
///
/// # Errors
/// Any SQLite error.
pub fn default_marka(conn: &Connection, account_id: i64) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT default_marka_id FROM accounts WHERE id = ?1",
        [account_id],
        |r| r.get(0),
    )
    .optional()
    .map(Option::flatten)
}

/// The ids of accounts by code, for the system accounts a بکری posts to. A code that does not
/// exist is not in the answer.
///
/// # Errors
/// Any SQLite error.
pub fn ids_by_codes(
    conn: &Connection,
    codes: &[&str],
) -> rusqlite::Result<std::collections::HashMap<String, i64>> {
    if codes.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let mut statement = conn.prepare(&format!(
        "SELECT code, id FROM accounts WHERE code IN ({})",
        marks(codes.len())
    ))?;
    let rows = statement.query_map(params_from_iter(codes), |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}
