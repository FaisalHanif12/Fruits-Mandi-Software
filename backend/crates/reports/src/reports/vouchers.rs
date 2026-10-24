//! The voucher lists: R15 ووچر (every kind of voucher), R23 وصولی رجسٹر (receipts) and R25 ادائیگی
//! رجسٹر (payments) (PRD 19.2, FR-19.10, FR-19.11, FR-15.1).
//!
//! One row is one voucher, oldest day first, then by kind and number. A cancelled voucher stays in
//! the list with its status word; the totals row adds up only the saved ones. A list of one kind
//! (R23, R25, or R15 with a kind picked) has that total. R15 with no kind picked mixes kinds
//! (a جنرل voucher's amount is its debit total, not money that moved), so its totals row has no
//! amount and a line above the table gives the saved money of each kind. There is no mode, bank
//! or cheque column: all money is روکڑ (v1.6), and an old row just shows its amount, account,
//! date and number. The page and the totals are two queries over the same filter (Code Standards
//! B31); the sums are `SUM()` of an `INTEGER` column.
//!
//! Example (PRD 4, E2E-6): R23 for 10-10-26 lists R-1, اکرم, 10,000, total 10,000. R25 lists P-1,
//! رحیم, 4,316.

use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{
    Conds, col, load_parties, names_cell, paisa, party_of_line, text_cell,
};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind, SummaryLine,
};

/// Which voucher list is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Layout {
    /// R15: every kind, or the kind the filter names.
    All,
    /// R23: receipts, with the collector.
    Receipts,
    /// R25: payments.
    Payments,
}

impl Layout {
    fn id(self) -> ReportId {
        match self {
            Layout::All => ReportId::R15,
            Layout::Receipts => ReportId::R23,
            Layout::Payments => ReportId::R25,
        }
    }
}

/// The conditions of a list. The kind is a constant of this crate or a checked enum, and every
/// value from the screen is bound as a parameter (Code Standards B14).
fn conditions(layout: Layout, typed: &Typed) -> Conds {
    let mut c = Conds::new();
    match layout {
        Layout::All => {
            c.text("v.type = ?", typed.voucher_type.map(|t| t.as_str()));
        }
        Layout::Receipts => {
            c.always("v.type = 'receipt'");
        }
        Layout::Payments => {
            c.always("v.type = 'payment'");
        }
    }
    c.text("v.entry_date >= ?", typed.from.as_deref())
        .text("v.entry_date <= ?", typed.to.as_deref())
        .id("v.account_id = ?", typed.party_id)
        .text("v.pukhta_person_key = ?", typed.person_key.as_deref());
    c
}

