//! The rows that an edit, a later sale, a cancel and a write-off change or add (plan 16, PRD
//! FR-12.16 to FR-12.22). SQL only: every number is worked out and checked before it comes here,
//! and the CHECKs and triggers of `0001_init.sql` and `0005_stable_posting_order.sql` are the last
//! guard (Code Standards 4.1 layer 4).
//!
//! Nothing here deletes a row (Code Standards D3). A sale line that leaves force keeps its row
//! (`is_active = 0`, `ended`), a stock move is undone by an opposite move, and an analytics fact
//! is taken back by a fact with sign −1.
//!
//! Sums of money are `SUM()` on `INTEGER` columns only (money agent Section 9); the caller adds
//! them with checked maths.

use mandi_core::accounts::RecordRef;
use rusqlite::{Connection, OptionalExtension, params};

/// A sale line of a posting that is in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaleRef {
    /// Row id.
    pub id: i64,
    /// Its lot.
    pub lot_id: i64,
}

/// The sale lines in force of one posting, in lot and line order.
///
/// # Errors
/// Any SQLite error.
pub fn active_sales_of_entry(conn: &Connection, entry_id: i64) -> rusqlite::Result<Vec<SaleRef>> {
    let mut statement = conn.prepare(
        "SELECT id, lot_id FROM bikri_sales
         WHERE journal_entry_id = ?1 AND is_active = 1 ORDER BY lot_id, line_no, id",
    )?;
    let rows = statement.query_map([entry_id], |r| {
        Ok(SaleRef {
            id: r.get(0)?,
            lot_id: r.get(1)?,
        })
    })?;
    rows.collect()
}

/// What the own lots of one posting took from their purchases: for each purchase, the نگ and the
/// cost (C21) of the sale lines in force. A reversal gives exactly this back (PRD FR-12.26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnRelease {
    /// The purchase.
    pub purchase_id: i64,
    /// نگ sold on those lines.
    pub qty: i64,
    /// The cost of those lines, paisa.
    pub cost_paisa: i64,
}

/// The own goods the lines in force of one posting took, by purchase.
///
/// # Errors
/// Any SQLite error.
pub fn own_release_of_entry(conn: &Connection, entry_id: i64) -> rusqlite::Result<Vec<OwnRelease>> {
    let mut statement = conn.prepare(
        "SELECT l.purchase_id, SUM(s.qty), SUM(s.cost_paisa)
         FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id
         WHERE s.journal_entry_id = ?1 AND s.is_active = 1 AND l.purchase_id IS NOT NULL
         GROUP BY l.purchase_id ORDER BY l.purchase_id",
    )?;
    let rows = statement.query_map([entry_id], |r| {
        Ok(OwnRelease {
            purchase_id: r.get(0)?,
            qty: r.get(1)?,
            cost_paisa: r.get(2)?,
        })
    })?;
    rows.collect()
}

/// The biggest line number the lot has had, in force or not (0 for a lot with no line). New lines
/// continue from it, so a line number is never used twice in a lot.
///
/// # Errors
/// Any SQLite error.
pub fn max_line_no(conn: &Connection, lot_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(line_no), 0) FROM bikri_sales WHERE lot_id = ?1",
        [lot_id],
        |r| r.get(0),
    )
}

/// How a sale line left force (`bikri_sales.ended`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    /// An edit wrote a new line in its place.
    Replaced,
    /// An edit took the line out.
    Removed,
    /// The whole بکری was cancelled.
    Cancelled,
}

impl Ended {
    const fn as_str(self) -> &'static str {
        match self {
            Ended::Replaced => "replaced",
            Ended::Removed => "removed",
            Ended::Cancelled => "cancelled",
        }
    }
}

