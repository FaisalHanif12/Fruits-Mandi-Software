//! R8 اڈا رپورٹ (the truck stand report): for each اڈا, the trucks that came, their lots, the
//! نگ received and the خام بکری of those lots (PRD 19.2, FR-19.10).
//!
//! A truck is one saved بکری (one truck of goods sold; PRD 12.2). خام بکری is the stored value on
//! each lot, all its postings together, so a lot sold over two days counts once. A cancelled بکری
//! is not counted. A بکری with no اڈا is one row of its own, last.
//!
//! Example (the worked day): سوات اڈا 1 truck, 1 lot, 50 نگ, خام 29,400; لاہور اڈا 1 truck,
//! 1 lot, 10 نگ, خام 12,000; the total row 2, 2, 60, 41,400.

use mandi_core::money::Paisa;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{Conds, add, col, count_of, names_cell, paisa};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{Cell, ColumnKind, ReportId, ReportPage, Row, RowKind};

/// The query of R8: one row for each اڈا with trucks in the filter.
fn query(typed: &Typed) -> (String, Vec<Value>) {
    let mut c = Conds::new();
    c.always("b.status = 'saved'")
        .text("b.entry_date >= ?", typed.from.as_deref())
        .text("b.entry_date <= ?", typed.to.as_deref())
        .id("b.adda_id = ?", typed.adda_id);
    let sql = format!(
        "SELECT a.name_ur, a.name_en, COUNT(DISTINCT b.id), COUNT(l.id),
            COALESCE(SUM(l.qty_received), 0), COALESCE(SUM(l.gross_paisa), 0)
         FROM bikri_entries b
         JOIN bikri_lots l ON l.bikri_id = b.id
         LEFT JOIN addas a ON a.id = b.adda_id
         WHERE {}
         GROUP BY b.adda_id ORDER BY (a.code IS NULL), a.code",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// Builds R8.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total
/// does not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let (sql, args) = query(typed);
    let mut statement = conn.prepare(&sql)?;
    let raw = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((
                r.get::<_, Option<String>>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let (mut trucks, mut lots, mut qty) = (0_i64, 0_i64, 0_i64);
    let mut gross = Paisa::ZERO;
    let mut all_rows = Vec::with_capacity(raw.len());
    let overflow = || ReportsError::Data("a count total does not fit".to_owned());
    for (name_ur, name_en, row_trucks, row_lots, row_qty, row_gross) in raw {
        trucks = trucks.checked_add(row_trucks).ok_or_else(overflow)?;
        lots = lots.checked_add(row_lots).ok_or_else(overflow)?;
        qty = qty.checked_add(row_qty).ok_or_else(overflow)?;
        add(&mut gross, paisa(row_gross)?)?;
        all_rows.push(Row {
            cells: vec![
                names_cell(name_ur, name_en),
                Cell::Number { value: row_trucks },
                Cell::Number { value: row_lots },
                Cell::Number { value: row_qty },
                Cell::Money {
                    paisa: paisa(row_gross)?,
                },
            ],
            source: None,
            kind: RowKind::Normal,
        });
    }

    Ok(ReportPage {
        group_above: None,
        id: ReportId::R8,
        title_id: ids::title_id("R8"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("adda", ids::COL_ADDA, ColumnKind::Names),
            col("trucks", ids::COL_TRUCKS, ColumnKind::Number),
            col("lots", ids::COL_LOTS, ColumnKind::Number),
            col("qty", ids::COL_QTY, ColumnKind::Number),
            col("gross", ids::COL_GROSS, ColumnKind::Money),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Number { value: trucks },
            Cell::Number { value: lots },
            Cell::Number { value: qty },
            Cell::Money { paisa: gross },
        ],
        summary: Vec::new(),
    })
}
