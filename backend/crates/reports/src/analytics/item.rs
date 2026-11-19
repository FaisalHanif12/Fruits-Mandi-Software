//! View A, by item, قسم مال وار (PRD 18.2, FR-18.6, FR-18.7, C18).
//!
//! One row for each item with sales in the filter: نگ, خام بکری, the average rate, the seller
//! and buyer commission, لاگا, the total commission and its share of all commission. The
//! figures are the stored values of the sale lines, summed from `analytics_facts`.
//!
//! Example (PRD 18.4, the worked day): کیلا 50 نگ, 29,400, average 588, 1,764, 1,764, 400, total
//! 3,928, 72.10%; سیب 10 نگ, 12,000, average 1,200, total 1,520, 27.90%; the total row 60, 41,400,
//! 2,484, 2,484, 480, 5,448, 100.00%.

use std::collections::HashMap;

use mandi_core::money::{Paisa, Qty, avg_rate};
use mandi_db::rusqlite::Connection;

use super::facts::{self, Group, Sums, compare_range, run_range};
use super::filters::Typed;
use super::ids;
use super::model::AnalyticsKey;
use super::table::{
    ItemRow, Part, Table, col, compare_cells, compare_columns, count_of, item_rows, part_cell,
    percent_cell, row,
};
use crate::error::ReportsError;
use crate::reports::PageSpec;
use crate::reports::common::add;
use crate::reports::{Cell, ColumnKind};

/// The average rate of a row (PRD C18), or an empty cell when there is no quantity.
pub(super) fn average(gross: Paisa, qty: i64) -> Result<Cell, ReportsError> {
    if qty <= 0 || gross.is_negative() {
        return Ok(Cell::Empty);
    }
    Ok(Cell::Money {
        paisa: avg_rate(gross, Qty::new(qty)?)?,
    })
}

struct Entry {
    id: i64,
    sums: Sums,
    total: Paisa,
    compared: Paisa,
}

/// The total commission of each id from a query by one id column.
pub(super) fn totals_by_id(
    groups: &[Group],
    typed: &Typed,
) -> Result<HashMap<i64, Paisa>, ReportsError> {
    let mut found = HashMap::new();
    for group in groups {
        if let Some(id) = group.int(0) {
            found.insert(id, group.sums.total_comm(typed.side)?);
        }
    }
    Ok(found)
}

/// Builds view A.
///
/// # Errors
/// A database error; `ReportsError::Data` for a fact net below 0; `ReportsError::Money`.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    let none = AnalyticsKey::default();
    let current = facts::groups(
        conn,
        &facts::conds(typed, run_range(typed), &none),
        &["f.item_id"],
    )?;
    let compared = if typed.compare.is_some() {
        totals_by_id(
            &facts::groups(
                conn,
                &facts::conds(typed, compare_range(typed), &none),
                &["f.item_id"],
            )?,
            typed,
        )?
    } else {
        HashMap::new()
    };

    let mut ids: Vec<i64> = current.iter().filter_map(|g| g.int(0)).collect();
    ids.extend(compared.keys().copied());
    let items = item_rows(conn, &ids)?;

    // One entry for each item in either period.
    let mut entries: Vec<Entry> = Vec::new();
    for group in &current {
        let Some(id) = group.int(0) else { continue };
        entries.push(Entry {
            id,
            sums: group.sums,
            total: group.sums.total_comm(typed.side)?,
            compared: compared.get(&id).copied().unwrap_or(Paisa::ZERO),
        });
    }
    for (id, amount) in &compared {
        if !entries.iter().any(|e| e.id == *id) {
            entries.push(Entry {
                id: *id,
                sums: Sums::default(),
                total: Paisa::ZERO,
                compared: *amount,
            });
        }
    }
    let code_of = |id: i64| -> String {
        items
            .get(&id)
            .map(|i: &ItemRow| i.code.clone())
            .unwrap_or_default()
    };
    entries.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| code_of(a.id).cmp(&code_of(b.id)))
            .then_with(|| a.id.cmp(&b.id))
    });

    // The totals over the whole filter.
    let mut sum = Sums::default();
    let mut grand = Paisa::ZERO;
    let mut grand_compared = Paisa::ZERO;
    for entry in &entries {
        sum.add(&entry.sums)?;
        add(&mut grand, entry.total)?;
        add(&mut grand_compared, entry.compared)?;
    }

    let with_compare = typed.compare.is_some();
    let mut columns = vec![
        col("item", ids::COL_ITEM, ColumnKind::Names),
        col("qty", ids::COL_QTY, ColumnKind::Number),
        col("gross", ids::COL_GROSS, ColumnKind::Money),
        col("avg_rate", ids::COL_AVG_RATE, ColumnKind::Money),
        col("seller_comm", ids::COL_SELLER_COMM, ColumnKind::Money),
        col("buyer_comm", ids::COL_BUYER_COMM, ColumnKind::Money),
        col("laga", ids::COL_LAGA, ColumnKind::Money),
        col("total_comm", ids::COL_TOTAL_COMM, ColumnKind::Money),
        col("share", ids::COL_SHARE, ColumnKind::Percent),
    ];
    if with_compare {
        columns.extend(compare_columns());
    }

    let cells_of = |sums: &Sums,
                    total: Paisa,
                    compared: Paisa,
                    item: Cell|
     -> Result<Vec<Cell>, ReportsError> {
        let mut cells = vec![
            item,
            Cell::Number { value: sums.qty },
            Cell::Money { paisa: sums.gross },
            average(sums.gross, sums.qty)?,
            part_cell(typed.side, Part::Seller, sums.seller_comm),
            part_cell(typed.side, Part::Buyer, sums.buyer_comm),
            part_cell(typed.side, Part::Buyer, sums.laga),
            Cell::Money { paisa: total },
            percent_cell(total, grand)?,
        ];
        if with_compare {
            cells.extend(compare_cells(total, compared)?);
        }
        Ok(cells)
    };

    let mut all_rows = Vec::with_capacity(entries.len());
    for entry in &entries {
        let names = items
            .get(&entry.id)
            .map(|i| Cell::Names {
                names: i.names.clone(),
            })
            .unwrap_or(Cell::Empty);
        all_rows.push(row(
            cells_of(&entry.sums, entry.total, entry.compared, names)?,
            AnalyticsKey {
                item_id: Some(entry.id),
                ..AnalyticsKey::default()
            },
        ));
    }
    // The totals row: no average rate, and the share is 100.00 when there is commission.
    let mut totals = cells_of(&sum, grand, grand_compared, Cell::Empty)?;
    if let Some(cell) = totals.get_mut(3) {
        *cell = Cell::Empty;
    }

    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals,
    })
}
