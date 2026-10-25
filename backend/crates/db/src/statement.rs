//! What a partner statement is made from, in SQL (PRD FR-17.9, FR-17.10, plan 17): the dated
//! amounts that reach a partner and the facts that word each line.
//!
//! No rules here. The statement service sorts the events, works the running کل رقم out with
//! checked maths and asks for the facts of one page only. Every read is a posted record: share
//! rows, journal lines, the stored parts of a trip lot's credit, stock moves and trips (so
//! nothing is typed twice, PRD FR-17.10). Sums that go to the user are `SUM()` on INTEGER columns
//! or checked sums in Rust; no `total()` or `avg()` (money agent Section 9).

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

/// The `?` marks for a list of ids.
fn marks(count: usize) -> String {
    vec!["?"; count].join(", ")
}

/// One share row of a partner: the day, the entry and the signed part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareEvent {
    /// Row id.
    pub id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The journal entry it belongs to.
    pub entry_id: i64,
    /// `income` or `expense`.
    pub kind: String,
    /// The signed part (income plus, expense minus; a reversal's rows are the opposite).
    pub amount_paisa: i64,
}

/// All share rows of a partner, in the order they were written.
///
/// # Errors
/// Any SQLite error.
pub fn share_events(conn: &Connection, partner_id: i64) -> rusqlite::Result<Vec<ShareEvent>> {
    let mut statement = conn.prepare(
        "SELECT id, entry_date, journal_entry_id, kind, amount_paisa FROM partner_share_rows
         WHERE partner_id = ?1 ORDER BY id",
    )?;
    let rows = statement.query_map([partner_id], |r| {
        Ok(ShareEvent {
            id: r.get(0)?,
            entry_date: r.get(1)?,
            entry_id: r.get(2)?,
            kind: r.get(3)?,
            amount_paisa: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// One journal line on one of a partner's accounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineEvent {
    /// The journal line.
    pub id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Its entry.
    pub entry_id: i64,
    /// The debit, paisa.
    pub dr_paisa: i64,
    /// The credit, paisa.
    pub cr_paisa: i64,
    /// The source of its entry (`voucher`, `bikri`, `account`, `purchase` ...).
    pub source_type: String,
}

/// The posted lines of an account, in the order they were written. The entries of month close
/// are left out: month close only moves profit into capital, so it never changes کل رقم (PRD
/// FR-17.6, FR-17.9).
///
/// # Errors
/// Any SQLite error.
pub fn line_events(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<LineEvent>> {
    let mut statement = conn.prepare(
        "SELECT l.id, e.entry_date, l.entry_id, l.dr_paisa, l.cr_paisa, e.source_type
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND e.status = 'posted' AND e.source_type <> 'month_close'
         ORDER BY l.id",
    )?;
    let rows = statement.query_map([account_id], |r| {
        Ok(LineEvent {
            id: r.get(0)?,
            entry_date: r.get(1)?,
            entry_id: r.get(2)?,
            dr_paisa: r.get(3)?,
            cr_paisa: r.get(4)?,
            source_type: r.get(5)?,
        })
    })?;
    rows.collect()
}

/// A trip partner's part of the seller credit of a trip lot (PRD C25), as stored with the posting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotCreditEvent {
    /// Row id.
    pub id: i64,
    /// The day of its entry, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The posting (or the reversal) it belongs to.
    pub entry_id: i64,
    /// The lot.
    pub lot_id: i64,
    /// His share, bps.
    pub share_bps: i64,
    /// His signed part, paisa.
    pub amount_paisa: i64,
}

/// The stored credit parts of a partner, in the order they were written.
///
/// # Errors
/// Any SQLite error.
pub fn lot_credit_events(
    conn: &Connection,
    partner_id: i64,
) -> rusqlite::Result<Vec<LotCreditEvent>> {
    let mut statement = conn.prepare(
        "SELECT c.id, e.entry_date, c.journal_entry_id, c.lot_id, c.share_bps, c.amount_paisa
         FROM lot_credit_parts c JOIN journal_entries e ON e.id = c.journal_entry_id
         WHERE c.partner_id = ?1 AND e.status = 'posted' ORDER BY c.id",
    )?;
    let rows = statement.query_map([partner_id], |r| {
        Ok(LotCreditEvent {
            id: r.get(0)?,
            entry_date: r.get(1)?,
            entry_id: r.get(2)?,
            lot_id: r.get(3)?,
            share_bps: r.get(4)?,
            amount_paisa: r.get(5)?,
        })
    })?;
    rows.collect()
}

/// A day and an id: the place of a note line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteEvent {
    /// The id of the thing the note is about (an entry, a stock move or a trip).
    pub id: i64,
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The trip, for the trip notes. 0 for month close.
    pub venture_id: i64,
}

/// The month closes that touched this capital account (PRD FR-17.6). A reversal of one is not
/// listed: the note says the profit was moved, and the total stays the same either way.
///
/// # Errors
/// Any SQLite error.
pub fn month_close_notes(conn: &Connection, capital_id: i64) -> rusqlite::Result<Vec<NoteEvent>> {
    let mut statement = conn.prepare(
        "SELECT DISTINCT e.id, e.entry_date FROM journal_entries e
         JOIN journal_lines l ON l.entry_id = e.id
         WHERE l.account_id = ?1 AND e.status = 'posted' AND e.source_type = 'month_close'
           AND e.reverses_entry_id IS NULL
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
         ORDER BY e.entry_date, e.id",
    )?;
    let rows = statement.query_map([capital_id], |r| {
        Ok(NoteEvent {
            id: r.get(0)?,
            date: r.get(1)?,
            venture_id: 0,
        })
    })?;
    rows.collect()
}

/// The write-offs of trip goods, not undone, on trips the partner is in (PRD FR-12.28).
///
/// # Errors
/// Any SQLite error.
pub fn write_off_notes(conn: &Connection, partner_id: i64) -> rusqlite::Result<Vec<NoteEvent>> {
    let mut statement = conn.prepare(
        "SELECT m.id, m.move_date, pu.venture_id
         FROM stock_moves m
         JOIN purchases pu ON pu.id = m.purchase_id
         JOIN venture_partners vp ON vp.venture_id = pu.venture_id AND vp.partner_id = ?1
         WHERE m.kind = 'written_off' AND pu.status = 'saved'
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = m.id)
         ORDER BY m.move_date, m.id",
    )?;
    let rows = statement.query_map([partner_id], |r| {
        Ok(NoteEvent {
            id: r.get(0)?,
            date: r.get(1)?,
            venture_id: r.get(2)?,
        })
    })?;
    rows.collect()
}

/// The trips the partner is in that are closed now (PRD FR-12.33). `id` is the trip.
///
/// # Errors
/// Any SQLite error.
pub fn closed_trip_notes(conn: &Connection, partner_id: i64) -> rusqlite::Result<Vec<NoteEvent>> {
    let mut statement = conn.prepare(
        "SELECT v.id, v.closed_date FROM ventures v
         JOIN venture_partners vp ON vp.venture_id = v.id AND vp.partner_id = ?1
         WHERE v.kind = 'trip' AND v.status = 'closed' AND v.closed_date IS NOT NULL
         ORDER BY v.closed_date, v.id",
    )?;
    let rows = statement.query_map([partner_id], |r| {
        Ok(NoteEvent {
            id: r.get(0)?,
            date: r.get(1)?,
            venture_id: r.get(0)?,
        })
    })?;
    rows.collect()
}

/// The garden closings that posted nothing for this partner: a balance of 0, or a part of his that
/// came to 0, so the closing has no line on his 0604 account (PRD FR-17.10, E64). They become note
/// lines with no amount, so the statement still shows the garden's cost, sales, result and his
/// share. `id` is the closing's venture entry, `venture_id` the garden. A closing that was undone is
/// left out.
///
/// # Errors
/// Any SQLite error.
pub fn unposted_garden_close_notes(
    conn: &Connection,
    partner_id: i64,
) -> rusqlite::Result<Vec<NoteEvent>> {
    let mut statement = conn.prepare(
        "SELECT x.id, x.entry_date, x.venture_id
         FROM venture_entries x
         JOIN venture_entry_parts p ON p.entry_id = x.id AND p.partner_id = ?1
         JOIN partners pa ON pa.id = ?1
         WHERE x.kind = 'close' AND x.status = 'saved'
           AND NOT EXISTS (SELECT 1 FROM journal_lines l
                           WHERE l.entry_id = x.journal_entry_id
                             AND l.account_id = pa.venture_account_id)
         ORDER BY x.entry_date, x.id",
    )?;
    let rows = statement.query_map([partner_id], |r| {
        Ok(NoteEvent {
            id: r.get(0)?,
            date: r.get(1)?,
            venture_id: r.get(2)?,
        })
    })?;
    rows.collect()
}

// ---- The facts that word the lines of one page ----

/// A named record.
pub type Named = (i64, String, String, String);

/// What the statement needs to know about an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryInfo {
    /// The entry.
    pub entry_id: i64,
    /// Its posting type, stored form.
    pub posting_type: String,
    /// What it belongs to: `bikri`, `voucher`, `purchase`, `venture`, `fruit`, `account`.
    pub source_type: String,
    /// The id of that record.
    pub source_id: i64,
    /// The entry it reverses, if it is a reversal.
    pub reverses_entry_id: Option<i64>,
    /// The posting type of the entry it reverses.
    pub original_type: Option<String>,
}

/// The entries with these ids.
///
/// # Errors
/// Any SQLite error.
pub fn entry_infos(conn: &Connection, ids: &[i64]) -> rusqlite::Result<HashMap<i64, EntryInfo>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT e.id, e.posting_type, e.source_type, e.source_id, e.reverses_entry_id, o.posting_type
         FROM journal_entries e LEFT JOIN journal_entries o ON o.id = e.reverses_entry_id
         WHERE e.id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        Ok(EntryInfo {
            entry_id: r.get(0)?,
            posting_type: r.get(1)?,
            source_type: r.get(2)?,
            source_id: r.get(3)?,
            reverses_entry_id: r.get(4)?,
            original_type: r.get(5)?,
        })
    })?;
    for row in rows {
        let info = row?;
        out.insert(info.entry_id, info);
    }
    Ok(out)
}

