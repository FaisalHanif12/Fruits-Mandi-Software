//! View B, by seller, بیوپاری وار (PRD 18.2, FR-18.6, FR-18.7).
//!
//! One row for each seller with sales in the filter: the number of بکری, نگ, خام بکری, the seller
//! commission, the buyer commission on his goods, the total commission, and صافی بکری.
//!
//! - **بکری count.** The number of distinct بکری with something left in the filter. A cancelled
//!   بکری counts 0; an edited one counts 1.
//! - **صافی بکری (money agent review).** The seller amounts less the seller commission, both from
//!   the facts in the filter, less the charges of the lots that pass the filter (freight only
//!   when the firm paid it, labour, munshiana, phone, store rent and the cash paid), taken from
//!   the saved lots whose بکری is dated in the period. It adds up across periods, which the
//!   lot's stored net does not. A seller whose lots in the period have charges but no sale line
//!   in it has a row with no sales and a negative net, so the charges are never lost. The cell is empty when a buyer or city filter is set (a lot's
//!   charges cannot be split by buyer) or when only the buyer side is counted.
//!
//! Example (PRD 18.4): رحیم 1 بکری, 50 نگ, 29,400, 1,764, 1,764, total 3,928, net
//! 29,400 − 1,764 − (3,000 + 250 + 50 + 5,000 + 0 + 20) = 19,316.

use std::collections::{HashMap, HashSet};

use mandi_core::money::Paisa;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::facts::{self, Sums, compare_range, run_range};
use super::filters::Typed;
use super::ids;
use super::item::totals_by_id;
use super::model::{AnalyticsKey, AnalyticsSide};
use super::table::{Part, Table, col, compare_cells, compare_columns, count_of, part_cell, row};
use crate::error::ReportsError;
use crate::reports::PageSpec;
use crate::reports::common::{Conds, add, load_parties, paisa};
use crate::reports::{Cell, ColumnKind};

