//! The items behind a ledger line (PRD FR-19.8, E2E-33).
//!
//! A buyer's debit line of a بکری shows the sale line it came from: item, packing, quantity and
//! rate, then the stored commission and لاگا. A seller's credit line shows its lot: item,
//! quantity, خام بکری and the deductions. Cash lent to a buyer shows its reason (FR-15.11).
//!
//! The journal line does not store which sale line it was built from. The entry builder writes
//! one buyer debit for each sale line, in the order of `(lot, line)`, and one seller credit for
//! each lot (`domain::ledger::bikri`). So the n-th debit line of an account in the entry is the
//! n-th sale line of that buyer. The pairing is checked against the stored amount: a line whose
//! stored buyer total or lot counts do not match is shown without items, never with wrong ones.
//! A reversal mirrors line for line, so a reversal line is paired through the line of the entry
//! it reverses.

use std::collections::HashMap;

use mandi_core::money::Paisa;
use mandi_db::rusqlite::{Connection, params_from_iter};

use crate::error::ReportsError;
use crate::reports::common::{IN_CHUNK, paisa, placeholders};
use crate::reports::model::{DetailPart, Names};

/// A ledger line that may have items.
#[derive(Debug, Clone)]
pub(super) struct DetailQuery {
    pub(super) entry_id: i64,
    pub(super) line_no: i64,
    /// For a line of a بکری entry: that entry, or the entry it reverses.
    pub(super) bikri_base_entry: Option<i64>,
    /// For a line of a voucher entry: the voucher type and its reason.
    pub(super) voucher: Option<(String, Option<String>)>,
}

struct BaseLine {
    line_no: i64,
    account_id: i64,
    person_key: Option<String>,
    dr: i64,
    cr: i64,
}

struct SaleRow {
    lot_id: i64,
    buyer_account_id: i64,
    person_key: Option<String>,
    qty: i64,
    buyer_rate: i64,
    buyer_value: i64,
    buyer_comm: i64,
    laga: i64,
    buyer_total: i64,
    seller_amount: i64,
    lot_seller_account_id: i64,
    lot_person_key: Option<String>,
    item: Names,
    packing: Names,
    bikri_number: i64,
}

/// The items of the lines asked for, by `(entry id, line no)`. A line with nothing to show has
/// no entry in the map.
///
/// # Errors
/// A database error; `ReportsError::Money` for a stored amount that is not whole rupees.
pub(super) fn load(
    conn: &Connection,
    queries: &[DetailQuery],
    ledger_is_buyer: bool,
) -> Result<HashMap<(i64, i64), Vec<DetailPart>>, ReportsError> {
    let mut found: HashMap<(i64, i64), Vec<DetailPart>> = HashMap::new();

    // Vouchers: cash lent, or the reason as typed.
    for q in queries {
        if let Some((kind, reason)) = &q.voucher {
            let reason = reason.as_deref().map(str::trim).filter(|r| !r.is_empty());
            if kind == "payment" && ledger_is_buyer {
                found.insert(
                    (q.entry_id, q.line_no),
                    vec![DetailPart::CashLent {
                        reason: reason.unwrap_or_default().to_owned(),
                    }],
                );
            } else if let Some(reason) = reason {
                found.insert(
                    (q.entry_id, q.line_no),
                    vec![DetailPart::Text {
                        text: reason.to_owned(),
                    }],
                );
            }
        }
    }

    // A بکری: pair the lines with sale lines and lots.
    let mut base_ids: Vec<i64> = queries.iter().filter_map(|q| q.bikri_base_entry).collect();
    base_ids.sort_unstable();
    base_ids.dedup();
    if base_ids.is_empty() {
        return Ok(found);
    }
    let base_lines = load_base_lines(conn, &base_ids)?;
    let sales = load_sales(conn, &base_ids)?;
    for q in queries {
        let Some(base_id) = q.bikri_base_entry else {
            continue;
        };
        let (Some(lines), Some(sales)) = (base_lines.get(&base_id), sales.get(&base_id)) else {
            continue;
        };
        if let Some(part) = detail_of_line(lines, sales, q.line_no)? {
            found.insert((q.entry_id, q.line_no), vec![part]);
        }
    }
    Ok(found)
}

fn load_base_lines(
    conn: &Connection,
    base_ids: &[i64],
) -> Result<HashMap<i64, Vec<BaseLine>>, ReportsError> {
    let mut by_entry: HashMap<i64, Vec<BaseLine>> = HashMap::new();
    for chunk in base_ids.chunks(IN_CHUNK) {
        let sql = format!(
            "SELECT entry_id, line_no, account_id, pukhta_person_key, dr_paisa, cr_paisa
             FROM journal_lines WHERE entry_id IN ({}) ORDER BY entry_id, line_no",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                BaseLine {
                    line_no: r.get(1)?,
                    account_id: r.get(2)?,
                    person_key: r.get(3)?,
                    dr: r.get(4)?,
                    cr: r.get(5)?,
                },
            ))
        })?;
        for row in rows {
            let (entry_id, line) = row?;
            by_entry.entry(entry_id).or_default().push(line);
        }
    }
    Ok(by_entry)
}