/// A voucher as the statement words it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoucherInfo {
    /// The voucher.
    pub id: i64,
    /// Its type, stored form.
    pub voucher_type: String,
    /// Its number in its series.
    pub number: i64,
    /// Its reason (for a shop expense: whom it was paid to). Empty if none.
    pub reason: String,
    /// The expense head, if it has one: `(id, code, name_ur, name_en)`.
    pub head: Option<Named>,
}

/// The vouchers with these ids.
///
/// # Errors
/// Any SQLite error.
pub fn voucher_infos(
    conn: &Connection,
    ids: &[i64],
) -> rusqlite::Result<HashMap<i64, VoucherInfo>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT v.id, v.type, v.number, COALESCE(v.reason, ''), h.id, h.code, h.name_ur, h.name_en
         FROM vouchers v LEFT JOIN expense_heads h ON h.id = v.expense_head_id
         WHERE v.id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        let head_id: Option<i64> = r.get(4)?;
        let head = match head_id {
            Some(id) => Some((id, r.get(5)?, r.get(6)?, r.get(7)?)),
            None => None,
        };
        Ok(VoucherInfo {
            id: r.get(0)?,
            voucher_type: r.get(1)?,
            number: r.get(2)?,
            reason: r.get(3)?,
            head,
        })
    })?;
    for row in rows {
        let info = row?;
        out.insert(info.id, info);
    }
    Ok(out)
}