/// The charges of the saved lots in the filter, for each seller: freight paid by the firm,
/// labour, munshiana, phone, store rent and cash paid, all summed from stored columns.
fn lot_charges(conn: &Connection, typed: &Typed) -> Result<HashMap<i64, Paisa>, ReportsError> {
    let mut c = Conds::new();
    c.always("b.status = 'saved'")
        .text("b.entry_date >= ?", typed.from.as_deref())
        .text("b.entry_date <= ?", typed.to.as_deref())
        .id("l.seller_account_id = ?", typed.seller_id)
        .id("l.item_id = ?", typed.item_id)
        .id("b.adda_id = ?", typed.adda_id);
    let sql = format!(
        "SELECT l.seller_account_id,
                COALESCE(SUM(CASE WHEN l.freight_paid_by = 'firm' THEN l.freight_paisa END), 0),
                COALESCE(SUM(l.labour_paisa), 0), COALESCE(SUM(l.munshiana_paisa), 0),
                COALESCE(SUM(l.phone_paisa), 0), COALESCE(SUM(l.store_rent_paisa), 0),
                COALESCE(SUM(l.cash_paid_paisa), 0)
         FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id
         WHERE {} GROUP BY l.seller_account_id",
        c.sql()
    );
    let mut statement = conn.prepare(&sql)?;
    let raw = statement
        .query_map(params_from_iter(c.args().iter()), |r| {
            let mut sums = [0_i64; 6];
            for (slot, index) in sums.iter_mut().zip(1_usize..) {
                *slot = r.get(index)?;
            }
            Ok((r.get::<_, i64>(0)?, sums))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut found = HashMap::new();
    for (seller, sums) in raw {
        let mut total = Paisa::ZERO;
        for value in sums {
            add(&mut total, paisa(value)?)?;
        }
        found.insert(seller, total);
    }
    Ok(found)
}

struct Entry {
    id: i64,
    sums: Sums,
    bikri: i64,
    total: Paisa,
    compared: Paisa,
}

/// Builds view B.
///
/// # Errors
/// A database error; `ReportsError::Data` for a fact net below 0; `ReportsError::Money`.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    let none = AnalyticsKey::default();
    // One query by seller and بکری gives the sums and the count of بکری.
    let current = facts::groups(
        conn,
        &facts::conds(typed, run_range(typed), &none),
        &["f.seller_account_id", "f.bikri_id"],
    )?;
    // The distinct بکری of the whole filter, for the total row.
    let all_bikri = facts::groups(
        conn,
        &facts::conds(typed, run_range(typed), &none),
        &["f.bikri_id"],
    )?;
    let compared = if typed.compare.is_some() {
        totals_by_id(
            &facts::groups(
                conn,
                &facts::conds(typed, compare_range(typed), &none),
                &["f.seller_account_id"],
            )?,
            typed,
        )?
    } else {
        HashMap::new()
    };

    let mut by_seller: HashMap<i64, (Sums, HashSet<i64>)> = HashMap::new();
    for group in &current {
        let (Some(seller), Some(bikri)) = (group.int(0), group.int(1)) else {
            continue;
        };
        let slot = by_seller.entry(seller).or_default();
        slot.0.add(&group.sums)?;
        slot.1.insert(bikri);
    }

    // The net is given when the lots' charges can be matched to the rows.
    let net_given =
        typed.buyer_id.is_none() && typed.city_id.is_none() && typed.side != AnalyticsSide::Buyer;
    let charges = if net_given {
        lot_charges(conn, typed)?
    } else {
        HashMap::new()
    };

    let mut entries: Vec<Entry> = Vec::new();
    for (id, (sums, bikri)) in &by_seller {
        entries.push(Entry {
            id: *id,
            sums: *sums,
            bikri: count_of(bikri.len()),
            total: sums.total_comm(typed.side)?,
            compared: compared.get(id).copied().unwrap_or(Paisa::ZERO),
        });
    }
    // A seller whose lots in the period carry charges but have no sale line in it (a lot that
    // waits in باقی مال) has a row with no sales: his net is the charges taken off. Without it the
    // charges would fall out of the total and the nets would not add up across periods (money
    // agent review of the code, finding 1).
    let mut in_rows: HashSet<i64> = by_seller.keys().copied().collect();
    for (id, charge) in &charges {
        if *charge > Paisa::ZERO && in_rows.insert(*id) {
            entries.push(Entry {
                id: *id,
                sums: Sums::default(),
                bikri: 0,
                total: Paisa::ZERO,
                compared: compared.get(id).copied().unwrap_or(Paisa::ZERO),
            });
        }
    }
    for (id, amount) in &compared {
        if in_rows.insert(*id) {
            entries.push(Entry {
                id: *id,
                sums: Sums::default(),
                bikri: 0,
                total: Paisa::ZERO,
                compared: *amount,
            });
        }
    }

    let ids_all: Vec<i64> = entries.iter().map(|e| e.id).collect();
    let parties = load_parties(conn, &ids_all)?;
    let code_of = |id: i64| parties.get(&id).map(|p| p.code.clone()).unwrap_or_default();
    entries.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| code_of(a.id).cmp(&code_of(b.id)))
            .then_with(|| a.id.cmp(&b.id))
    });

    let net_of = |entry: &Entry| -> Result<Cell, ReportsError> {
        if !net_given {
            return Ok(Cell::Empty);
        }
        let charge = charges.get(&entry.id).copied().unwrap_or(Paisa::ZERO);
        let net = entry
            .sums
            .gross
            .checked_sub(entry.sums.seller_comm)?
            .checked_sub(charge)?;
        Ok(Cell::Money { paisa: net })
    };

    let mut sum = Sums::default();
    let mut grand_compared = Paisa::ZERO;
    let mut grand_total = Paisa::ZERO;
    let mut grand_net = Paisa::ZERO;
    for entry in &entries {
        sum.add(&entry.sums)?;
        add(&mut grand_total, entry.total)?;
        add(&mut grand_compared, entry.compared)?;
        if let Cell::Money { paisa } = net_of(entry)? {
            add(&mut grand_net, paisa)?;
        }
    }

    let with_compare = typed.compare.is_some();
    let mut columns = vec![
        col("seller", ids::COL_SELLER, ColumnKind::Party),
        col("bikri_count", ids::COL_BIKRI_COUNT, ColumnKind::Number),
        col("qty", ids::COL_QTY, ColumnKind::Number),
        col("gross", ids::COL_GROSS, ColumnKind::Money),
        col("seller_comm", ids::COL_SELLER_COMM, ColumnKind::Money),
        col("buyer_comm", ids::COL_BUYER_COMM, ColumnKind::Money),
        col("total_comm", ids::COL_TOTAL_COMM, ColumnKind::Money),
        col("net", ids::COL_NET, ColumnKind::Money),
    ];
    if with_compare {
        columns.extend(compare_columns());
    }

    let cells_of = |sums: &Sums,
                    bikri: i64,
                    total: Paisa,
                    compared: Paisa,
                    seller: Cell,
                    net: Cell|
     -> Result<Vec<Cell>, ReportsError> {
        // PRD 18.2 lists the buyer commission and the total, and the total adds لاگا too
        // (3,928 = 1,764 + 1,764 + 400).
        let mut cells = vec![
            seller,
            Cell::Number { value: bikri },
            Cell::Number { value: sums.qty },
            Cell::Money { paisa: sums.gross },
            part_cell(typed.side, Part::Seller, sums.seller_comm),
            part_cell(typed.side, Part::Buyer, sums.buyer_comm),
            Cell::Money { paisa: total },
            net,
        ];
        if with_compare {
            cells.extend(compare_cells(total, compared)?);
        }
        Ok(cells)
    };

    let mut all_rows = Vec::with_capacity(entries.len());
    for entry in &entries {
        let seller = parties
            .get(&entry.id)
            .map(|p| Cell::Party { party: p.clone() })
            .unwrap_or(Cell::Empty);
        all_rows.push(row(
            cells_of(
                &entry.sums,
                entry.bikri,
                entry.total,
                entry.compared,
                seller,
                net_of(entry)?,
            )?,
            AnalyticsKey {
                seller_id: Some(entry.id),
                ..AnalyticsKey::default()
            },
        ));
    }
    let net_total = if net_given {
        Cell::Money { paisa: grand_net }
    } else {
        Cell::Empty
    };
    let totals = cells_of(
        &sum,
        count_of(all_bikri.len()),
        grand_total,
        grand_compared,
        Cell::Empty,
        net_total,
    )?;

    Ok(Table {
        columns,
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals,
    })
}