/// Takes a sale line out of force, with who and when. `replaced_by` is the new line of an edit.
/// The line stays (Code Standards D3). Returns false if it was not in force.
///
/// # Errors
/// Any SQLite error (the triggers and CHECKs of `bikri_sales`).
pub fn end_sale(
    conn: &Connection,
    sale_id: i64,
    ended: Ended,
    replaced_by: Option<i64>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE bikri_sales SET is_active = 0, ended = ?2, replaced_by_sale_id = ?3,
             updated_at = ?4, updated_by = ?5, version = version + 1
         WHERE id = ?1 AND is_active = 1",
        params![sale_id, ended.as_str(), replaced_by, now, user_id],
    )?;
    Ok(changed > 0)
}

/// The sums over the sale lines in force of a lot, in all its postings (paisa and نگ).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LotSums {
    /// Σ qty.
    pub qty_sold: i64,
    /// Σ seller amounts.
    pub gross_paisa: i64,
    /// Σ seller commissions.
    pub comm_paisa: i64,
}

/// The sums of a lot's lines in force. With `except_entry`, the lines of that posting are left
/// out (what the lot sold elsewhere).
///
/// # Errors
/// Any SQLite error.
pub fn lot_sums(
    conn: &Connection,
    lot_id: i64,
    except_entry: Option<i64>,
) -> rusqlite::Result<LotSums> {
    conn.query_row(
        "SELECT COALESCE(SUM(qty), 0), COALESCE(SUM(seller_amount_paisa), 0),
                COALESCE(SUM(seller_comm_paisa), 0)
         FROM bikri_sales
         WHERE lot_id = ?1 AND is_active = 1 AND (?2 IS NULL OR journal_entry_id <> ?2)",
        params![lot_id, except_entry],
        |r| {
            Ok(LotSums {
                qty_sold: r.get(0)?,
                gross_paisa: r.get(1)?,
                comm_paisa: r.get(2)?,
            })
        },
    )
}

/// The sums of a lot's lines in ONE posting, in force.
///
/// # Errors
/// Any SQLite error.
pub fn lot_sums_of_entry(
    conn: &Connection,
    lot_id: i64,
    entry_id: i64,
) -> rusqlite::Result<LotSums> {
    conn.query_row(
        "SELECT COALESCE(SUM(qty), 0), COALESCE(SUM(seller_amount_paisa), 0),
                COALESCE(SUM(seller_comm_paisa), 0)
         FROM bikri_sales WHERE lot_id = ?1 AND journal_entry_id = ?2 AND is_active = 1",
        params![lot_id, entry_id],
        |r| {
            Ok(LotSums {
                qty_sold: r.get(0)?,
                gross_paisa: r.get(1)?,
                comm_paisa: r.get(2)?,
            })
        },
    )
}

/// A lot as the edit, the later sale and the write-off read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotState {
    /// Row id.
    pub id: i64,
    /// Its بکری.
    pub bikri_id: i64,
    /// The lot number, from 1.
    pub lot_no: i64,
    /// `saved` or `cancelled`.
    pub bikri_status: String,
    /// The بکری number.
    pub bikri_number: i64,
    /// The day of the first posting of the بکری, `YYYY-MM-DD`.
    pub received_date: String,
    /// The seller account.
    pub seller_account_id: i64,
    /// The item.
    pub item_id: i64,
    /// The packing.
    pub packing_id: i64,
    /// The مارکہ.
    pub marka_id: Option<i64>,
    /// The store.
    pub store_id: Option<i64>,
    /// نگ received.
    pub qty_received: i64,
    /// Weight received, grams.
    pub weight_received_grams: Option<i64>,
    /// نگ sold, over all postings in force.
    pub qty_sold: i64,
    /// نگ written off.
    pub qty_written_off: i64,
    /// The tick "پیشگی سے کاٹیں".
    pub adjust_advance: bool,
    /// The truck stand of the بکری.
    pub adda_id: Option<i64>,
    /// The one-time seller's names, phone and key.
    pub pukhta: Option<(String, String, String, String)>,
    /// The first posting of the lot.
    pub first_entry_id: Option<i64>,
    /// The freight typed on the lot, paisa.
    pub freight_paisa: i64,
    /// True if the firm paid the driver.
    pub freight_by_firm: bool,
    /// مزدوری, paisa.
    pub labour_paisa: i64,
    /// منشیانہ, paisa.
    pub munshiana_paisa: i64,
    /// فون, paisa.
    pub phone_paisa: i64,
    /// نقد, paisa.
    pub cash_paid_paisa: i64,
    /// کرایہ سٹور, paisa.
    pub store_rent_paisa: i64,
    /// The purchase an own lot took its goods from (PRD FR-12.26).
    pub purchase_id: Option<i64>,
}