/// The numbers of these بکری.
///
/// # Errors
/// Any SQLite error.
pub fn bikri_numbers(conn: &Connection, ids: &[i64]) -> rusqlite::Result<HashMap<i64, i64>> {
    numbers_of(conn, "bikri_entries", ids)
}

/// What the statement says about a fruit entry (PRD FR-17.10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FruitInfo {
    /// Its number in the پھل series.
    pub number: i64,
    /// The item.
    pub item: Named,
    /// The packing.
    pub packing: Named,
    /// نگ taken.
    pub qty: i64,
    /// The rate, paisa.
    pub rate_paisa: i64,
    /// The value, paisa.
    pub value_paisa: i64,
    /// True for the shop's guests.
    pub guests: bool,
    /// The posting in force (after a cancel, the posting that was reversed). The row holds the values
    /// of this posting only; an older posting of an edited entry has no details (an edit updates the
    /// row).
    pub current_entry_id: Option<i64>,
}

/// The fruit entries with these ids.
///
/// # Errors
/// Any SQLite error.
pub fn fruit_infos(conn: &Connection, ids: &[i64]) -> rusqlite::Result<HashMap<i64, FruitInfo>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT f.id, f.number, it.id, it.code, it.name_ur, it.name_en,
                pk.id, pk.code, pk.name_ur, pk.name_en, f.qty, f.rate_paisa, f.value_paisa,
                f.partner_id IS NULL, f.current_entry_id
         FROM fruit_takes f JOIN items it ON it.id = f.item_id JOIN packings pk ON pk.id = f.packing_id
         WHERE f.id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            FruitInfo {
                number: r.get(1)?,
                item: (r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?),
                packing: (r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?),
                qty: r.get(10)?,
                rate_paisa: r.get(11)?,
                value_paisa: r.get(12)?,
                guests: r.get(13)?,
                current_entry_id: r.get(14)?,
            },
        ))
    })?;
    for row in rows {
        let (id, info) = row?;
        out.insert(id, info);
    }
    Ok(out)
}

