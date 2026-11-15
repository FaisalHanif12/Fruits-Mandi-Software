//! The partner part of view M, خرچہ: the kharch each partner spent himself (PRD 18.2 M, FR-9.5,
//! FR-17.11, FR-16.2, plan 77).
//!
//! One row for each partner (every active partner, and any other who has a figure in the period) with
//! his own kharch, the money he took for himself (ذاتی خرچہ): the debits of his drawings account, less
//! the takes that were cancelled or edited (`taken_between`, the sums of the partner strip and R30; a
//! cancelled take leaves the day it was made on). There is no share of the shop kharch and no returns
//! column. The totals row adds the rows (checked). A cell opens his kharch entries, each with its
//! date, amount and reason, and they add up to the cell. This part is behind the partner code, which
//! the command layer checks first.
//!
//! The kharch page of the dashboard (PRD FR-9.5) is this view read for one period, so the page and
//! the view can never differ.
//!
//! Speed (NFR-SPD-24, Code Standards B18, plan 77 robustness finding 4): the entries behind a
//! partner's kharch are found from his drawings account, not from the days. The partners are read
//! first (at most a few), then the lines of each drawings account through the covering index
//! `idx_journal_lines_account_amounts`, then each entry by its key. Ten years hold a few hundred such
//! lines among 3.5 million journal lines; the planner's own order started at the entries by date and
//! read every line of every entry of the period. `CROSS JOIN` fixes the order in SQLite. The
//! conditions and the figures are unchanged, and the totals still come from `taken_between`.
//!
//! Example (after Events 1 to 5, October 2026): الف 2,000, ب 0, ج 0, total 2,000.

use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_db::partners::{PartnerRecord, taken_between};
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::filters::Typed;
use super::ids;
use super::model::{
    AnalyticsKey, AnalyticsLinesInput, AnalyticsLinesPage, AnalyticsPartnerMoney, AnalyticsRow,
};
use super::table::{Table, col, count_of, row};
use crate::error::ReportsError;
use crate::reports::common::{add, paisa};
use crate::reports::{Cell, Column, ColumnKind, Names, PageSpec, Row, RowKind, RowSource};

/// The first and the last day the period may have, when it is open.
const FIRST_DAY: &str = "0000-01-01";
const LAST_DAY: &str = "9999-12-31";

/// Builds the partner part of view M.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does not
/// fit.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    let from = typed.from.as_deref().unwrap_or(FIRST_DAY);
    let to = typed.to.as_deref().unwrap_or(LAST_DAY);
    let partners: Vec<PartnerRecord> = mandi_db::partners::list(conn)?
        .into_iter()
        .filter(|p| typed.partner_id.is_none_or(|id| id == p.id))
        .collect();
    let drawings: Vec<i64> = partners.iter().map(|p| p.drawings_account_id).collect();
    let taken = taken_between(conn, &drawings, from, to)?;

    let mut entries: Vec<(PartnerRecord, Paisa)> = Vec::new();
    for partner in partners {
        let own = match taken.get(&partner.drawings_account_id) {
            Some(t) => paisa(t.debits)?.checked_sub(paisa(t.reversed_credits)?)?,
            None => Paisa::ZERO,
        };
        if partner.is_active || own != Paisa::ZERO {
            entries.push((partner, own));
        }
    }

    // The own kharch opens his entries (FR-18.2).
    let mut own_column = col("own", ids::COL_OWN_KHARCH, ColumnKind::Money);
    own_column.drill = Some(AnalyticsKey {
        money: Some(AnalyticsPartnerMoney::Taken),
        ..AnalyticsKey::default()
    });
    let columns = vec![
        col("partner", ids::COL_PARTNER, ColumnKind::Names),
        own_column,
    ];
    let mut grand = Paisa::ZERO;
    let mut all_rows: Vec<AnalyticsRow> = Vec::with_capacity(entries.len());
    for (partner, own) in &entries {
        add(&mut grand, *own)?;
        all_rows.push(row(
            vec![
                Cell::Names {
                    names: Names {
                        ur: partner.name_ur.clone(),
                        en: partner.name_en.clone(),
                    },
                },
                Cell::Money { paisa: *own },
            ],
            AnalyticsKey {
                partner_id: Some(partner.id),
                ..AnalyticsKey::default()
            },
        ));
    }
    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals: vec![Cell::Empty, Cell::Money { paisa: grand }],
    })
}

/// The columns of the entries behind a number of view M (the shop part and the partner part): the
/// date, the voucher, the head or the partner, the reason and the amount.
pub(crate) fn lines_columns() -> Vec<Column> {
    let column = |key: &str, title: &str, kind: ColumnKind| Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    };
    vec![
        column("date", ids::COL_DATE, ColumnKind::Date),
        column("doc", ids::COL_DOC, ColumnKind::Doc),
        column("name", ids::COL_NAME, ColumnKind::Names),
        column("reason", ids::COL_REASON, ColumnKind::Text),
        column("amount", ids::COL_EXPENSE, ColumnKind::Money),
    ]
}