/// One lot by id, with its بکری's status and day.
///
/// # Errors
/// Any SQLite error.
pub fn lot_state(conn: &Connection, lot_id: i64) -> rusqlite::Result<Option<LotState>> {
    conn.query_row(
        "SELECT l.id, l.bikri_id, l.lot_no, e.status, e.number, e.entry_date, l.seller_account_id,
                l.item_id, l.packing_id, l.marka_id, l.store_id, l.qty_received, l.qty_sold,
                l.qty_written_off, l.adjust_advance, l.pukhta_name_ur, l.pukhta_name_en,
                l.pukhta_phone, l.pukhta_person_key, l.first_entry_id, l.freight_paisa,
                l.freight_paid_by = 'firm', l.labour_paisa, l.munshiana_paisa, l.phone_paisa,
                l.cash_paid_paisa, l.store_rent_paisa, l.weight_received_grams, e.adda_id, l.purchase_id
         FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
         WHERE l.id = ?1",
        [lot_id],
        |r| {
            let key: Option<String> = r.get(18)?;
            let ur: Option<String> = r.get(15)?;
            let en: Option<String> = r.get(16)?;
            let phone: Option<String> = r.get(17)?;
            Ok(LotState {
                id: r.get(0)?,
                bikri_id: r.get(1)?,
                lot_no: r.get(2)?,
                bikri_status: r.get(3)?,
                bikri_number: r.get(4)?,
                received_date: r.get(5)?,
                seller_account_id: r.get(6)?,
                item_id: r.get(7)?,
                packing_id: r.get(8)?,
                marka_id: r.get(9)?,
                store_id: r.get(10)?,
                qty_received: r.get(11)?,
                qty_sold: r.get(12)?,
                qty_written_off: r.get(13)?,
                adjust_advance: r.get(14)?,
                pukhta: key.map(|key| {
                    (
                        ur.unwrap_or_default(),
                        en.unwrap_or_default(),
                        phone.unwrap_or_default(),
                        key,
                    )
                }),
                first_entry_id: r.get(19)?,
                freight_paisa: r.get(20)?,
                freight_by_firm: r.get(21)?,
                labour_paisa: r.get(22)?,
                munshiana_paisa: r.get(23)?,
                phone_paisa: r.get(24)?,
                cash_paid_paisa: r.get(25)?,
                store_rent_paisa: r.get(26)?,
                weight_received_grams: r.get(27)?,
                adda_id: r.get(28)?,
                purchase_id: r.get(29)?,
            })
        },
    )
    .optional()
}

/// The totals of a lot written back together (the table checks that deductions are the sum of the
/// charges and net is gross less deductions). One UPDATE, so the checks see a whole row.
#[derive(Debug, Clone, Copy)]
pub struct LotTotals {
    /// نگ sold in all postings in force.
    pub qty_sold: i64,
    /// خام بکری.
    pub gross_paisa: i64,
    /// The sum of the line commissions.
    pub seller_comm_paisa: i64,
    /// کل اخراجات.
    pub deductions_paisa: i64,
    /// صافی بکری.
    pub net_paisa: i64,
}