/// For each kind of voucher in the filter: how many vouchers there are and the money of the saved
/// ones, over the whole filter (Code Standards B31). Money of different kinds is never added into
/// one number when the list mixes kinds (see `run`): a جنرل voucher's amount is the debit total of
/// its lines (PRD 16.4), and an advance or a cash lent is not a receipt.
fn totals(conn: &Connection, c: &Conds) -> Result<Vec<(String, i64, i64)>, ReportsError> {
    // The only text joined into the statement is the filter's own condition list, which is built
    // from constants of this crate (Code Standards B14).
    let sql = format!(
        "SELECT v.type, COUNT(*),
                COALESCE(SUM(CASE WHEN v.status = 'saved' THEN v.amount_paisa END), 0)
         FROM vouchers v WHERE {} GROUP BY v.type ORDER BY v.type",
        c.sql()
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(c.args().iter()), |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// One stored voucher as the list reads it.
struct Stored {
    id: i64,
    kind: String,
    number: i64,
    date: String,
    account_id: Option<i64>,
    person_key: Option<String>,
    amount: i64,
    partner: Option<(String, String)>,
    expense_head: Option<(String, String)>,
    collector: Option<(String, String)>,
    reason: Option<String>,
    narration: Option<String>,
    cancelled: bool,
}

fn page_rows(conn: &Connection, c: &Conds, spec: PageSpec) -> Result<Vec<Stored>, ReportsError> {
    let sql = format!(
        "SELECT v.id, v.type, v.number, v.entry_date, v.account_id, v.pukhta_person_key,
                v.amount_paisa, p.name_ur, p.name_en, eh.name_ur, eh.name_en, st.name_ur, st.name_en,
                v.reason, v.narration, v.status = 'cancelled'
         FROM vouchers v
         LEFT JOIN partners p ON p.id = v.partner_id
         LEFT JOIN expense_heads eh ON eh.id = v.expense_head_id
         LEFT JOIN staff st ON st.id = v.collector_staff_id
         WHERE {}
         ORDER BY v.entry_date, v.type, v.number
         LIMIT ? OFFSET ?",
        c.sql()
    );
    let args: Vec<Value> = c.page_args(spec.sql_limit(), spec.offset);
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(args.iter()), |r| {
        let pair = |at: usize| -> mandi_db::rusqlite::Result<Option<(String, String)>> {
            let ur: Option<String> = r.get(at)?;
            let en: Option<String> = r.get(at + 1)?;
            Ok(ur.zip(en))
        };
        Ok(Stored {
            id: r.get(0)?,
            kind: r.get(1)?,
            number: r.get(2)?,
            date: r.get(3)?,
            account_id: r.get(4)?,
            person_key: r.get(5)?,
            amount: r.get(6)?,
            partner: pair(7)?,
            expense_head: pair(9)?,
            collector: pair(11)?,
            reason: r.get(13)?,
            narration: r.get(14)?,
            cancelled: r.get(15)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Builds R15, R23 or R25.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total does
/// not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    layout: Layout,
) -> Result<ReportPage, ReportsError> {
    let c = conditions(layout, typed);
    let per_kind = totals(conn, &c)?;
    let mut count = 0_i64;
    let mut saved_paisa = 0_i64;
    for (_, kind_count, kind_saved) in &per_kind {
        count = count
            .checked_add(*kind_count)
            .ok_or_else(|| ReportsError::Data("the voucher count does not fit".to_owned()))?;
        saved_paisa = saved_paisa
            .checked_add(*kind_saved)
            .ok_or_else(|| ReportsError::Data("the voucher total does not fit".to_owned()))?;
    }
    // A list of one kind (R23, R25, or R15 with a kind picked) has one total. R15 with no kind
    // picked mixes kinds, so it has no total amount; each kind with saved money gets its own line
    // above the table instead (money agent, round 8).
    let mixed = layout == Layout::All && typed.voucher_type.is_none();
    let stored = page_rows(conn, &c, spec)?;
    let account_ids: Vec<i64> = stored.iter().filter_map(|v| v.account_id).collect();
    let parties = load_parties(conn, &account_ids)?;

    let mut rows = Vec::with_capacity(stored.len());
    for v in stored {
        // The party of the voucher: an account (and the person inside a پختہ کھاتہ account), else
        // the partner it is for, else its expense head.
        let account = match v.account_id {
            Some(id) => Cell::Party {
                party: party_of_line(&parties, id, v.person_key.clone())?,
            },
            None => match (&v.partner, &v.expense_head) {
                (Some((ur, en)), _) | (None, Some((ur, en))) => {
                    names_cell(Some(ur.clone()), Some(en.clone()))
                }
                (None, None) => Cell::Empty,
            },
        };
        // The note of the voucher, else its reason or "paid to" (as the cash book).
        let note = v
            .narration
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
            .or(v.reason.clone());
        let status = if v.cancelled {
            Cell::Label {
                id: ids::STATUS_CANCELLED.to_owned(),
            }
        } else {
            Cell::Empty
        };
        let mut cells = vec![
            Cell::Date {
                iso: v.date.clone(),
            },
            Cell::Doc {
                type_id: ids::doc_id(&v.kind),
                number: Some(v.number),
            },
            account,
            Cell::Money {
                paisa: paisa(v.amount)?,
            },
        ];
        if layout == Layout::Receipts {
            cells.push(match &v.collector {
                Some((ur, en)) => names_cell(Some(ur.clone()), Some(en.clone())),
                None => Cell::Empty,
            });
        }
        cells.push(text_cell(note));
        cells.push(status);
        rows.push(Row {
            cells,
            source: Some(RowSource {
                kind: SourceKind::Voucher,
                id: v.id,
            }),
            kind: RowKind::Normal,
        });
    }

    let mut columns = vec![
        col("date", ids::COL_DATE, ColumnKind::Date),
        col("doc", ids::COL_DOC, ColumnKind::Doc),
        col("account", ids::COL_ACCOUNT, ColumnKind::Party),
        col("amount", ids::COL_AMOUNT, ColumnKind::Money),
    ];
    let mut totals_row = vec![
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        if mixed {
            Cell::Empty
        } else {
            Cell::Money {
                paisa: paisa(saved_paisa)?,
            }
        },
    ];
    if layout == Layout::Receipts {
        columns.push(col("collector", ids::COL_COLLECTOR, ColumnKind::Names));
        totals_row.push(Cell::Empty);
    }
    columns.push(col("narration", ids::COL_NARRATION, ColumnKind::Text));
    columns.push(col("status", ids::COL_STATUS, ColumnKind::Label));
    totals_row.push(Cell::Empty);
    totals_row.push(Cell::Empty);

    let mut summary = Vec::new();
    if mixed {
        for (kind, _, kind_saved) in &per_kind {
            if *kind_saved > 0 {
                summary.push(SummaryLine {
                    label_id: ids::doc_id(kind),
                    cell: Cell::Money {
                        paisa: paisa(*kind_saved)?,
                    },
                });
            }
        }
    }

    let id = layout.id();
    Ok(ReportPage {
        id,
        title_id: ids::title_id(id.code()),
        filters: typed.resolved.clone(),
        columns,
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: totals_row,
        summary,
    })
}
