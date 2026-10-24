//! The lines of a party's day read for the slip: sale lines, credit lines and the sums of the
//! day by kind of posting (R35).

use super::activity::SAME_DAY_PAIR;
use super::*;

/// One line of a buyer's slip.
pub(super) struct SaleLine {
    pub(super) qty: i64,
    pub(super) packing_ur: String,
    pub(super) packing_en: String,
    pub(super) item_ur: String,
    pub(super) item_en: String,
    pub(super) rate: Paisa,
    pub(super) total: Paisa,
    pub(super) cash: bool,
    pub(super) bikri_id: i64,
}

/// The sale lines of the day of a buyer (and one-time person), in the order they were saved.
pub(super) fn sale_lines(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
) -> Result<Vec<SaleLine>, ReportsError> {
    let mut statement = conn.prepare(
        "SELECT s.qty, pk.name_ur, pk.name_en, i.name_ur, i.name_en, s.buyer_rate_paisa,
                s.buyer_total_paisa, s.payment_type = 'cash', s.bikri_id
         FROM bikri_sales s
         JOIN bikri_lots l ON l.id = s.lot_id
         JOIN items i ON i.id = l.item_id
         JOIN packings pk ON pk.id = l.packing_id
         WHERE s.is_active = 1 AND s.entry_date = ?1 AND s.buyer_account_id = ?2
           AND (?3 IS NULL OR s.pukhta_person_key = ?3)
         ORDER BY s.bikri_id, s.line_no",
    )?;
    let rows = statement
        .query_map(params![day, account, person_key], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, bool>(7)?,
                r.get::<_, i64>(8)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (qty, packing_ur, packing_en, item_ur, item_en, rate, total, cash, bikri_id) in rows {
        out.push(SaleLine {
            qty,
            packing_ur,
            packing_en,
            item_ur,
            item_en,
            rate: paisa(rate)?,
            total: paisa(total)?,
            cash,
            bikri_id,
        });
    }
    Ok(out)
}

/// A lot of a posting: the quantity sold, the names of its packing and item, and the sums that
/// make its credit (sales, their commission, the lot charges of the first posting).
type LotRow = (i64, (String, String, String, String), [i64; 3]);

/// One credit line of a seller's slip: the posting's line with the lot it belongs to, when the
/// lots of the posting can be matched to the lines one for one.
pub(super) struct CreditLine {
    pub(super) bikri_id: i64,
    pub(super) amount: Paisa,
    pub(super) qty: Option<i64>,
    pub(super) names: Option<(String, String, String, String)>,
}

/// The seller's credit lines of بکری postings dated the day, with each lot's item and the quantity
/// sold in that posting. The lots of a posting that belong to the seller, in lot order, are
/// matched to his credit lines of the posting, in line order, only when there are as many lots
/// as lines; otherwise the item and quantity are left empty (nothing is guessed).
pub(super) fn credit_lines(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
) -> Result<Vec<CreditLine>, ReportsError> {
    let sql = format!(
        "SELECT e.id, e.source_id, l.cr_paisa
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?2 AND (?3 IS NULL OR l.pukhta_person_key = ?3)
           AND e.entry_date = ?1 AND e.status = 'posted' AND l.cr_paisa > 0
           AND e.source_type = 'bikri' AND e.posting_type IN ('bikri_first', 'bikri_later')
           AND NOT ({SAME_DAY_PAIR})
         ORDER BY e.id, l.line_no"
    );
    let mut statement = conn.prepare(&sql)?;
    let lines = statement
        .query_map(params![day, account, person_key], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    // For each lot of the posting: the quantity sold in it, its item, and whether its credit is
    // more than 0 (a lot with a net of 0 or less has no credit line, PRD FR-12.7). The credit is
    // the sales less their commission, and in the first posting less the lot charges (C8).
    let mut lots_stmt = conn.prepare(
        "SELECT SUM(s.qty), pk.name_ur, pk.name_en, i.name_ur, i.name_en,
                SUM(s.seller_amount_paisa), SUM(s.seller_comm_paisa),
                CASE WHEN l.first_entry_id = ?1
                     THEN l.labour_paisa + l.munshiana_paisa + l.phone_paisa + l.store_rent_paisa
                          + CASE WHEN l.freight_paid_by = 'firm' THEN l.freight_paisa ELSE 0 END
                     ELSE 0 END
         FROM bikri_lots l
         JOIN bikri_sales s ON s.lot_id = l.id AND s.is_active = 1 AND s.journal_entry_id = ?1
         JOIN items i ON i.id = l.item_id
         JOIN packings pk ON pk.id = l.packing_id
         WHERE l.bikri_id = ?2 AND l.seller_account_id = ?3
           AND (?4 IS NULL OR l.pukhta_person_key = ?4)
         GROUP BY l.id ORDER BY l.lot_no",
    )?;
    let mut out = Vec::with_capacity(lines.len());
    let mut at = 0_usize;
    while at < lines.len() {
        let Some(&(entry_id, bikri_id, _)) = lines.get(at) else {
            break;
        };
        let mut end = at;
        while lines.get(end).is_some_and(|l| l.0 == entry_id) {
            end = end.saturating_add(1);
        }
        let group = lines.get(at..end).unwrap_or_default();
        let lots = lots_stmt
            .query_map(params![entry_id, bikri_id, account, person_key], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    (
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                    ),
                    [
                        r.get::<_, i64>(5)?,
                        r.get::<_, i64>(6)?,
                        r.get::<_, i64>(7)?,
                    ],
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        // One credit line for each lot with a credit, in lot order. When the lots and the lines
        // do not match one for one, only the lots with a credit are tried; if that does not match
        // either, the item and quantity are left empty (nothing is guessed).
        let mut with_credit = Vec::new();
        for lot in &lots {
            let credit = paisa(lot.2[0])?
                .checked_sub(paisa(lot.2[1])?)?
                .checked_sub(paisa(lot.2[2])?)?;
            if credit > Paisa::ZERO {
                with_credit.push(lot);
            }
        }
        let candidates: Vec<&LotRow> = if lots.len() == group.len() {
            lots.iter().collect()
        } else if with_credit.len() == group.len() {
            with_credit
        } else {
            Vec::new()
        };
        for (index, (_, _, credit)) in group.iter().enumerate() {
            let lot = candidates.get(index).copied();
            out.push(CreditLine {
                bikri_id,
                amount: paisa(*credit)?,
                qty: lot.map(|l| l.0),
                names: lot.map(|l| l.1.clone()),
            });
        }
        at = end;
    }
    Ok(out)
}

/// The نقد given to a seller on the day: the cash of the lots whose first posting is dated the
/// day (stored on the lot, PRD FR-12.8), leaving out a posting that was cancelled or edited the
/// same day. The debit lines of a بکری posting are this cash and, for a lot whose net is
/// negative (PRD FR-12.7), the seller's debit; only the first is cash he got.
pub(super) fn cash_given(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
) -> Result<Paisa, ReportsError> {
    let sql = format!(
        "SELECT COALESCE(SUM(l.cash_paid_paisa), 0)
         FROM bikri_lots l JOIN journal_entries e ON e.id = l.first_entry_id
         WHERE l.seller_account_id = ?2 AND (?3 IS NULL OR l.pukhta_person_key = ?3)
           AND e.entry_date = ?1 AND e.status = 'posted' AND NOT ({SAME_DAY_PAIR})"
    );
    let total: i64 = conn.query_row(&sql, params![day, account, person_key], |r| r.get(0))?;
    paisa(total)
}

/// The sum of the debits (or credits) of the day on the party that came from بکری postings: the
/// cash part of a نقدی line (a credit on the buyer) and the debits of a seller.
pub(super) fn bikri_side(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
    credit: bool,
) -> Result<Paisa, ReportsError> {
    let column = if credit { "l.cr_paisa" } else { "l.dr_paisa" };
    let sql = format!(
        "SELECT COALESCE(SUM({column}), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?2 AND (?3 IS NULL OR l.pukhta_person_key = ?3)
           AND e.entry_date = ?1 AND e.status = 'posted' AND e.source_type = 'bikri'
           AND e.posting_type IN ('bikri_first', 'bikri_later') AND NOT ({SAME_DAY_PAIR})"
    );
    let total: i64 = conn.query_row(&sql, params![day, account, person_key], |r| r.get(0))?;
    paisa(total)
}

/// The sum of the debits (or credits) of the day on the party that came from postings of one
/// kind: the receipts a buyer paid (credits of `receipt`), the payments made to a seller (debits
/// of `payment`). A posting cancelled or edited the same day is left out with its reversal.
pub(super) fn posting_sum(
    conn: &Connection,
    day: &str,
    account: i64,
    person_key: Option<&str>,
    posting_type: &str,
    credit: bool,
) -> Result<Paisa, ReportsError> {
    let column = if credit { "l.cr_paisa" } else { "l.dr_paisa" };
    let sql = format!(
        "SELECT COALESCE(SUM({column}), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?2 AND (?3 IS NULL OR l.pukhta_person_key = ?3)
           AND e.entry_date = ?1 AND e.status = 'posted' AND e.posting_type = ?4
           AND NOT ({SAME_DAY_PAIR})"
    );
    let total: i64 =
        conn.query_row(&sql, params![day, account, person_key, posting_type], |r| {
            r.get(0)
        })?;
    paisa(total)
}