/// Writes the running totals of a lot in one update.
///
/// # Errors
/// Any SQLite error (the CHECKs of `bikri_lots`).
pub fn update_lot_totals(
    conn: &Connection,
    lot_id: i64,
    totals: &LotTotals,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE bikri_lots SET qty_sold = ?2, gross_paisa = ?3, seller_comm_paisa = ?4,
             deductions_paisa = ?5, net_paisa = ?6, updated_at = ?7, updated_by = ?8,
             version = version + 1
         WHERE id = ?1",
        params![
            lot_id,
            totals.qty_sold,
            totals.gross_paisa,
            totals.seller_comm_paisa,
            totals.deductions_paisa,
            totals.net_paisa,
            now,
            user_id
        ],
    )?;
    Ok(())
}

/// Rewrites a lot of the first posting in ONE update: its facts, its charges, its cash and its
/// totals (plan 16: "the lot totals are written in ONE update"). The بکری and the lot number stay.
///
/// # Errors
/// Any SQLite error (the CHECKs of `bikri_lots`).
pub fn update_lot(
    conn: &Connection,
    lot_id: i64,
    lot: &super::write::NewLot<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE bikri_lots SET seller_account_id = ?2, pukhta_name_ur = ?3, pukhta_name_en = ?4,
             pukhta_phone = ?5, pukhta_person_key = ?6, item_id = ?7, packing_id = ?8, marka_id = ?9,
             store_id = ?10, qty_received = ?11, weight_received_grams = ?12, freight_paisa = ?13,
             freight_paid_by = ?14, labour_paisa = ?15, labour_method = ?16, labour_value_paisa = ?17,
             labour_rule_id = ?18, munshiana_paisa = ?19, munshiana_method = ?20,
             munshiana_value_paisa = ?21, munshiana_rule_id = ?22, phone_paisa = ?23,
             phone_method = ?24, phone_value_paisa = ?25, phone_rule_id = ?26, cash_paid_paisa = ?27,
             store_rent_paisa = ?28, adjust_advance = ?29, qty_sold = ?30, gross_paisa = ?31,
             seller_comm_paisa = ?32, deductions_paisa = ?33, net_paisa = ?34, updated_at = ?35,
             updated_by = ?36, version = version + 1
         WHERE id = ?1",
        params![
            lot_id,
            lot.seller_account_id,
            lot.pukhta.map(|p| p.name_ur),
            lot.pukhta.map(|p| p.name_en),
            lot.pukhta.map(|p| p.phone),
            lot.pukhta.map(|p| p.person_key),
            lot.item_id,
            lot.packing_id,
            lot.marka_id,
            lot.store_id,
            lot.qty_received,
            lot.weight_received_grams,
            lot.freight_paisa,
            lot.freight_paid_by,
            lot.labour.paisa,
            lot.labour.method,
            lot.labour.value_paisa,
            lot.labour.rule_id,
            lot.munshiana.paisa,
            lot.munshiana.method,
            lot.munshiana.value_paisa,
            lot.munshiana.rule_id,
            lot.phone.paisa,
            lot.phone.method,
            lot.phone.value_paisa,
            lot.phone.rule_id,
            lot.cash_paid_paisa,
            lot.store_rent_paisa,
            lot.adjust_advance,
            lot.qty_sold,
            lot.gross_paisa,
            lot.seller_comm_paisa,
            lot.deductions_paisa,
            lot.net_paisa,
            now,
            user_id
        ],
    )?;
    Ok(())
}

/// The header columns an edit of the first posting writes back. The number and the day of the first
/// posting never change.
#[derive(Debug, Clone)]
pub struct EntryUpdate<'a> {
    /// The second number.
    pub book: Option<&'a str>,
    /// The bill day, `YYYY-MM-DD`.
    pub bill_date: &'a str,
    /// The truck number.
    pub truck_no: Option<&'a str>,
    /// The زمیندار.
    pub zamindar_account_id: Option<i64>,
    /// The truck stand.
    pub adda_id: Option<i64>,
    /// The driver's name.
    pub driver_name: Option<&'a str>,
    /// The driver's phone.
    pub driver_phone: Option<&'a str>,
    /// The header's total freight, paisa.
    pub total_freight_paisa: Option<i64>,
    /// The default store.
    pub store_id: Option<i64>,
    /// A note.
    pub note: Option<&'a str>,
}