fn numbers_of(conn: &Connection, table: &str, ids: &[i64]) -> rusqlite::Result<HashMap<i64, i64>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT id, number FROM {table} WHERE id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| Ok((r.get(0)?, r.get(1)?)))?;
    for row in rows {
        let (id, number) = row?;
        out.insert(id, number);
    }
    Ok(out)
}

/// The number and kind of these ventures (a garden or a trip).
///
/// # Errors
/// Any SQLite error.
pub fn venture_numbers(
    conn: &Connection,
    ids: &[i64],
) -> rusqlite::Result<HashMap<i64, (i64, String)>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT id, number, kind FROM ventures WHERE id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        Ok((r.get(0)?, (r.get(1)?, r.get(2)?)))
    })?;
    for row in rows {
        let (id, value) = row?;
        out.insert(id, value);
    }
    Ok(out)
}

/// A purchase of a trip as the statement words it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurchaseInfo {
    /// The purchase.
    pub id: i64,
    /// Its number (خ-n).
    pub number: i64,
    /// Its trip.
    pub venture_id: i64,
    /// Its item.
    pub item: Named,
}

/// The purchases with these ids.
///
/// # Errors
/// Any SQLite error.
pub fn purchase_infos(
    conn: &Connection,
    ids: &[i64],
) -> rusqlite::Result<HashMap<i64, PurchaseInfo>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT p.id, p.number, p.venture_id, i.id, i.code, i.name_ur, i.name_en
         FROM purchases p JOIN items i ON i.id = p.item_id WHERE p.id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        Ok(PurchaseInfo {
            id: r.get(0)?,
            number: r.get(1)?,
            venture_id: r.get(2)?,
            item: (r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?),
        })
    })?;
    for row in rows {
        let info = row?;
        out.insert(info.id, info);
    }
    Ok(out)
}

/// An entry of a garden or a trip as the statement words it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VentureEntryInfo {
    /// The venture entry.
    pub id: i64,
    /// `installment`, `cost`, `purchase`, `other_sale` or `close`.
    pub kind: String,
    /// Its number in its series.
    pub number: i64,
    /// Its garden or trip.
    pub venture_id: i64,
    /// The نگ sold, for a sale in another shop.
    pub qty: Option<i64>,
    /// The payment, or the money received, paisa.
    pub amount_paisa: i64,
    /// The cost of the goods sold, paisa.
    pub cost_paisa: i64,
    /// The purchase it is about.
    pub purchase_id: Option<i64>,
    /// Why. Empty if none.
    pub reason: String,
}

/// The venture entries with these ids.
///
/// # Errors
/// Any SQLite error.
pub fn venture_entry_infos(
    conn: &Connection,
    ids: &[i64],
) -> rusqlite::Result<HashMap<i64, VentureEntryInfo>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT id, kind, number, venture_id, qty, amount_paisa, cost_paisa, purchase_id,
                COALESCE(reason, '')
         FROM venture_entries WHERE id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        Ok(VentureEntryInfo {
            id: r.get(0)?,
            kind: r.get(1)?,
            number: r.get(2)?,
            venture_id: r.get(3)?,
            qty: r.get(4)?,
            amount_paisa: r.get(5)?,
            cost_paisa: r.get(6)?,
            purchase_id: r.get(7)?,
            reason: r.get(8)?,
        })
    })?;
    for row in rows {
        let info = row?;
        out.insert(info.id, info);
    }
    Ok(out)
}