fn load_sales(
    conn: &Connection,
    base_ids: &[i64],
) -> Result<HashMap<i64, Vec<SaleRow>>, ReportsError> {
    let mut by_entry: HashMap<i64, Vec<SaleRow>> = HashMap::new();
    for chunk in base_ids.chunks(IN_CHUNK) {
        // Rows replaced by an edit still belong to the entry they were posted in, so the old
        // entry and its reversal keep their items (the order is the one of the entry builder).
        let sql = format!(
            "SELECT s.journal_entry_id, s.lot_id, s.buyer_account_id, s.pukhta_person_key, s.qty,
                    s.buyer_rate_paisa, s.buyer_value_paisa, s.buyer_comm_paisa, s.laga_paisa,
                    s.buyer_total_paisa, s.seller_amount_paisa, l.seller_account_id,
                    l.pukhta_person_key, i.name_ur, i.name_en, p.name_ur, p.name_en, b.number
             FROM bikri_sales s
             JOIN bikri_lots l ON l.id = s.lot_id
             JOIN items i ON i.id = l.item_id
             JOIN packings p ON p.id = l.packing_id
             JOIN bikri_entries b ON b.id = s.bikri_id
             WHERE s.journal_entry_id IN ({})
             ORDER BY s.journal_entry_id, s.lot_id, s.line_no, s.id",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                SaleRow {
                    lot_id: r.get(1)?,
                    buyer_account_id: r.get(2)?,
                    person_key: r.get(3)?,
                    qty: r.get(4)?,
                    buyer_rate: r.get(5)?,
                    buyer_value: r.get(6)?,
                    buyer_comm: r.get(7)?,
                    laga: r.get(8)?,
                    buyer_total: r.get(9)?,
                    seller_amount: r.get(10)?,
                    lot_seller_account_id: r.get(11)?,
                    lot_person_key: r.get(12)?,
                    item: Names {
                        ur: r.get(13)?,
                        en: r.get(14)?,
                    },
                    packing: Names {
                        ur: r.get(15)?,
                        en: r.get(16)?,
                    },
                    bikri_number: r.get(17)?,
                },
            ))
        })?;
        for row in rows {
            let (entry_id, sale) = row?;
            by_entry.entry(entry_id).or_default().push(sale);
        }
    }
    Ok(by_entry)
}

/// The part for one line of the base entry, or `None` if the pairing does not hold.
fn detail_of_line(
    lines: &[BaseLine],
    sales: &[SaleRow],
    line_no: i64,
) -> Result<Option<DetailPart>, ReportsError> {
    let Some(line) = lines.iter().find(|l| l.line_no == line_no) else {
        return Ok(None);
    };
    let same_party = |other: &BaseLine| {
        other.account_id == line.account_id && other.person_key == line.person_key
    };
    if line.dr > 0 {
        // The n-th debit of this buyer is his n-th sale line.
        let n = lines
            .iter()
            .filter(|l| l.line_no < line.line_no && l.dr > 0 && same_party(l))
            .count();
        let candidates: Vec<&SaleRow> = sales
            .iter()
            .filter(|s| s.buyer_account_id == line.account_id && s.person_key == line.person_key)
            .collect();
        let Some(sale) = candidates.get(n) else {
            return Ok(None);
        };
        if sale.buyer_total != line.dr {
            return Ok(None);
        }
        return Ok(Some(DetailPart::Sale {
            bikri_number: sale.bikri_number,
            item: sale.item.clone(),
            packing: sale.packing.clone(),
            qty: sale.qty,
            rate_paisa: paisa(sale.buyer_rate)?,
            value_paisa: paisa(sale.buyer_value)?,
            commission_paisa: paisa(sale.buyer_comm)?,
            laga_paisa: paisa(sale.laga)?,
        }));
    }
    // A credit: the n-th credit of this seller is the credit of his n-th lot, if every credit
    // of the seller has a lot.
    let credits: Vec<&BaseLine> = lines.iter().filter(|l| l.cr > 0 && same_party(l)).collect();
    let n = credits.iter().filter(|l| l.line_no < line.line_no).count();
    let mut lot_ids: Vec<i64> = Vec::new();
    for sale in sales {
        if sale.lot_seller_account_id == line.account_id
            && sale.lot_person_key == line.person_key
            && !lot_ids.contains(&sale.lot_id)
        {
            lot_ids.push(sale.lot_id);
        }
    }
    if lot_ids.len() != credits.len() {
        return Ok(None);
    }
    let Some(lot_id) = lot_ids.get(n).copied() else {
        return Ok(None);
    };
    let of_lot: Vec<&SaleRow> = sales.iter().filter(|s| s.lot_id == lot_id).collect();
    let Some(first) = of_lot.first() else {
        return Ok(None);
    };
    let mut qty: i64 = 0;
    let mut gross = Paisa::ZERO;
    for sale in &of_lot {
        qty = qty.saturating_add(sale.qty);
        gross = gross.checked_add(paisa(sale.seller_amount)?)?;
    }
    let credit = paisa(line.cr)?;
    let deductions = gross.checked_sub(credit)?;
    if deductions.is_negative() {
        return Ok(None);
    }
    Ok(Some(DetailPart::Lot {
        bikri_number: first.bikri_number,
        item: first.item.clone(),
        packing: first.packing.clone(),
        qty,
        gross_paisa: gross,
        deductions_paisa: deductions,
    }))
}