/// Rewrites the header of a بکری and raises its version.
///
/// # Errors
/// Any SQLite error.
pub fn update_entry(
    conn: &Connection,
    bikri_id: i64,
    entry: &EntryUpdate<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE bikri_entries SET book = ?2, bill_date = ?3, truck_no = ?4, zamindar_account_id = ?5,
             adda_id = ?6, driver_name = ?7, driver_phone = ?8, total_freight_paisa = ?9,
             store_id = ?10, note = ?11, updated_at = ?12, updated_by = ?13, version = version + 1
         WHERE id = ?1",
        params![
            bikri_id,
            entry.book,
            entry.bill_date,
            entry.truck_no,
            entry.zamindar_account_id,
            entry.adda_id,
            entry.driver_name,
            entry.driver_phone,
            entry.total_freight_paisa,
            entry.store_id,
            entry.note,
            now,
            user_id
        ],
    )?;
    Ok(())
}

/// Raises the version of a بکری (an edit of a later posting, a later sale).
///
/// # Errors
/// Any SQLite error.
pub fn bump_version(
    conn: &Connection,
    bikri_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE bikri_entries SET updated_at = ?2, updated_by = ?3, version = version + 1
         WHERE id = ?1",
        params![bikri_id, now, user_id],
    )?;
    Ok(())
}

/// Marks a بکری cancelled with its reason, who and when, and raises its version (PRD FR-12.22).
/// The number and every row stay.
///
/// # Errors
/// Any SQLite error (a cancel needs a reason).
pub fn mark_cancelled(
    conn: &Connection,
    bikri_id: i64,
    reason: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE bikri_entries SET status = 'cancelled', cancel_reason = ?2, cancelled_by = ?3,
             cancelled_at = ?4, updated_at = ?4, updated_by = ?3, version = version + 1
         WHERE id = ?1",
        params![bikri_id, reason, user_id, now],
    )?;
    Ok(())
}

/// The version and status of a بکری, and its number.
///
/// # Errors
/// Any SQLite error.
pub fn entry_state(
    conn: &Connection,
    bikri_id: i64,
) -> rusqlite::Result<Option<(i64, String, i64)>> {
    conn.query_row(
        "SELECT number, status, version FROM bikri_entries WHERE id = ?1",
        [bikri_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .optional()
}

/// A posting of a بکری that is in force.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostingRef {
    /// The journal entry.
    pub entry_id: i64,
    /// `bikri_first` or `bikri_later`.
    pub posting_type: String,
    /// Its day, `YYYY-MM-DD`.
    pub entry_date: String,
}

/// The postings of a بکری that are in force (not reversed, not reversals), oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn postings_in_force(conn: &Connection, bikri_id: i64) -> rusqlite::Result<Vec<PostingRef>> {
    let mut statement = conn.prepare(
        "SELECT e.id, e.posting_type, e.entry_date FROM journal_entries e
         WHERE e.source_type = 'bikri' AND e.source_id = ?1 AND e.status = 'posted'
           AND e.posting_type IN ('bikri_first', 'bikri_later')
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
         ORDER BY e.id",
    )?;
    let rows = statement.query_map([bikri_id], |r| {
        Ok(PostingRef {
            entry_id: r.get(0)?,
            posting_type: r.get(1)?,
            entry_date: r.get(2)?,
        })
    })?;
    rows.collect()
}

/// The posting `entry_id`, if it belongs to this بکری and is a first or later posting in force.
///
/// # Errors
/// Any SQLite error.
pub fn posting_in_force(
    conn: &Connection,
    bikri_id: i64,
    entry_id: i64,
) -> rusqlite::Result<Option<PostingRef>> {
    Ok(postings_in_force(conn, bikri_id)?
        .into_iter()
        .find(|p| p.entry_id == entry_id))
}

