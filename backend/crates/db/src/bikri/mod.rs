//! The بکری tables (PRD 12, plans 12 and 13).
//!
//! This file reads for the entry screen: the facts of the accounts, items and masters a draft
//! names, the saved rates behind the rate hint and the rate check, and the memories of trucks
//! and sellers. Everything here only reads (Code Standards B29). `write` adds the rows of a
//! save (plan 13), and `saved` reads saved بکری back.
//!
//! Every statement uses parameters (Code Standards B14). A list of ids is read with one
//! statement that has one `?` for each id (Code Standards B22).

use rusqlite::{Connection, OptionalExtension, params_from_iter};

pub mod edit;
pub mod saved;
pub mod write;

/// The facts about an account that the maths and the checks of a بکری need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewAccount {
    /// Row id.
    pub id: i64,
    /// The control code, for example `03011` (seller) or `0304` (buyer).
    pub control_code: String,
    /// True for the چکتا کھاتہ account and the old closed one (person typed on the line).
    pub is_chukta_khata: bool,
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
        "SELECT a.id, c.code, a.is_chukta_khata, a.is_system, a.code, a.is_active,
                a.credit_allowed, a.credit_limit_paisa
         FROM accounts a JOIN account_controls c ON c.id = a.control_id
         WHERE a.id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids), |r| {
        Ok(PreviewAccount {
            id: r.get(0)?,
            control_code: r.get(1)?,
            is_chukta_khata: r.get(2)?,
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
    last_rate_except(conn, item_id, packing_id, on_or_before, None)
}

/// [`last_rate`] without the lines of one posting: an edit does not take its own lines as the
/// last rate (plan 16).
///
/// # Errors
/// Any SQLite error.
pub fn last_rate_except(
    conn: &Connection,
    item_id: i64,
    packing_id: i64,
    on_or_before: &str,
    except_entry: Option<i64>,
) -> rusqlite::Result<Option<SavedRate>> {
    conn.query_row(
        "SELECT s.seller_rate_paisa, s.entry_date
         FROM bikri_lots l JOIN bikri_sales s ON s.lot_id = l.id
         WHERE l.item_id = ?1 AND l.packing_id = ?2 AND s.is_active = 1 AND s.entry_date <= ?3
           AND (?4 IS NULL OR s.journal_entry_id <> ?4)
         ORDER BY s.entry_date DESC, s.id DESC
         LIMIT 1",
        rusqlite::params![item_id, packing_id, on_or_before, except_entry],
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
    item_sales_between_except(conn, item_id, from, to, None)
}

/// [`item_sales_between`] without the facts of one posting (and of its reversal): an edit does not
/// compare its rates with its own old lines (plan 16).
///
/// # Errors
/// Any SQLite error.
pub fn item_sales_between_except(
    conn: &Connection,
    item_id: i64,
    from: &str,
    to: &str,
    except_entry: Option<i64>,
) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT COALESCE(SUM(sign * qty), 0), COALESCE(SUM(sign * seller_amount_paisa), 0)
         FROM analytics_facts WHERE item_id = ?1 AND entry_date BETWEEN ?2 AND ?3
           AND (?4 IS NULL OR journal_entry_id <> ?4)",
        rusqlite::params![item_id, from, to, except_entry],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
}

/// The biggest lot counter ever given to each of these (seller, item) pairs (PRD FR-12.45), from
/// `lot_counters`. A pair that never had a lot is missing from the answer, which counts as 0. One
/// statement for all the pairs (Code Standards B22).
///
/// # Errors
/// Any SQLite error.
pub fn lot_counters_of(
    conn: &Connection,
    pairs: &[(i64, i64)],
) -> rusqlite::Result<std::collections::HashMap<(i64, i64), i64>> {
    let mut out = std::collections::HashMap::with_capacity(pairs.len());
    if pairs.is_empty() {
        return Ok(out);
    }
    let sellers: std::collections::BTreeSet<i64> = pairs.iter().map(|p| p.0).collect();
    let marks = vec!["?"; sellers.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT seller_account_id, item_id, last_counter FROM lot_counters
         WHERE seller_account_id IN ({marks})"
    ))?;
    let rows = statement.query_map(params_from_iter(sellers.iter()), |r| {
        Ok((
            (r.get::<_, i64>(0)?, r.get::<_, i64>(1)?),
            r.get::<_, i64>(2)?,
        ))
    })?;
    for row in rows {
        let (pair, last) = row?;
        if pairs.contains(&pair) {
            out.insert(pair, last);
        }
    }
    Ok(out)
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

/// The lots of saved (not cancelled) بکری of these sellers and items, with this counter if given,
/// newest day first, at most `limit` and how many there are in all (PRD FR-12.45, FR-7.77). An empty
/// list of sellers or items is not a filter. Reads only; one statement for the page and one for the
/// count (Code Standards B22).
///
/// # Errors
/// Any SQLite error.
pub fn lots_named(
    conn: &Connection,
    seller_ids: &[i64],
    item_ids: &[i64],
    counter: Option<i64>,
    lot_id: Option<i64>,
    limit: i64,
) -> rusqlite::Result<(Vec<mandi_core::bikri::LotFound>, i64)> {
    use mandi_core::accounts::RecordRef;
    let mut conditions = vec!["e.status = 'saved'".to_owned()];
    let mut args: Vec<rusqlite::types::Value> = Vec::new();
    let mut add_list = |column: &str, ids: &[i64], conditions: &mut Vec<String>| {
        if !ids.is_empty() {
            conditions.push(format!("{column} IN ({})", vec!["?"; ids.len()].join(", ")));
            args.extend(ids.iter().map(|id| rusqlite::types::Value::Integer(*id)));
        }
    };
    add_list("l.seller_account_id", seller_ids, &mut conditions);
    add_list("l.item_id", item_ids, &mut conditions);
    if let Some(counter) = counter {
        conditions.push("l.lot_counter = ?".to_owned());
        args.push(rusqlite::types::Value::Integer(counter));
    }
    if let Some(lot_id) = lot_id {
        conditions.push("l.id = ?".to_owned());
        args.push(rusqlite::types::Value::Integer(lot_id));
    }
    let filter = conditions.join(" AND ");
    let total: i64 = conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id WHERE {filter}"
        ),
        params_from_iter(args.iter()),
        |r| r.get(0),
    )?;
    let mut page_args = args.clone();
    page_args.push(rusqlite::types::Value::Integer(limit));
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.lot_counter, e.entry_date, e.number, l.qty_received,
                sa.id, sa.code, sa.name_ur, sa.name_en, sa.tag,
                it.id, it.code, it.name_ur, it.name_en
         FROM bikri_lots l
         JOIN bikri_entries e ON e.id = l.bikri_id
         JOIN accounts sa ON sa.id = l.seller_account_id
         JOIN items it ON it.id = l.item_id
         WHERE {filter}
         ORDER BY e.entry_date DESC, e.number DESC, l.lot_no
         LIMIT ?"
    ))?;
    let rows = statement.query_map(params_from_iter(page_args.iter()), |r| {
        Ok(mandi_core::bikri::LotFound {
            name: mandi_core::bikri::LotName {
                lot_id: r.get(0)?,
                counter: r.get(1)?,
                seller: RecordRef {
                    id: r.get(5)?,
                    code: r.get(6)?,
                    name_ur: r.get(7)?,
                    name_en: r.get(8)?,
                    tag: r.get(9)?,
                },
                item: RecordRef {
                    id: r.get(10)?,
                    code: r.get(11)?,
                    name_ur: r.get(12)?,
                    name_en: r.get(13)?,
                    tag: None,
                },
            },
            entry_date: r.get(2)?,
            bikri_number: r.get(3)?,
            qty_received: r.get(4)?,
        })
    })?;
    Ok((rows.collect::<rusqlite::Result<Vec<_>>>()?, total))
}