/// The share of a partner in these venture entries (his part of the money, PRD C23, C25), bps.
///
/// # Errors
/// Any SQLite error.
pub fn entry_part_shares(
    conn: &Connection,
    partner_id: i64,
    entry_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, i64>> {
    let mut out = HashMap::new();
    if entry_ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT entry_id, share_bps FROM venture_entry_parts
         WHERE partner_id = ? AND entry_id IN ({})",
        marks(entry_ids.len())
    ))?;
    let mut values = vec![partner_id];
    values.extend_from_slice(entry_ids);
    let rows = statement.query_map(params_from_iter(values.iter()), |r| {
        Ok((r.get(0)?, r.get(1)?))
    })?;
    for row in rows {
        let (id, share) = row?;
        out.insert(id, share);
    }
    Ok(out)
}

/// The stored figures of a garden's closing as one partner's statement words it (PRD FR-17.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GardenCloseFigures {
    /// Σ of the garden's live installments and costs: the total cost, paisa.
    pub cost_paisa: i64,
    /// Σ of the closing's parts, signed: the money the garden brought in, paisa.
    pub sales_paisa: i64,
    /// What the partner paid into the garden, from the shop and from his pocket, paisa.
    pub paid_paisa: i64,
    /// His share, bps.
    pub share_bps: i64,
    /// His part of the closing, signed: his money back, paisa.
    pub back_paisa: i64,
}

/// The figures of these closings for one partner, read from stored rows only. A closing the partner
/// has no part in is left out.
///
/// # Errors
/// Any SQLite error.
pub fn garden_close_figures(
    conn: &Connection,
    partner_id: i64,
    close_entry_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, GardenCloseFigures>> {
    let mut out = HashMap::new();
    for id in close_entry_ids {
        let Some((garden_id, share_bps, back)) = conn
            .query_row(
                "SELECT x.venture_id, p.share_bps, p.amount_paisa
                 FROM venture_entries x JOIN venture_entry_parts p ON p.entry_id = x.id
                 WHERE x.id = ?1 AND x.kind = 'close' AND p.partner_id = ?2",
                params![id, partner_id],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()?
        else {
            continue;
        };
        let cost: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM venture_entries
             WHERE venture_id = ?1 AND status = 'saved' AND kind IN ('installment', 'cost')",
            [garden_id],
            |r| r.get(0),
        )?;
        let sales: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM venture_entry_parts WHERE entry_id = ?1",
            [id],
            |r| r.get(0),
        )?;
        // What he paid is read by the kind of the entry, never by the source alone.
        let paid: i64 = conn.query_row(
            "SELECT COALESCE(SUM(p.amount_paisa), 0)
             FROM venture_entry_parts p JOIN venture_entries x ON x.id = p.entry_id
             WHERE x.venture_id = ?1 AND x.status = 'saved' AND x.kind IN ('installment', 'cost')
               AND p.partner_id = ?2",
            params![garden_id, partner_id],
            |r| r.get(0),
        )?;
        out.insert(
            *id,
            GardenCloseFigures {
                cost_paisa: cost,
                sales_paisa: sales,
                paid_paisa: paid,
                share_bps,
                back_paisa: back,
            },
        );
    }
    Ok(out)
}

/// A lot of the statement: its بکری, its purchase and its item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotInfo {
    /// The lot.
    pub lot_id: i64,
    /// The number of its بکری.
    pub bikri_number: i64,
    /// The purchase it sells (a trip lot), if any.
    pub purchase_id: Option<i64>,
    /// Its item.
    pub item: Named,
}