/// The بکری a journal entry belongs to, if the entry is a posting of a بکری.
///
/// # Errors
/// Any SQLite error.
pub fn bikri_of_entry(conn: &Connection, entry_id: i64) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT source_id FROM journal_entries WHERE id = ?1 AND source_type = 'bikri'",
        [entry_id],
        |r| r.get(0),
    )
    .optional()
}

// ---- Stock moves ----

/// A stock move of a lot to write.
#[derive(Debug, Clone, Copy)]
pub struct NewMove<'a> {
    /// The lot, or `None` for a move of a purchase (a write-off of own goods).
    pub lot_id: Option<i64>,
    /// The purchase, for a move of trip goods.
    pub purchase_id: Option<i64>,
    /// C21: the cost of the trip goods this move releases, paisa (a write-off or a sale in another
    /// shop; negative for the reversal of one). 0 for every other move.
    pub cost_paisa: i64,
    /// The posting, or `None` for a write-off of consigned goods.
    pub entry_id: Option<i64>,
    /// The day, `YYYY-MM-DD`.
    pub move_date: &'a str,
    /// `received`, `sold`, `written_off` or `reversal`.
    pub kind: &'a str,
    /// Signed quantity.
    pub qty: i64,
    /// The reason of a write-off.
    pub reason: Option<&'a str>,
    /// The move this one reverses.
    pub reverses_move_id: Option<i64>,
}

/// Adds a stock move (append-only) and returns its id.
///
/// # Errors
/// Any SQLite error (the trigger `stock_moves_reversal_mirrors`).
pub fn insert_move(
    conn: &Connection,
    stock_move: &NewMove<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO stock_moves (created_at, created_by, updated_at, updated_by, lot_id, journal_entry_id,
             move_date, kind, qty, reason, reverses_move_id, purchase_id, cost_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            now,
            user_id,
            stock_move.lot_id,
            stock_move.entry_id,
            stock_move.move_date,
            stock_move.kind,
            stock_move.qty,
            stock_move.reason,
            stock_move.reverses_move_id,
            stock_move.purchase_id,
            stock_move.cost_paisa
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Takes back the stock moves of a reversed posting: one opposite move for each move of
/// `from_entry` that has not been reversed, pointing at the reversal entry `to_entry`. Returns how
/// many were written.
///
/// # Errors
/// Any SQLite error.
pub fn reverse_moves(
    conn: &Connection,
    from_entry: i64,
    to_entry: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO stock_moves (created_at, created_by, updated_at, updated_by, lot_id, journal_entry_id,
             move_date, kind, qty, reason, reverses_move_id, purchase_id, cost_paisa)
         SELECT ?3, ?4, ?3, ?4, m.lot_id, ?2, m.move_date, 'reversal', -m.qty, NULL, m.id, m.purchase_id,
             -m.cost_paisa
         FROM stock_moves m
         WHERE m.journal_entry_id = ?1 AND m.kind <> 'reversal'
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = m.id)
         ORDER BY m.id",
        params![from_entry, to_entry, now, user_id],
    )
}

/// Takes back the analytics facts of a reversed posting: one fact with the opposite sign for each
/// fact of `from_entry`, pointing at the reversal entry `to_entry` (PRD FR-18.7). Returns how many
/// were written.
///
/// # Errors
/// Any SQLite error.
pub fn reverse_facts(
    conn: &Connection,
    from_entry: i64,
    to_entry: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO analytics_facts (created_at, created_by, updated_at, updated_by, journal_entry_id,
             entry_date, year, month, bikri_id, lot_id, sale_id, item_id, seller_account_id,
             buyer_account_id, pukhta_person_key, city_id, adda_id, qty, seller_amount_paisa,
             seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa, laga_paisa, sign, purchase_id, venture_id, cost_paisa)
         SELECT ?3, ?4, ?3, ?4, ?2, f.entry_date, f.year, f.month, f.bikri_id, f.lot_id, f.sale_id,
             f.item_id, f.seller_account_id, f.buyer_account_id, f.pukhta_person_key, f.city_id,
             f.adda_id, f.qty, f.seller_amount_paisa, f.seller_comm_paisa, f.buyer_value_paisa,
             f.buyer_comm_paisa, f.laga_paisa, -f.sign, f.purchase_id, f.venture_id, f.cost_paisa
         FROM analytics_facts f
         WHERE f.journal_entry_id = ?1
         ORDER BY f.id",
        params![from_entry, to_entry, now, user_id],
    )
}

