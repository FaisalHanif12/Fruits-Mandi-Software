//! R26 پیشگی کھاتہ (advances) (PRD 19.2, FR-15.8, FR-15.12, FR-19.10, E2E-7, E2E-31).
//!
//! One row for each seller who was given an advance by the "as of" day: what he was given, what
//! he returned in cash (a وصولی from him), what his sales adjusted, what is open and the day of
//! his last advance. The figures are the sums of the advance lines and of the active rows of
//! `advance_allocations` (`mandi_db::vouchers::advance_totals`, the same numbers as the voucher
//! screen); the open amount is `given − returned − adjusted`, worked out here with checked maths
//! and never below 0. A seller whose advance is fully adjusted still has his row, with 0 open.
//!
//! Example (E2E-7): رحیم given 15,000, returned 0, adjusted 15,000, open 0. (E2E-31) given
//! 15,000, returned 10,000, adjusted 5,000, open 0.

use mandi_core::money::Paisa;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::advance_totals;

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{add, col, count_of, date_cell, load_parties, paisa, party_of_line};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
};

/// Builds R26.
///
/// # Errors
/// A database error; `ReportsError::Money`; `ReportsError::Data` if more was returned and
/// adjusted than was given (a bug signal, never shown as a wrong number).
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let today = typed
        .today
        .ok_or_else(|| ReportsError::Data("the report has no today".to_owned()))?
        .to_iso();
    let as_of = typed.as_of.clone().unwrap_or(today);
    let mut totals = advance_totals(conn, &as_of, typed.seller_id)?;
    let ids_found: Vec<i64> = totals.iter().map(|t| t.account_id).collect();
    let parties = load_parties(conn, &ids_found)?;
    // By account code, like the voucher screen's report.
    totals.sort_by(|a, b| {
        let code = |id: i64| parties.get(&id).map(|p| p.code.clone()).unwrap_or_default();
        code(a.account_id).cmp(&code(b.account_id))
    });

    let mut sums = [Paisa::ZERO; 4];
    let mut all_rows = Vec::with_capacity(totals.len());
    for t in totals {
        let given = paisa(t.given_paisa)?;
        let returned = paisa(t.returned_paisa)?;
        let adjusted = paisa(t.adjusted_paisa)?;
        let left = given.checked_sub(returned)?.checked_sub(adjusted)?;
        if left.is_negative() {
            return Err(ReportsError::Data(format!(
                "account {}: more returned and adjusted than given as advance",
                t.account_id
            )));
        }
        for (slot, amount) in sums.iter_mut().zip([given, returned, adjusted, left]) {
            add(slot, amount)?;
        }
        all_rows.push(Row {
            cells: vec![
                Cell::Party {
                    party: party_of_line(&parties, t.account_id, None)?,
                },
                Cell::Money { paisa: given },
                Cell::Money { paisa: returned },
                Cell::Money { paisa: adjusted },
                Cell::Money { paisa: left },
                date_cell(t.last_date),
            ],
            source: Some(RowSource {
                kind: SourceKind::Account,
                id: t.account_id,
            }),
            kind: RowKind::Normal,
        });
    }

    let [given, returned, adjusted, left] = sums;
    Ok(ReportPage {
        id: ReportId::R26,
        title_id: ids::title_id("R26"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("seller", ids::COL_SELLER, ColumnKind::Party),
            col("given", ids::COL_GIVEN, ColumnKind::Money),
            col("returned", ids::COL_RETURNED, ColumnKind::Money),
            col("adjusted", ids::COL_ADJUSTED, ColumnKind::Money),
            col("open", ids::COL_OPEN, ColumnKind::Money),
            col("last_date", ids::COL_LAST_DATE, ColumnKind::Date),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Money { paisa: given },
            Cell::Money { paisa: returned },
            Cell::Money { paisa: adjusted },
            Cell::Money { paisa: left },
            Cell::Empty,
        ],
        summary: Vec::new(),
    })
}
