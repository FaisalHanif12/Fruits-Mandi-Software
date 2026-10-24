//! The lines behind a number of view F (PRD FR-18.2, FR-17.5): the share rows of a partner, one
//! line for each posting that gave him a share, with the base it was worked on, his share and his
//! amount. A share of a shop expense is negative; a reversal row is the opposite of the row it
//! takes back and keeps its own date, so the lines add up to exactly the profit share that was
//! opened.
//!
//! Example (the worked day, الف, 10-10-26): بکری 1 income 3,948 at 40% = 1,580; بکری 2 income 1,540
//! at 40% = 616; the electricity −3,000 at 40% = −1,200. The total row is 996.

use mandi_db::rusqlite::{Connection, params_from_iter, types::Value};

use super::filters::Typed;
use super::ids;
use super::model::{AnalyticsKey, AnalyticsLinesInput, AnalyticsLinesPage};
use crate::error::ReportsError;
use crate::reports::common::paisa;
use crate::reports::{Cell, Column, ColumnKind, PageSpec, Row, RowKind, RowSource, SourceKind};

fn column(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

fn columns() -> Vec<Column> {
    vec![
        column("date", ids::COL_DATE, ColumnKind::Date),
        column("doc", ids::COL_DOC, ColumnKind::Doc),
        column("kind", ids::COL_SHARE_KIND, ColumnKind::Label),
        column("base", ids::COL_SHARE_BASE, ColumnKind::Money),
        column("share", ids::COL_SHARE, ColumnKind::Percent),
        column("amount", ids::COL_SHARE_AMOUNT, ColumnKind::Money),
    ]
}

/// Where a row opens, by the kind of the journal entry's source.
fn source_of(source_type: &str, source_id: i64) -> RowSource {
    let kind = match source_type {
        "bikri" => SourceKind::Bikri,
        "voucher" => SourceKind::Voucher,
        "account" => SourceKind::Account,
        "month_close" => SourceKind::MonthClose,
        "purchase" => SourceKind::Purchase,
        "venture" => SourceKind::Venture,
        "fruit" => SourceKind::Fruit,
        _ => SourceKind::Entry,
    };
    RowSource {
        kind,
        id: source_id,
    }
}

/// The share rows of the partner in the key (or the filter), inside the period of the run and of
/// the key.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    key: &AnalyticsKey,
    input: &AnalyticsLinesInput,
    spec: PageSpec,
) -> Result<AnalyticsLinesPage, ReportsError> {
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
    let filter = "(?1 IS NULL OR s.partner_id = ?1) AND (?2 IS NULL OR s.entry_date >= ?2)
                  AND (?3 IS NULL OR s.entry_date <= ?3)";
    let args = vec![
        partner.map_or(Value::Null, Value::Integer),
        from.map_or(Value::Null, Value::Text),
        to.map_or(Value::Null, Value::Text),
    ];

    // The count and the sum over the whole filter (INTEGER columns only).
    let (total_count, total_amount): (i64, i64) = conn.query_row(
        &format!(
            "SELECT COUNT(*), COALESCE(SUM(s.amount_paisa), 0)
             FROM partner_share_rows s WHERE {filter}"
        ),
        params_from_iter(args.iter()),
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let mut page_args = args;
    page_args.push(Value::Integer(spec.sql_limit()));
    page_args.push(Value::Integer(spec.offset));
    let mut statement = conn.prepare(&format!(
        "SELECT s.entry_date, e.posting_type, e.source_type, e.source_id,
                COALESCE(b.number, v.number), s.kind, s.base_paisa, s.share_bps, s.amount_paisa
         FROM partner_share_rows s
         JOIN journal_entries e ON e.id = s.journal_entry_id
         LEFT JOIN bikri_entries b ON e.source_type = 'bikri' AND b.id = e.source_id
         LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id
         WHERE {filter} ORDER BY s.entry_date, s.id LIMIT ?4 OFFSET ?5"
    ))?;
    let raws = statement
        .query_map(params_from_iter(page_args.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, i64>(8)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut rows = Vec::with_capacity(raws.len());
    for (date, posting_type, source_type, source_id, number, kind, base, bps, amount) in raws {
        rows.push(Row {
            cells: vec![
                Cell::Date { iso: date },
                Cell::Doc {
                    type_id: crate::reports::ids::doc_id(&posting_type),
                    number,
                },
                Cell::Label {
                    id: if kind == "income" {
                        ids::COL_INCOME_SHARE.to_owned()
                    } else {
                        ids::COL_EXPENSE_SHARE.to_owned()
                    },
                },
                Cell::Money {
                    paisa: paisa(base)?,
                },
                Cell::Percent { hundredths: bps },
                Cell::Money {
                    paisa: paisa(amount)?,
                },
            ],
            source: Some(source_of(&source_type, source_id)),
            kind: RowKind::Normal,
        });
    }
    Ok(AnalyticsLinesPage {
        view: input.view,
        title_id: ids::LINES_TITLE.to_owned(),
        filters: typed.resolved.clone(),
        columns: columns(),
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: paisa(total_amount)?,
            },
        ],
    })
}