/// Writes the opposite of every trip partner's part of the posting `from_entry`, for its reversal
/// entry `to_entry` (PRD FR-12.26, C25). The sum of a lot's rows is what stands now.
///
/// # Errors
/// Any SQLite error.
pub fn reverse_credit_parts(
    conn: &Connection,
    from_entry: i64,
    to_entry: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO lot_credit_parts (created_at, created_by, updated_at, updated_by, lot_id,
             journal_entry_id, partner_id, share_bps, amount_paisa)
         SELECT ?3, ?4, ?3, ?4, c.lot_id, ?2, c.partner_id, c.share_bps, -c.amount_paisa
         FROM lot_credit_parts c
         WHERE c.journal_entry_id = ?1
         ORDER BY c.id",
        params![from_entry, to_entry, now, user_id],
    )
}

/// A write-off that has not been undone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteOff {
    /// The stock move.
    pub move_id: i64,
    /// The lot.
    pub lot_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub move_date: String,
    /// How many نگ (more than 0).
    pub qty: i64,
    /// The reason.
    pub reason: Option<String>,
}

fn read_write_off(r: &rusqlite::Row<'_>) -> rusqlite::Result<WriteOff> {
    let qty: i64 = r.get(3)?;
    Ok(WriteOff {
        move_id: r.get(0)?,
        lot_id: r.get(1)?,
        move_date: r.get(2)?,
        qty: qty.saturating_neg(),
        reason: r.get(4)?,
    })
}

/// A write-off by its move id, if it is one and has not been undone.
///
/// # Errors
/// Any SQLite error.
pub fn write_off(conn: &Connection, move_id: i64) -> rusqlite::Result<Option<WriteOff>> {
    conn.query_row(
        "SELECT m.id, m.lot_id, m.move_date, m.qty, m.reason FROM stock_moves m
         WHERE m.id = ?1 AND m.kind = 'written_off' AND m.lot_id IS NOT NULL
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = m.id)",
        [move_id],
        read_write_off,
    )
    .optional()
}

/// The write-offs of these lots that have not been undone, oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn write_offs_of_lots(conn: &Connection, lot_ids: &[i64]) -> rusqlite::Result<Vec<WriteOff>> {
    if lot_ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; lot_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT m.id, m.lot_id, m.move_date, m.qty, m.reason FROM stock_moves m
         WHERE m.lot_id IN ({marks}) AND m.kind = 'written_off'
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = m.id)
         ORDER BY m.move_date, m.id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(lot_ids), read_write_off)?;
    rows.collect()
}

/// Changes the written-off quantity of a lot by `delta` (plus for a write-off, minus for its
/// undo). The table checks that sold plus written off is never more than received.
///
/// # Errors
/// Any SQLite error.
pub fn add_written_off(
    conn: &Connection,
    lot_id: i64,
    delta: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE bikri_lots SET qty_written_off = qty_written_off + ?2, updated_at = ?3,
             updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![lot_id, delta, now, user_id],
    )?;
    Ok(())
}

// ---- The list of goods left (باقی مال) ----

/// What the list of goods left asks for. A field that is `None` is not a filter.
#[derive(Debug, Clone, Default)]
pub struct StockFilter {
    /// First day received, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day received, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// Only lots of this seller.
    pub seller_id: Option<i64>,
    /// Only lots of this item.
    pub item_id: Option<i64>,
    /// Only this lot.
    pub lot_id: Option<i64>,
    /// Also lots with a write-off that can be undone, even with nothing left.
    pub include_written_off: bool,
}