/// The text of a reason cell: the typed reason, else the narration, else empty.
pub(crate) fn reason_cell(reason: Option<String>) -> Cell {
    match reason
        .map(|r| r.trim().to_owned())
        .filter(|r| !r.is_empty())
    {
        Some(text) => Cell::Text { text },
        None => Cell::Empty,
    }
}

/// The select, tables and narrowing of the takes behind a partner's own kharch: the debits of
/// ordinary entries, and the credits of reversals as negatives (a take that was cancelled or edited),
/// as `taken_between` sums them. The partner and the dates are bound values.
const TAKEN_SELECT: &str = "e.entry_date, e.posting_type, e.source_type, e.source_id, v.number,
    p.name_ur, p.name_en, COALESCE(NULLIF(v.reason, ''), v.narration),
    CASE WHEN e.reverses_entry_id IS NULL THEN l.dr_paisa ELSE -l.cr_paisa END, e.id";
const TAKEN_FROM: &str = "FROM partners p
    CROSS JOIN journal_lines l ON l.account_id = p.drawings_account_id
    CROSS JOIN journal_entries e ON e.id = l.entry_id
    LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id";
const TAKEN_WHERE: &str = "e.status = 'posted' AND (?1 IS NULL OR p.id = ?1)
    AND (?2 IS NULL OR e.entry_date >= ?2) AND (?3 IS NULL OR e.entry_date <= ?3)
    AND ((e.reverses_entry_id IS NULL AND l.dr_paisa > 0)
         OR (e.reverses_entry_id IS NOT NULL AND l.cr_paisa > 0))";

/// The query of the entries behind a partner's own kharch, in day order. Every fragment is a
/// constant of this crate; the values are bound (Code Standards B14).
pub(crate) fn taken_sql() -> String {
    format!("SELECT {TAKEN_SELECT} {TAKEN_FROM} WHERE {TAKEN_WHERE} ORDER BY 1, 10")
}

/// The entries behind a partner's own kharch (or every partner's), inside the period of the run and
/// of the key. A cancelled or edited take and its reversal both show, the reversal as a negative
/// amount, so the entries add up to the cell. A partner has few such entries, so they are read
/// together and the page is cut from them (the total and the page cannot differ).
///
/// # Errors
/// `ERR_OUT_OF_RANGE` (`row_key.money`) for a kind of money that is not the own kharch; a database
/// or money error.
pub(crate) fn lines(
    conn: &Connection,
    typed: &Typed,
    key: &AnalyticsKey,
    money: AnalyticsPartnerMoney,
    input: &AnalyticsLinesInput,
    spec: PageSpec,
) -> Result<AnalyticsLinesPage, ReportsError> {
    let AnalyticsPartnerMoney::Taken = money else {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "row_key.money",
        )]));
    };
    // `YYYY-MM-DD` sorts as a date: the later start and the earlier end.
    let from = [typed.from.as_ref(), key.from.as_ref()]
        .into_iter()
        .flatten()
        .max()
        .cloned();
    let to = [typed.to.as_ref(), key.to.as_ref()]
        .into_iter()
        .flatten()
        .min()
        .cloned();
    let partner = key.partner_id.or(typed.partner_id);
    let args: Vec<Value> = vec![
        partner.map_or(Value::Null, Value::Integer),
        from.map_or(Value::Null, Value::Text),
        to.map_or(Value::Null, Value::Text),
    ];
    let mut statement = conn.prepare(&taken_sql())?;
    let all = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?,
                Names {
                    ur: r.get(5)?,
                    en: r.get(6)?,
                },
                r.get::<_, Option<String>>(7)?,
                r.get::<_, i64>(8)?,
                r.get::<_, i64>(9)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut sum = Paisa::ZERO;
    let mut rows: Vec<Row> = Vec::with_capacity(all.len());
    for (date, posting, source_type, source_id, number, names, reason, amount, entry_id) in all {
        let amount = paisa(amount)?;
        add(&mut sum, amount)?;
        rows.push(Row {
            cells: vec![
                Cell::Date { iso: date },
                Cell::Doc {
                    type_id: crate::reports::ids::doc_id(&posting),
                    number,
                },
                Cell::Names { names },
                reason_cell(reason),
                Cell::Money { paisa: amount },
            ],
            source: Some(RowSource::of_stored(
                &source_type,
                source_id,
                entry_id,
                None,
                None,
            )),
            kind: RowKind::Normal,
        });
    }
    Ok(AnalyticsLinesPage {
        view: input.view,
        title_id: ids::LINES_TITLE.to_owned(),
        filters: typed.resolved.clone(),
        columns: lines_columns(),
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: sum },
        ],
    })
}