/// The lots with these ids.
///
/// # Errors
/// Any SQLite error.
pub fn lot_infos(conn: &Connection, ids: &[i64]) -> rusqlite::Result<HashMap<i64, LotInfo>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, b.number, l.purchase_id, i.id, i.code, i.name_ur, i.name_en
         FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id JOIN items i ON i.id = l.item_id
         WHERE l.id IN ({})",
        marks(ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        Ok(LotInfo {
            lot_id: r.get(0)?,
            bikri_number: r.get(1)?,
            purchase_id: r.get(2)?,
            item: (r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?),
        })
    })?;
    for row in rows {
        let info = row?;
        out.insert(info.lot_id, info);
    }
    Ok(out)
}

/// What the lines of a lot sold in one posting add up to: the نگ and the cost of the goods (PRD
/// C21), by `(posting entry, lot)`.
///
/// # Errors
/// Any SQLite error.
pub fn lot_sold_figures(
    conn: &Connection,
    entry_ids: &[i64],
) -> rusqlite::Result<HashMap<(i64, i64), (i64, i64)>> {
    let mut out = HashMap::new();
    if entry_ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT journal_entry_id, lot_id, COALESCE(SUM(qty), 0), COALESCE(SUM(cost_paisa), 0)
         FROM bikri_sales WHERE journal_entry_id IN ({}) GROUP BY journal_entry_id, lot_id",
        marks(entry_ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(entry_ids.iter()), |r| {
        Ok(((r.get(0)?, r.get(1)?), (r.get(2)?, r.get(3)?)))
    })?;
    for row in rows {
        let (key, value) = row?;
        out.insert(key, value);
    }
    Ok(out)
}

/// What all the partners of a trip got from a lot in one posting (PRD C25), by
/// `(posting entry, lot)`: the seller credit of the lot, as stored.
///
/// # Errors
/// Any SQLite error.
pub fn lot_credit_totals(
    conn: &Connection,
    entry_ids: &[i64],
) -> rusqlite::Result<HashMap<(i64, i64), i64>> {
    let mut out = HashMap::new();
    if entry_ids.is_empty() {
        return Ok(out);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT journal_entry_id, lot_id, COALESCE(SUM(amount_paisa), 0) FROM lot_credit_parts
         WHERE journal_entry_id IN ({}) GROUP BY journal_entry_id, lot_id",
        marks(entry_ids.len())
    ))?;
    let rows = statement.query_map(params_from_iter(entry_ids.iter()), |r| {
        Ok(((r.get(0)?, r.get(1)?), r.get(2)?))
    })?;
    for row in rows {
        let (key, value) = row?;
        out.insert(key, value);
    }
    Ok(out)
}

/// The totals of a trip for its closing note (PRD FR-12.33): the cost of all its saved purchases,
/// the sales money of its own lots, the money of its other-shop sales and the cost released, all in
/// paisa. The note's result and net are worked out from these in Rust.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TripTotals {
    /// Σ total cost of the saved purchases.
    pub cost_paisa: i64,
    /// Σ cost released by sales and write-offs.
    pub released_paisa: i64,
    /// Σ seller credit parts of the own lots (reversals counted).
    pub sales_paisa: i64,
    /// Σ money of the sales in other shops.
    pub other_shop_paisa: i64,
}

/// The totals of a trip.
///
/// # Errors
/// Any SQLite error.
pub fn trip_totals(conn: &Connection, venture_id: i64) -> rusqlite::Result<TripTotals> {
    let (cost_paisa, released_paisa): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(total_cost_paisa), 0), COALESCE(SUM(cost_released_paisa), 0)
         FROM purchases WHERE venture_id = ?1 AND status = 'saved'",
        params![venture_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let sales_paisa: i64 = conn.query_row(
        "SELECT COALESCE(SUM(c.amount_paisa), 0)
         FROM lot_credit_parts c JOIN bikri_lots l ON l.id = c.lot_id
         JOIN purchases pu ON pu.id = l.purchase_id
         WHERE pu.venture_id = ?1 AND pu.status = 'saved'",
        params![venture_id],
        |r| r.get(0),
    )?;
    let other_shop_paisa: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_paisa), 0) FROM venture_entries
         WHERE venture_id = ?1 AND kind = 'other_sale' AND status = 'saved'",
        params![venture_id],
        |r| r.get(0),
    )?;
    Ok(TripTotals {
        cost_paisa,
        released_paisa,
        sales_paisa,
        other_shop_paisa,
    })
}