/// A lot of the list of goods left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockLot {
    /// Row id.
    pub lot_id: i64,
    /// The lot number.
    pub lot_no: i64,
    /// The بکری.
    pub bikri_id: i64,
    /// Its number.
    pub bikri_number: i64,
    /// The day received, `YYYY-MM-DD`.
    pub received_date: String,
    /// The seller account.
    pub seller: RecordRef,
    /// The one-time seller (Urdu name, English name, phone, key).
    pub pukhta: Option<(String, String, String, String)>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ received.
    pub qty_received: i64,
    /// نگ sold.
    pub qty_sold: i64,
    /// نگ written off.
    pub qty_written_off: i64,
}

/// The lots of saved (not cancelled) بکری with goods left, oldest received first (PRD FR-12.14).
///
/// # Errors
/// Any SQLite error.
pub fn stock_lots(conn: &Connection, filter: &StockFilter) -> rusqlite::Result<Vec<StockLot>> {
    let mut statement = conn.prepare(
        "SELECT l.id, l.lot_no, e.id, e.number, e.entry_date, sa.id, sa.code, sa.name_ur, sa.name_en,
                l.pukhta_name_ur, l.pukhta_name_en, l.pukhta_phone, l.pukhta_person_key,
                it.id, it.code, it.name_ur, it.name_en, pk.id, pk.code, pk.name_ur, pk.name_en,
                l.qty_received, l.qty_sold, l.qty_written_off
         FROM bikri_lots l
         JOIN bikri_entries e ON e.id = l.bikri_id
         JOIN accounts sa ON sa.id = l.seller_account_id
         JOIN items it ON it.id = l.item_id
         JOIN packings pk ON pk.id = l.packing_id
         WHERE e.status = 'saved'
           AND (?1 IS NULL OR e.entry_date >= ?1) AND (?2 IS NULL OR e.entry_date <= ?2)
           AND (?3 IS NULL OR l.seller_account_id = ?3) AND (?4 IS NULL OR l.item_id = ?4)
           AND (?6 IS NULL OR l.id = ?6)
           AND (l.qty_received - l.qty_sold - l.qty_written_off > 0
                OR (?5 = 1 AND EXISTS (SELECT 1 FROM stock_moves m
                                       WHERE m.lot_id = l.id AND m.kind = 'written_off'
                                         AND NOT EXISTS (SELECT 1 FROM stock_moves r
                                                         WHERE r.reverses_move_id = m.id))))
         ORDER BY e.entry_date, e.number, l.lot_no",
    )?;
    let rows = statement.query_map(
        params![
            filter.from,
            filter.to,
            filter.seller_id,
            filter.item_id,
            filter.include_written_off,
            filter.lot_id
        ],
        |r| {
            let record = |at: usize| -> rusqlite::Result<RecordRef> {
                Ok(RecordRef {
                    id: r.get(at)?,
                    code: r.get(at + 1)?,
                    name_ur: r.get(at + 2)?,
                    name_en: r.get(at + 3)?,
                })
            };
            let ur: Option<String> = r.get(9)?;
            let en: Option<String> = r.get(10)?;
            let phone: Option<String> = r.get(11)?;
            let key: Option<String> = r.get(12)?;
            Ok(StockLot {
                lot_id: r.get(0)?,
                lot_no: r.get(1)?,
                bikri_id: r.get(2)?,
                bikri_number: r.get(3)?,
                received_date: r.get(4)?,
                seller: record(5)?,
                pukhta: key.map(|key| {
                    (
                        ur.unwrap_or_default(),
                        en.unwrap_or_default(),
                        phone.unwrap_or_default(),
                        key,
                    )
                }),
                item: record(13)?,
                packing: record(17)?,
                qty_received: r.get(21)?,
                qty_sold: r.get(22)?,
                qty_written_off: r.get(23)?,
            })
        },
    )?;
    rows.collect()
}
