//! What the views share: the table they build, column and cell helpers, the compare columns, and
//! the names of items.

use std::collections::HashMap;

use mandi_core::money::Paisa;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::calc::share_hundredths;
use super::model::{AnalyticsColumn, AnalyticsHeading, AnalyticsKey, AnalyticsRow, AnalyticsSide};
use crate::error::ReportsError;
use crate::reports::common::{IN_CHUNK, placeholders};
use crate::reports::{Cell, ColumnKind, Names, RowKind};

/// A built view: the columns, the rows of the page asked for, how many rows the whole filter
/// has, and the totals row over the whole filter.
pub(crate) struct Table {
    pub(crate) columns: Vec<AnalyticsColumn>,
    pub(crate) rows: Vec<AnalyticsRow>,
    pub(crate) total_count: i64,
    pub(crate) totals: Vec<Cell>,
}

/// A fixed column.
pub(crate) fn col(key: &str, title_id: &str, kind: ColumnKind) -> AnalyticsColumn {
    AnalyticsColumn {
        key: key.to_owned(),
        title_id: title_id.to_owned(),
        kind,
        heading: None,
        drill: None,
    }
}

/// A column named by a heading, with what it adds to a row key.
pub(crate) fn dynamic_col(
    key: String,
    title_id: &str,
    heading: AnalyticsHeading,
    drill: AnalyticsKey,
) -> AnalyticsColumn {
    AnalyticsColumn {
        key,
        title_id: title_id.to_owned(),
        kind: ColumnKind::Money,
        heading: Some(heading),
        drill: Some(drill),
    }
}

/// A data row.
pub(crate) fn row(cells: Vec<Cell>, key: AnalyticsKey) -> AnalyticsRow {
    AnalyticsRow {
        cells,
        key: Some(key),
        kind: RowKind::Normal,
    }
}

/// The part of a commission a column belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Part {
    /// Seller commission.
    Seller,
    /// Buyer commission and لاگا.
    Buyer,
}

/// An amount of one side, or an empty cell when the side filter leaves that side out.
pub(crate) fn part_cell(side: AnalyticsSide, part: Part, amount: Paisa) -> Cell {
    let counted = matches!(
        (side, part),
        (AnalyticsSide::Both, _)
            | (AnalyticsSide::Seller, Part::Seller)
            | (AnalyticsSide::Buyer, Part::Buyer)
    );
    if counted {
        Cell::Money { paisa: amount }
    } else {
        Cell::Empty
    }
}

/// The two columns a comparison adds: the figure of the compared period and the change.
pub(crate) fn compare_columns() -> Vec<AnalyticsColumn> {
    vec![
        col("compare", super::ids::COL_COMPARE, ColumnKind::Money),
        col("change", super::ids::COL_CHANGE, ColumnKind::Money),
    ]
}

/// The cells of a comparison: the compared figure and `current − compared`, which can be
/// negative (a change is not a balance).
///
/// # Errors
/// `ReportsError::Money` if the difference does not fit.
pub(crate) fn compare_cells(current: Paisa, compared: Paisa) -> Result<Vec<Cell>, ReportsError> {
    Ok(vec![
        Cell::Money { paisa: compared },
        Cell::Money {
            paisa: current.checked_sub(compared)?,
        },
    ])
}

/// A share as a percent cell, or empty when the total is 0.
///
/// # Errors
/// As [`share_hundredths`].
pub(crate) fn percent_cell(part: Paisa, total: Paisa) -> Result<Cell, ReportsError> {
    Ok(share_hundredths(part, total)?
        .map_or(Cell::Empty, |hundredths| Cell::Percent { hundredths }))
}

/// A row count as the `i64` of the contract.
pub(crate) fn count_of(len: usize) -> i64 {
    i64::try_from(len).unwrap_or(i64::MAX)
}

/// An item with its code (to order ties) and names.
#[derive(Debug, Clone)]
pub(crate) struct ItemRow {
    pub(crate) code: String,
    pub(crate) names: Names,
}

/// The items by id. One query for each chunk of ids (Code Standards B22).
///
/// # Errors
/// A database error.
pub(crate) fn item_rows(
    conn: &Connection,
    ids: &[i64],
) -> Result<HashMap<i64, ItemRow>, ReportsError> {
    let mut unique = ids.to_vec();
    unique.sort_unstable();
    unique.dedup();
    let mut found = HashMap::new();
    for chunk in unique.chunks(IN_CHUNK) {
        let sql = format!(
            "SELECT id, code, name_ur, name_en FROM items WHERE id IN ({})",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                ItemRow {
                    code: r.get(1)?,
                    names: Names {
                        ur: r.get(2)?,
                        en: r.get(3)?,
                    },
                },
            ))
        })?;
        for item in rows {
            let (id, row) = item?;
            found.insert(id, row);
        }
    }
    Ok(found)
}
